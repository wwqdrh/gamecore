#include <gtest/gtest.h>

#include "collection.h"

using namespace gameconsole;

TEST(CollectionTest, TestCRUD) {
  Collection collection;
  bool testResult;

  std::cout << "\n=== Basic Operations Tests ===\n";

  // 添加元素测试
  collection.add("Hello");
  collection.add(42);
  collection.add(3.14);
  // 创建并添加数组
  auto array_variant = Collection::make_array({42, "test", 3.14});
  collection.add(array_variant);
  ASSERT_EQ(collection.size(), 4);

  // 通过索引获取元素测试
  auto value = collection.get_by_index(1);
  ASSERT_EQ(value.has_value(), true);
  ASSERT_EQ(std::get<int>(value.value()), 42);

  // 包含元素测试
  testResult = collection.contains(3.14);
  ASSERT_EQ(testResult, true);

  // 删除元素测试
  collection.remove_by_index(1);
  testResult = collection.size() == 3 && !collection.contains(42);
  ASSERT_EQ(testResult, true);
}

TEST(CollectionTest, TestIteration) {
  Collection collection;
  bool testResult;

  collection.add(1);
  collection.add(2);
  collection.add(3);
  // 测试 first() 和 next()
  auto first = collection.first();
  testResult = first.has_value() && std::get<int>(first.value()) == 1;
  ASSERT_EQ(testResult, true);

  auto next = collection.next();
  testResult = next.has_value() && std::get<int>(next.value()) == 2;
  ASSERT_EQ(testResult, true);

  // 测试 last()
  auto last = collection.last();
  testResult = last.has_value() && std::get<int>(last.value()) == 3;
  ASSERT_EQ(testResult, true);

  // 测试 previous()
  auto prev = collection.previous();
  testResult = prev.has_value() && std::get<int>(prev.value()) == 2;
  ASSERT_EQ(testResult, true);
}

TEST(CollectionTest, TestMapAct) {
  Collection collection;
  bool testResult;

  collection.clear();
  collection.set_value("name", "John");
  collection.set_value("age", 30);

  // 测试键值对
  auto value = collection.get_value("name");
  testResult =
      value.has_value() && std::get<std::string>(value.value()) == "John";
  ASSERT_EQ(testResult, true);

  // 测试键的存在性
  testResult = collection.contains_key("age");
  ASSERT_EQ(testResult, true);

  // 获取所有键
  auto keys = collection.get_keys();
  testResult = keys.size() == 2;
  ASSERT_EQ(testResult, true);
}

TEST(CollectionTest, TestFilter) {
  Collection collection;
  bool testResult;

  collection.add(1);
  collection.add(2);
  collection.add(3);
  collection.add(4);
  collection.add(5);

  // 过滤偶数
  auto filtered =
      collection.filter([](const Variant &key, const Variant &value, int index,
                           const Collection &coll) -> bool {
        // 过滤掉所有偶数
        return std::get<int>(value) % 2 != 0;
      });

  testResult = filtered->size() == 3;
  ASSERT_EQ(testResult, true);
}

TEST(CollectionTest, TestSpecial) {
  Collection emptyCollection;
  bool testResult;

  // 测试空集合操作
  testResult = emptyCollection.is_empty();
  ASSERT_EQ(testResult, true);

  // 测试无效索引
  auto invalidValue = emptyCollection.get_by_index(999);
  testResult = !invalidValue.has_value();
  ASSERT_EQ(testResult, true);

  // 测试清空操作
  emptyCollection.clear();
  testResult = emptyCollection.is_empty();
  ASSERT_EQ(testResult, true);
}