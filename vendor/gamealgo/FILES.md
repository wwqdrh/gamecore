# 功能文件索引

## 核心文件

- **src/lib.rs**：库入口，声明 rogue 模块并重导出所有公共类型
- **src/rogue/mod.rs**：rogue 模块入口，声明子模块并重导出
- **src/rogue/context.rs**：种子管理上下文 RogueContext，统一管理各子系统 RNG，支持快照/恢复
- **src/rogue/rng.rs**：可控随机数生成器（Xoshiro256++），支持种子、权重、分布、fork
- **src/rogue/dungeon.rs**：地牢/地图生成，4 种算法（BSP、细胞自动机、随机游走、房间走廊），统一 DungeonMap 输出
- **src/rogue/pathfind.rs**：寻路算法（A*、Dijkstra、JPS），支持四方向/八方向、自定义代价、多种启发函数
- **src/rogue/fov.rs**：视野/战争迷雾计算（Raycasting、Shadowcasting），支持已探索区域追踪
- **src/rogue/loot.rs**：战利品/掉落表系统，支持权重、条件逻辑、唯一物品、层数限制、嵌套表
- **src/rogue/stats.rs**：数值系统，Modifier 五种运算、可叠加 Buff、成长曲线、伤害公式
- **src/rogue/encounter.rs**：遭遇/事件系统，按房间类型和层数分配遭遇、难度曲线、敌人配置生成
- **src/rogue/progression.rs**：局外成长/遗物系统，效果注册、协同效果、互斥检查、解锁条件
- **src/rogue/entity.rs**：过程化实体生成，模板+缩放曲线按深度自动计算数值，支持权重随机、标签筛选
- **src/rogue/cardpile.rs**：卡堆生成系统，管理卡堆摆放、内容分配、出口隐藏、堆叠规则
- **Cargo.toml**：项目配置文件，定义依赖关系

## 示例程序

- **examples/cli_game.rs**：CLI 卡牌肉鸽游戏示例，输入数字选择牌堆操作，包含战斗、装备、药水、移动牌、找出口

## 文档文件

- **PROJECT.md**：项目总结文档
- **FILES.md**：功能文件索引

## 测试

- 各模块内嵌单元测试（`#[cfg(test)]`），共 100 个测试用例
