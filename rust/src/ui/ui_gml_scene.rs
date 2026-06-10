// GdGmlScene - GML 文件加载节点
// 继承 Control，设置 file 属性即可加载 .gml 文件并显示为 Control 节点树
// 用法：
//   在场景中添加 GmlScene 节点，设置 gml_file 属性为 .gml 文件路径
//   或在 GDScript 中：var gml = GmlScene.new(); gml.gml_file = "res://ui/scene.gml"; add_child(gml)
//   信号连接：auto_connect=true 时自动连接到 GdGmlScene 自身脚本
//   可创建继承 GdGmlScene 的 GDScript，在其中定义回调方法

use godot::prelude::*;
use godot::builtin::{GString, StringName, Variant, VariantType, Array};
use godot::classes::{
    IControl, Control, FileAccess, Node,
};
use godot::classes::control::LayoutPreset;
use godot::classes::notify::ControlNotification;
use godot::obj::WithBaseField;

use super::parser::UiParser;
use super::builder::UiBuilder;
use super::gdui_builder::connect_signals_recursive;
use crate::state::bean::{get_bean_by_id, get_all_bean_instances};

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdGmlScene {
    base: Base<Control>,

    /// GML 文件路径（如 res://ui/scene.gml，设置后自动加载）
    #[export]
    gml_file: GString,

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
            gml_file: GString::new(),
            auto_connect: true,
            content_root: None,
            loaded: false,
        }
    }

    fn on_notification(&mut self, what: ControlNotification) {
        if what == ControlNotification::READY {
            godot_print!("[GmlScene] READY notification, gml_file='{}'", self.gml_file);
            if !self.gml_file.is_empty() {
                // 延迟一帧加载，确保 GDScript 脚本变量已完全初始化
                // gdext 的 Object::get() 在 READY 通知时无法读取 GDScript var 变量
                godot_print!("[GmlScene] scheduling deferred load_gml");
                self.base_mut().call_deferred(
                    &StringName::from("load_gml"),
                    &[],
                );
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
    fn load_gml(&mut self) {
        godot_print!("[GmlScene] load_gml() called, gml_file='{}'", self.gml_file);
        if self.gml_file.is_empty() {
            godot_error!("[GmlScene] gml_file is empty");
            return;
        }

        let path = self.gml_file.clone();
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

        self.parse_and_build(&content.to_string());
    }

    /// 从字符串加载 GML 内容
    #[func]
    fn load_from_string(&mut self, gml_content: GString) {
        self.parse_and_build(&gml_content.to_string());
    }

    /// 重新加载 GML 文件
    #[func]
    fn reload(&mut self) {
        self.clear_content();
        self.load_gml();
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
    #[func]
    fn find_node(&self, name: GString) -> Option<Gd<Control>> {
        if let Some(ref root) = self.content_root {
            let found = root.find_child(&name);
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

    /// 延迟重新应用所有带 __anchor meta 的节点的 anchor preset
    /// 节点加入场景树后父容器大小已计算完成，此时 anchor offsets 才能正确设置
    #[func]
    fn refresh_anchors(&mut self) {
        if let Some(ref root) = self.content_root {
            Self::refresh_anchors_recursive(root);
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

    /// GdBean 响应式回调：属性变更时自动更新对应节点
    /// 通过 bean.watch() 注册，参数为 (node_name, new_value)
    #[func]
    fn on_bean_data_changed(&mut self, node_name: GString, data: Variant) {
        if data.get_type() == VariantType::ARRAY {
            if let Some(node) = self.find_node(node_name.clone()) {
                godot_print!("[GmlScene] on_bean_data_changed: updating node='{}' with {} items", node_name, data.to::<Array<Variant>>().len());
                let mut node_mut = node;
                node_mut.call(
                    &StringName::from("update"),
                    &[data, Variant::from(true)],
                );
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
                match builder.build(&parse_result) {
                    Ok(mut control) => {
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
    fn auto_bind_data(&mut self, root: &Gd<Control>) {
        let all_beans = get_all_bean_instances();
        let bean_ids: Vec<String> = all_beans.iter().map(|(id, _)| id.clone()).collect();
        godot_print!("[GmlScene] auto_bind_data: registered beans: {:?}", bean_ids);
        let self_node = self.base().clone();
        Self::auto_bind_data_recursive(root, &self_node.upcast::<Object>());
    }

    fn auto_bind_data_recursive(node: &Gd<Control>, script_obj: &Gd<Object>) {
        let node_name = node.get_name().to_string();
        // 检查当前节点是否有 __data_var 元数据
        if node.has_meta(&StringName::from("__data_var")) {
            let data_var_name = node.get_meta(&StringName::from("__data_var")).to_string();
            godot_print!("[GmlScene] auto_bind_data: node='{}' has __data_var='{}'", node_name, data_var_name);
            if !data_var_name.is_empty() {
                // 检查是否为 bean:bean_id:property_key 格式
                if data_var_name.starts_with("bean:") {
                    let parts: Vec<&str> = data_var_name[5..].splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let bean_id = parts[0];
                        let prop_key = parts[1];
                        if let Some(mut bean) = get_bean_by_id(bean_id) {
                            let data = bean.call("get_value_by_key", &[GString::from(prop_key).to_variant()]);
                            godot_print!("[GmlScene] auto_bind_data: bean '{}' key '{}' type={:?}", bean_id, prop_key, data.get_type());
                            if data.get_type() == VariantType::ARRAY {
                                let arr = data.to::<Array<Variant>>();
                                godot_print!("[GmlScene] auto_bind_data: calling update() on node='{}' with {} items", node_name, arr.len());
                                // 日志：打印前3个元素的内容
                                for i in 0..arr.len().min(3) {
                                    if let Some(item) = arr.get(i) {
                                        godot_print!("[GmlScene] auto_bind_data: item[{}] type={:?} value={}", i, item.get_type(), item);
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
                            // 注册响应式回调：bean 属性变更时自动更新节点
                            let self_obj = script_obj.clone();
                            let node_name_gstr = GString::from(&node_name);
                            let callable = self_obj.callable("on_bean_data_changed").bind(&[node_name_gstr.to_variant()]);
                            bean.call("watch", &[GString::from(prop_key).to_variant(), callable.to_variant()]);
                            godot_print!("[GmlScene] auto_bind_data: registered watch on bean '{}' key '{}' for node '{}'", bean_id, prop_key, node_name);
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
                    godot_print!("[GmlScene] auto_bind_data: script_obj.get('{}') type={:?}", data_var_name, data.get_type());
                    if data.get_type() == VariantType::ARRAY {
                        godot_print!("[GmlScene] auto_bind_data: calling update() on node='{}' with {} items", node_name, data.to::<Array<Variant>>().len());
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
                    Self::auto_bind_data_recursive(&child_ctrl, script_obj);
                }
            }
        }
    }
}
