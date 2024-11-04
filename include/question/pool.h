#pragma once
#include <memory>
#include <vector>

#include "question/item.h"
#include "rapidjson/document.h"

namespace gamedb {
class TaskPool {
private:
  std::vector<std::shared_ptr<TaskItem>> tasks;

public:
  std::string toJSON() const { return TaskItem::toJsonArr(tasks); }

  void fromJSON(const std::string &data) {
    tasks = TaskItem::fromJsonArr(data);
  }

  void fromJSONValue(const rapidjson::Value &data) {
    if (data.IsNull()) {
      return;
    }
    tasks = TaskItem::fromJsonValueArr(data);
  }

  std::vector<int> get_all_id() const {
    std::vector<int> res;
    for (size_t i = 0; i < tasks.size(); i++) {
      res.push_back(tasks[i]->id);
    }
    return res;
  }
  const std::vector<std::shared_ptr<TaskItem>> &get_all_tasks() const {
    return tasks;
  }
  // 如果某个task.id已经存在了，那么就不能再添加了
  bool add_task(std::shared_ptr<TaskItem> task) {
    if (!task || has_task(task->id)) {
      return false;
    }
    tasks.push_back(task);
    return true;
  }
  bool addTaskTarget(int taskid, const std::string &desc, int progress,
                     const std::string &event_flag = "") {
    for (auto item : tasks) {
      if (item->id == taskid) {
        return item->addTarget(desc, event_flag, progress);
      }
    }
    return false;
  }
  void updateTaskTarget(int taskid, int targetid, int progress) {
    for (auto item : tasks) {
      if (item->id == taskid) {
        item->updateTarget(targetid, progress);
      }
    }
  }
  bool has_task(int task_id) const {
    for (size_t i = 0; i < tasks.size(); i++) {
      std::shared_ptr<TaskItem> current_task = tasks[i];
      if (current_task->id == task_id) {
        return true;
      }
    }
    return false;
  }

  bool checkComplete(int taskid) const {
    for (auto item : tasks) {
      if (item->id == taskid) {
        return item->checkComplete();
      }
    }
    return false;
  }

  std::shared_ptr<TaskItem> pop_task(int task_id) {
    for (size_t i = 0; i < tasks.size(); i++) {
      std::shared_ptr<TaskItem> current_task = tasks[i];
      if (current_task->id == task_id) {
        tasks.erase(tasks.begin() + int(i));
        return current_task;
      }
    }
    return nullptr;
  }

  std::shared_ptr<TaskItem> remove_task(int task_id) {
    for (size_t i = 0; i < tasks.size(); i++) {
      std::shared_ptr<TaskItem> current_task = tasks[i];
      if (current_task->id == task_id) {
        tasks.erase(tasks.begin() + int(i));
        return current_task;
      }
    }
    return nullptr;
  }

  // void update_task_progress(int task_id, int progress) {
  //   for (size_t i = 0; i < tasks.size(); i++) {
  //     std::shared_ptr<TaskItem> current_task = tasks[i];
  //     if (current_task->id == task_id) {
  //       current_task->progress += progress;
  //       return;
  //     }
  //   }
  // }

  std::vector<std::shared_ptr<TaskItem>> get_all_tasks() { return tasks; }
};

class TaskAvaliablePool : public TaskPool {};
class TaskActivePool : public TaskPool {};
class TaskCompletePool : public TaskPool {};
} // namespace gamedb