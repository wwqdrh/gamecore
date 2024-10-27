#pragma once
#include <cstddef>
#include <memory>

#include "inventory/item.h"

namespace gamedb {
class Slot {
private:
  // Item item; // 指向物品的指针
  std::shared_ptr<GoodItem> good;
  // int count{0};  // 当前格子中的物品数量
  // int stack{-1}; // 最大堆叠数量, -1为不限制
  // bool empty_;   // 是否为空

public:
  Slot() = default;
  // std::string get_name() const { return item.name; }
  // int get_goodid() const { return item.id; }
  // int get_count() const { return count; }
  // bool is_empty() const { return item.is_empty(); }
  // void set_stack(int new_stack) { stack = new_stack; }
  // void clean() {
  //   item.clean();
  //   count = 0;
  //   stack = -1;
  // }

public:
  bool isEmpty() const { return good == nullptr; }
  std::shared_ptr<GoodItem> get_good() const { return good; }
  std::string get_good_name() const {
    if (isEmpty()) {
      return "";
    } else {
      return good->name;
    }
  }
  int get_good_count() const {
    if (isEmpty()) {
      return 0;
    } else {
      return good->count;
    }
  }
  bool addGood(std::shared_ptr<GoodItem> item) {
    if (isEmpty()) {
      good = item;
      return true;
    } else if (item->name == good->name) {
      good->count += item->count;
      return true;
    }
    return false;
  }
};
} // namespace gamedb