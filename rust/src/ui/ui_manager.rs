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
}
