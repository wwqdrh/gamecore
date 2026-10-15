// GDCore - 全局核心单例
// 继承 Object（手动内存），作为 Engine singleton 注册为 "GDCORE"
// 支持存档 ID 管理，根据 save_id 切换不同的存档文件
// 存档文件路径：user://coredata_{id}.data（id 为空时为 user://coredata.data）
// 每存档累计游玩时长：get_play_time/flush_play_time，持久化在
// GdCoreData 的 playtime;total（真实秒，跨开关应用累积）

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use godot::prelude::*;
use godot::classes::{DirAccess, DisplayServer, Engine, FileAccess, IObject, Object, Node, SceneTree, Time};
use godot::classes::file_access::ModeFlags;
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
            // 同档重选（点「新游戏」进入等）也要解除删除写盘抑制，
            // 否则该会话的所有进度更新都不会落盘
            if let Some(core) = self.core_data_cache.get(&id_str) {
                core.bind().set_save_suppressed(false);
            }
            return;
        }

        // 重新激活目标档：解除删除抑制（delete_save 开启，重新选中即恢复落盘）
        if let Some(core) = self.core_data_cache.get(&id_str) {
            core.bind().set_save_suppressed(false);
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

    /// 删除存档（游戏内删档的正规途径，勿手动删文件——运行中实例的 30s
    /// 自动落盘会把内存态写回，手动删除会被"复活"覆盖新进度）。
    /// 文件、缓存、当前档 Bean 全部重置；删除当前档时 Bean 切到空 core 回默认值。
    /// 返回文件是否删除成功（文件本不存在也返回 true，幂等）。
    #[func]
    pub fn delete_save(&mut self, id: GString) -> bool {
        let id_str = id.to_string();
        let filename = if id_str.is_empty() {
            GString::from("user://coredata.data")
        } else {
            GString::from(&format!("user://coredata_{}.data", id_str))
        };
        // 1) 删文件（file_exists 检查避免 remove 报错）
        let ok = if FileAccess::file_exists(&filename) {
            let err = DirAccess::remove_absolute(&filename);
            if err != godot::global::Error::OK {
                godot_warn!("GDCORE: delete_save 删除文件失败({:?}): {}", err, filename);
                false
            } else {
                true
            }
        } else {
            true
        };
        // 2) 重建缓存条目（新实例 existed_at_load=false → 后续正常写盘）
        let new_data = GdCoreData::build(
            filename.clone(),
            GString::from("{}"),
            false,
            GString::from("init"),
        );
        let replaced_current = self.save_id.to_string() == id_str;
        self.core_data_cache.insert(id_str.clone(), new_data);
        // 3) 删的是当前档：会话时长清零重算 + Bean 切到重建后的空 core
        if replaced_current {
            self.session_start_ms = Time::singleton().get_ticks_msec() as i64;
            if let Some(core) = self.core_data_cache.get(&id_str).cloned() {
                // 删除写盘抑制：Bean 切核回填默认值（action=""，对新核必写入）
                // 会触发落盘，把刚删的文件在同帧内"复活"——开启抑制后回填只进
                // 内存；抑制保持到 set_save_id 重新激活该档（点「新游戏」进入）
                core.bind().set_save_suppressed(true);
                Self::notify_beans_switch_core(&core);
                // 双保险：抑制外的任何写路径漏网时，最终再删一次
                if FileAccess::file_exists(&filename) {
                    DirAccess::remove_absolute(&filename);
                }
            }
        }
        ok
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

    /// 获取场景根管理器节点（GdSceneRoot——场景切换/转场/相机挂载的统一入口，
    /// ready 时经 register_to_gdcore 注册进全局节点表）。
    /// 先按默认 manager_id "default" 快取（类名校验防占位），未命中再扫全表
    /// 取第一个 GdSceneRoot；均未找到返回 nil（场景根未挂载/尚未 ready）。
    ///
    /// GDScript 用法（GdSceneRoot 原生 change_scene 为 5 参无默认值）：
    ///   GDCORE.get_scene_root().change_scene(alias, data, "", true, false)
    /// 各场景 GdScene 根的 2 参包装仅适合场景内部沿父链查找，组合根
    /// （Control 直挂 SceneLayer）下没有 GdScene，沿链会误中 5 参原生版。
    #[func]
    fn get_scene_root(&self) -> Variant {
        // 快路径：默认 manager_id
        let default_key = GString::from("default").to_variant();
        if let Some(obj) = self.global_nodes.get(&default_key) {
            if is_scene_root(&obj) {
                return obj;
            }
        }
        // 兜底：扫全表（manager_id 被改为非默认值时）
        let values = self.global_nodes.values_array();
        for i in 0..values.len() {
            let v = values.at(i);
            if is_scene_root(&v) {
                return v;
            }
        }
        godot_warn!("GDCORE: 场景根 GdSceneRoot 未注册（未挂载或尚未 ready）");
        Variant::nil()
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
    ///
    /// 编辑器守卫（2026-10-12）：编辑器进程严禁写存档——GDExtension 帧钩子
    /// 在编辑器主循环同样运行，编辑器内存态（可能来自编辑场景时加载的 Bean）
    /// 每 30s 全量回写会覆盖磁盘真档（用户实机踩过：编辑器开着，游戏进度被
    /// 定时打回）。编辑器内调用一律跳过。
    #[func]
    fn flush_play_time(&mut self) {
        if Engine::singleton().is_editor_hint() {
            return;
        }
        let id = self.save_id.to_string();
        let Some(mut data) = self.core_data_cache.get(&id).cloned() else {
            return;
        };
        // 幽灵建档守卫：存档文件不存在且本会话几乎无时长时跳过写盘——
        // 标题页空转 / 删除存档后停留的会话，避免把空默认档写回磁盘
        // （复活已删存档）。正常新游戏流程 Bean 回填会立即建档，不受影响。
        let filename = if id.is_empty() {
            GString::from("user://coredata.data")
        } else {
            GString::from(&format!("user://coredata_{}.data", id))
        };
        if !FileAccess::file_exists(&filename) && self.session_secs() < 1.0 {
            return;
        }
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
/// 编辑器进程直接跳过（帧钩子在编辑器主循环同样运行，见 flush_play_time 守卫注释）。
pub fn playtime_autoflush_tick() {
    if Engine::singleton().is_editor_hint() {
        return;
    }
    let now = Time::singleton().get_ticks_msec() as u64;
    let last = LAST_AUTOFLUSH_MS.load(Ordering::Relaxed);
    if now.saturating_sub(last) < PLAYTIME_AUTOFLUSH_MS {
        return;
    }
    LAST_AUTOFLUSH_MS.store(now, Ordering::Relaxed);
    refresh_instance_lock();
    if let Some(mut core) = gdcore_singleton() {
        core.bind_mut().flush_play_time();
    }
}

// ---- 多实例检测：两个实例同跑会互踩存档（各自 30s 全量回写，last-writer-wins，
// 2026-10-12 实机事故：旧实例把用户删掉的存档"复活"覆盖了新档） ----

/// 实例锁文件（记录最近活动时刻，非互斥锁——只提示不阻断）
const INSTANCE_LOCK_PATH: &str = "user://.game_instance.lock";
/// 锁文件多少秒内有更新视为"另一实例正在运行"
const INSTANCE_LOCK_FRESH_SECS: f64 = 90.0;

static INSTANCE_LOCK_WARNED: AtomicBool = AtomicBool::new(false);

/// 启动时检查 + 每 30s 刷新锁文件；headless 测试与编辑器进程不参与
/// （编辑器开着不应让真游戏启动时误告警"另一实例在运行"）。
fn refresh_instance_lock() {
    if DisplayServer::singleton().get_name().to_string() == "headless"
        || Engine::singleton().is_editor_hint()
    {
        return;
    }
    let path = GString::from(INSTANCE_LOCK_PATH);
    if !INSTANCE_LOCK_WARNED.load(Ordering::Relaxed) {
        if FileAccess::file_exists(&path) {
            let mtime = FileAccess::get_modified_time(&path) as f64;
            let now = Time::singleton().get_unix_time_from_system();
            if now - mtime < INSTANCE_LOCK_FRESH_SECS && mtime > 0.0 {
                INSTANCE_LOCK_WARNED.store(true, Ordering::Relaxed);
                godot_warn!(
                    "GDCORE: 检测到另一个游戏实例可能正在运行（{} 秒内有活动）——多实例会互相覆盖存档，请关闭多余实例！",
                    (now - mtime) as i64
                );
            }
        }
        INSTANCE_LOCK_WARNED.store(true, Ordering::Relaxed); // 只在启动时提示一次
    }
    // 刷新本实例活动时刻
    if let Some(mut f) = FileAccess::open(&path, ModeFlags::WRITE) {
        let now_str = Time::singleton().get_unix_time_from_system().to_string();
        f.store_string(now_str.as_str());
    }
}

/// 取 GDCORE 单例（未注册/已注销返回 None）
fn gdcore_singleton() -> Option<Gd<GDCore>> {
    let singleton = Engine::singleton().get_singleton(&StringName::from("GDCORE"))?;
    singleton.try_cast::<GDCore>().ok()
}

/// Variant 是否为 GdSceneRoot 节点（get_scene_root 的类名校验）
fn is_scene_root(v: &Variant) -> bool {
    match v.try_to::<Gd<Object>>() {
        Ok(obj) => obj.is_class("GdSceneRoot"),
        Err(_) => false,
    }
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
