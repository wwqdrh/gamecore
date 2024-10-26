#include <gtest/gtest.h>
#include <iostream>
#include <memory>
#include <vector>

#include "gjson.h"
#include "inventory/inventory.h"

using namespace gamedb;

TEST(InventoryTest, TestInventoryCRUD) {
  // 背包系统
  // 设置最大格数
  // 添加物品，根据id进行判断，如果id相同则加在一起，否则新建一个格子，如果格子不够了那么就不能继续添加了
  Inventory inv(3);
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品1"), "商品1", 1));
  ASSERT_EQ(inv.fill_slot_num(), 1);
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品2"), "商品2", 1));
  ASSERT_EQ(inv.fill_slot_num(), 2);
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品3"), "商品3", 1));
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品4"), "商品4", 1));
  ASSERT_EQ(inv.fill_slot_num(), 3);
  ASSERT_EQ(inv.has_item("商品1"), true);
  ASSERT_EQ(inv.has_item("商品4"), false);

  // 可以设置初始状态，用于控制id的获取生成
  // maxid是一定大于ids的大小的，因为ids可能中途删除了
  Inventory inv2(3, {{"商品1", 0}, {"商品2", 1}, {"商品3", 2}}, 6);
  ASSERT_EQ(inv2.get_create_id("商品4"), 7);
  ASSERT_EQ(inv2.get_create_id("商品4"), 7);
  ASSERT_EQ(inv2.get_create_id("商品5"), 8);

  // 控制背包分页
  Inventory inv3(12, 3);
  inv3.add_item(
      std::make_shared<GoodItem>(inv3.get_create_id("商品1"), "商品1", 1));
  ASSERT_EQ(inv3.page_size(), 1);
  inv3.add_item(
      std::make_shared<GoodItem>(inv3.get_create_id("商品2"), "商品2", 1));
  ASSERT_EQ(inv3.page_size(), 1);
  inv3.add_item(
      std::make_shared<GoodItem>(inv3.get_create_id("商品3"), "商品3", 1));
  ASSERT_EQ(inv3.page_size(), 1);
  inv3.add_item(
      std::make_shared<GoodItem>(inv3.get_create_id("商品4"), "商品4", 1));
  ASSERT_EQ(inv3.page_size(), 2);
}

TEST(InventoryTest, TestInventoryExtInfo) {
  // 为GoodItem添加扩展信息
  // 并且支持调用filter进行筛选
  Inventory inv(3);
  std::vector<std::string> exts({"category"});
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品1"), "商品1", 1, exts));
  if (auto v = inv.get_item("商品1")) {
    v->set_ext("category", "种类1");
  }
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品2"), "商品2", 1, exts));
  if (auto v = inv.get_item("商品2")) {
    v->set_ext("category", "种类2");
  }
  ASSERT_EQ(inv.filter("category", "种类1").size(), 1);
  ASSERT_EQ(inv.filter("category", "种类2").size(), 1);
}

TEST(InventoryTest, TestInventoryAutoStore) {
  Inventory inv(10, 3);
  auto json = std::make_shared<GJson>();
  inv.set_store(json);
  inv.load();
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品1"), "商品1", 1));
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品2"), "商品2", 1));
  inv.add_item(
      std::make_shared<GoodItem>(inv.get_create_id("商品3"), "商品3", 1));
  inv.store();
  ASSERT_EQ(json->queryv<size_t>(Inventory::DB_PREFIX + ";max_slot"), 10);
  ASSERT_EQ(json->queryv<size_t>(Inventory::DB_PREFIX + ";pagesize"), 3);
}
