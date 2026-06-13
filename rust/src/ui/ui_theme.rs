// UI 主题系统
// 提供内置卡通风格配色方案和变量替换机制
// GML 中通过 <ui theme="cartoon"> 指定主题，样式属性值中使用 $var_name 引用主题变量
// GDScript 中通过 GdUiBuilder.set_theme() / GdGmlScene.theme_name 切换主题
// 组件默认颜色：builder 构建节点时自动从主题变量取值，无需 GML 中显式声明

use std::collections::HashMap;

/// 主题变量表：变量名 -> 变量值
pub type ThemeVars = HashMap<String, String>;

/// 获取内置主题列表
pub fn builtin_theme_names() -> Vec<&'static str> {
    vec!["cartoon"]
}

/// 根据名称获取内置主题变量表
/// 返回 None 表示未找到对应内置主题
pub fn get_builtin_theme(name: &str) -> Option<ThemeVars> {
    match name {
        "cartoon" => Some(cartoon_theme()),
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

/// Cartoon 主题（卡通亮色风格，圆角、鲜明色彩、活泼配色）
fn cartoon_theme() -> ThemeVars {
    let mut vars = ThemeVars::new();
    // 背景色 - 柔和亮色
    vars.insert("bg_primary".into(), "#f8f4ff".into());       // 淡紫白
    vars.insert("bg_secondary".into(), "#eee8f8".into());     // 浅紫灰
    vars.insert("bg_panel".into(), "#ffffff".into());          // 纯白面板
    vars.insert("bg_button".into(), "#e8dff5".into());        // 淡紫按钮
    vars.insert("bg_button_primary".into(), "#7c4dff".into()); // 鲜紫主按钮
    vars.insert("bg_button_danger".into(), "#ff5252".into());  // 鲜红危险按钮
    // 边框色 - 鲜明描边
    vars.insert("border_default".into(), "#c5b3e6".into());   // 淡紫边框
    vars.insert("border_accent".into(), "#7c4dff".into());    // 鲜紫强调边框
    vars.insert("border_highlight".into(), "#ffab40".into()); // 橙黄高亮边框
    // 文字色 - 深色为主，保证可读性
    vars.insert("text_primary".into(), "#3a2d5c".into());     // 深紫文字
    vars.insert("text_secondary".into(), "#7b6fa0".into());   // 紫灰次要文字
    vars.insert("text_muted".into(), "#a99cc4".into());       // 淡紫弱化文字
    vars.insert("text_accent".into(), "#7c4dff".into());      // 鲜紫强调文字
    vars.insert("text_title".into(), "#5c3dbd".into());       // 深紫标题
    vars.insert("text_white".into(), "white".into());         // 白色文字
    // 功能色
    vars.insert("overlay".into(), "#3a2d5c60".into());        // 半透明紫遮罩
    vars.insert("popup_bg".into(), "#fffffffa".into());       // 白色弹窗背景
    vars.insert("popup_border".into(), "#c5b3e6".into());     // 淡紫弹窗边框
    vars.insert("highlight".into(), "#7c4dff30".into());      // 鲜紫高亮
    vars.insert("highlight_strong".into(), "#ffab40".into()); // 橙黄强高亮
    vars.insert("accent".into(), "#7c4dff".into());           // 鲜紫强调色
    // 组件默认颜色（builder 自动应用）
    vars.insert("panel_bg".into(), "$bg_panel".into());
    vars.insert("button_bg".into(), "$bg_button".into());
    vars.insert("button_font_color".into(), "$text_primary".into());
    vars.insert("label_font_color".into(), "$text_primary".into());
    vars.insert("input_bg".into(), "#ffffff".into());
    vars.insert("input_font_color".into(), "$text_primary".into());
    vars.insert("separator_color".into(), "$border_default".into());
    vars.insert("tab_bg".into(), "$bg_secondary".into());
    vars.insert("tab_font_color".into(), "$text_secondary".into());
    vars.insert("tab_selected_bg".into(), "#ffffff".into());
    vars.insert("tab_selected_font_color".into(), "$text_accent".into());
    vars.insert("scrollbar_color".into(), "#c5b3e6".into());
    vars.insert("progress_bg".into(), "$bg_button".into());
    vars.insert("progress_fill".into(), "$accent".into());
    vars.insert("checkbutton_bg".into(), "$bg_button".into());
    vars.insert("slider_bg".into(), "$bg_button".into());
    vars.insert("slider_fill".into(), "$accent".into());
    vars.insert("optionbutton_bg".into(), "$bg_button".into());
    vars.insert("optionbutton_font_color".into(), "$text_primary".into());
    vars.insert("popup_title_color".into(), "$text_title".into());
    vars.insert("drawer_title_color".into(), "$text_title".into());
    vars.insert("tooltip_title_color".into(), "$text_accent".into());
    vars.insert("tooltip_content_color".into(), "$text_primary".into());
    vars.insert("nav_item_color".into(), "$text_primary".into());
    vars.insert("nav_item_hover_color".into(), "$text_accent".into());
    vars.insert("nav_item_active_color".into(), "#ff6d00".into());   // 活力橙激活
    vars.insert("nav_item_hover_bg".into(), "#7c4dff18".into());
    vars.insert("nav_item_pressed_bg".into(), "#7c4dff28".into());
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
        let vars = cartoon_theme();
        // resolve_theme_vars 只做一层替换
        let resolved = resolve_theme_vars("$panel_bg", &vars);
        assert_eq!(resolved, "$bg_panel"); // 第一层替换
        let resolved2 = resolve_theme_vars(&resolved, &vars);
        assert_eq!(resolved2, "#ffffff"); // 第二层替换
    }

    #[test]
    fn test_get_theme_color() {
        let vars = cartoon_theme();
        let color = get_theme_color(&vars, "bg_primary");
        assert!(color.is_some());
        let c = color.unwrap();
        assert!((c.r - 0.973).abs() < 0.01);
    }
}
