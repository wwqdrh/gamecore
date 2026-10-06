// GdUiBuilder - UI 标记语言构建器
// 暴露给 GDScript 的 API，支持解析类 HTML 标记字符串/文件并生成 Godot Control 节点树
// 主题变量：gml <theme> 块 / set_theme_var() 注入，样式值中 $var 引用
// 用法：
//   var builder = GdUiBuilder.new()
//   var ui = builder.parse_string("<ui><Label text='Hello' /></ui>")
//   add_child(ui)
//   builder.connect_signals(ui, self)  # 连接信号到脚本方法

use godot::prelude::*;
use godot::builtin::{GString, StringName, PackedStringArray};
use godot::classes::{IRefCounted, Control, Engine, FileAccess, Node, PackedScene};
use godot::global::Error;

use super::parser::UiParser;
use super::builder::UiBuilder;
use super::ui_theme::ThemeVars;

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
    /// 自定义主题变量（通过 set_theme_var 设置）
    custom_theme_vars: ThemeVars,
    /// parse_string 的相对路径基准（<Gml src> / <ui script> 相对路径解析）
    /// parse_file 自动按文件目录推导，不读本字段
    base_dir: Option<String>,
}

#[godot_api]
impl IRefCounted for GdUiBuilder {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            base,
            custom_theme_vars: ThemeVars::new(),
            base_dir: None,
        }
    }
}

#[godot_api]
impl GdUiBuilder {
    /// 解析标记字符串，返回 Control 节点树
    /// 标记格式参考 docs/类html设计稿.md
    #[func]
    fn parse_string(&self, markup: GString) -> Gd<Control> {
        match self.build_from_markup(&markup.to_string()) {
            Ok(control) => {
                set_last_error(None);
                control
            }
            Err(e) => {
                set_last_error(Some(e.clone()));
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

        // 文件加载携带目录上下文，使 <Gml src="相对路径"> 可以解析
        match build_markup(
            &content.to_string(),
            &self.custom_theme_vars,
            super::builder::parent_dir_of(&path_str).as_deref(),
        ) {
            Ok(control) => {
                set_last_error(None);
                control
            }
            Err(e) => {
                set_last_error(Some(e.clone()));
                godot_error!("[GdUiBuilder] {}", e);
                Control::new_alloc()
            }
        }
    }

    /// 解析标记字符串并打包为 PackedScene（编辑器预览 / 资源化场景使用）
    /// 解析/构建失败时返回 None，错误信息可通过 last_error() 获取
    #[func]
    fn build_scene_string(&self, markup: GString) -> Option<Gd<PackedScene>> {
        self.build_scene_markup(markup, None)
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
        self.build_scene_markup(content, super::builder::parent_dir_of(&path_str))
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
        build_markup(
            markup,
            &self.custom_theme_vars,
            self.base_dir.as_deref(),
        )
    }

    /// 内部：带目录上下文构建并打包为 PackedScene（<Gml> 引用需要 base_dir）
    fn build_scene_markup(&self, markup: GString, base_dir: Option<String>) -> Option<Gd<PackedScene>> {
        match build_markup(
            &markup.to_string(),
            &self.custom_theme_vars,
            base_dir.as_deref(),
        ) {
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

    /// 设置 parse_string 的相对路径基准目录（如 "user://ui_test"）
    /// <Gml src="相对路径"> 与 <ui script="相对路径"> 以该目录解析；
    /// parse_file 无需设置（自动按文件所在目录推导）。传空字符串清除。
    #[func]
    fn set_base_dir(&mut self, dir: GString) {
        let d = dir.to_string();
        self.base_dir = if d.is_empty() { None } else { Some(d) };
    }

    /// 设置自定义主题变量
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

/// 解析 + 主题变量注入 + 构建的公共实现
/// base_dir：gml 文件所在目录（<Gml src="相对路径"> 的解析基准），字符串构建时为 None
fn build_markup(
    markup: &str,
    custom_theme_vars: &ThemeVars,
    base_dir: Option<&str>,
) -> Result<Gd<Control>, String> {
    let mut parser = UiParser::new(markup);
    let parse_result = parser
        .parse()
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut builder = UiBuilder::new();
    builder.set_base_dir(base_dir.map(|s| s.to_string()));

    // 主题变量：set_theme_var 注入的自定义变量（gml <theme> 块变量在 build 内合并）
    let mut theme_vars = ThemeVars::new();
    theme_vars.extend(custom_theme_vars.clone());
    if !theme_vars.is_empty() {
        builder.set_theme_vars(theme_vars);
    }

    builder.build(&parse_result)
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
    // 列表控件记录信号目标：条目由 slot.duplicate() 创建，外部脚本连接不会随
    // duplicate 复制，update() 重建条目后由 bind_events 依据该 meta 自动重连
    // 条目内 @pressed/on_pressed 声明的信号（见 ui_list_helper::auto_bind_item_signals）
    let class_name = node.get_class().to_string();
    if matches!(class_name.as_str(), "GdUIVList" | "GdUIHList" | "GdUIGrid") {
        node.set_meta(
            &StringName::from("__gml_signal_target"),
            &target.clone().to_variant(),
        );
    }

    // 连接本节点声明的全部信号绑定（@pressed / on_pressed，含参数补绑）
    super::ui_list_helper::connect_node_signal_meta(node, target);

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
pub(crate) fn get_signal_meta_list(node: &Gd<Control>) -> Vec<(String, String)> {
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

/// gml 树挂树自动连接信号（无包装层自举）：
/// `<ui script>` 声明的树根挂有 __gml_root 标记，节点挂树（SceneTree.node_added）
/// 时框架自动对整树执行 connect_signals_recursive（target = 树根自身）——
/// 编辑器生成的 .gml.tscn 直开运行即完整可用，无需 GdGmlScene 壳场景或手动
/// connect_signals。回调目标仍按"就近解析"优先各 <ui script> 脚本节点。
/// 由 GDCORE 单例驱动；编辑器模式下跳过（连接属运行时行为）。
pub fn auto_connect_gml_tree(node: Gd<Node>) {
    if Engine::singleton().is_editor_hint() {
        return;
    }
    // 向上找最外层 gml 树根（子文件 <ui script> 根也带标记，但整树只需连接一次；
    // 先触发的连接会打 __gml_signals_connected，后续节点触发时直接跳过）
    let mut root: Option<Gd<Node>> = None;
    let mut cur = Some(node);
    while let Some(n) = cur {
        let parent = n.get_parent();
        if n.has_meta(&StringName::from("__gml_root")) {
            root = Some(n.clone());
        }
        cur = parent;
    }
    let Some(mut root) = root else { return };
    connect_gml_tree_root(&mut root);
}

/// 对一棵 gml 树根执行信号自动连接（幂等：__gml_signals_connected 防重）
fn connect_gml_tree_root(root: &mut Gd<Node>) {
    let connected_key = StringName::from("__gml_signals_connected");
    if root.has_meta(&connected_key) {
        return;
    }
    let Ok(mut ctrl) = root.clone().try_cast::<Control>() else {
        return;
    };
    root.set_meta(&connected_key, &true.to_variant());
    // 统一 UI 管理层：先注册树上全部 ui_id 组件（先于信号连接，
    // 保证同树内后连接的按钮按 id 能查到目标）
    register_ui_ids(root);
    let target = root.clone().upcast::<Object>();
    connect_signals_recursive(&mut ctrl, &target);
}

/// 递归注册树上全部带 __ui_id meta 的节点到统一 UI 管理层
fn register_ui_ids(node: &Gd<Node>) {
    if node.has_meta(&StringName::from("__ui_id")) {
        let id = node.get_meta(&StringName::from("__ui_id")).to_string();
        crate::ui::ui_manager::register_ui(&id, node);
    }
    let children = node.get_children();
    for i in 0..children.len() {
        if let Some(child) = children.get(i) {
            register_ui_ids(&child);
        }
    }
}

/// 补扫已在树上的节点树（自举钩子在首帧才连上 node_added，而 F6/主场景
/// 在首帧之前就已挂树——从树根向下扫描，遇到 __gml_root 即整树连接并跳过
/// 其子树）。由 GDCORE 钩子连接成功后立即调用一次。
pub fn scan_and_connect_gml_trees(node: &Gd<Node>) {
    if Engine::singleton().is_editor_hint() {
        return;
    }
    if node.has_meta(&StringName::from("__gml_root")) {
        // 最先遇到的就是最外层 gml 根（从树顶向下），整树连接后子树无需再扫
        let mut root = node.clone();
        connect_gml_tree_root(&mut root);
        return;
    }
    let children = node.get_children();
    for i in 0..children.len() {
        if let Some(child) = children.get(i) {
            scan_and_connect_gml_trees(&child);
        }
    }
}
