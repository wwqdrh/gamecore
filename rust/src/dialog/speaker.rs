// GdRoleSpeaker - 人物与对话角色绑定
//
// 挂在人物节点（GdRoleMover）下，把 timeline 中的角色名绑定到当前人物。
// 对话触发器（GdDialogTrigger）启动对话时，会以 speaker 的 role_name
// 把宿主人物注册进 GdDialogue（register_role_node），对话框即可
// 通过角色名定位人物节点（站位、朝向、动画等）。
//
// 节点结构：
//   NPC (GdRoleMover)
//   ├── AnimatedSprite2D / Animator / Brain ...
//   ├── Speaker (GdRoleSpeaker)    <- role_name = timeline 中的角色名
//   └── Trigger (GdDialogTrigger)  <- 触发配置
//
// 玩家同样需要一个 Speaker（role_name 对应 timeline 里的玩家角色名），
// 对话触发时触发器会同时注册双方。

use godot::prelude::*;
use godot::builtin::GString;
use godot::classes::{INode, Node};

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdRoleSpeaker {
    /// timeline 中的角色名（需与时间线 (角色名) 声明一致）
    #[export]
    role_name: GString,

    /// 展示名（可选，供 UI 替代角色名显示，如头像旁的昵称）
    #[export]
    display_name: GString,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdRoleSpeaker {
    fn init(base: Base<Node>) -> Self {
        GdRoleSpeaker {
            role_name: GString::new(),
            display_name: GString::new(),
            base,
        }
    }
}

#[godot_api]
impl GdRoleSpeaker {
    // role_name / display_name 的 getter 由 #[export] 自动生成
}
