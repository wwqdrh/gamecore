// GdEventBus - 全局事件总线单例
// 注册为 Engine singleton "GDEVENTBUS"
//
// 解决跨场景广播要手动连信号、互相 get_node 拿引用的问题：
//   - publish / subscribe 全局解耦：玩家受伤、关卡完成、敌人死亡等
//   - 带参数（VariantArray）、一次性监听（once）、按 id 取消监听
//   - log_enabled 打开后在输出面板打印事件流水（原型调试用）
//
// GDScript 用法：
//   var bus = Engine.get_singleton("GDEVENTBUS")
//   var id = bus.subscribe("enemy_died", func(args): score += 10)
//   bus.publish("enemy_died", [enemy_id, 10])
//   bus.unsubscribe("enemy_died", id)

use std::collections::HashMap;

use godot::prelude::*;
use godot::builtin::{GString, StringName, VarArray, Variant};
use godot::classes::{Engine, IObject, Object};

/// 单条订阅
struct Subscription {
    id: u64,
    callable: Callable,
    once: bool,
}

#[derive(GodotClass)]
#[class(base = Object)]
pub struct GdEventBus {
    /// 事件名 -> 订阅列表（保序，主线程专用）
    subscribers: HashMap<String, Vec<Subscription>>,
    /// 订阅 id 分配器
    next_id: u64,
    /// 事件流水日志开关
    log_enabled: bool,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GdEventBus {
    fn init(base: Base<Object>) -> Self {
        Self {
            subscribers: HashMap::new(),
            next_id: 1,
            log_enabled: false,
            base,
        }
    }
}

#[godot_api]
impl GdEventBus {
    /// 订阅事件，返回订阅 id（用于取消订阅）
    #[func]
    pub fn subscribe(&mut self, event: GString, callback: Callable) -> i64 {
        self.add_subscription(event, callback, false)
    }

    /// 订阅一次性事件：触发一次后自动移除，返回订阅 id
    #[func]
    pub fn subscribe_once(&mut self, event: GString, callback: Callable) -> i64 {
        self.add_subscription(event, callback, true)
    }

    /// 取消订阅（按 subscribe 返回的 id），返回是否成功移除
    #[func]
    pub fn unsubscribe(&mut self, event: GString, id: i64) -> bool {
        let key = event.to_string();
        let Some(list) = self.subscribers.get_mut(&key) else {
            return false;
        };
        let before = list.len();
        list.retain(|s| s.id as i64 != id);
        let removed = list.len() != before;
        if list.is_empty() {
            self.subscribers.remove(&key);
        }
        removed
    }

    /// 发布事件：依次调用所有订阅者，参数以数组透传；
    /// once 订阅在触发后自动移除（本次发布的其他订阅者不受影响）
    #[func]
    pub fn publish(&mut self, event: GString, args: VarArray) {
        let key = event.to_string();
        if self.log_enabled {
            godot_print!(
                "[GdEventBus] publish \"{}\" args={} subscribers={}",
                key,
                args.len(),
                self.subscribers.get(&key).map_or(0, |l| l.len())
            );
        }
        let Some(list) = self.subscribers.get_mut(&key) else {
            return;
        };
        // 先收集调用，避免回调内部再订阅/退订导致迭代失效
        let calls: Vec<(Callable, u64, bool)> = list
            .iter()
            .map(|s| (s.callable.clone(), s.id, s.once))
            .collect();
        for (callable, id, once) in calls {
            if once {
                list.retain(|s| s.id != id);
            }
            if callable.is_valid() {
                callable.callv(&args);
            }
        }
        if list.is_empty() {
            self.subscribers.remove(&key);
        }
    }

    /// 清空某个事件的全部订阅（不传事件 = 清空所有）
    #[func]
    pub fn clear(&mut self, event: GString) {
        if event.is_empty() {
            self.subscribers.clear();
        } else {
            self.subscribers.remove(&event.to_string());
        }
    }

    /// 某个事件的订阅数量
    #[func]
    pub fn subscriber_count(&self, event: GString) -> i64 {
        self.subscribers
            .get(&event.to_string())
            .map_or(0, |l| l.len() as i64)
    }

    /// 开关事件流水日志（调试用）
    #[func]
    pub fn set_log_enabled(&mut self, enabled: bool) {
        self.log_enabled = enabled;
    }

    #[func]
    pub fn is_log_enabled(&self) -> bool {
        self.log_enabled
    }

    // ---- 内部实现 ----

    fn add_subscription(&mut self, event: GString, callback: Callable, once: bool) -> i64 {
        if !callback.is_valid() {
            godot_error!("[GdEventBus] subscribe: callback 无效（事件 {}）", event);
            return -1;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.subscribers
            .entry(event.to_string())
            .or_default()
            .push(Subscription {
                id,
                callable: callback,
                once,
            });
        id as i64
    }
}

/// 注册为 Engine singleton "GDEVENTBUS"（Object 手动内存，进程内永不回收）
pub fn register_gdeventbus_singleton() {
    let instance = Gd::<GdEventBus>::from_init_fn(|base| GdEventBus::init(base));
    let name = StringName::from("GDEVENTBUS");
    Engine::singleton().register_singleton(&name, &instance);
    std::mem::forget(instance);
}

pub fn unregister_gdeventbus_singleton() {
    let name = StringName::from("GDEVENTBUS");
    Engine::singleton().unregister_singleton(&name);
}
