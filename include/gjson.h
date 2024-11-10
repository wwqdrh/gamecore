#pragma once

#include <algorithm>
#include <any>
#include <map>
#include <memory>
#include <mutex>
#include <random>
#include <shared_mutex>
#include <sstream>
#include <stdexcept>
#include <unordered_set>
#include <variant>
#include <vector>

#include "cross.h"
#include "lock.h"
#include "rapidjson/document.h"
#include "rapidjson/stringbuffer.h"
#include "rapidjson/writer.h"
#include "store.h"
#include "traits.h"

using namespace rapidjson;

namespace gamedb {

class GJson {
public:
  // 定义回调函数类型
  using CallbackFunc = std::function<void(const std::string &path,
                                          const rapidjson::Value *value)>;

private:
  mutable Document raw_data;
  bool imported_ = false;
  std::shared_ptr<FileStore> store_;
  ConditionParser
      condition_; // 用于#conditon命令(在condition命令中传入json字符串用于检查)，检查当前json节点下#condition字段的条件

private:
  // mutable std::recursive_mutex mutex_;
  // mutable std::shared_mutex rw_mtx;
  mutable ReentrantRWLock rwlock;

  // 前缀树节点
  struct TrieNode {
    std::unordered_map<std::string, std::unique_ptr<TrieNode>> children;
    std::vector<CallbackFunc> callbacks;
    bool is_endpoint = false;
  };

  std::unique_ptr<TrieNode> callback_trie_;

public:
  ~GJson() = default;
  static Value toValue(const std::string &data) {
    rapidjson::Document doc;
    doc.Parse(data.c_str());
    if (doc.HasParseError()) {
      return Value();
    }
    rapidjson::Value val;
    val.CopyFrom(doc, doc.GetAllocator());
    return val;
  }

public:
  GJson() : callback_trie_(std::make_unique<TrieNode>()) {
    raw_data.Parse("{}");
  };
  explicit GJson(std::shared_ptr<FileStore> store) : GJson() {
    store_ = store;
    // 不要在构造函数这里直接初始化，要分成两段，因为传递的读取函数可能会用到这个core部分
    // 导致还未初始化完成，报错
    // load_or_store(store);
  }
  explicit GJson(const std::string &data) : GJson() { load_or_store(data); }
  rapidjson::Document::AllocatorType get_alloctor() {
    return raw_data.GetAllocator();
  }
  bool check_condition(Value &val, const std::string &data) {
    return checkCondition_(val, data) != nullptr;
  }
  void load_or_store(const std::string &data) {
    if (data.empty()) {
      return;
    }

    auto write = rwlock.unique_lock();
    // 判断解析是否出错
    raw_data.Parse(data.c_str());

    if (imported_) {
      trigger_all_callbacks(); // 通知注册者
    }
    imported_ = true;
  }
  bool HasParseError() { return raw_data.HasParseError(); }
  std::vector<uint8_t> encrypt(const std::string &data) const {
    if (store_ == nullptr) {
      return {};
    }
    return store_->encrypt(data);
  }
  void load_by_store() {
    if (!store_) {
      return;
    }
    auto write = rwlock.unique_lock();
    // std::unique_lock<std::shared_mutex> lock(rw_mtx);
    std::string data = store_->loadData();
    if (data.empty()) {
      store_->saveData("{}");
    } else {
      raw_data.Parse(data.c_str());
    }
    if (imported_) {
      trigger_all_callbacks(); // 通知注册者
    }
    imported_ = true;
  }
  // 注册通知
  // 订阅路径变化
  void subscribe(const std::string &path, CallbackFunc &&callback) {
    // 直接使用 std::forward 转发
    subscribeImpl(path, std::forward<CallbackFunc>(callback));
  }

  // 左值引用版本
  void subscribe(const std::string &path, const CallbackFunc &callback) {
    // 复制回调函数
    subscribeImpl(path, callback);
  }

  // 是否存在某个key
  bool has(const std::string &path) const {
    auto lock = rwlock.shared_lock();

    std::vector<std::string> parts = split(path, ';');
    Value *current = const_cast<Value *>(static_cast<const Value *>(&raw_data));
    if (current == nullptr) {
      return false;
    }

    for (const auto &part : parts) {
      if (current == nullptr) {
        return false;
      }
      if (part.empty())
        continue;

      current = traverse(*current, part);
    }

    return current != nullptr;
  }

  // 取消订阅
  void unsubscribe(const std::string &path) {
    auto guard = rwlock.unique_lock();

    std::vector<std::string> parts = split(path, ';');
    TrieNode *current = callback_trie_.get();
    if (current == nullptr) {
      return;
    }

    for (const auto &part : parts) {
      if (current->children.count(part) == 0) {
        return;
      }
      current = current->children[part].get();
    }

    current->callbacks.clear();
    current->is_endpoint = false;
  }
  std::string query(const std::string &field) const; // 返回的是json字符串
  // 查询指定字段的值并返回特定类型
  Value *query_value(const std::string &field) const;
  Value query_value_dynamic(
      const std::string &field) const; // 不是json数中的，是构造的
  template <typename T> T queryT(const std::string &field) const {
    Value *current = query_value(field);
    if (current == nullptr) {
      return T();
    }
    return convert<T>(*current);
  }
  std::vector<std::string> keys(const std::string &field) const;
  std::vector<std::string> values(const std::string &field) const;

  template <typename T>
  bool updateT(const std::string &field, const std::string &action, T val) {
    Document doc;
    Document::AllocatorType allo = doc.GetAllocator();

    Value v = toValue(val, allo);
    return update(field, action, v);
  }
  bool update(const std::string &field, const std::string &action,
              const std::string &val);
  bool update(const std::string &field, const std::string &action, Value &val) {
    auto write = rwlock.unique_lock();

    bool res = update_(field, action, val);
    if (res) {
      // 更新回调
      trigger_callbacks(field);

      if (store_ != nullptr) {
        store_->saveData(query(""));
      }
    }
    return res;
  }

private:
  void trigger_all_callbacks(); // 用于通知所有注册了的回调
  void trigger_callbacks(const std::string &field);
  bool check_object_(Value &curr, const std::string &key, const std::string &op,
                     const std::string &value) const;
  // 获取所有需要触发的回调
  void collect_affected_callbacks(
      TrieNode *node, const std::string &base_path,
      std::vector<std::pair<std::string, CallbackFunc>> &callbacks);
  bool update_(const std::string &field, const std::string &action, Value &val);
  bool safeReplaceValue(rapidjson::Value *current,
                        const rapidjson::Value &newVal);
  std::vector<std::string> split(const std::string &s, char delimiter) const;

  Value *traverse(Value &current, const std::string &key) const;

  Value getRandomElements(Value &current, size_t count) const;
  Value *checkCondition_(Value &current, const std::string &data) const;
  Value *getCompareElements(Value &current, const std::string &key,
                            const std::string &op, const std::string &value,
                            bool rindex = false) const;

public:
  // 类型转换，任意类型转rapidjson::Value
  // 辅助函数：将 JSON 值转换为基本类型
  // 主转换函数模板
  // T -> rapidjson::Value
  template <typename T>
  static rapidjson::Value
  toValue(const T &data, rapidjson::Document::AllocatorType &allocator) {
    if constexpr (std::is_same_v<T, std::any>) {
      return handleAnyType(data, allocator);
    } else if constexpr (std::is_same_v<T, std::string>) {
      return toValue(data.c_str(), allocator);
    } else if constexpr (std::is_same_v<T, const char *> ||
                         std::is_same_v<T, char *>) {
      rapidjson::Value v;
      v.SetString(data, allocator);
      return v;
    } else if constexpr (std::is_integral_v<T>) {
      if constexpr (std::is_signed_v<T>) {
        rapidjson::Value v;
        v.SetInt64(data);
        return v;
      } else {
        rapidjson::Value v;
        v.SetUint64(data);
        return v;
      }
    } else if constexpr (std::is_floating_point_v<T>) {
      rapidjson::Value v;
      v.SetDouble(data);
      return v;
    } else if constexpr (std::is_same_v<T, bool>) {
      rapidjson::Value v;
      v.SetBool(data);
      return v;
    }
    // variant类型
    // variant 类型的转换
    else if constexpr (is_variant<T>::value) {
      return std::visit(
          [&allocator](const auto &value) { return toValue(value, allocator); },
          data);
    }
    // vector 类型的转换
    else if constexpr (is_vector<T>::value) {
      rapidjson::Value arr(rapidjson::kArrayType);
      for (const auto &item : data) {
        arr.PushBack(toValue(item, allocator), allocator);
      }
      return arr;
    }
    // map 类型的转换
    else if constexpr (is_map<T>::value || is_unordered_map<T>::value) {
      rapidjson::Value obj(rapidjson::kObjectType);
      for (const auto &[key, value] : data) {
        auto keyStr = toString(key);
        obj.AddMember(rapidjson::Value(keyStr.c_str(), allocator).Move(),
                      toValue(value, allocator), allocator);
      }
      return obj;
    }
    // 自定义类型的转换 - 需要实现 toJson 方法
    else if constexpr (has_to_json<T>::value) {
      return data.toJson(allocator);
    }
    //  else {
    //   static_assert(always_false<T>::value,
    //                 "Unsupported type for JSON conversion");
    // }
    rapidjson::Value v;
    v.SetBool(false);
    return v;
  }
  // rapidjson::Value -> T
  template <typename T> static T convert(const Value &value) {
    if (value.IsNull())
      return T{};
    return convert_impl<T>(value);
  }

  // 辅助函数来处理 std::any 类型
  static rapidjson::Value
  handleAnyType(const std::any &data,
                rapidjson::Document::AllocatorType &allocator) {
    // 处理基本类型
    if (data.type() == typeid(std::string)) {
      return toValue(std::any_cast<std::string>(data), allocator);
    }
    if (data.type() == typeid(const char *)) {
      return toValue(std::any_cast<const char *>(data), allocator);
    }
    if (data.type() == typeid(int)) {
      return toValue(std::any_cast<int>(data), allocator);
    }
    if (data.type() == typeid(int64_t)) {
      return toValue(std::any_cast<int64_t>(data), allocator);
    }
    if (data.type() == typeid(unsigned int)) {
      return toValue(std::any_cast<unsigned int>(data), allocator);
    }
    if (data.type() == typeid(uint64_t)) {
      return toValue(std::any_cast<uint64_t>(data), allocator);
    }
    if (data.type() == typeid(double)) {
      return toValue(std::any_cast<double>(data), allocator);
    }
    if (data.type() == typeid(float)) {
      return toValue(std::any_cast<float>(data), allocator);
    }
    if (data.type() == typeid(bool)) {
      return toValue(std::any_cast<bool>(data), allocator);
    }

    // 处理容器类型
    if (data.type() == typeid(std::vector<std::any>)) {
      const auto &vec = std::any_cast<const std::vector<std::any> &>(data);
      rapidjson::Value arr(rapidjson::kArrayType);
      for (const auto &item : vec) {
        arr.PushBack(handleAnyType(item, allocator), allocator);
      }
      return arr;
    }

    if (data.type() == typeid(std::map<std::string, std::any>)) {
      const auto &map =
          std::any_cast<const std::map<std::string, std::any> &>(data);
      rapidjson::Value obj(rapidjson::kObjectType);
      for (const auto &[key, value] : map) {
        obj.AddMember(rapidjson::Value(key.c_str(), allocator).Move(),
                      handleAnyType(value, allocator), allocator);
      }
      return obj;
    }

    return toValue("", allocator);
  }

private:
  void subscribeImpl(const std::string &path, const CallbackFunc &callback) {
    std::unique_lock<ReentrantRWLock> lock(rwlock);

    std::vector<std::string> parts = split(path, ';');
    TrieNode *current = callback_trie_.get();
    if (current == nullptr) {
      return;
    }

    for (const auto &part : parts) {
      if (current->children.count(part) == 0) {
        current->children[part] = std::make_unique<TrieNode>();
      }
      current = current->children[part].get();
    }

    current->is_endpoint = true;
    current->callbacks.push_back(callback);
    Value *val = query_value(path);
    callback(path, val);
  }

private:
  template <typename T> static T convert_impl(const Value &value) {
    if constexpr (has_from_json<T>::value) {
      return T::fromJson(value);
    } else if constexpr (std::is_arithmetic_v<T>) {
      return convert_arithmetic<T>(value);
    } else if constexpr (std::is_same_v<T, std::string>) {
      return convert_string(value);
    } else if constexpr (is_vector<T>::value) {
      return convert_vector<T>(value);
    } else if constexpr (is_map<T>::value) {
      return convert_map<T>(value);
    } else {
      // 不支持的类型将返回默认值
      return T{};
    }
  }
  // 处理算术类型
  template <typename T>
  static typename std::enable_if_t<std::is_arithmetic_v<T>, T>
  convert_arithmetic(const Value &value) {
    if (value.IsInt())
      return static_cast<T>(value.GetInt());
    if (value.IsInt64())
      return static_cast<T>(value.GetInt64());
    if (value.IsDouble())
      return static_cast<T>(value.GetDouble());
    if (value.IsUint())
      return static_cast<T>(value.GetUint());
    if (value.IsUint64())
      return static_cast<T>(value.GetUint64());
    return T{};
  }

  // 处理字符串
  static std::string convert_string(const Value &value) {
    if (value.IsString())
      return value.GetString();

    // 非字符串值转换为 JSON 字符串
    StringBuffer buffer;
    Writer<StringBuffer> writer(buffer);
    value.Accept(writer);
    return buffer.GetString();
  }

  // 处理 vector
  template <typename VecType>
  static VecType convert_vector(const Value &value) {
    using T = typename VecType::value_type;
    VecType result;

    if (!value.IsArray())
      return result;

    result.reserve(value.Size());
    for (auto &item : value.GetArray()) {
      result.push_back(convert_impl<T>(item));
    }
    return result;
  }

  // 处理 map
  template <typename MapType> static MapType convert_map(const Value &value) {
    using ValueType = typename MapType::mapped_type;
    MapType result;

    if (!value.IsObject())
      return result;

    for (auto &m : value.GetObject()) {
      result[m.name.GetString()] = convert_impl<ValueType>(m.value);
    }
    return result;
  }
};

} // namespace gamedb