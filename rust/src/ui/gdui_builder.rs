// GdUiBuilder - UI 标记语言构建器
// 暴露给 GDScript 的 API，支持解析类 HTML 标记字符串/文件并生成 Godot Control 节点树
// 用法：
//   var builder = GdUiBuilder.new()
//   var ui = builder.parse_string("<ui><Label text='Hello' /></ui>")
//   add_child(ui)
//   builder.connect_signals(ui, self)  # 连接信号到脚本方法

use godot::prelude::*;
use godot::builtin::{GString, StringName};
use godot::classes::{IRefCounted, Control, FileAccess};

use super::parser::UiParser;
use super::builder::UiBuilder;

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdUiBuilder {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GdUiBuilder {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GdUiBuilder {
    /// 解析标记字符串，返回 Control 节点树
    /// 标记格式参考 docs/类html设计稿.md
    #[func]
    fn parse_string(&self, markup: GString) -> Gd<Control> {
        let input = markup.to_string();
        let mut parser = UiParser::new(&input);

        match parser.parse() {
            Ok(parse_result) => {
                let mut builder = UiBuilder::new();
                match builder.build(&parse_result) {
                    Ok(control) => control,
                    Err(e) => {
                        godot_error!("[GdUiBuilder] Build error: {}", e);
                        Control::new_alloc()
                    }
                }
            }
            Err(e) => {
                godot_error!("[GdUiBuilder] Parse error: {}", e);
                Control::new_alloc()
            }
        }
    }

    /// 解析 .gml 文件，返回 Control 节点树
    #[func]
    fn parse_file(&self, path: GString) -> Gd<Control> {
        let path_str = path.to_string();

        // 使用 Godot FileAccess 读取文件
        let fa = FileAccess::open(&path, godot::classes::file_access::ModeFlags::READ);
        if fa.is_none() {
            godot_error!("[GdUiBuilder] Cannot open file: {}", path_str);
            return Control::new_alloc();
        }

        let fa = unsafe { fa.unwrap_unchecked() };
        let content = fa.get_as_text();

        self.parse_string(content)
    }

    /// 连接 UI 节点树中的信号到目标脚本
    /// 遍历节点树中所有带有 __signal_xxx 元数据的节点，
    /// 将 on_xxx 属性指定的方法名连接为信号
    #[func]
    fn connect_signals(&self, mut root: Gd<Control>, target: Gd<Object>) {
        connect_signals_recursive(&mut root, &target);
    }

    /// 获取解析错误信息（空字符串表示无错误）
    #[func]
    fn validate(&self, markup: GString) -> GString {
        let input = markup.to_string();
        let mut parser = UiParser::new(&input);
        match parser.parse() {
            Ok(_) => GString::new(),
            Err(e) => GString::from(e.to_string().as_str()),
        }
    }
}

/// 递归连接信号
pub fn connect_signals_recursive(node: &mut Gd<Control>, target: &Gd<Object>) {
    // 检查节点是否有信号元数据
    let meta_list = get_signal_meta_list(node);
    for (signal_name, method_name) in meta_list {
        let callable = Callable::from_object_method(target, &StringName::from(method_name.as_str()));
        node.connect(&StringName::from(signal_name.as_str()), &callable);
    }

    // 递归处理子节点
    let children = node.get_children();
    for i in 0..children.len() {
        if let Some(child) = children.get(i) {
            if let Ok(mut control) = child.clone().try_cast::<Control>() {
                connect_signals_recursive(&mut control, target);
            }
        }
    }
}

/// 获取节点上的信号元数据列表
fn get_signal_meta_list(node: &Gd<Control>) -> Vec<(String, String)> {
    let mut result = Vec::new();

    // 获取所有元数据键
    let meta_list = node.get_meta_list();
    for i in 0..meta_list.len() {
        if let Some(key_sn) = meta_list.get(i) {
            let key = key_sn.to_string();
            if key.starts_with("__signal_") {
                let signal_name = key[9..].to_string(); // 去掉 "__signal_" 前缀
                let method_name = node.get_meta(&StringName::from(key.as_str())).to_string();
                result.push((signal_name, method_name));
            }
        }
    }

    result
}
