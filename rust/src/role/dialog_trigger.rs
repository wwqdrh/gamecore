// GdDialogTrigger - 对话触发器
//
// 挂在人物节点（GdRoleMover）下，负责在合适的时机启动一场对话：
//   - TRIGGER_PROXIMITY  玩家进入 trigger_radius 自动触发；停留期间条件若变满足会自动重试
//   - TRIGGER_INTERACT   玩家在范围内按下交互键触发（interact 动作，InputMap 缺失回退 E 键）
//   - TRIGGER_AUTO       节点就绪 auto_delay 秒后自动触发（NPC 自言自语 / 剧情演出）
//   - TRIGGER_MANUAL     仅响应外部 start_dialog() 调用（剧情脚本 / 信号驱动）
//
// 触发条件门控：condition_fn = "方法名[:参数1,参数2]"，依次在
// 触发器自身 -> 宿主人物 -> 对话 control 节点（如 DialogBox.has_flag）上
// 查找并调用，返回 falsy 则不触发。配合 timeline 内 @set_flag 函数可实现
// "对话内容改变状态 -> 影响后续触发" 的闭环。
//
// 对话进行中（pause_roles = true）：
//   - 暂停宿主与玩家的移动（GdRoleMover.set_paused）
//   - 双方互相面向对方（set_facing）
//   - 对话结束自动恢复；同一场对话期间不会重复触发
//
// 角色解析：player_path > "player" 分组 > 宿主的兄弟节点中第一个 GdRoleMover
// 对话解析：dialogue_path > 宿主子节点中的 GdDialogue
// 双方节点下的 GdRoleSpeaker 会被自动注册进 GdDialogue。
//
// 信号：
//   s_trigger_enter()   玩家进入触发范围
//   s_trigger_exit()    玩家离开触发范围
//   s_dialog_started()  对话开始
//   s_dialog_ended()    对话结束

use godot::prelude::*;
use godot::builtin::{GString, NodePath, VarArray, Vector2};
use godot::classes::{INode, Input, InputMap, Node, Node2D};
use godot::global::Key;

use super::movement::GdRoleMover;
use super::speaker::GdRoleSpeaker;
use crate::dialog::GdDialogue;

/// 触发模式：玩家进入范围自动触发
pub const TRIGGER_PROXIMITY: i64 = 0;
/// 触发模式：范围内按交互键触发
pub const TRIGGER_INTERACT: i64 = 1;
/// 触发模式：就绪后延迟自动触发
pub const TRIGGER_AUTO: i64 = 2;
/// 触发模式：仅外部调用 start_dialog 触发
pub const TRIGGER_MANUAL: i64 = 3;

/// proximity 模式下停留在范围内时的条件重试间隔（秒）
const RETRY_INTERVAL: f64 = 0.5;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdDialogTrigger {
    /// 触发模式 TRIGGER_PROXIMITY / TRIGGER_INTERACT / TRIGGER_AUTO / TRIGGER_MANUAL
    #[export]
    trigger_mode: i64,

    /// 触发范围半径（像素，PROXIMITY / INTERACT 模式）
    #[export]
    trigger_radius: f64,

    /// 交互动作名（InputMap 缺失时回退 E 键，INTERACT 模式）
    #[export]
    interact_action: GString,

    /// 自动触发延迟（秒，AUTO 模式）
    #[export]
    auto_delay: f64,

    /// 是否只触发一次
    #[export]
    trigger_once: bool,

    /// 触发冷却（秒，对话结束后开始计时）
    #[export]
    cooldown: f64,

    /// 对话期间暂停双方移动并互相面向
    #[export]
    pause_roles: bool,

    /// GdDialogue 节点路径（为空时在宿主子节点中查找）
    #[export]
    dialogue_path: NodePath,

    /// 玩家节点路径（为空时按 "player" 分组 > 兄弟节点 GdRoleMover 查找）
    #[export]
    player_path: NodePath,

    /// 对话起始 stage（空 = timeline 开头）
    #[export]
    entry_stage: GString,

    /// 触发条件："方法名[:参数1,参数2]"，返回 falsy 则不触发
    #[export]
    condition_fn: GString,

    // ---- 运行时状态 ----
    cooldown_timer: f64,
    auto_timer: f64,
    auto_fired: bool,
    retry_timer: f64,
    was_inside: bool,
    interact_held_prev: bool,
    dialog_active: bool,
    fired_once: bool,
    /// 对话结束后要求玩家先离开触发范围才重新武装（仅 PROXIMITY）
    rearm_on_exit: bool,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdDialogTrigger {
    fn init(base: Base<Node>) -> Self {
        GdDialogTrigger {
            trigger_mode: TRIGGER_PROXIMITY,
            trigger_radius: 90.0,
            interact_action: GString::from("interact"),
            auto_delay: 1.0,
            trigger_once: false,
            cooldown: 0.0,
            pause_roles: true,
            dialogue_path: NodePath::default(),
            player_path: NodePath::default(),
            entry_stage: GString::new(),
            condition_fn: GString::new(),
            cooldown_timer: 0.0,
            auto_timer: 0.0,
            auto_fired: false,
            retry_timer: 0.0,
            was_inside: false,
            interact_held_prev: false,
            dialog_active: false,
            fired_once: false,
            rearm_on_exit: false,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // 对话进行中：轮询 GdDialogue 播完（is_playing=false）后收尾
        if self.dialog_active {
            let finished = match self.resolve_dialogue() {
                Some(mut dia) => !dia.bind().is_playing(),
                None => true,
            };
            if finished {
                self.end_dialog();
            }
            return;
        }

        if self.cooldown_timer > 0.0 {
            self.cooldown_timer -= delta;
        }

        match self.trigger_mode {
            TRIGGER_PROXIMITY => {
                let inside = self.player_in_range();
                self.emit_range_edges(inside);
                if self.rearm_on_exit {
                    // 上一场对话刚结束：等玩家走出范围再重新武装，避免原地立刻重触发
                    if !inside {
                        self.rearm_on_exit = false;
                    }
                } else if inside {
                    self.retry_timer -= delta;
                    if self.retry_timer <= 0.0 {
                        self.retry_timer = RETRY_INTERVAL;
                        self.start_dialog();
                    }
                }
            }
            TRIGGER_INTERACT => {
                let inside = self.player_in_range();
                self.emit_range_edges(inside);
                let held = self.interact_held();
                if inside && held && !self.interact_held_prev {
                    self.start_dialog();
                }
                self.interact_held_prev = held;
            }
            TRIGGER_AUTO => {
                self.auto_timer += delta;
                if !self.auto_fired && self.auto_timer >= self.auto_delay {
                    self.auto_fired = true;
                    self.start_dialog();
                }
            }
            _ => {}
        }
    }
}

#[godot_api]
impl GdDialogTrigger {
    #[signal]
    fn s_trigger_enter();

    #[signal]
    fn s_trigger_exit();

    #[signal]
    fn s_dialog_started();

    #[signal]
    fn s_dialog_ended();

    /// 当前是否在对话中
    #[func]
    pub fn is_dialog_active(&self) -> bool {
        self.dialog_active
    }

    /// 手动触发对话（MANUAL 模式入口；同样受 once/cooldown/condition 约束）
    #[func]
    pub fn start_dialog(&mut self) -> bool {
        if self.dialog_active {
            return false;
        }
        if self.trigger_once && self.fired_once {
            return false;
        }
        if self.cooldown_timer > 0.0 {
            return false;
        }
        let mut dia = match self.resolve_dialogue() {
            Some(d) => d,
            None => return false,
        };
        if dia.bind().is_playing() {
            return false;
        }
        if !self.check_condition(&mut dia) {
            return false;
        }

        let host = self.base().get_parent();
        let player = self.resolve_player();

        // 注册宿主与玩家的对话角色（GdRoleSpeaker 提供角色名）
        if let Some(ref host) = host {
            if let Some(sp) = Self::find_speaker(host) {
                let name = sp.bind().get_role_name();
                if !name.is_empty() {
                    dia.bind_mut().register_role_node(name, host.clone());
                }
            }
        }
        if let Some(ref p) = player {
            if let Some(sp) = Self::find_speaker(p) {
                let name = sp.bind().get_role_name();
                if !name.is_empty() {
                    dia.bind_mut().register_role_node(name, p.clone());
                }
            }
        }

        // 暂停双方移动并互相面向对方
        if self.pause_roles {
            let host_mover = host.as_ref().and_then(|h| h.clone().try_cast::<GdRoleMover>().ok());
            let player_mover = player.as_ref().and_then(|p| p.clone().try_cast::<GdRoleMover>().ok());
            let host_pos = host.as_ref().and_then(|h| Self::pos_of(h)).unwrap_or(Vector2::ZERO);
            let player_pos = player.as_ref().and_then(|p| Self::pos_of(p)).unwrap_or(Vector2::ZERO);
            if let Some(mut hm) = host_mover {
                hm.bind_mut().set_paused(true);
                Self::face(&mut hm, player_pos);
            }
            if let Some(mut pm) = player_mover {
                pm.bind_mut().set_paused(true);
                Self::face(&mut pm, host_pos);
            }
        }

        // 重置 timeline 位置：有 entry_stage 跳到指定 stage，
        // 否则回到开头，保证 NPC 可重复触发完整对话
        if !self.entry_stage.is_empty() {
            dia.bind_mut().goto_stage(self.entry_stage.clone());
        } else {
            let stages = dia.bind_mut().all_stages();
            if let Some(first) = stages.as_slice().first() {
                dia.bind_mut().goto_stage(first.clone());
            }
        }
        dia.bind_mut().next(GString::new());

        self.dialog_active = true;
        self.fired_once = true;
        self.base_mut().emit_signal("s_dialog_started", &[]);
        true
    }

    /// 强制结束当前对话（立即恢复双方移动）
    #[func]
    pub fn cancel_dialog(&mut self) {
        if self.dialog_active {
            self.end_dialog();
        }
    }

    // ---- 内部实现 ----

    /// 解析 GdDialogue：dialogue_path 优先，回退宿主子节点
    fn resolve_dialogue(&self) -> Option<Gd<GdDialogue>> {
        if !self.dialogue_path.is_empty() {
            if let Some(node) = self.base().get_node_or_null(&self.dialogue_path) {
                return node.try_cast::<GdDialogue>().ok();
            }
            return None;
        }
        let host = self.base().get_parent()?;
        for i in 0..host.get_child_count() {
            if let Some(child) = host.get_child(i) {
                if let Ok(d) = child.try_cast::<GdDialogue>() {
                    return Some(d);
                }
            }
        }
        None
    }

    /// 解析玩家：player_path > "player" 分组 > 宿主兄弟节点中第一个 GdRoleMover
    fn resolve_player(&self) -> Option<Gd<Node>> {
        if !self.player_path.is_empty() {
            return self.base().get_node_or_null(&self.player_path);
        }
        let tree = self.base().get_tree();
        if let Some(n) = tree.get_first_node_in_group("player") {
            return Some(n);
        }
        let host = self.base().get_parent()?;
        let container = host.get_parent()?;
        for i in 0..container.get_child_count() {
            let Some(child) = container.get_child(i) else {
                continue;
            };
            if child.instance_id() == host.instance_id() {
                continue;
            }
            if child.clone().try_cast::<GdRoleMover>().is_ok() {
                return Some(child);
            }
        }
        None
    }

    /// 在节点子级中查找 GdRoleSpeaker
    fn find_speaker(node: &Gd<Node>) -> Option<Gd<GdRoleSpeaker>> {
        for i in 0..node.get_child_count() {
            if let Some(child) = node.get_child(i) {
                if let Ok(sp) = child.try_cast::<GdRoleSpeaker>() {
                    return Some(sp);
                }
            }
        }
        None
    }

    /// 玩家是否在触发范围内
    fn player_in_range(&self) -> bool {
        let Some(host) = self.base().get_parent() else {
            return false;
        };
        let Some(player) = self.resolve_player() else {
            return false;
        };
        let (Some(hp), Some(pp)) = (Self::pos_of(&host), Self::pos_of(&player)) else {
            return false;
        };
        hp.distance_to(pp) <= self.trigger_radius as f32
    }

    /// 玩家进出范围的边沿信号
    fn emit_range_edges(&mut self, inside: bool) {
        if inside && !self.was_inside {
            self.base_mut().emit_signal("s_trigger_enter", &[]);
        } else if !inside && self.was_inside {
            self.base_mut().emit_signal("s_trigger_exit", &[]);
        }
        self.was_inside = inside;
    }

    /// 交互键检测：动作优先，回退 E 键
    fn interact_held(&self) -> bool {
        let input = Input::singleton();
        let sn = StringName::from(&self.interact_action);
        if InputMap::singleton().has_action(&sn) {
            input.is_action_pressed(&sn)
        } else {
            input.is_physical_key_pressed(Key::E)
        }
    }

    /// 条件门控：依次在触发器自身 -> 宿主人物 -> 对话 control 上查找方法
    fn check_condition(&mut self, dia: &mut Gd<GdDialogue>) -> bool {
        let expr = self.condition_fn.to_string();
        if expr.is_empty() {
            return true;
        }
        let parts: Vec<&str> = expr.splitn(2, ':').collect();
        let name = parts[0].trim();
        if name.is_empty() {
            return true;
        }
        let args: VarArray = parts
            .get(1)
            .map(|s| {
                s.split(',')
                    .filter(|a| !a.trim().is_empty())
                    .map(|a| a.trim().to_variant())
                    .collect()
            })
            .unwrap_or_default();

        // 1) 触发器自身
        if self.base().has_method(name) {
            return Self::truthy(self.base_mut().callv(name, &args));
        }
        // 2) 宿主人物节点
        if let Some(mut host) = self.base().get_parent() {
            if host.has_method(name) {
                return Self::truthy(host.callv(name, &args));
            }
        }
        // 3) 对话 control 节点（如 DialogBox.has_flag）
        Self::truthy(dia.bind_mut().call_control(GString::from(name), args))
    }

    /// Variant 真值：nil = false，bool 原样，其余为 true
    fn truthy(v: Variant) -> bool {
        match v.get_type() {
            VariantType::NIL => false,
            VariantType::BOOL => v.to::<bool>(),
            _ => true,
        }
    }

    /// 节点全局位置（非 Node2D 返回 None）
    fn pos_of(node: &Gd<Node>) -> Option<Vector2> {
        node.clone()
            .try_cast::<Node2D>()
            .ok()
            .map(|mut n| n.get_global_position())
    }

    /// 让角色面向目标点（取主导轴）
    fn face(mover: &mut Gd<GdRoleMover>, toward: Vector2) {
        let from = mover.bind().base().get_global_position();
        let d = toward - from;
        let facing = if d.length() < 0.001 {
            "down"
        } else if d.x.abs() >= d.y.abs() {
            if d.x > 0.0 {
                "right"
            } else {
                "left"
            }
        } else if d.y > 0.0 {
            "down"
        } else {
            "up"
        };
        mover.bind_mut().set_facing(GString::from(facing));
    }

    /// 对话收尾：恢复移动、启动冷却、发信号
    fn end_dialog(&mut self) {
        self.dialog_active = false;
        self.cooldown_timer = self.cooldown;
        if self.trigger_mode == TRIGGER_PROXIMITY {
            self.rearm_on_exit = true;
        }
        if let Some(host) = self.base().get_parent() {
            if let Ok(mut m) = host.try_cast::<GdRoleMover>() {
                m.bind_mut().set_paused(false);
            }
        }
        if let Some(p) = self.resolve_player() {
            if let Ok(mut m) = p.try_cast::<GdRoleMover>() {
                m.bind_mut().set_paused(false);
            }
        }
        self.base_mut().emit_signal("s_dialog_ended", &[]);
    }
}
