#pragma once
#include <cstddef>
#include <map>
#include <memory>
#include <string>
#include <vector>

#include "gjson.h"
#include "inventory/bag.h"
#include "inventory/database.h"
#include "inventory/item.h"
#include "inventory/slot.h"
#include "rapidjson/document.h"

namespace gamedb {

// 管理Bag以及Database
class Inventory {
public:
  static inline std::string DB_PREFIX = "gamedb;inventory";

private:
  int max_slot_ = -1;
  int pagesize_slot_ = -1;
  std::map<std::string, int> ids_;
  int max_ids_ = -1;

  std::vector<std::shared_ptr<Slot>> slots_;
  std::shared_ptr<GJson> gjson_;

public:
  Inventory() = default;
  Inventory(std::shared_ptr<GJson> store) {
    set_store(store);
  }
  Inventory(int slot) : max_slot_(slot) {}
  Inventory(int slot, int page_slot)
      : max_slot_(slot), pagesize_slot_(page_slot) {}
  Inventory(int slot, std::map<std::string, int> ids, int max_ids)
      : max_slot_(slot), ids_(ids), max_ids_(max_ids) {}

public:
  // 序列化与反序列化
  rapidjson::Value toJson(rapidjson::Document::AllocatorType &allocator) const {
    rapidjson::Value obj(rapidjson::kObjectType);
    obj.AddMember("max_slot", GJson::toValue(max_slot_, allocator), allocator);
    obj.AddMember("pagesize", GJson::toValue(pagesize_slot_, allocator),
                  allocator);
    obj.AddMember("ids", GJson::toValue(ids_, allocator), allocator);
    obj.AddMember("max_ids", GJson::toValue(max_ids_, allocator), allocator);
    return obj;
  }

  static Inventory fromJson(const rapidjson::Value &value) {
    Inventory inventory;
    if (value.IsNull() || !value.IsObject()) {
      return inventory;
    }

    if (value.HasMember("max_slot")) {
      inventory.max_slot_ = GJson::convert<int>(value["max_slot"]);
    }
    if (value.HasMember("pagesize")) {
      inventory.pagesize_slot_ = GJson::convert<int>(value["pagesize"]);
    }
    if (value.HasMember("ids")) {
      inventory.ids_ = GJson::convert<std::map<std::string, int>>(value["ids"]);
    }
    if (value.HasMember("max_ids")) {
      inventory.max_ids_ = GJson::convert<int>(value["max_ids"]);
    }
    return inventory;
  }

public:
  // 移动赋值语义
  Inventory(Inventory &&other) noexcept = default;
  Inventory &operator=(Inventory &&other) noexcept {
    if (this != &other) {
      max_slot_ = other.max_slot_;
      pagesize_slot_ = other.pagesize_slot_;
      ids_ = std::move(other.ids_);
      max_ids_ = other.max_ids_;
      if (other.slots_.size() > 0) {
        slots_ = std::move(other.slots_);
      }
      if (other.gjson_) {
        gjson_ = other.gjson_;
      }
    }
    return *this;
  }

public:
  void set_store(std::shared_ptr<GJson> g) {
    gjson_ = g;
    load();
  }

  void store() {
    if (gjson_ == nullptr) {
      return;
    }
    auto all = gjson_->get_alloctor();
    rapidjson::Value val = toJson(all);
    gjson_->update(DB_PREFIX, "~", val);
  }
  bool add_item(std::shared_ptr<GoodItem> good) {
    for (auto item : slots_) {
      if (item->addGood(good)) {
        // 判断是否存在, 不存在则创建name与id的映射
        // 可以快速查找一个商品是否存在
        if (!has_item(good->name)) {
          get_create_id(good->name);
        }
        return true;
      }
    }

    if (max_slot_ == -1 || slots_.size() < max_slot_) {
      slots_.push_back(std::make_shared<Slot>());
      slots_.back()->addGood(good);
      if (!has_item(good->name)) {
        get_create_id(good->name);
      }
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
    auto it = ids_.find(name);
    return it != ids_.end();
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

private:
  void load() {
    if (gjson_ == nullptr) {
      return;
    }
    auto v = gjson_->query_value(DB_PREFIX);
    if (v == nullptr) {
      return;
    }
    Inventory other = Inventory::fromJson(*v);
    *this = std::move(other);
  }
};

} // namespace gamedb