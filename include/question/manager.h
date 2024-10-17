#pragma once

#include "question/pool.h"

namespace libs {
class QuesManager {
private:
  TaskAvaliablePool avaliable_pool;
  TaskActivePool active_pool;
  TaskCompletePool complete_pool;

public:
  void startTask(int id) { avaliable_pool.has_task(id); }
  void updateTask(int id) { active_pool.has_task(id); }
  void completeTask(int id) { complete_pool.has_task(id); }
};
} // namespace libs