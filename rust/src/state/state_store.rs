// GdState - 临时状态总线（非持久化）
// 参考 GdBean 的 watch 机制（键值 + 按键监听 + 注册即回调当前值），但不挂
// GdCoreData、不落盘、无 scope/prefix——专门用于 UI 区块间联动等"进程内临时
// 状态"。与 GdBean 的分工：
//   · GdBean  —— 游戏可持久化状态（存档，随 GDCORE 落盘恢复）
//   · GdState —— 临时状态（当前选中物品/当前页签等 UI 联动状态，进程内有效）
// 作为 Engine singleton "GDSTATE" 注册（Object 手动内存，与其他单例一致）。

use std::collections::HashMap;

use godot::prelude::*;
use godot::classes::{Engine, IObject, Object};

#[derive(GodotClass)]
#[class(base = Object)]
pub struct GdState {
    /// 键 -> 当前值
    values: HashMap<String, Variant>,
    /// 键 -> 监听回调列表
    watchers: HashMap<String, Vec<Callable>>,
    /// 回调 hash -> 注册的键（防重复注册 + unwatch 定位）
    registered: HashMap<u32, String>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GdState {
    fn init(base: Base<Object>) -> Self {
        Self {
            values: HashMap::new(),
            watchers: HashMap::new(),
            registered: HashMap::new(),
            base,
        }
    }
}

#[godot_api]
impl GdState {
    /// 写入状态并通知该键的全部监听者。
    /// 值未变化（Variant 深比较）时跳过通知——UI 写入方无需自行判重。
    #[func]
    fn set_state(&mut self, key: GString, value: Variant) {
        let key_str = key.to_string();
        if let Some(cur) = self.values.get(&key_str) {
            if *cur == value {
                return;
            }
        }
        self.values.insert(key_str.clone(), value.clone());
        Self::notify(&mut self.watchers, &key_str, value);
    }

    /// 读取状态（未设置返回 nil）
    #[func]
    fn get_state(&self, key: GString) -> Variant {
        self.values
            .get(&key.to_string())
            .cloned()
            .unwrap_or(Variant::nil())
    }

    /// 键是否已有值
    #[func]
    fn has_state(&self, key: GString) -> bool {
        self.values.contains_key(&key.to_string())
    }

    /// 移除键值（监听注册保留，写入方下次 set 仍会通知；不触发回调）
    #[func]
    fn erase_state(&mut self, key: GString) {
        self.values.remove(&key.to_string());
    }

    /// 注册键监听。与 GdBean.watch 语义一致：
    /// · 注册即用当前值回调一次（未设置时为 nil）——UI 初始填充零样板
    /// · 同一 Callable 重复注册全局去重
    /// · 回调参数：1 参收 value；2+ 参收 (value, key)
    #[func]
    fn watch(&mut self, key: GString, callback: Callable) {
        let key_str = key.to_string();
        let cbid = callback.hash_u32();
        if self.registered.contains_key(&cbid) {
            return;
        }
        self.watchers
            .entry(key_str.clone())
            .or_default()
            .push(callback.clone());
        self.registered.insert(cbid, key_str.clone());

        // 注册即回调当前值（GdBean 同语义）：监听方 _ready 里 watch 即完成初始同步
        let val = self
            .values
            .get(&key_str)
            .cloned()
            .unwrap_or(Variant::nil());
        Self::invoke(&callback, val, &key_str);
    }

    /// 取消监听（key/callable 与注册时不匹配时静默忽略）
    #[func]
    fn unwatch(&mut self, key: GString, callback: Callable) {
        let key_str = key.to_string();
        let cbid = callback.hash_u32();
        match self.registered.get(&cbid) {
            Some(registered_key) if *registered_key == key_str => {}
            _ => return,
        }
        if let Some(list) = self.watchers.get_mut(&key_str) {
            list.retain(|cb| cb.hash_u32() != cbid);
            if list.is_empty() {
                self.watchers.remove(&key_str);
            }
        }
        self.registered.remove(&cbid);
    }
}

impl GdState {
    /// 通知某键的全部监听者（先克隆回调列表再调用，允许回调内再写状态——重入安全）
    fn notify(
        watchers: &mut HashMap<String, Vec<Callable>>,
        key_str: &str,
        value: Variant,
    ) {
        let cbs: Vec<Callable> = match watchers.get_mut(key_str) {
            Some(list) => {
                list.retain(|cb| cb.is_valid());
                list.clone()
            }
            None => return,
        };
        for cb in cbs {
            Self::invoke(&cb, value.clone(), key_str);
        }
    }

    /// 回调参数约定：1 参 (value)；2+ 参 (value, key)
    fn invoke(cb: &Callable, value: Variant, key_str: &str) {
        let arg_count = cb.get_argument_count();
        let args = if arg_count >= 2 {
            vec![value, key_str.to_variant()]
        } else {
            vec![value]
        };
        cb.call(&args);
    }
}

pub fn register_gdstate_singleton() {
    let instance = Gd::<GdState>::from_init_fn(|base| GdState::init(base));
    let name = StringName::from("GDSTATE");
    Engine::singleton().register_singleton(&name, &instance);
    std::mem::forget(instance);
}

pub fn unregister_gdstate_singleton() {
    let name = StringName::from("GDSTATE");
    Engine::singleton().unregister_singleton(&name);
}
