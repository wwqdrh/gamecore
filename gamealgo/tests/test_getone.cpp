#include <gtest/gtest.h>

#include "getone.h"

using namespace gamealgo;

TEST(TestGetOne, testpity) {
  // 定义物品
  Item commonItem("Common Sword", 1);
  Item rareItem("Rare Shield", 5);
  Item epicItem("Epic Bow", 7);

  // 创建掉落池
  LootPool pool;
  pool.addItem(commonItem, 0.7); // 70%概率掉落普通物品
  pool.addItem(rareItem, 0.3);  // 25%概率掉落稀有物品
  pool.addItem(epicItem, 0);     // 0%概率掉落史诗物品
  // 概率需要总和为1

  // 创建保底系统，假设连续10次未掉落稀有物品时触发保底
  PitySystem pity(2, epicItem);

  // 模拟掉落
  for (int i = 1; i <= 20; ++i) {
    std::optional<Item> guaranteedItem = pity.checkPity();
    // 由于史诗物品概率为0，所以永远抽不到这个，只有保底机制
    if (i > 1 && i % 3 == 0) {
      ASSERT_TRUE(guaranteedItem.has_value());
      ASSERT_EQ(guaranteedItem->name, "Epic Bow");
      ASSERT_EQ(guaranteedItem->rarity, 7);
    }
    auto droppedItem = pool.getRandomItem(guaranteedItem);
    if (droppedItem) {
      pity.updatePity(*droppedItem); // 更新保底系统状态
    }
  }
}