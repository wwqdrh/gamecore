#pragma once

#include "dataclass.h"

namespace gamedb {
enum class TaskStatus { NotStarted, InProgress, Completed, Failed };

class TaskItem : public DataClass<TaskItem> {
public:
  int id = 0;
  std::string name = "";
  std::string description = "";
  std::string objective = "";
  int progress = 0;
  bool objective_completed = false;

  TaskItem() {
    addMember("id", &TaskItem::id);
    addMember("name", &TaskItem::name);
    addMember("description", &TaskItem::description);
    addMember("objective", &TaskItem::objective);
    addMember("progress", &TaskItem::progress);
    addMember("objective_completed", &TaskItem::objective_completed);
  }

  explicit TaskItem(const std::string &data) : TaskItem() { fromJson(data); }
};
} // namespace libs