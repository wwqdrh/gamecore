#include <gtest/gtest.h>
#include <iostream>
#include <memory>
#include <thread>
#include <vector>

#include "gjson.h"
#include "inventory/inventory.h"

using namespace gamedb;

TEST(InventoryTest, TestInventoryCRUD) {
  // 背包系统
  // 设置最大格数
  // 添加物品，根据id进行判断，如果id相同则加在一起，否则新建一个格子，如果格子不够了那么就不能继续添加了
  Inventory inv(3);
  inv.add_item(std::make_shared<GoodItem>("商品1", 1));
  ASSERT_EQ(inv.fill_slot_num(), 1);
  inv.add_item(std::make_shared<GoodItem>("商品2", 1));
  ASSERT_EQ(inv.fill_slot_num(), 2);
  inv.add_item(std::make_shared<GoodItem>("商品3", 1));
  inv.add_item(std::make_shared<GoodItem>("商品4", 1));
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
  inv3.add_item(std::make_shared<GoodItem>("商品1", 1));
  ASSERT_EQ(inv3.page_size(), 1);
  inv3.add_item(std::make_shared<GoodItem>("商品2", 1));
  ASSERT_EQ(inv3.page_size(), 1);
  inv3.add_item(std::make_shared<GoodItem>("商品3", 1));
  ASSERT_EQ(inv3.page_size(), 1);
  inv3.add_item(std::make_shared<GoodItem>("商品4", 1));
  ASSERT_EQ(inv3.page_size(), 2);
}

TEST(InventoryTest, TestInventoryExtInfo) {
  // 为GoodItem添加扩展信息
  // 并且支持调用filter进行筛选
  Inventory inv(3);
  std::vector<std::string> exts({"category"});
  inv.add_item(std::make_shared<GoodItem>("商品1", 1, exts));
  if (auto v = inv.get_item("商品1")) {
    v->set_ext("category", "种类1");
  }
  inv.add_item(std::make_shared<GoodItem>("商品2", 1, exts));
  if (auto v = inv.get_item("商品2")) {
    v->set_ext("category", "种类2");
  }
  ASSERT_EQ(inv.filter("category", "种类1").size(), 1);
  ASSERT_EQ(inv.filter("category", "种类2").size(), 1);
}

TEST(InventoryTest, TestInventoryAutoStore) {
  std::string test_file = "test_inventory_autostore.json";

  auto json = std::make_shared<GJson>(std::make_shared<FileStore>(test_file));

  Inventory inv(10, 3);
  inv.set_store(json);
  inv.add_item(std::make_shared<GoodItem>("商品1", 1));
  inv.add_item(std::make_shared<GoodItem>("商品2", 1));
  inv.add_item(std::make_shared<GoodItem>("商品3", 1));
  inv.store();
  std::cout << json->query("") << std::endl;
  ASSERT_EQ(json->queryT<int>(Inventory::DB_PREFIX + ";max_slot"), 10);
  ASSERT_EQ(json->queryT<int>(Inventory::DB_PREFIX + ";pagesize"), 3);

  // 测试是否自动保存了
  // 从test_filesave.json中加载数据
  Inventory inv2(
      std::make_shared<GJson>(std::make_shared<FileStore>(test_file)));
  ASSERT_EQ(inv2.has_item("商品1"), true);
  ASSERT_EQ(inv2.has_item("商品2"), true);
  ASSERT_EQ(inv2.has_item("商品4"), false);
  // 删除test_filesave.json文件
  std::remove(test_file.c_str());
}

TEST(InventoryTest, TestConcurrentInventoryExtInfo) {
  const int NUM_THREADS = 4;
  const int ITEMS_PER_THREAD = 25;

  Inventory inv(NUM_THREADS * ITEMS_PER_THREAD);
  std::vector<std::string> exts({"category"});

  // 用于同步所有线程的开始
  std::vector<std::thread> threads;

  // 用于收集每个线程的断言结果
  std::vector<bool> assert_results(NUM_THREADS, true);
  std::mutex assert_mutex;

  auto thread_func = [&](int thread_id) {
    try {
      // 每个线程添加自己的一组商品
      for (int i = 0; i < ITEMS_PER_THREAD; i++) {
        std::string item_name =
            "商品_" + std::to_string(thread_id) + "_" + std::to_string(i);
        std::string category = "种类_" + std::to_string(thread_id);

        // 添加商品
        inv.add_item(std::make_shared<GoodItem>(item_name, 1, exts));

        // 设置扩展信息
        if (auto v = inv.get_item(item_name)) {
          v->set_ext("category", category);
        }

        // 随机延时模拟真实场景
        // std::this_thread::sleep_for(std::chrono::milliseconds(rand() % 5));
        std::this_thread::yield();

        // 执行过滤操作
        auto filtered_items = inv.filter("category", category);

        // // 验证结果（注意：由于并发，数量可能在增加）
        if (filtered_items.size() < (i + 1)) {
          std::lock_guard<std::mutex> lock(assert_mutex);
          assert_results[thread_id] = false;
        }
      }
    } catch (const std::exception &e) {
      std::lock_guard<std::mutex> lock(assert_mutex);
      assert_results[thread_id] = false;
    }
  };

  // 启动所有线程
  for (int i = 0; i < NUM_THREADS; i++) {
    threads.emplace_back(thread_func, i);
  }

  // 等待所有线程完成
  for (auto &thread : threads) {
    thread.join();
  }

  // 验证最终结果
  for (int thread_id = 0; thread_id < NUM_THREADS; thread_id++) {
    ASSERT_TRUE(assert_results[thread_id])
        << "Thread " << thread_id << " failed its assertions";

    std::string category = "种类_" + std::to_string(thread_id);
    auto filtered_items = inv.filter("category", category);
    ASSERT_EQ(filtered_items.size(), ITEMS_PER_THREAD)
        << "Category " << category << " has wrong number of items";
  }

  // 验证总商品数
  ASSERT_EQ(inv.fill_slot_num(), NUM_THREADS * ITEMS_PER_THREAD)
      << "Total inventory size is incorrect";
}