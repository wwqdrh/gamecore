#pragma once

#include <map>

namespace libs {

class Item {
public:
  int id{0};            // 物品ID
  std::string name{""}; // 物品名称
  int value{0};         // 价值
  int weight{0};        // 物品重量

public:
  Item() {}
  // 拷贝构造函数
  Item(const Item &other) {
    // 执行成员变量的拷贝
    id = other.id;
    name = other.name;
    value = other.value;
    weight = other.weight;
  }
  Item(std::map<std::string, std::string> data) {
    if (data.find("id") != data.end())
      id = std::stoi(data.find("id")->second);

    if (data.find("name") != data.end())
      name = data.find("name")->second;

    if (data.find("value") != data.end())
      value = std::stoi(data.find("value")->second);

    if (data.find("weight") != data.end())
      weight = std::stoi(data.find("weight")->second);
  }
  Item &operator=(const Item &other) {
    // 检查是否是自我赋值
    if (this != &other) {
      // 执行成员变量的拷贝
      id = other.id;
      name = other.name;
      value = other.value;
      weight = other.weight;
    }
    return *this; // 返回当前对象的引用
  }
  // 允许从 const Item* 进行赋值的运算符
  Item &operator=(const Item *other) {
    if (other != nullptr) {
      // 将指针指向的对象的值赋给当前对象
      id = other->id;
      name = other->name;
      value = other->value;
      weight = other->weight;
    }
    return *this; // 返回当前对象的引用
  }
  // 由于需要将其加入到map中，需要定义hash
  bool operator==(const Item &other) const { return id == other.id; }

public:
  // 从map对象中获取数据并构造
  void clean() {
    id = 0;
    name = "";
    value = 0;
    weight = 0;
  }

  bool is_empty() const { return id == 0; }
};

class Database {
private:
  std::unordered_map<int, Item> item_map; // 通过 good_id 查找物品
  // 动态查询数据的函数，传入id，返回*Item类型
  std::function<Item(int)> query_func;

public:
  void setQueryFunc(std::function<Item(int)> func) { query_func = func; }
  bool addItem(int good_id, Item data);
  // 通过 good_id 获取物品信息
  Item getItem(int good_id);
};
} // namespace libs

namespace std {
template <> struct hash<libs::Item> {
  size_t operator()(const libs::Item &item) const {
    return std::hash<int>()(item.id); // 假设用 id 作为哈希值
  }
};
} // namespace std