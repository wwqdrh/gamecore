#pragma once

#include <atomic>
#include <functional>
#include <map>
#include <mutex>
#include <optional>
#include <shared_mutex>
#include <stdexcept>
#include <variant>
#include <vector>

#include "lock.h"
#include "types.h"

namespace gameconsole {

// Forward declaration
class Collection;
using CollectionPtr = std::shared_ptr<Collection>;

class Collection {
private:
  // 保护map的读写锁
  mutable ReentrantRWLock rwlock;

protected:
  std::map<Variant, Variant> _collection;
  // 使用原子变量保护计数器
  std::atomic<int> _iterationCurrent;
  std::atomic<int> _maxId;

public:
  Collection() : _iterationCurrent(-1), _maxId(0) {}
  Collection(const Variant &val) : Collection() { _collection = to_dict(val); }
  // 拷贝构造函数
  Collection(const Collection &other)
      : _iterationCurrent(other._iterationCurrent.load()),
        _maxId(other._maxId.load()) {
    auto guard_other = other.rwlock.shared_lock();
    _collection = other._collection;
    _iterationCurrent.store(other._iterationCurrent.load());
    _maxId.store(other._maxId.load());
  }

  // 拷贝赋值运算符
  Collection &operator=(const Collection &other) {
    if (this != &other) {
      // 先获取other的数据
      int itCurrent = other._iterationCurrent.load();
      int maxId = other._maxId.load();

      {
        // 使用嵌套作用域来尽快释放锁
        auto guard_other = other.rwlock.shared_lock();
        auto guard_this = rwlock.unique_lock();

        _collection = other._collection;
        _iterationCurrent.store(itCurrent);
        _maxId.store(maxId);
      }
    }
    return *this;
  }

public:
  // 设置键值对
  void set_value(const Variant &key, const Variant &value) {
    auto guard = rwlock.unique_lock();

    _collection[key] = value;
  }

  // 添加值(使用size作为key)
  void add(const Variant &value) {
    auto guard = rwlock.unique_lock();

    // 计算新的maxId
    int current_max = _maxId.load();
    int collection_size = static_cast<int>(_collection.size());
    int new_max = std::max(current_max, collection_size);

    // 更新maxId并使用其值作为key
    _maxId.store(new_max + 1);
    _collection[new_max] = value;
  }

  // 移除指定key的元素
  void remove(const Variant &key) {
    auto guard = rwlock.unique_lock();

    _collection.erase(key);
  }

  // 移除指定值的元素
  bool remove_element(const Variant &element) {
    auto guard = rwlock.unique_lock();

    for (auto it = _collection.begin(); it != _collection.end(); ++it) {
      if (it->second == element) {
        _collection.erase(it);
        return true;
      }
    }
    return false;
  }

  // 通过索引移除元素
  void remove_by_index(int index) {
    auto guard = rwlock.unique_lock();

    if (index >= 0 && index < static_cast<int>(_collection.size())) {
      auto it = std::next(_collection.begin(), index);
      _collection.erase(it);
    }
  }

  // 检查是否包含指定的key
  bool contains_key(const Variant &key) const {
    auto guard = rwlock.shared_lock();

    return _collection.find(key) != _collection.end();
  }

  // 检查是否包含指定的值
  bool contains(const Variant &element) const {
    auto guard = rwlock.shared_lock();

    for (const auto &pair : _collection) {
      if (pair.second == element) {
        return true;
      }
    }
    return false;
  }

  // 获取元素的索引
  std::optional<Variant> index_of(const Variant &element) const {
    auto guard = rwlock.shared_lock();

    for (const auto &pair : _collection) {
      if (pair.second == element) {
        return pair.first;
      }
    }
    return std::nullopt;
  }

  // 获取指定key的值
  std::optional<Variant> get_value(const Variant &key) const {
    auto guard = rwlock.shared_lock();

    auto it = _collection.find(key);
    if (it != _collection.end()) {
      return it->second;
    }
    return std::nullopt;
  }

  // 通过索引获取值
  std::optional<Variant> get_by_index(int index) const {
    auto guard = rwlock.shared_lock();

    if (index >= 0 && index < static_cast<int>(_collection.size())) {
      auto it = std::next(_collection.begin(), index);
      return it->second;
    }
    return std::nullopt;
  }

  // 获取所有键
  std::vector<Variant> get_keys() const {
    auto guard = rwlock.shared_lock();

    std::vector<Variant> keys;
    for (const auto &pair : _collection) {
      keys.push_back(pair.first);
    }
    return keys;
  }

  // 获取所有值
  std::vector<Variant> get_values() const {
    auto guard = rwlock.shared_lock();

    std::vector<Variant> values;
    for (const auto &pair : _collection) {
      values.push_back(pair.second);
    }
    return values;
  }

  // 检查集合是否为空
  bool is_empty() const {
    auto guard = rwlock.shared_lock();

    return _collection.empty();
  }

  // 清空集合
  void clear() {
    auto guard = rwlock.shared_lock();

    _collection.clear();
  }

  // 获取第一个元素
  std::optional<Variant> first() {
    auto guard = rwlock.shared_lock();

    if (!is_empty()) {
      _iterationCurrent.store(0);
      return get_by_index(_iterationCurrent);
    }
    return std::nullopt;
  }

  // 获取最后一个元素
  std::optional<Variant> last() {
    auto guard = rwlock.shared_lock();

    if (!is_empty()) {
      _iterationCurrent.store(_collection.size() - 1);
      return get_by_index(_iterationCurrent);
    }
    return std::nullopt;
  }

  // 设置当前迭代位置
  void seek(int id) { _iterationCurrent.store(id); }

  // 检查是否还有下一个元素
  bool has_next() const {
    auto guard = rwlock.shared_lock();

    return _iterationCurrent.load() < static_cast<int>(_collection.size()) - 1;
  }

  // 获取下一个元素
  std::optional<Variant> next() {
    auto guard = rwlock.unique_lock();

    if (!is_empty() && has_next()) {
      _iterationCurrent.fetch_add(1);
      return get_by_index(_iterationCurrent);
    }
    return std::nullopt;
  }

  // 获取当前元素
  std::optional<Variant> current() const {
    auto guard = rwlock.shared_lock();

    if (!is_empty() && _iterationCurrent.load() >= 0) {
      return get_by_index(_iterationCurrent);
    }
    return std::nullopt;
  }

  // 获取前一个元素
  std::optional<Variant> previous() {
    auto guard = rwlock.unique_lock();

    if (!is_empty() && _iterationCurrent.load() > 0) {
      _iterationCurrent.fetch_add(-1);
      return get_by_index(_iterationCurrent.load());
    }
    return std::nullopt;
  }

  // 获取集合大小
  size_t size() const {
    auto guard = rwlock.shared_lock();

    return _collection.size();
  }

  CollectionPtr fill(const Variant &value = std::monostate{},
                     int startIndex = 0,
                     const Variant &length = std::monostate{}) {
    auto guard = rwlock.unique_lock();

    int fill_length;

    if (std::holds_alternative<std::monostate>(length)) {
      fill_length = static_cast<int>(_collection.size()) - startIndex;
    } else {
      fill_length = std::get<int>(length);
    }

    for (int i = startIndex; i < startIndex + fill_length &&
                             i < static_cast<int>(_collection.size());
         i++) {
      _collection[i] = value;
    }

    return std::make_shared<Collection>(*this);
  }

  // 转换为字典的辅助函数
  std::map<Variant, Variant> to_dict(const Variant &value) const {
    auto guard = rwlock.shared_lock();

    std::map<Variant, Variant> d;

    if (auto map_ptr = std::get_if<std::shared_ptr<VariantMap>>(&value)) {
      return (*map_ptr)->values;
    } else if (!std::holds_alternative<std::monostate>(value)) {
      if (auto arr_ptr = std::get_if<std::shared_ptr<VariantArray>>(&value)) {
        const auto &arr = (*arr_ptr)->values;
        for (size_t i = 0; i < arr.size(); i++) {
          d[static_cast<int>(i)] = arr[i];
        }
      } else {
        d[0] = value;
      }
    }
    return d;
  }

  // filter 函数实现
  std::shared_ptr<Collection>
  filter(std::function<bool(const Variant &, const Variant &, int,
                            const Collection &)>
             callback = nullptr) const {
    auto guard = rwlock.unique_lock();

    auto new_collection = std::make_shared<Collection>();
    new_collection->_collection = _collection;

    int i = 0;
    if (callback) {
      while (i < static_cast<int>(new_collection->_collection.size())) {
        auto keys = new_collection->get_keys();
        Variant key = keys[i];
        Variant value =
            new_collection->get_value(key).value_or(std::monostate{});

        bool result = callback(key, value, i, *new_collection);

        if (!result) {
          new_collection->remove_by_index(i);
        } else {
          i++;
        }
      }
    } else {
      while (i < static_cast<int>(new_collection->_collection.size())) {
        auto value_opt = new_collection->get_by_index(i);
        if (!value_opt.has_value()) {
          new_collection->remove_by_index(i);
          continue;
        }

        const Variant &value = value_opt.value();
        bool should_remove = false;

        if (std::holds_alternative<std::monostate>(value)) {
          should_remove = true;
        } else if (std::holds_alternative<std::string>(value)) {
          should_remove = std::get<std::string>(value).empty();
        } else if (auto arr_ptr =
                       std::get_if<std::shared_ptr<VariantArray>>(&value)) {
          should_remove = (*arr_ptr)->values.empty();
        } else if (auto map_ptr =
                       std::get_if<std::shared_ptr<VariantMap>>(&value)) {
          should_remove = (*map_ptr)->values.empty();
        }

        if (should_remove) {
          new_collection->remove_by_index(i);
        } else {
          i++;
        }
      }
    }

    return new_collection;
  }

  // 辅助函数：创建数组
  static Variant make_array(const std::vector<Variant> &values) {
    auto arr = std::make_shared<VariantArray>();
    arr->values = values;
    return arr;
  }

  // 辅助函数：创建字典
  static Variant make_map(const std::map<Variant, Variant> &values) {
    auto map = std::make_shared<VariantMap>();
    map->values = values;
    return map;
  }
};

} // namespace gameconsole