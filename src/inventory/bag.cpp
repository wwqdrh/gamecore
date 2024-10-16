#include "inventory/bag.h"
#include "inventory/slot.h"

using namespace libs;

void Bag::set_size(size_t rows, size_t cols) {
  this->rows = rows;
  this->cols = cols;
  slots = std::vector<std::vector<Slot>>(rows, std::vector<Slot>(cols));
}

int Bag::get_used_capacity() const {
  int used = 0;
  for (int i = 0; i < rows; ++i) {
    for (int j = 0; j < cols; ++j) {
      used += slots[i][j].get_count();
    }
  }
  return used;
}

bool Bag::addItem(const Item *item, int num) {
  if (capacity != -1 && get_used_capacity() + num > capacity) {
    return false;
  }

  for (int i = 0; i < rows; ++i) {
    for (int j = 0; j < cols; ++j) {
      if (slots[i][j].addItem(item, num)) {
        return true;
      }
    }
  }
  return false; // 没有找到合适的空位或叠加失败
}

Slot Bag::getItem(int good_id) {
  for (int i = 0; i < rows; ++i) {
    for (int j = 0; j < cols; ++j) {
      if (!slots[i][j].is_empty() && slots[i][j].get_goodid() == good_id) {
        return slots[i][j];
      }
    }
  }
  return Slot();
}

std::vector<const Slot *> Bag::get_all_data() {
  std::vector<const Slot *> data;
  for (int i = 0; i < rows; ++i) {
    for (int j = 0; j < cols; ++j) {
      if (!slots[i][j].is_empty()) {
        data.push_back(&slots[i][j]);
      }
    }
  }
  return data;
}

bool Bag::consumeItem(int good_id, int num) {
  for (int i = 0; i < rows; ++i) {
    for (int j = 0; j < cols; ++j) {
      if (!slots[i][j].is_empty() && slots[i][j].get_goodid() == good_id) {
        if (slots[i][j].consumeItem(num)) {
          return true;
        }
      }
    }
  }
  return false; // 没有找到对应的物品
}