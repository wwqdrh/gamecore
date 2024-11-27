#pragma once

#include "dataclass.h"
#include <string>
#include <vector>
namespace gamedb {

// 任务目标
class QuesTarget : public DataClass<QuesTarget> {
public:
  std::string desc;
  std::string event_flag; // 用于标识哪些事件可以触发这个target的更新
  int progress; // 目标值，所有的任务完成进度都用int来量化

public:
  QuesTarget() {
    addMember("desc", &QuesTarget::desc);
    addMember("flag", &QuesTarget::event_flag);
    addMember("progress", &QuesTarget::progress);
  }
  ~QuesTarget() = default;
  explicit QuesTarget(const std::string &data) : QuesTarget() {
    fromJson(data);
  }
  explicit QuesTarget(const std::string &desc, const std::string &event_flag,
                      int progress = 0)
      : QuesTarget() {
    this->desc = desc;
    this->event_flag = event_flag;
    this->progress = progress;
  }
};
} // namespace gamedb