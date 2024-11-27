#pragma once
#include <cmath>
#include <cstdlib>
#include <ctime>
#include <iostream>
#include <vector>
namespace gamealgo {
// 1. 系统设计
// 敌人生成与平衡系统通常需要考虑以下几个方面：
// 敌人种类：每种敌人具有不同的属性，比如生命值、攻击力、移动速度等。
// 生成机制：控制敌人生成的数量、时间和位置。
// 平衡机制：随着游戏进展或玩家等级的提升，敌人的强度应该逐步增加，但也不能过于强大，避免玩家无法应对。平衡机制应基于玩家的能力或游戏进度动态调整敌人属性。
// 2. 关键设计要素
// 敌人属性：包括生命值、攻击力、防御力、速度等。
// 难度因子：游戏难度随着时间、玩家等级或其他条件的变化而逐渐上升。
// 敌人生成策略：包括生成的敌人数、种类和位置。
// 动态调整：根据游戏状态和玩家表现实时调整生成的敌人属性，确保平衡。
// 敌人类
class Enemy {
public:
  std::string type;
  int health;
  int attackPower;
  int defense;
  double speed;

public:
  Enemy(std::string t, int h, int a, int d, double s)
      : type(t), health(h), attackPower(a), defense(d), speed(s) {}

  // 静态随机生成这个类
  static Enemy random(int difficultyLevel) {
    // 敌人类型
    std::vector<std::string> enemyTypes = {"Goblin", "Orc", "Troll", "Dragon"};

    // 根据难度等级动态调整敌人属性
    int baseHealth = 50 + difficultyLevel * 10; // 难度越高，生命值越高
    int baseAttack = 10 + difficultyLevel * 5;  // 攻击力随难度提升
    int baseDefense = 5 + difficultyLevel * 3;  // 防御随难度提升
    double baseSpeed = 1.0 + (difficultyLevel * 0.1); // 速度轻微随难度增加

    // 随机选择一个敌人类型
    std::string enemyType = enemyTypes[rand() % enemyTypes.size()];

    return Enemy(enemyType, baseHealth, baseAttack, baseDefense, baseSpeed);
  }

public:
  void print() const {
    std::cout << "Type: " << type << ", Health: " << health
              << ", Attack Power: " << attackPower << ", Defense: " << defense
              << ", Speed: " << speed << std::endl;
  }
};

// 敌人生成与平衡系统类
class EnemyGenerator {
public:
  int difficultyLevel;
  std::vector<Enemy> enemies;

  EnemyGenerator(int level) : difficultyLevel(level) {}

  // 生成敌人波次
  void generateEnemies(int waveNumber) {
    int enemyCount = std::min(5 + waveNumber,
                              20); // 每波生成的敌人数随波次增加，但限制在20个

    for (int i = 0; i < enemyCount; ++i) {
      // 根据当前难度等级生成敌人
      enemies.push_back(Enemy::random(difficultyLevel + waveNumber / 2));
    }
  }

  // 提升难度
  void increaseDifficulty() {
    difficultyLevel += 1;
  }

  // 显示所有敌人信息
  void printEnemies() const {
    for (const auto &enemy : enemies) {
      enemy.print();
    }
  }
};
} // namespace gamealgo