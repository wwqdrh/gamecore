// GdConfigManager - 通用配置管理器
// 继承 RefCounted，按需创建时读取 res://game_config.json
// 内置 scenes 属性（alias → PackedScene），支持自定义配置项
// 通过 GDCORE 的 add_global_node 注册为全局节点
// 使用 RefCounted 确保引用计数正确管理生命周期，避免被提前释放

use godot::prelude::*;
use godot::builtin::{GString, VarDictionary, Variant, StringName, VarArray};
use godot::classes::{IRefCounted, FileAccess, Json, ResourceLoader, Engine, PackedScene};
use godot::classes::file_access::ModeFlags;

/// 默认配置文件路径
const DEFAULT_CONFIG_PATH: &str = "res://game_config.json";

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdConfigManager {
    /// 配置文件路径
    #[export]
    config_path: GString,

    /// 原始配置数据
    config_data: VarDictionary,

    /// 已加载的场景映射 (alias -> Gd<PackedScene>)
    scenes: VarDictionary,

    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GdConfigManager {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            config_path: GString::from(DEFAULT_CONFIG_PATH),
            config_data: VarDictionary::new(),
            scenes: VarDictionary::new(),
            base,
        }
    }
}

#[godot_api]
impl GdConfigManager {
    /// 获取配置项
    /// key: 配置键名
    /// default: 默认值
    #[func]
    fn get_config(&self, key: GString, default: Variant) -> Variant {
        let key_var = key.to_variant();
        let val = self.config_data.get_or_nil(&key_var);
        if val.is_nil() {
            default
        } else {
            val
        }
    }

    /// 设置配置项（运行时修改，不会持久化到文件）
    #[func]
    fn set_config(&mut self, key: GString, value: Variant) {
        self.config_data.set(&key.to_variant(), &value);
    }

    /// 获取已加载的场景 PackedScene
    /// alias: 场景别名
    #[func]
    fn get_scene(&self, alias: GString) -> Variant {
        self.scenes.get_or_nil(&alias.to_variant())
    }

    /// 获取所有已注册的场景别名
    #[func]
    fn get_scene_aliases(&self) -> VarArray {
        self.scenes.keys_array()
    }

    /// 获取完整配置数据
    #[func]
    fn get_config_data(&self) -> VarDictionary {
        self.config_data.clone()
    }

    /// 重新加载配置文件
    #[func]
    fn reload_config(&mut self) -> bool {
        self.load_config()
    }
}

impl GdConfigManager {
    /// 从文件加载配置
    fn load_config(&mut self) -> bool {
        let path = if self.config_path.is_empty() {
            GString::from(DEFAULT_CONFIG_PATH)
        } else {
            self.config_path.clone()
        };

        // 读取文件
        let file = FileAccess::open(&path, ModeFlags::READ);
        if file.is_none() {
            godot_warn!("GdConfigManager: config file '{}' not found, using empty config", path);
            return false;
        }

        let mut f = file.unwrap();
        let content = f.get_as_text();

        // 解析 JSON
        let json_result = Json::parse_string(&content);
        if json_result.get_type() != godot::builtin::VariantType::DICTIONARY {
            godot_warn!("GdConfigManager: config file '{}' is not a valid JSON object", path);
            return false;
        }

        let config = json_result.to::<VarDictionary>();
        self.config_data = config;

        // 加载内置 scenes 配置
        self.load_scenes();

        godot_print!("GdConfigManager: loaded config from '{}'", path);
        true
    }

    /// 从配置中加载 scenes 映射
    fn load_scenes(&mut self) {
        let scenes_key = GString::from("scenes").to_variant();
        let scenes_val = self.config_data.get_or_nil(&scenes_key);

        if scenes_val.get_type() != godot::builtin::VariantType::DICTIONARY {
            return;
        }

        let scenes_dict = scenes_val.to::<VarDictionary>();
        let keys = scenes_dict.keys_array();

        for i in 0..keys.len() {
            let key = keys.at(i);
            let path_var = scenes_dict.get_or_nil(&key);

            if path_var.get_type() != godot::builtin::VariantType::STRING {
                godot_warn!("GdConfigManager: scene path for key '{}' is not a string", key);
                continue;
            }

            let scene_path = path_var.to::<GString>();

            // 使用 ResourceLoader 加载 PackedScene
            if let Some(res) = ResourceLoader::singleton().load(&scene_path) {
                if let Ok(packed_scene) = res.try_cast::<PackedScene>() {
                    self.scenes.set(&key, &packed_scene.to_variant());
                } else {
                    godot_warn!(
                        "GdConfigManager: resource at '{}' is not a PackedScene",
                        scene_path
                    );
                }
            } else {
                godot_warn!("GdConfigManager: failed to load scene '{}'", scene_path);
            }
        }
    }
}

/// 确保 GdConfigManager 已创建并注册到 GDCORE
/// 由 GdSceneRoot 在 _ready 时调用，此时场景树已就绪，可以安全调用 ResourceLoader
/// 返回 GdConfigManager 的 Gd 引用（如果已存在则返回已有的）
pub fn ensure_config_manager() -> Option<Gd<GdConfigManager>> {
    let mut engine = Engine::singleton();
    let mut gdcore = engine.get_singleton("GDCORE")?;

    // 先检查是否已注册
    let config_var = gdcore.call(
        &StringName::from("get_global_node"),
        &[GString::from("config").to_variant()],
    );
    if !config_var.is_nil() {
        // 已存在，尝试转换
        if let Ok(config) = config_var.try_to::<Gd<GdConfigManager>>() {
            return Some(config);
        }
    }

    // 创建新的 GdConfigManager（RefCounted，引用计数自动管理）
    let mut config_manager = Gd::<GdConfigManager>::from_init_fn(|base| GdConfigManager::init(base));

    // 直接加载配置（此时场景树已就绪，可以安全调用 ResourceLoader）
    config_manager.bind_mut().load_config();

    // 注册到 GDCore（RefCounted 的 Variant 会保持引用计数）
    gdcore.call(
        &StringName::from("add_global_node"),
        &[GString::from("config").to_variant(), config_manager.to_variant()],
    );

    Some(config_manager)
}
