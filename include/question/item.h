#pragma once

#include "dataclass.h"
#include <vector>

namespace gamedb {
enum class TaskStatus { NotStarted, InProgress, Completed, Failed };

class TaskItem : public DataClass<TaskItem> {
public:
  int id = 0;
  std::string desc = "";

private:
  std::vector<std::string> progress_desc;
  std::vector<int> progress_target;
  std::vector<int> progress_current;

public:
  TaskItem() {
    addMember("id", &TaskItem::id);
    addMember("desc", &TaskItem::desc);
    addMember("progress_desc", &TaskItem::progress_desc);
    addMember("progress_target", &TaskItem::progress_target);
    addMember("progress_current", &TaskItem::progress_current);
  }
  explicit TaskItem(int id) : TaskItem() { this->id = id; }
  explicit TaskItem(const std::string &data) : TaskItem() { fromJson(data); }

public:
  bool addTarget(const std::string &desc, int target) {
    progress_desc.push_back(desc);
    progress_target.push_back(target);
    progress_current.push_back(0);
    return true;
  }
  bool updateTarget(int targetid, int current) {
    if (targetid < 0 || targetid >= progress_target.size()) {
      return false;
    }
    progress_current[targetid] += current;
    return true;
  }
  bool checkComplete() {
    for (int i = 0; i < progress_target.size(); ++i) {
      if (progress_current[i] < progress_target[i]) {
        return false;
      }
    }
    return true;
  }
};
} // namespace gamedb