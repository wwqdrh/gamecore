// GdPopupPanel - 通用弹窗面板节点
// 继承 Control，内置模态遮罩、标题栏+关闭按钮、内容区域
// GML 标签：<PopupPanel popup_title="Settings" popup_width="400">
// 支持通过 GML 子节点定义弹窗内容，支持显示/隐藏切换

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, NodePath, Side};
use godot::classes::{
    IControl, Control, PanelContainer, VBoxContainer, HBoxContainer,
    Label, Button, MarginContainer, HSeparator, ColorRect, StyleBoxFlat,
    InputEvent, InputEventMouseButton, Tween,
};
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::global::MouseButton;
use godot::obj::WithBaseField;

use crate::anim::easing::ease_out_cubic;

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdPopupPanel {
    base: Base<Control>,

    #[export]
    popup_title: GString,
    #[export]
    popup_width: i32,
    #[export]
    popup_height: i32,
    #[export]
    popup_bg_color: Color,
    #[export]
    popup_border_color: Color,
    #[export]
    overlay_color: Color,
    #[export]
    title_font_size: i32,
    #[export]
    title_color: Color,
    #[export]
    corner_radius: i32,
    #[export]
    close_on_overlay: bool,
    #[export]
    animation_duration: f64,

    // 内部节点引用
    overlay: Option<Gd<ColorRect>>,
    popup_panel: Option<Gd<PanelContainer>>,
    title_label: Option<Gd<Label>>,
    content_container: Option<Gd<MarginContainer>>,
    is_visible: bool,
    ui_built: bool,
    // 动画状态
    anim_tween: Option<Gd<Tween>>,
}

#[godot_api]
impl IControl for GdPopupPanel {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            popup_title: GString::from("Popup"),
            popup_width: 400,
            popup_height: 400,
            popup_bg_color: Color::from_rgba(0.08, 0.08, 0.14, 0.95),
            popup_border_color: Color::from_rgb(0.35, 0.4, 0.55),
            overlay_color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            title_font_size: 20,
            title_color: Color::from_rgb(0.4, 0.8, 1.0),
            corner_radius: 8,
            close_on_overlay: true,
            animation_duration: 0.25,
            overlay: None,
            popup_panel: None,
            title_label: None,
            content_container: None,
            is_visible: false,
            ui_built: false,
            anim_tween: None,
        }
    }

    fn ready(&mut self) {
        // build_ui 已在 builder 或 init 中调用，此处仅确保初始隐藏
        if !self.ui_built {
            self.build_ui();
        }
        self.hide_popup();
    }
}

#[godot_api]
impl GdPopupPanel {
    #[signal]
    fn s_popup_shown();

    #[signal]
    fn s_popup_hidden();

    /// 显示弹窗（带弹入动画）
    #[func]
    fn show_popup(&mut self) {
        self.is_visible = true;
        self.base_mut().set_visible(true);
        self.base_mut().emit_signal(&StringName::from("s_popup_shown"), &[]);

        // 弹入动画
        self.play_enter_animation();
    }

    /// 隐藏弹窗（带弹出动画）
    #[func]
    fn hide_popup(&mut self) {
        // 播放退出动画，动画结束后隐藏
        self.play_exit_animation();
    }

    /// 弹窗是否可见
    #[func]
    fn is_popup_visible(&self) -> bool {
        self.is_visible
    }

    /// 切换弹窗显示/隐藏
    #[func]
    fn toggle_popup(&mut self) {
        if self.is_visible {
            self.hide_popup();
        } else {
            self.show_popup();
        }
    }

    /// 设置弹窗标题（同时更新已显示的标题 Label）
    #[func]
    fn set_popup_title_text(&mut self, text: GString) {
        self.popup_title = text.clone();
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
    fn add_content_child(&mut self, mut child: Gd<Node>) {
        if let Some(ref mut cc) = self.content_container {
            cc.add_child(&child);
            child.set_owner(&cc.clone().upcast::<Node>());
        } else {
            godot_error!("[PopupPanel] add_content_child: content_container is None!");
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
                self.hide_popup();
            }
        }
    }

    /// 处理关闭按钮点击
    #[func]
    fn _on_close_pressed(&mut self) {
        self.hide_popup();
    }

    /// 退出动画完成后的回调
    #[func]
    fn _on_exit_anim_finished(&mut self) {
        self.base_mut().set_visible(false);
        self.base_mut().emit_signal(&StringName::from("s_popup_hidden"), &[]);
        // 恢复面板缩放和透明度
        if let Some(ref panel) = self.popup_panel {
            let mut p = panel.clone();
            p.set_scale(Vector2::new(1.0, 1.0));
            p.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 1.0));
        }
    }

    /// 确保内部 UI 已构建（供 builder 在添加子节点前调用）
    #[func]
    fn ensure_ui_built(&mut self) {
        //godot_print!("[PopupPanel] ensure_ui_built called, ui_built={}", self.ui_built);
        if !self.ui_built {
            self.build_ui();
        }
    }

    /// 重新计算弹窗布局（popup_width 变化后调用）
    /// 更新 PanelContainer 的 offset 使其居中并匹配新的宽度
    #[func]
    fn update_layout(&mut self) {
        if let Some(ref panel) = self.popup_panel {
            let half_w = self.popup_width as f32 / 2.0;
            let half_h = self.popup_height as f32 / 2.0;
            let mut p = panel.clone();
            p.set_offset(Side::LEFT, -half_w);
            p.set_offset(Side::RIGHT, half_w);
            p.set_offset(Side::TOP, -half_h);
            p.set_offset(Side::BOTTOM, half_h);
        }
    }
}

impl GdPopupPanel {
    /// 构建 UI 结构
    fn build_ui(&mut self) {
        if self.ui_built {
            return;
        }
        self.ui_built = true;
        //godot_print!("[PopupPanel] build_ui called, popup_title={}, popup_width={}", self.popup_title, self.popup_width);

        // 先读取属性值，避免借用冲突
        let overlay_color = self.overlay_color;
        let popup_width = self.popup_width;
        let popup_height = self.popup_height;
        let popup_bg_color = self.popup_bg_color;
        let popup_border_color = self.popup_border_color;
        let corner_radius = self.corner_radius;
        let popup_title = self.popup_title.clone();
        let title_font_size = self.title_font_size;
        let title_color = self.title_color;

        // 临时变量存储创建的节点
        let mut overlay_node: Option<Gd<ColorRect>> = None;
        let mut panel_node: Option<Gd<PanelContainer>> = None;
        let mut title_label_node: Option<Gd<Label>> = None;
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

            // 弹窗容器（居中）
            let mut panel = PanelContainer::new_alloc();
            panel.set_name("PopupContainer");
            panel.set_anchors_and_offsets_preset(LayoutPreset::CENTER);
            let half_w = popup_width as f32 / 2.0;
            let half_h = popup_height as f32 / 2.0;
            panel.set_offset(Side::LEFT, -half_w);
            panel.set_offset(Side::RIGHT, half_w);
            panel.set_offset(Side::TOP, -half_h);
            panel.set_offset(Side::BOTTOM, half_h);
            let mut style = StyleBoxFlat::new_gd();
            style.set_bg_color(popup_bg_color);
            style.set_border_color(popup_border_color);
            style.set_border_width_all(2);
            style.set_corner_radius_all(corner_radius);
            panel.add_theme_stylebox_override(&StringName::from("panel"), &style);
            base.add_child(&panel);
            panel_node = Some(panel.clone());

            // 弹窗内部垂直布局
            let mut vbox = VBoxContainer::new_alloc();
            vbox.add_theme_constant_override(&StringName::from("separation"), 8);
            panel.add_child(&vbox);

            // 标题栏
            let mut title_bar = HBoxContainer::new_alloc();
            vbox.add_child(&title_bar);

            let mut title_label = Label::new_alloc();
            title_label.set_text(&popup_title);
            title_label.add_theme_font_size_override(&StringName::from("font_size"), title_font_size);
            title_label.add_theme_color_override(&StringName::from("font_color"), title_color);
            title_label.set_h_size_flags(SizeFlags::EXPAND_FILL);
            title_bar.add_child(&title_label);
            title_label_node = Some(title_label);

            let mut close_btn = Button::new_alloc();
            close_btn.set_text(&GString::from("X"));
            close_btn.set_custom_minimum_size(Vector2::new(32.0, 32.0));
            close_btn.add_theme_font_size_override(&StringName::from("font_size"), 18);
            close_btn.add_theme_color_override(&StringName::from("font_color"), Color::from_rgb(1.0, 1.0, 1.0));
            close_btn.add_theme_color_override(&StringName::from("font_hover_color"), Color::from_rgb(1.0, 1.0, 1.0));
            close_btn.add_theme_color_override(&StringName::from("font_pressed_color"), Color::from_rgb(0.9, 0.9, 0.9));
            // normal 状态：圆形红色
            let mut close_normal = StyleBoxFlat::new_gd();
            close_normal.set_bg_color(Color::from_rgb(0.96, 0.26, 0.26));
            close_normal.set_corner_radius_all(16);
            close_normal.set_content_margin_all(4.0);
            close_normal.set_border_width_all(0);
            close_btn.add_theme_stylebox_override(&StringName::from("normal"), &close_normal);
            // hover 状态：稍亮红色
            let mut close_hover = StyleBoxFlat::new_gd();
            close_hover.set_bg_color(Color::from_rgb(1.0, 0.4, 0.4));
            close_hover.set_corner_radius_all(16);
            close_hover.set_content_margin_all(4.0);
            close_hover.set_border_width_all(0);
            close_btn.add_theme_stylebox_override(&StringName::from("hover"), &close_hover);
            // pressed 状态：深红色
            let mut close_pressed = StyleBoxFlat::new_gd();
            close_pressed.set_bg_color(Color::from_rgb(0.8, 0.18, 0.18));
            close_pressed.set_corner_radius_all(16);
            close_pressed.set_content_margin_all(4.0);
            close_pressed.set_border_width_all(0);
            close_btn.add_theme_stylebox_override(&StringName::from("pressed"), &close_pressed);
            let close_cb = Callable::from_object_method(
                &*base,
                "_on_close_pressed",
            );
            close_btn.connect(&StringName::from("pressed"), &close_cb);
            title_bar.add_child(&close_btn);

            // 分隔线
            let sep = HSeparator::new_alloc();
            vbox.add_child(&sep);

            // 内容区域
            let mut content = MarginContainer::new_alloc();
            content.set_name("ContentContainer");
            content.add_theme_constant_override(&StringName::from("margin_left"), 16);
            content.add_theme_constant_override(&StringName::from("margin_right"), 16);
            content.add_theme_constant_override(&StringName::from("margin_top"), 8);
            content.add_theme_constant_override(&StringName::from("margin_bottom"), 16);
            content.set_v_size_flags(SizeFlags::EXPAND_FILL);
            vbox.add_child(&content);
            content_node = Some(content);
        }

        // 在借用结束后赋值
        self.overlay = overlay_node;
        self.popup_panel = panel_node;
        self.title_label = title_label_node;
        self.content_container = content_node;
    }

    /// 播放弹入动画
    fn play_enter_animation(&mut self) {
        // 停止之前的动画
        if let Some(ref mut tween) = self.anim_tween {
            if tween.is_valid() {
                tween.kill();
            }
        }

        let duration = self.animation_duration;

        if let Some(ref panel) = self.popup_panel {
            let mut p = panel.clone();
            // 设置初始状态
            p.set_scale(Vector2::new(0.85, 0.85));
            p.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));

            let mut tween = p.create_tween();

            // 缩放动画
            tween.tween_property(
                &p,
                &NodePath::from("scale"),
                &Vector2::new(1.0, 1.0).to_variant(),
                duration,
            ).set_trans(godot::classes::tween::TransitionType::BACK)
             .set_ease(godot::classes::tween::EaseType::OUT);

            // 并行：淡入
            let mut alpha_tween = tween.parallel();
            alpha_tween.tween_property(
                &p,
                &NodePath::from("modulate:a"),
                &1.0_f32.to_variant(),
                duration * 0.6,
            );

            self.anim_tween = Some(tween);
        }

        // 遮罩淡入
        if let Some(ref overlay) = self.overlay {
            let mut o = overlay.clone();
            o.set_color(Color::from_rgba(
                self.overlay_color.r,
                self.overlay_color.g,
                self.overlay_color.b,
                0.0,
            ));
            let mut tween = o.create_tween();
            tween.tween_property(
                &o,
                &NodePath::from("color:a"),
                &self.overlay_color.a.to_variant(),
                duration * 0.6,
            );
        }
    }

    /// 播放弹出动画
    fn play_exit_animation(&mut self) {
        // 停止之前的动画
        if let Some(ref mut tween) = self.anim_tween {
            if tween.is_valid() {
                tween.kill();
            }
        }

        let duration = self.animation_duration * 0.6; // 退出更快

        if let Some(ref panel) = self.popup_panel {
            let mut p = panel.clone();

            let mut tween = p.create_tween();

            // 缩放动画
            tween.tween_property(
                &p,
                &NodePath::from("scale"),
                &Vector2::new(0.85, 0.85).to_variant(),
                duration,
            ).set_trans(godot::classes::tween::TransitionType::SINE)
             .set_ease(godot::classes::tween::EaseType::IN);

            // 并行：淡出
            let mut alpha_tween = tween.parallel();
            alpha_tween.tween_property(
                &p,
                &NodePath::from("modulate:a"),
                &0.0_f32.to_variant(),
                duration,
            );

            // 完成后隐藏（使用对象方法回调）
            let base = self.base_mut().clone();
            let cb = Callable::from_object_method(&base, "_on_exit_anim_finished");
            tween.tween_callback(&cb);

            self.anim_tween = Some(tween);
        }

        // 遮罩淡出
        if let Some(ref overlay) = self.overlay {
            let mut o = overlay.clone();
            let mut tween = o.create_tween();
            tween.tween_property(
                &o,
                &NodePath::from("color:a"),
                &0.0_f32.to_variant(),
                duration,
            );
        }

        self.is_visible = false;
    }
}
