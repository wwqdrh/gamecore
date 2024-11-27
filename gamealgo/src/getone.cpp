#include "getone.h"

namespace gamealgo {
std::optional<Item>
LootPool::getRandomItem(std::optional<Item> guaranteedItem) const {
  if (guaranteedItem) {
    return guaranteedItem; // 如果触发保底，直接返回稀有物品
  }

  std::uniform_real_distribution<double> dist(0.0, totalProbability);
  double randomValue = dist(rng);

  double cumulativeProbability = 0.0;
  for (const auto &[item, probability] : items) {
    cumulativeProbability += probability;
    if (randomValue <= cumulativeProbability) {
      return item;
    }
  }

  return std::nullopt; // 未成功掉落任何物品
}
} // namespace gamealgo