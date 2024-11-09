#include <gtest/gtest.h>

#include "timedmap.h"

TEST(TimedMapTest, TestOrder) {
  gamedb::TimedOrderedMap<std::string, int> tmap;

  // 插入数据
  tmap.insert("key1", 100);
  tmap.insert("key2", 200);

  ASSERT_TRUE(tmap.get("key1").has_value() && tmap.get("key1").value() == 100);
  ASSERT_TRUE(tmap.get("key2").has_value() && tmap.get("key2").value() == 200);
  // 获取最早插入的数据

  ASSERT_EQ(tmap.getKeysByInsertionOrder()[0], "key1");
  ASSERT_EQ(tmap.getKeysByInsertionOrder()[1], "key2");
}
