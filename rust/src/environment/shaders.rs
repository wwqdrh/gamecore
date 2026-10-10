// 内置天气 shader 源码注册表
//
// 两类通道（与 GdWeather 的双通道结构对应）：
//   ① 色调（tint）：单一通用 shader，uniform 为调色板（mul_color/add_color/
//      add_gradient/vig）+ tint_strength。四相位共用一个 shader，相位切换由
//      Rust 侧逐帧缓动 uniform 实现（颜色/亮度连续插值，绝无跳变）。
//   ② 效果（fx）：rain/storm/snow/fog/wind 各自 shader，统一 uniform 为
//      density（0..1）。雨条/雪花/雾团按哈希门控逐个"点亮"——density 渐变时
//      可见雨滴数量真实增减（由少到多/由多到少），而非整屏透明度缩放。
//
// 约定：density=0 时 fx shader 输出必须恒等于原画面（无效果），
//       tint_strength=0 时 tint shader 同理——过渡两端自然衔接。

/// 单一色调 shader（四相位共用，uniform 由 Rust 侧缓动驱动）
pub const TINT_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform vec3 mul_color = vec3(1.0);
uniform vec3 add_color = vec3(0.0);
uniform float add_gradient = 0.0;
uniform float vig = 1.0;
uniform float tint_strength : hint_range(0.0, 1.0) = 0.0;
uniform sampler2D screen_tex : hint_screen_texture;

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	// 乘法色调 + 顶部渐变附加光（add_gradient 控制附加光是否带纵向衰减）
	vec3 tint = base.rgb * mul_color
		+ add_color * mix(1.0, 1.0 - SCREEN_UV.y, add_gradient);
	// 暗角（vig = 边缘最暗倍率，1.0 = 无暗角）
	float vigf = smoothstep(0.85, 0.30, distance(SCREEN_UV, vec2(0.5, 0.45)));
	tint *= mix(vig, 1.0, vigf);
	COLOR = vec4(mix(base.rgb, tint, tint_strength), base.a);
}
"#;

/// 下雨：密度门控的两层视差雨条 + 随密度加深的阴冷压暗
const RAIN_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float density : hint_range(0.0, 1.0) = 0.0;
uniform sampler2D screen_tex : hint_screen_texture;
uniform float speed = 1.0;

float hash12(vec2 p) {
	p = fract(p * vec2(123.34, 345.45));
	p += dot(p, p + 34.345);
	return fract(p.x * p.y);
}

// 一层雨条：屏幕按 col_w 切竖列，每列随机速度/相位；density 门控：
// 只有列哈希 < density 的列在下雨（密度渐增 → 可见雨柱逐列点亮）
float streak(vec2 frag, float col_w, float len, float thick, float slant, float seed) {
	frag.x += frag.y * slant;
	float x_id = floor(frag.x / col_w);
	float h = hash12(vec2(x_id, seed));
	float on = step(h, density);
	float y = fract(frag.y / len - TIME * speed * (0.8 + h * 0.6) + h * 7.0);
	float dx = fract(frag.x / col_w) - (0.2 + 0.6 * h);
	return step(y, thick) * step(abs(dx), 0.08) * on;
}

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	vec2 frag = SCREEN_UV / SCREEN_PIXEL_SIZE;
	float amount = smoothstep(0.0, 1.0, density);
	// 阴雨压暗偏冷（随密度渐深）
	vec3 tint = base.rgb * mix(vec3(1.0), vec3(0.62, 0.68, 0.80), amount);
	float r1 = streak(frag, 22.0, 90.0, 0.12, 0.12, 3.7);
	float r2 = streak(frag, 38.0, 150.0, 0.08, 0.18, 9.1) * 0.6;
	float drops = clamp(r1 + r2, 0.0, 1.0);
	vec3 drop_col = vec3(0.75, 0.85, 1.0) * 0.7;
	vec3 result = mix(tint, drop_col, drops * 0.5);
	COLOR = vec4(mix(base.rgb, result, amount), base.a);
}
"#;

/// 下雪：密度门控的两层视差雪花（每格哈希决定该格雪花是否可见）
const SNOW_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float density : hint_range(0.0, 1.0) = 0.0;
uniform sampler2D screen_tex : hint_screen_texture;
uniform float speed = 1.0;

float hash12(vec2 p) {
	p = fract(p * vec2(123.34, 345.45));
	p += dot(p, p + 34.345);
	return fract(p.x * p.y);
}

float flake(vec2 frag, float cell, float fall, float sway, float seed) {
	vec2 uv = frag / cell;
	uv.y -= TIME * speed * fall;
	uv.x += sin(TIME * 0.7 + uv.y * 2.0 + seed) * sway;
	vec2 id = floor(uv);
	vec2 f = fract(uv) - 0.5;
	vec2 off = vec2(hash12(id + seed), hash12(id + seed + 1.7)) - 0.5;
	float d = length(f - off * 0.55);
	float r = 0.05 + hash12(id + seed + 4.2) * 0.16;
	float on = step(hash12(id * 1.7 + seed + 11.3), density);
	return smoothstep(r, r * 0.4, d) * on;
}

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	vec2 frag = SCREEN_UV / SCREEN_PIXEL_SIZE;
	float amount = smoothstep(0.0, 1.0, density);
	vec3 tint = base.rgb * mix(vec3(1.0), vec3(0.78, 0.82, 0.92), amount);
	float s1 = flake(frag, 90.0, 0.14, 0.4, 1.3);
	float s2 = flake(frag, 55.0, 0.24, 0.5, 7.9) * 0.7;
	float flakes = clamp(s1 + s2, 0.0, 1.0);
	vec3 result = mix(tint, vec3(0.95, 0.97, 1.0), flakes * 0.85);
	COLOR = vec4(mix(base.rgb, result, amount), base.a);
}
"#;

/// 刮风：流动云影（密度控制云影浓度）
const WIND_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float density : hint_range(0.0, 1.0) = 0.0;
uniform sampler2D screen_tex : hint_screen_texture;

float hash12(vec2 p) {
	p = fract(p * vec2(123.34, 345.45));
	p += dot(p, p + 34.345);
	return fract(p.x * p.y);
}

float vnoise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	float a = hash12(i);
	float b = hash12(i + vec2(1.0, 0.0));
	float c = hash12(i + vec2(0.0, 1.0));
	float d = hash12(i + vec2(1.0, 1.0));
	return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	vec2 frag = SCREEN_UV / SCREEN_PIXEL_SIZE;
	float n = vnoise(frag / 260.0 + vec2(TIME * 0.35, TIME * 0.05));
	float shadow = smoothstep(0.55, 0.85, n);
	vec3 tint = base.rgb * mix(1.0, 0.80, shadow * 0.6);
	tint *= vec3(0.97, 1.0, 0.96);
	COLOR = vec4(mix(base.rgb, tint, density), base.a);
}
"#;

/// 浓雾：密度控制去饱和与雾团浓度（雾由薄到厚）
const FOG_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float density : hint_range(0.0, 1.0) = 0.0;
uniform sampler2D screen_tex : hint_screen_texture;

float hash12(vec2 p) {
	p = fract(p * vec2(123.34, 345.45));
	p += dot(p, p + 34.345);
	return fract(p.x * p.y);
}

float vnoise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	float a = hash12(i);
	float b = hash12(i + vec2(1.0, 0.0));
	float c = hash12(i + vec2(0.0, 1.0));
	float d = hash12(i + vec2(1.0, 1.0));
	return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	float amount = smoothstep(0.0, 1.0, density);
	// 去饱和灰化（随密度渐深）
	float gray = dot(base.rgb, vec3(0.299, 0.587, 0.114));
	vec3 desat = mix(base.rgb, vec3(gray), 0.45) * vec3(0.92, 0.94, 0.96);
	vec3 tinted = mix(base.rgb, desat, amount);
	// 两层流动雾团（不同尺度/速度）
	vec2 frag = SCREEN_UV / SCREEN_PIXEL_SIZE;
	float n1 = vnoise(frag / 170.0 + vec2(TIME * 0.045, TIME * 0.012));
	float n2 = vnoise(frag / 90.0 - vec2(TIME * 0.03, TIME * 0.02));
	float fog = smoothstep(0.35, 0.85, n1) * 0.65 + smoothstep(0.4, 0.9, n2) * 0.35;
	vec3 result = mix(tinted, vec3(0.80, 0.82, 0.86), fog * 0.75 * amount);
	COLOR = vec4(result, base.a);
}
"#;

/// 雷暴：密度门控密雨 + 暗青压暗 + 闪电（强降雨时才触发）
const STORM_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float density : hint_range(0.0, 1.0) = 0.0;
uniform sampler2D screen_tex : hint_screen_texture;
uniform float speed = 1.0;

float hash12(vec2 p) {
	p = fract(p * vec2(123.34, 345.45));
	p += dot(p, p + 34.345);
	return fract(p.x * p.y);
}

float streak(vec2 frag, float col_w, float len, float thick, float slant, float seed) {
	frag.x += frag.y * slant;
	float x_id = floor(frag.x / col_w);
	float h = hash12(vec2(x_id, seed));
	float on = step(h, density);
	float y = fract(frag.y / len - TIME * speed * (1.2 + h * 0.8) + h * 7.0);
	float dx = fract(frag.x / col_w) - (0.2 + 0.6 * h);
	return step(y, thick) * step(abs(dx), 0.08) * on;
}

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	vec2 frag = SCREEN_UV / SCREEN_PIXEL_SIZE;
	float amount = smoothstep(0.0, 1.0, density);
	// 雷雨云底压暗（比 rain 更暗更冷，随密度渐深）
	vec3 tint = base.rgb * mix(vec3(1.0), vec3(0.45, 0.50, 0.62), amount);
	float r1 = streak(frag, 16.0, 70.0, 0.16, 0.22, 3.7);
	float r2 = streak(frag, 30.0, 120.0, 0.10, 0.30, 9.1) * 0.7;
	float drops = clamp(r1 + r2, 0.0, 1.0);
	vec3 result = mix(tint, vec3(0.70, 0.78, 0.92) * 0.7, drops * 0.5);
	// 闪电：按时间片哈希触发的短促白幕（雨势足够大才出现）
	float flash = step(0.985, hash12(vec2(floor(TIME * 1.7), 3.3)))
		+ step(0.992, hash12(vec2(floor(TIME * 2.3), 7.7)));
	result += vec3(0.85, 0.88, 1.0) * clamp(flash, 0.0, 1.0) * 0.35
		* smoothstep(0.6, 1.0, density);
	COLOR = vec4(mix(base.rgb, result, amount), base.a);
}
"#;

/// 色调调色板（相位 → shader uniforms 的映射）
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TintPalette {
    /// 乘法色调（RGB 各通道倍率）
    pub mul: [f32; 3],
    /// 顶部附加光颜色（已含强度）
    pub add: [f32; 3],
    /// 附加光是否带纵向衰减（1.0 = 越靠顶部越亮）
    pub grad: f32,
    /// 暗角边缘最暗倍率（1.0 = 无暗角）
    pub vig: f32,
}

/// 相位调色板（源自原四套 tint shader 的视觉参数，拆成可插值的 uniform）
pub fn tint_palette(name: &str) -> TintPalette {
    match name {
        "day" => TintPalette {
            mul: [1.07, 1.03, 0.93],
            add: [0.060, 0.057, 0.048],
            grad: 1.0,
            vig: 1.0,
        },
        "night" => TintPalette {
            mul: [0.42, 0.50, 0.78],
            add: [0.02, 0.03, 0.08],
            grad: 0.0,
            vig: 0.70,
        },
        "dawn" => TintPalette {
            mul: [1.06, 0.99, 0.87],
            add: [0.090, 0.062, 0.035],
            grad: 1.0,
            vig: 1.0,
        },
        "dusk" => TintPalette {
            mul: [1.02, 0.84, 0.74],
            add: [0.102, 0.054, 0.030],
            grad: 1.0,
            vig: 0.86,
        },
        // 中性（清除色调）
        _ => TintPalette {
            mul: [1.0, 1.0, 1.0],
            add: [0.0, 0.0, 0.0],
            grad: 0.0,
            vig: 1.0,
        },
    }
}

/// 色调相位名（走 tint 通道的天气）
pub const TINT_WEATHERS: [&str; 4] = ["day", "night", "dawn", "dusk"];

pub fn is_tint_weather(name: &str) -> bool {
    TINT_WEATHERS.contains(&name)
}

/// 效果天气名（走 fx 通道，density 门控）
pub fn is_fx_weather(name: &str) -> bool {
    matches!(name, "rain" | "storm" | "snow" | "fog" | "wind")
}

/// 按名字取内置效果 shader 源码（非 fx 天气返回 None）
pub fn builtin_fx_shader(name: &str) -> Option<&'static str> {
    match name {
        "rain" => Some(RAIN_SHADER_SRC),
        "storm" => Some(STORM_SHADER_SRC),
        "snow" => Some(SNOW_SHADER_SRC),
        "fog" => Some(FOG_SHADER_SRC),
        "wind" => Some(WIND_SHADER_SRC),
        _ => None,
    }
}

/// 全部内置天气名（clear 表示清除；tint 相位 4 + fx 效果 5 + clear）
pub const WEATHERS: [&str; 10] = [
    "clear", "day", "night", "dawn", "dusk", "fog", "rain", "storm", "snow", "wind",
];
