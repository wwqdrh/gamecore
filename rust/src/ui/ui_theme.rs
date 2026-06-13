// UI 主题系统
// 提供内置配色方案和变量替换机制
// GML 中通过 <ui theme="dark"> 指定主题，样式属性值中使用 $var_name 引用主题变量
// GDScript 中通过 GdUiBuilder.set_theme() / GdGmlScene.theme_name 切换主题
// 组件默认颜色：builder 构建节点时自动从主题变量取值，无需 GML 中显式声明

use std::collections::HashMap;

/// 主题变量表：变量名 -> 变量值
pub type ThemeVars = HashMap<String, String>;

/// 获取内置主题列表
pub fn builtin_theme_names() -> Vec<&'static str> {
    vec!["dark", "light", "forest", "ocean"]
}

/// 根据名称获取内置主题变量表
/// 返回 None 表示未找到对应内置主题
pub fn get_builtin_theme(name: &str) -> Option<ThemeVars> {
    match name {
        "dark" => Some(dark_theme()),
        "light" => Some(light_theme()),
        "forest" => Some(forest_theme()),
        "ocean" => Some(ocean_theme()),
        _ => None,
    }
}

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

// === 内置主题定义 ===

/// Dark 主题（默认，深色游戏风格）
fn dark_theme() -> ThemeVars {
    let mut vars = ThemeVars::new();
    // 背景色
    vars.insert("bg_primary".into(), "#1a1a3e".into());
    vars.insert("bg_secondary".into(), "#12122a".into());
    vars.insert("bg_panel".into(), "#0e1a2e".into());
    vars.insert("bg_button".into(), "#2a2a4e".into());
    vars.insert("bg_button_primary".into(), "#2e7d32".into());
    vars.insert("bg_button_danger".into(), "#c62828".into());
    // 边框色
    vars.insert("border_default".into(), "#3a3a6e".into());
    vars.insert("border_accent".into(), "#2a5a8e".into());
    vars.insert("border_highlight".into(), "#4a8a4e".into());
    // 文字色
    vars.insert("text_primary".into(), "#ccccee".into());
    vars.insert("text_secondary".into(), "#888899".into());
    vars.insert("text_muted".into(), "#8888aa".into());
    vars.insert("text_accent".into(), "#88aaff".into());
    vars.insert("text_title".into(), "#4488cc".into());
    vars.insert("text_white".into(), "white".into());
    // 组件色
    vars.insert("overlay".into(), "#00000080".into());
    vars.insert("popup_bg".into(), "#141123f0".into());
    vars.insert("popup_border".into(), "#5a5a8e".into());
    vars.insert("highlight".into(), "#ffffff40".into());
    vars.insert("highlight_strong".into(), "#ffff00".into());
    vars.insert("accent".into(), "#4488cc".into());
    // 组件默认颜色（builder 自动应用）
    vars.insert("panel_bg".into(), "$bg_primary".into());
    vars.insert("button_bg".into(), "$bg_button".into());
    vars.insert("button_font_color".into(), "$text_primary".into());
    vars.insert("label_font_color".into(), "$text_primary".into());
    vars.insert("input_bg".into(), "#0a0a1e".into());
    vars.insert("input_font_color".into(), "$text_primary".into());
    vars.insert("separator_color".into(), "$border_default".into());
    vars.insert("tab_bg".into(), "$bg_secondary".into());
    vars.insert("tab_font_color".into(), "$text_secondary".into());
    vars.insert("tab_selected_bg".into(), "$bg_primary".into());
    vars.insert("tab_selected_font_color".into(), "$text_accent".into());
    vars.insert("scrollbar_color".into(), "#3a3a5e".into());
    vars.insert("progress_bg".into(), "$bg_button".into());
    vars.insert("progress_fill".into(), "$accent".into());
    vars.insert("checkbutton_bg".into(), "$bg_button".into());
    vars.insert("slider_bg".into(), "$bg_button".into());
    vars.insert("slider_fill".into(), "$accent".into());
    vars.insert("optionbutton_bg".into(), "$bg_button".into());
    vars.insert("optionbutton_font_color".into(), "$text_primary".into());
    vars.insert("popup_title_color".into(), "$text_accent".into());
    vars.insert("drawer_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_content_color".into(), "$text_primary".into());
    vars.insert("nav_item_color".into(), "$text_primary".into());
    vars.insert("nav_item_hover_color".into(), "$text_white".into());
    vars.insert("nav_item_active_color".into(), "$text_accent".into());
    vars.insert("nav_item_hover_bg".into(), "#ffffff14".into());
    vars.insert("nav_item_pressed_bg".into(), "#4488cc20".into());
    vars
}

/// Light 主题（浅色风格）
fn light_theme() -> ThemeVars {
    let mut vars = ThemeVars::new();
    vars.insert("bg_primary".into(), "#f0f0f5".into());
    vars.insert("bg_secondary".into(), "#e0e0e8".into());
    vars.insert("bg_panel".into(), "#f5f5fa".into());
    vars.insert("bg_button".into(), "#d0d0d8".into());
    vars.insert("bg_button_primary".into(), "#4caf50".into());
    vars.insert("bg_button_danger".into(), "#e53935".into());
    vars.insert("border_default".into(), "#b0b0c0".into());
    vars.insert("border_accent".into(), "#88aacc".into());
    vars.insert("border_highlight".into(), "#66bb6a".into());
    vars.insert("text_primary".into(), "#1a1a2e".into());
    vars.insert("text_secondary".into(), "#555566".into());
    vars.insert("text_muted".into(), "#777788".into());
    vars.insert("text_accent".into(), "#2266aa".into());
    vars.insert("text_title".into(), "#2266aa".into());
    vars.insert("text_white".into(), "white".into());
    vars.insert("overlay".into(), "#00000040".into());
    vars.insert("popup_bg".into(), "#f8f8fcf0".into());
    vars.insert("popup_border".into(), "#a0a0b8".into());
    vars.insert("highlight".into(), "#2266aa30".into());
    vars.insert("highlight_strong".into(), "#ff8800".into());
    vars.insert("accent".into(), "#2266aa".into());
    // 组件默认颜色
    vars.insert("panel_bg".into(), "$bg_primary".into());
    vars.insert("button_bg".into(), "$bg_button".into());
    vars.insert("button_font_color".into(), "$text_primary".into());
    vars.insert("label_font_color".into(), "$text_primary".into());
    vars.insert("input_bg".into(), "#ffffff".into());
    vars.insert("input_font_color".into(), "$text_primary".into());
    vars.insert("separator_color".into(), "$border_default".into());
    vars.insert("tab_bg".into(), "$bg_secondary".into());
    vars.insert("tab_font_color".into(), "$text_secondary".into());
    vars.insert("tab_selected_bg".into(), "$bg_primary".into());
    vars.insert("tab_selected_font_color".into(), "$text_accent".into());
    vars.insert("scrollbar_color".into(), "#c0c0d0".into());
    vars.insert("progress_bg".into(), "$bg_button".into());
    vars.insert("progress_fill".into(), "$accent".into());
    vars.insert("checkbutton_bg".into(), "$bg_button".into());
    vars.insert("slider_bg".into(), "$bg_button".into());
    vars.insert("slider_fill".into(), "$accent".into());
    vars.insert("optionbutton_bg".into(), "$bg_button".into());
    vars.insert("optionbutton_font_color".into(), "$text_primary".into());
    vars.insert("popup_title_color".into(), "$text_accent".into());
    vars.insert("drawer_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_content_color".into(), "$text_primary".into());
    vars.insert("nav_item_color".into(), "$text_primary".into());
    vars.insert("nav_item_hover_color".into(), "#1a1a2e".into());
    vars.insert("nav_item_active_color".into(), "$text_accent".into());
    vars.insert("nav_item_hover_bg".into(), "#2266aa14".into());
    vars.insert("nav_item_pressed_bg".into(), "#2266aa20".into());
    vars
}

/// Forest 主题（森林绿色风格）
fn forest_theme() -> ThemeVars {
    let mut vars = ThemeVars::new();
    vars.insert("bg_primary".into(), "#1a2e1a".into());
    vars.insert("bg_secondary".into(), "#0e1e0e".into());
    vars.insert("bg_panel".into(), "#122012".into());
    vars.insert("bg_button".into(), "#2a4a2e".into());
    vars.insert("bg_button_primary".into(), "#388e3c".into());
    vars.insert("bg_button_danger".into(), "#b71c1c".into());
    vars.insert("border_default".into(), "#2a5a2a".into());
    vars.insert("border_accent".into(), "#4a8a4e".into());
    vars.insert("border_highlight".into(), "#66bb6a".into());
    vars.insert("text_primary".into(), "#cceecc".into());
    vars.insert("text_secondary".into(), "#88aa88".into());
    vars.insert("text_muted".into(), "#6a8a6a".into());
    vars.insert("text_accent".into(), "#88ee88".into());
    vars.insert("text_title".into(), "#44aa44".into());
    vars.insert("text_white".into(), "white".into());
    vars.insert("overlay".into(), "#00000060".into());
    vars.insert("popup_bg".into(), "#0e1e0ef0".into());
    vars.insert("popup_border".into(), "#4a8a4e".into());
    vars.insert("highlight".into(), "#88ff8840".into());
    vars.insert("highlight_strong".into(), "#ffcc00".into());
    vars.insert("accent".into(), "#44aa44".into());
    // 组件默认颜色
    vars.insert("panel_bg".into(), "$bg_primary".into());
    vars.insert("button_bg".into(), "$bg_button".into());
    vars.insert("button_font_color".into(), "$text_primary".into());
    vars.insert("label_font_color".into(), "$text_primary".into());
    vars.insert("input_bg".into(), "#0a160a".into());
    vars.insert("input_font_color".into(), "$text_primary".into());
    vars.insert("separator_color".into(), "$border_default".into());
    vars.insert("tab_bg".into(), "$bg_secondary".into());
    vars.insert("tab_font_color".into(), "$text_secondary".into());
    vars.insert("tab_selected_bg".into(), "$bg_primary".into());
    vars.insert("tab_selected_font_color".into(), "$text_accent".into());
    vars.insert("scrollbar_color".into(), "#2a4a2e".into());
    vars.insert("progress_bg".into(), "$bg_button".into());
    vars.insert("progress_fill".into(), "$accent".into());
    vars.insert("checkbutton_bg".into(), "$bg_button".into());
    vars.insert("slider_bg".into(), "$bg_button".into());
    vars.insert("slider_fill".into(), "$accent".into());
    vars.insert("optionbutton_bg".into(), "$bg_button".into());
    vars.insert("optionbutton_font_color".into(), "$text_primary".into());
    vars.insert("popup_title_color".into(), "$text_accent".into());
    vars.insert("drawer_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_content_color".into(), "$text_primary".into());
    vars.insert("nav_item_color".into(), "$text_primary".into());
    vars.insert("nav_item_hover_color".into(), "$text_white".into());
    vars.insert("nav_item_active_color".into(), "$text_accent".into());
    vars.insert("nav_item_hover_bg".into(), "#88ff8814".into());
    vars.insert("nav_item_pressed_bg".into(), "#44aa4420".into());
    vars
}

/// Ocean 主题（海洋蓝色风格）
fn ocean_theme() -> ThemeVars {
    let mut vars = ThemeVars::new();
    vars.insert("bg_primary".into(), "#0e1a2e".into());
    vars.insert("bg_secondary".into(), "#0a1222".into());
    vars.insert("bg_panel".into(), "#081420".into());
    vars.insert("bg_button".into(), "#1a3a5e".into());
    vars.insert("bg_button_primary".into(), "#0277bd".into());
    vars.insert("bg_button_danger".into(), "#c62828".into());
    vars.insert("border_default".into(), "#2a5a8e".into());
    vars.insert("border_accent".into(), "#4488cc".into());
    vars.insert("border_highlight".into(), "#42a5f5".into());
    vars.insert("text_primary".into(), "#cceeee".into());
    vars.insert("text_secondary".into(), "#88aacc".into());
    vars.insert("text_muted".into(), "#6688aa".into());
    vars.insert("text_accent".into(), "#88ddff".into());
    vars.insert("text_title".into(), "#42a5f5".into());
    vars.insert("text_white".into(), "white".into());
    vars.insert("overlay".into(), "#00000050".into());
    vars.insert("popup_bg".into(), "#0a1a2ef0".into());
    vars.insert("popup_border".into(), "#4488cc".into());
    vars.insert("highlight".into(), "#88ddff40".into());
    vars.insert("highlight_strong".into(), "#ff8800".into());
    vars.insert("accent".into(), "#42a5f5".into());
    // 组件默认颜色
    vars.insert("panel_bg".into(), "$bg_primary".into());
    vars.insert("button_bg".into(), "$bg_button".into());
    vars.insert("button_font_color".into(), "$text_primary".into());
    vars.insert("label_font_color".into(), "$text_primary".into());
    vars.insert("input_bg".into(), "#060e18".into());
    vars.insert("input_font_color".into(), "$text_primary".into());
    vars.insert("separator_color".into(), "$border_default".into());
    vars.insert("tab_bg".into(), "$bg_secondary".into());
    vars.insert("tab_font_color".into(), "$text_secondary".into());
    vars.insert("tab_selected_bg".into(), "$bg_primary".into());
    vars.insert("tab_selected_font_color".into(), "$text_accent".into());
    vars.insert("scrollbar_color".into(), "#1a3a5e".into());
    vars.insert("progress_bg".into(), "$bg_button".into());
    vars.insert("progress_fill".into(), "$accent".into());
    vars.insert("checkbutton_bg".into(), "$bg_button".into());
    vars.insert("slider_bg".into(), "$bg_button".into());
    vars.insert("slider_fill".into(), "$accent".into());
    vars.insert("optionbutton_bg".into(), "$bg_button".into());
    vars.insert("optionbutton_font_color".into(), "$text_primary".into());
    vars.insert("popup_title_color".into(), "$text_accent".into());
    vars.insert("drawer_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_content_color".into(), "$text_primary".into());
    vars.insert("nav_item_color".into(), "$text_primary".into());
    vars.insert("nav_item_hover_color".into(), "$text_white".into());
    vars.insert("nav_item_active_color".into(), "$text_accent".into());
    vars.insert("nav_item_hover_bg".into(), "#88ddff14".into());
    vars.insert("nav_item_pressed_bg".into(), "#42a5f520".into());
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_themes() {
        for name in builtin_theme_names() {
            let theme = get_builtin_theme(name);
            assert!(theme.is_some(), "Missing builtin theme: {}", name);
            let vars = theme.unwrap();
            // 每个内置主题至少包含这些核心变量
            assert!(vars.contains_key("bg_primary"), "{} missing bg_primary", name);
            assert!(vars.contains_key("text_primary"), "{} missing text_primary", name);
            assert!(vars.contains_key("border_default"), "{} missing border_default", name);
            // 组件默认颜色变量
            assert!(vars.contains_key("panel_bg"), "{} missing panel_bg", name);
            assert!(vars.contains_key("button_bg"), "{} missing button_bg", name);
            assert!(vars.contains_key("button_font_color"), "{} missing button_font_color", name);
            assert!(vars.contains_key("label_font_color"), "{} missing label_font_color", name);
        }
    }

    #[test]
    fn test_parse_theme_block() {
        let content = r#"
            // 这是注释
            bg_primary: #1a1a3e;
            text_primary: #ccccee;
            border_default: #3a3a6e
        "#;
        let vars = parse_theme_block(content);
        assert_eq!(vars.get("bg_primary").unwrap(), "#1a1a3e");
        assert_eq!(vars.get("text_primary").unwrap(), "#ccccee");
        assert_eq!(vars.get("border_default").unwrap(), "#3a3a6e");
    }

    #[test]
    fn test_resolve_theme_vars() {
        let mut vars = ThemeVars::new();
        vars.insert("bg_primary".into(), "#1a1a3e".into());
        vars.insert("text_primary".into(), "#ccccee".into());

        // 简单替换
        assert_eq!(resolve_theme_vars("$bg_primary", &vars), "#1a1a3e");
        // 混合文本
        assert_eq!(resolve_theme_vars("color: $text_primary;", &vars), "color: #ccccee;");
        // 未找到变量保持原样
        assert_eq!(resolve_theme_vars("$unknown_var", &vars), "$unknown_var");
        // 无变量引用
        assert_eq!(resolve_theme_vars("#ff0000", &vars), "#ff0000");
    }

    #[test]
    fn test_resolve_theme_vars_chained() {
        // 测试变量引用链：panel_bg -> $bg_primary -> #1a1a3e
        let vars = dark_theme();
        // resolve_theme_vars 只做一层替换
        let resolved = resolve_theme_vars("$panel_bg", &vars);
        assert_eq!(resolved, "$bg_primary"); // 第一层替换
        let resolved2 = resolve_theme_vars(&resolved, &vars);
        assert_eq!(resolved2, "#1a1a3e"); // 第二层替换
    }

    #[test]
    fn test_get_theme_color() {
        let vars = dark_theme();
        let color = get_theme_color(&vars, "bg_primary");
        assert!(color.is_some());
        let c = color.unwrap();
        assert!((c.r - 0.102).abs() < 0.01);
    }
}
