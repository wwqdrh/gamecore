// GdDataLinkList - 基于字典的链表数据容器
// 内部结构为 Dictionary<String, Array>，每个 key 对应一个链表（用 Array 模拟）
// 支持序列化/反序列化（带 [GdDataLinkList] 前缀标记）

use godot::prelude::*;
use godot::classes::Json;
use godot::builtin::{VarArray, VarDictionary};

const LINKLIST_PREFIX: &str = "[GdDataLinkList]";

#[derive(GodotClass)]
#[class(init, base = Resource)]
pub struct GdDataLinkList {
    data: VarDictionary,

    base: Base<Resource>,
}

#[godot_api]
impl GdDataLinkList {
    #[func]
    fn from_json(&mut self, p_data: Variant) {
        self.data.clear();

        if let Ok(s) = p_data.try_to::<GString>() {
            let s = s.to_string();
            if let Some(trimmed) = s.strip_prefix(LINKLIST_PREFIX) {
                let json = Json::parse_string(trimmed);
                if let Ok(dict) = json.try_to::<VarDictionary>() {
                    let keys = dict.keys_array();
                    let values = dict.values_array();
                    for i in 0..keys.len() {
                        let key = keys.get(i);
                        let value = values.get(i);
                        if let (Some(key), Some(value)) = (key, value) {
                            if let Ok(arr) = value.try_to::<VarArray>() {
                                self.data.set(&key, &arr);
                            }
                        }
                    }
                }
            }
        }
    }

    #[func]
    fn to_json(&self) -> GString {
        let json_str = Json::stringify(&self.data.to_variant());
        let result = format!("{}{}", LINKLIST_PREFIX, json_str);
        GString::from(&result)
    }

    #[func]
    fn get_list(&self, key: GString) -> VarArray {
        let key_var = key.to_variant();
        self.data
            .get_or_nil(&key_var)
            .try_to::<VarArray>()
            .unwrap_or_default()
    }

    #[func]
    fn has(&self, key: GString) -> bool {
        let key_var = key.to_variant();
        if !self.data.contains_key(&key_var) {
            return false;
        }
        if let Ok(arr) = self.data.get_or_nil(&key_var).try_to::<VarArray>() {
            arr.len() > 0
        } else {
            false
        }
    }

    #[func]
    fn add_one(&mut self, key: GString, item: Variant) {
        let key_var = key.to_variant();
        if !self.data.contains_key(&key_var) {
            self.data.set(&key_var, &VarArray::new());
        }
        if let Ok(mut arr) = self.data.get_or_nil(&key_var).try_to::<VarArray>() {
            arr.push(&item);
        }
    }
}
