#include <gtest/gtest.h>

#include "rankboard.h"

using namespace gamealgo;

TEST(TestRankBoard, boardcrud) {
  RankBoard leaderboard;

  // 添加一些玩家数据
  leaderboard.updateScore(1, "Player1", 1000);
  leaderboard.updateScore(2, "Player2", 2000);
  leaderboard.updateScore(3, "Player3", 1500);
  leaderboard.updateScore(4, "Player4", 3000);

  // 获取前3名玩家
  auto top3 = leaderboard.getTopN(3);
  ASSERT_EQ(top3[0].second, 3000);
  ASSERT_EQ(top3[1].second, 2000);
  ASSERT_EQ(top3[2].second, 1500);

  // 获取Player2前后2名的玩家
  auto neighbors = leaderboard.getPlayerNeighbors(2, 2);
  ASSERT_EQ(neighbors[0].second, 3000);
  ASSERT_EQ(neighbors[1].second, 2000);
  ASSERT_EQ(neighbors[2].second, 1500);
  ASSERT_EQ(neighbors[3].second, 1000);

  // 获取Player2的排名
  int rank = leaderboard.getPlayerRank(2);
  ASSERT_EQ(rank, 2);
}