#pragma once

#include "inventory/bag.h"
#include "inventory/database.h"
#include <vector>

namespace libs {

// 管理Bag以及Database
class Inventory {
private:
  Bag backpack;
  Database database;

public:
  Inventory() {}
  ~Inventory() {}
  void init_data(std::vector<Item> data);
  void init_data_by_id(std::vector<int> ids, std::vector<int> counts) {
    std::vector<Item> items;
    for (size_t i = 0; i < ids.size(); i++) {
      Item val = database.getItem(ids[i]);
      if (!val.is_empty())
        items.push_back(val);
    }
    backpack.init_data(items, counts);
  }

public:
  void setQueryFunc(std::function<Item(int)> func) {
    database.setQueryFunc(func);
  }
  int get_count(int good_id);
  std::vector<const Slot *> get_all_data();
  std::vector<std::map<std::string, std::string>> marshal();
  void unmarshal(std::vector<std::map<std::string, std::string>> data);
  void set_size(int rows, int cols);
  void set_capacity(int cap) { backpack.set_capacity(cap); }
  bool addItem(int good_id, int num);
  bool consumeItem(int good_id, int num);
};
} // namespace libs