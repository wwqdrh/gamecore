#pragma once

#include "dataclass.h"

class TaskItem : public DataClass<TaskItem> {
public:
  std::string name;
  int age;
  double height;

  TaskItem() {
    addMember("name", &TaskItem::name);
    addMember("age", &TaskItem::age);
    addMember("height", &TaskItem::height);
  }
};