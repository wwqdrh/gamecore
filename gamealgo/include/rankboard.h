#pragma once

#include <algorithm>
#include <iostream>
#include <memory>
#include <random>
#include <unordered_map>
#include <utility>
#include <vector>

#include "cross.h"

namespace gamealgo {
// 排行榜涉及排序和排名算法，通常按玩家得分、完成时间或其他指标排序。
// 底层使用快表数据结构，相较于普通链表有较快查找速度，并且维护排序结构
// TODO: 支持按照多种指标进行排序
struct SkipNode {
  int playerId;
  int64_t score;
  std::vector<std::shared_ptr<SkipNode>> forward;

  SkipNode(int pid, int64_t s, int level)
      : playerId(pid), score(s), forward(level, nullptr) {}
};

class SkipList {
private:
  static inline const int MAX_LEVEL = 16; // 最大层数
  static inline constexpr float P = 0.5f; // 层数增加的概率

  std::shared_ptr<SkipNode> header;
  int level;
  std::random_device rd;
  std::mt19937 gen;
  std::uniform_real_distribution<> dis;

  // 生成随机层数
  int randomLevel() {
    int lvl = 1;
    while (dis(gen) < P && lvl < MAX_LEVEL) {
      lvl++;
    }
    return lvl;
  }

public:
  SkipList() : level(1), gen(rd()), dis(0, 1) {
    header = std::make_shared<SkipNode>(0, INT64_MAX, MAX_LEVEL);
  }

  void insert(int playerId, int64_t score);

  std::vector<std::pair<int, int64_t>> getRangeByRank(int start, int end);

  int64_t getPlayerScore(int playerId);

  int getPlayerRank(int playerId);
};

// 排行榜类
class RankBoard {
private:
  SkipList rankings;
  std::unordered_map<int, std::string> playerNames; // 玩家ID到名字的映射

public:
  // 更新玩家分数
  void updateScore(int playerId, const std::string &name, int64_t score);

  // 获取前N名玩家
  std::vector<std::pair<std::string, int64_t>> getTopN(int n);

  // 获取某个玩家前后N名玩家
  std::vector<std::pair<std::string, int64_t>> getPlayerNeighbors(int playerId,
                                                                  int n);

  // 获取玩家排名
  int getPlayerRank(int playerId) { return rankings.getPlayerRank(playerId); }
};

} // namespace gamealgo