#pragma once

#include <map>
#include <optional>
#include <set>

namespace gamedb {
template <typename K, typename V> class TimedOrderedMap {
private:
  // 存储实际的key-value对
  std::map<K, V> data_map;

  // 存储插入时间和对应的key
  std::map<int, K> time_map;

  // 反向映射：key到插入时间
  std::map<K, int> key_to_time;

  int start_index_ = 0;

public:
  ~TimedOrderedMap() {}
  // 插入或更新元素
  void insert(const K &key, const V &value) {
    // 如果key已存在，先删除旧的时间记录
    auto it = key_to_time.find(key);
    if (it != key_to_time.end()) {
      time_map.erase(it->second);
      key_to_time.erase(it);
    }

    // 更新所有映射
    data_map[key] = value;
    time_map[start_index_] = key;
    key_to_time[key] = start_index_;
    start_index_++;
  }

  // 删除元素
  void erase(const K &key) {
    if (data_map.find(key) == data_map.end()) {
      return;
    }
    data_map.erase(key);
    auto it = key_to_time.find(key);
    if (it != key_to_time.end()) {
      time_map.erase(it->second);
      key_to_time.erase(it);
    }
  }

  // 获取值
  std::optional<V> get(const K &key) const {
    auto it = data_map.find(key);
    if (it != data_map.end()) {
      return (it->second);
    }
    return std::nullopt;
  }

  // 按插入时间顺序获取所有键
  std::vector<K> getKeysByInsertionOrder() const {
    std::vector<K> keys;
    for (const auto &pair : time_map) {
      keys.push_back(pair.second);
    }
    return keys;
  }

  // 获取大小
  size_t size() const { return data_map.size(); }

  // 检查是否为空
  bool empty() const { return data_map.empty(); }

  // 清空容器
  void clear() {
    data_map.clear();
    time_map.clear();
    key_to_time.clear();
  }
};
} // namespace gamedb