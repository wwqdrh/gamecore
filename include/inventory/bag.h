#pragma once

#include <vector>

#include "inventory/database.h"
#include "inventory/slot.h"

namespace gamedb {
class Bag {
private:
  size_t rows{3};                       // 行数
  size_t cols{3};                       // 列数
  int capacity{-1};                     // 容量,当值为-1表示无限容量
  std::vector<std::vector<Slot>> slots; // 格子矩阵

public:
  Bag() { set_size(rows, cols); }

public:
  void init_data(std::vector<Item> data, std::vector<int> nums) {
    clean();
    // id以及count
    for (size_t i = 0; i < data.size() && i < nums.size(); i++) {
      addItem(&data[i], nums[i]);
    }
  }

  void clean() {
    // for (size_t i = 0; i < rows; ++i) {
    //   for (size_t j = 0; j < cols; ++j) {
    //     slots[i][j].clean();
    //   }
    // }
  }
  void set_size(size_t rows, size_t cols);
  void set_capacity(int cap) { capacity = cap; }
  // 添加物品到背包
  bool addItem(const Item *item, int num);
  // 通过goodid获取物品信息
  Slot getItem(int good_id);
  // 消耗物品
  bool consumeItem(int good_id, int num);
  int get_used_capacity() const;
  int get_all_capacity() const { return capacity; }
  // 获取背包状态
  std::vector<const Slot *> get_all_data();
};
} // namespace libs