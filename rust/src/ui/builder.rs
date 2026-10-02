// UI 构建器
// 将解析器生成的 AST 节点树转换为 Godot Control 节点树
// 支持容器/控件/样式/信号绑定/布局属性

use std::cell::RefCell;
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
    FileAccess,
};
use godot::classes::control::LayoutPreset;
use godot::classes::texture_rect::StretchMode;
use godot::obj::NewGd;

use super::parser::{UiNode, StyleRule, ParseResult, UiParser, DataValue};
use super::ui_hlist::GdUIHList;
use super::ui_vlist::GdUIVList;
use super::ui_grid::GdUIGrid;
use super::ui_popup_panel::GdPopupPanel;
use super::ui_tooltip::GdUITooltip;
use super::ui_drawer::GdUIDrawer;
use super::ui_nav_menu::GdUINavMenu;
use super::ui_theme::{ThemeVars, get_builtin_theme, get_theme_color, resolve_theme_vars};

/// UI 构建器：将 AST 转换为 Godot Control 节点树
pub struct UiBuilder {
    /// 样式规则表：class_name -> StyleRule
    styles: HashMap<String, StyleRule>,
    /// 主题变量表
    theme_vars: ThemeVars,
    /// 当前构建的 gml 文件所在目录（用于 <Gml src="相对路径"> 解析）
    base_dir: Option<String>,
    /// <Gml> 引用链（含当前文件），用于检测循环引用
    include_stack: Vec<String>,
    /// <script> 块定义的数据变量（已合并 provided_vars 覆盖）
    script_vars: HashMap<String, DataValue>,
    /// 父文件 <Gml data-xxx="父变量名"> 注入的具名数据（xxx 为本文件 <script> 变量名，构建前覆盖同名变量）
    provided_vars: HashMap<String, DataValue>,
    /// <ui script="xxx.gd"> 声明的脚本（构建后挂载到内容根节点）
    ui_script: Option<String>,
    /// 待应用的数据绑定（节点, data="变量名"），树构建完成后统一应用，
    /// 保证列表的 slot 模板子节点已就位
    pending_data: RefCell<Vec<(Gd<Control>, String)>>,
}

impl UiBuilder {
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
            theme_vars: ThemeVars::new(),
            base_dir: None,
            include_stack: Vec::new(),
            script_vars: HashMap::new(),
            provided_vars: HashMap::new(),
            ui_script: None,
            pending_data: RefCell::new(Vec::new()),
        }
    }

    /// 设置当前构建文件的所在目录（<Gml> 相对路径基准）
    pub(crate) fn set_base_dir(&mut self, dir: Option<String>) {
        self.base_dir = dir;
    }

    /// 设置主题变量表
    pub fn set_theme_vars(&mut self, vars: ThemeVars) {
        self.theme_vars = vars;
    }

    /// 从解析结果构建 Control 节点树
    pub fn build(&mut self, parse_result: &ParseResult) -> Result<Gd<Control>, String> {
        // 构建样式索引
        for style in &parse_result.styles {
            self.styles.insert(style.class_name.clone(), style.clone());
        }

        // 构建主题变量：先加载内置主题，再用 <theme> 块覆盖
        if let Some(ref theme_name) = parse_result.theme_name {
            if let Some(builtin_vars) = get_builtin_theme(theme_name) {
                for (key, value) in builtin_vars {
                    self.theme_vars.entry(key).or_insert(value);
                }
            }
        }
        // <theme> 块中的变量覆盖内置主题
        for (key, value) in &parse_result.theme_vars {
            self.theme_vars.insert(key.clone(), value.clone());
        }

        // <ui script="xxx.gd"> 声明
        self.ui_script = parse_result.ui_script.clone();

        // <script> 块数据变量
        self.script_vars.extend(parse_result.script_vars.clone());
        // 父文件 data-xxx 具名数据覆盖同名 <script> 变量（<Gml> 引用复用时的数据注入点，
        // 子文件内部节点继续用自己的变量名 data="xxx" / {{xxx}} 引用）
        for (name, value) in &self.provided_vars {
            self.script_vars.insert(name.clone(), value.clone());
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

        // 后处理：应用 data="变量名" 绑定（<script> 数据 → 列表 update / 节点 meta）
        self.apply_data_bindings(&mut root);

        // 后处理：挂载 <ui script="xxx.gd"> 声明的脚本
        self.attach_ui_script(&mut root);

        Ok(root)
    }

    /// 构建单个 AST 节点为 Control
    fn build_node(&self, node: &UiNode) -> Result<Gd<Control>, String> {
        // <Gml src="...">：引用另一个 gml 文件，构建后剥壳嫁接到当前位置
        if node.tag == "Gml" {
            return self.build_gml_include(node);
        }

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
                "data" => {
                    if value.starts_with("bean:") {
                        // bean:bean_id:prop_key —— GdBean 运行时响应式绑定，
                        // 存 __data_var meta 由 GdGmlScene auto_bind_data 消费
                        // （构建期不解析，<script> 变量机制不接管）
                        control.set_meta(
                            &StringName::from("__data_var"),
                            &value.to_variant(),
                        );
                    } else {
                        // <script> 数据绑定延迟到树构建完成后应用
                        self.pending_data
                            .borrow_mut()
                            .push((control.clone(), value.clone()));
                    }
                }
                _ => {
                    // 信号绑定声明：on_pressed / @pressed（@ 为简写）→ 存 meta，
                    // 由 connect_signals（静态节点）/ 列表 bind_events（条目内）统一连接
                    if let Some(signal_name) =
                        key.strip_prefix("on_").or_else(|| key.strip_prefix('@'))
                    {
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

        // 应用主题默认颜色（在 class 样式之前，class 可覆盖）
        self.apply_theme_defaults(&mut control, &node.tag);

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

        // NinePatchRect 作为按钮使用时（有 on_pressed/@pressed 属性）：
        // 添加不可见 Button 子节点处理点击事件，NinePatchRect 本身设为鼠标穿透
        if node.tag == "NinePatchRect" {
            let has_signal_pressed = node
                .attributes
                .iter()
                .any(|(k, _)| k == "on_pressed" || k == "@pressed");
            if has_signal_pressed {
                control.set_mouse_filter(godot::classes::control::MouseFilter::IGNORE);
                let mut btn = Button::new_alloc();
                btn.set_name(&StringName::from("__click_handler"));
                btn.set_anchor(Side::LEFT, 0.0);
                btn.set_anchor(Side::RIGHT, 1.0);
                btn.set_anchor(Side::TOP, 0.0);
                btn.set_anchor(Side::BOTTOM, 1.0);
                btn.set_offset(Side::LEFT, 0.0);
                btn.set_offset(Side::RIGHT, 0.0);
                btn.set_offset(Side::TOP, 0.0);
                btn.set_offset(Side::BOTTOM, 0.0);
                // 透明样式：移除默认 StyleBox
                let mut transparent_box = StyleBoxFlat::new_gd();
                transparent_box.set_bg_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
                transparent_box.set_border_width_all(0);
                transparent_box.set_content_margin_all(0.0);
                btn.add_theme_stylebox_override(&StringName::from("normal"), &transparent_box.clone());
                btn.add_theme_stylebox_override(&StringName::from("hover"), &transparent_box.clone());
                btn.add_theme_stylebox_override(&StringName::from("pressed"), &transparent_box.clone());
                // 转移鼠标光标形状
                if let Some((_, shape)) = node.attributes.iter().find(|(k, _)| k == "mouse_default_cursor_shape") {
                    use godot::classes::control::CursorShape;
                    let cursor_shape = match shape.as_str() {
                        "pointing_hand" => CursorShape::POINTING_HAND,
                        "cross" => CursorShape::CROSS,
                        "move" => CursorShape::MOVE,
                        "forbidden" => CursorShape::FORBIDDEN,
                        _ => CursorShape::ARROW,
                    };
                    btn.set_default_cursor_shape(cursor_shape);
                }
                // 转移 __signal_pressed meta 到 Button
                let meta_key = StringName::from("__signal_pressed");
                if control.has_meta(&meta_key) {
                    let signal_value = control.get_meta(&meta_key);
                    btn.set_meta(&meta_key, &signal_value);
                    control.remove_meta(&meta_key);
                }
                control.add_child(&btn);
                btn.set_owner(&control);
            }
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

            // TextureButton 的子 Label：自动配置锚点填满父节点、文字居中、鼠标穿透
            if node.tag == "TextureButton" && child_node.tag == "Label" {
                if let Ok(mut lbl) = child_control.clone().try_cast::<Label>() {
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
                }
            }

            // NinePatchRect 的子 Label：自动配置锚点填满父节点、文字居中、鼠标穿透
            if node.tag == "NinePatchRect" && child_node.tag == "Label" {
                if let Ok(mut lbl) = child_control.clone().try_cast::<Label>() {
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
                }
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

    /// 构建 <Gml src="..."> 引用：解析目标 gml → 构建子树 → 剥掉 UiRoot 包装层，
    /// 返回真正的根控件（由调用方挂到引用位置的父节点上）。
    /// Gml 标签上的其余属性（name/anchor/margin/class/on_xxx 等）会覆盖式地
    /// 应用到被引用文件的根节点上。
    /// 主题与样式继承：子文件继承引用方的主题变量与 class 样式；
    /// 子文件自己的 theme 属性 / <theme> 块 / <style> 块优先。
    fn build_gml_include(&self, node: &UiNode) -> Result<Gd<Control>, String> {
        let src = node
            .attributes
            .iter()
            .find(|(k, _)| k == "src")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| "<Gml> 标签缺少 src 属性".to_string())?;

        let path = resolve_include_path(&src, self.base_dir.as_deref());
        if self.include_stack.contains(&path) {
            return Err(format!(
                "GML 循环引用: {} -> {}",
                self.include_stack.join(" -> "),
                path
            ));
        }

        let path_g = GString::from(&path);
        let content = match FileAccess::open(&path_g, godot::classes::file_access::ModeFlags::READ) {
            Some(fa) => fa.get_as_text().to_string(),
            None => {
                return Err(format!(
                    "<Gml> 无法打开引用文件: {}（base_dir={:?}）",
                    path, self.base_dir
                ))
            }
        };

        let mut parser = UiParser::new(&content);
        let parse_result = parser
            .parse()
            .map_err(|e| format!("GML 引用解析失败 {}: {}", path, e))?;

        // 被引用文件根节点（<ui> 的第一个子元素）的标签，用于 class 样式的标签匹配
        let root_tag = parse_result
            .root
            .children
            .first()
            .map(|n| n.tag.clone())
            .unwrap_or_default();

        // 收集 data-xxx="父变量名" 具名数据：xxx 为子文件 <script> 变量名，
        // 用父文件的变量值覆盖子文件同名变量（子文件节点用 data="xxx" / {{xxx}} 引用）
        let mut provided_vars: HashMap<String, DataValue> = HashMap::new();
        for (key, value) in &node.attributes {
            if let Some(child_name) = key.strip_prefix("data-") {
                match self.script_vars.get(value) {
                    Some(v) => {
                        provided_vars.insert(child_name.to_string(), v.clone());
                    }
                    None => godot_error!(
                        "[GdUiBuilder] data-{}=\"{}\" 引用的父文件 <script> 变量不存在",
                        child_name, value
                    ),
                }
            }
        }

        // 子构建器：继承引用方的主题变量与 class 样式
        let mut sub = UiBuilder {
            styles: self.styles.clone(),
            theme_vars: self.theme_vars.clone(),
            base_dir: parent_dir_of(&path),
            include_stack: {
                let mut stack = self.include_stack.clone();
                stack.push(path.clone());
                stack
            },
            script_vars: HashMap::new(),
            provided_vars,
            ui_script: None,
            pending_data: RefCell::new(Vec::new()),
        };
        let mut wrapper = sub.build(&parse_result)?;

        // 剥壳：UiRoot 包装层是构建器的实现细节，引用场景只保留根控件
        let grafted_node = wrapper
            .get_child(0)
            .ok_or_else(|| format!("GML 引用文件为空: {}", path))?;
        let mut grafted = grafted_node
            .try_cast::<Control>()
            .map_err(|_| format!("GML 引用根节点不是 Control: {}", path))?;
        wrapper.remove_child(&grafted.clone().upcast::<godot::classes::Node>());
        // pack 后包装层不再需要（Node 为手动内存，需显式释放）
        wrapper.free();

        // Gml 标签上的其余属性覆盖式应用到被引用根节点
        for (key, value) in &node.attributes {
            match key.as_str() {
                "src" => {}
                "name" => grafted.set_name(&StringName::from(value)),
                "class" => self.apply_class_style(&mut grafted, &root_tag, value),
                "data" => {
                    // 覆盖被引用文件的数据绑定：按引用方（父文件）的 <script> 变量解析
                    // （旧版整体覆盖语法，推荐改用 data-xxx 具名覆盖）
                    self.pending_data
                        .borrow_mut()
                        .push((grafted.clone(), value.clone()));
                }
                k if k.starts_with("data-") => {
                    // 具名数据覆盖（data-子变量名="父变量名"）已在构建前注入子文件
                    // script_vars，此处无需再处理
                }
                k if k.starts_with("on_") || k.starts_with('@') => {
                    // 信号绑定声明（on_pressed / @pressed 简写）
                    let signal_name = k.strip_prefix("on_").or_else(|| k.strip_prefix('@'))
                        .unwrap_or(k);
                    grafted.set_meta(
                        &StringName::from(format!("__signal_{}", signal_name).as_str()),
                        &value.to_variant(),
                    );
                }
                k => {
                    grafted = apply_attribute(grafted, &root_tag, k, value);
                }
            }
        }

        Ok(grafted)
    }

    /// 应用 data="变量名" 绑定（树构建完成后调用）：
    /// - 列表节点（UIVList/UIHList/UIGrid）+ 数组变量：直接 update(data, false) 驱动条目
    /// - 其他节点：变量存为节点 meta "__script_data"
    /// - 所有变量同时挂到根节点 meta "__script_vars"（含每个顶层子节点），控制器可读取
    /// 挂载 <ui script="xxx.gd"> 声明的脚本到本文件的内容根节点
    /// 挂载目标是 UiRoot 包装层的顶层子节点——<Gml> 嫁接与场景打包均保留该节点，
    /// 条目被列表 duplicate 时脚本随节点复制，条目内 @pressed 声明可就近绑定自身脚本
    fn attach_ui_script(&self, root: &mut Gd<Control>) {
        let Some(rel) = &self.ui_script else { return; };
        let path = resolve_include_path(rel, self.base_dir.as_deref());
        let Some(child) = root.get_child(0) else {
            godot_warn!("[GdUiBuilder] <ui script=\"{}\"> 无顶层节点可挂载", rel);
            return;
        };
        let Ok(mut target) = child.try_cast::<Control>() else {
            godot_warn!("[GdUiBuilder] <ui script=\"{}\"> 顶层节点不是 Control，跳过挂载", rel);
            return;
        };
        // 外部已挂脚本（如 GdGmlScene 场景脚本）时不覆盖
        if target.get_script().is_some() {
            godot_warn!(
                "[GdUiBuilder] <ui script=\"{}\"> 挂载目标已存在脚本，跳过（外部脚本优先）",
                rel
            );
            return;
        }
        match ResourceLoader::singleton().load(&GString::from(path.as_str())) {
            Some(script) => {
                if let Ok(script) = script.try_cast::<godot::classes::Script>() {
                    // set_script 的类型安全包装对 Option<Gd<Script>> 的 AsArg 判定有
                    // corner case，走通用 call（Variant 签名）最稳
                    target.call(&StringName::from("set_script"), &[script.to_variant()]);
                } else {
                    godot_error!(
                        "[GdUiBuilder] <ui script=\"{}\"> 资源不是脚本: {}",
                        rel, path
                    );
                }
            }
            None => godot_error!(
                "[GdUiBuilder] <ui script=\"{}\"> 脚本加载失败: {}",
                rel, path
            ),
        }
    }

    fn apply_data_bindings(&self, root: &mut Gd<Control>) {
        let bindings: Vec<(Gd<Control>, String)> = self.pending_data.borrow().clone();
        self.pending_data.borrow_mut().clear();

        // script 变量表挂根节点 meta
        let mut dict: Dictionary<Variant, Variant> = Dictionary::new();
        for (name, value) in &self.script_vars {
            dict.set(&Variant::from(name.as_str()), &data_value_to_variant(value));
        }
        let dict_variant = dict.to_variant();
        root.set_meta(&StringName::from("__script_vars"), &dict_variant.clone());
        // 顶层子节点（剥壳嫁接后真正存活的根控件）也挂一份，供控制器按文件读取
        let children = root.get_children();
        for i in 0..children.len() {
            if let Some(child) = children.get(i) {
                if let Ok(mut control) = child.try_cast::<Control>() {
                    control.set_meta(&StringName::from("__script_vars"), &dict_variant.clone());
                }
            }
        }

        for (mut node, var_name) in bindings {
            let Some(value) = self.script_vars.get(&var_name) else {
                godot_error!(
                    "[GdUiBuilder] data=\"{}\" 引用的 <script> 变量不存在",
                    var_name
                );
                continue;
            };

            // 数组数据：优先绑定到列表控件（UIVList/UIHList/UIGrid）。
            // 节点本身不是列表时（如 <Gml> 覆盖的根是 ScrollContainer 等容器），
            // 向后代查找最近的列表控件作为绑定目标
            if let DataValue::Array(_) = value {
                let target = if is_list_control(&node) {
                    Some(node.clone())
                } else {
                    find_list_descendant(&node)
                };
                if let Some(mut list) = target {
                    list.set_meta(
                        &StringName::from("__script_data"),
                        &data_value_to_variant(value),
                    );
                    // #[func] update(data, force)：构建期直接驱动列表条目
                    list.call(
                        &StringName::from("update"),
                        &[
                            data_value_to_array(value).to_variant(),
                            false.to_variant(),
                        ],
                    );
                    continue;
                }
            }

            // 非数组 / 无列表目标：变量原样存 meta，任何组件/控制器都能取到
            node.set_meta(
                &StringName::from("__script_data"),
                &data_value_to_variant(value),
            );
            if let DataValue::Array(_) = value {
                godot_warn!(
                    "[GdUiBuilder] data=\"{}\" 为数组但节点 {} 及其后代中没有列表控件（UIVList/UIHList/UIGrid），已存为 meta",
                    var_name,
                    node.get_name()
                );
            }
        }
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
                // //godot_print!("[UiBuilder] Unknown tag '{}', falling back to Control", tag);
                Control::new_alloc()
            }
        };
        Ok(control)
    }

    /// 应用 class 样式到控件
    fn apply_class_style(&self, control: &mut Gd<Control>, tag: &str, class_name: &str) {
        if let Some(style_rule) = self.styles.get(class_name) {
            // 解析样式属性值，替换主题变量
            let resolved_props: HashMap<String, String> = style_rule.properties.iter()
                .map(|(k, v)| (k.clone(), resolve_theme_vars(v, &self.theme_vars)))
                .collect();

            let bg_color_key = if resolved_props.contains_key("background") {
                "background"
            } else if resolved_props.contains_key("bg_color") {
                "bg_color"
            } else {
                ""
            };

            // 对于按钮类组件，需要同时设置 normal/hover/pressed 三个状态
            // 否则 class 只覆盖 normal 状态，hover/pressed 会跳回主题默认色
            let is_button = matches!(tag, "Button" | "CheckButton" | "TextureButton" | "OptionButton");

            if is_button && !bg_color_key.is_empty() {
                if let Some(color) = parse_color(resolved_props.get(bg_color_key).unwrap()) {
                    let border_color = resolved_props.get("border_color")
                        .and_then(|v| parse_color(v))
                        .or_else(|| get_theme_color(&self.theme_vars, "border_default"));
                    let border_width = resolved_props.get("border_width")
                        .and_then(|v| v.parse::<i32>().ok())
                        .unwrap_or(2);
                    let border_radius = resolved_props.get("border_radius")
                        .and_then(|v| v.parse::<i32>().ok())
                        .unwrap_or(12);
                    let padding_str = resolved_props.get("padding").map(|s| s.as_str());

                    // normal 状态
                    let mut normal_box = StyleBoxFlat::new_gd();
                    normal_box.set_bg_color(color);
                    normal_box.set_corner_radius_all(border_radius);
                    normal_box.set_content_margin_all(8.0);
                    normal_box.set_content_margin(Side::LEFT, 16.0);
                    normal_box.set_content_margin(Side::RIGHT, 16.0);
                    if let Some(bc) = border_color {
                        normal_box.set_border_color(bc);
                        normal_box.set_border_width_all(border_width);
                    }
                    if let Some(ps) = padding_str {
                        apply_stylebox_padding(&mut normal_box, ps);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("normal"),
                        &normal_box,
                    );

                    // hover 状态 - 变亮 + accent 边框
                    let mut hover_box = StyleBoxFlat::new_gd();
                    hover_box.set_bg_color(Color::from_rgba(
                        (color.r + 0.12).min(1.0),
                        (color.g + 0.12).min(1.0),
                        (color.b + 0.12).min(1.0),
                        color.a,
                    ));
                    hover_box.set_corner_radius_all(border_radius);
                    hover_box.set_content_margin_all(8.0);
                    hover_box.set_content_margin(Side::LEFT, 16.0);
                    hover_box.set_content_margin(Side::RIGHT, 16.0);
                    if let Some(bc) = get_theme_color(&self.theme_vars, "border_accent") {
                        hover_box.set_border_color(bc);
                        hover_box.set_border_width_all(border_width);
                    }
                    if let Some(ps) = padding_str {
                        apply_stylebox_padding(&mut hover_box, ps);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("hover"),
                        &hover_box,
                    );

                    // pressed 状态 - 变暗
                    let mut pressed_box = StyleBoxFlat::new_gd();
                    pressed_box.set_bg_color(Color::from_rgba(
                        (color.r - 0.1).max(0.0),
                        (color.g - 0.1).max(0.0),
                        (color.b - 0.1).max(0.0),
                        color.a,
                    ));
                    pressed_box.set_corner_radius_all(border_radius);
                    pressed_box.set_content_margin_all(8.0);
                    pressed_box.set_content_margin(Side::LEFT, 16.0);
                    pressed_box.set_content_margin(Side::RIGHT, 16.0);
                    if let Some(bc) = border_color {
                        pressed_box.set_border_color(bc);
                        pressed_box.set_border_width_all(border_width);
                    }
                    if let Some(ps) = padding_str {
                        apply_stylebox_padding(&mut pressed_box, ps);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("pressed"),
                        &pressed_box,
                    );
                }
            } else if tag == "ProgressBar" {
                // ProgressBar 特殊处理：background → fill（填充色），track → background（轨道色）
                let border_radius = resolved_props.get("border_radius")
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(4);

                // fill 样式（填充部分）
                if !bg_color_key.is_empty() {
                    if let Some(color) = parse_color(resolved_props.get(bg_color_key).unwrap()) {
                        let mut fill_box = StyleBoxFlat::new_gd();
                        fill_box.set_bg_color(color);
                        fill_box.set_corner_radius_all(border_radius);
                        control.add_theme_stylebox_override(
                            &StringName::from("fill"),
                            &fill_box,
                        );
                    }
                }

                // track 样式（轨道背景）
                let track_color = resolved_props.get("track")
                    .and_then(|v| parse_color(v))
                    .unwrap_or(Color::from_rgba(0.2, 0.2, 0.2, 1.0));
                let mut bg_box = StyleBoxFlat::new_gd();
                bg_box.set_bg_color(track_color);
                bg_box.set_corner_radius_all(border_radius);
                control.add_theme_stylebox_override(
                    &StringName::from("background"),
                    &bg_box,
                );
            } else {
                // 非按钮组件，或按钮没有设置背景色的情况
                // 仅当有背景/边框/圆角/padding属性时才创建 StyleBoxFlat
                let needs_stylebox = !bg_color_key.is_empty()
                    || resolved_props.contains_key("border_radius")
                    || resolved_props.contains_key("border_color")
                    || resolved_props.contains_key("border_width")
                    || resolved_props.contains_key("padding");

                if needs_stylebox {
                    let mut style_box = StyleBoxFlat::new_gd();

                    if !bg_color_key.is_empty() {
                        if let Some(color) = parse_color(resolved_props.get(bg_color_key).unwrap()) {
                            style_box.set_bg_color(color);
                        }
                    }

                    if let Some(border_radius) = resolved_props.get("border_radius") {
                        if let Ok(r) = border_radius.parse::<i32>() {
                            style_box.set_corner_radius_all(r);
                        }
                    }

                    if let Some(border_color) = resolved_props.get("border_color") {
                        if let Some(color) = parse_color(border_color) {
                            style_box.set_border_color(color);
                            let border_width = resolved_props.get("border_width")
                                .and_then(|v| v.parse::<i32>().ok())
                                .unwrap_or(1);
                            style_box.set_border_width_all(border_width);
                        }
                    }

                    if let Some(padding) = resolved_props.get("padding") {
                        apply_stylebox_padding(&mut style_box, padding);
                    }

                    // 将 StyleBox 应用到控件
                    let stylebox_name = get_stylebox_name_for_tag(tag);
                    control.add_theme_stylebox_override(
                        &StringName::from(stylebox_name),
                        &style_box,
                    );
                }
            }

            // 应用 color 属性（文字颜色）到控件
            if let Some(color_str) = resolved_props.get("color") {
                if let Some(color) = parse_color(color_str) {
                    apply_text_color(control, tag, color);
                }
            }

            // 应用 texture 属性（纹理）到 TextureButton / NinePatchRect
            if let Some(texture_path) = resolved_props.get("texture") {
                if tag == "TextureButton" {
                    let path = GString::from(texture_path);
                    if let Some(res) = ResourceLoader::singleton().load(&path) {
                        if let Ok(tex) = res.try_cast::<Texture2D>() {
                            let mut tb = control.clone().cast::<TextureButton>();
                            tb.set_texture_normal(&tex);
                        }
                    }
                } else if tag == "NinePatchRect" {
                    let path = GString::from(texture_path);
                    if let Some(res) = ResourceLoader::singleton().load(&path) {
                        if let Ok(tex) = res.try_cast::<Texture2D>() {
                            let mut nr = control.clone().cast::<NinePatchRect>();
                            nr.set_texture(&tex);
                        }
                    }
                }
            }
        }
    }

    /// 根据组件类型自动应用主题默认颜色
    /// 卡通风格：大圆角(12px)、鲜明边框、活泼 hover/pressed 变化
    /// 在 class 样式之前调用，class 样式可覆盖这些默认值
    fn apply_theme_defaults(&self, control: &mut Gd<Control>, tag: &str) {
        if self.theme_vars.is_empty() {
            return;
        }

        match tag {
            // Panel：白色背景 + 紫色边框 + 大圆角 + 内边距
            "Panel" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "panel_bg") {
                    let mut style_box = StyleBoxFlat::new_gd();
                    style_box.set_bg_color(color);
                    style_box.set_corner_radius_all(12);
                    style_box.set_content_margin_all(8.0);
                    if let Some(border_color) = get_theme_color(&self.theme_vars, "border_default") {
                        style_box.set_border_color(border_color);
                        style_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("panel"),
                        &style_box,
                    );
                }
            }
            // Button / CheckButton：大圆角 + 边框 + 内边距 + 鲜明 hover/pressed
            "Button" | "CheckButton" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "button_bg") {
                    let border_color = get_theme_color(&self.theme_vars, "border_default");
                    // normal 状态
                    let mut style_box = StyleBoxFlat::new_gd();
                    style_box.set_bg_color(color);
                    style_box.set_corner_radius_all(12);
                    style_box.set_content_margin_all(8.0);
                    style_box.set_content_margin(Side::LEFT, 16.0);
                    style_box.set_content_margin(Side::RIGHT, 16.0);
                    if let Some(bc) = border_color {
                        style_box.set_border_color(bc);
                        style_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("normal"),
                        &style_box,
                    );
                    // hover 状态 - 明显变亮
                    let mut hover_box = StyleBoxFlat::new_gd();
                    hover_box.set_bg_color(Color::from_rgba(
                        (color.r + 0.12).min(1.0),
                        (color.g + 0.12).min(1.0),
                        (color.b + 0.12).min(1.0),
                        color.a,
                    ));
                    hover_box.set_corner_radius_all(12);
                    hover_box.set_content_margin_all(8.0);
                    hover_box.set_content_margin(Side::LEFT, 16.0);
                    hover_box.set_content_margin(Side::RIGHT, 16.0);
                    if let Some(bc) = get_theme_color(&self.theme_vars, "border_accent") {
                        hover_box.set_border_color(bc);
                        hover_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("hover"),
                        &hover_box,
                    );
                    // pressed 状态 - 明显变暗
                    let mut pressed_box = StyleBoxFlat::new_gd();
                    pressed_box.set_bg_color(Color::from_rgba(
                        (color.r - 0.1).max(0.0),
                        (color.g - 0.1).max(0.0),
                        (color.b - 0.1).max(0.0),
                        color.a,
                    ));
                    pressed_box.set_corner_radius_all(12);
                    pressed_box.set_content_margin_all(8.0);
                    pressed_box.set_content_margin(Side::LEFT, 16.0);
                    pressed_box.set_content_margin(Side::RIGHT, 16.0);
                    if let Some(bc) = border_color {
                        pressed_box.set_border_color(bc);
                        pressed_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("pressed"),
                        &pressed_box,
                    );
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "button_font_color") {
                    control.add_theme_color_override(
                        &StringName::from("font_color"),
                        color,
                    );
                    control.add_theme_color_override(
                        &StringName::from("font_hover_color"),
                        Color::from_rgba(
                            (color.r + 0.15).min(1.0),
                            (color.g + 0.15).min(1.0),
                            (color.b + 0.15).min(1.0),
                            color.a,
                        ),
                    );
                    control.add_theme_color_override(
                        &StringName::from("font_pressed_color"),
                        Color::from_rgba(
                            (color.r - 0.1).max(0.0),
                            (color.g - 0.1).max(0.0),
                            (color.b - 0.1).max(0.0),
                            color.a,
                        ),
                    );
                }
                // 卡通风格按钮默认字号 16
                control.add_theme_font_size_override(
                    &StringName::from("font_size"),
                    16,
                );
            }
            // Label：设置文字色 + 默认字号
            "Label" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "label_font_color") {
                    control.add_theme_color_override(
                        &StringName::from("font_color"),
                        color,
                    );
                }
                // 卡通风格默认字号 16，确保可读性
                control.add_theme_font_size_override(
                    &StringName::from("font_size"),
                    16,
                );
            }
            // LineEdit：白色背景 + 边框 + 大圆角 + 内边距
            "LineEdit" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "input_bg") {
                    let mut style_box = StyleBoxFlat::new_gd();
                    style_box.set_bg_color(color);
                    style_box.set_corner_radius_all(12);
                    style_box.set_content_margin_all(8.0);
                    style_box.set_content_margin(Side::LEFT, 12.0);
                    style_box.set_content_margin(Side::RIGHT, 12.0);
                    if let Some(border_color) = get_theme_color(&self.theme_vars, "border_default") {
                        style_box.set_border_color(border_color);
                        style_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("normal"),
                        &style_box,
                    );
                    // focus 状态 - 强调边框
                    let mut focus_box = StyleBoxFlat::new_gd();
                    focus_box.set_bg_color(color);
                    focus_box.set_corner_radius_all(12);
                    focus_box.set_content_margin_all(8.0);
                    focus_box.set_content_margin(Side::LEFT, 12.0);
                    focus_box.set_content_margin(Side::RIGHT, 12.0);
                    if let Some(border_color) = get_theme_color(&self.theme_vars, "border_accent") {
                        focus_box.set_border_color(border_color);
                        focus_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("focus"),
                        &focus_box,
                    );
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "input_font_color") {
                    control.add_theme_color_override(
                        &StringName::from("font_color"),
                        color,
                    );
                }
            }
            // OptionButton：大圆角 + 边框 + 内边距
            "OptionButton" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "optionbutton_bg") {
                    let mut style_box = StyleBoxFlat::new_gd();
                    style_box.set_bg_color(color);
                    style_box.set_corner_radius_all(12);
                    style_box.set_content_margin_all(8.0);
                    style_box.set_content_margin(Side::LEFT, 12.0);
                    style_box.set_content_margin(Side::RIGHT, 12.0);
                    if let Some(border_color) = get_theme_color(&self.theme_vars, "border_default") {
                        style_box.set_border_color(border_color);
                        style_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("normal"),
                        &style_box,
                    );
                    // hover 状态
                    let mut hover_box = StyleBoxFlat::new_gd();
                    hover_box.set_bg_color(Color::from_rgba(
                        (color.r + 0.12).min(1.0),
                        (color.g + 0.12).min(1.0),
                        (color.b + 0.12).min(1.0),
                        color.a,
                    ));
                    hover_box.set_corner_radius_all(12);
                    hover_box.set_content_margin_all(8.0);
                    hover_box.set_content_margin(Side::LEFT, 12.0);
                    hover_box.set_content_margin(Side::RIGHT, 12.0);
                    if let Some(bc) = get_theme_color(&self.theme_vars, "border_accent") {
                        hover_box.set_border_color(bc);
                        hover_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("hover"),
                        &hover_box,
                    );
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "optionbutton_font_color") {
                    control.add_theme_color_override(
                        &StringName::from("font_color"),
                        color,
                    );
                }
            }
            // HSeparator / VSeparator：设置分隔线颜色
            "HSeparator" | "VSeparator" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "separator_color") {
                    control.add_theme_color_override(
                        &StringName::from("color"),
                        color,
                    );
                }
            }
            // ProgressBar：设置填充色和轨道色
            "ProgressBar" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "progress_fill") {
                    let mut fill_box = StyleBoxFlat::new_gd();
                    fill_box.set_bg_color(color);
                    fill_box.set_corner_radius_all(4);
                    control.add_theme_stylebox_override(
                        &StringName::from("fill"),
                        &fill_box,
                    );
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "progress_bg") {
                    let mut bg_box = StyleBoxFlat::new_gd();
                    bg_box.set_bg_color(color);
                    bg_box.set_corner_radius_all(4);
                    control.add_theme_stylebox_override(
                        &StringName::from("background"),
                        &bg_box,
                    );
                }
            }
            // TabContainer：卡通风格标签页
            "TabContainer" => {
                // 内容区域面板
                if let Some(color) = get_theme_color(&self.theme_vars, "tab_bg") {
                    let mut style_box = StyleBoxFlat::new_gd();
                    style_box.set_bg_color(color);
                    style_box.set_corner_radius_all(12);
                    style_box.set_content_margin_all(8.0);
                    if let Some(border_color) = get_theme_color(&self.theme_vars, "border_default") {
                        style_box.set_border_color(border_color);
                        style_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("panel"),
                        &style_box,
                    );
                }
                // 标签栏背景
                let tab_bar_bg = get_theme_color(&self.theme_vars, "bg_secondary")
                    .unwrap_or(Color::from_rgba(0.93, 0.91, 0.97, 1.0));
                let mut tab_bar_box = StyleBoxFlat::new_gd();
                tab_bar_box.set_bg_color(tab_bar_bg);
                tab_bar_box.set_corner_radius_all(8);
                control.add_theme_stylebox_override(
                    &StringName::from("tab_bar_background"),
                    &tab_bar_box,
                );
                // 选中标签
                if let Some(color) = get_theme_color(&self.theme_vars, "tab_selected_bg") {
                    let mut selected_box = StyleBoxFlat::new_gd();
                    selected_box.set_bg_color(color);
                    selected_box.set_corner_radius_all(8);
                    selected_box.set_content_margin_all(6.0);
                    selected_box.set_content_margin(Side::LEFT, 12.0);
                    selected_box.set_content_margin(Side::RIGHT, 12.0);
                    if let Some(border_color) = get_theme_color(&self.theme_vars, "border_accent") {
                        selected_box.set_border_color(border_color);
                        selected_box.set_border_width_all(2);
                    }
                    control.add_theme_stylebox_override(
                        &StringName::from("tab_selected"),
                        &selected_box,
                    );
                }
                // 未选中标签
                let unselected_bg = get_theme_color(&self.theme_vars, "bg_button")
                    .unwrap_or(Color::from_rgba(0.91, 0.87, 0.96, 1.0));
                let mut unselected_box = StyleBoxFlat::new_gd();
                unselected_box.set_bg_color(unselected_bg);
                unselected_box.set_corner_radius_all(8);
                unselected_box.set_content_margin_all(6.0);
                unselected_box.set_content_margin(Side::LEFT, 12.0);
                unselected_box.set_content_margin(Side::RIGHT, 12.0);
                control.add_theme_stylebox_override(
                    &StringName::from("tab_unselected"),
                    &unselected_box,
                );
                // 悬停标签
                let mut hovered_box = StyleBoxFlat::new_gd();
                hovered_box.set_bg_color(Color::from_rgba(
                    (unselected_bg.r + 0.08).min(1.0),
                    (unselected_bg.g + 0.08).min(1.0),
                    (unselected_bg.b + 0.08).min(1.0),
                    unselected_bg.a,
                ));
                hovered_box.set_corner_radius_all(8);
                hovered_box.set_content_margin_all(6.0);
                hovered_box.set_content_margin(Side::LEFT, 12.0);
                hovered_box.set_content_margin(Side::RIGHT, 12.0);
                control.add_theme_stylebox_override(
                    &StringName::from("tab_hovered"),
                    &hovered_box,
                );
                // 标签文字颜色
                if let Some(color) = get_theme_color(&self.theme_vars, "tab_selected_font_color") {
                    control.add_theme_color_override(
                        &StringName::from("font_selected_color"),
                        color,
                    );
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "tab_font_color") {
                    control.add_theme_color_override(
                        &StringName::from("font_unselected_color"),
                        color,
                    );
                    control.add_theme_color_override(
                        &StringName::from("font_hovered_color"),
                        Color::from_rgba(
                            (color.r + 0.15).min(1.0),
                            (color.g + 0.15).min(1.0),
                            (color.b + 0.15).min(1.0),
                            color.a,
                        ),
                    );
                }
                // 标签字号
                control.add_theme_font_size_override(
                    &StringName::from("font_size"),
                    16,
                );
            }
            // PopupPanel：设置弹窗默认颜色
            "PopupPanel" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_bg") {
                    control.set(&StringName::from("popup_bg_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_border") {
                    control.set(&StringName::from("popup_border_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "overlay") {
                    control.set(&StringName::from("overlay_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_title_color") {
                    control.set(&StringName::from("title_color"), &color.to_variant());
                }
            }
            // Drawer：设置抽屉默认颜色
            "Drawer" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_bg") {
                    control.set(&StringName::from("drawer_bg_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_border") {
                    control.set(&StringName::from("drawer_border_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "overlay") {
                    control.set(&StringName::from("overlay_color"), &color.to_variant());
                }
            }
            // Tooltip：设置提示框默认颜色
            "Tooltip" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_bg") {
                    control.set(&StringName::from("bg_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_border") {
                    control.set(&StringName::from("border_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "tooltip_title_color") {
                    control.set(&StringName::from("title_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "tooltip_content_color") {
                    control.set(&StringName::from("content_color"), &color.to_variant());
                }
            }
            // NavMenu：设置导航菜单默认颜色
            "NavMenu" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_bg") {
                    control.set(&StringName::from("menu_bg_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "popup_border") {
                    control.set(&StringName::from("menu_border_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "overlay") {
                    control.set(&StringName::from("overlay_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "nav_item_color") {
                    control.set(&StringName::from("item_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "nav_item_hover_color") {
                    control.set(&StringName::from("item_hover_color"), &color.to_variant());
                }
                if let Some(color) = get_theme_color(&self.theme_vars, "nav_item_active_color") {
                    control.set(&StringName::from("item_active_color"), &color.to_variant());
                }
            }
            // TextureButton：设置文字色（如果有叠加 Label）
            "TextureButton" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "button_font_color") {
                    // TextureButton 的文字在叠加的 Label 上
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
            }
            // NinePatchRect（按钮模式）：设置子 Label 文字色
            "NinePatchRect" => {
                if let Some(color) = get_theme_color(&self.theme_vars, "button_font_color") {
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
            }
            _ => {}
        }
    }
}

/// 应用根节点属性
fn apply_root_attribute(control: &mut Gd<Control>, key: &str, value: &str) {
    match key {
        "theme" => {
            // 主题名称，已由 UiBuilder::build() 处理
            // 此处存储为 meta 供 GdGmlScene 读取
            control.set_meta(
                &StringName::from("__theme_name"),
                &GString::from(value).to_variant(),
            );
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
                    // //godot_print!("[UiBuilder] Unhandled root attribute: {}='{}'", key, value);
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
                    // //godot_print!("[UiBuilder] Cannot set text on <{}>", tag);
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
        "patch_margin" => {
            if tag == "NinePatchRect" {
                let mut nr = control.cast::<NinePatchRect>();
                // 支持格式: "10" (四边相同) 或 "10 5 10 5" (left top right bottom)
                let parts: Vec<i32> = value.split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() == 1 {
                    nr.set_patch_margin(Side::LEFT, parts[0] as i32);
                    nr.set_patch_margin(Side::TOP, parts[0] as i32);
                    nr.set_patch_margin(Side::RIGHT, parts[0] as i32);
                    nr.set_patch_margin(Side::BOTTOM, parts[0] as i32);
                } else if parts.len() == 4 {
                    nr.set_patch_margin(Side::LEFT, parts[0] as i32);
                    nr.set_patch_margin(Side::TOP, parts[1] as i32);
                    nr.set_patch_margin(Side::RIGHT, parts[2] as i32);
                    nr.set_patch_margin(Side::BOTTOM, parts[3] as i32);
                }
                return nr.upcast();
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
                // //godot_print!("[UiBuilder] Setting __data_var='{}' on node '{}' (tag={})", value, control.get_name(), tag);
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
        "valign" => {
            use godot::global::VerticalAlignment;
            let alignment = match value {
                "top" => VerticalAlignment::TOP,
                "center" => VerticalAlignment::CENTER,
                "bottom" => VerticalAlignment::BOTTOM,
                _ => VerticalAlignment::TOP,
            };
            match tag {
                "Label" => {
                    let mut lbl = control.cast::<Label>();
                    lbl.set_vertical_alignment(alignment);
                    return lbl.upcast();
                }
                _ => {}
            }
        }
        "font_color" => {
            if let Some(color) = parse_color(value) {
                apply_text_color(&mut control, tag, color);
            }
        }
        "anchor_left" | "anchor_right" | "anchor_top" | "anchor_bottom" => {
            if let Ok(val) = value.parse::<f32>() {
                let side = match key {
                    "anchor_left" => Side::LEFT,
                    "anchor_right" => Side::RIGHT,
                    "anchor_top" => Side::TOP,
                    "anchor_bottom" => Side::BOTTOM,
                    _ => return control,
                };
                control.set_anchor(side, val);
            }
        }
        "offset_left" | "offset_right" | "offset_top" | "offset_bottom" => {
            if let Ok(val) = value.parse::<f32>() {
                let side = match key {
                    "offset_left" => Side::LEFT,
                    "offset_right" => Side::RIGHT,
                    "offset_top" => Side::TOP,
                    "offset_bottom" => Side::BOTTOM,
                    _ => return control,
                };
                control.set_offset(side, val);
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
        "width" => {
            if tag == "PopupPanel" {
                if let Some(pct) = parse_percent(value) {
                    control.set_meta(
                        &StringName::from("__pct_popup_width"),
                        &pct.to_variant(),
                    );
                } else if let Ok(w) = value.parse::<i32>() {
                    control.set(&StringName::from("popup_width"), &w.to_variant());
                }
            }
        }
        "height" => {
            if tag == "PopupPanel" {
                if let Some(pct) = parse_percent(value) {
                    control.set_meta(
                        &StringName::from("__pct_popup_height"),
                        &pct.to_variant(),
                    );
                } else if let Ok(h) = value.parse::<i32>() {
                    control.set(&StringName::from("popup_height"), &h.to_variant());
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
                if let Some(pct) = parse_percent(value) {
                    control.set_meta(
                        &StringName::from("__pct_slide_width"),
                        &pct.to_variant(),
                    );
                } else if let Ok(val) = value.parse::<i32>() {
                    control.set(&StringName::from("slide_width"), &val.to_variant());
                }
            }
        }
        "menu_width" | "sub_menu_width" => {
            if tag == "NavMenu" {
                let meta_key = format!("__pct_{}", key);
                if let Some(pct) = parse_percent(value) {
                    control.set_meta(
                        &StringName::from(meta_key.as_str()),
                        &pct.to_variant(),
                    );
                } else if let Ok(val) = value.parse::<i32>() {
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
        // 动画属性：anim_enter/anim_hover/anim_click
        // 存储为 meta，由 GdGmlScene.setup_animations() 在节点加入场景树后处理
        "anim_enter" => {
            // 值为方向：bottom/top/left/right，默认 bottom
            control.set_meta(&StringName::from("__anim_enter"), &value.to_variant());
        }
        "anim_hover" => {
            // 值为缩放倍数（如 "1.08"）或 "true"（默认 1.08）
            let scale = if value == "true" {
                1.08f32
            } else if let Ok(s) = value.parse::<f32>() {
                s
            } else {
                1.08f32
            };
            control.set_meta(&StringName::from("__anim_hover"), &scale.to_variant());
        }
        "anim_click" => {
            // 值为 "true" 启用点击反馈
            control.set_meta(&StringName::from("__anim_click"), &true.to_variant());
        }
        _ => {
            // //godot_print!("[UiBuilder] Unhandled attribute: {}='{}' on <{}>", key, value, tag);
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
        "NinePatchRect" => {
            // NinePatchRect 的文字在子 Label 上
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
/// 支持百分比: "5%" (四边相同), "5% 3%" (水平 垂直), "5% 3% 5% 3%" (左 上 右 下)
/// 通过设置 offset 属性实现（Side 枚举在 gdext 0.5 中未公开导出）
/// 百分比基于父容器大小，存为 meta 延迟计算
fn apply_margin(control: &mut Gd<Control>, value: &str) {
    let parts: Vec<&str> = value.split_whitespace().collect();

    // 检查是否有百分比
    let has_pct = parts.iter().any(|p| parse_percent(p).is_some());

    if has_pct {
        // 存储百分比信息为 meta，延迟计算
        control.set_meta(
            &StringName::from("__pct_margin"),
            &GString::from(value).to_variant(),
        );
        // 先用像素值设置非百分比部分
        let (left, top, right, bottom) = match parts.len() {
            1 => {
                let (v, is_pct, _) = parse_size_value(parts[0]);
                let val = if is_pct { 0.0 } else { v };
                (val, val, val, val)
            }
            2 => {
                let (h, h_pct, _) = parse_size_value(parts[0]);
                let (v, v_pct, _) = parse_size_value(parts[1]);
                let hval = if h_pct { 0.0 } else { h };
                let vval = if v_pct { 0.0 } else { v };
                (hval, vval, hval, vval)
            }
            4 => {
                let (l, l_pct, _) = parse_size_value(parts[0]);
                let (t, t_pct, _) = parse_size_value(parts[1]);
                let (r, r_pct, _) = parse_size_value(parts[2]);
                let (b, b_pct, _) = parse_size_value(parts[3]);
                (
                    if l_pct { 0.0 } else { l },
                    if t_pct { 0.0 } else { t },
                    if r_pct { 0.0 } else { r },
                    if b_pct { 0.0 } else { b },
                )
            }
            _ => return,
        };
        control.set_offset(Side::LEFT, left);
        control.set_offset(Side::TOP, top);
        control.set_offset(Side::RIGHT, -right);
        control.set_offset(Side::BOTTOM, -bottom);
    } else {
        // 纯像素值，直接设置
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
        control.set_offset(Side::LEFT, left);
        control.set_offset(Side::TOP, top);
        control.set_offset(Side::RIGHT, -right);
        control.set_offset(Side::BOTTOM, -bottom);
    }
}

/// 解析百分比字符串，返回百分比值（0.0~1.0）
/// 支持 "80%" 格式，纯数字返回 None
pub(crate) fn parse_percent(value: &str) -> Option<f32> {
    let v = value.trim();
    if v.ends_with('%') {
        let num_str = &v[..v.len() - 1];
        num_str.trim().parse::<f32>().ok().map(|p| p / 100.0)
    } else {
        None
    }
}

/// 解析尺寸值，支持百分比和像素混合
/// 返回 (像素值, 是否百分比, 百分比值)
/// "80%" -> (0.0, true, 0.8)
/// "400" -> (400.0, false, 0.0)
pub(crate) fn parse_size_value(value: &str) -> (f32, bool, f32) {
    if let Some(pct) = parse_percent(value) {
        (0.0, true, pct)
    } else {
        let px = value.trim().parse::<f32>().unwrap_or(0.0);
        (px, false, 0.0)
    }
}

/// 应用大小
/// 格式: "width,height"，支持百分比如 "80%,50%" 或混合 "80%,400"
/// 百分比基于父容器大小，存为 meta 延迟计算
fn apply_size(control: &mut Gd<Control>, value: &str) {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() == 2 {
        let (w_px, w_pct, _w_pct_val) = parse_size_value(parts[0].trim());
        let (h_px, h_pct, _h_pct_val) = parse_size_value(parts[1].trim());

        if w_pct || h_pct {
            // 存储百分比信息为 meta，延迟计算
            control.set_meta(
                &StringName::from("__pct_size"),
                &GString::from(value).to_variant(),
            );
            // 先设置像素值（非百分比部分）
            let w = if w_pct { 0.0 } else { w_px };
            let h = if h_pct { 0.0 } else { h_px };
            control.set_custom_minimum_size(Vector2::new(w, h));
            control.set_size(Vector2::new(w, h));
        } else {
            control.set_custom_minimum_size(Vector2::new(w_px, h_px));
            control.set_size(Vector2::new(w_px, h_px));
        }
    }
}

/// 应用自定义最小尺寸
/// 格式: "width,height"，支持百分比如 "80%,50%"
/// 百分比基于父容器大小，存为 meta 延迟计算
fn apply_custom_minimum_size(control: &mut Gd<Control>, value: &str) {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() == 2 {
        let (w_px, w_pct, _) = parse_size_value(parts[0].trim());
        let (h_px, h_pct, _) = parse_size_value(parts[1].trim());

        if w_pct || h_pct {
            // 存储百分比信息为 meta，延迟计算
            control.set_meta(
                &StringName::from("__pct_min_size"),
                &GString::from(value).to_variant(),
            );
            // 先设置像素值（非百分比部分）
            let w = if w_pct { 0.0 } else { w_px };
            let h = if h_pct { 0.0 } else { h_px };
            control.set_custom_minimum_size(Vector2::new(w, h));
        } else {
            control.set_custom_minimum_size(Vector2::new(w_px, h_px));
        }
    }
}

/// 应用 padding 到 StyleBoxFlat
/// 支持格式: "20" (四边相同), "20 12" (上下 左右), "10 20 10 20" (上 右 下 左)
fn apply_stylebox_padding(style_box: &mut Gd<StyleBoxFlat>, value: &str) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            if let Ok(p) = parts[0].parse::<f32>() {
                style_box.set_content_margin_all(p);
            }
        }
        2 => {
            let v = parts[0].parse::<f32>().unwrap_or(0.0);
            let h = parts[1].parse::<f32>().unwrap_or(0.0);
            style_box.set_content_margin(Side::TOP, v);
            style_box.set_content_margin(Side::BOTTOM, v);
            style_box.set_content_margin(Side::LEFT, h);
            style_box.set_content_margin(Side::RIGHT, h);
        }
        4 => {
            let t = parts[0].parse::<f32>().unwrap_or(0.0);
            let r = parts[1].parse::<f32>().unwrap_or(0.0);
            let b = parts[2].parse::<f32>().unwrap_or(0.0);
            let l = parts[3].parse::<f32>().unwrap_or(0.0);
            style_box.set_content_margin(Side::TOP, t);
            style_box.set_content_margin(Side::RIGHT, r);
            style_box.set_content_margin(Side::BOTTOM, b);
            style_box.set_content_margin(Side::LEFT, l);
        }
        _ => {}
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
                    if let Some(target) = root.find_child_ex(&GString::from(target_name.as_str())).recursive(true).owned(false).done() {
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

/// 解析 <Gml src="..."> 的目标路径：
/// - res:// / user:// / absolute 开头：按原样使用
/// - 相对路径：基于引用方文件所在目录；无目录上下文时回退到 res:// 根
fn resolve_include_path(src: &str, base_dir: Option<&str>) -> String {
    if src.starts_with("res://") || src.starts_with("user://") || src.starts_with('/') {
        return src.to_string();
    }
    match base_dir {
        Some(dir) => {
            let dir = dir.trim_end_matches('/');
            if dir.is_empty() || dir == "res:" {
                format!("res://{}", src)
            } else {
                format!("{}/{}", dir, src)
            }
        }
        None => format!("res://{}", src),
    }
}

/// 取文件路径的父目录（"res://a/b/c.gml" -> "res://a/b"），无父目录时返回 None
pub(crate) fn parent_dir_of(path: &str) -> Option<String> {
    let p = path.trim_end_matches('/');
    let idx = p.rfind('/')?;
    let dir = &p[..idx];
    if dir.is_empty() || dir == "res:" || dir == "user:" {
        None
    } else {
        Some(dir.to_string())
    }
}

/// 节点是否为列表控件（UIVList/UIHList/UIGrid）
fn is_list_control(node: &Gd<Control>) -> bool {
    node.clone().try_cast::<GdUIVList>().is_ok()
        || node.clone().try_cast::<GdUIHList>().is_ok()
        || node.clone().try_cast::<GdUIGrid>().is_ok()
}

/// 深度优先查找后代中最近的列表控件（<Gml data=...> 覆盖容器根时使用）
fn find_list_descendant(root: &Gd<Control>) -> Option<Gd<Control>> {
    let children = root.get_children();
    for i in 0..children.len() {
        if let Some(child) = children.get(i) {
            if let Ok(control) = child.try_cast::<Control>() {
                if is_list_control(&control) {
                    return Some(control);
                }
                if let Some(found) = find_list_descendant(&control) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// DataValue -> Godot Variant（数组/对象递归转换）
fn data_value_to_variant(value: &DataValue) -> Variant {
    match value {
        DataValue::Str(s) => GString::from(s.as_str()).to_variant(),
        DataValue::Num(n) => n.to_variant(),
        DataValue::Bool(b) => b.to_variant(),
        DataValue::Null => Variant::nil(),
        DataValue::Array(_) => data_value_to_array(value).to_variant(),
        DataValue::Dict(pairs) => {
            let mut dict: Dictionary<Variant, Variant> = Dictionary::new();
            for (k, v) in pairs {
                dict.set(&Variant::from(k.as_str()), &data_value_to_variant(v));
            }
            dict.to_variant()
        }
    }
}

/// DataValue::Array -> godot Array<Variant>（非数组返回空数组）
fn data_value_to_array(value: &DataValue) -> Array<Variant> {
    let mut arr = Array::new();
    if let DataValue::Array(items) = value {
        for item in items {
            arr.push(&data_value_to_variant(item));
        }
    }
    arr
}
