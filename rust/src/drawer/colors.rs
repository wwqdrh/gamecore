// 颜色工具：秋季叶色调色板、hex 解析、颜色插值

use godot::prelude::*;

/// 秋季树叶调色板（绿 → 黄绿 → 金黄 → 橙 → 红 → 棕）
pub fn autumn_leaf_palette() -> [Color; 6] {
    [
        Color::from_rgb(0.29, 0.49, 0.18),
        Color::from_rgb(0.49, 0.60, 0.18),
        Color::from_rgb(0.79, 0.64, 0.15),
        Color::from_rgb(0.85, 0.48, 0.16),
        Color::from_rgb(0.71, 0.27, 0.16),
        Color::from_rgb(0.54, 0.35, 0.23),
    ]
}

/// 按序取调色板色（idx 越界自动取模）
pub fn palette_color(idx: usize) -> Color {
    let p = autumn_leaf_palette();
    p[idx % p.len()]
}

/// 线性混合两色（t=0 取 a，t=1 取 b）
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

/// 颜色加深（rgb 等比缩小，alpha 不变）
pub fn darkened(c: Color, amount: f32) -> Color {
    let k = (1.0 - amount).clamp(0.0, 1.0);
    Color::from_rgba(c.r * k, c.g * k, c.b * k, c.a)
}

/// 解析 "#RRGGBB" 或 "#RRGGBBAA"（不区分大小写），失败返回 None
pub fn hex_to_color(hex: &str) -> Option<Color> {
    let h = hex.trim().trim_start_matches('#');
    let v = u32::from_str_radix(h, 16).ok()?;
    match h.len() {
        6 => Some(Color::from_rgba(
            ((v >> 16) & 0xFF) as f32 / 255.0,
            ((v >> 8) & 0xFF) as f32 / 255.0,
            (v & 0xFF) as f32 / 255.0,
            1.0,
        )),
        8 => Some(Color::from_rgba(
            ((v >> 24) & 0xFF) as f32 / 255.0,
            ((v >> 16) & 0xFF) as f32 / 255.0,
            ((v >> 8) & 0xFF) as f32 / 255.0,
            (v & 0xFF) as f32 / 255.0,
        )),
        _ => None,
    }
}

/// 颜色转 "#RRGGBBAA"
pub fn color_to_hex(c: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        (c.r * 255.0).round().clamp(0.0, 255.0) as u32,
        (c.g * 255.0).round().clamp(0.0, 255.0) as u32,
        (c.b * 255.0).round().clamp(0.0, 255.0) as u32,
        (c.a * 255.0).round().clamp(0.0, 255.0) as u32,
    )
}

// ---------- Godot 静态方法接口 ----------

/// 颜色工具类（静态方法），供 GDScript 侧使用
#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdColorUtil {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GdColorUtil {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GdColorUtil {
    /// 解析 "#RRGGBB[AA]"，失败返回白色
    #[func]
    pub fn hex_to_color(hex: GString) -> Color {
        hex_to_color(&hex.to_string()).unwrap_or(Color::WHITE)
    }

    /// 颜色转 "#RRGGBBAA"
    #[func]
    pub fn color_to_hex(c: Color) -> GString {
        GString::from(color_to_hex(c).as_str())
    }

    /// 线性混合两色
    #[func]
    pub fn mix_colors(a: Color, b: Color, t: f32) -> Color {
        mix(a, b, t)
    }

    /// 颜色加深
    #[func]
    pub fn darken(c: Color, amount: f32) -> Color {
        darkened(c, amount)
    }

    /// 按序取秋季叶色调色板（idx 越界取模）
    #[func]
    pub fn autumn_leaf_color(idx: i32) -> Color {
        let i = if idx < 0 { 0 } else { idx as usize };
        palette_color(i)
    }
}
