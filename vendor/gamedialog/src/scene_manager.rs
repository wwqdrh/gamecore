// SceneManager: 场景管理器
//
// 管理多个 Timeline 和全局变量。
// 非单例设计，由 GdDialogue 或外部持有

use crate::timeline::Timeline;
use std::collections::HashMap;

/// 场景管理器，管理多个 Timeline 和全局变量
pub struct SceneManager {
    /// 名称→Timeline 映射
    timelines: HashMap<String, Timeline>,
    /// 当前活跃的 Timeline 名称
    current_timeline: String,
    /// 全局变量存储
    global_variables: HashMap<String, String>,
}

impl SceneManager {
    pub fn new() -> Self {
        SceneManager {
            timelines: HashMap::new(),
            current_timeline: String::new(),
            global_variables: HashMap::new(),
        }
    }

    /// 添加 Timeline
    pub fn add_timeline(&mut self, name: &str, data: &str) {
        let timeline = Timeline::new(data);
        self.timelines.insert(name.to_string(), timeline);
    }

    /// 设置当前 Timeline
    pub fn set_current_timeline(&mut self, name: &str) {
        if self.timelines.contains_key(name) {
            self.current_timeline = name.to_string();
        }
    }

    /// 获取当前 Timeline 的可变引用
    pub fn get_current_timeline(&mut self) -> Option<&mut Timeline> {
        if self.current_timeline.is_empty() {
            return None;
        }
        self.timelines.get_mut(&self.current_timeline)
    }

    /// 获取指定 Timeline 的可变引用
    pub fn get_timeline(&mut self, name: &str) -> Option<&mut Timeline> {
        self.timelines.get_mut(name)
    }

    /// 获取指定 Timeline 的不可变引用
    pub fn get_timeline_ref(&self, name: &str) -> Option<&Timeline> {
        self.timelines.get(name)
    }

    /// 设置全局变量
    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.global_variables.insert(key.to_string(), value.to_string());
    }

    /// 获取全局变量
    pub fn get_variable(&self, key: &str) -> String {
        self.global_variables.get(key).cloned().unwrap_or_default()
    }

    /// 检查全局变量是否存在
    pub fn has_variable(&self, key: &str) -> bool {
        self.global_variables.contains_key(key)
    }

    /// 获取所有全局变量
    pub fn get_all_variables(&self) -> Vec<String> {
        self.global_variables
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect()
    }

    /// 获取所有 Timeline 中满足条件的 stage 列表
    pub fn get_all_available_stages(&self) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        for (timeline_name, timeline) in &self.timelines {
            result.insert(timeline_name.clone(), timeline.get_available_stages());
        }
        result
    }
}
