// DiaStage: 场景阶段
//
// 核心解析和执行单元。一个 Stage 包含多条对话行和控制流指令。
// 支持脚本语法：stage标记、变量块、角色名列表、对话内容、
// 响应选项、函数调用、控制流、标签、条件入口

use crate::flow::ControlFlow;
use crate::word::DialogueWord;
use std::collections::HashMap;

/// 行变体：对话词条或控制流
#[derive(Debug, Clone)]
pub enum LineVariant {
    Word(DialogueWord),
    Flow(ControlFlow),
}

/// 条件表达式
#[derive(Debug, Clone)]
pub struct Condition {
    pub variable: String,
    pub op: String,
    pub value: String,
    pub is_global: bool,
}

/// 场景阶段，核心解析和执行单元
#[derive(Debug, Clone)]
pub struct DiaStage {
    /// stage 名称
    stage_name: String,
    /// flag 列表（用于 precheck 过滤）
    stage_flags: Vec<String>,
    /// 所有行（DialogueWord 或 ControlFlow）
    dialogue_keys: Vec<LineVariant>,
    /// 当前执行位置
    current: usize,
    /// 场景局部变量
    scene_variables: HashMap<String, String>,
    /// 标签→位置映射
    labels: HashMap<String, usize>,
    /// 入口条件列表
    entry_conditions: Vec<Condition>,
}

impl DiaStage {
    pub fn new() -> Self {
        DiaStage {
            stage_name: String::new(),
            stage_flags: Vec::new(),
            dialogue_keys: Vec::new(),
            current: 0,
            scene_variables: HashMap::new(),
            labels: HashMap::new(),
            entry_conditions: Vec::new(),
        }
    }

    /// 从文本数据初始化
    pub fn initial_from_str(&mut self, data: &str) {
        let lines: Vec<&str> = data.lines().collect();
        self.initial(&lines);
    }

    /// 从行列表初始化
    pub fn initial(&mut self, data: &[&str]) {
        let mut cur_names: Vec<String> = Vec::new();
        let mut cur_word: Vec<String> = Vec::new();
        let mut var_block = String::new();
        let mut in_var_block = false;

        for line in data {
            let trimmed = line.trim();

            // 跳过注释和空行
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // 处理变量块
            if *line == "```" {
                if !in_var_block {
                    in_var_block = true;
                    continue;
                } else {
                    in_var_block = false;
                    self.parse_variables(&var_block);
                    continue;
                }
            }

            if in_var_block {
                var_block.push_str(line);
                var_block.push('\n');
                continue;
            }

            // 处理场景标记 [scene], 可能存在标记，例如[stage@flag1;flag2]
            if line.starts_with('[') && line.ends_with(']') {
                let inner = &line[1..line.len() - 1];
                let stage_parts: Vec<&str> = inner.split('@').collect();
                self.stage_name = stage_parts[0].to_string();
                if stage_parts.len() == 2 {
                    self.stage_flags = stage_parts[1].split(';').map(|s| s.to_string()).collect();
                }
            }
            // 处理角色名称 (name1,name2)
            else if line.starts_with('(') && line.ends_with(')') {
                if !cur_names.is_empty() && !cur_word.is_empty() {
                    self.parse_section(&cur_names, &cur_word);
                    cur_names.clear();
                    cur_word.clear();
                }
                let names_str = &line[1..line.len() - 1];
                cur_names = names_str.split(',').map(|s| s.to_string()).collect();
            }
            // 处理对话内容
            else {
                cur_word.push(line.to_string());
            }
        }

        // 处理最后一段对话
        if !cur_word.is_empty() {
            if cur_names.is_empty() {
                self.parse_section(&[String::new()], &cur_word);
            } else {
                self.parse_section(&cur_names, &cur_word);
            }
        }
    }

    /// 解析一组对话行
    fn parse_section(&mut self, names: &[String], words: &[String]) {
        let mut cur_index = 0;
        let mut cur_key_index: Option<usize> = None;

        for word in words {
            // 跳过注释和空行
            if word.starts_with('#') || word.is_empty() {
                continue;
            }

            // 处理响应选项
            if word.starts_with('-') {
                if let Some(idx) = cur_key_index {
                    let response = &word[1..];
                    let parts: Vec<&str> = response.split('@').collect();
                    if parts.len() == 2 {
                        if let Some(LineVariant::Word(ref mut w)) = self.dialogue_keys.get_mut(idx) {
                            w.add_response(parts[0], parts[1]);
                        }
                    }
                }
            }
            // 处理函数调用
            else if word.starts_with('@') {
                if let Some(idx) = cur_key_index {
                    let fn_expr = &word[1..];
                    // 先收集 label 信息，避免借用冲突
                    let label_to_set = fn_expr.strip_prefix("label:").map(|l| l.trim().to_string());
                    if let Some(label) = label_to_set {
                        self.set_label(&label, self.dialogue_keys.len());
                    }
                    if let Some(LineVariant::Word(ref mut w)) = self.dialogue_keys.get_mut(idx) {
                        w.add_fn(fn_expr);
                    }
                }
            }
            // 控制流程
            else if word.starts_with(':') {
                if let Some(mut flow) = ControlFlow::create_from_string(word) {
                    flow.set_stage_name(&self.stage_name);
                    self.dialogue_keys.push(LineVariant::Flow(flow));
                }
            }
            // 处理普通对话
            else {
                let mut new_word = DialogueWord::new();
                new_word.set_stage(&self.stage_name);
                new_word.set_name(&names[cur_index]);

                let text = word.strip_suffix('+').unwrap_or(word);
                new_word.set_text(text);
                self.dialogue_keys.push(LineVariant::Word(new_word));

                if !word.ends_with('+') {
                    cur_index = (cur_index + 1) % names.len();
                }

                // 记录最后添加的 word 索引，用于后续添加 response/fn
                cur_key_index = Some(self.dialogue_keys.len() - 1);
            }
        }
    }

    /// 解析变量块
    fn parse_variables(&mut self, var_block: &str) {
        for line in var_block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with('?') {
                self.parse_condition_expression(&trimmed[1..]);
                continue;
            }

            // 变量解析 key=value
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                self.scene_variables.insert(
                    parts[0].trim().to_string(),
                    parts[1].trim().to_string(),
                );
            }
        }
    }

    /// 解析条件表达式
    fn parse_condition_expression(&mut self, expr: &str) {
        let conditions = expr.split('&');

        for cond in conditions {
            let cond = cond.trim();
            if cond.is_empty() {
                continue;
            }

            // 查找运算符位置（优先匹配 >= 和 <=）
            let (op_pos, op) = if let Some(pos) = cond.find(">=") {
                (pos, ">=")
            } else if let Some(pos) = cond.find("<=") {
                (pos, "<=")
            } else if let Some(pos) = cond.find('>') {
                (pos, ">")
            } else if let Some(pos) = cond.find('<') {
                (pos, "<")
            } else if let Some(pos) = cond.find('=') {
                (pos, "=")
            } else {
                continue; // 无效条件
            };

            let var_name = cond[..op_pos].trim();
            let value = cond[op_pos + op.len()..].trim();

            let (variable, is_global) = if let Some(stripped) = var_name.strip_prefix("global.") {
                (stripped.to_string(), true)
            } else {
                (var_name.to_string(), false)
            };

            self.entry_conditions.push(Condition {
                variable,
                op: op.to_string(),
                value: value.to_string(),
                is_global,
            });
        }
    }

    pub fn get_stage_name(&self) -> &str {
        &self.stage_name
    }

    pub fn get_line_size(&self) -> usize {
        self.dialogue_keys.len()
    }

    pub fn get_flags(&self) -> &[String] {
        &self.stage_flags
    }

    pub fn has_next(&self) -> bool {
        if self.current >= self.dialogue_keys.len() {
            return false;
        }
        // 如果当前是控制流，需要检查其 hasNext
        if let LineVariant::Flow(ref flow) = self.dialogue_keys[self.current] {
            // hasNext 需要访问 Timeline，这里简化处理：
            // Start/Skip 始终 true，End 始终 false，Goto 需要外部检查
            return match flow {
                ControlFlow::End { .. } => false,
                ControlFlow::Goto { .. } => true, // 简化：Goto 默认有 next
                _ => true,
            };
        }
        true
    }

    /// 获取下一个 DialogueWord，返回 None 表示遇到了控制流
    /// 返回 (Some(word), stage_changed) 或 (None, stage_changed)
    pub fn next(&mut self) -> (Option<DialogueWord>, bool) {
        if self.current >= self.dialogue_keys.len() {
            return (None, false);
        }

        match &self.dialogue_keys[self.current] {
            LineVariant::Word(_) => {
                if let LineVariant::Word(word) = self.dialogue_keys[self.current].clone() {
                    self.current += 1;
                    // 执行所有函数
                    let functions = word.get_functions().to_vec();
                    for fns in &functions {
                        if let Some(rest) = fns.strip_prefix("set:") {
                            let parts: Vec<&str> = rest.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                self.set_variable(parts[0].trim(), parts[1].trim());
                            }
                        } else if let Some(expr) = fns.strip_prefix("if:") {
                            let parts: Vec<&str> = expr.split(':').collect();
                            if parts.len() == 3 {
                                let conditions = parts[0].trim();
                                let true_label = parts[1].trim();
                                let false_label = parts[2].trim();

                                if self.check_conditions(conditions) {
                                    self.goto_label(true_label);
                                } else {
                                    self.goto_label(false_label);
                                }
                            }
                        } else if let Some(label) = fns.strip_prefix("goto:") {
                            self.goto_label(label.trim());
                        }
                    }
                    return (Some(word), false);
                }
                (None, false)
            }
            LineVariant::Flow(_) => {
                self.current += 1;
                // 控制流由 Timeline 执行，这里只返回 None
                // 实际的 flow 执行在 Timeline::next() 中处理
                (None, false)
            }
        }
    }

    /// 获取当前行的控制流（如果有的话），供 Timeline 执行
    pub fn current_flow(&self) -> Option<&ControlFlow> {
        if self.current >= self.dialogue_keys.len() {
            return None;
        }
        if let LineVariant::Flow(ref flow) = self.dialogue_keys[self.current] {
            Some(flow)
        } else {
            None
        }
    }

    pub fn clean(&mut self) {
        self.current = 0;
    }

    pub fn is_start(&self) -> bool {
        self.current == 0
    }

    pub fn is_doing(&self) -> bool {
        self.current > 0 && self.current < self.dialogue_keys.len()
    }

    pub fn get_variables(&self) -> &HashMap<String, String> {
        &self.scene_variables
    }

    pub fn get_variable(&self, key: &str) -> String {
        self.scene_variables.get(key).cloned().unwrap_or_default()
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.scene_variables.insert(key.to_string(), value.to_string());
    }

    pub fn set_label(&mut self, label: &str, position: usize) {
        self.labels.insert(label.to_string(), position);
    }

    pub fn goto_label(&mut self, label: &str) -> bool {
        if let Some(&pos) = self.labels.get(label) {
            self.current = pos;
            true
        } else {
            false
        }
    }

    /// 检查条件表达式（& 分隔多条件，= 比较）
    fn check_conditions(&self, conditions: &str) -> bool {
        for pair in conditions.split('&') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() != 2 {
                continue;
            }
            let var = kv[0].trim();
            let expected = kv[1].trim();
            let actual = self.get_variable(var);
            if actual != expected {
                return false;
            }
        }
        true
    }

    /// 检查所有入口条件是否满足
    pub fn check_entry_conditions(&self, global_getter: &dyn Fn(&str) -> String) -> bool {
        for cond in &self.entry_conditions {
            if !self.evaluate_condition(cond, global_getter) {
                return false;
            }
        }
        true
    }

    fn evaluate_condition(&self, cond: &Condition, global_getter: &dyn Fn(&str) -> String) -> bool {
        let actual = if cond.is_global {
            global_getter(&cond.variable)
        } else {
            self.get_variable(&cond.variable)
        };

        if cond.op == "=" {
            return actual == cond.value;
        }

        // 尝试数值比较
        let actual_num = match actual.parse::<f64>() {
            Ok(n) => n,
            Err(_) => return false,
        };
        let value_num = match cond.value.parse::<f64>() {
            Ok(n) => n,
            Err(_) => return false,
        };

        match cond.op.as_str() {
            ">" => actual_num > value_num,
            "<" => actual_num < value_num,
            ">=" => actual_num >= value_num,
            "<=" => actual_num <= value_num,
            _ => false,
        }
    }
}
