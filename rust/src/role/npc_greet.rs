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
// 气泡显示：宿主有 show_bubble(text, secs) 时鸭子调用（复用宿主样式，
// 与闲聊/自言自语共用一个气泡位，后者被覆盖属预期）；否则自建 Label 子
// 节点（本组件继承 Node2D 保证变换链不断——红线：空间组件不可挂非
// CanvasItem 父下）。
//
// 玩家解析：与 GdDialogTrigger 同范式——"player" 分组 > 宿主兄弟节点中
// 第一个 GdRoleMover。

use rand::Rng;

use godot::prelude::*;
use godot::builtin::{GString, PackedStringArray, Vector2};
use godot::classes::{INode2D, Label, Node, Node2D};

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
    /// 自建气泡剩余显示时间（宿主 show_bubble 时不用）
    show_left: f64,
    /// tick 累计（0.2s 节流）
    acc: f64,
    /// 自建气泡标签（宿主无 show_bubble 时才创建）
    label: Option<Gd<Label>>,

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
            show_left: 0.0,
            acc: 0.0,
            label: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.cooldown_left = self.initial_cooldown;
    }

    fn process(&mut self, delta: f64) {
        if !self.enable {
            self.hide_label();
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
        // 自建气泡倒计时收起
        if self.show_left > 0.0 {
            self.show_left -= step;
            if self.show_left <= 0.0 {
                self.hide_label();
            }
        }
        // 宿主忙（对话/闲聊/赶路）：不触发且收起气泡（对话开始时立即让位）
        if self.host_busy() {
            self.hide_label();
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
        // 显示：宿主 show_bubble 优先（共用气泡位），否则自建标签
        if host.has_method("show_bubble") {
            host.call("show_bubble", &[GString::from(text.as_str()).to_variant(), secs.to_variant()]);
            self.hide_label();
        } else {
            self.show_label(&text, secs);
        }
        self.cooldown_left = self.cooldown + rng.gen_range(0.0..3.0);
        self.base_mut().emit_signal(
            &StringName::from("s_greet"),
            &[GString::from(text.as_str()).to_variant()],
        );
    }

    // ---- 自建气泡（宿主无 show_bubble 时的兜底显示） ----

    fn ensure_label(&mut self) -> Gd<Label> {
        if let Some(ref l) = self.label {
            if l.is_instance_valid() {
                return l.clone();
            }
        }
        let mut l = Label::new_alloc();
        l.set_name("__GreetBubble");
        l.add_theme_font_size_override(&StringName::from("font_size"), 11);
        l.add_theme_color_override(&StringName::from("font_color"), Color::from_rgb(1.0, 0.97, 0.9));
        l.add_theme_color_override(&StringName::from("font_outline_color"), Color::from_rgb(0.12, 0.1, 0.06));
        l.add_theme_constant_override(&StringName::from("outline_size"), 4);
        l.set_horizontal_alignment(godot::global::HorizontalAlignment::CENTER);
        l.set_autowrap_mode(godot::classes::text_server::AutowrapMode::WORD_SMART);
        l.set_position(Vector2::new(-80.0, -70.0));
        l.set_size(Vector2::new(160.0, 30.0));
        l.set_mouse_filter(godot::classes::control::MouseFilter::IGNORE);
        l.set_visible(false);
        {
            let mut base = self.base_mut();
            base.add_child(&l);
        }
        self.label = Some(l.clone());
        l
    }

    fn show_label(&mut self, text: &str, secs: f64) {
        let mut l = self.ensure_label();
        l.set_text(&GString::from(text));
        l.set_visible(true);
        self.show_left = secs;
    }

    fn hide_label(&mut self) {
        self.show_left = 0.0;
        if let Some(ref l) = self.label {
            if l.is_instance_valid() {
                let mut l = l.clone();
                l.set_visible(false);
            }
        }
    }
}
