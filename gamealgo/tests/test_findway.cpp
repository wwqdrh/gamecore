#include <gtest/gtest.h>

#include "findway.h"

using namespace gamealgo;

TEST(TestFindWay, findway) {
  FindWay grid(5, 5);

  // 设置一些障碍物
  grid.updatePosition(2, 1, 1);
  grid.updatePosition(2, 2, 1);
  grid.updatePosition(2, 3, 1);

  // 定义起点和终点
  int startX = 0, startY = 0;
  int goalX = 4, goalY = 4;
  // 动态计算路径
  while (startX != goalX || startY != goalY) {
    auto result = grid.next_way(startX, startY, goalX, goalY);
    ASSERT_TRUE(result.has_value());
    // 移动到下一步
    Node nextStep = result.value();
    startX = nextStep.x;
    startY = nextStep.y;
    // 假设地图可能发生变化，这里可以随机更新一些障碍物
    // TODO: 如果地图没有变化那么可以直接使用之前计算过的路径，避免重复计算
    // grid.updatePosition(3, 3, 1); // 动态增加障碍物
    // grid.printGrid();
  }
}