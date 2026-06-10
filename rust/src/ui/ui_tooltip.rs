// GdUITooltip - 鼠标跟随提示框节点
// 继承 Control，浮动面板跟随鼠标位置显示
// 支持延迟显示、自动位置调整、标题+内容布局
// GML 标签：<Tooltip name="MyTooltip" tooltip_title="标题" tooltip_content="内容">

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Vector2};
use godot::classes::{
    IControl, Control, PanelContainer, VBoxContainer,
    Label, HSeparator, StyleBoxFlat,
};
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::obj::WithBaseField;

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdUITooltip {
    base: Base<Control>,

    #[export]
    tooltip_title_text: GString,
    #[export]
    tooltip_content_text: GString,
    #[export]
    delay: f64,
    #[export]
    offset_x: f32,
    #[export]
    offset_y: f32,
    #[export]
    max_width: i32,
    #[export]
    bg_color: Color,
    #[export]
    border_color: Color,
    #[export]
    title_color: Color,
    #[export]
    content_color: Color,
    #[export]
    corner_radius: i32,

    // 内部节点引用
    panel: Option<Gd<PanelContainer>>,
    title_label: Option<Gd<Label>>,
    content_label: Option<Gd<Label>>,
    is_showing: bool,
    delay_timer: f64,
    ui_built: bool,
}

#[godot_api]
impl IControl for GdUITooltip {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            tooltip_title_text: GString::new(),
            tooltip_content_text: GString::new(),
            delay: 0.3,
            offset_x: 12.0,
            offset_y: 12.0,
            max_width: 300,
            bg_color: Color::from_rgba(0.1, 0.1, 0.18, 0.95),
            border_color: Color::from_rgb(0.4, 0.5, 0.7),
            title_color: Color::from_rgb(0.5, 0.85, 1.0),
            content_color: Color::from_rgb(0.85, 0.85, 0.9),
            corner_radius: 6,
            panel: None,
            title_label: None,
            content_label: None,
            is_showing: false,
            delay_timer: 0.0,
            ui_built: false,
        }
    }

    fn ready(&mut self) {
        if !self.ui_built {
            self.build_ui();
        }
        self.base_mut().set_visible(false);
        // Tooltip 不拦截鼠标事件
        self.base_mut().set_mouse_filter(MouseFilter::IGNORE);
    }

    fn process(&mut self, delta: f64) {
        if !self.is_showing {
            return;
        }

        // 延迟显示
        if self.delay_timer < self.delay {
            self.delay_timer += delta;
            if self.delay_timer >= self.delay {
                self.base_mut().set_visible(true);
                self.update_position();
            }
            return;
        }

        // 已显示，持续跟随鼠标
        self.update_position();
    }
}

#[godot_api]
impl GdUITooltip {
    #[signal]
    fn s_tooltip_shown();

    #[signal]
    fn s_tooltip_hidden();

    /// 显示提示框（开始延迟计时）
    #[func]
    fn show_tooltip(&mut self) {
        self.is_showing = true;
        self.delay_timer = 0.0;
        if self.delay <= 0.0 {
            self.base_mut().set_visible(true);
            self.update_position();
        }
    }

    /// 隐藏提示框
    #[func]
    fn hide_tooltip(&mut self) {
        self.is_showing = false;
        self.delay_timer = 0.0;
        self.base_mut().set_visible(false);
        self.base_mut().emit_signal(&StringName::from("s_tooltip_hidden"), &[]);
    }

    /// 设置提示框标题
    #[func]
    fn set_tooltip_title(&mut self, text: GString) {
        self.tooltip_title_text = text.clone();
        if let Some(ref label) = self.title_label {
            let mut l = label.clone();
            l.set_text(&text);
        }
    }

    /// 设置提示框内容
    #[func]
    fn set_tooltip_content(&mut self, text: GString) {
        self.tooltip_content_text = text.clone();
        if let Some(ref label) = self.content_label {
            let mut l = label.clone();
            l.set_text(&text);
        }
    }

    /// 确保内部 UI 已构建（供 builder 在添加子节点前调用）
    #[func]
    fn ensure_ui_built(&mut self) {
        if !self.ui_built {
            self.build_ui();
        }
    }

    /// 添加子节点到内容区域（供 builder 调用）
    #[func]
    fn add_content_child(&mut self, mut child: Gd<godot::classes::Node>) {
        if let Some(ref mut panel) = self.panel {
            panel.add_child(&child);
            child.set_owner(&panel.clone().upcast::<godot::classes::Node>());
        } else {
            godot_error!("[UITooltip] add_content_child: panel is None!");
        }
    }
}

impl GdUITooltip {
    fn build_ui(&mut self) {
        if self.ui_built {
            return;
        }
        self.ui_built = true;

        let bg_color = self.bg_color;
        let border_color = self.border_color;
        let corner_radius = self.corner_radius;
        let max_width = self.max_width;
        let tooltip_title = self.tooltip_title_text.clone();
        let tooltip_content = self.tooltip_content_text.clone();
        let title_color = self.title_color;
        let content_color = self.content_color;

        let mut panel_node: Option<Gd<PanelContainer>> = None;
        let mut title_node: Option<Gd<Label>> = None;
        let mut content_node: Option<Gd<Label>> = None;

        {
            let mut base = self.base_mut();
            base.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
            base.set_mouse_filter(MouseFilter::IGNORE);

            // 提示框面板
            let mut panel = PanelContainer::new_alloc();
            panel.set_name("TooltipPanel");
            panel.set_custom_minimum_size(Vector2::new(max_width as f32, 0.0));
            panel.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
            panel.set_v_size_flags(SizeFlags::SHRINK_BEGIN);
            panel.set_mouse_filter(MouseFilter::IGNORE);

            let mut style = StyleBoxFlat::new_gd();
            style.set_bg_color(bg_color);
            style.set_border_color(border_color);
            style.set_border_width_all(1);
            style.set_corner_radius_all(corner_radius);
            style.set_content_margin_all(8.0);
            panel.add_theme_stylebox_override(&StringName::from("panel"), &style);

            // 内容布局
            let mut vbox = VBoxContainer::new_alloc();
            vbox.add_theme_constant_override(&StringName::from("separation"), 4);
            vbox.set_mouse_filter(MouseFilter::IGNORE);

            // 标题
            let has_title = !tooltip_title.is_empty();
            if has_title {
                let mut title_label = Label::new_alloc();
                title_label.set_name("TooltipTitle");
                title_label.set_text(&tooltip_title);
                title_label.add_theme_font_size_override(&StringName::from("font_size"), 16);
                title_label.add_theme_color_override(&StringName::from("font_color"), title_color);
                title_label.set_mouse_filter(MouseFilter::IGNORE);
                vbox.add_child(&title_label);
                title_node = Some(title_label);

                // 分隔线
                let mut sep = HSeparator::new_alloc();
                sep.set_mouse_filter(MouseFilter::IGNORE);
                vbox.add_child(&sep);
            }

            // 内容文本
            let mut content_label = Label::new_alloc();
            content_label.set_name("TooltipContent");
            content_label.set_text(&tooltip_content);
            content_label.add_theme_font_size_override(&StringName::from("font_size"), 14);
            content_label.add_theme_color_override(&StringName::from("font_color"), content_color);
            content_label.set_autowrap_mode(AutowrapMode::WORD_SMART);
            content_label.set_mouse_filter(MouseFilter::IGNORE);
            vbox.add_child(&content_label);
            content_node = Some(content_label);

            panel.add_child(&vbox);
            base.add_child(&panel);
            panel_node = Some(panel);
        }

        self.panel = panel_node;
        self.title_label = title_node;
        self.content_label = content_node;
    }

    fn update_position(&mut self) {
        let mouse_pos = self.base().get_global_mouse_position();
        let offset_x = self.offset_x;
        let offset_y = self.offset_y;

        // 获取面板大小
        let panel_size = if let Some(ref panel) = self.panel {
            panel.get_size()
        } else {
            Vector2::ZERO
        };

        // 获取视口大小
        let viewport_size = if let Some(viewport) = self.base().get_viewport() {
            viewport.get_visible_rect().size
        } else {
            Vector2::new(1920.0, 1080.0)
        };

        // 计算位置，避免超出屏幕
        let mut x = mouse_pos.x + offset_x;
        let mut y = mouse_pos.y + offset_y;

        if x + panel_size.x > viewport_size.x {
            x = mouse_pos.x - panel_size.x - offset_x;
        }
        if y + panel_size.y > viewport_size.y {
            y = mouse_pos.y - panel_size.y - offset_y;
        }

        if let Some(ref mut panel) = self.panel {
            panel.set_position(Vector2::new(x, y));
        }
    }
}
