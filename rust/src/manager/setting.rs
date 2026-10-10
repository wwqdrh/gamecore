// GdViewSetting - 游戏设置管理器
// 继承 RefCounted，统一封装音频/窗口/自定义设置的读取、应用与持久化
// 音频：AudioServer 总线音量（Master/Music/Audio/Voice，线性值 0..1 自动换算 dB）
// 窗口：根 Window 节点全屏切换（保持 Window 内部状态同步）、DisplayServer 垂直同步
// 持久化：GdCoreData 独立设置文件（默认 user://settings.data，与游戏存档分离），
//         每次修改立即加密落盘，build 时自动恢复并应用
// 自定义设置：任意 Variant 键值，同样即时持久化，支持 watch 订阅

use godot::prelude::*;
use godot::builtin::{GString, Variant};
use godot::classes::{
    AudioServer, DisplayServer, Engine, IRefCounted, RefCounted, SceneTree, Window,
};
use godot::classes::display_server::VSyncMode;
use godot::classes::window::Mode;
use godot::global::{linear_to_db, db_to_linear};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::state::coredata::GdCoreData;

/// 已知设置的持久化键名（音量按总线名小写 + fullscreen/vsync）
const VOLUME_BUSES: [&str; 4] = ["Master", "Music", "Audio", "Voice"];

/// 进程级一次性恢复标记（restore_persisted_once）
static SETTINGS_RESTORED: AtomicBool = AtomicBool::new(false);

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdViewSetting {
    /// 设置文件路径（独立于游戏存档）
    #[var(pub)]
    save_file: GString,

    /// 设置数据的作用域（GdCoreData 顶层键）
    #[var(pub)]
    scope: GString,

    /// 持久化后端
    core: Option<Gd<GdCoreData>>,

    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GdViewSetting {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            save_file: GString::from("user://settings.data"),
            scope: GString::from("setting"),
            core: None,
            base,
        }
    }
}

#[godot_api]
impl GdViewSetting {
    /// 构建设置管理器：加载存档并应用已保存的音频/窗口设置
    /// save_file: 设置文件路径（user://settings.data）
    /// scope: 数据作用域（setting）
    #[func]
    pub fn build(save_file: GString, scope: GString) -> Gd<Self> {
        let mut instance = Gd::<Self>::from_init_fn(|base| Self {
            save_file,
            scope,
            core: None,
            base,
        });
        instance.bind_mut().do_init();
        instance
    }

    /// 读取设置值（自定义键），缺失时返回 default
    #[func]
    pub fn get_value(&self, key: GString, default: Variant) -> Variant {
        match &self.core {
            Some(core) => core.bind().value(key, default, self.scope.clone()),
            None => default,
        }
    }

    /// 写入自定义设置（立即持久化）
    #[func]
    pub fn set_value(&mut self, key: GString, value: Variant) {
        self.persist(key, value);
    }

    /// 设置指定音频总线的音量（线性值 0..1，自动换算 dB 并处理静音）
    #[func]
    pub fn set_volume(&mut self, bus_name: GString, value: f64) {
        let v = value.clamp(0.0, 1.0);
        Self::apply_volume(&bus_name.to_string(), v);
        let key = format!("volume_{}", bus_name.to_string().to_lowercase());
        self.persist(GString::from(key.as_str()), v.to_variant());
    }

    /// 读取指定音频总线当前音量（线性值 0..1；总线不存在返回 1.0）
    #[func]
    pub fn get_volume(&self, bus_name: GString) -> f64 {
        let server = AudioServer::singleton();
        let idx = server.get_bus_index(&StringName::from(&bus_name));
        if idx < 0 {
            return 1.0;
        }
        if server.is_bus_mute(idx) {
            return 0.0;
        }
        db_to_linear(server.get_bus_volume_db(idx) as f64)
    }

    /// 切换全屏（经根 Window 节点设置，保持其内部状态同步；
    /// 立即持久化，headless 下只持久化不应用）
    #[func]
    pub fn set_fullscreen(&mut self, enabled: bool) {
        self.apply_fullscreen(enabled);
        self.persist(GString::from("fullscreen"), enabled.to_variant());
    }

    /// 当前是否全屏
    #[func]
    pub fn is_fullscreen(&self) -> bool {
        match Self::main_window() {
            Some(win) => win.get_mode() == Mode::FULLSCREEN,
            None => false,
        }
    }

    /// 切换垂直同步（headless 下只持久化不应用）
    #[func]
    pub fn set_vsync(&mut self, enabled: bool) {
        // 编辑器内只持久化不应用（垂直同步作用于编辑器窗口无意义且有害）
        if !Self::is_headless() && !Engine::singleton().is_editor_hint() {
            let mode = if enabled { VSyncMode::ENABLED } else { VSyncMode::DISABLED };
            DisplayServer::singleton().window_set_vsync_mode(mode);
        }
        self.persist(GString::from("vsync"), enabled.to_variant());
    }

    /// 订阅设置变更（强制写入触发；回调参数为字段路径）
    #[func]
    pub fn watch(&mut self, key: GString, callback: Callable) {
        if let Some(ref mut core) = self.core {
            core.bind_mut().watch(key, callback, self.scope.clone());
        }
    }

    /// 从设置文件恢复并应用已知设置（音频音量 + 窗口模式）。
    /// 只应用"已存储"的键：缺省键不写入也不应用，默认值交给 UI 层决定，
    /// 避免加载阶段把兜底值（如音量 1.0）覆盖进文件
    #[func]
    pub fn load_and_apply(&mut self) {
        // 编辑器内只加载不应用：编辑器启动会自动恢复上次打开的场景标签，
        // settings 面板的 tool 组件 ready 时构建本类，若应用会把编辑器窗口
        // 切回 WINDOWED（最大化被重置成固定大小）、并改写编辑器音频总线
        if Engine::singleton().is_editor_hint() {
            return;
        }
        for bus_name in VOLUME_BUSES {
            let key = format!("volume_{}", bus_name.to_lowercase());
            if self.has_stored(GString::from(key.as_str())) {
                let v = self.get_value(GString::from(key.as_str()), 1.0.to_variant());
                Self::apply_volume(&bus_name, v.to::<f64>());
            }
        }
        // 全屏恢复：只应用"开启"偏好。窗口化是进程启动默认态，restore 严禁
        // 主动强制 WINDOWED——本管理器是懒构建的（首个设置组件 ready 时才 build），
        // 若在会话中途强制窗口化会踢掉用户已做的系统级全屏（如 macOS 绿灯按钮），
        // 表现为「标题页全屏 → 点开始游戏进主场景时被弹回固定窗口」。
        // 关闭全屏只应来自用户显式切换（SettingSwitch → set_fullscreen）。
        if self.has_stored(GString::from("fullscreen")) {
            let fs = self.get_value(GString::from("fullscreen"), false.to_variant());
            if fs.to::<bool>() {
                self.apply_fullscreen(true);
            }
        }
        if self.has_stored(GString::from("vsync")) && !Self::is_headless() {
            let vs = self.get_value(GString::from("vsync"), true.to_variant());
            let mode = if vs.to::<bool>() { VSyncMode::ENABLED } else { VSyncMode::DISABLED };
            DisplayServer::singleton().window_set_vsync_mode(mode);
        }
    }
}

/// 应用启动时一次性恢复持久化设置（音频音量/全屏/垂直同步）。
///
/// 背景：此前 GdViewSetting 只在设置 UI（SettingSwitch 等接口组件）首次
/// ready 时才经 ui_form::fetch_setting 懒构建并 load_and_apply——标题场景
/// 没有设置 UI，用户在设置里开启的全屏要等进主场景才生效（症状：
/// 「启动在标题页是窗口，点开始游戏进主场景才突然全屏」）。改为进程
/// 首帧（lib.rs on_main_loop_frame）主动恢复，每进程仅一次（幂等）。
pub fn restore_persisted_once() {
    if SETTINGS_RESTORED.swap(true, Ordering::Relaxed) {
        return;
    }
    // build 内部 do_init → load_and_apply：恢复音量 + 全屏（只应用 true，
    // 详见 load_and_apply 注释）+ 垂直同步。headless / 编辑器内自动跳过应用。
    let _setting = GdViewSetting::build(
        GString::from("user://settings.data"),
        GString::from("setting"),
    );
    godot_print!("[GdViewSetting] 启动恢复持久化设置（音频/全屏/垂直同步）");
}

// 私有实现
impl GdViewSetting {
    fn has_stored(&self, key: GString) -> bool {
        match &self.core {
            Some(core) => core.bind().has(key, self.scope.clone()),
            None => false,
        }
    }

    /// 仅应用音量到 AudioServer，不持久化（加载路径使用）
    fn apply_volume(bus_name: &str, v: f64) {
        // 编辑器内禁止应用：tool 组件在编辑器构建设置桥时会走到这里，
        // 改的是编辑器自身的 AudioServer 总线
        if Engine::singleton().is_editor_hint() {
            return;
        }
        let v = v.clamp(0.0, 1.0);
        let idx = Self::ensure_bus(&GString::from(bus_name));
        if idx < 0 {
            return;
        }
        let mut server = AudioServer::singleton();
        if v > 0.0 {
            server.set_bus_volume_db(idx, linear_to_db(v) as f32);
            server.set_bus_mute(idx, false);
        } else {
            server.set_bus_volume_db(idx, -80.0);
            server.set_bus_mute(idx, true);
        }
    }
    fn do_init(&mut self) {
        let core = GdCoreData::build(
            self.save_file.clone(),
            GString::from("{}"),
            false,
            self.scope.clone(),
        );
        self.core = Some(core);
        self.load_and_apply();
    }

    /// 应用 + 持久化（update("~") 立即落盘）
    fn persist(&mut self, key: GString, value: Variant) {
        if let Some(ref mut core) = self.core {
            core.bind_mut().update(key, GString::from("~"), value, self.scope.clone());
        }
    }

    /// 确保音频总线存在并返回索引（Music/Audio/Voice 兜底创建，与 GdViewAudio 一致）
    fn ensure_bus(bus_name: &GString) -> i32 {
        let mut server = AudioServer::singleton();
        let bus_sn = StringName::from(bus_name);
        let idx = server.get_bus_index(&bus_sn);
        if idx >= 0 {
            return idx;
        }
        server.add_bus();
        let new_idx = server.get_bus_count() - 1;
        server.set_bus_name(new_idx, bus_name);
        new_idx
    }

    /// 全屏应用：走根 Window 节点（不要直接调 DisplayServer.window_set_mode，
    /// 否则 Window 内部缓存的 mode 会在窗口事件时把模式覆盖回窗口化）。
    /// 注意：从编辑器"嵌入游戏窗口"模式运行时 fullscreen 无视觉效果，属正常现象
    fn apply_fullscreen(&self, enabled: bool) {
        // 编辑器内禁止应用：会把编辑器自身窗口切到 WINDOWED（最大化被重置成固定大小）
        if Self::is_headless() || Engine::singleton().is_editor_hint() {
            return;
        }
        if let Some(mut win) = Self::main_window() {
            let mode = if enabled { Mode::FULLSCREEN } else { Mode::WINDOWED };
            win.set_mode(mode);
        }
    }

    fn main_window() -> Option<Gd<Window>> {
        let main_loop = Engine::singleton().get_main_loop()?;
        let tree: Gd<SceneTree> = main_loop.try_cast().ok()?;
        tree.get_root()
    }

    fn is_headless() -> bool {
        DisplayServer::singleton().get_name().to_string() == "headless"
    }
}
