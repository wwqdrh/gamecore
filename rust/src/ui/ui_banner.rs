// GdUIBanner - 全屏演出字幅（突破/通关/大事件，居中大字 + 暗幕）
//
// GML 用法：
//   <Banner name="PerfBanner" anchor="full" hold_secs="1.6" anim_secs="0.6"
//           title_font="44" dim_bg="true" @s_banner_finished="_on_banner_done" />
//
// 能力：
//   - play_banner(title, subtitle)：淡入（标题 1.25→1.0 缩放）→停留→淡出，
//     全程自动收束回隐藏，发 s_banner_finished
//   - 播放期间 mouse_filter=STOP 挡输入（防演出中误点），结束回 IGNORE
//   - dim_bg=true 时铺半透明暗幕（颜色 dim_color）聚焦视线
//   - 编辑器内静态隐藏（tool 类，运行期行为全部守卫）
//
// 程序化 API：
//   play_banner(title, subtitle)  播放一次演出（播放中重复调用忽略）
//   is_playing()                  是否播放中
//
// demo 范本 example/demo/xiuxian/scenes/main/ui/mainhud/mainhud_guide.gd：
//   突破成功演出 / 阶段通关字幅（_process 轮询状态触发）

use godot::prelude::*;
use godot::obj::NewGd;
use godot::builtin::{Color, GString, StringName, Vector2};
use godot::classes::{
    CenterContainer, ColorRect, Control, Engine, IControl, Label, VBoxContainer,
};
use godot::classes::control::{GrowDirection, LayoutPreset, MouseFilter};

#[derive(GodotClass)]
#[class(base = Control, tool)]
pub struct GdUIBanner {
    /// 停留时长（秒）
    #[export]
    hold_secs: f32,
    /// 单段动画时长（秒，淡入/淡出各一次）
    #[export]
    anim_secs: f32,
    /// 标题字号
    #[export]
    title_font: i32,
    /// 副标题字号
    #[export]
    sub_font: i32,
    /// 标题色
    #[export]
    title_color: Color,
    /// 副标题色
    #[export]
    sub_color: Color,
    /// 是否铺暗幕
    #[export]
    dim_bg: bool,
    /// 暗幕颜色
    #[export]
    dim_color: Color,

    // ---- 运行时状态 ----
    /// 播放阶段：0=空闲 1=淡入 2=停留 3=淡出
    phase: u8,
    phase_t: f32,
    /// 播放中收到的后续请求（先排队，收束后逐个播）
    pending: Vec<(GString, GString)>,
    dim: Option<Gd<ColorRect>>,
    title: Option<Gd<Label>>,
    sub: Option<Gd<Label>>,

    base: Base<Control>,
}

#[godot_api]
impl IControl for GdUIBanner {
    fn init(base: Base<Control>) -> Self {
        Self {
            hold_secs: 1.6,
            anim_secs: 0.6,
            title_font: 44,
            sub_font: 20,
            title_color: Color::from_rgba(1.0, 0.88, 0.55, 1.0),
            sub_color: Color::from_rgba(0.92, 0.90, 0.85, 1.0),
            dim_bg: true,
            dim_color: Color::from_rgba(0.0, 0.0, 0.0, 0.55),
            phase: 0,
            phase_t: 0.0,
            pending: Vec::new(),
            dim: None,
            title: None,
            sub: None,
            base,
        }
    }

    fn ready(&mut self) {
        // 播放中 STOP 挡输入；默认 IGNORE 穿透（构建期白名单兜底）
        self.base_mut().set_mouse_filter(MouseFilter::IGNORE);

        // 暗幕
        if self.dim_bg {
            let mut rect = ColorRect::new_alloc();
            rect.set_color(self.dim_color);
            rect.set_mouse_filter(MouseFilter::IGNORE);
            self.base_mut().add_child(&rect.clone().upcast::<Node>());
            rect.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
            rect.set_visible(false);
            self.dim = Some(rect);
        }

        // 居中容器 + 标题/副标题
        let mut center = CenterContainer::new_alloc();
        center.set_mouse_filter(MouseFilter::IGNORE);
        self.base_mut().add_child(&center.clone().upcast::<Node>());
        center.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);

        let mut vbox = VBoxContainer::new_alloc();
        vbox.set_mouse_filter(MouseFilter::IGNORE);
        vbox.add_theme_constant_override(&StringName::from("separation"), 14);
        center.add_child(&vbox.clone().upcast::<Node>());

        let mut t = Label::new_alloc();
        t.set_horizontal_alignment(godot::global::HorizontalAlignment::CENTER);
        t.add_theme_font_size_override(&StringName::from("font_size"), self.title_font);
        t.add_theme_color_override(&StringName::from("font_color"), self.title_color);
        vbox.add_child(&t.clone().upcast::<Node>());

        let mut s = Label::new_alloc();
        s.set_horizontal_alignment(godot::global::HorizontalAlignment::CENTER);
        s.add_theme_font_size_override(&StringName::from("font_size"), self.sub_font);
        s.add_theme_color_override(&StringName::from("font_color"), self.sub_color);
        vbox.add_child(&s.clone().upcast::<Node>());

        self.title = Some(t);
        self.sub = Some(s);

        self.base_mut().set(
            &StringName::from("grow_horizontal"),
            &GrowDirection::BOTH.to_variant(),
        );
        self.base_mut().set(
            &StringName::from("grow_vertical"),
            &GrowDirection::BOTH.to_variant(),
        );
        self.base_mut().set_visible(false);
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() || self.phase == 0 {
            return;
        }
        let dt = delta as f32;
        let anim = self.anim_secs.max(0.05);
        self.phase_t += dt;
        match self.phase {
            1 => {
                // 淡入：整体 alpha 0→1，标题 scale 1.25→1.0
                let k = (self.phase_t / anim).min(1.0);
                self.apply_dim_alpha(k);
                self.base_mut().set_modulate(Color::from_rgba(1.0, 1.0, 1.0, k));
                self.apply_title_scale(1.25 - 0.25 * k);
                if k >= 1.0 {
                    self.phase_t = 0.0;
                    self.phase = 2;
                }
            }
            2 => {
                // 停留
                if self.phase_t >= self.hold_secs.max(0.05) {
                    self.phase_t = 0.0;
                    self.phase = 3;
                }
            }
            3 => {
                // 淡出
                let k = (1.0 - self.phase_t / anim).max(0.0);
                self.apply_dim_alpha(k);
                self.base_mut().set_modulate(Color::from_rgba(1.0, 1.0, 1.0, k));
                if k <= 0.0 {
                    self.finish();
                }
            }
            _ => {}
        }
    }
}

#[godot_api]
impl GdUIBanner {
    /// 演出收束时发出（一次）
    #[signal]
    fn s_banner_finished();

    /// 播放一次演出；播放中调用改为排队（收束后逐个播，上限 4 条）
    #[func]
    pub fn play_banner(&mut self, title: GString, subtitle: GString) {
        if Engine::singleton().is_editor_hint() {
            return;
        }
        if self.phase != 0 {
            if self.pending.len() < 4 {
                self.pending.push((title, subtitle));
            }
            return;
        }
        self.start_play(title, subtitle);
    }

    /// 是否播放中
    #[func]
    pub fn is_playing(&self) -> bool {
        self.phase != 0
    }

    // ---- 内部实现 ----

    fn start_play(&mut self, title: GString, subtitle: GString) {
        if let Some(t) = &self.title {
            let mut t = t.clone();
            t.set_text(&title);
            t.set_visible(!title.is_empty());
        }
        if let Some(s) = &self.sub {
            let mut s = s.clone();
            s.set_text(&subtitle);
            s.set_visible(!subtitle.is_empty());
        }
        if let Some(d) = &self.dim {
            let mut d = d.clone();
            d.set_visible(true);
        }
        self.phase = 1;
        self.phase_t = 0.0;
        {
            let mut base = self.base_mut();
            base.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));
            base.set_visible(true);
            // 挡输入防演出中误点
            base.set_mouse_filter(MouseFilter::STOP);
        }
        self.apply_title_scale(1.25);
    }

    fn apply_dim_alpha(&mut self, k: f32) {
        if let Some(d) = &self.dim {
            let mut d = d.clone();
            let c = self.dim_color;
            d.set_color(Color::from_rgba(c.r, c.g, c.b, c.a * k));
        }
    }

    /// 标题缩放（绕自身中心；size 未布局好时跳过）
    fn apply_title_scale(&mut self, scale: f32) {
        if let Some(t) = &self.title {
            let mut t = t.clone();
            let size = t.get_size();
            if size.x > 0.0 {
                t.set_pivot_offset(size * 0.5);
            }
            t.set_scale(Vector2::new(scale, scale));
        }
    }

    fn finish(&mut self) {
        self.phase = 0;
        self.phase_t = 0.0;
        if let Some(d) = &self.dim {
            let mut d = d.clone();
            d.set_visible(false);
        }
        {
            let mut base = self.base_mut();
            base.set_visible(false);
            base.set_mouse_filter(MouseFilter::IGNORE);
        }
        self.base_mut().emit_signal("s_banner_finished", &[]);
        // 队列有后续请求：无缝接播（不再发 finished，每条收束各发一次）
        if let Some((t, s)) = self.pending.first().cloned() {
            self.pending.remove(0);
            self.start_play(t, s);
        }
    }
}
