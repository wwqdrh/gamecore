#pragma once
#include <vector>

#include "question/item.h"

namespace libs {
class TaskPool {
private:
  std::vector<TaskItem> tasks;

public:
  std::string toJSON() const { return TaskItem::toJsonArr(tasks); }

  void fromJSON(const std::string &data) {
    tasks = TaskItem::fromJsonArr(data);
  }

  // 如果某个task.id已经存在了，那么就不能再添加了
  void add_task(TaskItem task) {
    if (has_task(task.id)) {
      return;
    }
    tasks.push_back(task);
  }

  bool has_task(int task_id) {
    for (size_t i = 0; i < tasks.size(); i++) {
      TaskItem current_task = tasks[i];
      if (current_task.id == task_id) {
        return true;
      }
    }
    return false;
  }

  TaskItem pop_task(int task_id) {
    for (size_t i = 0; i < tasks.size(); i++) {
      TaskItem current_task = tasks[i];
      if (current_task.id == task_id) {
        tasks.erase(tasks.begin() + int(i));
        return current_task;
      }
    }
    return TaskItem();
  }

  void remove_task(int task_id) {
    for (size_t i = 0; i < tasks.size(); i++) {
      TaskItem current_task = tasks[i];
      if (current_task.id == task_id) {
        tasks.erase(tasks.begin() + int(i));
        return;
      }
    }
  }

  void update_task_progress(int task_id, int progress) {
    for (size_t i = 0; i < tasks.size(); i++) {
      TaskItem current_task = tasks[i];
      if (current_task.id == task_id) {
        current_task.progress += progress;
        return;
      }
    }
  }

  std::vector<TaskItem> get_all_tasks() { return tasks; }
};

class TaskAvaliablePool : public TaskPool {};
class TaskActivePool : public TaskPool {};
class TaskCompletePool : public TaskPool {};
} // namespace libs