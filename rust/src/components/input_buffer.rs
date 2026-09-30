// GdInputBuffer - 输入缓冲组件
// 平台/动作游戏原型必备：玩家在落地前一小段窗口按下的跳跃/攻击
// 不丢失，落地瞬间 consume() 消费执行。
//
// 不重写 Godot 输入映射，只包装检测窗口：
//   - watch_action("jump")：开始监听某动作的 just_pressed
//   - consume("jump") -> bool：窗口内有效则消费并返回 true
//   - is_buffered("jump") / clear_buffer("jump")

use std::collections::HashMap;

use godot::prelude::*;
use godot::builtin::{GString, PackedStringArray, StringName};
use godot::classes::{INode, Input, InputMap, Node, Time};

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdInputBuffer {
    /// 缓冲窗口时长（秒）：按下后多久内仍可消费
    #[export]
    buffer_window: f64,

    // ---- 运行时状态 ----
    /// 动作 -> 按下时刻（Time 秒钟），不存在键 = 未监听或已消费
    buffered: HashMap<String, f64>,
    base: Base<Node>,
}

#[godot_api]
impl INode for GdInputBuffer {
    fn init(base: Base<Node>) -> Self {
        Self {
            buffer_window: 0.15,
            buffered: HashMap::new(),
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        // 检测所有已监听动作的 just_pressed，记录时间戳
        let input = Input::singleton();
        for key in self.buffered.keys().cloned().collect::<Vec<_>>() {
            let gs = GString::from(key.as_str());
            let sn = StringName::from(gs.to_string().as_str());
            if input.is_action_just_pressed(&sn) {
                self.buffered.insert(key, Time::singleton().get_ticks_msec() as f64 / 1000.0);
            }
        }
        // 清理过期窗口
        let now = Time::singleton().get_ticks_msec() as f64 / 1000.0;
        self.buffered
            .retain(|_, t| now - *t <= self.buffer_window);
    }
}

#[godot_api]
impl GdInputBuffer {
    /// 开始监听某动作（必须是 InputMap 中已定义的动作）
    #[func]
    pub fn watch_action(&mut self, action: GString) {
        let sn = StringName::from(action.to_string().as_str());
        if !InputMap::singleton().has_action(&sn) {
            godot_error!(
                "[GdInputBuffer] watch_action: InputMap 中不存在动作 \"{}\"",
                action
            );
            return;
        }
        self.buffered.entry(action.to_string()).or_insert(-1.0);
    }

    /// 停止监听
    #[func]
    pub fn unwatch_action(&mut self, action: GString) {
        self.buffered.remove(&action.to_string());
    }

    /// 当前监听的动作列表
    #[func]
    pub fn watched_actions(&self) -> PackedStringArray {
        self.buffered
            .keys()
            .map(|k| GString::from(k.as_str()))
            .collect()
    }

    /// 外部注入"按下"事件（虚拟按键 / 测试用），与真实 just_pressed 等效
    #[func]
    pub fn notify_pressed(&mut self, action: GString) {
        if self.buffered.contains_key(&action.to_string()) {
            self.buffered
                .insert(action.to_string(), Time::singleton().get_ticks_msec() as f64 / 1000.0);
        }
    }

    /// 窗口内是否待消费
    #[func]
    pub fn is_buffered(&self, action: GString) -> bool {
        self.buffered
            .get(&action.to_string())
            .map_or(false, |t| *t >= 0.0)
    }

    /// 消费：窗口内有效则清除记录并返回 true（一次性）
    #[func]
    pub fn consume(&mut self, action: GString) -> bool {
        match self.buffered.get(&action.to_string()) {
            Some(t) if *t >= 0.0 => {
                self.buffered.insert(action.to_string(), -1.0);
                true
            }
            _ => false,
        }
    }

    /// 清除某动作的待消费记录
    #[func]
    pub fn clear_buffer(&mut self, action: GString) {
        if let Some(t) = self.buffered.get_mut(&action.to_string()) {
            *t = -1.0;
        }
    }

    /// 清空全部待消费记录（切场景/开门等防误触）
    #[func]
    pub fn clear_all(&mut self) {
        let keys: Vec<String> = self.buffered.keys().cloned().collect();
        for k in keys {
            self.buffered.insert(k, -1.0);
        }
    }
}
