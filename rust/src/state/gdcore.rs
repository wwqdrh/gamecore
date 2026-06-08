// GDCore - 全局核心单例
// 继承 RefCounted，作为 Engine singleton 注册为 "GDCORE"
// 支持存档 ID 管理，根据 save_id 切换不同的存档文件
// 存档文件路径：user://coredata_{id}.data（id 为空时为 user://coredata.data）

use std::collections::HashMap;

use godot::prelude::*;
use godot::classes::{Engine, IRefCounted};
use godot::builtin::StringName;

use super::coredata::GdCoreData;
use super::bean::GdBean;

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GDCore {
    save_id: GString,
    core_data_cache: HashMap<String, Gd<GdCoreData>>,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GDCore {
    fn init(base: Base<RefCounted>) -> Self {
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
}

impl GDCore {
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

pub fn register_gdcore_singleton() {
    let instance = Gd::<GDCore>::from_init_fn(|base| GDCore::init(base));
    let name = StringName::from("GDCORE");
    Engine::singleton().register_singleton(&name, &instance);
    std::mem::forget(instance);
}

pub fn unregister_gdcore_singleton() {
    let name = StringName::from("GDCORE");
    Engine::singleton().unregister_singleton(&name);
}
