#pragma once
#include <cstddef>

#include "inventory/database.h"

namespace libs {
class Slot {
private:
  Item item;     // 指向物品的指针
  int count{0};  // 当前格子中的物品数量
  int stack{-1}; // 最大堆叠数量, -1为不限制

public:
  Slot() {}
  std::string get_name() const { return item.name; }
  int get_goodid() const { return item.id; }
  int get_count() const { return count; }
  bool is_empty() const { return item.is_empty(); }
  void set_stack(int new_stack) { stack = new_stack; }
  void clean() {
    item.clean();
    count = 0;
    stack = -1;
  }

public:
  // 添加物品到格子
  bool addItem(const Item *new_item, int num);

  // 消耗物品
  bool consumeItem(int num);

  // 判断是否可以叠加
  bool canCombine(Slot *new_item) const;
};
} // namespace libs