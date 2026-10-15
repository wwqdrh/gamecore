// GdUIToast - 队列提示条（淡入淡出逐条播放，引导/反馈用）
//
// GML 用法：
//   <Toast name="HintToast" anchor="top_center" duration="2.5" fade_secs="0.25"
//          font_size="18" bg_color="0.08,0.08,0.10,0.88" @s_toast_done="_on_done" />
//
// 能力：
//   - 队列：show_toast 连续调用不互相覆盖，逐条 淡入→停留→淡出 播放
//   - 信号：s_toast_shown(text)（每条开始时）/ s_toast_done()（队列播空时）
//   - 鼠标穿透：纯展示组件，mouse_filter 恒 IGNORE（构建期白名单兜底）
//   - 编辑器内静态隐藏（tool 类，运行期行为全部守卫）
//
// 程序化 API：
//   show_toast(text)            入队一条（用默认 duration）
//   show_toast_secs(text, secs) 入队一条（自定义停留时长）
//   is_busy()                   队列非空或正在播放
//   clear_toasts()              清空队列并立即隐藏
//
// demo 范本 example/demo/xiuxian/scenes/main/ui/mainhud/mainhud_guide.gd：
//   教学提示连发 / 传送点门控提示（player.s_portal_locked 转发）

use godot::prelude::*;
use godot::obj::NewGd;
use godot::builtin::{Color, GString, Side, StringName};
use godot::classes::{
    Engine, IPanelContainer, Label, PanelContainer, StyleBoxFlat,
};
use godot::classes::control::{GrowDirection, MouseFilter};
use godot::global::HorizontalAlignment;

#[derive(GodotClass)]
#[class(base = PanelContainer, tool)]
pub struct GdUIToast {
    /// 单条默认停留时长（秒，淡入淡出之外）
    #[export]
    duration: f32,
    /// 淡入/淡出时长（秒）
    #[export]
    fade_secs: f32,
    /// 正文字号
    #[export]
    font_size: i32,
    /// 背景色
    #[export]
    bg_color: Color,
    /// 描边色
    #[export]
    border_color: Color,
    /// 文字色
    #[export]
    text_color: Color,
    /// 圆角半径
    #[export]
    corner_radius: i32,

    // ---- 运行时状态 ----
    queue: Vec<(GString, f32)>,
    /// 播放阶段：0=空闲 1=淡入 2=停留 3=淡出
    phase: u8,
    /// 当前阶段已耗时（秒）
    phase_t: f32,
    /// 当前条停留时长（show_toast_secs 指定或默认 duration）
    cur_hold: f32,
    label: Option<Gd<Label>>,

    base: Base<PanelContainer>,
}

#[godot_api]
impl IPanelContainer for GdUIToast {
    fn init(base: Base<PanelContainer>) -> Self {
        Self {
            duration: 2.5,
            fade_secs: 0.25,
            font_size: 18,
            bg_color: Color::from_rgba(0.08, 0.08, 0.10, 0.88),
            border_color: Color::from_rgba(0.90, 0.75, 0.40, 0.90),
            text_color: Color::from_rgba(0.95, 0.92, 0.85, 1.0),
            corner_radius: 8,
            queue: Vec::new(),
            phase: 0,
            phase_t: 0.0,
            cur_hold: 0.0,
            label: None,
            base,
        }
    }

    fn ready(&mut self) {
        // top_center 锚定后向两侧生长，保证水平居中
        self.base_mut().set(
            &StringName::from("grow_horizontal"),
            &GrowDirection::BOTH.to_variant(),
        );

        let mut style = StyleBoxFlat::new_gd();
        style.set_bg_color(self.bg_color);
        style.set_corner_radius_all(self.corner_radius);
        style.set_border_width_all(1);
        style.set_border_color(self.border_color);
        style.set_content_margin_all(10.0);
        style.set_content_margin(Side::LEFT, 22.0);
        style.set_content_margin(Side::RIGHT, 22.0);
        self.base_mut().add_theme_stylebox_override(
            &StringName::from("panel"),
            &style,
        );

        let mut lb = Label::new_alloc();
        lb.set_horizontal_alignment(HorizontalAlignment::CENTER);
        lb.add_theme_font_size_override(&StringName::from("font_size"), self.font_size);
        lb.add_theme_color_override(&StringName::from("font_color"), self.text_color);
        self.base_mut().add_child(&lb.clone().upcast::<Node>());
        self.label = Some(lb);

        // 编辑器内保持隐藏（无运行期内容可预览）
        self.base_mut().set_visible(false);
        self.base_mut().set_mouse_filter(MouseFilter::IGNORE);
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() || self.phase == 0 {
            return;
        }
        let dt = delta as f32;
        let fade = self.fade_secs.max(0.01);
        self.phase_t += dt;
        match self.phase {
            1 => {
                // 淡入
                let a = (self.phase_t / fade).min(1.0);
                self.base_mut().set_modulate(Color::from_rgba(1.0, 1.0, 1.0, a));
                if a >= 1.0 {
                    self.phase_t = 0.0;
                    self.phase = 2;
                }
            }
            2 => {
                // 停留
                if self.phase_t >= self.cur_hold {
                    self.phase_t = 0.0;
                    self.phase = 3;
                }
            }
            3 => {
                // 淡出
                let a = (1.0 - self.phase_t / fade).max(0.0);
                self.base_mut().set_modulate(Color::from_rgba(1.0, 1.0, 1.0, a));
                if a <= 0.0 {
                    self.advance_queue();
                }
            }
            _ => {}
        }
    }
}

#[godot_api]
impl GdUIToast {
    /// 每条消息开始播放时发出（参数：文本）
    #[signal]
    fn s_toast_shown(text: GString);

    /// 队列全部播完时发出（一次入队批次只发一次）
    #[signal]
    fn s_toast_done();

    /// 入队一条提示（用默认 duration）
    #[func]
    pub fn show_toast(&mut self, text: GString) {
        self.push(text, self.duration.max(0.1));
    }

    /// 入队一条提示（自定义停留时长，秒）
    #[func]
    pub fn show_toast_secs(&mut self, text: GString, secs: f32) {
        self.push(text, secs.max(0.1));
    }

    /// 队列非空或正在播放
    #[func]
    pub fn is_busy(&self) -> bool {
        self.phase != 0 || !self.queue.is_empty()
    }

    /// 清空队列并立即隐藏（正在播的这条也丢弃）
    #[func]
    pub fn clear_toasts(&mut self) {
        self.queue.clear();
        self.phase = 0;
        self.phase_t = 0.0;
        self.base_mut().set_visible(false);
        self.base_mut().set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 1.0));
    }

    // ---- 内部实现 ----

    fn push(&mut self, text: GString, hold: f32) {
        if Engine::singleton().is_editor_hint() {
            return;
        }
        self.queue.push((text, hold));
        if self.phase == 0 {
            self.advance_queue();
        }
    }

    /// 取下一条进入淡入阶段；队列空则收束回空闲并发 s_toast_done
    fn advance_queue(&mut self) {
        let Some((text, hold)) = self.queue.first().cloned() else {
            self.phase = 0;
            self.phase_t = 0.0;
            self.base_mut().set_visible(false);
            self.base_mut().emit_signal("s_toast_done", &[]);
            return;
        };
        self.queue.remove(0);
        self.cur_hold = hold;
        self.phase = 1;
        self.phase_t = 0.0;
        if let Some(lb) = &self.label {
            let mut lb = lb.clone();
            lb.set_text(&text);
        }
        {
            let mut base = self.base_mut();
            base.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));
            base.set_visible(true);
        }
        self.base_mut()
            .emit_signal("s_toast_shown", &[text.to_variant()]);
    }
}
