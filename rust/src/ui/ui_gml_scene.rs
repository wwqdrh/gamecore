// GdGmlScene - GML 文件加载节点
// 继承 Control，设置 file 属性即可加载 .gml 文件并显示为 Control 节点树
// 用法：
//   在场景中添加 GmlScene 节点，设置 gml_file 属性为 .gml 文件路径
//   或在 GDScript 中：var gml = GmlScene.new(); gml.gml_file = "res://ui/scene.gml"; add_child(gml)
//   信号连接：gml.connect_signals(self)

use godot::prelude::*;
use godot::builtin::{GString, StringName};
use godot::classes::{
    IControl, Control, FileAccess, Node,
};
use godot::obj::WithBaseField;

use super::parser::UiParser;
use super::builder::UiBuilder;
use super::gdui_builder::connect_signals_recursive;

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdGmlScene {
    base: Base<Control>,

    /// GML 文件路径（设置后自动加载）
    #[export]
    gml_file: GString,

    /// 是否在 ready 时自动连接信号到父节点脚本
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

    fn ready(&mut self) {
        if !self.gml_file.is_empty() {
            self.load_gml();
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

                        // 自动连接信号到父节点脚本
                        self.content_root = Some(control);
                        self.loaded = true;

                        if self.auto_connect {
                            if let Some(parent) = self.base().get_parent() {
                                let obj = parent.clone().upcast::<Object>();
                                self.connect_signals(obj);
                            }
                        }

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
}
