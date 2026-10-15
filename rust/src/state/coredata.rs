// GdCoreData - 核心数据引擎
// 继承 Resource，底层使用 GJson 提供路径查询的 JSON 数据存储
// 支持加密、文件持久化、变更订阅、作用域管理

use godot::prelude::*;
use godot::classes::{DirAccess, Engine, FileAccess, IResource, Json, Resource};
use godot::classes::file_access::ModeFlags;

use super::gjson::{FileStore, GJson};

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct GdCoreData {
    core: Option<GJson>,

    initial_filename: GString,
    initial_force: bool,
    initial_scope: GString,
    initial_data: GString,
    inited: bool,

    /// 写盘抑制开关（与 save_fn 闭包共享）：删除存档后的 Bean 默认值回填等
    /// 初始化写不应把刚删的文件"复活"，由 GDCORE.delete_save 开启、
    /// set_save_id 重新激活该档时关闭。内存数据照常更新，只是不落盘。
    suppress_save: std::rc::Rc<std::cell::Cell<bool>>,

    base: Base<Resource>,
}

#[godot_api]
impl IResource for GdCoreData {
    fn init(base: Base<Resource>) -> Self {
        Self {
            core: None,
            initial_filename: GString::new(),
            initial_force: false,
            initial_scope: GString::from("init"),
            initial_data: GString::from("{}"),
            inited: false,
            suppress_save: std::rc::Rc::new(std::cell::Cell::new(false)),
            base,
        }
    }
}

#[godot_api]
impl GdCoreData {
    #[func]
    pub fn build(
        filename: GString,
        data: GString,
        force: bool,
        scope: GString,
    ) -> Gd<Self> {
        let mut instance = Gd::<GdCoreData>::from_init_fn(|base| GdCoreData {
            core: None,
            initial_filename: GString::new(),
            initial_force: false,
            initial_scope: GString::new(),
            initial_data: GString::new(),
            inited: false,
            suppress_save: std::rc::Rc::new(std::cell::Cell::new(false)),
            base,
        });
        instance.bind_mut().initial(filename, data, force, scope);
        instance
    }

    #[func]
    fn initial(
        &mut self,
        filename_: GString,
        data: GString,
        force: bool,
        scope: GString,
    ) -> bool {
        self.initial_filename = filename_.clone();
        self.initial_force = force;
        self.initial_scope = scope.clone();
        self.initial_data = data.clone();

        // 从 Godot 路径中提取父目录（不使用 std::path::Path，它不认识 user:// 等协议前缀）
        // 先定位 :// 协议前缀，再在前缀之后查找子目录
        // 例如 "user://coredata.data" -> 无子目录，无需创建
        // 例如 "user://saves/coredata.data" -> "user://saves"，需要创建
        let filename_str = filename_.to_string();
        let dir_path = if let Some(proto_end) = filename_str.find("://") {
            let after_proto = &filename_str[proto_end + 3..];
            if let Some(last_slash) = after_proto.rfind('/') {
                let dir_after_proto = &after_proto[..last_slash];
                if !dir_after_proto.is_empty() {
                    format!("{}://{}", &filename_str[..proto_end], dir_after_proto)
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if !dir_path.is_empty() {
            let dir_gstr = GString::from(&dir_path);
            if let Some(mut dir) = DirAccess::open(&dir_gstr) {
                if !dir.dir_exists(&dir_gstr) {
                    let err = DirAccess::make_dir_recursive_absolute(&dir_gstr);
                    if err != godot::global::Error::OK {
                        godot_error!("Failed to create directories: {}", dir_path);
                        return false;
                    }
                }
            } else {
                let err = DirAccess::make_dir_recursive_absolute(&dir_gstr);
                if err != godot::global::Error::OK {
                    godot_error!("Failed to create directories: {}", dir_path);
                    return false;
                }
            }
        }

        // //godot_print!("make a coredata, with filestore");

        let filename_owned = filename_.to_string();
        let force_owned = self.initial_force;
        let scope_owned = self.initial_scope.to_string();
        let data_owned = self.initial_data.to_string();

        // 外部删除保护：记录加载时文件是否存在。存在而会话中被外部删除（用户
        // 手动删档）→ 回写会把内存里的旧档"复活"并覆盖用户新进度（last-writer-
        // wins，2026-10-12 实机事故），故此后跳过写盘并告警一次。
        // 新档（加载时不存在）不受影响，正常写盘创建。
        let existed_at_load = FileAccess::file_exists(&filename_);

        let load_fn = Box::new(move || -> Vec<u8> {
            let file_gstr = GString::from(&filename_owned);
            let file_in = FileAccess::open(&file_gstr, ModeFlags::READ);
            if file_in.is_none() || force_owned {
                let new_data = format!("{{\"{}\": {}}}", scope_owned, data_owned);
                // //godot_print!(
                //     "coredata, do load_data, file: {} use default value",
                //     filename_owned
                // );
                GJson::encrypt(&new_data)
            } else {
                // //godot_print!("coredata, do load_data, use value in file");
                let mut f = file_in.unwrap();
                let file_len = f.get_length();
                let content = f.get_buffer(file_len as i64);
                content.to_vec()
            }
        });

        let filename_save = filename_.to_string();
        let file_existed = existed_at_load;
        let warn_once = std::rc::Rc::new(std::cell::Cell::new(false));
        let warn_once_save = warn_once.clone();
        let suppress_save = self.suppress_save.clone();
        let save_fn = Box::new(move |data: Vec<u8>| {
            // 写盘抑制（删除存档流程）：内存照常更新，只是不落盘——
            // 防止 Bean 默认值回填把刚删除的存档文件立即写回复活
            if suppress_save.get() {
                return;
            }
            // 编辑器守卫（2026-10-12）：编辑器进程严禁写存档——编辑器内存态
            // 可能来自编辑场景时的 Bean 加载，回写会覆盖磁盘真档（与 30s
            // flush 同源事故）。玩家按 F5 运行的是独立进程，不受影响。
            if Engine::singleton().is_editor_hint() {
                return;
            }
            let file_gstr = GString::from(&filename_save);
            if file_existed && !FileAccess::file_exists(&file_gstr) {
                if !warn_once_save.get() {
                    warn_once_save.set(true);
                    godot_warn!(
                        "GdCoreData: 存档文件在会话期间被外部删除，跳过回写以免复活旧档: {}（删除存档请退出游戏后操作，或调用 GDCORE.delete_save）",
                        filename_save
                    );
                }
                return;
            }
            let file = FileAccess::open(&file_gstr, ModeFlags::WRITE);
            if let Some(mut f) = file {
                f.store_buffer(&PackedByteArray::from(data.as_slice()));
                // //godot_print!("save success");
            } else {
                godot_error!("Failed to save data to file");
            }
        });

        let store = FileStore::new(load_fn, save_fn);
        let mut core = GJson::new(store);
        core.enable_encrypt();
        core.load_by_store();
        self.core = Some(core);
        self.inited = true;
        true
    }

    #[func]
    fn add_scope(&mut self, scope: GString, data: GString) {
        if !self.check_initial() {
            return;
        }
        let v = GJson::to_value(&data.to_string());
        if let Some(ref mut core) = self.core {
            core.update(&scope.to_string(), "", v);
        }
    }

    #[func]
    pub fn watch(&mut self, p_string: GString, p_callback: Callable, scope: GString) {
        if !self.check_initial() || !p_callback.is_valid() {
            return;
        }

        let nfield = format!("{};{}", scope, p_string);

        if let Some(ref mut core) = self.core {
            core.subscribe(
                &nfield,
                Box::new(move |path: &str, _value: &serde_json::Value| -> bool {
                    if p_callback.is_valid() {
                        let val_str = path.to_string();
                        p_callback.call(&[val_str.to_variant()]);
                        true
                    } else {
                        false
                    }
                }),
            );
        }
    }

    /// 写盘抑制开关：开启后所有 save 请求直接跳过（内存数据照常更新）。
    /// 删除存档流程专用（GDCORE.delete_save 开启，set_save_id 重新激活关闭），
    /// 业务侧请勿自行调用。
    #[func]
    pub fn set_save_suppressed(&self, on: bool) {
        self.suppress_save.set(on);
    }

    /// 写盘抑制是否生效中
    #[func]
    pub fn is_save_suppressed(&self) -> bool {
        self.suppress_save.get()
    }

    #[func]
    fn change(
        &mut self,
        field: GString,
        action: GString,
        value: GString,
        scope: GString,
    ) {
        if !self.check_initial() {
            return;
        }
        let v = GJson::to_value(&value.to_string());
        let nfield = format!("{};{}", scope, field);
        if let Some(ref mut core) = self.core {
            core.update(&nfield, &action.to_string(), v);
        }
    }

    #[func]
    pub fn update(
        &mut self,
        field: GString,
        action: GString,
        value: Variant,
        scope: GString,
    ) {
        if !self.check_initial() {
            return;
        }
        let json_str = Json::stringify(&value);
        self.change(field, action, json_str, scope);
    }

    #[func]
    pub fn value(&self, field: GString, default: Variant, scope: GString) -> Variant {
        if !self.check_initial() {
            return false.to_variant();
        }
        let godot_json_str = self.value_str_(field, scope);
        if godot_json_str.to_string().is_empty() {
            return default;
        }

        let json = Json::parse_string(&godot_json_str);
        if json.get_type() == godot::builtin::VariantType::NIL {
            return default;
        }
        json
    }

    #[func]
    pub fn has(&self, field: GString, scope: GString) -> bool {
        if !self.check_initial() {
            return false;
        }
        let result = self.value_str_(field, scope);
        !result.to_string().is_empty()
    }

    #[func]
    fn duplicate_all_string(&self) -> GString {
        if let Some(ref core) = self.core {
            GString::from(&core.duplicate_all_string())
        } else {
            GString::new()
        }
    }

    #[func]
    fn reload_data(&mut self, data: GString) {
        if let Some(ref mut core) = self.core {
            core.reload_data(&data.to_string());
        }
    }

    #[func]
    fn is_inited(&self) -> bool {
        self.inited
    }
}

impl GdCoreData {
    fn check_initial(&self) -> bool {
        self.inited && self.core.is_some()
    }

    fn value_str_(&self, field: GString, scope: GString) -> GString {
        let nfield = format!("{};{}", scope, field);
        if let Some(ref core) = self.core {
            let result = core.query(&nfield);
            if result.is_empty() {
                GString::new()
            } else {
                GString::from(&result)
            }
        } else {
            GString::new()
        }
    }
}
