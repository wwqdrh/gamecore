// UI 构建器
// 将解析器生成的 AST 节点树转换为 Godot Control 节点树
// 支持容器/控件/样式/信号绑定/布局属性

use std::collections::HashMap;

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Vector2, Side};
use godot::classes::{
    Control, Label, Button, Panel, PanelContainer, BaseButton,
    VBoxContainer, HBoxContainer, GridContainer, MarginContainer,
    ScrollContainer, TabContainer, CenterContainer,
    TextureRect, TextureButton, RichTextLabel, LineEdit, ProgressBar,
    SpinBox, HSeparator, VSeparator, NinePatchRect,
    StyleBoxFlat, ResourceLoader, Range, Texture2D,
    CheckButton, HSlider, ColorRect, OptionButton,
};
use godot::classes::control::LayoutPreset;
use godot::classes::texture_rect::StretchMode;
use godot::obj::NewGd;

use super::parser::{UiNode, StyleRule, ParseResult};
use super::ui_hlist::GdUIHList;
use super::ui_vlist::GdUIVList;
use super::ui_grid::GdUIGrid;
use super::ui_popup_panel::GdPopupPanel;
use super::ui_tooltip::GdUITooltip;
use super::ui_drawer::GdUIDrawer;
use super::ui_nav_menu::GdUINavMenu;

/// UI 构建器：将 AST 转换为 Godot Control 节点树
pub struct UiBuilder {
    /// 样式规则表：class_name -> StyleRule
    styles: HashMap<String, StyleRule>,
}

impl UiBuilder {
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
        }
    }

    /// 从解析结果构建 Control 节点树
    pub fn build(&mut self, parse_result: &ParseResult) -> Result<Gd<Control>, String> {
        // 构建样式索引
        for style in &parse_result.styles {
            self.styles.insert(style.class_name.clone(), style.clone());
        }

        // 创建根 Control 节点
        let mut root = Control::new_alloc();
        root.set_name("UiRoot");

        // 应用 <ui> 根属性
        for (key, value) in &parse_result.root.attributes {
            apply_root_attribute(&mut root, key, value);
        }

        // 递归构建子节点
        for child_node in &parse_result.root.children {
            let mut child_control = self.build_node(child_node)?;
            root.add_child(&child_control);
            child_control.set_owner(&root);
        }

        // 后处理：解析内部信号绑定（show:/hide:/toggle:NodeName）
        resolve_internal_signals(&mut root);

        Ok(root)
    }

    /// 构建单个 AST 节点为 Control
    fn build_node(&self, node: &UiNode) -> Result<Gd<Control>, String> {
        let mut control = self.instantiate_control(&node.tag)?;

        // 设置节点名
        if let Some(name) = node.attributes.iter().find(|(k, _)| k == "name") {
            control.set_name(&StringName::from(&name.1));
        }
        // Tab 标签的 title 属性覆盖节点名（TabContainer 用节点名作为 tab 标题）
        if node.tag == "Tab" {
            if let Some(title) = node.attributes.iter().find(|(k, _)| k == "title") {
                control.set_name(&StringName::from(&title.1));
            }
        }

        // 应用属性
        let mut class_name: Option<String> = None;
        for (key, value) in &node.attributes {
            match key.as_str() {
                "class" => class_name = Some(value.clone()),
                "name" => { /* 已处理 */ }
                "title" => { /* Tab 标签的 title 已在上方处理（设置节点名） */ }
                _ => {
                    if key.starts_with("on_") {
                        // 信号绑定延迟处理（需要节点在场景树中）
                        // 存储为元数据，由 connect_signals 方法连接
                        let signal_name = &key[3..];
                        control.set_meta(
                            &StringName::from(format!("__signal_{}", signal_name).as_str()),
                            &value.to_variant(),
                        );
                    } else {
                        control = apply_attribute(control, &node.tag, key, value);
                    }
                }
            }
        }

        // 应用 class 样式
        if let Some(ref cn) = class_name {
            self.apply_class_style(&mut control, &node.tag, cn);
        }

        // PopupPanel/Drawer/Tooltip：属性设置完成后立即构建内部 UI
        // 这样 ContentContainer 在添加子节点前就已存在
        // NavMenu 不在此处构建，因为需要先添加 NavItem 子节点再解析数据，由 ready() 处理
        if node.tag == "PopupPanel" || node.tag == "Drawer" || node.tag == "Tooltip" {
            control.call(&StringName::from("ensure_ui_built"), &[]);
        }

        // 递归构建子节点
        for child_node in &node.children {
            let mut child_control = self.build_node(child_node)?;

            // NavItem：设置 meta 标记（NavMenu 和 NavItem 的子 NavItem 都需要标记）
            if child_node.tag == "NavItem" {
                child_control.set_meta(&StringName::from("__nav_item"), &true.to_variant());
            }

            // PopupPanel 的子节点添加到内容区域
            if node.tag == "PopupPanel" || node.tag == "Drawer" || node.tag == "Tooltip" {
                control.call(
                    &StringName::from("add_content_child"),
                    &[child_control.clone().upcast::<godot::classes::Node>().to_variant()],
                );
                continue;
            }
            control.add_child(&child_control);
            // 列表容器的子节点（slot 模板）不设置 owner，避免运行时 duplicate 后的 owner 不一致警告
            if node.tag != "UIHList" && node.tag != "UIVList" && node.tag != "UIGrid" {
                child_control.set_owner(&control);
            }
        }

        // 列表扩展节点：子节点构建完成后调用 initial()
        match node.tag.as_str() {
            "UIHList" | "UIVList" | "UIGrid" => {
                control.call(&StringName::from("initial"), &[]);
            }
            _ => {}
        }

        Ok(control)
    }

    /// 根据标签名实例化对应的 Godot Control
    fn instantiate_control(&self, tag: &str) -> Result<Gd<Control>, String> {
        let control: Gd<Control> = match tag {
            // 容器
            "VBoxContainer" => VBoxContainer::new_alloc().upcast(),
            "HBoxContainer" => HBoxContainer::new_alloc().upcast(),
            "GridContainer" => GridContainer::new_alloc().upcast(),
            "MarginContainer" => MarginContainer::new_alloc().upcast(),
            "ScrollContainer" => ScrollContainer::new_alloc().upcast(),
            "TabContainer" => TabContainer::new_alloc().upcast(),
            "CenterContainer" => CenterContainer::new_alloc().upcast(),
            "PanelContainer" => PanelContainer::new_alloc().upcast(),
            // Tab 页容器（TabContainer 的子标签）
            "Tab" => VBoxContainer::new_alloc().upcast(),
            // 控件
            "Label" => Label::new_alloc().upcast(),
            "Button" => Button::new_alloc().upcast(),
            "TextureButton" => TextureButton::new_alloc().upcast(),
            "Panel" => Panel::new_alloc().upcast(),
            "TextureRect" => TextureRect::new_alloc().upcast(),
            "RichTextLabel" => RichTextLabel::new_alloc().upcast(),
            "LineEdit" => LineEdit::new_alloc().upcast(),
            "ProgressBar" => ProgressBar::new_alloc().upcast(),
            "SpinBox" => SpinBox::new_alloc().upcast(),
            "HSeparator" => HSeparator::new_alloc().upcast(),
            "VSeparator" => VSeparator::new_alloc().upcast(),
            "NinePatchRect" => NinePatchRect::new_alloc().upcast(),
            // 表单控件
            "CheckButton" => CheckButton::new_alloc().upcast(),
            "HSlider" => HSlider::new_alloc().upcast(),
            "ColorRect" => ColorRect::new_alloc().upcast(),
            "OptionButton" => OptionButton::new_alloc().upcast(),
            // 弹窗面板
            "PopupPanel" => GdPopupPanel::new_alloc().upcast(),
            // 提示框
            "Tooltip" => GdUITooltip::new_alloc().upcast(),
            // 抽屉面板
            "Drawer" => GdUIDrawer::new_alloc().upcast(),
            // 导航菜单
            "NavMenu" => GdUINavMenu::new_alloc().upcast(),
            // 导航菜单项（递归嵌套，使用 Control 占位）
            "NavItem" => Control::new_alloc(),
            // 列表扩展节点
            "UIHList" => GdUIHList::new_alloc().upcast(),
            "UIVList" => GdUIVList::new_alloc().upcast(),
            "UIGrid" => GdUIGrid::new_alloc().upcast(),
            // 通用 Control
            "Control" => Control::new_alloc(),
            _ => {
                godot_print!("[UiBuilder] Unknown tag '{}', falling back to Control", tag);
                Control::new_alloc()
            }
        };
        Ok(control)
    }

    /// 应用 class 样式到控件
    fn apply_class_style(&self, control: &mut Gd<Control>, tag: &str, class_name: &str) {
        if let Some(style_rule) = self.styles.get(class_name) {
            // 创建 StyleBoxFlat 并应用样式属性
            let mut style_box = StyleBoxFlat::new_gd();
            let bg_color_key = if style_rule.properties.contains_key("background") {
                "background"
            } else if style_rule.properties.contains_key("bg_color") {
                "bg_color"
            } else {
                ""
            };

            if !bg_color_key.is_empty() {
                if let Some(color) = parse_color(style_rule.properties.get(bg_color_key).unwrap()) {
                    style_box.set_bg_color(color);
                }
            }

            if let Some(border_radius) = style_rule.properties.get("border_radius") {
                if let Ok(r) = border_radius.parse::<i32>() {
                    style_box.set_corner_radius_all(r);
                }
            }

            if let Some(border_color) = style_rule.properties.get("border_color") {
                if let Some(color) = parse_color(border_color) {
                    style_box.set_border_color(color);
                    let border_width = style_rule.properties.get("border_width")
                        .and_then(|v| v.parse::<i32>().ok())
                        .unwrap_or(1);
                    style_box.set_border_width_all(border_width);
                }
            }

            if let Some(padding) = style_rule.properties.get("padding") {
                if let Ok(p) = padding.parse::<i32>() {
                    style_box.set_content_margin_all(p as f32);
                }
            }

            // 应用 color 属性（文字颜色）到控件
            if let Some(color_str) = style_rule.properties.get("color") {
                if let Some(color) = parse_color(color_str) {
                    apply_text_color(control, tag, color);
                }
            }

            // 应用 texture 属性（纹理）到 TextureButton
            if let Some(texture_path) = style_rule.properties.get("texture") {
                if tag == "TextureButton" {
                    let path = GString::from(texture_path);
                    if let Some(res) = ResourceLoader::singleton().load(&path) {
                        if let Ok(tex) = res.try_cast::<Texture2D>() {
                            let mut tb = control.clone().cast::<TextureButton>();
                            tb.set_texture_normal(&tex);
                        }
                    }
                }
            }

            // 将 StyleBox 应用到控件
            let stylebox_name = get_stylebox_name_for_tag(tag);
            control.add_theme_stylebox_override(
                &StringName::from(stylebox_name),
                &style_box,
            );
        }
    }
}

/// 应用根节点属性
fn apply_root_attribute(control: &mut Gd<Control>, key: &str, value: &str) {
    match key {
        "theme" => {
            // 主题名称，暂不实现主题加载
        }
        "anchor" => {
            apply_anchor(control, value);
        }
        _ => {
            // 根属性也用通用方法处理
            match key {
                "margin" => apply_margin(control, value),
                "size" => apply_size(control, value),
                "visible" => control.set_visible(value != "false" && value != "0"),
                _ => {
                    godot_print!("[UiBuilder] Unhandled root attribute: {}='{}'", key, value);
                }
            }
        }
    }
}

/// 应用通用属性到控件（消耗并返回 Gd<Control>，因为 cast() 消耗 self）
fn apply_attribute(mut control: Gd<Control>, tag: &str, key: &str, value: &str) -> Gd<Control> {
    // 模板绑定语法：{{data_key}} — 不设置属性值，而是记录绑定关系
    // 当 UIHList/UIGrid 的 update() 被调用时，根据绑定关系从数据中取值
    if value.starts_with("{{") && value.ends_with("}}") && value.len() > 4 {
        let data_key = &value[2..value.len()-2];
        let tpl_meta_key = format!("__tpl_{}", key);
        control.set_meta(&StringName::from(tpl_meta_key.as_str()), &data_key.to_variant());
        // 记录该节点有哪些模板绑定属性（逗号分隔）
        let keys_str = if control.has_meta(&StringName::from("__tpl_keys")) {
            let existing = control.get_meta(&StringName::from("__tpl_keys"));
            if existing.get_type() == godot::builtin::VariantType::STRING {
                let mut s = existing.to_string();
                s.push(',');
                s.push_str(key);
                s
            } else {
                key.to_string()
            }
        } else {
            key.to_string()
        };
        control.set_meta(&StringName::from("__tpl_keys"), &keys_str.to_variant());
        return control;
    }

    match key {
        "text" => {
            match tag {
                "NavItem" => {
                    // NavItem 的 text 存储为 __nav_text meta
                    control.set_meta(&StringName::from("__nav_text"), &value.to_variant());
                }
                "Label" => {
                    let mut lbl = control.cast::<Label>();
                    lbl.set_text(&GString::from(value));
                    return lbl.upcast();
                }
                "Button" | "CheckButton" => {
                    let mut btn = control.cast::<Button>();
                    btn.set_text(&GString::from(value));
                    return btn.upcast();
                }
                "TextureButton" => {
                    // TextureButton 不支持 text，叠加一个居中 Label 显示文字
                    let mut lbl = Label::new_alloc();
                    lbl.set_text(&GString::from(value));
                    // 手动设置锚点和偏移，确保 Label 填满整个 TextureButton
                    lbl.set_anchor(Side::LEFT, 0.0);
                    lbl.set_anchor(Side::RIGHT, 1.0);
                    lbl.set_anchor(Side::TOP, 0.0);
                    lbl.set_anchor(Side::BOTTOM, 1.0);
                    lbl.set_offset(Side::LEFT, 0.0);
                    lbl.set_offset(Side::RIGHT, 0.0);
                    lbl.set_offset(Side::TOP, 0.0);
                    lbl.set_offset(Side::BOTTOM, 0.0);
                    lbl.set_horizontal_alignment(godot::global::HorizontalAlignment::CENTER);
                    lbl.set_vertical_alignment(godot::global::VerticalAlignment::CENTER);
                    lbl.set_mouse_filter(godot::classes::control::MouseFilter::IGNORE);
                    control.add_child(&lbl);
                    lbl.set_owner(&control);
                    // 存储文字到 meta，以便样式中的 color 属性能正确应用
                    control.set_meta(&StringName::from("__has_text_label"), &true.to_variant());
                    return control;
                }
                "RichTextLabel" => {
                    let mut rt = control.cast::<RichTextLabel>();
                    rt.set_text(&GString::from(value));
                    return rt.upcast();
                }
                "LineEdit" => {
                    let mut le = control.cast::<LineEdit>();
                    le.set_text(&GString::from(value));
                    return le.upcast();
                }
                _ => {
                    godot_print!("[UiBuilder] Cannot set text on <{}>", tag);
                }
            }
        }
        "font_size" => {
            if let Ok(size) = value.parse::<i32>() {
                control.add_theme_font_size_override(
                    &StringName::from("font_size"),
                    size,
                );
            }
        }
        "align" => {
            use godot::global::HorizontalAlignment;
            let alignment = match value {
                "left" => HorizontalAlignment::LEFT,
                "center" => HorizontalAlignment::CENTER,
                "right" => HorizontalAlignment::RIGHT,
                "fill" => HorizontalAlignment::FILL,
                _ => HorizontalAlignment::LEFT,
            };
            match tag {
                "Label" => {
                    let mut lbl = control.cast::<Label>();
                    lbl.set_horizontal_alignment(alignment);
                    return lbl.upcast();
                }
                _ => {}
            }
        }
        "anchor" => apply_anchor(&mut control, value),
        "margin" => apply_margin(&mut control, value),
        "size" => apply_size(&mut control, value),
        "custom_minimum_size" => apply_custom_minimum_size(&mut control, value),
        "stretch_mode" => {
            if tag == "TextureRect" {
                let mode = match value {
                    "scale" => StretchMode::SCALE,
                    "tile" => StretchMode::TILE,
                    "keep" => StretchMode::KEEP,
                    "keep_center" => StretchMode::KEEP_CENTERED,
                    "keep_aspect" => StretchMode::KEEP_ASPECT,
                    "keep_aspect_centered" => StretchMode::KEEP_ASPECT_CENTERED,
                    "keep_aspect_covered" => StretchMode::KEEP_ASPECT_COVERED,
                    _ => StretchMode::KEEP_ASPECT,
                };
                let mut tr = control.cast::<TextureRect>();
                tr.set_stretch_mode(mode);
                return tr.upcast();
            }
        }
        "texture" => {
            if tag == "TextureRect" {
                let path = GString::from(value);
                if let Some(res) = ResourceLoader::singleton().load(&path) {
                    if let Ok(tex) = res.try_cast::<Texture2D>() {
                        let mut tr = control.cast::<TextureRect>();
                    tr.set_texture(&tex);
                        return tr.upcast();
                    }
                }
            } else if tag == "NinePatchRect" {
                let path = GString::from(value);
                if let Some(res) = ResourceLoader::singleton().load(&path) {
                    if let Ok(tex) = res.try_cast::<Texture2D>() {
                        let mut nr = control.cast::<NinePatchRect>();
                    nr.set_texture(&tex);
                        return nr.upcast();
                    }
                }
            } else if tag == "TextureButton" {
                let path = GString::from(value);
                if let Some(res) = ResourceLoader::singleton().load(&path) {
                    if let Ok(tex) = res.try_cast::<Texture2D>() {
                        let mut tb = control.cast::<TextureButton>();
                        tb.set_texture_normal(&tex);
                        return tb.upcast();
                    }
                }
            }
        }
        "texture_normal" => {
            if tag == "TextureButton" {
                let path = GString::from(value);
                if let Some(res) = ResourceLoader::singleton().load(&path) {
                    if let Ok(tex) = res.try_cast::<Texture2D>() {
                        let mut tb = control.cast::<TextureButton>();
                        tb.set_texture_normal(&tex);
                        return tb.upcast();
                    }
                }
            }
        }
        "texture_pressed" => {
            if tag == "TextureButton" {
                let path = GString::from(value);
                if let Some(res) = ResourceLoader::singleton().load(&path) {
                    if let Ok(tex) = res.try_cast::<Texture2D>() {
                        let mut tb = control.cast::<TextureButton>();
                        tb.set_texture_pressed(&tex);
                        return tb.upcast();
                    }
                }
            }
        }
        "texture_hover" => {
            if tag == "TextureButton" {
                let path = GString::from(value);
                if let Some(res) = ResourceLoader::singleton().load(&path) {
                    if let Ok(tex) = res.try_cast::<Texture2D>() {
                        let mut tb = control.cast::<TextureButton>();
                        tb.set_texture_hover(&tex);
                        return tb.upcast();
                    }
                }
            }
        }
        "texture_disabled" => {
            if tag == "TextureButton" {
                let path = GString::from(value);
                if let Some(res) = ResourceLoader::singleton().load(&path) {
                    if let Ok(tex) = res.try_cast::<Texture2D>() {
                        let mut tb = control.cast::<TextureButton>();
                        tb.set_texture_disabled(&tex);
                        return tb.upcast();
                    }
                }
            }
        }
        "bbcode" => {
            if tag == "RichTextLabel" {
                let mut rt = control.cast::<RichTextLabel>();
                rt.set_use_bbcode(true);
                rt.set_text(&GString::from(value));
                return rt.upcast();
            }
        }
        "placeholder_text" => {
            if tag == "LineEdit" {
                let mut le = control.cast::<LineEdit>();
                le.set_placeholder(&GString::from(value));
                return le.upcast();
            }
        }
        "columns" => {
            if tag == "GridContainer" || tag == "UIGrid" {
                if let Ok(cols) = value.parse::<i32>() {
                    let mut gc = control.cast::<GridContainer>();
                    gc.set_columns(cols);
                    return gc.upcast();
                }
            }
        }
        "h_separation" | "v_separation" => {
            let sep_name = if key == "h_separation" { "h_separation" } else { "v_separation" };
            if let Ok(val) = value.parse::<i32>() {
                control.add_theme_constant_override(
                    &StringName::from(sep_name),
                    val,
                );
            }
        }
        "expand" => {
            if tag == "TextureRect" {
                use godot::classes::texture_rect::ExpandMode;
                let mode = if value == "true" || value == "1" {
                    ExpandMode::FIT_WIDTH
                } else {
                    ExpandMode::IGNORE_SIZE
                };
                let mut tr = control.cast::<TextureRect>();
                tr.set_expand_mode(mode);
                return tr.upcast();
            }
        }
        "horizontal" | "vertical" => {
            if tag == "ScrollContainer" {
                use godot::classes::scroll_container::ScrollMode;
                let mut sc = control.cast::<ScrollContainer>();
                if key == "horizontal" {
                    sc.set_horizontal_scroll_mode(if value == "disabled" { ScrollMode::DISABLED } else { ScrollMode::AUTO });
                } else {
                    sc.set_vertical_scroll_mode(if value == "disabled" { ScrollMode::DISABLED } else { ScrollMode::AUTO });
                }
                return sc.upcast();
            }
        }
        "use_top_left" => {
            if tag == "CenterContainer" {
                let mut cc = control.cast::<CenterContainer>();
                cc.set_use_top_left(value == "true" || value == "1");
                return cc.upcast();
            }
        }
        "percent_visible" => {
            if tag == "ProgressBar" {
                let mut pb = control.cast::<ProgressBar>();
                pb.set_show_percentage(value != "false" && value != "0");
                return pb.upcast();
            }
        }
        "min_value" | "max_value" | "step" => {
            if let Ok(val) = value.parse::<f64>() {
                let mut c = control.cast::<Range>();
                match key {
                    "min_value" => c.set_min(val),
                    "max_value" => c.set_max(val),
                    "step" => c.set_step(val),
                    _ => {}
                }
                return c.upcast();
            }
        }
        "value" => {
            if tag == "ProgressBar" || tag == "SpinBox" || tag == "HSlider" {
                if let Ok(val) = value.parse::<f64>() {
                    let mut c = control.cast::<Range>();
                    c.set_value(val);
                    return c.upcast();
                }
            }
        }
        "visible" => {
            control.set_visible(value != "false" && value != "0");
        }
        "tooltip_text" => {
            control.set_tooltip_text(&GString::from(value));
        }
        "disabled" => {
            // try_cast 消耗 self，失败时返回原始 control
            let result = control.try_cast::<BaseButton>();
            match result {
                Ok(mut base_btn) => {
                    base_btn.set_disabled(value == "true" || value == "1");
                    return base_btn.upcast();
                }
                Err(original) => {
                    control = original;
                }
            }
        }
        "clip_contents" => {
            control.set_clip_contents(value == "true" || value == "1");
        }
        "mouse_default_cursor_shape" => {
            use godot::classes::control::CursorShape;
            let shape = match value {
                "pointing_hand" => CursorShape::POINTING_HAND,
                "cross" => CursorShape::CROSS,
                "move" => CursorShape::MOVE,
                "forbidden" => CursorShape::FORBIDDEN,
                _ => CursorShape::ARROW,
            };
            control.set_default_cursor_shape(shape);
        }
        // 列表扩展节点属性 - 需要类型转换
        "count" | "highlight_mode" | "fill_mode" => {
            if let Ok(val) = value.parse::<i32>() {
                control.set(&StringName::from(key), &val.to_variant());
            }
        }
        "enable_random_pos" => {
            control.set(&StringName::from(key), &(value == "true" || value == "1").to_variant());
        }
        "highlight_color" | "fill_color" => {
            if let Some(color) = parse_color(value) {
                control.set(&StringName::from(key), &color.to_variant());
            }
        }
        "random_rotate" | "space_left" | "space_right" => {
            if let Ok(val) = value.parse::<f32>() {
                control.set(&StringName::from(key), &val.to_variant());
            }
        }
        "tooltip" => {
            if tag == "UIHList" || tag == "UIVList" || tag == "UIGrid" {
                control.set(&StringName::from("tooltip"), &value.to_variant());
            }
        }
        "data" => {
            if tag == "UIHList" || tag == "UIVList" || tag == "UIGrid" {
                // 存储数据变量名，由 GdGmlScene 在加载后自动绑定
                godot_print!("[UiBuilder] Setting __data_var='{}' on node '{}' (tag={})", value, control.get_name(), tag);
                control.set_meta(&StringName::from("__data_var"), &value.to_variant());
            } else {
                godot_warn!("[UiBuilder] 'data' attribute ignored on non-list tag '{}' (node='{}')", tag, control.get_name());
            }
        }
        "size_flags_horizontal" => {
            apply_size_flags_horizontal(&mut control, value);
        }
        "size_flags_vertical" => {
            apply_size_flags_vertical(&mut control, value);
        }
        "color" => {
            if tag == "ColorRect" {
                if let Some(c) = parse_color(value) {
                    let mut cr = control.cast::<ColorRect>();
                    cr.set_color(c);
                    return cr.upcast();
                }
            }
        }
        "toggle_mode" => {
            if tag == "Button" || tag == "CheckButton" || tag == "TextureButton" {
                let result = control.try_cast::<BaseButton>();
                match result {
                    Ok(mut base_btn) => {
                        base_btn.set_toggle_mode(value == "true" || value == "1");
                        return base_btn.upcast();
                    }
                    Err(original) => {
                        control = original;
                    }
                }
            }
        }
        "button_pressed" => {
            if tag == "CheckButton" || tag == "Button" || tag == "TextureButton" {
                let result = control.try_cast::<BaseButton>();
                match result {
                    Ok(mut base_btn) => {
                        base_btn.set_pressed(value == "true" || value == "1");
                        return base_btn.upcast();
                    }
                    Err(original) => {
                        control = original;
                    }
                }
            }
        }
        "items" => {
            if tag == "OptionButton" {
                let mut ob = control.cast::<OptionButton>();
                // items 格式: "item1,item2,item3"
                for item in value.split(',') {
                    let item = item.trim();
                    if !item.is_empty() {
                        ob.add_item(&GString::from(item));
                    }
                }
                return ob.upcast();
            }
        }
        "selected" => {
            if tag == "OptionButton" {
                if let Ok(idx) = value.parse::<i32>() {
                    let mut ob = control.cast::<OptionButton>();
                    ob.select(idx);
                    return ob.upcast();
                }
            }
        }
        // PopupPanel 特有属性
        "popup_title" => {
            if tag == "PopupPanel" {
                control.set(&StringName::from("popup_title"), &value.to_variant());
            }
        }
        "popup_width" => {
            if tag == "PopupPanel" {
                if let Ok(w) = value.parse::<i32>() {
                    control.set(&StringName::from("popup_width"), &w.to_variant());
                }
            }
        }
        "close_on_overlay" => {
            if tag == "PopupPanel" || tag == "Drawer" || tag == "NavMenu" {
                control.set(&StringName::from("close_on_overlay"), &(value == "true" || value == "1").to_variant());
            }
        }
        // Tooltip 特有属性
        "tooltip_title" => {
            if tag == "Tooltip" {
                control.set(&StringName::from("tooltip_title_text"), &value.to_variant());
            }
        }
        "tooltip_content" => {
            if tag == "Tooltip" {
                control.set(&StringName::from("tooltip_content_text"), &value.to_variant());
            }
        }
        "delay" => {
            if tag == "Tooltip" {
                if let Ok(val) = value.parse::<f64>() {
                    control.set(&StringName::from("delay"), &val.to_variant());
                }
            }
        }
        "offset_x" | "offset_y" => {
            if tag == "Tooltip" {
                if let Ok(val) = value.parse::<f32>() {
                    control.set(&StringName::from(key), &val.to_variant());
                }
            }
        }
        "max_width" => {
            if tag == "Tooltip" {
                if let Ok(val) = value.parse::<i32>() {
                    control.set(&StringName::from("max_width"), &val.to_variant());
                }
            }
        }
        "max_height" => {
            if tag == "Tooltip" {
                if let Ok(val) = value.parse::<i32>() {
                    control.set(&StringName::from("max_height"), &val.to_variant());
                }
            }
        }
        // Drawer 特有属性
        "direction" => {
            if tag == "Drawer" {
                let dir = match value {
                    "right" => 0,
                    "left" => 1,
                    "top" => 2,
                    "bottom" => 3,
                    _ => 0,
                };
                control.set(&StringName::from("direction"), &dir.to_variant());
            } else if tag == "NavMenu" {
                let dir = match value {
                    "left" => 0,
                    "right" => 1,
                    _ => 0,
                };
                control.set(&StringName::from("direction"), &dir.to_variant());
            }
        }
        // TabContainer 特有属性
        "current_tab" => {
            if tag == "TabContainer" {
                if let Ok(idx) = value.parse::<i32>() {
                    let mut tc = control.cast::<TabContainer>();
                    tc.set_current_tab(idx);
                    return tc.upcast();
                }
            }
        }
        "tabs_visible" => {
            if tag == "TabContainer" {
                let mut tc = control.cast::<TabContainer>();
                tc.set_tabs_visible(value != "false" && value != "0");
                return tc.upcast();
            }
        }
        "slide_width" => {
            if tag == "Drawer" {
                if let Ok(val) = value.parse::<i32>() {
                    control.set(&StringName::from("slide_width"), &val.to_variant());
                }
            }
        }
        "menu_width" | "sub_menu_width" => {
            if tag == "NavMenu" {
                if let Ok(val) = value.parse::<i32>() {
                    control.set(&StringName::from(key), &val.to_variant());
                }
            }
        }
        "animation_duration" => {
            if tag == "Drawer" || tag == "NavMenu" {
                if let Ok(val) = value.parse::<f64>() {
                    control.set(&StringName::from("animation_duration"), &val.to_variant());
                }
            }
        }
        "drawer_title" => {
            if tag == "Drawer" {
                control.set(&StringName::from("drawer_title_text"), &value.to_variant());
            }
        }
        _ => {
            godot_print!("[UiBuilder] Unhandled attribute: {}='{}' on <{}>", key, value, tag);
        }
    }
    control
}

/// 设置文字颜色
fn apply_text_color(control: &mut Gd<Control>, tag: &str, color: Color) {
    match tag {
        "Label" | "Button" | "CheckButton" => {
            control.add_theme_color_override(
                &StringName::from("font_color"),
                color,
            );
        }
        "TextureButton" => {
            // TextureButton 的文字在叠加的 Label 上，需要找到子 Label 设置颜色
            for i in 0..control.get_child_count() {
                if let Some(child) = control.get_child(i) {
                    if let Ok(mut lbl) = child.try_cast::<Label>() {
                        lbl.add_theme_color_override(
                            &StringName::from("font_color"),
                            color,
                        );
                        break;
                    }
                }
            }
        }
        "LineEdit" => {
            control.add_theme_color_override(
                &StringName::from("font_color"),
                color,
            );
        }
        "OptionButton" => {
            control.add_theme_color_override(
                &StringName::from("font_color"),
                color,
            );
        }
        _ => {}
    }
}

/// 应用锚点预设
fn apply_anchor(control: &mut Gd<Control>, value: &str) {
    let preset = match value {
        "top_left" => LayoutPreset::TOP_LEFT,
        "top_right" => LayoutPreset::TOP_RIGHT,
        "bottom_left" => LayoutPreset::BOTTOM_LEFT,
        "bottom_right" => LayoutPreset::BOTTOM_RIGHT,
        "center" => LayoutPreset::CENTER,
        "left_center" => LayoutPreset::CENTER_LEFT,
        "top_center" => LayoutPreset::CENTER_TOP,
        "right_center" => LayoutPreset::CENTER_RIGHT,
        "bottom_center" => LayoutPreset::CENTER_BOTTOM,
        "full" => LayoutPreset::FULL_RECT,
        "top_wide" => LayoutPreset::TOP_WIDE,
        "bottom_wide" => LayoutPreset::BOTTOM_WIDE,
        "left_wide" => LayoutPreset::LEFT_WIDE,
        "right_wide" => LayoutPreset::RIGHT_WIDE,
        "vcenter_wide" => LayoutPreset::VCENTER_WIDE,
        "hcenter_wide" => LayoutPreset::HCENTER_WIDE,
        _ => return,
    };
    control.set_anchors_and_offsets_preset(preset);
    // 存储 anchor 值为 meta，以便节点加入场景树后重新应用
    control.set_meta(&StringName::from("__anchor"), &GString::from(value).to_variant());
}

/// 应用边距
/// 支持格式: "12" (四边相同), "10 20" (水平 垂直), "10 20 30 40" (左 上 右 下)
/// 通过设置 offset 属性实现（Side 枚举在 gdext 0.5 中未公开导出）
fn apply_margin(control: &mut Gd<Control>, value: &str) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let (left, top, right, bottom) = match parts.len() {
        1 => {
            let v = parts[0].parse::<f32>().unwrap_or(0.0);
            (v, v, v, v)
        }
        2 => {
            let h = parts[0].parse::<f32>().unwrap_or(0.0);
            let v = parts[1].parse::<f32>().unwrap_or(0.0);
            (h, v, h, v)
        }
        4 => {
            let l = parts[0].parse::<f32>().unwrap_or(0.0);
            let t = parts[1].parse::<f32>().unwrap_or(0.0);
            let r = parts[2].parse::<f32>().unwrap_or(0.0);
            let b = parts[3].parse::<f32>().unwrap_or(0.0);
            (l, t, r, b)
        }
        _ => return,
    };
    // 使用 set_offset 配合 Side 值（0=LEFT, 1=TOP, 2=RIGHT, 3=BOTTOM）
    // Side 枚举未公开导出，使用 call 方式设置
    control.set_offset(Side::LEFT, left);
    control.set_offset(Side::TOP, top);
    control.set_offset(Side::RIGHT, -right);
    control.set_offset(Side::BOTTOM, -bottom);
}

/// 应用大小
/// 格式: "width,height"
fn apply_size(control: &mut Gd<Control>, value: &str) {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() == 2 {
        let w = parts[0].trim().parse::<f32>().unwrap_or(0.0);
        let h = parts[1].trim().parse::<f32>().unwrap_or(0.0);
        control.set_custom_minimum_size(Vector2::new(w, h));
        control.set_size(Vector2::new(w, h));
    }
}

/// 应用自定义最小尺寸
fn apply_custom_minimum_size(control: &mut Gd<Control>, value: &str) {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() == 2 {
        let w = parts[0].trim().parse::<f32>().unwrap_or(0.0);
        let h = parts[1].trim().parse::<f32>().unwrap_or(0.0);
        control.set_custom_minimum_size(Vector2::new(w, h));
    }
}

/// 解析颜色字符串
/// 支持: "#RRGGBB", "#RRGGBBAA", 颜色名称
fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim();
    if value.starts_with('#') {
        let hex = &value[1..];
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0))
            }
            _ => None,
        }
    } else {
        match value {
            "white" => Some(Color::from_rgb(1.0, 1.0, 1.0)),
            "black" => Some(Color::from_rgb(0.0, 0.0, 0.0)),
            "red" => Some(Color::from_rgb(1.0, 0.0, 0.0)),
            "green" => Some(Color::from_rgb(0.0, 1.0, 0.0)),
            "blue" => Some(Color::from_rgb(0.0, 0.0, 1.0)),
            "yellow" => Some(Color::from_rgb(1.0, 1.0, 0.0)),
            "gray" | "grey" => Some(Color::from_rgb(0.5, 0.5, 0.5)),
            "transparent" => Some(Color::from_rgba(0.0, 0.0, 0.0, 0.0)),
            _ => None,
        }
    }
}

/// 获取标签对应的 StyleBox 名称
fn get_stylebox_name_for_tag(tag: &str) -> &'static str {
    match tag {
        "Button" | "CheckButton" | "TextureButton" => "normal",
        "Panel" => "panel",
        "LineEdit" => "normal",
        "OptionButton" => "normal",
        "RichTextLabel" => "normal",
        "ProgressBar" => "background",
        "HSlider" => "slider",
        _ => "panel",
    }
}

/// 应用 size_flags_horizontal
/// 支持格式: "fill" (SIZE_FILL), "expand" (SIZE_EXPAND), "expand_fill" (SIZE_EXPAND_FILL)
/// 或 Godot 原始整数值
fn apply_size_flags_horizontal(control: &mut Gd<Control>, value: &str) {
    use godot::classes::control::SizeFlags;
    let flag = match value {
        "fill" => SizeFlags::FILL,
        "expand" => SizeFlags::EXPAND,
        "expand_fill" => SizeFlags::EXPAND_FILL,
        "shrink_center" => SizeFlags::SHRINK_CENTER,
        "shrink_end" => SizeFlags::SHRINK_END,
        _ => SizeFlags::FILL,
    };
    control.set_h_size_flags(flag);
}

/// 应用 size_flags_vertical
/// 支持格式同 apply_size_flags_horizontal
fn apply_size_flags_vertical(control: &mut Gd<Control>, value: &str) {
    use godot::classes::control::SizeFlags;
    let flag = match value {
        "fill" => SizeFlags::FILL,
        "expand" => SizeFlags::EXPAND,
        "expand_fill" => SizeFlags::EXPAND_FILL,
        "shrink_center" => SizeFlags::SHRINK_CENTER,
        "shrink_end" => SizeFlags::SHRINK_END,
        _ => SizeFlags::FILL,
    };
    control.set_v_size_flags(flag);
}

/// 内部信号动作类型
enum InternalAction {
    Show,
    Hide,
    Toggle,
    Open,
    Close,
}

/// 后处理：解析内部信号绑定
/// 遍历节点树中所有带 __signal_xxx 元数据的节点，
/// 如果元数据值匹配 "show:NodeName"、"hide:NodeName"、"toggle:NodeName" 格式，
/// 则在根节点树中查找目标节点并直接连接信号
fn resolve_internal_signals(root: &mut Gd<Control>) {
    // 克隆 root 用于不可变引用查找
    let root_clone = root.clone();
    resolve_internal_signals_recursive(root, &root_clone);
}

fn resolve_internal_signals_recursive(node: &mut Gd<Control>, root: &Gd<Control>) {
    let meta_list = node.get_meta_list();
    let mut resolved_keys: Vec<StringName> = Vec::new();

    for i in 0..meta_list.len() {
        if let Some(key_sn) = meta_list.get(i) {
            let key = key_sn.to_string();
            if key.starts_with("__signal_") {
                let signal_name = key[9..].to_string();
                let method_value = node.get_meta(&StringName::from(key.as_str())).to_string();

                // 检查是否为内部动作绑定
                if let Some((action, target_name)) = parse_internal_action(&method_value) {
                    // 在根节点树中查找目标节点
                    if let Some(target) = root.find_child(&GString::from(target_name.as_str())) {
                        let target_obj = target.clone().upcast::<Object>();
                        let callable = match action {
                            InternalAction::Show => {
                                if target_obj.has_method(&StringName::from("show_popup")) {
                                    Callable::from_object_method(&target, &StringName::from("show_popup"))
                                } else {
                                    Callable::from_object_method(&target, &StringName::from("open"))
                                }
                            }
                            InternalAction::Hide => {
                                if target_obj.has_method(&StringName::from("hide_popup")) {
                                    Callable::from_object_method(&target, &StringName::from("hide_popup"))
                                } else {
                                    Callable::from_object_method(&target, &StringName::from("close"))
                                }
                            }
                            InternalAction::Toggle => {
                                if target_obj.has_method(&StringName::from("toggle_popup")) {
                                    Callable::from_object_method(&target, &StringName::from("toggle_popup"))
                                } else {
                                    Callable::from_object_method(&target, &StringName::from("toggle"))
                                }
                            }
                            InternalAction::Open => Callable::from_object_method(&target, &StringName::from("open")),
                            InternalAction::Close => Callable::from_object_method(&target, &StringName::from("close")),
                        };
                        node.connect(&StringName::from(signal_name.as_str()), &callable);
                        resolved_keys.push(key_sn.clone());
                    } else {
                        godot_error!("[UiBuilder] Cannot find target node '{}' for internal signal binding", target_name);
                    }
                }
            }
        }
    }

    // 移除已解析的内部绑定元数据（不再传递给外部 connect_signals）
    for key in resolved_keys {
        node.remove_meta(&key);
    }

    // 递归处理子节点
    let children = node.get_children();
    for i in 0..children.len() {
        if let Some(child) = children.get(i) {
            if let Ok(mut control) = child.clone().try_cast::<Control>() {
                resolve_internal_signals_recursive(&mut control, root);
            }
        }
    }
}

/// 解析内部动作绑定
/// 格式: "show:NodeName", "hide:NodeName", "toggle:NodeName", "open:NodeName", "close:NodeName"
fn parse_internal_action(value: &str) -> Option<(InternalAction, String)> {
    let value = value.trim();
    if let Some(rest) = value.strip_prefix("show:") {
        let name = rest.trim().to_string();
        if !name.is_empty() {
            return Some((InternalAction::Show, name));
        }
    } else if let Some(rest) = value.strip_prefix("hide:") {
        let name = rest.trim().to_string();
        if !name.is_empty() {
            return Some((InternalAction::Hide, name));
        }
    } else if let Some(rest) = value.strip_prefix("toggle:") {
        let name = rest.trim().to_string();
        if !name.is_empty() {
            return Some((InternalAction::Toggle, name));
        }
    } else if let Some(rest) = value.strip_prefix("open:") {
        let name = rest.trim().to_string();
        if !name.is_empty() {
            return Some((InternalAction::Open, name));
        }
    } else if let Some(rest) = value.strip_prefix("close:") {
        let name = rest.trim().to_string();
        if !name.is_empty() {
            return Some((InternalAction::Close, name));
        }
    }
    None
}
