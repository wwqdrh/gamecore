// gamealgo: 肉鸽游戏核心算法库
//
// 所有算法模块位于 rogue/ 子目录下
// RogueContext 是库的入口，设置种子后所有随机内容均可复现
//
// 子模块：
// - rng: 可控随机数生成器，支持种子、权重、加权随机、洗牌、分布控制
// - dungeon: 地牢/地图生成（BSP、Cellular Automata、Random Walk、房间走廊）
// - pathfind: 寻路算法（A*、Dijkstra、JPS），支持网格和自定义代价
// - fov: 视野/战争迷雾计算（Raycasting、Shadowcasting）
// - loot: 战利品/掉落表系统，支持权重、条件、稀有度层级
// - stats: 数值系统（属性计算、Buff/Debuff、伤害公式、成长曲线）
// - encounter: 遭遇/事件系统（房间事件分配、难度曲线、敌人配置生成）
// - progression: 局外成长/遗物系统（遗物效果注册、协同、解锁条件）
// - entity: 过程化实体生成（模板+缩放曲线，按深度自动计算数值）
// - cardpile: 卡堆生成系统（卡堆摆放、内容分配、出口隐藏、堆叠规则）
// - context: 种子管理上下文 RogueContext

pub mod rogue;

pub use rogue::{
    GameRng, RogueContext,
    DungeonMap, DungeonConfig, DungeonGenerator, Tile, Rect, Room, RoomType, Corridor,
    BspGenerator, CellularAutomataGenerator, RandomWalkGenerator, RoomCorridorGenerator,
    PathFinder, PathConfig, Heuristic, AStarFinder, DijkstraFinder, JpsFinder,
    FovAlgorithm, FovResult, RaycastingFov, ShadowcastingFov,
    LootTable, LootEntry, LootCondition, LootContext,
    StatSystem, Modifier, ModifierOp, ModifierSource, GrowthCurve, DamageFormula,
    EncounterSystem, EncounterTable, EncounterEntry, Encounter, EncounterType, EnemySpawn, DifficultyCurve,
    ProgressionSystem, RelicDef, RelicTier, RelicEffect, Synergy, SynergyGraph,
    EntityTemplate, EntityStats, EntityPool, EntityType, StatScale,
    Card, CardPile, CardPileConfig, CardPileLayout, generate_card_piles,
};
