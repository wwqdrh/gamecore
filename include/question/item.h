#pragma once

#include "dataclass.h"
#include "question/target.h"
#include <memory>
#include <vector>

namespace gamedb {
enum class TaskStatus { NotStarted, InProgress, Completed, Failed };

class TaskItem : public DataClass<TaskItem> {
public:
  int id = 0;
  std::vector<std::string> progress_flags;

private:
  std::vector<std::string> progress_desc;
  std::vector<int> progress_target;
  std::vector<int> progress_current;

public:
  TaskItem() {
    addMember("id", &TaskItem::id);
    addMember("progress_flags", &TaskItem::progress_flags);
    addMember("progress_desc", &TaskItem::progress_desc);
    addMember("progress_target", &TaskItem::progress_target);
    addMember("progress_current", &TaskItem::progress_current);
  }
  explicit TaskItem(int id) : TaskItem() { this->id = id; }
  explicit TaskItem(const std::string &data) : TaskItem() { fromJson(data); }

public:
  bool addTarget(const std::string &desc, const std::string &flag, int target) {
    progress_flags.push_back(flag);
    progress_desc.push_back(desc);
    progress_target.push_back(target);
    progress_current.push_back(0);
    return true;
  }
  bool addTarget(std::shared_ptr<QuesTarget> target) {
    return addTarget(target->desc, target->event_flag, target->progress);
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