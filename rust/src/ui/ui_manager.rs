// 统一 UI 管理层 —— 跨文件夹/跨面板组件注册与查找
//
// 解决的问题：组件（面板/抽屉/弹窗）分属不同目录，彼此之间不 import、
// 不 find_child 互相翻树，跨组件调用全部经由本管理层按 ui_id 查找：
//
//   GML 声明（组件在自己文件里声明唯一身份，零外部引用）：
//     <Drawer ui_id="TaskDrawer" ...>...</Drawer>
//
//   GD 侧跨组件调用（按需）：
//     var drawer = GdUIManager.find_ui("TaskDrawer")
//     if drawer: drawer.open()
//
//   GML 内部动作（运行期按下时按 id 延迟解析，无挂载顺序要求）：
//     <Button @pressed="show:TaskDrawer" />
//
// 注册时机：gml 树挂树自举（connect_gml_tree_root）时，带 __ui_id meta 的
// 节点自动注册；也可在 GD/Rust 侧手动 register_ui。
// 生命周期：注册表只存 InstanceId（不持有引用、不阻止释放），
// find_ui 时校验实例有效性，失效条目自动清理。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use godot::prelude::*;

// ---------------------------------------------------------------------------
// Rust 侧注册表
// ---------------------------------------------------------------------------

/// 全局组件注册表：ui_id -> InstanceId
/// （只存实例 id 不持有 Gd 引用：组件随场景释放不会泄漏，查找时校验有效性）
static UI_REGISTRY: OnceLock<Mutex<HashMap<String, InstanceId>>> = OnceLock::new();

/// 拿到注册表 map（首次访问时初始化）
fn lock_map() -> std::sync::MutexGuard<'static, HashMap<String, InstanceId>> {
	UI_REGISTRY
		.get_or_init(|| Mutex::new(HashMap::new()))
		.lock()
		.expect("[UIManager] 注册表锁中毒")
}

/// 注册/覆盖组件（id 重复时后者生效）
pub fn register_ui(id: &str, node: &Gd<Node>) {
	lock_map().insert(id.to_string(), node.instance_id());
}

/// 注销组件（组件释放也会被 find_ui 懒清理，本方法供显式管理用）
pub fn unregister_ui(id: &str) {
	lock_map().remove(id);
}

/// 按 ui_id 查找组件：未注册返回 None；已注册但实例已释放则清理条目并返回 None
pub fn find_ui(id: &str) -> Option<Gd<Node>> {
	let mut map = lock_map();
	let sid = map.get(id).copied()?;
	match Gd::<Node>::try_from_instance_id(sid) {
		Ok(node) => Some(node),
		Err(_) => {
			map.remove(id);
			None
		}
	}
}

/// 组件是否已注册且存活
pub fn has_ui(id: &str) -> bool {
	find_ui(id).is_some()
}

// ---------------------------------------------------------------------------
// GML name → 节点 注册表（面板脚本免写长路径，按 name 直取）
// ---------------------------------------------------------------------------

/// 全局 name 映射表：gml 树根 InstanceId -> (节点 name -> InstanceId)
/// （键按树根分作用域：子 <Gml> 文件各自成表，同名互不串扰；
/// 只存 InstanceId 不持有 Gd 引用，随树释放不泄漏，命中时校验有效性）
static GML_NAME_REGISTRY: OnceLock<Mutex<HashMap<i64, HashMap<String, InstanceId>>>> =
	OnceLock::new();

fn lock_name_map()
-> std::sync::MutexGuard<'static, HashMap<i64, HashMap<String, InstanceId>>> {
	GML_NAME_REGISTRY
		.get_or_init(|| Mutex::new(HashMap::new()))
		.lock()
		.expect("[UIManager] name 注册表锁中毒")
}

/// 就近定位 from 所属的 gml 树根（子 <Gml> 文件根也带 __gml_root，
/// 就近命中保证各文件独立作用域）
pub(crate) fn nearest_gml_root(from: &Gd<Node>) -> Option<Gd<Node>> {
	let mut cur = Some(from.clone());
	while let Some(n) = cur {
		if n.has_meta(&StringName::from("__gml_root")) {
			return Some(n);
		}
		cur = n.get_parent();
	}
	None
}

/// 全树 DFS 收集 name -> InstanceId（首见优先，同 find_child 语义；
/// 嵌套子 gml 树不入外层表；@Class@id 自动名跳过）
fn collect_name_ids(node: &Gd<Node>, is_root: bool, out: &mut HashMap<String, InstanceId>) {
	if !is_root && node.has_meta(&StringName::from("__gml_root")) {
		return;
	}
	let node_name = node.get_name().to_string();
	if !node_name.is_empty() && !node_name.contains('@') && !out.contains_key(&node_name) {
		out.insert(node_name, node.instance_id());
	}
	for i in 0..node.get_child_count() {
		if let Some(child) = node.get_child(i) {
			collect_name_ids(&child, false, out);
		}
	}
}

/// 在 from 所属 gml 树内按 name 查找节点（内部实现）。
/// 映射由 rust 解析维护：首查全树建表，此后 O(1) 命中；
/// 悬垂/未命中/树根已释放时自动重建或清理。
pub fn get_gml_node(from: &Gd<Node>, name: &str) -> Option<Gd<Node>> {
	let root = nearest_gml_root(from)?;
	let root_id = root.instance_id().to_i64();

	// 缓存命中（存活校验）
	{
		let reg = lock_name_map();
		if let Some(inner) = reg.get(&root_id) {
			if let Some(&sid) = inner.get(name) {
				if let Ok(node) = Gd::<Node>::try_from_instance_id(sid) {
					return Some(node);
				}
			}
			// 未命中或条目悬垂 → 落到重建
		}
	}

	// 重建：树根已释放 → 清理注册项；否则全树重扫
	match Gd::<Node>::try_from_instance_id(root.instance_id()) {
		Ok(_) => {}
		Err(_) => {
			lock_name_map().remove(&root_id);
			return None;
		}
	}
	let mut inner: HashMap<String, InstanceId> = HashMap::new();
	collect_name_ids(&root, true, &mut inner);
	let hit = inner.get(name).copied();
	lock_name_map().insert(root_id, inner);
	let sid = hit?;
	Gd::<Node>::try_from_instance_id(sid).ok()
}

// ---------------------------------------------------------------------------
// GD 侧 API（静态方法，GDScript 直接 GdUIManager.find_ui("TaskDrawer")）
// ---------------------------------------------------------------------------

/// 统一 UI 管理器：组件注册 / 跨组件查找（GD 侧入口）
#[derive(GodotClass)]
#[class(base = Object, init)]
pub struct GdUIManager {
	base: Base<Object>,
}

#[godot_api]
impl GdUIManager {
	/// 按 ui_id 查找已注册组件（跨文件夹/跨面板），未注册或已释放返回 null。
	/// 返回后按需调用组件方法（如 drawer.open()）——GD 动态调用无需转型
	#[func]
	pub fn find_ui(id: GString) -> Option<Gd<Object>> {
		find_ui(&id.to_string()).map(|n| n.upcast())
	}

	/// 手动注册组件（ui_id 属性声明的节点挂树时自动注册，本方法供动态创建的
	/// 组件使用；id 重复时覆盖）
	#[func]
	pub fn register_ui(id: GString, node: Gd<Object>) {
		match node.try_cast::<Node>() {
			Ok(n) => register_ui(&id.to_string(), &n),
			Err(_) => godot_error!("[UIManager] register_ui: 节点不是 Node 类型"),
		}
	}

	/// 显式注销（组件释放会被自动懒清理，一般无需手动调用）
	#[func]
	pub fn unregister_ui(id: GString) {
		unregister_ui(&id.to_string());
	}

	/// 组件是否已注册且存活
	#[func]
	pub fn has_ui(id: GString) -> bool {
		has_ui(&id.to_string())
	}

	/// 列出全部已注册的 ui_id（调试用）
	#[func]
	pub fn get_ui_ids() -> PackedStringArray {
		let map = lock_map();
		map.keys()
			.map(|k| GString::from(k.as_str()))
			.collect::<PackedStringArray>()
	}

	/// 在 GML 树内按 name 查找节点（面板脚本免写长路径）。
	/// from：目标树内任意节点（通常传 self）；name→节点映射由 rust 解析维护，
	/// 首次查找建表，之后 O(1) 命中；子 <Gml> 文件各自成作用域，同名互不串扰。
	/// 未找到返回 null。用法：`var menu = GdUIManager.get_gml_node(self, "DocMenu")`
	#[func]
	pub fn get_gml_node(from: Gd<Object>, name: GString) -> Option<Gd<Object>> {
		let Ok(node) = from.try_cast::<Node>() else {
			return None;
		};
		get_gml_node(&node, &name.to_string()).map(|n| n.upcast())
	}
}
