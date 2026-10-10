// GDCore - 全局核心单例
// 继承 Object（手动内存），作为 Engine singleton 注册为 "GDCORE"
// 支持存档 ID 管理，根据 save_id 切换不同的存档文件
// 存档文件路径：user://coredata_{id}.data（id 为空时为 user://coredata.data）
// 每存档累计游玩时长：get_play_time/flush_play_time，持久化在
// GdCoreData 的 playtime;total（真实秒，跨开关应用累积）

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use godot::prelude::*;
use godot::classes::{Engine, IObject, Object, Node, SceneTree, Time};
use godot::builtin::{StringName, VarDictionary};

use super::coredata::GdCoreData;
use super::bean::GdBean;

/// 每存档游玩时长的持久化位置（GdCoreData 的 scope/field）
const PLAYTIME_SCOPE: &str = "playtime";
const PLAYTIME_FIELD: &str = "total";
/// 自动落盘间隔（毫秒）：崩退最多丢这个窗口内的时长
const PLAYTIME_AUTOFLUSH_MS: u64 = 30_000;

#[derive(GodotClass)]
#[class(base = Object)]
pub struct GDCore {
    save_id: GString,
    core_data_cache: HashMap<String, Gd<GdCoreData>>,
    /// 全局节点映射 (alias -> Node)
    global_nodes: VarDictionary,
    /// 当前存档本次会话的开始时刻（get_ticks_msec）——会话时长实时累加在
    /// 持久化值之上，落盘（flush/切档/退出）时并入并重置起点
    session_start_ms: i64,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GDCore {
    fn init(base: Base<Object>) -> Self {
        let mut core_data_cache = HashMap::new();
        let default_data = GdCoreData::build(
            GString::from("user://coredata.data"),
            GString::from("{}"),
            false,
            GString::from("init"),
        );
        core_data_cache.insert(String::new(), default_data);

        Self {
            save_id: GString::new(),
            core_data_cache,
            global_nodes: VarDictionary::new(),
            session_start_ms: Time::singleton().get_ticks_msec() as i64,
            base,
        }
    }
}

#[godot_api]
impl GDCore {
    #[func]
    fn get_root_data(&self) -> Variant {
        let id = self.save_id.to_string();
        if let Some(ref data) = self.core_data_cache.get(&id) {
            data.to_variant()
        } else {
            Variant::nil()
        }
    }

    #[func]
    fn get_save_id(&self) -> GString {
        self.save_id.clone()
    }

    #[func]
    fn set_save_id(&mut self, id: GString) {
        let id_str = id.to_string();
        if self.save_id.to_string() == id_str {
            return;
        }

        // 会话时长属于旧存档：切档前先落盘（并入旧档累计值）
        self.flush_play_time();

        if !self.core_data_cache.contains_key(&id_str) {
            let filename = if id_str.is_empty() {
                GString::from("user://coredata.data")
            } else {
                GString::from(&format!("user://coredata_{}.data", id_str))
            };
            let new_data = GdCoreData::build(
                filename,
                GString::from("{}"),
                false,
                GString::from("init"),
            );
            self.core_data_cache.insert(id_str.clone(), new_data);
        }

        self.save_id = id;

        let new_core = self.core_data_cache.get(&id_str).cloned();
        if let Some(core) = new_core {
            Self::notify_beans_switch_core(&core);
        }
    }

    /// 注册全局对象（支持 Node 和 RefCounted）
    #[func]
    fn add_global_node(&mut self, alias: GString, obj: Variant) {
        let key = alias.to_variant();
        if self.global_nodes.contains_key(&key) {
            godot_warn!("GDCORE: add_global_node alias '{}' already exists", alias);
            return;
        }
        self.global_nodes.set(&key, &obj);
    }

    /// 获取全局节点
    #[func]
    fn get_global_node(&self, alias: GString) -> Variant {
        let key = alias.to_variant();
        self.global_nodes.get_or_nil(&key)
    }

    /// 移除全局节点
    #[func]
    fn remove_global_node(&mut self, alias: GString) {
        let key = alias.to_variant();
        self.global_nodes.erase(&key);
    }

    /// 当前存档的累计游玩时长（真实秒）= 历史落盘值 + 本次会话已进行时长。
    /// 跨开关应用累积：每次落盘（flush/切档/退出/每 30 秒自动）都会把会话
    /// 时长并入存档文件的 playtime;total，重新打开接着累计。
    #[func]
    pub fn get_play_time(&self) -> f64 {
        let id = self.save_id.to_string();
        let Some(data) = self.core_data_cache.get(&id) else {
            return 0.0;
        };
        stored_playtime(data) + self.session_secs()
    }

    /// 把当前会话时长并入当前存档的累计值并立即写盘（重置会话起点）。
    /// 切换存档 / 进程退出 / 每 30 秒会自动调用，业务一般无需手动触发。
    #[func]
    fn flush_play_time(&mut self) {
        let id = self.save_id.to_string();
        let Some(mut data) = self.core_data_cache.get(&id).cloned() else {
            return;
        };
        let now = Time::singleton().get_ticks_msec() as i64;
        let total = stored_playtime(&data) + self.session_secs();
        // action "~" = 强制覆盖写（GJson 默认动作只写不存在的路径）
        data.bind_mut().update(
            GString::from(PLAYTIME_FIELD),
            GString::from("~"),
            total.to_variant(),
            GString::from(PLAYTIME_SCOPE),
        );
        self.session_start_ms = now;
    }
}

impl GDCore {
    /// 本次会话已进行的秒数（自上次落盘起算）
    fn session_secs(&self) -> f64 {
        let now = Time::singleton().get_ticks_msec() as i64;
        ((now - self.session_start_ms).max(0)) as f64 / 1000.0
    }

    fn notify_beans_switch_core(new_core: &Gd<GdCoreData>) {
        let bean_ids: Vec<(String, i64)> = {
            super::bean::get_all_bean_instances()
        };
        for (_bean_id, instance_id) in bean_ids {
            if let Ok(mut gd) = Gd::<GdBean>::try_from_instance_id(InstanceId::from_i64(instance_id)) {
                gd.bind_mut().do_switch_core(new_core.clone());
            }
        }
    }
}

/// 读取存档数据中已落盘的累计游玩秒数
fn stored_playtime(data: &Gd<GdCoreData>) -> f64 {
    let v = data.bind().value(
        GString::from(PLAYTIME_FIELD),
        0.0.to_variant(),
        GString::from(PLAYTIME_SCOPE),
    );
    v.try_to::<f64>().unwrap_or(0.0)
}

/// 自动落盘节拍（on_main_loop_frame 每帧调用，内部按间隔过滤）
static LAST_AUTOFLUSH_MS: AtomicU64 = AtomicU64::new(0);

/// 每隔 PLAYTIME_AUTOFLUSH_MS 把当前会话时长并入当前存档（崩退保护）。
/// 由 lib.rs 首帧钩子每帧驱动，命中间隔才实际写盘。
pub fn playtime_autoflush_tick() {
    let now = Time::singleton().get_ticks_msec() as u64;
    let last = LAST_AUTOFLUSH_MS.load(Ordering::Relaxed);
    if now.saturating_sub(last) < PLAYTIME_AUTOFLUSH_MS {
        return;
    }
    LAST_AUTOFLUSH_MS.store(now, Ordering::Relaxed);
    if let Some(mut core) = gdcore_singleton() {
        core.bind_mut().flush_play_time();
    }
}

/// 取 GDCORE 单例（未注册/已注销返回 None）
fn gdcore_singleton() -> Option<Gd<GDCore>> {
    let singleton = Engine::singleton().get_singleton(&StringName::from("GDCORE"))?;
    singleton.try_cast::<GDCore>().ok()
}

pub fn register_gdcore_singleton() {
    let instance = Gd::<GDCore>::from_init_fn(|base| GDCore::init(base));
    let name = StringName::from("GDCORE");
    Engine::singleton().register_singleton(&name, &instance);
    std::mem::forget(instance);
    LAST_AUTOFLUSH_MS.store(Time::singleton().get_ticks_msec() as u64, Ordering::Relaxed);
}

/// gml 自举钩子是否已连接（on_main_loop_frame 每帧探测，连接成功即停止）
static GML_HOOK_CONNECTED: AtomicBool = AtomicBool::new(false);

/// 连接 gml 树挂树自动连接钩子（幂等）：监听 SceneTree.node_added，
/// 带标记的 gml 树根挂树后自动连接全部信号绑定（@pressed/@s_click_item 等）——
/// 编辑器生成的 .gml.tscn 直开运行即完整可用（见 ui::gdui_builder::auto_connect_gml_tree）。
/// 由 on_main_loop_frame 首帧调用（Scene stage init 时 main loop 尚未创建）
pub fn connect_gml_auto_connect_hook() {
    if GML_HOOK_CONNECTED.load(Ordering::Relaxed) {
        return;
    }
    let Some(main_loop) = Engine::singleton().get_main_loop() else {
        return;
    };
    if let Ok(mut tree) = main_loop.try_cast::<SceneTree>() {
        tree.connect(
            &StringName::from("node_added"),
            &Callable::from_fn("gml_auto_connect", move |args: &[&Variant]| {
                if let Some(v) = args.first() {
                    if let Ok(n) = v.try_to::<Gd<Node>>() {
                        crate::ui::gdui_builder::auto_connect_gml_tree(n);
                    }
                }
                Variant::nil()
            }),
        );
        GML_HOOK_CONNECTED.store(true, Ordering::Relaxed);
        // 关键兜底：钩子在首帧才连上 node_added，而 F6/主场景在首帧之前
        // 就已挂树（node_added 已错过）——补扫现有树，否则主场景内的 gml
        // 面板信号全部不会连接
        if let Some(root) = tree.get_root() {
            let root_node = root.upcast::<Node>();
            crate::ui::gdui_builder::scan_and_connect_gml_trees(&root_node);
        }
    }
}

pub fn unregister_gdcore_singleton() {
    // 退出前最后落盘一次当前会话时长（此后单例注销，无法再 flush）
    if let Some(mut core) = gdcore_singleton() {
        core.bind_mut().flush_play_time();
    }
    let name = StringName::from("GDCORE");
    Engine::singleton().unregister_singleton(&name);
}
