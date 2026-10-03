// UI 标记语言解析器
// 将类 HTML 的 UI 描述文本解析为 AST 节点树
// 支持标签、属性、样式块、自闭合标签等语法

use std::collections::HashMap;

/// AST 节点：表示一个 UI 元素
#[derive(Debug, Clone)]
pub struct UiNode {
    /// 标签名（如 VBoxContainer, Button 等）
    pub tag: String,
    /// 属性列表（保持顺序）
    pub attributes: Vec<(String, String)>,
    /// 子节点
    pub children: Vec<UiNode>,
}

/// 样式规则：一个 CSS 类的样式定义
#[derive(Debug, Clone)]
pub struct StyleRule {
    /// 类名（不含点号）
    pub class_name: String,
    /// 属性键值对
    pub properties: HashMap<String, String>,
}

/// <script> 块中定义的数据值（JSON 风格字面量）
#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Str(String),
    Num(f64),
    Bool(bool),
    Null,
    Array(Vec<DataValue>),
    Dict(Vec<(String, DataValue)>),
}

impl DataValue {
    /// 便捷取值：数组元素个数
    pub fn array_len(&self) -> Option<usize> {
        match self {
            DataValue::Array(items) => Some(items.len()),
            _ => None,
        }
    }
}

/// 解析结果：包含根节点、样式规则和主题变量
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// 根节点
    pub root: UiNode,
    /// 样式规则列表
    pub styles: Vec<StyleRule>,
    /// 主题变量（来自 <theme> 块和内置主题）
    pub theme_vars: HashMap<String, String>,
    /// 主题名称（来自 <ui theme="xxx">）
    /// <script> 块定义的数据变量（供节点 data="变量名" 绑定）
    pub script_vars: HashMap<String, DataValue>,
    /// <ui script="xxx.gd"> 声明的脚本（构建期自动挂载到内容根节点）
    pub ui_script: Option<String>,
}

/// 解析错误
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

/// 标记语言解析器
pub struct UiParser {
    input: Vec<char>,
    pos: usize,
}

impl UiParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// 解析完整的 UI 标记文本
    pub fn parse(&mut self) -> Result<ParseResult, ParseError> {
        let mut styles = Vec::new();
        let mut theme_vars = HashMap::new();
        let mut script_vars = HashMap::new();
        let mut ui_script: Option<String> = None;
        let mut root_children = Vec::new();

        self.skip_whitespace_and_comments();

        // 期望根元素 <ui>
        if !self.expect_str("<ui") {
            return Err(ParseError {
                message: "Expected <ui> root element".to_string(),
                position: self.pos,
            });
        }

        // 解析 <ui> 的属性
        let ui_attrs = self.parse_attributes()?;

        // 提取 script 属性
        for (key, value) in &ui_attrs {
            if key == "script" {
                // <ui script="xxx.gd">：脚本随本文件构建结果自动挂载
                ui_script = Some(value.clone());
            }
        }

        self.skip_whitespace();

        // 期望 >
        if !self.expect_char('>') {
            return Err(ParseError {
                message: "Expected '>' after <ui> attributes".to_string(),
                position: self.pos,
            });
        }

        // 解析 <ui> 的子元素
        loop {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                return Err(ParseError {
                    message: "Unexpected end of input, expected </ui>".to_string(),
                    position: self.pos,
                });
            }

            // 检查 </ui>
            if self.expect_str("</ui") {
                self.skip_whitespace();
                if !self.expect_char('>') {
                    return Err(ParseError {
                        message: "Expected '>' after </ui".to_string(),
                        position: self.pos,
                    });
                }
                break;
            }

            // 检查 <theme> 块
            if self.expect_str("<theme") {
                self.skip_whitespace();
                if !self.expect_char('>') {
                    return Err(ParseError {
                        message: "Expected '>' after <theme".to_string(),
                        position: self.pos,
                    });
                }
                let theme_content = self.read_until_close_tag("theme")?;
                let parsed_vars = crate::ui::ui_theme::parse_theme_block(&theme_content);
                theme_vars.extend(parsed_vars);
                continue;
            }

            // 检查 <style> 块
            if self.expect_str("<style") {
                self.skip_whitespace();
                if !self.expect_char('>') {
                    return Err(ParseError {
                        message: "Expected '>' after <style".to_string(),
                        position: self.pos,
                    });
                }
                let style_content = self.read_until_close_tag("style")?;
                let parsed_styles = parse_style_block(&style_content);
                styles.extend(parsed_styles);
                continue;
            }

            // 检查 <script> 块（数据变量定义，如 var tasks = [...]）
            if self.expect_str("<script") {
                self.skip_whitespace();
                if !self.expect_char('>') {
                    return Err(ParseError {
                        message: "Expected '>' after <script".to_string(),
                        position: self.pos,
                    });
                }
                let script_content = self.read_until_close_tag("script")?;
                let parsed_vars = parse_script_block(&script_content)
                    .map_err(|msg| ParseError { message: format!("<script> {}", msg), position: self.pos })?;
                script_vars.extend(parsed_vars);
                continue;
            }

            // 解析普通子节点
            let node = self.parse_node()?;
            root_children.push(node);
        }

        let root = UiNode {
            tag: "ui".to_string(),
            attributes: ui_attrs,
            children: root_children,
        };

        Ok(ParseResult { root, styles, theme_vars, script_vars, ui_script })
    }

    /// 解析一个节点（标签 + 属性 + 子节点）
    fn parse_node(&mut self) -> Result<UiNode, ParseError> {
        if !self.expect_char('<') {
            return Err(ParseError {
                message: "Expected '<' to start a tag".to_string(),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // 读取标签名
        let tag = self.read_tag_name()?;
        if tag.is_empty() {
            return Err(ParseError {
                message: "Empty tag name".to_string(),
                position: self.pos,
            });
        }

        // 解析属性
        let attributes = self.parse_attributes()?;

        self.skip_whitespace();

        // 检查自闭合标签 />
        if self.expect_str("/>") {
            return Ok(UiNode {
                tag,
                attributes,
                children: Vec::new(),
            });
        }

        // 期望 >
        if !self.expect_char('>') {
            return Err(ParseError {
                message: format!("Expected '>' or '/>' after tag '{}'", tag),
                position: self.pos,
            });
        }

        // 解析子节点
        let mut children = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                return Err(ParseError {
                    message: format!("Unexpected end of input, expected </{}>", tag),
                    position: self.pos,
                });
            }

            // 检查闭合标签 </tag>
            if self.expect_str("</") {
                self.skip_whitespace();
                let close_tag = self.read_tag_name()?;
                self.skip_whitespace();
                if !self.expect_char('>') {
                    return Err(ParseError {
                        message: format!("Expected '>' after </{}", close_tag),
                        position: self.pos,
                    });
                }
                if close_tag != tag {
                    return Err(ParseError {
                        message: format!("Mismatched tags: <{}> and </{}>", tag, close_tag),
                        position: self.pos,
                    });
                }
                break;
            }

            // 解析子节点
            let child = self.parse_node()?;
            children.push(child);
        }

        Ok(UiNode {
            tag,
            attributes,
            children,
        })
    }

    /// 解析属性列表
    fn parse_attributes(&mut self) -> Result<Vec<(String, String)>, ParseError> {
        let mut attrs = Vec::new();
        loop {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            // 检查是否到达标签结束
            let c = self.current_char();
            if c == '>' || (c == '/' && self.peek_char(1) == '>') {
                break;
            }

            // 读取属性名
            let name = self.read_attr_name()?;
            if name.is_empty() {
                break;
            }

            self.skip_whitespace();

            // 检查是否有 = 值
            if self.expect_char('=') {
                self.skip_whitespace();
                let value = self.read_attr_value()?;
                attrs.push((name, value));
            } else {
                // 布尔属性（无值）
                attrs.push((name, String::new()));
            }
        }
        Ok(attrs)
    }

    /// 读取标签名
    fn read_tag_name(&mut self) -> Result<String, ParseError> {
        let mut name = String::new();
        while !self.is_at_end() {
            let c = self.current_char();
            if c.is_alphanumeric() || c == '_' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        Ok(name)
    }

    /// 读取属性名（允许 @ 前缀，如 @pressed="_on_xxx" 信号绑定声明）
    fn read_attr_name(&mut self) -> Result<String, ParseError> {
        let mut name = String::new();
        while !self.is_at_end() {
            let c = self.current_char();
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '@' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        Ok(name)
    }

    /// 读取属性值（支持单引号、双引号、无引号）
    fn read_attr_value(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace();
        if self.is_at_end() {
            return Err(ParseError {
                message: "Unexpected end of input while reading attribute value".to_string(),
                position: self.pos,
            });
        }

        let c = self.current_char();
        if c == '"' || c == '\'' {
            let quote = c;
            self.advance(); // 跳过引号
            let mut value = String::new();
            while !self.is_at_end() {
                let ch = self.current_char();
                if ch == quote {
                    self.advance(); // 跳过闭合引号
                    break;
                }
                value.push(ch);
                self.advance();
            }
            Ok(value)
        } else {
            // 无引号值，读到空格或 > 或 /
            let mut value = String::new();
            while !self.is_at_end() {
                let ch = self.current_char();
                if ch.is_whitespace() || ch == '>' || ch == '/' {
                    break;
                }
                value.push(ch);
                self.advance();
            }
            Ok(value)
        }
    }

    /// 读取直到遇到闭合标签 </tag>
    fn read_until_close_tag(&mut self, tag: &str) -> Result<String, ParseError> {
        let close_tag = format!("</{}>", tag);
        let close_chars: Vec<char> = close_tag.chars().collect();
        let mut content = String::new();

        while !self.is_at_end() {
            // 检查是否匹配闭合标签
            if self.pos + close_chars.len() <= self.input.len() {
                let slice: String = self.input[self.pos..self.pos + close_chars.len()].iter().collect();
                if slice == close_tag {
                    self.pos += close_chars.len();
                    return Ok(content);
                }
            }
            content.push(self.input[self.pos]);
            self.pos += 1;
        }

        Err(ParseError {
            message: format!("Unexpected end of input, expected </{}>", tag),
            position: self.pos,
        })
    }

    // === 辅助方法 ===

    fn current_char(&self) -> char {
        self.input[self.pos]
    }

    fn peek_char(&self, offset: usize) -> char {
        let idx = self.pos + offset;
        if idx < self.input.len() {
            self.input[idx]
        } else {
            '\0'
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn expect_char(&mut self, c: char) -> bool {
        if !self.is_at_end() && self.current_char() == c {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.pos + chars.len() > self.input.len() {
            return false;
        }
        for (i, &c) in chars.iter().enumerate() {
            if self.input[self.pos + i] != c {
                return false;
            }
        }
        self.pos += chars.len();
        true
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            // 跳过 <!-- --> 注释
            if self.pos + 4 <= self.input.len() {
                let slice: String = self.input[self.pos..self.pos + 4].iter().collect();
                if slice == "<!--" {
                    // 找到 -->
                    self.pos += 4;
                    while self.pos + 3 <= self.input.len() {
                        let end_slice: String = self.input[self.pos..self.pos + 3].iter().collect();
                        if end_slice == "-->" {
                            self.pos += 3;
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }
            }
            break;
        }
    }
}

/// 解析 <script> 块内容为数据变量表
///
/// 语法（JSON 风格字面量，宽松处理）：
/// ```text
/// // 行注释
/// var tasks = [
///   { icon: "🌿", title: "采集灵草", count: 3, done: false },
///   "纯字符串也可以",   // 尾逗号允许
/// ]
/// title = "单变量"      // var/let 关键字可选
/// ```
pub fn parse_script_block(content: &str) -> Result<HashMap<String, DataValue>, String> {
    let mut parser = ScriptParser {
        chars: content.chars().collect(),
        pos: 0,
    };
    parser.parse_declarations()
}

/// <script> 块解析器
struct ScriptParser {
    chars: Vec<char>,
    pos: usize,
}

impl ScriptParser {
    fn parse_declarations(&mut self) -> Result<HashMap<String, DataValue>, String> {
        let mut vars = HashMap::new();
        loop {
            self.skip_ws_comments();
            if self.is_at_end() {
                break;
            }
            // var / let 关键字可选
            if self.peek_word("var") || self.peek_word("let") {
                self.pos += 3;
                self.skip_ws_comments();
            }
            // 变量名
            let name = self.read_ident();
            if name.is_empty() {
                return Err(self.err_msg("期望变量名"));
            }
            self.skip_ws_comments();
            if !self.expect_char('=') || self.current_is('=') {
                return Err(self.err_msg(&format!("变量 '{}' 期望 '=' 赋值", name)));
            }
            self.skip_ws_comments();
            let value = self.parse_value()?;
            self.skip_ws_comments();
            self.expect_char(';'); // 结尾分号可选
            vars.insert(name, value);
        }
        Ok(vars)
    }

    /// 解析一个值：字符串 / 数字 / 布尔 / null / 数组 / 对象
    fn parse_value(&mut self) -> Result<DataValue, String> {
        self.skip_ws_comments();
        if self.is_at_end() {
            return Err(self.err_msg("期望值，但输入已结束"));
        }
        match self.current() {
            '"' | '\'' => Ok(DataValue::Str(self.parse_string()?)),
            c if c == '-' || c.is_ascii_digit() => self.parse_number(),
            c if c.is_alphabetic() || c == '_' => {
                let word = self.read_ident();
                match word.as_str() {
                    "true" => Ok(DataValue::Bool(true)),
                    "false" => Ok(DataValue::Bool(false)),
                    "null" => Ok(DataValue::Null),
                    other => Err(self.err_msg(&format!("未知字面量 '{}'", other))),
                }
            }
            '[' => self.parse_array(),
            '{' => self.parse_dict(),
            c => Err(self.err_msg(&format!("意外的字符 '{}'", c))),
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        let quote = self.current();
        self.pos += 1;
        let mut out = String::new();
        while !self.is_at_end() {
            let c = self.current();
            self.pos += 1;
            if c == quote {
                return Ok(out);
            }
            if c == '\\' && !self.is_at_end() {
                let esc = self.current();
                self.pos += 1;
                match esc {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    other => out.push(other), // \" \\ \' 原样还原
                }
            } else {
                out.push(c);
            }
        }
        Err(self.err_msg("字符串缺少闭合引号"))
    }

    fn parse_number(&mut self) -> Result<DataValue, String> {
        let start = self.pos;
        if self.current_is('-') {
            self.pos += 1;
        }
        while !self.is_at_end()
            && (self.current().is_ascii_digit()
                || matches!(self.current(), '.' | 'e' | 'E' | '+' | '-'))
        {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        text.parse::<f64>()
            .map(DataValue::Num)
            .map_err(|_| self.err_msg(&format!("非法数字 '{}'", text)))
    }

    fn parse_array(&mut self) -> Result<DataValue, String> {
        self.pos += 1; // 跳过 [
        let mut items = Vec::new();
        loop {
            self.skip_ws_comments();
            if self.is_at_end() {
                return Err(self.err_msg("数组缺少闭合 ']'"));
            }
            if self.current_is(']') {
                self.pos += 1;
                return Ok(DataValue::Array(items));
            }
            items.push(self.parse_value()?);
            self.skip_ws_comments();
            if self.current_is(',') {
                self.pos += 1; // 尾逗号允许：下一轮先检查 ']'
            } else if !self.current_is(']') {
                return Err(self.err_msg("数组元素之间期望 ',' 或 ']'"));
            }
        }
    }

    fn parse_dict(&mut self) -> Result<DataValue, String> {
        self.pos += 1; // 跳过 {
        let mut pairs = Vec::new();
        loop {
            self.skip_ws_comments();
            if self.is_at_end() {
                return Err(self.err_msg("对象缺少闭合 '}'"));
            }
            if self.current_is('}') {
                self.pos += 1;
                return Ok(DataValue::Dict(pairs));
            }
            // 键：带引号字符串或裸标识符
            let key = if self.current_is('"') || self.current_is('\'') {
                self.parse_string()?
            } else {
                self.read_ident()
            };
            if key.is_empty() {
                return Err(self.err_msg("对象键不能为空"));
            }
            self.skip_ws_comments();
            if !self.expect_char(':') {
                return Err(self.err_msg(&format!("对象键 '{}' 后期望 ':'", key)));
            }
            let value = self.parse_value()?;
            pairs.push((key, value));
            self.skip_ws_comments();
            if self.current_is(',') {
                self.pos += 1; // 尾逗号允许
            } else if !self.current_is('}') {
                return Err(self.err_msg("对象键值对之间期望 ',' 或 '}'"));
            }
        }
    }

    // === 辅助方法 ===

    fn skip_ws_comments(&mut self) {
        loop {
            while !self.is_at_end() && self.current().is_whitespace() {
                self.pos += 1;
            }
            // // 行注释
            if !self.is_at_end() && self.current_is('/') && self.peek(1) == '/' {
                while !self.is_at_end() && self.current() != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            // /* 块注释 */
            if !self.is_at_end() && self.current_is('/') && self.peek(1) == '*' {
                self.pos += 2;
                while self.pos + 1 < self.chars.len()
                    && !(self.chars[self.pos] == '*' && self.chars[self.pos + 1] == '/')
                {
                    self.pos += 1;
                }
                self.pos = (self.pos + 2).min(self.chars.len());
                continue;
            }
            break;
        }
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while !self.is_at_end() && (self.current().is_alphanumeric() || self.current() == '_') {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// 判断当前位置是否以 word 开头（且后面不是标识符字符，避免 varX 误判）
    fn peek_word(&self, word: &str) -> bool {
        let w: Vec<char> = word.chars().collect();
        if self.pos + w.len() > self.chars.len() {
            return false;
        }
        if self.chars[self.pos..self.pos + w.len()] != w[..] {
            return false;
        }
        let next = self.chars.get(self.pos + w.len()).copied().unwrap_or(' ');
        !(next.is_alphanumeric() || next == '_')
    }

    fn expect_char(&mut self, c: char) -> bool {
        if !self.is_at_end() && self.current_is(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn current(&self) -> char {
        self.chars[self.pos]
    }

    fn current_is(&self, c: char) -> bool {
        !self.is_at_end() && self.chars[self.pos] == c
    }

    fn peek(&self, offset: usize) -> char {
        self.chars.get(self.pos + offset).copied().unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn err_msg(&self, msg: &str) -> String {
        let line = self.chars[..self.pos.min(self.chars.len())]
            .iter()
            .filter(|&&c| c == '\n')
            .count()
            + 1;
        format!("第 {} 行: {}", line, msg)
    }
}

/// 解析 <style> 块内容为样式规则列表
fn parse_style_block(content: &str) -> Vec<StyleRule> {
    let mut rules = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = content.chars().collect();

    while pos < chars.len() {
        // 跳过空白
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        if pos >= chars.len() {
            break;
        }

        // 读取类选择器（以 . 开头）
        if chars[pos] != '.' {
            pos += 1;
            continue;
        }
        pos += 1; // 跳过 .

        // 读取类名
        let mut class_name = String::new();
        while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_' || chars[pos] == '-') {
            class_name.push(chars[pos]);
            pos += 1;
        }

        // 跳过空白
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        // 期望 {
        if pos >= chars.len() || chars[pos] != '{' {
            continue;
        }
        pos += 1; // 跳过 {

        // 读取属性直到 }
        let mut props_str = String::new();
        let mut depth = 1;
        while pos < chars.len() && depth > 0 {
            if chars[pos] == '{' {
                depth += 1;
            } else if chars[pos] == '}' {
                depth -= 1;
                if depth == 0 {
                    pos += 1;
                    break;
                }
            }
            props_str.push(chars[pos]);
            pos += 1;
        }

        // 解析属性
        let properties = parse_style_properties(&props_str);

        if !class_name.is_empty() {
            rules.push(StyleRule {
                class_name,
                properties,
            });
        }
    }

    rules
}

/// 解析样式属性字符串为 HashMap
fn parse_style_properties(input: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();

    for line in input.split(';') {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if !key.is_empty() {
                props.insert(key, value);
            }
        }
    }

    props
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input = r#"<ui>
            <Label text="Hello" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.root.tag, "ui");
        assert_eq!(result.root.children.len(), 1);
        assert_eq!(result.root.children[0].tag, "Label");
        assert_eq!(result.root.children[0].attributes[0], ("text".to_string(), "Hello".to_string()));
    }

    #[test]
    fn test_parse_nested() {
        let input = r#"<ui>
            <VBoxContainer>
                <Label text="Title" />
                <Button text="Click" />
            </VBoxContainer>
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.root.children[0].tag, "VBoxContainer");
        assert_eq!(result.root.children[0].children.len(), 2);
    }

    #[test]
    fn test_parse_style() {
        let input = r#"<ui>
            <style>
                .button-primary {
                    background: #2e7d32;
                    color: white;
                }
            </style>
            <Button text="OK" class="button-primary" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.styles.len(), 1);
        assert_eq!(result.styles[0].class_name, "button-primary");
        assert_eq!(result.styles[0].properties.get("background").unwrap(), "#2e7d32");
    }

    #[test]
    fn test_parse_attributes() {
        let input = r#"<ui>
            <VBoxContainer anchor="full" margin="12">
                <Label text='Single Quote' />
            </VBoxContainer>
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        let vbox = &result.root.children[0];
        assert_eq!(vbox.attributes.len(), 2);
        assert_eq!(vbox.attributes[0], ("anchor".to_string(), "full".to_string()));
        assert_eq!(vbox.attributes[1], ("margin".to_string(), "12".to_string()));
    }

    #[test]
    fn test_parse_ui_script() {
        let input = r#"<ui script="task_item.gd">
            <Panel name="ItemRoot" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.ui_script, Some("task_item.gd".to_string()));
        // 无 script 属性时为 None
        let plain = UiParser::new(r#"<ui><Label /></ui>"#).parse().unwrap();
        assert_eq!(plain.ui_script, None);
    }

    #[test]
    fn test_parse_signal_binding() {
        let input = r#"<ui>
            <Button text="Start" on_pressed="_on_start" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        let btn = &result.root.children[0];
        assert_eq!(btn.attributes[1], ("on_pressed".to_string(), "_on_start".to_string()));
    }

    #[test]
    fn test_parse_at_signal_binding() {
        // @pressed 简写：@ 前缀属性名，与 on_pressed 语义一致
        let input = r#"<ui>
            <Button text="Start" @pressed="_on_start" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        let btn = &result.root.children[0];
        assert_eq!(btn.attributes[1], ("@pressed".to_string(), "_on_start".to_string()));
    }

    #[test]
    fn test_parse_list_tags() {
        let input = r##"<ui>
            <UIHList count="5" highlight_mode="1" highlight_color="#ffff00">
                <Button text="Item" />
            </UIHList>
            <UIVList count="3" fill_mode="2" enable_random_pos="true" />
            <UIGrid count="6" highlight_mode="1" />
        </ui>"##;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.root.children.len(), 3);

        // UIHList
        let hlist = &result.root.children[0];
        assert_eq!(hlist.tag, "UIHList");
        assert_eq!(hlist.attributes[0], ("count".to_string(), "5".to_string()));
        assert_eq!(hlist.attributes[1], ("highlight_mode".to_string(), "1".to_string()));
        assert_eq!(hlist.children.len(), 1); // slot 子节点

        // UIVList
        let vlist = &result.root.children[1];
        assert_eq!(vlist.tag, "UIVList");
        // 验证 fill_mode 属性存在
        let has_fill_mode = vlist.attributes.iter().any(|(k, v)| k == "fill_mode" && v == "2");
        assert!(has_fill_mode);

        // UIGrid
        let grid = &result.root.children[2];
        assert_eq!(grid.tag, "UIGrid");
    }

    #[test]
    fn test_parse_multiple_styles() {
        let input = r#"<ui>
            <style>
                .btn-primary { background: #2e7d32; color: white; }
                .btn-danger { background: #c62828; color: white; border_radius: 4; }
                .panel-dark { bg_color: #333333; padding: 15; }
            </style>
            <VBoxContainer />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.styles.len(), 3);
        assert_eq!(result.styles[0].class_name, "btn-primary");
        assert_eq!(result.styles[1].class_name, "btn-danger");
        assert_eq!(result.styles[2].class_name, "panel-dark");
        assert_eq!(result.styles[1].properties.get("border_radius").unwrap(), "4");
    }

    #[test]
    fn test_parse_deep_nesting() {
        let input = r#"<ui>
            <VBoxContainer>
                <HBoxContainer>
                    <Panel>
                        <MarginContainer>
                            <Label text="Deep" />
                        </MarginContainer>
                    </Panel>
                </HBoxContainer>
            </VBoxContainer>
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        let vbox = &result.root.children[0];
        let hbox = &vbox.children[0];
        let panel = &hbox.children[0];
        let margin = &panel.children[0];
        let label = &margin.children[0];
        assert_eq!(label.tag, "Label");
        assert_eq!(label.attributes[0], ("text".to_string(), "Deep".to_string()));
    }

    #[test]
    fn test_parse_error_mismatched_tags() {
        let input = r#"<ui>
            <VBoxContainer>
                <Label text="test" />
            </HBoxContainer>
        </ui>"#;
        let result = UiParser::new(input).parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_missing_root() {
        let input = r#"<VBoxContainer><Label text="test" /></VBoxContainer>"#;
        let result = UiParser::new(input).parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_validate() {
        let valid = r#"<ui><Label text="OK" /></ui>"#;
        let result = UiParser::new(valid).parse();
        assert!(result.is_ok());

        let invalid = r#"<ui><Label text="unclosed"#;
        let result = UiParser::new(invalid).parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ui_attributes() {
        let input = r#"<ui title="demo">
            <Label text="test" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.root.attributes[0], ("title".to_string(), "demo".to_string()));
    }

    #[test]
    fn test_parse_theme_block() {
        let input = r#"<ui>
            <theme>
                bg_primary: #f8f4ff;
                text_primary: #3a2d5c;
            </theme>
            <Label text="test" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.theme_vars.get("bg_primary").unwrap(), "#f8f4ff");
        assert_eq!(result.theme_vars.get("text_primary").unwrap(), "#3a2d5c");
    }

    #[test]
    fn test_parse_script_block_values() {
        let input = r#"<ui>
            <script>
                // 行注释
                var tasks = [
                    { icon: "🌿", title: "采集灵草", count: 3, done: false },
                    { icon: "🐺", title: "击败妖狼", count: 8.5, done: true, }, // 尾逗号
                ]
                title = "单变量";
                /* 块注释 */ empty = null
            </script>
            <Label text="test" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.root.children.len(), 1, "<script> 块不应产生 UI 节点");

        let tasks = result.script_vars.get("tasks").unwrap();
        let items = match tasks {
            DataValue::Array(items) => items,
            other => panic!("tasks 应为数组，实际 {:?}", other),
        };
        assert_eq!(items.len(), 2);
        let first = match &items[0] {
            DataValue::Dict(pairs) => pairs,
            other => panic!("条目应为对象，实际 {:?}", other),
        };
        let get = |k: &str| {
            first
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        assert_eq!(get("icon"), DataValue::Str("🌿".to_string()));
        assert_eq!(get("count"), DataValue::Num(3.0));
        assert_eq!(get("done"), DataValue::Bool(false));
        assert_eq!(
            result.script_vars.get("title"),
            Some(&DataValue::Str("单变量".to_string()))
        );
        assert_eq!(result.script_vars.get("empty"), Some(&DataValue::Null));
    }

    #[test]
    fn test_parse_script_block_nested() {
        let input = r#"<ui>
            <script>
                var cfg = { list: [1, [2, 3], "x"], name: '单引号 "嵌套"' }
            </script>
            <Label />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        let cfg = result.script_vars.get("cfg").unwrap();
        match cfg {
            DataValue::Dict(pairs) => {
                assert_eq!(pairs.len(), 2);
                match &pairs[0].1 {
                    DataValue::Array(items) => {
                        assert_eq!(items.len(), 3);
                        match &items[1] {
                            DataValue::Array(inner) => assert_eq!(inner.len(), 2),
                            other => panic!("嵌套数组错误: {:?}", other),
                        }
                    }
                    other => panic!("list 应为数组: {:?}", other),
                }
            }
            other => panic!("cfg 应为对象: {:?}", other),
        }
    }

    #[test]
    fn test_parse_script_block_errors() {
        // 缺少 '='
        let bad = r#"<ui><script>
            tasks [1, 2]
        </script></ui>"#;
        assert!(UiParser::new(bad).parse().is_err());

        // 未闭合字符串
        let bad2 = r#"<ui><script>
            name = "abc
        </script></ui>"#;
        assert!(UiParser::new(bad2).parse().is_err());

        // 未闭合数组
        let bad3 = r#"<ui><script>
            tasks = [1, 2
        </script></ui>"#;
        assert!(UiParser::new(bad3).parse().is_err());
    }
}
