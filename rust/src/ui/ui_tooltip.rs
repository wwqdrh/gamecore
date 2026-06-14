// GdUITooltip - 鼠标跟随提示框节点
// 继承 Control，浮动面板跟随鼠标位置显示
// 支持延迟显示、自动位置调整、标题+内容布局
// 支持自定义子节点（GML 中定义 Label 等），通过 update_data 解析 {{key}} 模板绑定
// GML 标签：<Tooltip name="MyTooltip" tooltip_title="标题" tooltip_content="内容" max_height="100">
//           <Tooltip name="MyTooltip" max_width="250" max_height="100">
//             <Label text="{{name}}" />
//             <Label text="{{desc}}" />
//           </Tooltip>

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Vector2, Dictionary, Variant};
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
    max_height: i32,
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
    vbox: Option<Gd<VBoxContainer>>,
    title_label: Option<Gd<Label>>,
    content_label: Option<Gd<Label>>,
    is_showing: bool,
    delay_timer: f64,
    ui_built: bool,
    has_custom_content: bool,
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
            max_height: 0,
            bg_color: Color::from_rgba(0.1, 0.1, 0.18, 0.95),
            border_color: Color::from_rgb(0.4, 0.5, 0.7),
            title_color: Color::from_rgb(0.5, 0.85, 1.0),
            content_color: Color::from_rgb(0.85, 0.85, 0.9),
            corner_radius: 6,
            panel: None,
            vbox: None,
            title_label: None,
            content_label: None,
            is_showing: false,
            delay_timer: 0.0,
            ui_built: false,
            has_custom_content: false,
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
    /// 首次添加自定义子节点时，移除内置的 title/content/separator
    #[func]
    fn add_content_child(&mut self, mut child: Gd<godot::classes::Node>) {
        let _child_name = child.get_name().to_string();
        //godot_print!("[UITooltip] add_content_child: name='{}', has_custom_content={}, panel={}", child_name, self.has_custom_content, self.panel.is_some());
        // 首次添加自定义子节点时，移除内置的 title/content/separator
        if !self.has_custom_content {
            self.has_custom_content = true;
            self.remove_builtin_labels();
        }
        if let Some(ref mut vbox) = self.vbox {
            vbox.add_child(&child);
            child.set_owner(&vbox.clone().upcast::<godot::classes::Node>());
        } else {
            godot_error!("[UITooltip] add_content_child: vbox is None!");
        }
    }

    /// 更新数据：解析 {{key}} 模板绑定，将数据字典中的值设置到子节点属性上
    /// 供 UIHList/UIGrid 的 show_tooltip_for_item 调用，传递列表项的完整数据
    #[func]
    fn update_data(&mut self, data: Dictionary<Variant, Variant>) {
        //godot_print!("[UITooltip] update_data called, vbox={}", self.vbox.is_some());
        if let Some(ref vbox) = self.vbox {
            let keys = data.keys_array();
            let simple_keys: Vec<(String, Variant)> = (0..keys.len())
                .filter_map(|i| {
                    keys.get(i).map(|key_var| {
                        let key = key_var.to_string();
                        let val = data.get(&key_var).unwrap_or(Variant::nil());
                        (key, val)
                    })
                })
                .collect();

            //godot_print!("[UITooltip] update_data: keys={:?}", simple_keys.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>());

            let mut vbox = vbox.clone().upcast::<Control>();
            // 打印 vbox 子节点信息
            let child_count = vbox.get_child_count();
            for i in 0..child_count {
                if let Some(c) = vbox.get_child(i) {
                    let _name = c.get_name().to_string();
                    let _has_tpl = if let Ok(ctrl) = c.clone().try_cast::<Control>() {
                        ctrl.has_meta(&StringName::from("__tpl_keys"))
                    } else {
                        false
                    };
                    //godot_print!("[UITooltip] update_data: vbox child[{}] name='{}', has_tpl_keys={}", i, name, has_tpl);
                }
            }
            self.resolve_template_bindings(&mut vbox, &simple_keys);
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
        let max_height = self.max_height;
        let tooltip_title = self.tooltip_title_text.clone();
        let tooltip_content = self.tooltip_content_text.clone();
        let title_color = self.title_color;
        let content_color = self.content_color;

        let mut panel_node: Option<Gd<PanelContainer>> = None;
        let mut vbox_node: Option<Gd<VBoxContainer>> = None;
        let mut title_node: Option<Gd<Label>> = None;
        let mut content_node: Option<Gd<Label>> = None;

        {
            let mut base = self.base_mut();
            base.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
            base.set_mouse_filter(MouseFilter::IGNORE);

            // 提示框面板
            let mut panel = PanelContainer::new_alloc();
            panel.set_name("TooltipPanel");
            panel.set_custom_minimum_size(Vector2::new(max_width as f32, if max_height > 0 { max_height as f32 } else { 0.0 }));
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
                sep.set_name("TooltipSeparator");
                sep.set_mouse_filter(MouseFilter::IGNORE);
                vbox.add_child(&sep);
            }

            // 内容文本
            let has_content = !tooltip_content.is_empty();
            if has_content {
                let mut content_label = Label::new_alloc();
                content_label.set_name("TooltipContent");
                content_label.set_text(&tooltip_content);
                content_label.add_theme_font_size_override(&StringName::from("font_size"), 14);
                content_label.add_theme_color_override(&StringName::from("font_color"), content_color);
                content_label.set_autowrap_mode(AutowrapMode::WORD_SMART);
                content_label.set_mouse_filter(MouseFilter::IGNORE);
                vbox.add_child(&content_label);
                content_node = Some(content_label);
            }

            panel.add_child(&vbox);
            base.add_child(&panel);
            panel_node = Some(panel);
            vbox_node = Some(vbox);
        }

        self.panel = panel_node;
        self.vbox = vbox_node;
        self.title_label = title_node;
        self.content_label = content_node;
    }

    /// 移除内置的 title/content label 和 separator
    fn remove_builtin_labels(&mut self) {
        if let Some(ref mut vbox) = self.vbox {
            let mut children_to_remove: Vec<Gd<godot::classes::Node>> = Vec::new();
            let child_count = vbox.get_child_count();
            for i in 0..child_count {
                if let Some(child) = vbox.get_child(i) {
                    let name = child.get_name().to_string();
                    if name == "TooltipTitle" || name == "TooltipContent" || name == "TooltipSeparator" {
                        children_to_remove.push(child);
                    }
                }
            }
            for mut child in children_to_remove {
                vbox.remove_child(&child);
                child.queue_free();
            }
        }
        self.title_label = None;
        self.content_label = None;
    }

    /// 递归解析模板绑定：遍历节点及其子节点，查找 __tpl_keys 和 __tpl_attr 元数据，
    /// 将数据字典中对应的值设置到节点的属性上
    fn resolve_template_bindings(
        &self,
        node: &mut Gd<Control>,
        simple_keys: &[(String, Variant)],
    ) {
        let node_name = node.get_name().to_string();
        // 检查当前节点的模板绑定
        if node.has_meta(&StringName::from("__tpl_keys")) {
            let tpl_keys_var = node.get_meta(&StringName::from("__tpl_keys"));
            if tpl_keys_var.get_type() == godot::builtin::VariantType::STRING {
                let keys_str = tpl_keys_var.to_string();
                //godot_print!("[UITooltip] resolve_template: node='{}' has __tpl_keys='{}'", node_name, keys_str);
                for attr_name in keys_str.split(',') {
                    let attr_name = attr_name.trim();
                    if attr_name.is_empty() {
                        continue;
                    }
                    let tpl_meta_key = format!("__tpl_{}", attr_name);
                    if !node.has_meta(&StringName::from(tpl_meta_key.as_str())) {
                        godot_warn!("[UITooltip] resolve_template: node='{}' missing meta '{}'", node_name, tpl_meta_key);
                        continue;
                    }
                    let data_key_var = node.get_meta(&StringName::from(tpl_meta_key.as_str()));
                    if data_key_var.get_type() == godot::builtin::VariantType::STRING {
                        let data_key = data_key_var.to_string();
                        for (key, val) in simple_keys {
                            if key == &data_key {
                                //godot_print!("[UITooltip] resolve_template: node='{}' set {} = {} (data_key={})", node_name, attr_name, val, data_key);
                                node.set(&StringName::from(attr_name), val);
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 递归处理子节点
        let child_count = node.get_child_count();
        for i in 0..child_count {
            if let Some(child) = node.get_child(i) {
                if let Ok(mut child_ctrl) = child.try_cast::<Control>() {
                    self.resolve_template_bindings(&mut child_ctrl, simple_keys);
                }
            }
        }
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
