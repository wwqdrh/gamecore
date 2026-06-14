// GdUIDrawer - 抽屉面板节点
// 继承 Control，从屏幕边缘滑入/滑出
// 支持动画过渡、模态遮罩、内容区域
// GML 标签：<Drawer name="MyDrawer" direction="right" slide_width="320">

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Side};
use godot::classes::{
    IControl, Control, PanelContainer, VBoxContainer, HBoxContainer,
    MarginContainer, Label, Button, ColorRect, HSeparator, StyleBoxFlat,
    InputEvent, InputEventMouseButton,
};
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::global::MouseButton;
use godot::obj::WithBaseField;

use crate::anim::easing::ease_out_cubic;

/// 抽屉方向
const DIR_RIGHT: i32 = 0;
const DIR_LEFT: i32 = 1;
const DIR_TOP: i32 = 2;
const DIR_BOTTOM: i32 = 3;

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdUIDrawer {
    base: Base<Control>,

    #[export]
    direction: i32,
    #[export]
    slide_width: i32,
    #[export]
    overlay_color: Color,
    #[export]
    drawer_bg_color: Color,
    #[export]
    drawer_border_color: Color,
    #[export]
    corner_radius: i32,
    #[export]
    animation_duration: f64,
    #[export]
    close_on_overlay: bool,
    #[export]
    drawer_title_text: GString,

    // 内部节点引用
    overlay: Option<Gd<ColorRect>>,
    drawer_panel: Option<Gd<PanelContainer>>,
    title_label: Option<Gd<Label>>,
    content_container: Option<Gd<MarginContainer>>,
    is_open: bool,
    ui_built: bool,
    anim_progress: f64,
    animating: bool,
    opening: bool,
}

#[godot_api]
impl IControl for GdUIDrawer {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            direction: DIR_RIGHT,
            slide_width: 320,
            overlay_color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            drawer_bg_color: Color::from_rgba(0.08, 0.08, 0.14, 0.97),
            drawer_border_color: Color::from_rgb(0.35, 0.4, 0.55),
            corner_radius: 0,
            animation_duration: 0.25,
            close_on_overlay: true,
            drawer_title_text: GString::new(),
            overlay: None,
            drawer_panel: None,
            title_label: None,
            content_container: None,
            is_open: false,
            ui_built: false,
            anim_progress: 0.0,
            animating: false,
            opening: false,
        }
    }

    fn ready(&mut self) {
        if !self.ui_built {
            self.build_ui();
        }
        // 直接隐藏，不走动画（close() 在初始状态会跳过）
        self.is_open = false;
        self.animating = false;
        self.anim_progress = 0.0;
        self.base_mut().set_visible(false);
        if let Some(ref overlay) = self.overlay {
            let mut o = overlay.clone();
            o.set_color(Color::from_rgba(
                self.overlay_color.r,
                self.overlay_color.g,
                self.overlay_color.b,
                0.0,
            ));
        }
    }

    fn process(&mut self, delta: f64) {
        if !self.animating {
            return;
        }

        let speed = if self.animation_duration > 0.0 {
            1.0 / self.animation_duration
        } else {
            100.0
        };

        if self.opening {
            self.anim_progress += delta * speed;
            if self.anim_progress >= 1.0 {
                self.anim_progress = 1.0;
                self.animating = false;
            }
        } else {
            self.anim_progress -= delta * speed;
            if self.anim_progress <= 0.0 {
                self.anim_progress = 0.0;
                self.animating = false;
                // 动画结束，隐藏整个 Drawer
                self.base_mut().set_visible(false);
            }
        }

        self.apply_animation();
    }
}

#[godot_api]
impl GdUIDrawer {
    #[signal]
    fn s_drawer_opened();

    #[signal]
    fn s_drawer_closed();

    /// 打开抽屉
    #[func]
    fn open(&mut self) {
        if self.is_open {
            return;
        }
        self.is_open = true;
        self.opening = true;
        self.animating = true;
        self.base_mut().set_visible(true);
        // 遮罩从0开始渐入
        if let Some(ref overlay) = self.overlay {
            let mut o = overlay.clone();
            o.set_color(Color::from_rgba(
                self.overlay_color.r,
                self.overlay_color.g,
                self.overlay_color.b,
                0.0,
            ));
        }
    }

    /// 关闭抽屉
    #[func]
    fn close(&mut self) {
        if !self.is_open && !self.animating {
            return;
        }
        self.is_open = false;
        self.opening = false;
        self.animating = true;
        self.base_mut().emit_signal(&StringName::from("s_drawer_closed"), &[]);
    }

    /// 切换抽屉开关
    #[func]
    fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// 抽屉是否打开
    #[func]
    fn is_drawer_open(&self) -> bool {
        self.is_open
    }

    /// 设置抽屉标题
    #[func]
    fn set_drawer_title(&mut self, text: GString) {
        self.drawer_title_text = text.clone();
        if let Some(ref label) = self.title_label {
            let mut l = label.clone();
            l.set_text(&text);
        }
    }

    /// 获取内容区域节点路径
    #[func]
    fn get_content_path(&self) -> GString {
        if let Some(ref cc) = self.content_container {
            let path = cc.get_path();
            return GString::from(&path);
        }
        GString::new()
    }

    /// 添加子节点到内容区域（供 builder 调用）
    #[func]
    fn add_content_child(&mut self, mut child: Gd<godot::classes::Node>) {
        if let Some(ref mut cc) = self.content_container {
            cc.add_child(&child);
            child.set_owner(&cc.clone().upcast::<godot::classes::Node>());
        } else {
            godot_error!("[UIDrawer] add_content_child: content_container is None!");
        }
    }

    /// 处理遮罩点击关闭
    #[func]
    fn _on_overlay_gui_input(&mut self, event: Gd<InputEvent>) {
        if !self.close_on_overlay {
            return;
        }
        if let Ok(mouse_btn) = event.try_cast::<InputEventMouseButton>() {
            if mouse_btn.get_button_index() == MouseButton::LEFT && mouse_btn.is_pressed() {
                self.close();
            }
        }
    }

    /// 处理关闭按钮点击
    #[func]
    fn _on_close_pressed(&mut self) {
        self.close();
    }

    /// 确保内部 UI 已构建（供 builder 在添加子节点前调用）
    #[func]
    fn ensure_ui_built(&mut self) {
        if !self.ui_built {
            self.build_ui();
        }
    }

    /// 重新计算抽屉布局（slide_width 变化后调用）
    /// 更新 DrawerPanel 的锚点偏移使其匹配新的宽度
    #[func]
    fn update_layout(&mut self) {
        if let Some(ref panel) = self.drawer_panel {
            let mut p = panel.clone();
            let sw = self.slide_width as f32;
            match self.direction {
                DIR_RIGHT => {
                    p.set_anchor_and_offset(Side::LEFT, 1.0, -sw);
                    p.set_anchor_and_offset(Side::RIGHT, 1.0, 0.0);
                }
                DIR_LEFT => {
                    p.set_anchor_and_offset(Side::LEFT, 0.0, 0.0);
                    p.set_anchor_and_offset(Side::RIGHT, 0.0, sw);
                }
                DIR_TOP => {
                    p.set_anchor_and_offset(Side::TOP, 0.0, 0.0);
                    p.set_anchor_and_offset(Side::BOTTOM, 0.0, sw);
                }
                DIR_BOTTOM => {
                    p.set_anchor_and_offset(Side::TOP, 1.0, -sw);
                    p.set_anchor_and_offset(Side::BOTTOM, 1.0, 0.0);
                }
                _ => {
                    p.set_anchor_and_offset(Side::LEFT, 1.0, -sw);
                    p.set_anchor_and_offset(Side::RIGHT, 1.0, 0.0);
                }
            }
        }
    }
}

impl GdUIDrawer {
    fn build_ui(&mut self) {
        if self.ui_built {
            return;
        }
        self.ui_built = true;

        let overlay_color = self.overlay_color;
        let slide_width = self.slide_width;
        let direction = self.direction;
        let drawer_bg_color = self.drawer_bg_color;
        let drawer_border_color = self.drawer_border_color;
        let corner_radius = self.corner_radius;
        let drawer_title = self.drawer_title_text.clone();

        let mut overlay_node: Option<Gd<ColorRect>> = None;
        let mut panel_node: Option<Gd<PanelContainer>> = None;
        let mut title_node: Option<Gd<Label>> = None;
        let mut content_node: Option<Gd<MarginContainer>> = None;

        {
            let mut base = self.base_mut();
            base.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);

            // 遮罩层
            let mut overlay = ColorRect::new_alloc();
            overlay.set_name("Overlay");
            overlay.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
            overlay.set_color(overlay_color);
            overlay.set_mouse_filter(MouseFilter::STOP);
            let overlay_input_cb = Callable::from_object_method(
                &*base,
                "_on_overlay_gui_input",
            );
            overlay.connect(&StringName::from("gui_input"), &overlay_input_cb);
            base.add_child(&overlay);
            overlay_node = Some(overlay);

            // 抽屉面板
            let mut panel = PanelContainer::new_alloc();
            panel.set_name("DrawerPanel");

            // 根据方向设置锚点和位置
            match direction {
                DIR_RIGHT => {
                    panel.set_anchor_and_offset(Side::LEFT, 1.0, -(slide_width as f32));
                    panel.set_anchor_and_offset(Side::RIGHT, 1.0, 0.0);
                    panel.set_anchor_and_offset(Side::TOP, 0.0, 0.0);
                    panel.set_anchor_and_offset(Side::BOTTOM, 1.0, 0.0);
                }
                DIR_LEFT => {
                    panel.set_anchor_and_offset(Side::LEFT, 0.0, 0.0);
                    panel.set_anchor_and_offset(Side::RIGHT, 0.0, slide_width as f32);
                    panel.set_anchor_and_offset(Side::TOP, 0.0, 0.0);
                    panel.set_anchor_and_offset(Side::BOTTOM, 1.0, 0.0);
                }
                DIR_TOP => {
                    panel.set_anchor_and_offset(Side::LEFT, 0.0, 0.0);
                    panel.set_anchor_and_offset(Side::RIGHT, 1.0, 0.0);
                    panel.set_anchor_and_offset(Side::TOP, 0.0, 0.0);
                    panel.set_anchor_and_offset(Side::BOTTOM, 0.0, slide_width as f32);
                }
                DIR_BOTTOM => {
                    panel.set_anchor_and_offset(Side::LEFT, 0.0, 0.0);
                    panel.set_anchor_and_offset(Side::RIGHT, 1.0, 0.0);
                    panel.set_anchor_and_offset(Side::TOP, 1.0, -(slide_width as f32));
                    panel.set_anchor_and_offset(Side::BOTTOM, 1.0, 0.0);
                }
                _ => {
                    // 默认右侧
                    panel.set_anchor_and_offset(Side::LEFT, 1.0, -(slide_width as f32));
                    panel.set_anchor_and_offset(Side::RIGHT, 1.0, 0.0);
                    panel.set_anchor_and_offset(Side::TOP, 0.0, 0.0);
                    panel.set_anchor_and_offset(Side::BOTTOM, 1.0, 0.0);
                }
            }

            let mut style = StyleBoxFlat::new_gd();
            style.set_bg_color(drawer_bg_color);
            style.set_border_color(drawer_border_color);
            style.set_border_width_all(1);
            style.set_corner_radius_all(corner_radius);
            panel.add_theme_stylebox_override(&StringName::from("panel"), &style);
            base.add_child(&panel);
            panel_node = Some(panel.clone());

            // 内部布局
            let mut vbox = VBoxContainer::new_alloc();
            vbox.add_theme_constant_override(&StringName::from("separation"), 4);
            panel.add_child(&vbox);

            // 标题栏（如果有标题）
            let has_title = !drawer_title.is_empty();
            if has_title {
                let mut title_bar = HBoxContainer::new_alloc();
                title_bar.add_theme_constant_override(&StringName::from("separation"), 8);

                let mut title_label = Label::new_alloc();
                title_label.set_text(&drawer_title);
                title_label.add_theme_font_size_override(&StringName::from("font_size"), 20);
                title_label.add_theme_color_override(
                    &StringName::from("font_color"),
                    Color::from_rgb(0.5, 0.85, 1.0),
                );
                title_label.set_h_size_flags(SizeFlags::EXPAND_FILL);
                title_bar.add_child(&title_label);
                title_node = Some(title_label);

                let mut close_btn = Button::new_alloc();
                close_btn.set_text(&GString::from("X"));
                close_btn.add_theme_font_size_override(&StringName::from("font_size"), 16);
                let close_cb = Callable::from_object_method(&*base, "_on_close_pressed");
                close_btn.connect(&StringName::from("pressed"), &close_cb);
                title_bar.add_child(&close_btn);

                vbox.add_child(&title_bar);

                let sep = HSeparator::new_alloc();
                vbox.add_child(&sep);
            }

            // 内容区域
            let mut content = MarginContainer::new_alloc();
            content.set_name("ContentContainer");
            content.add_theme_constant_override(&StringName::from("margin_left"), 12);
            content.add_theme_constant_override(&StringName::from("margin_right"), 12);
            content.add_theme_constant_override(&StringName::from("margin_top"), 8);
            content.add_theme_constant_override(&StringName::from("margin_bottom"), 12);
            content.set_v_size_flags(SizeFlags::EXPAND_FILL);
            vbox.add_child(&content);
            content_node = Some(content);
        }

        self.overlay = overlay_node;
        self.drawer_panel = panel_node;
        self.title_label = title_node;
        self.content_container = content_node;
    }

    /// 应用动画插值
    fn apply_animation(&mut self) {
        let t = ease_out_cubic(self.anim_progress as f32) as f64;

        // 更新遮罩透明度
        if let Some(ref overlay) = self.overlay {
            let mut o = overlay.clone();
            o.set_color(Color::from_rgba(
                self.overlay_color.r,
                self.overlay_color.g,
                self.overlay_color.b,
                self.overlay_color.a * t as f32,
            ));
        }

        // 更新面板位置
        if let Some(ref panel) = self.drawer_panel {
            let mut p = panel.clone();
            let slide_width = self.slide_width as f32;
            let offset = slide_width * (1.0 - t as f32);

            match self.direction {
                DIR_RIGHT => {
                    p.set_anchor_and_offset(Side::LEFT, 1.0, -(slide_width) + offset);
                    p.set_anchor_and_offset(Side::RIGHT, 1.0, offset);
                }
                DIR_LEFT => {
                    p.set_anchor_and_offset(Side::LEFT, 0.0, -offset);
                    p.set_anchor_and_offset(Side::RIGHT, 0.0, slide_width - offset);
                }
                DIR_TOP => {
                    p.set_anchor_and_offset(Side::TOP, 0.0, -offset);
                    p.set_anchor_and_offset(Side::BOTTOM, 0.0, slide_width - offset);
                }
                DIR_BOTTOM => {
                    p.set_anchor_and_offset(Side::TOP, 1.0, -(slide_width) + offset);
                    p.set_anchor_and_offset(Side::BOTTOM, 1.0, offset);
                }
                _ => {
                    p.set_anchor_and_offset(Side::LEFT, 1.0, -(slide_width) + offset);
                    p.set_anchor_and_offset(Side::RIGHT, 1.0, offset);
                }
            }
        }

        // 打开完成时发送信号
        if self.anim_progress >= 1.0 && self.opening {
            self.base_mut().emit_signal(&StringName::from("s_drawer_opened"), &[]);
        }
    }
}
