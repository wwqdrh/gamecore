// UI 主题变量系统
// 无内置主题：变量来自 gml 的 <theme> 块（文件级自定义）与
// GdUiBuilder.set_theme_var()（代码注入），样式属性值中使用 $var_name 引用变量
// 组件默认颜色：builder 构建节点时自动从主题变量取值，无需 GML 中显式声明

use std::collections::HashMap;

/// 主题变量表：变量名 -> 变量值
pub type ThemeVars = HashMap<String, String>;

/// 解析 <theme> 块内容为变量表
/// 格式：每行一个变量定义，"var_name: value;" 或 "var_name: value"
/// 支持注释（// 开头的行）
pub fn parse_theme_block(content: &str) -> ThemeVars {
    let mut vars = ThemeVars::new();
    for line in content.lines() {
        let line = line.trim();
        // 跳过空行和注释
        if line.is_empty() || line.starts_with("//") || line.starts_with("<!--") {
            continue;
        }
        // 去掉行尾分号
        let line = line.trim_end_matches(';').trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if !key.is_empty() && !value.is_empty() {
                vars.insert(key, value);
            }
        }
    }
    vars
}

/// 替换字符串中的主题变量引用
/// $var_name 格式替换为变量值，未找到变量时保持原样
pub fn resolve_theme_vars(value: &str, vars: &ThemeVars) -> String {
    if !value.contains('$') {
        return value.to_string();
    }
    let mut result = value.to_string();
    // 匹配 $var_name 模式（var_name 由字母/数字/下划线组成）
    let re = regex_lite::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    result = re.replace_all(&result, |caps: &regex_lite::Captures| {
        let var_name = caps.get(1).unwrap().as_str();
        if let Some(val) = vars.get(var_name) {
            val.clone()
        } else {
            // 未找到变量，保持原样
            format!("${}", var_name)
        }
    }).to_string();
    result
}

/// 从主题变量中获取指定变量值，解析为 Color
/// 支持链式引用（如 panel_bg -> $bg_primary -> #1a1a3e）
/// 返回 None 表示变量不存在或无法解析
pub fn get_theme_color(vars: &ThemeVars, var_name: &str) -> Option<godot::builtin::Color> {
    let value = vars.get(var_name)?;
    let resolved = resolve_theme_vars_full(value, vars, 5);
    parse_theme_color(&resolved)
}

/// 递归替换主题变量引用，最多递归 max_depth 层
fn resolve_theme_vars_full(value: &str, vars: &ThemeVars, max_depth: usize) -> String {
    if max_depth == 0 || !value.contains('$') {
        return value.to_string();
    }
    let resolved = resolve_theme_vars(value, vars);
    if resolved == value {
        return resolved;
    }
    resolve_theme_vars_full(&resolved, vars, max_depth - 1)
}

/// 解析主题变量值为 Color
fn parse_theme_color(value: &str) -> Option<godot::builtin::Color> {
    let value = value.trim();
    if value.starts_with('#') {
        let hex = &value[1..];
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(godot::builtin::Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(godot::builtin::Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0))
            }
            _ => None,
        }
    } else {
        match value {
            "white" => Some(godot::builtin::Color::from_rgb(1.0, 1.0, 1.0)),
            "black" => Some(godot::builtin::Color::from_rgb(0.0, 0.0, 0.0)),
            "red" => Some(godot::builtin::Color::from_rgb(1.0, 0.0, 0.0)),
            "green" => Some(godot::builtin::Color::from_rgb(0.0, 1.0, 0.0)),
            "blue" => Some(godot::builtin::Color::from_rgb(0.0, 0.0, 1.0)),
            "transparent" => Some(godot::builtin::Color::from_rgba(0.0, 0.0, 0.0, 0.0)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_theme_block() {
        let content = r#"
            // 这是注释
            bg_primary: #f8f4ff;
            text_primary: #3a2d5c;
            border_default: #c5b3e6
        "#;
        let vars = parse_theme_block(content);
        assert_eq!(vars.get("bg_primary").unwrap(), "#f8f4ff");
        assert_eq!(vars.get("text_primary").unwrap(), "#3a2d5c");
        assert_eq!(vars.get("border_default").unwrap(), "#c5b3e6");
    }

    #[test]
    fn test_resolve_theme_vars() {
        let mut vars = ThemeVars::new();
        vars.insert("bg_primary".into(), "#f8f4ff".into());
        vars.insert("text_primary".into(), "#3a2d5c".into());

        // 简单替换
        assert_eq!(resolve_theme_vars("$bg_primary", &vars), "#f8f4ff");
        // 混合文本
        assert_eq!(resolve_theme_vars("color: $text_primary;", &vars), "color: #3a2d5c;");
        // 未找到变量保持原样
        assert_eq!(resolve_theme_vars("$unknown_var", &vars), "$unknown_var");
        // 无变量引用
        assert_eq!(resolve_theme_vars("#ff0000", &vars), "#ff0000");
    }

    #[test]
    fn test_resolve_theme_vars_chained() {
        // 测试变量引用链：panel_bg -> $bg_panel -> #ffffff
        let mut vars = ThemeVars::new();
        vars.insert("bg_panel".into(), "#ffffff".into());
        vars.insert("panel_bg".into(), "$bg_panel".into());
        // resolve_theme_vars 只做一层替换
        let resolved = resolve_theme_vars("$panel_bg", &vars);
        assert_eq!(resolved, "$bg_panel"); // 第一层替换
        let resolved2 = resolve_theme_vars(&resolved, &vars);
        assert_eq!(resolved2, "#ffffff"); // 第二层替换
    }

    #[test]
    fn test_get_theme_color() {
        let mut vars = ThemeVars::new();
        vars.insert("bg_primary".into(), "#f8f4ff".into());
        let color = get_theme_color(&vars, "bg_primary");
        assert!(color.is_some());
        let c = color.unwrap();
        assert!((c.r - 0.973).abs() < 0.01);
    }
}
