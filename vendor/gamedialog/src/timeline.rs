// Timeline: 时间线
//
// 管理一组 DiaStage 的有序序列，提供全局导航。
// 支持 precheck 回调（flag 过滤）、stage 跳转、控制流执行等

use crate::flow::ControlFlow;
use crate::stage::DiaStage;
use crate::word::DialogueWord;
use std::collections::HashMap;

/// 预检查回调类型：接收 flag 表达式，返回是否通过
pub type PrecheckFn = Box<dyn Fn(&str) -> bool>;

/// 时间线，管理 DiaStage 序列
pub struct Timeline {
    /// stage 列表
    stages: Vec<DiaStage>,
    /// 当前 stage 索引
    current: usize,
    /// stage 名→索引映射
    stage_map: HashMap<String, usize>,
    /// 预检查回调
    precheck: Option<PrecheckFn>,
}

impl Timeline {
    /// 从文本数据创建 Timeline
    pub fn new(data: &str) -> Self {
        let mut timeline = Timeline {
            stages: Vec::new(),
            current: 0,
            stage_map: HashMap::new(),
            precheck: None,
        };
        timeline.parse(data);
        timeline
    }

    fn parse(&mut self, data: &str) {
        let mut cur_sections: Vec<String> = Vec::new();
        let mut tt = String::new();

        for line in data.lines() {
            if line.is_empty() {
                continue;
            }

            // 处理场景标记 [scene]
            if line.starts_with('[') && line.ends_with(']') {
                if !cur_sections.is_empty() && !tt.is_empty() {
                    let mut stage = DiaStage::new();
                    let section_refs: Vec<&str> = cur_sections.iter().map(|s| s.as_str()).collect();
                    stage.initial(&section_refs);
                    let name = stage.get_stage_name().to_string();
                    self.stages.push(stage);
                    self.stage_map.insert(name, self.stages.len() - 1);
                    cur_sections.clear();
                    tt.clear();
                }
                tt = line[1..line.len() - 1].to_string();
            }
            cur_sections.push(line.to_string());
        }

        // 处理最后一段对话
        if !cur_sections.is_empty() {
            let mut stage = DiaStage::new();
            let section_refs: Vec<&str> = cur_sections.iter().map(|s| s.as_str()).collect();
            stage.initial(&section_refs);
            let name = stage.get_stage_name().to_string();
            self.stages.push(stage);
            self.stage_map.insert(name, self.stages.len() - 1);
        }
    }

    pub fn set_precheck(&mut self, fn_box: PrecheckFn) {
        self.precheck = Some(fn_box);
    }

    /// 获取下一个 DialogueWord
    pub fn next(&mut self) -> Option<DialogueWord> {
        // 如果是刚开始，寻找满足 flag 的第一个 stage
        if self.current == 0 && !self.stages.is_empty() && self.stages[self.current].is_start() {
            self.get_first_flag();
        } else if !self.check_stage_flag(&self.current_stage()) {
            self.goto_end();
        }

        if !self.has_next() {
            return None;
        }

        let prev_stage = self.current_stage();
        let result = self.stages[self.current].next();

        match result {
            (Some(word), _) => Some(word),
            (None, _) => {
                // 可能是控制流，检查当前行是否是 flow
                let flow = self.stages[self.current].current_flow().cloned();
                if let Some(flow) = flow {
                    self.execute_flow(&flow);
                    let cur_stage = self.current_stage();
                    if prev_stage != cur_stage {
                        return self.next();
                    }
                    return None;
                }
                // stage 内部 next 返回 None 但不是 flow，尝试下一个 stage
                None
            }
        }
    }

    /// 执行控制流
    fn execute_flow(&mut self, flow: &ControlFlow) {
        match flow {
            ControlFlow::Start { .. } => self.goto_begin(),
            ControlFlow::End { .. } => self.goto_end(),
            ControlFlow::Skip { skip_count, .. } => self.skip_stage_count(*skip_count),
            ControlFlow::Goto { target_name, .. } => self.goto_stage(target_name),
        }
    }

    pub fn has_next(&self) -> bool {
        if self.current >= self.stages.len() {
            return false;
        }
        self.stages[self.current].has_next()
    }

    pub fn goto_stage(&mut self, stage: &str) {
        if !self.check_stage_flag(stage) {
            return;
        }
        if let Some(&idx) = self.stage_map.get(stage) {
            if self.current < self.stages.len() {
                self.stages[self.current].clean();
            }
            self.current = idx;
        }
    }

    pub fn goto_begin(&mut self) {
        if self.current < self.stages.len() {
            self.stages[self.current].clean();
            self.current = 0;
        }
    }

    pub fn goto_end(&mut self) {
        if self.current < self.stages.len() {
            self.stages[self.current].clean();
        }
        self.current = self.stages.len();
    }

    pub fn skip_stage_count(&mut self, count: i32) {
        if self.current < self.stages.len() {
            self.stages[self.current].clean();
            self.current = (self.current as i32 + count) as usize;
            if self.current > self.stages.len() - 1 {
                self.current = self.stages.len();
            }
        }
    }

    /// 找到第一个满足 flag 条件的 stage 并跳转过去
    pub fn get_first_flag(&mut self) {
        if self.precheck.is_none() {
            return;
        }

        // 先收集满足条件的 stage 名称
        let target = self.stages.iter()
            .find(|s| self.check_stage_flag(s.get_stage_name()))
            .map(|s| s.get_stage_name().to_string());

        if let Some(name) = target {
            self.goto_stage(&name);
        } else {
            // 没有一个满足条件的，直接 end
            self.goto_end();
        }
    }

    /// 检查 stage 的所有 flags 是否都通过 precheck
    pub fn check_stage_flag(&self, stage: &str) -> bool {
        if self.precheck.is_none() {
            return true;
        }

        let idx = match self.stage_map.get(stage) {
            Some(&i) => i,
            None => return false,
        };

        let stage_data = &self.stages[idx];
        for flag in stage_data.get_flags() {
            if let Some(ref fn_box) = self.precheck {
                if !fn_box(flag) {
                    return false;
                }
            }
        }
        true
    }

    pub fn all_stages(&self) -> Vec<String> {
        self.stages.iter().map(|s| s.get_stage_name().to_string()).collect()
    }

    pub fn has_stage(&self, name: &str) -> bool {
        self.stage_map.contains_key(name)
    }

    pub fn stage_index(&self, label: &str) -> i32 {
        match self.stage_map.get(label) {
            Some(&idx) => idx as i32,
            None => -1,
        }
    }

    pub fn current_stage(&self) -> String {
        if self.current >= self.stages.len() {
            return String::new();
        }
        self.stages[self.current].get_stage_name().to_string()
    }

    pub fn current_stage_is_doing(&self) -> bool {
        if self.current >= self.stages.len() {
            return false;
        }
        self.stages[self.current].is_doing()
    }

    pub fn get_available_stages(&self) -> Vec<String> {
        self.stages
            .iter()
            .filter(|s| self.check_stage_flag(s.get_stage_name()))
            .map(|s| s.get_stage_name().to_string())
            .collect()
    }

    /// 检查入口条件（需要全局变量获取器）
    pub fn check_entry_conditions(&self, stage: &str, global_getter: &dyn Fn(&str) -> String) -> bool {
        let idx = match self.stage_map.get(stage) {
            Some(&i) => i,
            None => return false,
        };
        self.stages[idx].check_entry_conditions(global_getter)
    }
}
