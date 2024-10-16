#include "inventory/inventory.h"
#include <iostream>

using namespace libs;

void Inventory::init_data(std::vector<Item> data) {
  for (auto item : data) {
    database.addItem(item.id, item);
  }
}

std::vector<const Slot *> Inventory::get_all_data() {
  return backpack.get_all_data();
}

int Inventory::get_count(int good_id) {
  return backpack.getItem(good_id).get_count();
}

void Inventory::set_size(int rows, int cols) { backpack.set_size(rows, cols); }

bool Inventory::addItem(int good_id, int num) {
  Item item = database.getItem(good_id);
  if (item.is_empty()) {
    std::cout << "物品不存在，无法添加" << std::endl;
    return false;
  }
  return backpack.addItem(&item, num);
}

std::vector<std::map<std::string, std::string>> Inventory::marshal() {
  std::vector<std::map<std::string, std::string>> ids;
  for (auto it : backpack.get_all_data()) {
    ids.push_back({
        {"id", std::to_string(it->get_goodid())},
        {"count", std::to_string(it->get_count())},
    });
  }
  return ids;
}

void Inventory::unmarshal(
    std::vector<std::map<std::string, std::string>> data) {
  std::vector<int> ids;
  std::vector<int> counts;

  for (auto it : data) {
    if (it.find("id") == it.end() || it.find("count") == it.end()) {
      continue;
    }
    ids.push_back(std::stoi(it["id"]));
    counts.push_back(std::stoi(it["count"]));
  }
  init_data_by_id(ids, counts);
}

bool Inventory::consumeItem(int good_id, int num) {
  return backpack.consumeItem(good_id, num);
}