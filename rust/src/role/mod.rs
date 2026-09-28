// 角色模块
// 提供开箱即用的游戏角色移动控制、动画管理与 NPC AI 行为
// 三个类配合使用：节点结构如下
//
//   Player (GdRoleMover)            <- CharacterBody2D，移动控制
//   ├── AnimatedSprite2D / Sprite2D <- 精灵
//   └── Animator (GdRoleAnimator)   <- 自动驱动精灵动画
//   └── Brain (GdNpcBrain)          <- NPC 才需要，行为决策
//
// 典型用法：
//   - 玩家：GdRoleMover(control_mode=键盘/鼠标) + GdRoleAnimator
//   - NPC：GdRoleMover(control_mode=AI) + GdRoleAnimator + GdNpcBrain(behavior=游走/巡逻/跟随)

pub mod movement;
pub mod animator;
pub mod npc_ai;
pub mod speaker;
pub mod dialog_trigger;

pub use movement::{
    GdRoleMover,
    CONTROL_NONE, CONTROL_KEYBOARD, CONTROL_MOUSE, CONTROL_AI,
    MODE_FOUR_WAY, MODE_HORIZONTAL,
};
pub use animator::GdRoleAnimator;
pub use npc_ai::{
    GdNpcBrain,
    AI_IDLE, AI_WANDER, AI_PATROL, AI_FOLLOW, AI_FLEE,
};
pub use speaker::GdRoleSpeaker;
pub use dialog_trigger::{
    GdDialogTrigger,
    TRIGGER_PROXIMITY, TRIGGER_INTERACT, TRIGGER_AUTO, TRIGGER_MANUAL,
};
