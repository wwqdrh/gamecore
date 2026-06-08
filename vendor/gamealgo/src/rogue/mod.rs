// rogue: 肉鸽游戏核心算法模块
//
// 包含 10 个子模块和一个种子管理上下文 RogueContext
// RogueContext 是整个库的入口，设置种子后所有随机内容均可复现

pub mod rng;
pub mod dungeon;
pub mod pathfind;
pub mod fov;
pub mod loot;
pub mod stats;
pub mod encounter;
pub mod progression;
pub mod entity;
pub mod cardpile;
pub mod context;

pub use rng::GameRng;
pub use dungeon::{
    DungeonMap, DungeonConfig, DungeonGenerator, Tile, Rect, Room, RoomType, Corridor,
    BspGenerator, CellularAutomataGenerator, RandomWalkGenerator, RoomCorridorGenerator,
};
pub use pathfind::{PathFinder, PathConfig, Heuristic, AStarFinder, DijkstraFinder, JpsFinder};
pub use fov::{FovAlgorithm, FovResult, RaycastingFov, ShadowcastingFov};
pub use loot::{LootTable, LootEntry, LootCondition, LootContext};
pub use stats::{StatSystem, Modifier, ModifierOp, ModifierSource, GrowthCurve, DamageFormula};
pub use encounter::{
    EncounterSystem, EncounterTable, EncounterEntry, Encounter, EncounterType,
    EnemySpawn, DifficultyCurve,
};
pub use progression::{
    ProgressionSystem, RelicDef, RelicTier, RelicEffect, Synergy, SynergyGraph,
};
pub use entity::{EntityTemplate, EntityStats, EntityPool, EntityType, StatScale};
pub use cardpile::{Card, CardPile, CardPileConfig, CardPileLayout, generate_card_piles};
pub use context::RogueContext;
