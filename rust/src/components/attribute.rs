// GdAttributeSet - 属性组件
// 力量、耐力等任意数值/字符串属性（RPG 原型用），
// 每次修改触发 s_attribute_changed(key, old_value, new_value)。

use std::collections::HashMap;

use godot::prelude::*;
use godot::builtin::{GString, Variant};
use godot::classes::{INode, Node};

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdAttributeSet {
    attrs: HashMap<String, Variant>,
    base: Base<Node>,
}

#[godot_api]
impl INode for GdAttributeSet {
    fn init(base: Base<Node>) -> Self {
        Self {
            attrs: HashMap::new(),
            base,
        }
    }
}

#[godot_api]
impl GdAttributeSet {
    /// 属性变更信号
    #[signal]
    fn s_attribute_changed(key: GString, old_value: Variant, new_value: Variant);

    /// 设置属性（新增或覆盖），返回变更信号是否已发出
    #[func]
    pub fn set_attribute(&mut self, key: GString, value: Variant) -> bool {
        let old = self.attrs.insert(key.to_string(), value.clone());
        let changed = old.as_ref().map_or(true, |o| *o != value);
        if changed {
            let nil = Variant::nil();
            let o = old.unwrap_or(nil);
            self.base_mut().emit_signal(
                "s_attribute_changed",
                &[key.to_variant(), o, value],
            );
        }
        changed
    }

    /// 读取属性（不存在返回 default）
    #[func]
    pub fn get_attribute(&self, key: GString, default: Variant) -> Variant {
        self.attrs
            .get(&key.to_string())
            .cloned()
            .unwrap_or(default)
    }

    /// 数值属性的便捷读取（int/float 均可，不存在或类型不符返回 default）
    #[func]
    pub fn get_attribute_float(&self, key: GString, default: f64) -> f64 {
        match self.attrs.get(&key.to_string()) {
            Some(v) => match v.get_type() {
                godot::builtin::VariantType::INT => {
                    v.try_to::<i64>().map(|i| i as f64).unwrap_or(default)
                }
                godot::builtin::VariantType::FLOAT => v.try_to::<f64>().unwrap_or(default),
                _ => default,
            },
            None => default,
        }
    }

    #[func]
    pub fn has_attribute(&self, key: GString) -> bool {
        self.attrs.contains_key(&key.to_string())
    }

    /// 删除属性，返回删除的值
    #[func]
    pub fn erase_attribute(&mut self, key: GString) -> Variant {
        self.attrs
            .remove(&key.to_string())
            .unwrap_or_else(Variant::nil)
    }

    /// 全部属性（GDScript Dictionary）
    #[func]
    pub fn get_all_attributes(&self) -> VarDictionary {
        let mut dict = VarDictionary::new();
        for (k, v) in &self.attrs {
            let key = GString::from(k.as_str());
            dict.set(&key, v);
        }
        dict
    }

    /// 属性数量
    #[func]
    pub fn attribute_count(&self) -> i64 {
        self.attrs.len() as i64
    }
}
