// GdBean - 数据绑定 Bean
// 继承 RefCounted，持有 GdCoreData 引用作为数据后端
// 支持属性监听、UI 绑定、表达式更新、存档管理
// 通过全局 instances 映射管理所有 Bean 实例的生命周期

use std::collections::HashMap;
use std::sync::LazyLock;

use godot::prelude::*;
use godot::classes::{Engine, IRefCounted, Json};
use godot::builtin::{VarArray, VarDictionary};

use super::coredata::GdCoreData;

fn variant_has_method(v: &Variant, method: &str) -> bool {
    if v.get_type() != godot::builtin::VariantType::OBJECT {
        return false;
    }
    v.call("has_method", &[method.to_variant()]).to()
}

const PROPERTY_USAGE_SCRIPT_VARIABLE: i64 = 8192;
const PROPERTY_USAGE_STORE_IF_NULL: i64 = 4096;

static BEAN_INSTANCES: LazyLock<parking_lot::Mutex<HashMap<String, i64>>> =
    LazyLock::new(|| parking_lot::Mutex::new(HashMap::new()));

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdBean {
    core: Option<Gd<GdCoreData>>,
    prefix: GString,
    propers: VarArray,
    excludes: VarArray,
    force: bool,
    scope: GString,
    initial_data: VarDictionary,
    bind_ui_text_node: VarDictionary,

    callback_map: HashMap<String, Vec<Callable>>,
    registered_callback: HashMap<u32, bool>,
    property_cb_map: HashMap<String, Vec<Callable>>,
    property_val_cache: HashMap<String, Variant>,

    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GdBean {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            core: None,
            prefix: GString::new(),
            propers: VarArray::new(),
            excludes: VarArray::new(),
            force: false,
            scope: GString::from("init"),
            initial_data: VarDictionary::new(),
            bind_ui_text_node: VarDictionary::new(),
            callback_map: HashMap::new(),
            registered_callback: HashMap::new(),
            property_cb_map: HashMap::new(),
            property_val_cache: HashMap::new(),
            base,
        }
    }
}

#[godot_api]
impl GdBean {
    #[func]
    fn bean(bean_id: GString, fn_callable: Callable) -> Gd<Self> {
        let core = {
            let engine = Engine::singleton();
            if let Some(mut c) = engine.get_singleton("GDCORE") {
                c.call("get_root_data", &[])
                    .try_to::<Gd<GdCoreData>>()
                    .ok()
                    .map(|gd| gd.to_variant())
                    .unwrap_or(Variant::nil())
            } else {
                Variant::nil()
            }
        };

        let id = bean_id.to_string();
        {
            let instances = BEAN_INSTANCES.lock();
            if let Some(&instance_id) = instances.get(&id) {
                if let Ok(gd) =
                    Gd::<GdBean>::try_from_instance_id(InstanceId::from_i64(instance_id))
                {
                    return gd;
                }
            }
        }

        let result = fn_callable.call(&[]);
        let mut ins = match result.try_to::<Gd<GdBean>>() {
            Ok(gd) => gd,
            Err(_) => {
                godot_error!("the coredata initial is after gdbean");
                return Gd::<GdBean>::from_init_fn(|base| GdBean::init(base));
            }
        };

        if let Ok(core_gd) = core.try_to::<Gd<GdCoreData>>() {
            ins.bind_mut().initial(bean_id.clone(), core_gd);
            ins.call("on_ready", &[]);

            let instance_id = ins.instance_id().to_i64();
            BEAN_INSTANCES.lock().insert(id, instance_id);
        } else {
            godot_error!("the coredata is unrefed");
        }

        ins
    }

    #[func]
    fn initial(&mut self, prefix: GString, core: Gd<GdCoreData>) {
        self.core = Some(core.clone());
        self.prefix = prefix.clone();

        {
            let mut core_gd = self.core.as_ref().unwrap().clone();
            core_gd.call(
                "change",
                &[
                    prefix.to_variant(),
                    GString::from("").to_variant(),
                    GString::from("{}").to_variant(),
                    self.scope.to_variant(),
                ],
            );
        }

        self.propers.clear();

        let properties: Vec<GString> = {
            let mut base = self.base_mut();
            let props = base.call("get_property_list", &[]);
            let mut result = Vec::new();
            let len: i64 = props.call("size", &[]).to();
            for i in 0..len {
                let pi = props.call("get", &[i.to_variant()]);
                let usage: i64 = pi.call("get", &["usage".to_variant()]).to();
                if usage & (PROPERTY_USAGE_SCRIPT_VARIABLE | PROPERTY_USAGE_STORE_IF_NULL) != 0 {
                    let name: GString = pi.call("get", &["name".to_variant()]).to();
                    let name_str = name.to_string();
                    if name_str == "script" {
                        continue;
                    }
                    result.push(name);
                }
            }
            result
        };

        for key in properties {
            let key_str = key.to_string();
            let key_var = key.to_variant();
            if self.excludes.contains(&key_var)
                || key_str.starts_with('_')
                || key_str.ends_with('_')
            {
                continue;
            }
            self.propers.push(&key);

            let current_val = {
                let mut base = self.base_mut();
                base.call("get", &[key.clone().to_variant()])
            };
            self.initial_data.set(&key_var, &current_val);

            let key_prefix = format!("{};{}", prefix, key);
            let core_has: bool = {
                let mut core_gd = self.core.as_ref().unwrap().clone();
                core_gd.call("has", &[key_prefix.to_variant(), self.scope.to_variant()]).to()
            };

            if core_has && !self.force {
                let has_from_json: bool = variant_has_method(&current_val, "from_json");
                if has_from_json {
                    let json_val = {
                        let mut core_gd = self.core.as_ref().unwrap().clone();
                        core_gd.call(
                            "value",
                            &[key_prefix.to_variant(), Variant::nil(), self.scope.to_variant()],
                        )
                    };
                    let new_val = current_val.call("from_json", &[json_val]);
                    self.base_mut().call("set", &[key.to_variant(), new_val]);
                } else {
                    let core_val = {
                        let mut core_gd = self.core.as_ref().unwrap().clone();
                        core_gd.call(
                            "value",
                            &[key_prefix.to_variant(), Variant::nil(), self.scope.to_variant()],
                        )
                    };
                    self.base_mut().call("set", &[key.to_variant(), core_val]);
                }
            } else {
                let has_to_json: bool = variant_has_method(&current_val, "to_json");
                if has_to_json {
                    let json_val = current_val.call("to_json", &[]);
                    let mut core_gd = self.core.as_ref().unwrap().clone();
                    core_gd.call(
                        "update",
                        &[
                            format!("{};{}", prefix, key).to_variant(),
                            GString::from("").to_variant(),
                            json_val,
                            self.scope.to_variant(),
                        ],
                    );
                } else {
                    let mut core_gd = self.core.as_ref().unwrap().clone();
                    core_gd.call(
                        "update",
                        &[
                            format!("{};{}", prefix, key).to_variant(),
                            GString::from("").to_variant(),
                            current_val,
                            self.scope.to_variant(),
                        ],
                    );
                }
            }
        }
    }

    #[func]
    fn set_excludes(&mut self, data: VarArray) {
        self.excludes = data;
    }

    #[func]
    fn set_force(&mut self, data: bool) {
        self.force = data;
    }

    #[func]
    fn set_scope(&mut self, data: GString) {
        self.scope = data;
    }

    #[func]
    fn bind_node_text(&mut self, name: GString, mut target: Gd<godot::classes::Control>) {
        let name_var = name.to_variant();
        if !self.bind_ui_text_node.contains_key(&name_var) {
            self.bind_ui_text_node
                .set(&name_var, &VarArray::new());
        }
        if let Ok(mut nodes) = self
            .bind_ui_text_node
            .get_or_nil(&name_var)
            .try_to::<VarArray>()
        {
            nodes.push(&target.to_variant());
        }

        let val = self.get_value_by_key(name.clone());
        let text = variant_to_text(&val);
        target.call("set_text", &[text.to_variant()]);
    }

    #[func]
    fn emit_node_text(&mut self, name: GString) {
        let name_var = name.to_variant();
        if !self.bind_ui_text_node.contains_key(&name_var) {
            return;
        }
        if let Ok(nodes) = self
            .bind_ui_text_node
            .get_or_nil(&name_var)
            .try_to::<VarArray>()
        {
            for i in 0..nodes.len() {
                let Some(node_var) = nodes.get(i) else { continue };
                let has_set_text: bool = variant_has_method(&node_var, "set_text");
                if has_set_text {
                    let val = self.get_value_by_key_read(name.clone());
                    let text = variant_to_text(&val);
                    node_var.call("set_text", &[text.to_variant()]);
                }
            }
        }
    }

    #[func]
    fn to_dict(&mut self, excludes: VarArray) -> VarDictionary {
        let mut result = VarDictionary::new();
        let properties = self.base_mut().call("get_property_list", &[]);
        let len: i64 = properties.call("size", &[]).to();
        for i in 0..len {
            let pi = properties.call("get", &[i.to_variant()]);
            let usage: i64 = pi.call("get", &["usage".to_variant()]).to();
            if usage & (PROPERTY_USAGE_SCRIPT_VARIABLE | PROPERTY_USAGE_STORE_IF_NULL) != 0 {
                let key: GString = pi.call("get", &["name".to_variant()]).to();
                let key_str = key.to_string();
                if key_str == "script" || excludes.contains(&key.to_variant()) {
                    continue;
                }
                let val = self.base_mut().call("get", &[key.to_variant()]);
                result.set(&key.to_variant(), &val);
            }
        }
        result
    }

    #[func]
    fn emit(&mut self, keys: PackedStringArray, #[opt(default = true)] force: bool) {
        for key in keys.as_slice() {
            let key_gstr = key.clone();
            let val = self.get_value_by_key(key_gstr.clone());
            self.update(key_gstr, val, VarDictionary::new(), force);
        }
    }

    #[func]
    fn watch_property(&mut self, key: GString, cb: Callable) {
        let method_name = format!("property_{}", key);
        let has: bool = self
            .base_mut()
            .call("has_method", &[method_name.to_variant()])
            .to();
        if !has {
            return;
        }
        let key_str = key.to_string();
        if !self.property_cb_map.contains_key(&key_str) {
            self.property_cb_map.insert(key_str.clone(), Vec::new());
        }
        if let Some(callbacks) = self.property_cb_map.get_mut(&key_str) {
            callbacks.push(cb.clone());
        }

        let first_val = self.base_mut().call(&method_name, &[]);
        self.property_val_cache.insert(key_str, first_val.clone());
        cb.call(&[first_val]);
    }

    #[func]
    fn check_property_val(&mut self, key: GString) {
        let method_name = format!("property_{}", key);
        let has: bool = self
            .base_mut()
            .call("has_method", &[method_name.to_variant()])
            .to();
        if !has {
            return;
        }
        let val = self.base_mut().call(&method_name, &[]);
        let key_str = key.to_string();
        if let Some(cached) = self.property_val_cache.get(&key_str) {
            if &val != cached {
                if let Some(callbacks) = self.property_cb_map.get(&key_str) {
                    for cb in callbacks {
                        cb.call(&[val.clone()]);
                    }
                }
            }
        }
        self.property_val_cache.insert(key_str, val);
    }

    #[func]
    fn patch_value(&mut self, mut target: Gd<godot::classes::Node>) {
        for i in 0..self.propers.len() {
            let Some(key) = self.propers.get(i) else { continue };
            let key: GString = key.to();
            let val = self.base_mut().call("get", &[key.clone().to_variant()]);
            target.call("set", &[key.to_variant(), val]);
        }
    }

    #[func]
    fn flush(&mut self, excludes: VarArray) {
        let keys = self.initial_data.keys_array();
        for i in 0..keys.len() {
            let Some(key) = keys.get(i) else { continue };
            let key: GString = key.to();
            let key_var = key.to_variant();
            if !excludes.contains(&key_var) {
                let val = self.base_mut().call("get", &[key.to_variant()]);
                self.update(key, val, VarDictionary::new(), true);
            }
        }
    }

    #[func]
    fn reinit(&mut self, excludes: VarArray) {
        let keys = self.initial_data.keys_array();
        for i in 0..keys.len() {
            let Some(key) = keys.get(i) else { continue };
            let key: GString = key.to();
            let key_var = key.to_variant();
            if !excludes.contains(&key_var) {
                let val = self.initial_data.get_or_nil(&key_var);
                self.update(key, val, VarDictionary::new(), false);
            }
        }
    }

    #[func]
    fn update_by_expression(&mut self, expression: GString) {
        let expr_str = expression.to_string();
        let parts: Vec<&str> = expr_str.split('|').collect();
        for part in parts {
            if part.contains('@') {
                let bean_exp: Vec<&str> = part.split('@').collect();
                if bean_exp.len() >= 2 {
                    if let Some(mut target_bean) = get_bean_by_id(bean_exp[0]) {
                        target_bean.bind_mut().update_by_expression(bean_exp[1].into());
                    } else {
                        godot_warn!("not found gdbean: {}", bean_exp[0]);
                    }
                }
                continue;
            }

            let subparts: Vec<&str> = part.split(':').collect();
            if subparts.len() != 3 {
                godot_warn!("expression format error");
                continue;
            }

            let action = subparts[1];
            let value_str = subparts[2];
            let current_val = self.get_value_by_key(subparts[0].into());

            match current_val.get_type() {
                godot::builtin::VariantType::INT => {
                    let mut new_val: i64 = current_val.to();
                    let value_int: i64 = value_str.parse().unwrap_or(0);
                    match action {
                        "+" => new_val += value_int,
                        "-" => new_val -= value_int,
                        "*" => new_val *= value_int,
                        "/" => new_val /= value_int,
                        "=" => new_val = value_int,
                        _ => {
                            godot_warn!("unsupported action for int");
                            continue;
                        }
                    }
                    self.update(
                        subparts[0].into(),
                        new_val.to_variant(),
                        VarDictionary::new(),
                        false,
                    );
                }
                godot::builtin::VariantType::FLOAT => {
                    let mut new_val: f64 = current_val.to();
                    let value_float: f64 = value_str.parse().unwrap_or(0.0);
                    match action {
                        "+" => new_val += value_float,
                        "-" => new_val -= value_float,
                        "*" => new_val *= value_float,
                        "/" => new_val /= value_float,
                        "=" => new_val = value_float,
                        _ => {
                            godot_warn!("unsupported action for float");
                            continue;
                        }
                    }
                    self.update(
                        subparts[0].into(),
                        new_val.to_variant(),
                        VarDictionary::new(),
                        false,
                    );
                }
                godot::builtin::VariantType::PACKED_INT32_ARRAY => {
                    let mut new_val: PackedInt32Array = current_val.to();
                    let value_int: i32 = value_str.parse().unwrap_or(0);
                    match action {
                        "+" => new_val.push(value_int),
                        "-" => {
                            if let Some(idx) =
                                new_val.as_slice().iter().position(|&x| x == value_int)
                            {
                                new_val.remove(idx);
                            }
                        }
                        _ => {
                            godot_warn!("unsupported action for packed int array");
                            continue;
                        }
                    }
                    self.update(
                        subparts[0].into(),
                        new_val.to_variant(),
                        VarDictionary::new(),
                        false,
                    );
                }
                godot::builtin::VariantType::PACKED_STRING_ARRAY => {
                    let mut new_val: PackedStringArray = current_val.to();
                    match action {
                        "+" => new_val.push(&GString::from(value_str)),
                        "-" => {
                            if let Some(idx) = new_val
                                .as_slice()
                                .iter()
                                .position(|x| x.to_string() == value_str)
                            {
                                new_val.remove(idx);
                            }
                        }
                        _ => {
                            godot_warn!("unsupported action for packed string array");
                            continue;
                        }
                    }
                    self.update(
                        subparts[0].into(),
                        new_val.to_variant(),
                        VarDictionary::new(),
                        false,
                    );
                }
                _ => {
                    godot_warn!("unsupported type for expression");
                    continue;
                }
            }
        }
    }

    #[func]
    fn updates(&mut self, data: VarDictionary, metas: VarDictionary, force: bool) {
        let keys = data.keys_array();
        for i in 0..keys.len() {
            let Some(key) = keys.get(i) else { continue };
            let key: GString = key.to();
            let key_var = key.to_variant();
            let value = data.get_or_nil(&key_var);
            self.update(key, value, metas.clone(), force);
        }
    }

    #[func]
    fn update(&mut self, key: GString, value: Variant, metas: VarDictionary, force: bool) {
        if self.core.is_none() {
            return;
        }

        let key_str = key.to_string();
        let parts: Vec<&str> = key_str.split(';').collect();
        let label = parts[0];
        let label_var = GString::from(label).to_variant();

        if !self.propers.contains(&label_var) {
            godot_warn!("{} not in propers", label);
            return;
        }

        if parts.len() == 1 {
            let current = self.base_mut().call("get", &[key.clone().to_variant()]);
            if !force && current == value {
                return;
            }
            self.base_mut()
                .call("set", &[key.clone().to_variant(), value.clone()]);
        } else {
            let mut curr = self
                .base_mut()
                .call("get", &[GString::from(parts[0]).to_variant()]);
            for i in 1..parts.len() - 1 {
                curr = curr.call("get", &[GString::from(parts[i]).to_variant()]);
            }
            let last_key = GString::from(parts[parts.len() - 1]);
            let leaf = curr.call("get", &[last_key.clone().to_variant()]);

            if leaf.get_type() == godot::builtin::VariantType::ARRAY {
                if let Ok(mut arr) = leaf.try_to::<VarArray>() {
                    arr.push(&value);
                }
            } else if curr.get_type() == godot::builtin::VariantType::DICTIONARY {
                if let Ok(mut dict) = curr.try_to::<VarDictionary>() {
                    if !force && dict.get_or_nil(&last_key.to_variant()) == value {
                        return;
                    }
                    dict.set(&last_key.to_variant(), &value);
                }
            } else {
                let has_insert: bool = variant_has_method(&curr, "insert");
                if has_insert {
                    let existing = curr.call(
                        "get",
                        &[last_key.clone().to_variant(), Variant::nil()],
                    );
                    if !force && existing == value {
                        return;
                    }
                    curr.call("insert", &[last_key.to_variant(), value.clone()]);
                } else {
                    godot_warn!(";only set array or dictionary");
                    return;
                }
            }
        }

        let key_prefix = format!("{};{}", self.prefix, key);
        let has_to_json: bool = variant_has_method(&value, "to_json");
        {
            let mut core_gd = self.core.as_ref().unwrap().clone();
            if has_to_json {
                let json_val = value.call("to_json", &[]);
                core_gd.call(
                    "update",
                    &[
                        key_prefix.to_variant(),
                        GString::from("~").to_variant(),
                        json_val,
                        self.scope.to_variant(),
                    ],
                );
            } else {
                core_gd.call(
                    "update",
                    &[
                        key_prefix.to_variant(),
                        GString::from("~").to_variant(),
                        value.clone(),
                        self.scope.to_variant(),
                    ],
                );
            }
        }

        if let Some(callbacks) = self.callback_map.get_mut(&key_str) {
            callbacks.retain(|cb| cb.is_valid());
            for cb in callbacks.iter() {
                cb.call(&[value.clone()]);
                cb.call(&[value.clone(), metas.clone().to_variant()]);
            }
        }

        self.emit_node_text(key.clone());
        self.update_all_property();

        let has_on_change: bool = self
            .base_mut()
            .call("has_method", &["on_change".to_variant()])
            .to();
        if has_on_change {
            self.base_mut()
                .call("on_change", &[key.to_variant(), value]);
        }
    }

    #[func]
    fn get_value_by_key(&mut self, key: GString) -> Variant {
        let key_str = key.to_string();
        let parts: Vec<&str> = key_str.split(';').collect();
        let label = parts[0];
        let label_var = GString::from(label).to_variant();

        if !self.propers.contains(&label_var) {
            godot_print!("{} not in propers", label);
            return Variant::nil();
        }

        if parts.len() == 1 {
            return self.base_mut().call("get", &[key.to_variant()]);
        }

        let mut curr = self
            .base_mut()
            .call("get", &[GString::from(parts[0]).to_variant()]);
        for i in 1..parts.len() - 1 {
            curr = curr.call("get", &[GString::from(parts[i]).to_variant()]);
        }

        let last = parts[parts.len() - 1];
        if curr.get_type() == godot::builtin::VariantType::ARRAY {
            if let Ok(arr) = curr.try_to::<VarArray>() {
                if let Ok(idx) = last.parse::<i64>() {
                    if idx >= 0 && idx < arr.len() as i64 {
                        return arr.get(idx as usize).unwrap_or(Variant::nil());
                    }
                }
            }
            Variant::nil()
        } else if curr.get_type() == godot::builtin::VariantType::DICTIONARY {
            if let Ok(dict) = curr.try_to::<VarDictionary>() {
                dict.get_or_nil(&GString::from(last).to_variant())
            } else {
                Variant::nil()
            }
        } else {
            curr.call("get", &[GString::from(last).to_variant()])
        }
    }

    #[func]
    fn watch(&mut self, key: GString, callback: Callable) {
        let cbid = callback.hash_u32();
        if self.registered_callback.contains_key(&cbid) {
            return;
        }

        let key_str = key.to_string();
        let parts: Vec<&str> = key_str.split(';').collect();
        let label = parts[0];
        let label_var = GString::from(label).to_variant();

        if !self.propers.contains(&label_var) {
            return;
        }

        if !self.callback_map.contains_key(&key_str) {
            self.callback_map.insert(key_str.clone(), Vec::new());
        }
        if let Some(callbacks) = self.callback_map.get_mut(&key.to_string()) {
            callbacks.push(callback.clone());
        }
        self.registered_callback.insert(cbid, true);

        let val = self.get_value_by_key(key.clone());
        callback.call(&[val.clone()]);
        callback.call(&[val, VarDictionary::new().to_variant()]);
    }

    #[func]
    fn on_ready(&self) {}

    #[func]
    fn switch_core(&mut self, new_core: Gd<GdCoreData>) {
        self.do_switch_core(new_core);
    }
}

impl GdBean {
    fn get_value_by_key_read(&mut self, key: GString) -> Variant {
        self.get_value_by_key(key)
    }

    fn update_all_property(&mut self) {
        let keys: Vec<String> = self.property_val_cache.keys().cloned().collect();
        for key in keys {
            self.check_property_val((&key).into());
        }
    }

    pub fn do_switch_core(&mut self, new_core: Gd<GdCoreData>) {
        self.core = Some(new_core.clone());

        {
            let mut core_gd = self.core.as_ref().unwrap().clone();
            core_gd.call(
                "change",
                &[
                    self.prefix.to_variant(),
                    GString::from("").to_variant(),
                    GString::from("{}").to_variant(),
                    self.scope.to_variant(),
                ],
            );
        }

        for i in 0..self.propers.len() {
            let Some(key) = self.propers.get(i) else { continue };
            let key: GString = key.to();
            let key_str = key.to_string();
            let key_prefix = format!("{};{}", self.prefix, key);

            let core_has: bool = {
                let mut core_gd = self.core.as_ref().unwrap().clone();
                core_gd.call("has", &[key_prefix.to_variant(), self.scope.to_variant()]).to()
            };

            if core_has {
                let current_val = self.base_mut().call("get", &[key.clone().to_variant()]);
                let has_from_json: bool = variant_has_method(&current_val, "from_json");
                if has_from_json {
                    let json_val = {
                        let mut core_gd = self.core.as_ref().unwrap().clone();
                        core_gd.call(
                            "value",
                            &[key_prefix.to_variant(), Variant::nil(), self.scope.to_variant()],
                        )
                    };
                    let new_val = current_val.call("from_json", &[json_val]);
                    self.base_mut().call("set", &[key.to_variant(), new_val]);
                } else {
                    let core_val = {
                        let mut core_gd = self.core.as_ref().unwrap().clone();
                        core_gd.call(
                            "value",
                            &[key_prefix.to_variant(), Variant::nil(), self.scope.to_variant()],
                        )
                    };
                    self.base_mut().call("set", &[key.to_variant(), core_val]);
                }
            } else {
                let default_val = self.initial_data.get_or_nil(&key.to_variant());
                let has_from_json: bool = variant_has_method(&default_val, "from_json");
                if has_from_json {
                    self.base_mut().call("set", &[key.to_variant(), default_val.clone()]);
                    let json_val = default_val.call("to_json", &[]);
                    let mut core_gd = self.core.as_ref().unwrap().clone();
                    core_gd.call(
                        "update",
                        &[
                            key_prefix.to_variant(),
                            GString::from("").to_variant(),
                            json_val,
                            self.scope.to_variant(),
                        ],
                    );
                } else {
                    self.base_mut().call("set", &[key.to_variant(), default_val.clone()]);
                    let mut core_gd = self.core.as_ref().unwrap().clone();
                    core_gd.call(
                        "update",
                        &[
                            key_prefix.to_variant(),
                            GString::from("").to_variant(),
                            default_val,
                            self.scope.to_variant(),
                        ],
                    );
                }
            }

            let val = self.get_value_by_key(key.clone());
            if let Some(callbacks) = self.callback_map.get_mut(&key_str) {
                callbacks.retain(|cb| cb.is_valid());
                for cb in callbacks.iter() {
                    cb.call(&[val.clone()]);
                    cb.call(&[val.clone(), VarDictionary::new().to_variant()]);
                }
            }
            self.emit_node_text(key);
        }

        self.update_all_property();

        let has_on_save_switch: bool = self
            .base_mut()
            .call("has_method", &["on_save_switch".to_variant()])
            .to();
        if has_on_save_switch {
            self.base_mut()
                .call("on_save_switch", &[]);
        }
    }

    pub fn clear_instances() {
        BEAN_INSTANCES.lock().clear();
    }
}

fn get_bean_by_id(bean_id: &str) -> Option<Gd<GdBean>> {
    let instances = BEAN_INSTANCES.lock();
    if let Some(&instance_id) = instances.get(bean_id) {
        Gd::<GdBean>::try_from_instance_id(InstanceId::from_i64(instance_id)).ok()
    } else {
        None
    }
}

pub fn get_all_bean_instances() -> Vec<(String, i64)> {
    BEAN_INSTANCES.lock().iter().map(|(k, v)| (k.clone(), *v)).collect()
}

fn variant_to_text(val: &Variant) -> GString {
    match val.get_type() {
        godot::builtin::VariantType::INT => {
            let v: i64 = val.to();
            GString::from(&v.to_string())
        }
        godot::builtin::VariantType::FLOAT => {
            let v: f64 = val.to();
            GString::from(&format!("{:.2}", v))
        }
        godot::builtin::VariantType::STRING => val.to::<GString>(),
        _ => Json::stringify(val),
    }
}
