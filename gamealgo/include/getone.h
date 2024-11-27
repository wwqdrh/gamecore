#pragma once

#include <iostream>
#include <numeric>
#include <optional>
#include <random>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace gamealgo {
// 假设游戏中的物品有不同的掉落概率，且存在稀有物品的保底机制，即在多次未掉落稀有物品后，强制掉落一个稀有物品。
// 核心组件
// 物品类（Item）：定义物品的基本属性，如名称和稀有度。
// 掉落池（LootPool）：存储不同物品及其掉落概率。
// 保底机制（PitySystem）：确保在一定次数后必定掉落稀有物品。
// 物品类
class Item {
public:
  std::string name;
  int rarity; // 稀有度，值越大越稀有

  Item(std::string name, int rarity) : name(std::move(name)), rarity(rarity) {}
};

// 掉落池，存储物品和它们的掉落概率
class LootPool {
private:
  std::vector<std::pair<Item, double>> items;
  double totalProbability = 0.0;
  mutable std::mt19937 rng; // 随机数生成器

  // 按稀有度排序的比较函数
  static bool compareByRarity(const std::pair<Item, double> &a,
                              const std::pair<Item, double> &b) {
    return a.first.rarity > b.first.rarity;
  }

public:
  LootPool() : rng(std::random_device{}()) {}

  // 添加物品及其概率
  // 需要按照稀有度排序
  void addItem(const Item &item, double probability) {
    items.emplace_back(item, probability);
    totalProbability += probability;
    // 添加后按稀有度排序
    std::sort(items.begin(), items.end(), compareByRarity);
  }

  void removeItem(const Item &item) {
    for (auto it = items.begin(); it != items.end(); ++it) {
      if (it->first.name == item.name) {
        items.erase(it);
        totalProbability -= it->first.rarity;
        break;
      }
    }
  }

  // 从掉落池中随机选择物品，保底机制作为可选参数
  std::optional<Item>
  getRandomItem(std::optional<Item> guaranteedItem = std::nullopt) const;
};

// 保底系统
class PitySystem {
private:
  int rollsWithoutRare = 0; // 记录连续未掉落稀有物品的次数
  const int pityThreshold;  // 保底次数阈值
  const Item rareItem;      // 保底掉落的稀有物品

public:
  PitySystem(int threshold, const Item &rareItem)
      : pityThreshold(threshold), rareItem(rareItem) {}

  // 检查是否触发保底
  std::optional<Item> checkPity() {
    if (rollsWithoutRare >= pityThreshold) {
      rollsWithoutRare = 0; // 触发保底后重置计数
      return rareItem;
    }
    return std::nullopt;
  }

  // 更新保底系统状态
  void updatePity(const Item &item) {
    if (item.rarity <= 5) { // 假设稀有度 >= 5 是稀有物品
      rollsWithoutRare++;
    } else {
      rollsWithoutRare = 0; // 掉落稀有物品后重置计数
    }
  }
};
} // namespace gamealgo