#pragma once

#include <chrono>
#include <map>
#include <set>

namespace gamedb {
template <typename K, typename V> class TimedOrderedMap {
private:
  // 存储实际的key-value对
  std::map<K, V> data_map;

  // 存储插入时间和对应的key
  std::map<std::chrono::system_clock::time_point, K> time_map;

  // 反向映射：key到插入时间
  std::map<K, std::chrono::system_clock::time_point> key_to_time;

public:
  ~TimedOrderedMap() {}
  // 插入或更新元素
  void insert(const K &key, const V &value) {
    auto now = std::chrono::system_clock::now();

    // 只记录第一次添加的位置
    if (key_to_time.find(key) == key_to_time.end()) {
      time_map[now] = key;
      key_to_time[key] = now;
    }

    // 更新所有映射
    data_map[key] = value;
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
  V get(const K &key) const {
    auto it = data_map.find(key);
    if (it != data_map.end()) {
      return (it->second);
    }
    return nullptr;
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