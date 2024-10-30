#pragma once
#include <cstddef>
#include <map>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

#include "rapidjson/document.h"

#include "gjson.h"
#include "inventory/item.h"
#include "inventory/slot.h"
#include "lock.h"
#include "timedmap.h"

namespace gamedb {

// 管理Bag以及Database
class Inventory {
public:
  static inline std::string DB_PREFIX = "gamedb;inventory";
  using CallbackFunc =
      std::function<void(const std::string &path, const int value)>;

private:
  std::string name = "default";
  int max_slot_ = -1;
  int pagesize_slot_ = -1;
  std::map<std::string, int> ids_;
  int max_ids_ = -1;

  // std::vector<std::shared_ptr<Slot>> slots_;
  TimedOrderedMap<std::string, std::shared_ptr<Slot>> slots_;
  std::shared_ptr<GJson> gjson_;
  std::map<std::string, std::vector<CallbackFunc>> callbacks;

  // mutable std::recursive_mutex rw_mtx;
  mutable ReentrantRWLock rwlock;

public:
  Inventory() = default;
  Inventory(std::shared_ptr<GJson> store) { set_store(store); }
  Inventory(int slot) : max_slot_(slot) {}
  Inventory(int slot, int page_slot)
      : max_slot_(slot), pagesize_slot_(page_slot) {}
  Inventory(int slot, std::map<std::string, int> ids, int max_ids)
      : max_slot_(slot), ids_(ids), max_ids_(max_ids) {}

public:
  // 序列化与反序列化
  rapidjson::Value toJson(rapidjson::Document::AllocatorType &allocator) const {
    auto reader = rwlock.shared_lock();

    rapidjson::Value obj(rapidjson::kObjectType);
    obj.AddMember("name", GJson::toValue(name, allocator), allocator);
    obj.AddMember("max_slot", GJson::toValue(max_slot_, allocator), allocator);
    obj.AddMember("pagesize", GJson::toValue(pagesize_slot_, allocator),
                  allocator);
    obj.AddMember("ids", GJson::toValue(ids_, allocator), allocator);
    obj.AddMember("max_ids", GJson::toValue(max_ids_, allocator), allocator);
    // slots name
    std::vector<std::string> slot_names;
    for (auto &slot : slots_.getKeysByInsertionOrder()) {
      slot_names.push_back(slot);
    }
    obj.AddMember("good_names", GJson::toValue(slot_names, allocator),
                  allocator);

    // slots count
    std::vector<int> slot_counts;
    for (auto &name : slots_.getKeysByInsertionOrder()) {
      slot_counts.push_back(get_item(name)->count);
    }
    obj.AddMember("good_counts", GJson::toValue(slot_counts, allocator),
                  allocator);
    return obj;
  }

  static Inventory fromJson(const rapidjson::Value &value) {
    Inventory inventory;
    if (value.IsNull() || !value.IsObject()) {
      return inventory;
    }

    if (value.HasMember("name")) {
      inventory.name = GJson::convert<std::string>(value["name"]);
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
    if (value.HasMember("good_names") && value.HasMember("good_counts")) {
      auto names =
          GJson::convert<std::vector<std::string>>(value["good_names"]);
      auto counts = GJson::convert<std::vector<int>>(value["good_counts"]);
      if (names.size() != counts.size()) {
        return inventory;
      }
      for (int i = 0; i < names.size(); i++) {
        inventory.add_item(std::make_shared<GoodItem>(names[i], counts[i]));
      }
    }
    return inventory;
  }

public:
  // mutex不能拷贝不能移动
  // 禁用拷贝构造和拷贝赋值
  Inventory(const Inventory &) = delete;
  Inventory &operator=(const Inventory &) = delete;

  // 移动赋值语义
  Inventory(Inventory &&other) noexcept {
    // std::lock_guard<std::recursive_mutex> lock(other.rw_mtx);
    auto reader = other.rwlock.unique_lock();

    max_slot_ = other.max_slot_;
    pagesize_slot_ = other.pagesize_slot_;
    ids_ = std::move(other.ids_);
    max_ids_ = other.max_ids_;
    if (other.slots_.size() > 0) {
      slots_ = std::move(other.slots_);
    }
    if (other.gjson_) {
      gjson_ = other.gjson_;
    };
  }

  Inventory &operator=(Inventory &&other) noexcept {
    if (this != &other) {
      auto reader_this = rwlock.unique_lock();
      auto reader_other = other.rwlock.unique_lock();

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
  void set_name(const std::string &name) { this->name = name; }
  void set_maxslot(int slot) { max_slot_ = slot; }
  void set_pagesize(int slot) { pagesize_slot_ = slot; }
  // ====
  // 订阅通知
  // ====
  // 注册通知
  // 订阅路径变化
  void subscribe(const std::string &path, CallbackFunc &&callback) {
    // 直接使用 std::forward 转发
    subscribeImpl(path, std::forward<CallbackFunc>(callback));
  }

  // 左值引用版本
  void subscribe(const std::string &path, const CallbackFunc &callback) {
    // 复制回调函数
    subscribeImpl(path, callback);
  }

  // ====
  // crud
  // ====
  void set_store(std::shared_ptr<GJson> g) {
    auto writer = rwlock.unique_lock();

    gjson_ = g;
    load();
  }

  void store() {
    auto writer = rwlock.unique_lock();

    if (gjson_ == nullptr) {
      return;
    }
    auto all = gjson_->get_alloctor();
    rapidjson::Value val = toJson(all);
    gjson_->update(DB_PREFIX + ";" + name, "~", val);
  }
  bool add_item(std::shared_ptr<GoodItem> good) {
    auto writer = rwlock.unique_lock();

    for (auto goodname : slots_.getKeysByInsertionOrder()) {
      auto item = slots_.get(goodname);
      if (item->addGood(good)) {
        // 判断是否存在, 不存在则创建name与id的映射
        // 可以快速查找一个商品是否存在
        if (!has_item(good->name)) {
          get_create_id(good->name);
        }
        // 添加成功，那么进行通知
        if (callbacks.find(good->name) != callbacks.end()) {
          for (auto itemfn : callbacks[good->name]) {
            itemfn(good->name, item->get_good_count());
          }
        }
        store();
        return true;
      }
    }

    if (max_slot_ == -1 || slots_.size() < max_slot_) {
      auto slot = std::make_shared<Slot>();
      slot->addGood(good);
      slots_.insert(good->name, slot);
      if (!has_item(good->name)) {
        get_create_id(good->name);
      }
      // 添加成功，那么进行通知
      if (callbacks.find(good->name) != callbacks.end()) {
        for (auto itemfn : callbacks[good->name]) {
          itemfn(good->name, slot->get_good_count());
        }
      }
      store();
      return true;
    }

    return false;
  }
  bool clear() {
    auto writer = rwlock.unique_lock();

    slots_.clear();
    store();
    return true;
  }
  bool consume_item(const std::string &name, int count) {
    auto item = get_item(name);
    if (!item || item->count < count) {
      return false;
    }
    item->count -= count;
    if (item->count == 0) {
      // 需要将这个slot置空, slots_中间删除一个元素
      slots_.erase(name);
    }
    // 消费成功，那么进行通知
    if (callbacks.find(item->name) != callbacks.end()) {
      for (auto itemfn : callbacks[item->name]) {
        itemfn(item->name, item->count);
      }
    }
    store();
    return true;
  }
  std::vector<std::string> get_goods_name() const {
    std::vector<std::string> res;
    for (auto item : slots_.getKeysByInsertionOrder()) {
      res.push_back(item);
    }
    return res;
  }
  int get_create_id(const std::string &name) {
    auto writer = rwlock.unique_lock();

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
    auto read = rwlock.shared_lock();

    for (auto key_name : slots_.getKeysByInsertionOrder()) {
      auto item = slots_.get(key_name);
      if (!item->isEmpty() && item->get_good_name() == name) {
        return true;
      }
    }
    return false;
  }
  std::shared_ptr<GoodItem> get_item(const std::string &name) const {
    auto read = rwlock.shared_lock();

    for (auto key_name : slots_.getKeysByInsertionOrder()) {
      auto item = slots_.get(key_name);
      if (!item->isEmpty() && item->get_good_name() == name) {
        return item->get_good();
      }
    }
    return nullptr;
  }
  std::vector<std::shared_ptr<GoodItem>> filter(const std::string &name,
                                                GoodItem::variant val) const {
    auto read = rwlock.shared_lock();

    std::vector<std::shared_ptr<GoodItem>> res;
    for (const std::string &key_name : slots_.getKeysByInsertionOrder()) {
      auto item = slots_.get(key_name);
      if (!item->isEmpty() && item->get_good()->check_ext(name, val)) {
        res.push_back(item->get_good());
      }
    }
    return res;
  }
  int fill_slot_num() const {
    auto read = rwlock.shared_lock();

    int count = 0;
    for (auto name : slots_.getKeysByInsertionOrder()) {
      auto item = slots_.get(name);
      if (!item->isEmpty())
        count++;
    }
    return count;
  }
  // 控制背包内容分页的
  int page_size() const {
    auto read = rwlock.shared_lock();

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
    auto v = gjson_->query_value(DB_PREFIX + ";" + name);
    if (v == nullptr) {
      return;
    }
    Inventory other = Inventory::fromJson(*v);
    *this = std::move(other);
  }

  void subscribeImpl(const std::string &path, const CallbackFunc &callback) {
    std::unique_lock<ReentrantRWLock> lock(rwlock);
    if (callbacks.find(path) == callbacks.end()) {
      callbacks[path] = std::vector<CallbackFunc>();
    }
    callbacks[path].push_back(callback);
  }
};

} // namespace gamedb