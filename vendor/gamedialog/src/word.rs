// DialogueWord: 对话词条
//
// 表示一句对话/一个词条，包含说话人、文本、选项列表和触发函数列表

/// 对话词条，表示一句对话
#[derive(Debug, Clone)]
pub struct DialogueWord {
    /// 说话者名称
    name: String,
    /// 对话文本
    text: String,
    /// 所属 stage 名称
    stage: String,
    /// 选项列表 (选项文本, 跳转目标)
    responses: Vec<(String, String)>,
    /// 函数调用列表（如 "set:age=31"、"if:cond:true:false"、"goto:label"）
    functions: Vec<String>,
}

impl DialogueWord {
    pub fn new() -> Self {
        DialogueWord {
            name: String::new(),
            text: String::new(),
            stage: String::new(),
            responses: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn set_stage(&mut self, s: &str) {
        self.stage = s.to_string();
    }

    pub fn set_name(&mut self, n: &str) {
        self.name = n.to_string();
    }

    pub fn set_text(&mut self, t: &str) {
        self.text = t.to_string();
    }

    pub fn add_response(&mut self, response_text: &str, target: &str) {
        self.responses.push((response_text.to_string(), target.to_string()));
    }

    pub fn add_fn(&mut self, function_name: &str) {
        self.functions.push(function_name.to_string());
    }

    pub fn get_stage(&self) -> &str {
        &self.stage
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn get_responses(&self) -> &[(String, String)] {
        &self.responses
    }

    pub fn get_functions(&self) -> &[String] {
        &self.functions
    }
}
