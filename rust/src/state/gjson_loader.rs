// GdJson / GdJsonLoader - .gjson 加密定义表资源载体与 ResourceFormatLoader
//
// 用途：编辑器中双击 .gjson 文件时以 GdJson 资源打开（inspector 视图），
//   而非文本编辑器——产物是密文字节流，按纯文本打开只会产生无效 UTF-8
//   错误日志且无编辑意义。当前 GdJson 不暴露任何属性（密文无展示价值，
//   后续需要属性面板时再加导出属性/自定义编辑器）。
//
// 组成：
//   - GdJson        base=Resource 的资源载体，解密解析后的数据经
//                   get_data / query 访问（运行时同样可用）
//   - GdJsonLoader  base=ResourceFormatLoader，识别 .gjson 扩展名，
//                   load 时读字节 → GJson::decrypt → serde 解析 → GdJson
//
// 注册：lib.rs Scene stage init 时 add_resource_format_loader；deinit 时
//   移除。与 .gml 的刻意区别见 lib.rs 注释：.gml 是明文源码必须按文本
//   打开，.gjson 是加密产物必须按资源打开。

use std::sync::atomic::{AtomicU64, Ordering};

use godot::prelude::*;
use godot::builtin::{VarArray, VarDictionary};
use godot::classes::{FileAccess, IResourceFormatLoader, ResourceFormatLoader, ResourceLoader};
use serde_json::Value;

use super::gjson::GJson;

/// .gjson 产物资源的运行时载体（解密解析后的 JSON 数据）
#[derive(GodotClass)]
#[class(base = Resource)]
pub struct GdJson {
    base: Base<Resource>,
    /// 解密解析后的 JSON 数据（Dictionary/Array/标量；非导出属性——
    /// inspector 无需展示，经 get_data/query 访问）
    data: Variant,
}

#[godot_api]
impl IResource for GdJson {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            data: Variant::nil(),
        }
    }
}

#[godot_api]
impl GdJson {
    /// 取解密解析后的完整数据（Dictionary 等）
    #[func]
    pub fn get_data(&self) -> Variant {
        self.data.clone()
    }

    /// GJson 分号路径查询（如 "items;pill_hp;name"），未命中返回 nil。
    /// 中间节点须为 Dictionary（按键取）或 Array（按下标取），否则返回 nil
    #[func]
    pub fn query(&self, path: GString) -> Variant {
        let mut cur: Variant = self.data.clone();
        for part in path.to_string().split(';') {
            if part.is_empty() {
                continue;
            }
            if let Ok(dict) = cur.try_to::<VarDictionary>() {
                let key = GString::from(part).to_variant();
                cur = dict.get_or_nil(&key);
            } else if let Ok(arr) = cur.try_to::<VarArray>() {
                let Ok(idx) = part.parse::<usize>() else {
                    return Variant::nil();
                };
                if idx >= arr.len() {
                    return Variant::nil();
                }
                cur = arr.get(idx).unwrap_or_default();
            } else {
                return Variant::nil();
            }
        }
        cur
    }
}

/// .gjson 资源格式加载器（编辑器双击/运行时 ResourceLoader.load 均走此入口）
#[derive(GodotClass)]
#[class(base = ResourceFormatLoader)]
pub struct GdJsonLoader;

#[godot_api]
impl IResourceFormatLoader for GdJsonLoader {
    fn init(_base: Base<ResourceFormatLoader>) -> Self {
        Self
    }

    fn get_recognized_extensions(&self) -> PackedStringArray {
        let mut ext = PackedStringArray::new();
        ext.push(&GString::from("gjson"));
        ext
    }

    fn handles_type(&self, type_: StringName) -> bool {
        let t = type_.to_string();
        t == "Resource" || t == "GdJson"
    }

    /// 编辑器据此在 FileSystem 面板显示资源类型为 GdJson
    fn get_resource_type(&self, _path: GString) -> GString {
        GString::from("GdJson")
    }

    fn load(
        &self,
        path: GString,
        _original_path: GString,
        _use_sub_threads: bool,
        _cache_mode: i32,
    ) -> Variant {
        let path_str = path.to_string();
        if !FileAccess::file_exists(&path) {
            godot_error!("[GdJson] 文件不存在: {}", path_str);
            return Variant::nil();
        }
        let bytes = FileAccess::get_file_as_bytes(&path);
        let text = GJson::decrypt(&bytes.to_vec());
        let parsed: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                godot_error!("[GdJson] {} 解密解析失败: {}（产物损坏？重跑管线重建）", path_str, e);
                return Variant::nil();
            }
        };
        Gd::<GdJson>::from_init_fn(|base| GdJson {
            base,
            data: value_to_variant(&parsed),
        })
        .to_variant()
    }
}

/// serde_json::Value -> Godot Variant（Number 整数保 i64，小数 f64）
fn value_to_variant(v: &Value) -> Variant {
    match v {
        Value::Null => Variant::nil(),
        Value::Bool(b) => b.to_variant(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_variant()
            } else {
                n.as_f64().unwrap_or(0.0).to_variant()
            }
        }
        Value::String(s) => GString::from(s.as_str()).to_variant(),
        Value::Array(arr) => {
            let mut out = VarArray::new();
            for item in arr {
                out.push(&value_to_variant(item));
            }
            out.to_variant()
        }
        Value::Object(map) => {
            let mut out = VarDictionary::new();
            for (k, item) in map {
                out.set(&GString::from(k.as_str()).to_variant(), &value_to_variant(item));
            }
            out.to_variant()
        }
    }
}

// ------------------------------------------------------------------ 注册

/// 已注册 loader 的实例 id（0 = 未注册；Gd 非线程安全，只存 id 便于注销时还原）
static GJSON_LOADER_ID: AtomicU64 = AtomicU64::new(0);

/// 注册 .gjson ResourceFormatLoader（幂等）
pub fn register_gjson_loader() {
    if GJSON_LOADER_ID.load(Ordering::SeqCst) != 0 {
        return;
    }
    let loader = Gd::<GdJsonLoader>::from_init_fn(|base| GdJsonLoader::init(base));
    let l = loader.upcast::<ResourceFormatLoader>();
    ResourceLoader::singleton().add_resource_format_loader(&l);
    GJSON_LOADER_ID.store(l.instance_id().to_i64() as u64, Ordering::SeqCst);
}

/// 注销 loader（编辑器/游戏退出时调用；幂等）
pub fn unregister_gjson_loader() {
    let id = GJSON_LOADER_ID.swap(0, Ordering::SeqCst);
    if id == 0 {
        return;
    }
    if let Ok(l) = Gd::<ResourceFormatLoader>::try_from_instance_id(InstanceId::from_i64(id as i64))
    {
        ResourceLoader::singleton().remove_resource_format_loader(&l);
    }
}
