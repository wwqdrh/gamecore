#include <gtest/gtest.h>
#include <vector>

#include "algo/boids.h"

using namespace gamealgo;

TEST(TestAlgoBoids, boidsInsert) {
  Rect2 boundary(Vector2(0, 0), Vector2(100, 100));
  QuadTree tree(boundary, 2); // 容量为2的四叉树

  auto obj1 = std::make_shared<QuadTreeObj>(Vector2(25, 25));
  auto obj2 = std::make_shared<QuadTreeObj>(Vector2(75, 75));
  auto obj3 = std::make_shared<QuadTreeObj>(Vector2(25, 75));

  // 测试基本插入
  ASSERT_TRUE(tree.insert(obj1));
  ASSERT_TRUE(tree.insert(obj2));

  // 测试超出边界的插入
  auto outOfBounds = std::make_shared<QuadTreeObj>(Vector2(150, 150));
  ASSERT_TRUE(!tree.insert(outOfBounds));

  // 测试null指针插入
  ASSERT_TRUE(!tree.insert(nullptr));
}

TEST(TestAlgoBoids, boidsQuery) {
  Rect2 boundary(Vector2(0, 0), Vector2(100, 100));
  QuadTree tree(boundary, 2);

  // 插入一些测试对象
  auto obj1 = std::make_shared<QuadTreeObj>(Vector2(25, 25));
  auto obj2 = std::make_shared<QuadTreeObj>(Vector2(75, 75));
  auto obj3 = std::make_shared<QuadTreeObj>(Vector2(25, 75));

  tree.insert(obj1);
  tree.insert(obj2);
  tree.insert(obj3);

  // 测试不同的查询区域
  Rect2 topLeft(Vector2(0, 0), Vector2(50, 50));
  auto results1 = tree.query(topLeft);
  ASSERT_TRUE(results1.size() == 1);

  Rect2 fullArea(Vector2(0, 0), Vector2(100, 100));
  auto results2 = tree.query(fullArea);
  ASSERT_TRUE(results2.size() == 3);

  Rect2 emptyArea(Vector2(100, 100), Vector2(50, 50));
  auto results3 = tree.query(emptyArea);
  ASSERT_TRUE(results3.empty());
}

TEST(TestAlgoBoids, boidsRemove) {
  Rect2 boundary(Vector2(0, 0), Vector2(100, 100));
  QuadTree tree(boundary, 2);

  auto obj1 = std::make_shared<QuadTreeObj>(Vector2(25, 25));
  auto obj2 = std::make_shared<QuadTreeObj>(Vector2(75, 75));

  tree.insert(obj1);
  tree.insert(obj2);

  // 测试移除存在的对象
  ASSERT_TRUE(tree.remove(obj1));

  // 验证对象确实被移除
  Rect2 fullArea(Vector2(0, 0), Vector2(100, 100));
  auto results = tree.query(fullArea);
  ASSERT_TRUE(results.size() == 1);

  // 测试移除不存在的对象
  auto nonExistent = std::make_shared<QuadTreeObj>(Vector2(50, 50));
  ASSERT_TRUE(!tree.remove(nonExistent));

  // 测试移除null指针
  ASSERT_TRUE(!tree.remove(nullptr));
}

TEST(TestAlgoBoids, boidsSubdivide) {
  Rect2 boundary(Vector2(0, 0), Vector2(100, 100));
  QuadTree tree(boundary, 2);

  // 插入足够多的对象触发细分
  auto obj1 = std::make_shared<QuadTreeObj>(Vector2(25, 25));
  auto obj2 = std::make_shared<QuadTreeObj>(Vector2(75, 75));
  auto obj3 = std::make_shared<QuadTreeObj>(Vector2(25, 75));

  tree.insert(obj1);
  tree.insert(obj2);
  tree.insert(obj3);

  // 验证所有对象都能被找到
  Rect2 fullArea(Vector2(0, 0), Vector2(100, 100));
  auto results = tree.query(fullArea);
  ASSERT_TRUE(results.size() == 3);
}

TEST(TestAlgoBoids, boidsMerge) {
  Rect2 boundary(Vector2(0, 0), Vector2(100, 100));
  QuadTree tree(boundary, 4); // 使用更大的容量以测试合并

  // 插入一些对象触发细分
  auto obj1 = std::make_shared<QuadTreeObj>(Vector2(25, 25));
  auto obj2 = std::make_shared<QuadTreeObj>(Vector2(75, 75));
  auto obj3 = std::make_shared<QuadTreeObj>(Vector2(25, 75));

  tree.insert(obj1);
  tree.insert(obj2);
  tree.insert(obj3);

  // 移除对象触发合并
  tree.remove(obj1);
  tree.remove(obj2);

  // 验证剩余对象仍然可以被找到
  Rect2 fullArea(Vector2(0, 0), Vector2(100, 100));
  auto results = tree.query(fullArea);
  ASSERT_TRUE(results.size() == 1);
  ASSERT_TRUE(results[0] == obj3);
}

TEST(TestAlgoBoids, boidsEdgeCases) {
  // 测试最小边界
  Rect2 smallBoundary(Vector2(0, 0), Vector2(1, 1));
  QuadTree smallTree(smallBoundary, 1);
  auto obj = std::make_shared<QuadTreeObj>(Vector2(0.5, 0.5));
  ASSERT_TRUE(smallTree.insert(obj));

  // 测试边界上的点
  Rect2 boundary(Vector2(0, 0), Vector2(100, 100));
  QuadTree tree(boundary, 2);
  auto boundaryObj = std::make_shared<QuadTreeObj>(Vector2(100, 100));
  ASSERT_TRUE(tree.insert(boundaryObj));

  // 测试空查询区域
  Rect2 emptyArea(Vector2(0, 0), Vector2(0, 0));
  auto results = tree.query(emptyArea);
  ASSERT_TRUE(results.empty());
}

TEST(TestAlgoBoids, boidsItemsUpdate) {
  Rect2 boundary(Vector2(0, 0), Vector2(100, 100));
  std::shared_ptr<QuadTree> tree =
      std::make_shared<QuadTree>(boundary, 10); // 使用更大的容量以测试合并
  std::vector<std::shared_ptr<Boid>> boids;
  for (int i = 0; i < 100; i++) {
    auto obj = std::make_shared<Boid>();
    obj->position =
        Vector2(rand() % int(boundary.size.x), rand() % int(boundary.size.y));
    boids.push_back(obj);
    tree->insert(obj);
  }
  Vector2 target_pos(50, 50);
  for (int i = 0; i < 100; i++) {
    // 100轮次更新
    for (auto item : boids) {
      item->update(10, tree, Vector2(100, 100), target_pos);
    }
  }
}