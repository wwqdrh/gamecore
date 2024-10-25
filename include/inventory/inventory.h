#pragma once

#include "inventory/bag.h"
#include "inventory/database.h"
#include "inventory/item.h"
#include "inventory/slot.h"
#include <cstddef>
#include <map>
#include <memory>
#include <vector>

namespace gamedb {

// 管理Bag以及Database
class Inventory {
private:
  size_t max_slot_ = -1;
  size_t pagesize_slot_ = -1;
  std::map<std::string, int> ids_;
  int max_ids_ = -1;
  std::vector<std::shared_ptr<Slot>> slots_;
  Bag backpack;
  Database database;

public:
  Inventory() = default;
  Inventory(size_t slot) : max_slot_(slot) {}
  Inventory(size_t slot, size_t page_slot)
      : max_slot_(slot), pagesize_slot_(page_slot) {}
  Inventory(size_t slot, std::map<std::string, int> ids, int max_ids)
      : max_slot_(slot), ids_(ids), max_ids_(max_ids) {}
  bool add_item(std::shared_ptr<GoodItem> good) {
    for (auto item : slots_) {
      if (item->addGood(good)) {
        return true;
      }
    }

    if (max_slot_ == -1 || slots_.size() < max_slot_) {
      slots_.push_back(std::make_shared<Slot>());
      slots_.back()->addGood(good);
      return true;
    }

    return false;
  }
  int get_create_id(const std::string &name) {
    auto it = ids_.find(name);
    if (it == ids_.end()) {
      int id = 0;
      if (max_ids_ == -1) {
        id = ids_.size();
      } else {
        max_ids_ += 1;
        id = max_ids_;
      }
      ids_[name] = id;
      return id;
    }
    return it->second;
  }
  bool has_item(const std::string &name) const {
    for (auto item : slots_) {
      if (!item->isEmpty() && item->get_good_name() == name) {
        return true;
      }
    }
    return false;
  }
  std::shared_ptr<GoodItem> get_item(const std::string &name) const {
    for (auto item : slots_) {
      if (!item->isEmpty() && item->get_good_name() == name) {
        return item->get_good();
      }
    }
    return nullptr;
  }
  std::vector<std::shared_ptr<GoodItem>> filter(const std::string &name,
                                                GoodItem::variant val) {
    std::vector<std::shared_ptr<GoodItem>> res;
    for (auto item : slots_) {
      if (!item->isEmpty() && item->get_good()->check_ext(name, val)) {
        res.push_back(item->get_good());
      }
    }
    return res;
  }
  int fill_slot_num() {
    int count = 0;
    for (auto item : slots_) {
      if (!item->isEmpty())
        count++;
    }
    return count;
  }
  // 控制背包内容分页的
  int page_size() {
    if (pagesize_slot_ == -1) {
      // 不分页
      return 1;
    } else {
      int slot_num = fill_slot_num();
      return (slot_num - 1) / pagesize_slot_ + 1;
    }
  }

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
} // namespace gamedb