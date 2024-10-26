#pragma once

#include <algorithm>
#include <map>
#include <memory>
#include <random>
#include <sstream>
#include <stdexcept>
#include <vector>

#include "rapidjson/document.h"
#include "rapidjson/stringbuffer.h"
#include "rapidjson/writer.h"
#include "store.h"
#include "traits.h"

using namespace rapidjson;

namespace gamedb {
class GJson {
private:
  Document raw_data;
  std::shared_ptr<FileStore> store_;

public:
  GJson() { raw_data.Parse("{}"); };
  explicit GJson(std::shared_ptr<FileStore> store) : GJson() {
    load_or_store(store);
  }
  explicit GJson(const std::string &data) { raw_data.Parse(data.c_str()); }
  rapidjson::Document::AllocatorType get_alloctor() {
    return raw_data.GetAllocator();
  }
  void load_or_store(std::shared_ptr<FileStore> store) {
    store_ = store;
    if (store_ == nullptr) {
      return;
    }
    std::string data = store_->loadData();
    if (data.empty()) {
      store_->saveData("{}");
    } else {
      raw_data.Parse(data.c_str());
    }
  }
  void parse_file(const std::string &filename);
  Value parse(const std::string &data);
  std::string query(const std::string &field); // 返回的是json字符串
  // 查询指定字段的值并返回特定类型
  Value *query_value(const std::string &field);
  template <typename T> T queryT(const std::string &field) {
    Value *current = query_value(field);
    if (current == nullptr) {
      return T();
    }
    return convert<T>(*current);
  }
  std::vector<std::string> keys(const std::string &field);
  std::vector<std::string> values(const std::string &field);

  template <typename T>
  bool updateT(const std::string &field, const std::string &action, T val) {
    Document::AllocatorType allo = raw_data.GetAllocator();
    Value v = toValue(val, allo);
    return update(field, action, v);
  }
  bool update(const std::string &field, const std::string &action, Value &val) {
    bool res = update_(field, action, val);
    if (res && store_ != nullptr) {
      store_->saveData(query(""));
    }
    return res;
  }

private:
  bool update_(const std::string &field, const std::string &action, Value &val);
  std::vector<std::string> split(const std::string &s, char delimiter) const;

  Value *traverse(Value &current, const std::string &key);

  Value getRandomElements(Value &current, size_t count);
  Value *getCompareElements(Value &current, const std::string &key,
                            const std::string &op, const std::string &value,
                            bool rindex = false);

public:
  // 类型转换，任意类型转rapidjson::Value
  // 辅助函数：将 JSON 值转换为基本类型
  // 主转换函数模板
  // T -> rapidjson::Value
  template <typename T>
  static rapidjson::Value
  toValue(const T &data, rapidjson::Document::AllocatorType &allocator) {
    if constexpr (std::is_same_v<T, std::string>) {
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
    } else {
      static_assert(always_false<T>::value,
                    "Unsupported type for JSON conversion");
    }
  }
  // rapidjson::Value -> T
  template <typename T> static T convert(const Value &value) {
    if (value.IsNull())
      return T{};
    return convert_impl<T>(value);
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