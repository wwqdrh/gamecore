#include "rankboard.h"

namespace gamealgo {
// =====
// skiplist
// =====
void SkipList::insert(int playerId, int64_t score) {
  std::vector<std::shared_ptr<SkipNode>> update(MAX_LEVEL);
  std::shared_ptr<SkipNode> current = header;
  // 从最高层开始查找，加快查找速度
  for (int i = level - 1; i >= 0; i--) {
    while (current->forward[i] != nullptr &&
           (current->forward[i]->score > score ||
            (current->forward[i]->score == score &&
             current->forward[i]->playerId < playerId))) {
      current = current->forward[i];
    }
    update[i] = current;
  }
  current = current->forward[0];

  // 如果玩家已经存在则更新分数
  if (current != nullptr && current->playerId == playerId) {
    if (current->score == score)
      return;
    // 删除旧节点
    for (int i = 0; i < level; i++) {
      if (update[i]->forward[i] != current)
        break;
      update[i]->forward[i] = current->forward[i];
    }
    current.reset();
  }

  // 创建新节点
  int newLevel = randomLevel();
  if (newLevel > level) {
    for (int i = level; i < newLevel; i++) {
      update[i] = header;
    }
    level = newLevel;
  }

  current = std::make_shared<SkipNode>(playerId, score, newLevel);
  for (int i = 0; i < newLevel; i++) {
    current->forward[i] = update[i]->forward[i];
    update[i]->forward[i] = current;
  }
}

std::vector<std::pair<int, int64_t>> SkipList::getRangeByRank(int start,
                                                              int end) {
  std::vector<std::pair<int, int64_t>> result;
  std::shared_ptr<SkipNode> current = header->forward[0];
  int rank = 0;

  while (current != nullptr && rank < end) {
    rank++;
    if (rank >= start) {
      result.emplace_back(current->playerId, current->score);
    }
    current = current->forward[0];
  }

  return result;
}
int64_t SkipList::getPlayerScore(int playerId) {
  std::shared_ptr<SkipNode> current = header;
  for (int i = level - 1; i >= 0; i--) {
    while (current->forward[i] != nullptr &&
           current->forward[i]->playerId < playerId) {
      current = current->forward[i];
    }
  }
  current = current->forward[0];
  return (current != nullptr && current->playerId == playerId) ? current->score
                                                               : -1;
}
int SkipList::getPlayerRank(int playerId) {
  int rank = 0;
  std::shared_ptr<SkipNode> current = header->forward[0];
  while (current != nullptr) {
    if (current->playerId == playerId) {
      return rank + 1;
    }
    rank++;
    current = current->forward[0];
  }
  return -1;
}

// =====
// RankBoard
// =====
void RankBoard::updateScore(int playerId, const std::string &name,
                            int64_t score) {
  playerNames[playerId] = name;
  rankings.insert(playerId, score);
}

std::vector<std::pair<std::string, int64_t>> RankBoard::getTopN(int n) {
  auto topPlayers = rankings.getRangeByRank(1, n);
  std::vector<std::pair<std::string, int64_t>> result;
  for (const auto &player : topPlayers) {
    result.emplace_back(playerNames[player.first], player.second);
  }
  return result;
}

std::vector<std::pair<std::string, int64_t>>
RankBoard::getPlayerNeighbors(int playerId, int n) {
  int rank = rankings.getPlayerRank(playerId);
  if (rank == -1)
    return {};

  int start = std::max(1, rank - n);
  int end = rank + n;
  auto neighbors = rankings.getRangeByRank(start, end);

  std::vector<std::pair<std::string, int64_t>> result;
  for (const auto &player : neighbors) {
    result.emplace_back(playerNames[player.first], player.second);
  }
  return result;
}
} // namespace gamealgo