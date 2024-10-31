#pragma once

#include "gjson.h"
#include "lock.h"
#include "question/item.h"
#include "question/pool.h"
#include "question/target.h"
#include "rapidjson/document.h"
#include <algorithm>
#include <memory>
#include <vector>

namespace gamedb {
class QuesManager {
public:
  static inline std::string DB_PREFIX = "gamedb;question";

private:
  TaskActivePool active_pool;
  TaskCompletePool complete_pool;

  std::shared_ptr<GJson> gjson_;
  // mutable std::recursive_mutex rw_mtx;
  mutable ReentrantRWLock rwlock;

public:
  QuesManager() = default;
  QuesManager(std::shared_ptr<GJson> store) { set_store(store); }
  QuesManager(const std::string &activeJson, const std::string &completeJson) {
    active_pool.fromJSON(activeJson);
    complete_pool.fromJSON(completeJson);
  }
  QuesManager &operator=(QuesManager &&other) noexcept {
    // std::lock_guard<std::recursive_mutex> lock(other.rw_mtx);
    auto reader = other.rwlock.unique_lock();

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
  void set_activate(const rapidjson::Value &data) {
    active_pool.fromJSONValue(data);
  }
  void set_complete(const rapidjson::Value &data) {
    complete_pool.fromJSONValue(data);
  }

  bool startTask(int id,
                 std::vector<std::shared_ptr<QuesTarget>> targets = {}) {
    if (active_pool.has_task(id)) {
      // 不能重复开始
      return false;
    }

    auto task = std::make_shared<TaskItem>(id);
    for (auto &target : targets) {
      task->addTarget(target);
    }
    if (active_pool.add_task(task)) {
      save();
      return true;
    }
    return false;
  }

  // currentProgress是增量形式内无
  // 直接通过指定taskid以及targetid来更新具体的re
  bool updateTaskTarget(int taskid, int targetid, int progress) {
    if (!active_pool.has_task(taskid)) {
      return false;
    }
    active_pool.updateTaskTarget(taskid, targetid, progress);
    if (checkComplete(taskid)) {
      completeTask(taskid);
    }
    save();
    return true;
  }
  // 通过事件flag来判断哪些任务可以更新
  void updateTaskTarget(const std::string &event_flag, int progress) {
    for (auto task : active_pool.get_all_tasks()) {
      // 获取event_flag在task->flags的位置
      auto it = std::find(task->progress_flags.begin(),
                          task->progress_flags.end(), event_flag);
      if (it != task->progress_flags.end()) {
        updateTaskTarget(task->id, it - task->progress_flags.begin(), progress);
      }
    }
  }
  bool checkComplete(int taskId) const {
    if (!active_pool.has_task(taskId)) {
      return false; // 没有开始这个任务
    }
    return active_pool.checkComplete(taskId);
  }

  bool completeTask(int id) {
    if (!active_pool.has_task(id)) {
      return false;
    }
    if (!checkComplete(id)) {
      return false;
    }

    auto task = active_pool.remove_task(id);
    if (task && complete_pool.add_task(task)) {
      save();
      return true;
    }
    return false;
  }
  const std::vector<std::shared_ptr<TaskItem>> &get_active_task() const {
    return active_pool.get_all_tasks();
  }
  std::vector<int> get_active_task_ids() const {
    return active_pool.get_all_id();
  }
  std::vector<int> get_complete_task() const {
    return complete_pool.get_all_id();
  }

private:
  void save() {
    if (gjson_) {
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
    auto active = gjson_->query_value(DB_PREFIX + ";active");
    auto complete = gjson_->query_value(DB_PREFIX + ";complete");

    QuesManager other = QuesManager();
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