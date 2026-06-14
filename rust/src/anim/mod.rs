// 动画模块
// 移植自 C++ juice 库，提供丰富的 UI 动画效果
// 包含：缓动函数、核心动画、平滑移动、过渡效果

pub mod easing;
pub mod juice;
pub mod easy_move;
pub mod transition;

pub use easing::Easing;
pub use juice::*;
pub use easy_move::EaseMover;
pub use transition::{TransitionFade, FadeType};
