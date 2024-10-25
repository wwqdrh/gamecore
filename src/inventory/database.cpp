#include "inventory/database.h"

using namespace gamedb;

bool Database::addItem(int good_id, Item data) {
  if (item_map.find(good_id) != item_map.end()) {
    return false;
  }
  item_map[good_id] = data;
  return true;
}

Item Database::getItem(int good_id) {
  if (item_map.find(good_id) != item_map.end()) {
    return item_map[good_id];
  }
  // 从query_func中获取物品信息
  if (query_func != nullptr) {
    return query_func(good_id);
  }
  return Item();
}