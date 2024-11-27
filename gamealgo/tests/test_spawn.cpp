#include <gtest/gtest.h>

#include "spawn.h"

using namespace gamealgo;

TEST(TestSpawn, testspawnenemy) {
    srand(static_cast<unsigned>(time(0)));  // 初始化随机数种子

    // 创建敌人生成系统，初始难度等级为1
    EnemyGenerator generator(1);

    // 模拟多个波次的敌人生成
    for (int wave = 1; wave <= 5; ++wave) {
        generator.generateEnemies(wave);
        // generator.printEnemies();

        // 每两波次提升一次难度
        if (wave % 2 == 0) {
            generator.increaseDifficulty();
        }
    }
}