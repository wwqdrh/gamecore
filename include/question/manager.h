#pragma once

#include "gjson.h"
#include "lock.h"
#include "question/item.h"
#include "question/pool.h"
#include "rapidjson/document.h"
#include <algorithm>
#include <memory>
#include <vector>

namespace gamedb {
class QuesManager {
public:
  static inline std::string DB_PREFIX = "gamedb;question";

private:
  TaskAvaliablePool avaliable_pool;
  TaskActivePool active_pool;
  TaskCompletePool complete_pool;

  std::shared_ptr<GJson> gjson_;
  // mutable std::recursive_mutex rw_mtx;
  mutable ReentrantRWLock rwlock;

public:
  QuesManager() = default;
  QuesManager(std::shared_ptr<GJson> store) { set_store(store); }
  QuesManager(const std::string &abaliableJson, const std::string &activeJson,
              const std::string &completeJson) {
    avaliable_pool.fromJSON(abaliableJson);
    active_pool.fromJSON(activeJson);
    complete_pool.fromJSON(completeJson);
  }
  QuesManager &operator=(QuesManager &&other) noexcept {
    // std::lock_guard<std::recursive_mutex> lock(other.rw_mtx);
    auto reader = other.rwlock.unique_lock();

    avaliable_pool = std::move(other.avaliable_pool);
    active_pool = std::move(other.active_pool);
    complete_pool = std::move(other.complete_pool);
    if (other.gjson_) {
      gjson_ = other.gjson_;
    };
    return *this;
  }

public:
  void set_store(std::shared_ptr<GJson> g) {
    auto writer = rwlock.unique_lock();

    gjson_ = g;
    load();
  }
  void set_avaialable(const rapidjson::Value &data) {
    avaliable_pool.fromJSONValue(data);
  }
  void set_activate(const rapidjson::Value &data) {
    active_pool.fromJSONValue(data);
  }
  void set_complete(const rapidjson::Value &data) {
    complete_pool.fromJSONValue(data);
  }
  bool addTask(int id) {
    if (!avaliable_pool.add_task(std::make_shared<TaskItem>(id))) {
      return false;
    }
    save();
    return true;
  }
  bool startTask(int id) {
    if (!avaliable_pool.has_task(id)) {
      return false;
    }

    auto task = avaliable_pool.remove_task(id);
    if (task && active_pool.add_task(task)) {
      save();
      return true;
    }
    return false;
  }
  bool completeTask(int id) {
    if (!active_pool.has_task(id)) {
      return false;
    }

    auto task = active_pool.remove_task(id);
    if (task && complete_pool.add_task(task)) {
      save();
      return true;
    }
    return false;
  }
  std::vector<int> get_available_task() const {
    return avaliable_pool.get_all_id();
  }
  std::vector<int> get_active_task() const { return active_pool.get_all_id(); }
  std::vector<int> get_complete_task() const {
    return complete_pool.get_all_id();
  }

private:
  void save() {
    if (gjson_) {
      gjson_->update(DB_PREFIX + ";avaliable", "~", avaliable_pool.toJSON());
      gjson_->update(DB_PREFIX + ";active", "~", active_pool.toJSON());
      gjson_->update(DB_PREFIX + ";complete", "~", complete_pool.toJSON());
    }
  }
  void load() {
    if (gjson_ == nullptr) {
      return;
    }
    auto v = gjson_->query_value(DB_PREFIX);
    if (v == nullptr) {
      return;
    }
    auto ava = gjson_->query_value(DB_PREFIX + ";avaliable");
    auto active = gjson_->query_value(DB_PREFIX + ";active");
    auto complete = gjson_->query_value(DB_PREFIX + ";complete");

    QuesManager other = QuesManager();
    if (ava) {
      other.set_avaialable(*ava);
    }
    if (active) {
      other.set_activate(*active);
    }
    if (complete) {
      other.set_complete(*complete);
    }
    *this = std::move(other);
  }
};
} // namespace gamedb