// 内置天气 shader 源码注册表
// 约定：每个 shader 都有 uniform float intensity（0..1 效果强度），
// 并以 "mix(画面原色, 处理后颜色, intensity)" 作为输出骨架。

/// 白天：暖阳色调 + 顶部微光
const DAY_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float intensity : hint_range(0.0, 1.0) = 1.0;
uniform sampler2D screen_tex : hint_screen_texture;

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	// 暖阳色调 + 顶部微光
	vec3 tint = base.rgb * vec3(1.07, 1.03, 0.93);
	tint += vec3(1.0, 0.95, 0.80) * (1.0 - SCREEN_UV.y) * 0.06;
	COLOR = vec4(mix(base.rgb, tint, intensity), base.a);
}
"#;

/// 夜晚：冷蓝压暗 + 四周暗角
const NIGHT_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float intensity : hint_range(0.0, 1.0) = 1.0;
uniform sampler2D screen_tex : hint_screen_texture;

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	// 冷蓝压暗
	vec3 tint = base.rgb * vec3(0.42, 0.50, 0.78) + vec3(0.02, 0.03, 0.08);
	// 四周暗角（中心亮、边缘暗）
	float vig = smoothstep(0.85, 0.30, distance(SCREEN_UV, vec2(0.5, 0.45)));
	tint *= mix(0.70, 1.0, vig);
	COLOR = vec4(mix(base.rgb, tint, intensity), base.a);
}
"#;

/// 下雨：阴冷压暗 + 两层视差雨条（竖直下落、随风微斜）
const RAIN_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float intensity : hint_range(0.0, 1.0) = 1.0;
uniform sampler2D screen_tex : hint_screen_texture;
uniform float speed = 1.0;

float hash12(vec2 p) {
	p = fract(p * vec2(123.34, 345.45));
	p += dot(p, p + 34.345);
	return fract(p.x * p.y);
}

// 一层雨条：把屏幕按 col_w 切竖列，每列随机速度/相位，
// y 方向按 len 循环下落，落在 [0, thick] 段内即为雨条
float streak(vec2 frag, float col_w, float len, float thick, float slant, float seed) {
	frag.x += frag.y * slant;
	float x_id = floor(frag.x / col_w);
	float h = hash12(vec2(x_id, seed));
	float y = fract(frag.y / len - TIME * speed * (0.8 + h * 0.6) + h * 7.0);
	float dx = fract(frag.x / col_w) - (0.2 + 0.6 * h);
	return step(y, thick) * step(abs(dx), 0.08);
}

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	vec2 frag = SCREEN_UV / SCREEN_PIXEL_SIZE;
	// 阴雨压暗偏冷
	vec3 tint = base.rgb * vec3(0.62, 0.68, 0.80);
	float r1 = streak(frag, 22.0, 90.0, 0.12, 0.12, 3.7);
	float r2 = streak(frag, 38.0, 150.0, 0.08, 0.18, 9.1) * 0.6;
	float drops = clamp(r1 + r2, 0.0, 1.0) * intensity;
	vec3 drop_col = vec3(0.75, 0.85, 1.0) * 0.7;
	vec3 result = mix(tint, drop_col, drops * 0.5);
	COLOR = vec4(mix(base.rgb, result, intensity), base.a);
}
"#;

/// 下雪：轻微冷调 + 两层视差雪花（网格哈希定位，正弦横漂）
const SNOW_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float intensity : hint_range(0.0, 1.0) = 1.0;
uniform sampler2D screen_tex : hint_screen_texture;
uniform float speed = 1.0;

float hash12(vec2 p) {
	p = fract(p * vec2(123.34, 345.45));
	p += dot(p, p + 34.345);
	return fract(p.x * p.y);
}

// 一层雪花：屏幕按 cell 切格，每格哈希出位置/大小，
// y 方向按速度下落，x 方向正弦横漂，画圆点
float flake(vec2 frag, float cell, float fall, float sway, float seed) {
	vec2 uv = frag / cell;
	uv.y -= TIME * speed * fall;
	uv.x += sin(TIME * 0.7 + uv.y * 2.0 + seed) * sway;
	vec2 id = floor(uv);
	vec2 f = fract(uv) - 0.5;
	vec2 off = vec2(hash12(id + seed), hash12(id + seed + 1.7)) - 0.5;
	float d = length(f - off * 0.55);
	float r = 0.05 + hash12(id + seed + 4.2) * 0.16;
	return smoothstep(r, r * 0.4, d);
}

void fragment() {
	vec4 base = texture(screen_tex, SCREEN_UV);
	vec2 frag = SCREEN_UV / SCREEN_PIXEL_SIZE;
	// 轻微冷调
	vec3 tint = base.rgb * vec3(0.78, 0.82, 0.92);
	float s1 = flake(frag, 90.0, 0.14, 0.4, 1.3);
	float s2 = flake(frag, 55.0, 0.24, 0.5, 7.9) * 0.7;
	float flakes = clamp(s1 + s2, 0.0, 1.0) * intensity;
	vec3 result = mix(tint, vec3(0.95, 0.97, 1.0), flakes * 0.85);
	COLOR = vec4(mix(base.rgb, result, intensity), base.a);
}
"#;

/// 刮风：流动云影（值噪声带扫过画面）+ 轻微冷绿调
const WIND_SHADER_SRC: &str = r#"
shader_type canvas_item;
uniform float intensity : hint_range(0.0, 1.0) = 1.0;
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
	// 云影带：噪声随风横扫，暗块周期性压过画面
	float n = vnoise(frag / 260.0 + vec2(TIME * 0.35, TIME * 0.05));
	float shadow = smoothstep(0.55, 0.85, n);
	vec3 tint = base.rgb * mix(1.0, 0.80, shadow * 0.6);
	tint *= vec3(0.97, 1.0, 0.96);
	COLOR = vec4(mix(base.rgb, tint, intensity), base.a);
}
"#;

/// 按名字取内置天气 shader 源码（未知名字返回 None = 无效果）
pub fn builtin_weather_shader(name: &str) -> Option<&'static str> {
    match name {
        "day" => Some(DAY_SHADER_SRC),
        "night" => Some(NIGHT_SHADER_SRC),
        "rain" => Some(RAIN_SHADER_SRC),
        "snow" => Some(SNOW_SHADER_SRC),
        "wind" => Some(WIND_SHADER_SRC),
        _ => None,
    }
}

/// 全部内置天气名（clear 表示清除效果）
pub const WEATHERS: [&str; 6] = ["clear", "day", "night", "rain", "snow", "wind"];
