# gamealgo

## 项目简介

肉鸽（Roguelike/Roguelite）游戏核心算法库，Rust 实现。提供可复现的随机数生成、地牢地图生成、寻路、视野计算、掉落表、数值系统、遭遇分配、遗物成长、过程化实体生成和卡堆系统等核心算法，方便其他项目快速搭建肉鸽游戏。

## 核心设计

- **RogueContext**: 种子管理上下文，整个库的入口。设置种子后，所有随机内容均可复现。内部为每个子系统派生独立子 RNG，互不干扰
- **GameRng**: 基于 Xoshiro256++ 的可控随机数生成器，支持种子复现、权重随机、fork 子 RNG、分布采样
- **DungeonMap**: 统一地图结构，4 种生成算法（BSP、细胞自动机、随机游走、房间走廊）
- **PathFinder**: A*、Dijkstra、JPS 三种寻路算法，支持四方向/八方向、自定义代价
- **FovAlgorithm**: Raycasting 和 Shadowcasting 两种视野算法，支持战争迷雾
- **LootTable**: 掉落表系统，支持权重、条件逻辑、唯一物品、层数限制、嵌套表
- **StatSystem**: 数值系统，支持 Add/Multiply/Override/Min/Max 五种 Modifier 运算、可叠加 Buff、成长曲线、伤害公式
- **EncounterSystem**: 遭遇系统，支持按房间类型和层数分配遭遇、难度曲线、敌人配置生成
- **ProgressionSystem**: 遗物系统，支持效果注册、协同效果、互斥检查、解锁条件
- **EntityPool**: 过程化实体生成，通过模板+缩放曲线按深度自动计算数值，无需逐级定义
- **CardPileLayout**: 卡堆生成系统，管理卡堆摆放、内容分配、出口隐藏、堆叠规则

## 种子管理

`RogueContext` 是整个库的种子管理中心：

```rust
use gamealgo::*;

let mut ctx = RogueContext::new(42);
```

内部为 7 个域各派生一个独立子 RNG：
- `master`: 主 RNG，用于派生新域
- `dungeon`: 地牢生成
- `loot`: 掉落计算
- `encounter`: 遭遇分配
- `entity`: 实体生成
- `cardpile`: 卡堆生成
- `misc`: 其他随机需求

支持快照/恢复（`snapshot()`/`restore()`），可用于存档/读档。

## 过程化实体生成

不再需要为每个难度逐级定义怪物数值，只需定义模板和缩放规则：

```rust
let mut pool = EntityPool::new();

pool.register(
    EntityTemplate::new("goblin", "哥布林", EntityType::Monster)
        .with_stat("hp", StatScale::linear(20.0, 5.0, 0.1))   // 基础20，每层+5，±10%波动
        .with_stat("atk", StatScale::linear(5.0, 1.5, 0.1))    // 基础5，每层+1.5
        .with_stat("def", StatScale::fixed(2.0))                // 固定2
        .with_weight(5.0)                                        // 出现权重
        .with_depth_range(1, usize::MAX),                        // 深度1起出现
);

// 深度1: hp≈20, atk≈5
// 深度5: hp≈40, atk≈11
// 深度10: hp≈65, atk≈18.5
// 每次生成还有随机波动，同种子可复现
```

## 卡堆系统

```rust
let config = CardPileConfig {
    pile_count: 3,           // 卡堆数量（不固定）
    cards_per_pile: 3..6,    // 每堆3-5张牌
    type_weights: vec![       // 类型权重（可自定义扩展）
        ("monster".to_string(), 5.0),
        ("weapon".to_string(), 2.0),
        ("armor".to_string(), 2.0),
        ("item".to_string(), 3.0),
    ],
    spacing: 200.0,          // 卡堆间距
};

let layout = ctx.generate_card_piles(&pool, &config);
```

## 依赖

- `serde` + `serde_json` — JSON 配置序列化
- `thiserror` — 错误处理

## 编译与测试

- 编译：`cargo build`
- 测试：`cargo test`
