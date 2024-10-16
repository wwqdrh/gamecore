#pragma once
#include <vector>

#include "question/task.h"

namespace libs {
class TaskPool {
private:
  std::vector<Task> tasks;

public:
  TaskPool() {}

  std::string marshalJSON() {
    std::string result = "[";
    for (size_t i = 0; i < tasks.size(); i++) {
      result += tasks[i].to_json();
      if (i != tasks.size() - 1) {
        result += ",";
      }
    }
    result += "]";
    return result;
  }

  // 如果某个task.id已经存在了，那么就不能再添加了
  void add_task(Task task) {
    if (has_task(task.id)) {
      return;
    }
    tasks.push_back(task);
  }

  bool has_task(int task_id) {
    for (size_t i = 0; i < tasks.size(); i++) {
      Task current_task = tasks[i];
      if (current_task.id == task_id) {
        return true;
      }
    }
    return false;
  }

  Task pop_task(int task_id) {
    for (size_t i = 0; i < tasks.size(); i++) {
      Task current_task = tasks[i];
      if (current_task.id == task_id) {
        tasks.erase(tasks.begin() + int(i));
        return current_task;
      }
    }
    return Task();
  }

  void remove_task(int task_id) {
    for (size_t i = 0; i < tasks.size(); i++) {
      Task current_task = tasks[i];
      if (current_task.id == task_id) {
        tasks.erase(tasks.begin() + int(i));
        return;
      }
    }
  }

  void update_task_progress(int task_id, int progress) {
    for (size_t i = 0; i < tasks.size(); i++) {
      Task current_task = tasks[i];
      if (current_task.id == task_id) {
        current_task.update_progress(progress);
        return;
      }
    }
  }

  std::vector<Task> get_all_tasks() { return tasks; }
};
} // namespace libs