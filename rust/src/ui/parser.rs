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
    pub theme_name: Option<String>,
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
        let mut theme_name: Option<String> = None;
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

        // 提取 theme 属性
        for (key, value) in &ui_attrs {
            if key == "theme" {
                theme_name = Some(value.clone());
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

            // 解析普通子节点
            let node = self.parse_node()?;
            root_children.push(node);
        }

        let root = UiNode {
            tag: "ui".to_string(),
            attributes: ui_attrs,
            children: root_children,
        };

        Ok(ParseResult { root, styles, theme_vars, theme_name })
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

    /// 读取属性名
    fn read_attr_name(&mut self) -> Result<String, ParseError> {
        let mut name = String::new();
        while !self.is_at_end() {
            let c = self.current_char();
            if c.is_alphanumeric() || c == '_' || c == '-' {
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
    fn test_parse_signal_binding() {
        let input = r#"<ui>
            <Button text="Start" on_pressed="_on_start" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        let btn = &result.root.children[0];
        assert_eq!(btn.attributes[1], ("on_pressed".to_string(), "_on_start".to_string()));
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
        let input = r#"<ui theme="dark">
            <Label text="test" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.root.attributes[0], ("theme".to_string(), "dark".to_string()));
        assert_eq!(result.theme_name, Some("dark".to_string()));
    }

    #[test]
    fn test_parse_theme_block() {
        let input = r#"<ui theme="dark">
            <theme>
                bg_primary: #1a1a3e;
                text_primary: #ccccee;
            </theme>
            <Label text="test" />
        </ui>"#;
        let result = UiParser::new(input).parse().unwrap();
        assert_eq!(result.theme_name, Some("dark".to_string()));
        assert_eq!(result.theme_vars.get("bg_primary").unwrap(), "#1a1a3e");
        assert_eq!(result.theme_vars.get("text_primary").unwrap(), "#ccccee");
    }
}
