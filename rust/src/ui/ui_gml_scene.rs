// GdGmlScene - GML 文件加载节点
// 继承 Control，设置 file 属性即可加载 .gml 文件并显示为 Control 节点树
// 用法：
//   在场景中添加 GmlScene 节点，设置 gml_file 属性为 .gml 文件路径
//   或在 GDScript 中：var gml = GmlScene.new(); gml.gml_file = "res://ui/scene.gml"; add_child(gml)
//   信号连接：auto_connect=true 时自动连接到 GdGmlScene 自身脚本
//   可创建继承 GdGmlScene 的 GDScript，在其中定义回调方法

use godot::prelude::*;
use godot::builtin::{GString, StringName, Variant, VariantType, Array, Vector2, Side, PackedStringArray};
use godot::classes::{
    IControl, Control, FileAccess, Node,
};
use godot::classes::control::LayoutPreset;
use godot::classes::notify::ControlNotification;
use godot::obj::WithBaseField;

use super::parser::UiParser;
use super::builder::{UiBuilder, parse_size_value};
use super::gdui_builder::connect_signals_recursive;
use super::ui_theme::{ThemeVars, get_builtin_theme, builtin_theme_names, resolve_theme_vars};
use crate::state::bean::{get_bean_by_id, get_all_bean_instances};

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdGmlScene {
    base: Base<Control>,

    /// 是否在 ready 时自动连接信号到自身脚本
    #[export]
    auto_connect: bool,

    // 内部状态
    content_root: Option<Gd<Control>>,
    loaded: bool,
}

#[godot_api]
impl IControl for GdGmlScene {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            auto_connect: true,
            content_root: None,
            loaded: false,
        }
    }

    fn on_notification(&mut self, what: ControlNotification) {
        if what == ControlNotification::RESIZED {
            // 窗口/父容器大小变化时，重新计算百分比布局
            if self.loaded {
                self.refresh_percent_layouts();
            }
        }
    }
}

#[godot_api]
impl GdGmlScene {
    /// 信号：GML 加载完成
    #[signal]
    fn s_gml_loaded();

    /// 信号：GML 加载失败
    #[signal]
    fn s_gml_load_failed(error: GString);

    /// 手动加载当前 gml_file 指定的文件
    #[func]
    fn load_gml(&mut self, gml_file: GString) {
        let path = gml_file.clone();
        let path_str = path.to_string();

        // 读取文件
        let fa = FileAccess::open(&path, godot::classes::file_access::ModeFlags::READ);
        if fa.is_none() {
            godot_error!("[GmlScene] Cannot open file: {}", path_str);
            self.base_mut().emit_signal(
                &StringName::from("s_gml_load_failed"),
                &[GString::from(&format!("Cannot open file: {}", path_str)).to_variant()],
            );
            return;
        }

        let fa = unsafe { fa.unwrap_unchecked() };
        let content = fa.get_as_text();

        self.clear_content();
        self.parse_and_build(&content.to_string());
    }

    /// 从字符串加载 GML 内容
    #[func]
    fn load_from_string(&mut self, gml_content: GString) {
        self.parse_and_build(&gml_content.to_string());
    }
    /// 连接 GML 中定义的信号到目标对象
    #[func]
    fn connect_signals(&mut self, target: Gd<Object>) {
        if let Some(ref mut root) = self.content_root {
            connect_signals_recursive(root, &target);
        }
    }

    /// 获取内容根节点
    #[func]
    fn get_content(&self) -> Option<Gd<Control>> {
        self.content_root.clone()
    }

    /// 按 name 查找内容中的子节点
    /// 使用 owned=false 以支持查找 PopupPanel/Drawer/Tooltip 内部内容区域的子节点
    /// （这些子节点的 owner 被设为 content_container 而非场景根节点）
    #[func]
    fn find_node(&self, name: GString) -> Option<Gd<Control>> {
        if let Some(ref root) = self.content_root {
            let found = root.find_child_ex(&name).recursive(true).owned(false).done();
            if let Some(node) = found {
                return node.try_cast::<Control>().ok();
            }
        }
        None
    }

    /// 清除已加载的内容
    #[func]
    fn clear_content(&mut self) {
        if let Some(ref mut root) = self.content_root {
            root.queue_free();
        }
        self.content_root = None;
        self.loaded = false;
    }

    /// 是否已加载
    #[func]
    fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// 切换主题并重新加载（最简单的主题切换方式）
    /// 传入内置主题名称（cartoon），自动修改 GML 中的 theme 属性并重新加载
    #[func]
    fn apply_theme(&mut self, theme_name: GString) {
        if !self.loaded {
            return;
        }
        if let Some(ref root) = self.content_root {
            if root.has_meta(&StringName::from("__gml_content")) {
                let gml_content: GString = root.get_meta(&StringName::from("__gml_content")).to();
                let mut content = gml_content.to_string();
                // 替换 <ui theme="xxx"> 中的 theme 属性值
                let new_theme = theme_name.to_string();
                let re = regex_lite::Regex::new(r#"<ui\s+theme="[^"]*""#).unwrap();
                if re.is_match(&content) {
                    content = re.replace_all(&content, format!(r#"<ui theme="{}""#, new_theme)).to_string();
                } else {
                    // 没有 theme 属性，添加一个
                    content = content.replacen("<ui", &format!("<ui theme=\"{}\"", new_theme), 1);
                }
                self.parse_and_build(&content);
            }
        }
    }

    /// 获取所有内置主题名称
    #[func]
    fn get_builtin_themes(&self) -> PackedStringArray {
        let names: Vec<GString> = builtin_theme_names().iter()
            .map(|s| GString::from(*s))
            .collect();
        PackedStringArray::from(names.as_slice())
    }

    /// 延迟重新应用所有带 __anchor meta 的节点的 anchor preset
    /// 节点加入场景树后父容器大小已计算完成，此时 anchor offsets 才能正确设置
    /// 同时刷新百分比布局
    #[func]
    fn refresh_anchors(&mut self) {
        if let Some(ref root) = self.content_root {
            let root_size = root.get_size();
            Self::refresh_anchors_recursive(root);
            Self::refresh_percent_layouts_recursive(root, root_size);
        }
    }

    /// 刷新所有百分比布局（窗口大小变化时调用）
    #[func]
    fn refresh_percent_layouts(&mut self) {
        if let Some(ref root) = self.content_root {
            let root_size = root.get_size();
            Self::refresh_percent_layouts_recursive(root, root_size);
        }
    }

    fn refresh_anchors_recursive(node: &Gd<Control>) {
        if node.has_meta(&StringName::from("__anchor")) {
            let anchor_val: GString = node
                .get_meta(&StringName::from("__anchor"))
                .to();
            let preset = match anchor_val.to_string().as_str() {
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
            let mut node_mut = node.clone();
            node_mut.set_anchors_and_offsets_preset(preset);
        }

        // 递归处理子节点
        let children = node.get_children();
        for i in 0..children.len() {
            if let Some(child_var) = children.get(i) {
                if let Ok(child) = child_var.clone().try_cast::<Control>() {
                    Self::refresh_anchors_recursive(&child);
                }
            }
        }
    }

    /// 递归刷新百分比布局
    /// 所有百分比基于根容器（GmlContent）大小计算，避免嵌套容器中的循环依赖
    /// 处理 __pct_size、__pct_min_size、__pct_margin、__pct_popup_width、__pct_popup_height、__pct_slide_width、__pct_menu_width、__pct_sub_menu_width
    fn refresh_percent_layouts_recursive(node: &Gd<Control>, root_size: Vector2) {
        // 处理 __pct_size: size 属性中的百分比
        if node.has_meta(&StringName::from("__pct_size")) {
            let pct_str: GString = node.get_meta(&StringName::from("__pct_size")).to();
            let pct_string = pct_str.to_string();
            let parts: Vec<&str> = pct_string.split(',').collect();
            if parts.len() == 2 {
                let (w_px, w_pct, w_pct_val) = parse_size_value(parts[0].trim());
                let (h_px, h_pct, h_pct_val) = parse_size_value(parts[1].trim());
                let w = if w_pct { root_size.x * w_pct_val } else { w_px };
                let h = if h_pct { root_size.y * h_pct_val } else { h_px };
                let mut node_mut = node.clone();
                node_mut.set_custom_minimum_size(Vector2::new(w, h));
                node_mut.set_size(Vector2::new(w, h));
            }
        }

        // 处理 __pct_min_size: custom_minimum_size 属性中的百分比
        if node.has_meta(&StringName::from("__pct_min_size")) {
            let pct_str: GString = node.get_meta(&StringName::from("__pct_min_size")).to();
            let pct_string = pct_str.to_string();
            let parts: Vec<&str> = pct_string.split(',').collect();
            if parts.len() == 2 {
                let (w_px, w_pct, w_pct_val) = parse_size_value(parts[0].trim());
                let (h_px, h_pct, h_pct_val) = parse_size_value(parts[1].trim());
                let w = if w_pct { root_size.x * w_pct_val } else { w_px };
                let h = if h_pct { root_size.y * h_pct_val } else { h_px };
                let mut node_mut = node.clone();
                node_mut.set_custom_minimum_size(Vector2::new(w, h));
            }
        }

        // 处理 __pct_margin: margin 属性中的百分比
        if node.has_meta(&StringName::from("__pct_margin")) {
            let pct_str: GString = node.get_meta(&StringName::from("__pct_margin")).to();
            let pct_string = pct_str.to_string();
            let parts: Vec<&str> = pct_string.split_whitespace().collect();
            let (left, top, right, bottom) = match parts.len() {
                1 => {
                    let (v, is_pct, pct_val) = parse_size_value(parts[0]);
                    let val = if is_pct { root_size.x * pct_val } else { v };
                    (val, val, val, val)
                }
                2 => {
                    let (h, h_pct, h_pct_val) = parse_size_value(parts[0]);
                    let (v, v_pct, v_pct_val) = parse_size_value(parts[1]);
                    let hval = if h_pct { root_size.x * h_pct_val } else { h };
                    let vval = if v_pct { root_size.y * v_pct_val } else { v };
                    (hval, vval, hval, vval)
                }
                4 => {
                    let (l, l_pct, l_pct_val) = parse_size_value(parts[0]);
                    let (t, t_pct, t_pct_val) = parse_size_value(parts[1]);
                    let (r, r_pct, r_pct_val) = parse_size_value(parts[2]);
                    let (b, b_pct, b_pct_val) = parse_size_value(parts[3]);
                    (
                        if l_pct { root_size.x * l_pct_val } else { l },
                        if t_pct { root_size.y * t_pct_val } else { t },
                        if r_pct { root_size.x * r_pct_val } else { r },
                        if b_pct { root_size.y * b_pct_val } else { b },
                    )
                }
                _ => (0.0, 0.0, 0.0, 0.0),
            };
            let mut node_mut = node.clone();
            node_mut.set_offset(Side::LEFT, left);
            node_mut.set_offset(Side::TOP, top);
            node_mut.set_offset(Side::RIGHT, -right);
            node_mut.set_offset(Side::BOTTOM, -bottom);
        }

        // 处理 __pct_popup_width: PopupPanel 弹窗宽度的百分比
        if node.has_meta(&StringName::from("__pct_popup_width")) {
            let pct: f32 = node.get_meta(&StringName::from("__pct_popup_width")).to();
            let width = (root_size.x * pct) as i32;
            let mut node_mut = node.clone();
            node_mut.set(&StringName::from("popup_width"), &width.to_variant());
            node_mut.call(&StringName::from("update_layout"), &[]);
        }

        // 处理 __pct_popup_height: PopupPanel 弹窗高度的百分比
        if node.has_meta(&StringName::from("__pct_popup_height")) {
            let pct: f32 = node.get_meta(&StringName::from("__pct_popup_height")).to();
            let height = (root_size.y * pct) as i32;
            let mut node_mut = node.clone();
            node_mut.set(&StringName::from("popup_height"), &height.to_variant());
            node_mut.call(&StringName::from("update_layout"), &[]);
        }

        // 处理 __pct_slide_width: Drawer 抽屉宽度的百分比
        if node.has_meta(&StringName::from("__pct_slide_width")) {
            let pct: f32 = node.get_meta(&StringName::from("__pct_slide_width")).to();
            let width = (root_size.x * pct) as i32;
            let mut node_mut = node.clone();
            node_mut.set(&StringName::from("slide_width"), &width.to_variant());
            // 调用 update_layout 重新计算 DrawerPanel 位置
            node_mut.call(&StringName::from("update_layout"), &[]);
        }

        // 处理 __pct_menu_width: NavMenu 菜单宽度的百分比
        if node.has_meta(&StringName::from("__pct_menu_width")) {
            let pct: f32 = node.get_meta(&StringName::from("__pct_menu_width")).to();
            let width = (root_size.x * pct) as i32;
            let mut node_mut = node.clone();
            node_mut.set(&StringName::from("menu_width"), &width.to_variant());
            // 调用 update_layout 重新计算面板位置
            node_mut.call(&StringName::from("update_layout"), &[]);
        }

        // 处理 __pct_sub_menu_width: NavMenu 子菜单宽度的百分比
        if node.has_meta(&StringName::from("__pct_sub_menu_width")) {
            let pct: f32 = node.get_meta(&StringName::from("__pct_sub_menu_width")).to();
            let width = (root_size.x * pct) as i32;
            let mut node_mut = node.clone();
            node_mut.set(&StringName::from("sub_menu_width"), &width.to_variant());
            // 调用 update_layout 重新计算面板位置
            node_mut.call(&StringName::from("update_layout"), &[]);
        }

        // 递归处理子节点
        let children = node.get_children();
        for i in 0..children.len() {
            if let Some(child_var) = children.get(i) {
                if let Ok(child) = child_var.clone().try_cast::<Control>() {
                    Self::refresh_percent_layouts_recursive(&child, root_size);
                }
            }
        }
    }

    /// 内部实现：根据 node_name 查找节点并用 data 更新
    fn update_node_with_bean_data(&mut self, node_name: GString, data: Variant) {
        if data.get_type() == VariantType::ARRAY {
            if let Some(node) = self.find_node(node_name.clone()) {
                // //godot_print!("[GmlScene] on_bean_data_changed: updating node='{}' with {} items", node_name, data.to::<Array<Variant>>().len());
                let mut node_mut = node;
                node_mut.call(
                    &StringName::from("update"),
                    &[data, Variant::from(true)],
                );
            }
        }
    }

    /// GdBean 响应式回调：属性变更时自动更新对应节点
    /// 通过 bean.watch() 注册，参数为 (node_name, new_value)
    #[func]
    fn on_bean_data_changed(&mut self, node_name: GString, data: Variant) {
        self.update_node_with_bean_data(node_name, data);
    }

    /// GdBean 响应式回调（bind 版）：属性变更时自动更新对应节点
    /// 通过 bean.watch() + callable.bind() 注册，bind 将 node_name 追加到末尾
    /// bean 统一以 2 参数调用回调 (value, metas)，加上 bind 的 node_name 共 3 参数
    #[func]
    fn on_bean_data_changed_bound(&mut self, data: Variant, _metas: Variant, node_name: GString) {
        self.update_node_with_bean_data(node_name, data);
    }

    /// 延迟注册 watch 回调（由 auto_bind_data 通过 call_deferred 调用）
    #[func]
    fn register_watches(&mut self, registrations: Array<Variant>) {
        for i in 0..registrations.len() {
            if let Some(item) = registrations.get(i) {
                let arr: Array<Variant> = item.to();
                if arr.len() == 3 {
                    if let (Some(bean_var), Some(prop_key_var), Some(callable_var)) =
                        (arr.get(0), arr.get(1), arr.get(2))
                    {
                        if let Ok(bean) = bean_var.try_to::<Gd<Object>>() {
                            let prop_key: GString = prop_key_var.to();
                            let callable: Callable = callable_var.to();
                            let mut bean_gd = bean;
                            bean_gd.call_deferred("watch", &[prop_key.to_variant(), callable.to_variant()]);
                        }
                    }
                }
            }
        }
    }
}

impl GdGmlScene {
    /// 解析 GML 文本并构建节点树
    fn parse_and_build(&mut self, content: &str) {
        // 先清除旧内容
        self.clear_content();

        let mut parser = UiParser::new(content);
        match parser.parse() {
            Ok(parse_result) => {
                let mut builder = UiBuilder::new();

                // 注入主题变量：根据 GML 中 <ui theme="xxx"> 属性加载内置主题
                if let Some(ref gml_theme) = parse_result.theme_name {
                    if let Some(builtin_vars) = get_builtin_theme(gml_theme) {
                        let mut theme_vars = builtin_vars;
                        // <theme> 块中的变量覆盖内置主题
                        for (key, value) in &parse_result.theme_vars {
                            theme_vars.insert(key.clone(), value.clone());
                        }
                        builder.set_theme_vars(theme_vars);
                    }
                } else if !parse_result.theme_vars.is_empty() {
                    // 没有 theme 属性，但有 <theme> 块
                    let mut theme_vars = ThemeVars::new();
                    for (key, value) in &parse_result.theme_vars {
                        theme_vars.insert(key.clone(), value.clone());
                    }
                    builder.set_theme_vars(theme_vars);
                }

                match builder.build(&parse_result) {
                    Ok(mut control) => {
                        // 存储 GML 内容到 meta，供 apply_theme 重新加载
                        control.set_meta(
                            &StringName::from("__gml_content"),
                            &GString::from(content).to_variant(),
                        );

                        // 设置内容根节点占满 GmlScene
                        control.set_anchors_and_offsets_preset(
                            godot::classes::control::LayoutPreset::FULL_RECT,
                        );
                        control.set_name("GmlContent");

                        {
                            let mut base = self.base_mut();
                            base.add_child(&control);
                            control.set_owner(&base.clone().upcast::<Node>());
                        }

                        // 延迟一帧重新应用 anchor，因为节点刚加入场景树时
                        // 父容器大小可能还未计算完成，导致 offsets 不正确
                        self.base_mut().call_deferred(
                            &StringName::from("refresh_anchors"),
                            &[],
                        );

                        // 自动连接信号到自身脚本（而非父节点）
                        self.content_root = Some(control.clone());
                        self.loaded = true;

                        if self.auto_connect {
                            let self_obj = self.base().clone().upcast::<Object>();
                            self.connect_signals(self_obj);
                        }

                        // 自动绑定数据：扫描带有 __data_var 元数据的节点，
                        // 从脚本中读取对应变量并调用 update()
                        self.auto_bind_data(&control);

                        self.base_mut().emit_signal(
                            &StringName::from("s_gml_loaded"),
                            &[],
                        );
                    }
                    Err(e) => {
                        godot_error!("[GmlScene] Build error: {}", e);
                        self.base_mut().emit_signal(
                            &StringName::from("s_gml_load_failed"),
                            &[GString::from(&format!("Build error: {}", e)).to_variant()],
                        );
                    }
                }
            }
            Err(e) => {
                godot_error!("[GmlScene] Parse error: {}", e);
                self.base_mut().emit_signal(
                    &StringName::from("s_gml_load_failed"),
                    &[GString::from(&format!("Parse error: {}", e)).to_variant()],
                );
            }
        }
    }

/// 自动绑定数据：扫描节点树中带有 __data_var 元数据的节点，
/// 从脚本中直接读取对应变量并调用 update()
/// watch 回调信息收集后延迟注册，避免与当前 &mut self 借用冲突
fn auto_bind_data(&mut self, root: &Gd<Control>) {
    let all_beans = get_all_bean_instances();
    let bean_ids: Vec<String> = all_beans.iter().map(|(id, _)| id.clone()).collect();
    //godot_print!("[GmlScene] auto_bind_data: registered beans: {:?}", bean_ids);
    let self_node = self.base().clone();
    let watch_registrations = Self::auto_bind_data_recursive(root, &self_node.upcast::<Object>());

    // 延迟注册 watch 回调，避免 bean 立即触发回调时与当前 &mut self 借用冲突
    if !watch_registrations.is_empty() {
        let mut self_gd = self.base().clone().upcast::<Object>();
        self_gd.call_deferred(
            &StringName::from("register_watches"),
            &[Variant::from(watch_registrations)],
        );
    }
}

fn auto_bind_data_recursive(node: &Gd<Control>, script_obj: &Gd<Object>) -> Array<Variant> {
    let mut watch_registrations = Array::new();
    let node_name = node.get_name().to_string();
    // 检查当前节点是否有 __data_var 元数据
    if node.has_meta(&StringName::from("__data_var")) {
        let data_var_name = node.get_meta(&StringName::from("__data_var")).to_string();
        //godot_print!("[GmlScene] auto_bind_data: node='{}' has __data_var='{}'", node_name, data_var_name);
        if !data_var_name.is_empty() {
            // 检查是否为 bean:bean_id:property_key 格式
            if data_var_name.starts_with("bean:") {
                let parts: Vec<&str> = data_var_name[5..].splitn(2, ':').collect();
                if parts.len() == 2 {
                    let bean_id = parts[0];
                    let prop_key = parts[1];
                    if let Some(mut bean) = get_bean_by_id(bean_id) {
                        let data = bean.call("get_value_by_key", &[GString::from(prop_key).to_variant()]);
                        //godot_print!("[GmlScene] auto_bind_data: bean '{}' key '{}' type={:?}", bean_id, prop_key, data.get_type());
                        if data.get_type() == VariantType::ARRAY {
                            let arr = data.to::<Array<Variant>>();
                            //godot_print!("[GmlScene] auto_bind_data: calling update() on node='{}' with {} items", node_name, arr.len());
                            // 日志：打印前3个元素的内容
                            for i in 0..arr.len().min(3) {
                                if let Some(item) = arr.get(i) {
                                    //godot_print!("[GmlScene] auto_bind_data: item[{}] type={:?} value={}", i, item.get_type(), item);
                                }
                            }
                            let mut node_mut = node.clone();
                            node_mut.call(
                                &StringName::from("update"),
                                &[data.clone(), Variant::from(true)],
                            );
                        } else {
                            godot_warn!("[GmlScene] auto_bind_data: bean '{}' key '{}' is not Array (type={:?}), value={}", bean_id, prop_key, data.get_type(), data);
                        }
                        // 收集 watch 注册信息，延迟注册
                        let self_obj = script_obj.clone();
                        let node_name_gstr = GString::from(&node_name);
                        let callable = self_obj.callable("on_bean_data_changed_bound").bind(&[node_name_gstr.to_variant()]);
                        // 将 (bean, prop_key, callable) 打包为 Array<Variant>
                        let reg = Array::from(&[
                            bean.clone().upcast::<Object>().to_variant(),
                            GString::from(prop_key).to_variant(),
                            callable.to_variant(),
                        ]);
                        watch_registrations.push(&reg.to_variant());
                        //godot_print!("[GmlScene] auto_bind_data: will defer-register watch on bean '{}' key '{}' for node '{}'", bean_id, prop_key, node_name);
                    } else {
                        let all_beans = get_all_bean_instances();
                        let bean_ids: Vec<String> = all_beans.iter().map(|(id, _)| id.clone()).collect();
                        godot_warn!("[GmlScene] auto_bind_data: bean '{}' not found, registered beans: {:?}", bean_id, bean_ids);
                    }
                } else {
                    godot_warn!("[GmlScene] auto_bind_data: invalid bean format '{}', expected 'bean:bean_id:property_key'", data_var_name);
                }
            } else {
                // 原有逻辑：从脚本对象读取变量
                let data = script_obj.get(&StringName::from(data_var_name.as_str()));
                //godot_print!("[GmlScene] auto_bind_data: script_obj.get('{}') type={:?}", data_var_name, data.get_type());
                if data.get_type() == VariantType::ARRAY {
                    //godot_print!("[GmlScene] auto_bind_data: calling update() on node='{}' with {} items", node_name, data.to::<Array<Variant>>().len());
                    let mut node_mut = node.clone();
                    node_mut.call(
                        &StringName::from("update"),
                        &[data.clone(), Variant::from(true)],
                    );
                } else if data.get_type() != VariantType::NIL {
                    godot_warn!("[GmlScene] auto_bind_data: variable '{}' is not Array (type={:?}), skipping", data_var_name, data.get_type());
                } else {
                    godot_warn!("[GmlScene] auto_bind_data: variable '{}' not found in script (type=NIL)", data_var_name);
                }
            }
        }
        // 清理元数据
        let mut node_mut = node.clone();
        node_mut.remove_meta(&StringName::from("__data_var"));
    }

    // 递归处理子节点
    let children = node.get_children();
    for i in 0..children.len() {
        if let Some(child) = children.get(i) {
            if let Ok(child_ctrl) = child.clone().try_cast::<Control>() {
                let child_watches = Self::auto_bind_data_recursive(&child_ctrl, script_obj);
                for j in 0..child_watches.len() {
                    if let Some(item) = child_watches.get(j) {
                        watch_registrations.push(&item);
                    }
                }
            }
        }
    }

    watch_registrations
}
}
