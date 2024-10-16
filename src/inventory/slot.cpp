#include "inventory/slot.h"

using namespace libs;

bool Slot::addItem(const Item *new_item, int num) {
  if (item.is_empty()) {
    item = new_item;
    count = num;
    return true;
  } else if (item.id == new_item->id &&
             (this->stack == -1 || count + num <= this->stack)) {
    count += num;
    return true;
  }
  return false; // 无法叠加
}

bool Slot::consumeItem(int num) {
  if (count >= num) {
    count -= num;
    if (count == 0) {
      // 物品用完，清空格子
      item.clean();
    }
    return true;
  }
  return false;
}

bool Slot::canCombine(Slot *new_item) const {
  if (item.is_empty()) {
    return true;
  }

  if (item.id != new_item->get_goodid()) {
    return false;
  }

  if (this->count + new_item->count <= this->stack || this->stack == -1) {
    return true;
  }
  return false;
}