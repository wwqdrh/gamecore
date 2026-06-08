// rogue: Godot 暴露层
//
// 将 gamealgo 的肉鸽核心算法暴露给 Godot
// RogueEngine 为核心单例，支持 JSON 配置初始化

mod engine;
mod card;
mod card_pile;

pub use engine::RogueEngine;
pub use card::RogueCard;
pub use card_pile::RogueCardPile;
