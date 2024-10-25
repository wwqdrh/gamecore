#pragma once

#include "question/pool.h"

namespace gamedb {
class QuesManager {
private:
  TaskAvaliablePool avaliable_pool;
  TaskActivePool active_pool;
  TaskCompletePool complete_pool;

public:
  QuesManager(const std::string &abaliableJson, const std::string &activeJson,
              const std::string &completeJson) {
    avaliable_pool.fromJSON(abaliableJson);
    active_pool.fromJSON(activeJson);
    complete_pool.fromJSON(completeJson);
  }
  void startTask(int id) { avaliable_pool.has_task(id); }
  void updateTask(int id) { active_pool.has_task(id); }
  void completeTask(int id) { complete_pool.has_task(id); }
};
} // namespace libs