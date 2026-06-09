// gamedialog: 游戏对话脚本引擎
//
// 解析和执行结构化对话脚本，支持多角色对话、分支控制流（跳转/条件/循环）、
// 场景变量、全局变量、条件入口等特性
//
// 子模块：
// - word: DialogueWord 对话词条（说话人、文本、选项、触发函数）
// - flow: ControlFlow 控制流体系（Start/End/Skip/Goto）
// - stage: DiaStage 场景阶段（核心解析和执行单元）
// - timeline: Timeline 时间线（管理 DiaStage 序列，提供全局导航）
// - scene_manager: SceneManager 场景管理器（管理多个 Timeline 和全局变量）

pub mod word;
pub mod flow;
pub mod stage;
pub mod timeline;
pub mod scene_manager;

pub use word::DialogueWord;
pub use flow::ControlFlow;
pub use stage::DiaStage;
pub use timeline::Timeline;
pub use scene_manager::SceneManager;
