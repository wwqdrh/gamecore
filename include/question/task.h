#pragma once

#include <map>
#include <string>

namespace libs {

class Task {
private:
  int cur_progress{0};

  // 任务状态的枚举值
  enum TaskStatus { NotStarted, InProgress, Completed, Failed };

public:
  int id{0};
  std::string name{""};
  std::string description{""};
  int progress{0};               // 给定一个目标值
  TaskStatus status{NotStarted}; // Not Started, In Progress, Completed, Failed
  int reward{0};
  bool enable{false}; // 是否激活, 只有激活了才能显示出来
  std::string enable_expression{""}; // 满足这个条件才能激活
  std::string check_expression{
      ""}; // 判断是否完成的表达式，与progress不同在于这个更像0，1

  Task() {}

  bool is_null() { return this->id == 0; }

  std::string to_json() const {
    return "{ \"id\": " + std::to_string(id) + ", \"name\": \"" + name +
           "\", \"progress\": " + std::to_string(progress) + " }";
  }

  void update_progress(int new_progress) {
    cur_progress += new_progress;
    if (cur_progress >= progress) {
      complete_task();
    }
  }

  void complete_task() { status = TaskStatus::Completed; }

  void fail_task() { status = TaskStatus::Failed; }

  void reset_task() {
    cur_progress = 0;
    status = TaskStatus::NotStarted;
  }
};
} // namespace libs