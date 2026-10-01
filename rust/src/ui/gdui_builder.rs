// GdUiBuilder - UI 标记语言构建器
// 暴露给 GDScript 的 API，支持解析类 HTML 标记字符串/文件并生成 Godot Control 节点树
// 支持主题切换：通过 set_theme() 设置内置主题名称，重新解析时自动应用主题变量
// 用法：
//   var builder = GdUiBuilder.new()
//   builder.set_theme("cartoon")  # 设置主题（可选，默认无主题）
//   var ui = builder.parse_string("<ui theme='cartoon'><Label text='Hello' /></ui>")
//   add_child(ui)
//   builder.connect_signals(ui, self)  # 连接信号到脚本方法

use godot::prelude::*;
use godot::builtin::{GString, StringName, PackedStringArray};
use godot::classes::{IRefCounted, Control, FileAccess, Node, PackedScene};
use godot::global::Error;

use super::parser::UiParser;
use super::builder::UiBuilder;
use super::ui_theme::{ThemeVars, get_builtin_theme, builtin_theme_names};

use std::sync::Mutex;

/// 最近一次 build_scene_* 失败的错误信息（供编辑器自动生成流程展示）
static LAST_BUILD_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_last_error(err: Option<String>) {
    if let Ok(mut slot) = LAST_BUILD_ERROR.lock() {
        *slot = err;
    }
}

pub(crate) fn get_last_error() -> Option<String> {
    LAST_BUILD_ERROR
        .lock()
        .ok()
        .and_then(|s: std::sync::MutexGuard<Option<String>>| s.clone())
}

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdUiBuilder {
    base: Base<RefCounted>,
    /// 当前主题名称（None 表示不使用主题）
    theme_name: Option<String>,
    /// 自定义主题变量（通过 set_theme_var 设置）
    custom_theme_vars: ThemeVars,
}

#[godot_api]
impl IRefCounted for GdUiBuilder {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            base,
            theme_name: None,
            custom_theme_vars: ThemeVars::new(),
        }
    }
}

#[godot_api]
impl GdUiBuilder {
    /// 解析标记字符串，返回 Control 节点树
    /// 标记格式参考 docs/类html设计稿.md
    /// 如果设置了 theme_name，会自动注入内置主题变量
    #[func]
    fn parse_string(&self, markup: GString) -> Gd<Control> {
        match self.build_from_markup(&markup.to_string()) {
            Ok(control) => control,
            Err(e) => {
                godot_error!("[GdUiBuilder] {}", e);
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

    /// 解析标记字符串并打包为 PackedScene（编辑器预览 / 资源化场景使用）
    /// 解析/构建失败时返回 None，错误信息可通过 last_error() 获取
    #[func]
    fn build_scene_string(&self, markup: GString) -> Option<Gd<PackedScene>> {
        match self.build_from_markup(&markup.to_string()) {
            Ok(control) => {
                let scene = pack_control_to_scene(&control);
                set_last_error(None);
                Some(scene)
            }
            Err(e) => {
                set_last_error(Some(e.clone()));
                godot_error!("[GdUiBuilder] Build scene error: {}", e);
                None
            }
        }
    }

    /// 解析 .gml 文件并打包为 PackedScene（编辑器预览 / 资源化场景使用）
    /// 解析/构建失败时返回 None，错误信息可通过 last_error() 获取
    #[func]
    fn build_scene_file(&self, path: GString) -> Option<Gd<PackedScene>> {
        let path_str = path.to_string();
        let fa = FileAccess::open(&path, godot::classes::file_access::ModeFlags::READ);
        if fa.is_none() {
            let msg = format!("无法打开文件: {}", path_str);
            set_last_error(Some(msg.clone()));
            godot_error!("[GdUiBuilder] {}", msg);
            return None;
        }
        let fa = unsafe { fa.unwrap_unchecked() };
        let content = fa.get_as_text();
        self.build_scene_string(content)
    }

    /// 最近一次 build_scene_string/build_scene_file 的错误信息（空串表示无错误）
    #[func]
    fn last_error(&self) -> GString {
        match get_last_error() {
            Some(e) => GString::from(e.as_str()),
            None => GString::new(),
        }
    }

    /// 内部：解析 + 主题注入 + 构建，返回 Control 节点树
    fn build_from_markup(&self, markup: &str) -> Result<Gd<Control>, String> {
        build_markup(markup, self.theme_name.as_deref(), &self.custom_theme_vars)
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

    /// 设置内置主题名称（cartoon）
    /// 设置后，下次 parse_string/parse_file 时自动注入主题变量
    /// GML 中使用 $var_name 引用主题变量
    #[func]
    fn set_theme(&mut self, theme_name: GString) {
        let name = theme_name.to_string();
        if name.is_empty() {
            self.theme_name = None;
        } else if get_builtin_theme(&name).is_some() {
            self.theme_name = Some(name);
        } else {
            godot_warn!("[GdUiBuilder] Unknown theme '{}', available: {:?}", name, builtin_theme_names());
        }
    }

    /// 获取当前主题名称
    #[func]
    fn get_theme(&self) -> GString {
        match &self.theme_name {
            Some(name) => GString::from(name.as_str()),
            None => GString::new(),
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

    /// 设置自定义主题变量（覆盖内置主题同名变量）
    /// key: 变量名（不含 $ 前缀），value: 变量值（如 "#1a1a3e"）
    #[func]
    fn set_theme_var(&mut self, key: GString, value: GString) {
        self.custom_theme_vars.insert(key.to_string(), value.to_string());
    }

    /// 清除所有自定义主题变量
    #[func]
    fn clear_custom_theme_vars(&mut self) {
        self.custom_theme_vars.clear();
    }
}

/// 解析 + 主题注入 + 构建的公共实现
fn build_markup(
    markup: &str,
    theme_name: Option<&str>,
    custom_theme_vars: &ThemeVars,
) -> Result<Gd<Control>, String> {
    let mut parser = UiParser::new(markup);
    let parse_result = parser
        .parse()
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut builder = UiBuilder::new();

    // 注入主题变量：先设置内置主题，再设置自定义变量
    let mut theme_vars = ThemeVars::new();
    if let Some(name) = theme_name {
        if let Some(builtin) = get_builtin_theme(name) {
            theme_vars.extend(builtin);
        }
    }
    theme_vars.extend(custom_theme_vars.clone());
    if !theme_vars.is_empty() {
        builder.set_theme_vars(theme_vars);
    }

    builder.build(&parse_result)
}

/// 供 GdGmlLoader 调用：读取 .gml 文件并构建 PackedScene
/// 与 GdUiBuilder.new() 的行为一致（不注入运行时主题，与编辑器预览保持一致）
/// 失败时返回 None，错误信息可通过 get_last_error() 获取
pub(crate) fn build_scene_from_file_for_loader(path: &GString) -> Option<Gd<PackedScene>> {
    let fa = FileAccess::open(path, godot::classes::file_access::ModeFlags::READ);
    if fa.is_none() {
        let msg = format!("无法打开文件: {}", path);
        set_last_error(Some(msg.clone()));
        godot_error!("[GdUiBuilder] {}", msg);
        return None;
    }
    let fa = unsafe { fa.unwrap_unchecked() };
    let content = fa.get_as_text();

    match build_markup(&content.to_string(), None, &ThemeVars::new()) {
        Ok(control) => {
            let scene = pack_control_to_scene(&control);
            set_last_error(None);
            Some(scene)
        }
        Err(e) => {
            set_last_error(Some(e.clone()));
            godot_error!("[GdUiBuilder] Build scene error: {}", e);
            None
        }
    }
}

/// 将构建好的 Control 树打包为 PackedScene
/// pack 只序列化 owner == 根 的节点，因此递归为所有后代设置 owner，
/// 包括列表（UIHList/UIVList/UIGrid）的 slot 模板——否则打包后列表为空
pub(crate) fn pack_control_to_scene(root: &Gd<Control>) -> Gd<PackedScene> {
    let root_node: Gd<Node> = root.clone().upcast();
    assign_owners(&root_node, &root_node);

    let mut scene = PackedScene::new_gd();
    let err = scene.pack(&root_node);
    if err != Error::OK {
        godot_error!("[GdUiBuilder] PackedScene.pack 失败: {:?}", err);
    }
    // pack 只序列化属性快照；构建产生的临时节点树是手动内存，打包后释放防泄漏
    root_node.free();
    scene
}

/// 递归为所有后代节点设置 owner
fn assign_owners(node: &Gd<Node>, owner: &Gd<Node>) {
    for i in 0..node.get_child_count() {
        if let Some(mut child) = node.get_child(i) {
            child.set_owner(owner);
            assign_owners(&child, owner);
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
