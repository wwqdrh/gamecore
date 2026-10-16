// GdNpcGreet - NPC 主动问候（玩家靠近显示浮动对话）
//
// 挂在角色（Node2D 宿主）的子节点上：玩家进入 radius 时在角色附近弹出一条
// 预制问候气泡，玩家不搭理（不按 E）也能感受到 NPC"活着"。边沿触发 +
// 冷却：待在范围内不刷屏，反复进出有冷却。
//
// 抑制条件（不触发且隐藏气泡）：
//   - 宿主 paused（对话中 / 闲聊中）
//   - 宿主赶路中（is_grid_path_active，走过去再打招呼）
//
// 问候文案解析顺序：
//   1. 宿主鸭子方法 resolve_greeting() -> String（非空即用；demo 侧用它实现
//      「重逢问候」优先——remeet_flag 已置时返回 remeet_greeting）
//   2. 本组件 lines 随机一条
//   3. 都没有 → 本次不显示
//
// 气泡显示：统一走 dialog/movie_ui.rs 的 show_bubble_on——宿主有
// show_bubble(text, secs) 时鸭子调用（复用宿主样式，与闲聊/自言自语
// 共用一个气泡位，后者被覆盖属预期）；否则自动挂 rust 默认头顶气泡
// （GdMovieBubble，自带超时隐藏）。
//
// 玩家解析：与 GdDialogTrigger 同范式——"player" 分组 > 宿主兄弟节点中
// 第一个 GdRoleMover。

use rand::Rng;

use godot::prelude::*;
use godot::builtin::{GString, PackedStringArray};
use godot::classes::{INode2D, Node, Node2D};

use crate::dialog::{hide_bubble_on, show_bubble_on};

use crate::role::GdRoleMover;

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdNpcGreet {
    /// 是否启用
    #[export]
    enable: bool,

    /// 预设问候台词（随机一条；宿主 resolve_greeting 非空时优先用宿主的）
    #[export]
    lines: PackedStringArray,

    /// 触发距离（像素）
    #[export]
    radius: f64,

    /// 冷却（秒，触发后附加 0~3s 随机）
    #[export]
    cooldown: f64,

    /// 气泡持续时长（秒）
    #[export]
    line_secs: f64,

    /// 进场初始冷却（秒）：避免玩家出生点贴脸 NPC 时瞬间弹气泡
    #[export]
    initial_cooldown: f64,

    // ---- 运行时状态 ----
    cooldown_left: f64,
    /// 玩家上一帧是否在范围内（边沿触发）
    was_inside: bool,
    /// tick 累计（0.2s 节流）
    acc: f64,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdNpcGreet {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            enable: true,
            lines: PackedStringArray::new(),
            radius: 140.0,
            cooldown: 20.0,
            line_secs: 3.0,
            initial_cooldown: 2.0,
            cooldown_left: 2.0,
            was_inside: false,
            acc: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        self.cooldown_left = self.initial_cooldown;
    }

    fn process(&mut self, delta: f64) {
        if !self.enable {
            // 禁用（演出模式/测试）只冻结触发，不逐帧收气泡——每帧 hide 会
            // 打架：演出 bubble 指令刚 show 完下一帧就被这里隐藏（实测一闪
            // 而过）。气泡本身超时自隐，禁用残留几秒无害。
            return;
        }
        // 0.2s 节流：距离判定是分钟级逻辑
        self.acc += delta;
        if self.acc < 0.2 {
            return;
        }
        let step = self.acc; // 实际经过时间（含掉帧补偿）
        self.acc = 0.0;
        if self.cooldown_left > 0.0 {
            self.cooldown_left -= step;
        }
        // 宿主忙（对话/闲聊/赶路）：不触发且收起气泡（对话开始时立即让位）
        if self.host_busy() {
            self.hide_host_bubble();
            self.was_inside = self.player_inside();
            return;
        }
        let inside = self.player_inside();
        let edge = inside && !self.was_inside;
        self.was_inside = inside;
        if edge && self.cooldown_left <= 0.0 {
            self.try_greet();
        }
    }
}

#[godot_api]
impl GdNpcGreet {
    #[signal]
    fn s_greet(text: GString);

    /// 手动触发一次问候（绕过边沿/冷却，仍受文案解析与宿主状态约束）
    #[func]
    pub fn greet_now(&mut self) {
        if !self.enable || self.host_busy() {
            return;
        }
        self.try_greet();
    }
}

impl GdNpcGreet {
    fn host(&self) -> Option<Gd<Node>> {
        self.base().get_parent()
    }

    /// 宿主忙：对话/闲聊中（paused）或赶路中
    fn host_busy(&self) -> bool {
        let Some(mut host) = self.host() else { return false };
        if host.has_method("is_paused") && host.call("is_paused", &[]).to::<bool>() {
            return true;
        }
        if host.has_method("is_grid_path_active")
            && host.call("is_grid_path_active", &[]).to::<bool>()
        {
            return true;
        }
        false
    }

    /// 玩家是否在触发范围内
    fn player_inside(&self) -> bool {
        let Some(player) = self.resolve_player() else { return false };
        let (Some(mut hp), Some(mut pp)) = (
            self.base().clone().try_cast::<Node2D>().ok(),
            player.clone().try_cast::<Node2D>().ok(),
        ) else {
            return false;
        };
        hp.get_global_position().distance_to(pp.get_global_position()) <= self.radius as f32
    }

    /// 玩家解析："player" 分组 > 宿主兄弟节点中第一个 GdRoleMover
    fn resolve_player(&self) -> Option<Gd<Node>> {
        let tree = self.base().get_tree();
        if let Some(n) = tree.get_first_node_in_group("player") {
            return Some(n);
        }
        let host = self.base().get_parent()?;
        let container = host.get_parent()?;
        for i in 0..container.get_child_count() {
            let Some(child) = container.get_child(i) else { continue };
            if child.instance_id() == host.instance_id() {
                continue;
            }
            if child.clone().try_cast::<GdRoleMover>().is_ok() {
                return Some(child);
            }
        }
        None
    }

    /// 解析并显示问候
    fn try_greet(&mut self) {
        let Some(mut host) = self.host() else { return };
        // 1) 宿主解析（demo：重逢问候优先）
        let mut text = String::new();
        if host.has_method("resolve_greeting") {
            if let Ok(s) = host.call("resolve_greeting", &[]).try_to::<GString>() {
                text = s.to_string();
            }
        }
        // 2) 预设台词随机一条
        let mut rng = rand::thread_rng();
        if text.is_empty() && !self.lines.is_empty() {
            let i = rng.gen_range(0..self.lines.len());
            text = self.lines.get(i).unwrap_or_default().to_string();
        }
        if text.is_empty() {
            return;
        }
        let secs = self.line_secs + rng.gen_range(0.0..0.5);
        // 显示：宿主 show_bubble 优先（共用气泡位），否则 rust 默认头顶气泡
        show_bubble_on(&mut host, &text, secs);
        self.cooldown_left = self.cooldown + rng.gen_range(0.0..3.0);
        self.base_mut().emit_signal(
            &StringName::from("s_greet"),
            &[GString::from(text.as_str()).to_variant()],
        );
    }

    /// 收起宿主身上的气泡（宿主忙时让位）
    fn hide_host_bubble(&mut self) {
        let Some(mut host) = self.host() else { return };
        hide_bubble_on(&mut host);
    }
}
