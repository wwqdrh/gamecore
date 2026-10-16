// dialog 默认 UI 实现（rust 侧，供 GdMoviePlayer 等演出系统开箱即用）
//
// 两块屏幕 UI 的框架默认实现，宿主可覆写：
//   · GdMovieDialogBox —— 屏幕下方对话框（"黑框"）：show_movie_line /
//     movie_line_done / close_movie_box 鸭子契约，点击/E/空格推进。
//     demo 侧 dialog_box.gd 的 movie 模式已由本实现替代。
//   · GdMovieBubble    —— 头顶气泡：挂角色节点下跟随移动，show_bubble/
//     hide_bubble，超时自动隐藏。GdNpcSchedule/GdNpcChat/GdNpcGreet 与
//     movie bubble 指令统一经 show_bubble_on 调用（宿主有 show_bubble
//     方法则复用宿主样式，否则自动挂本组件）。
//
// 对接方式（movie_player.rs）：
//   say   → 配置宿主有 show_movie_line 用宿主，否则懒创建 GdMovieDialogBox
//   bubble→ show_bubble_on(actor, text, secs)

use godot::prelude::*;

use godot::builtin::{GString, Side, VarDictionary, Vector2};
use godot::classes::{
    control::{LayoutPreset, MouseFilter}, CanvasLayer, Control, ICanvasLayer, InputEvent, InputEventKey, InputEventMouseButton, Label,
    Node, Panel, StyleBoxFlat, Timer,
};
use godot::global::MouseButton;
use godot::obj::NewAlloc;

// ---------------------------------------------------------------------------
// 默认对话框（屏幕下方"黑框"）
// ---------------------------------------------------------------------------

/// 演出 UI 保留层：CanvasLayer 嵌套层是全局排序的，默认 layer=1 会与 demo
/// 主 HUD(1) 同层被底部道具栏遮住（demo dialog_layer.gd 同坑）。取高位
/// （高于 demo 全屏层段 10），演出期恒最顶。demo 层级总表见
/// example/demo/xiuxian/ui/ui_layers.gd（XiuUILayers.MOVIE 同值）。
const MOVIE_DIALOG_LAYER: i32 = 100;

#[derive(GodotClass)]
#[class(base = CanvasLayer)]
pub struct GdMovieDialogBox {
    base: Base<CanvasLayer>,

    /// 台词确认标志（玩家点击/E/空格置位；GdMoviePlayer 轮询消费）
    #[var]
    movie_line_done: bool,

    panel: Option<Gd<Panel>>,
}

#[godot_api]
impl ICanvasLayer for GdMovieDialogBox {
    fn init(base: Base<CanvasLayer>) -> Self {
        GdMovieDialogBox { base, movie_line_done: false, panel: None }
    }

    fn ready(&mut self) {
        if self.panel.is_some() {
            return;
        }
        self.base_mut().set_layer(MOVIE_DIALOG_LAYER);
        self.build_ui();
        self.hide_box();
    }

    /// 点击 / E / 空格推进（仅台词显示期间置位标志）
    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        let hit = if let Ok(mb) = event.clone().try_cast::<InputEventMouseButton>() {
            mb.is_pressed() && mb.get_button_index() == MouseButton::LEFT
        } else if let Ok(k) = event.clone().try_cast::<InputEventKey>() {
            k.is_pressed()
                && !k.is_echo()
                && (k.get_keycode() == godot::global::Key::E
                    || k.get_keycode() == godot::global::Key::SPACE)
        } else {
            false
        };
        if hit {
            self.advance();
        }
    }
}

#[godot_api]
impl GdMovieDialogBox {
    /// 显示一句演出台词：{name, display_name?, text}（GdMoviePlayer say 契约）
    #[func]
    pub fn show_movie_line(&mut self, line: VarDictionary) {
        self.show_box();
        self.movie_line_done = false;
        let Some(panel) = self.panel.clone() else { return };
        let text: GString =
            line.get("text").and_then(|v| v.try_to::<GString>().ok()).unwrap_or_default();
        let disp: GString = line
            .get("display_name")
            .and_then(|v| v.try_to::<GString>().ok())
            .unwrap_or_default();
        let name: GString =
            line.get("name").and_then(|v| v.try_to::<GString>().ok()).unwrap_or_default();
        let shown = if disp.is_empty() { name } else { disp };
        if let Some(l) = panel.get_node_or_null(&NodePath::from("NameLabel")) {
            let mut l = l;
            l.set("text", &shown.to_variant());
        }
        if let Some(l) = panel.get_node_or_null(&NodePath::from("TextLabel")) {
            let mut l = l;
            l.set("text", &text.to_variant());
        }
    }

    /// 演出结束收尾：隐藏面板、复位标志
    #[func]
    pub fn close_movie_box(&mut self) {
        self.movie_line_done = false;
        self.hide_box();
    }

    /// 玩家推进（点击/按键落点）：台词显示中才置位
    #[func]
    pub fn advance(&mut self) {
        if self.base().is_visible() {
            self.movie_line_done = true;
        }
    }

    // === 内部 ===

    fn show_box(&mut self) {
        self.base_mut().set_visible(true);
    }

    fn hide_box(&mut self) {
        self.base_mut().set_visible(false);
    }

    /// 构建 UI：底部黑底圆角面板 + 名字/文本/提示（精简版，无立绘无选项）
    fn build_ui(&mut self) {
        let mut panel = Panel::new_alloc();
        panel.set_name("Panel");
        let mut style = StyleBoxFlat::new_gd();
        style.set_bg_color(Color::from_rgba(0.08, 0.09, 0.12, 0.92));
        style.set_corner_radius_all(10);
        style.set_border_width_all(2);
        style.set_border_color(Color::from_rgba(0.35, 0.5, 0.75, 1.0));
        panel.add_theme_stylebox_override(&StringName::from("panel"), &style);
        panel.set_anchors_preset(LayoutPreset::CENTER_BOTTOM);
        panel.set_offset(Side::LEFT, -450.0);
        panel.set_offset(Side::RIGHT, 450.0);
        panel.set_offset(Side::TOP, -230.0);
        panel.set_offset(Side::BOTTOM, -24.0);
        // 面板不拦点击：点击落到 unhandled_input 推进（演出期玩家已锁，
        // 无误触世界寻路的风险）
        panel.set_mouse_filter(MouseFilter::IGNORE);

        let mut name_label = Label::new_alloc();
        name_label.set_name("NameLabel");
        name_label.set_position(Vector2::new(28.0, 14.0));
        name_label.add_theme_font_size_override(&StringName::from("font_size"), 22);
        name_label.add_theme_color_override(
            &StringName::from("font_color"),
            Color::from_rgba(1.0, 0.85, 0.45, 1.0),
        );
        panel.add_child(&name_label);

        let mut text_label = Label::new_alloc();
        text_label.set_name("TextLabel");
        text_label.set_position(Vector2::new(28.0, 52.0));
        text_label.set_size(Vector2::new(844.0, 120.0));
        text_label.set_autowrap_mode(godot::classes::text_server::AutowrapMode::WORD_SMART);
        text_label.add_theme_font_size_override(&StringName::from("font_size"), 19);
        text_label.add_theme_color_override(
            &StringName::from("font_color"),
            Color::from_rgba(0.92, 0.94, 0.98, 1.0),
        );
        panel.add_child(&text_label);

        let mut hint = Label::new_alloc();
        hint.set_text("点击 / E 继续");
        hint.add_theme_font_size_override(&StringName::from("font_size"), 14);
        hint.add_theme_color_override(
            &StringName::from("font_color"),
            Color::from_rgba(0.6, 0.65, 0.75, 1.0),
        );
        hint.set_anchors_preset(LayoutPreset::BOTTOM_RIGHT);
        hint.set_offset(Side::LEFT, -130.0);
        hint.set_offset(Side::TOP, -32.0);
        hint.set_offset(Side::RIGHT, -16.0);
        hint.set_offset(Side::BOTTOM, -10.0);
        panel.add_child(&hint);

        self.base_mut().add_child(&panel);
        self.panel = Some(panel);
    }
}

// ---------------------------------------------------------------------------
// 头顶气泡（跟随角色）
// ---------------------------------------------------------------------------

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdMovieBubble {
    base: Base<Node2D>,

    label: Option<Gd<Label>>,
    panel: Option<Gd<Panel>>,
    timer: Option<Gd<Timer>>,
}

const BUBBLE_MAX_WIDTH: f32 = 160.0;
/// 气泡底部距角色脚底原点的间隙
const BUBBLE_GAP: f32 = 44.0;

#[godot_api]
impl INode2D for GdMovieBubble {
    fn init(base: Base<Node2D>) -> Self {
        GdMovieBubble { base, label: None, panel: None, timer: None }
    }

    fn ready(&mut self) {
        if self.panel.is_some() {
            return;
        }
        // 半透明黑底圆角描边，浮在角色头顶（z 抬高防被遮挡）
        let mut panel = Panel::new_alloc();
        let mut style = StyleBoxFlat::new_gd();
        style.set_bg_color(Color::from_rgba(0.1, 0.1, 0.14, 0.88));
        style.set_corner_radius_all(8);
        style.set_border_width_all(1);
        style.set_border_color(Color::from_rgba(0.85, 0.65, 0.3, 0.9));
        style.set_content_margin_all(8.0);
        panel.add_theme_stylebox_override(&StringName::from("panel"), &style);
        panel.set_mouse_filter(MouseFilter::IGNORE);

        let mut label = Label::new_alloc();
        label.set_name("Text");
        label.set_custom_minimum_size(Vector2::new(BUBBLE_MAX_WIDTH, 0.0));
        label.set_autowrap_mode(godot::classes::text_server::AutowrapMode::WORD_SMART);
        label.set_horizontal_alignment(godot::global::HorizontalAlignment::CENTER);
        label.add_theme_font_size_override(&StringName::from("font_size"), 11);
        label.add_theme_color_override(
            &StringName::from("font_color"),
            Color::from_rgba(1.0, 0.97, 0.9, 1.0),
        );
        label.add_theme_color_override(
            &StringName::from("font_outline_color"),
            Color::from_rgba(0.12, 0.1, 0.06, 1.0),
        );
        label.add_theme_constant_override(&StringName::from("outline_size"), 4);
        label.set_mouse_filter(MouseFilter::IGNORE);
        panel.add_child(&label);

        let mut timer = Timer::new_alloc();
        timer.set_one_shot(true);

        self.base_mut().add_child(&panel);
        self.base_mut().add_child(&timer);
        self.base_mut().set_z_index(100);
        // 超时自动隐藏（回调绑定本节点方法）
        let cb = Callable::from_object_method(&self.to_gd(), "_on_bubble_timeout");
        timer.connect(&StringName::from("timeout"), &cb);
        self.panel = Some(panel);
        self.label = Some(label);
        self.timer = Some(timer);
    }
}

#[godot_api]
impl GdMovieBubble {
    /// 显示气泡 secs 秒（超时自动隐藏；重复调用刷新文本与计时）
    #[func]
    pub fn show_bubble(&mut self, text: GString, secs: f64) {
        let Some(label) = self.label.clone() else { return };
        let mut label = label;
        label.set_text(&text);
        // 容器按文本自适应后水平居中于角色、底缘悬在头顶
        let Some(panel) = self.panel.clone() else { return };
        let mut panel = panel;
        panel.reset_size();
        let size = panel.get_size();
        panel.set_position(Vector2::new(-size.x / 2.0, -(size.y + BUBBLE_GAP)));
        self.base_mut().set_visible(true);
        if let Some(t) = self.timer.clone() {
            let mut t = t;
            t.set_wait_time(secs);
            t.start();
        }
    }

    /// 隐藏气泡
    #[func]
    pub fn hide_bubble(&mut self) {
        self.base_mut().set_visible(false);
    }

    /// 超时回调（Timer timeout → 隐藏）
    #[func]
    fn _on_bubble_timeout(&mut self) {
        self.base_mut().set_visible(false);
    }

    /// 取宿主身上的气泡组件（有则复用，无则创建挂载；跨系统共享一个气泡位）
    #[func]
    pub fn ensure(mut host: Gd<Node>) -> Gd<GdMovieBubble> {
        if let Some(child) = host.get_node_or_null(&NodePath::from("__MovieBubble")) {
            if let Ok(b) = child.try_cast::<GdMovieBubble>() {
                return b;
            }
        }
        let mut bubble = GdMovieBubble::new_alloc();
        bubble.set_name("__MovieBubble");
        host.add_child(&bubble.clone());
        bubble
    }
}

// ---------------------------------------------------------------------------
// 共享入口（rust 侧各系统统一调用）
// ---------------------------------------------------------------------------

/// 在宿主身上显示气泡：宿主实现了 show_bubble(text, secs) 则复用宿主样式，
/// 否则自动挂 GdMovieBubble 默认实现（框架开箱即用，demo 零配置）
pub fn show_bubble_on(host: &mut Gd<Node>, text: &str, secs: f64) {
    if host.has_method("show_bubble") {
        host.call("show_bubble", &[text.to_variant(), secs.to_variant()]);
        return;
    }
    let mut bubble = GdMovieBubble::ensure(host.clone());
    bubble.call("show_bubble", &[text.to_variant(), secs.to_variant()]);
}

/// 隐藏宿主气泡（与 show_bubble_on 成对）
pub fn hide_bubble_on(host: &mut Gd<Node>) {
    if host.has_method("hide_bubble") {
        host.call("hide_bubble", &[]);
        return;
    }
    let mut bubble = GdMovieBubble::ensure(host.clone());
    bubble.call("hide_bubble", &[]);
}
