// dialog: 对话系统
//
// 将 gamedialog 的对话引擎暴露给 Godot，包含对话全链路三个类：
//   GdDialogue       对话控制节点，管理 Timeline 与对话推进
//   GdRoleSpeaker    人物与对话角色绑定（挂在人物节点下）
//   GdDialogTrigger  对话触发器（接近/交互/自动/手动四种触发方式）

mod gddialogue;
mod speaker;
mod dialog_trigger;

pub use gddialogue::GdDialogue;
pub use speaker::GdRoleSpeaker;
pub use dialog_trigger::{
    GdDialogTrigger,
    TRIGGER_PROXIMITY, TRIGGER_INTERACT, TRIGGER_AUTO, TRIGGER_MANUAL,
};
