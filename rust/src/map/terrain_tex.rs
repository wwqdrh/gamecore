//! terrain_tex - 内置「地形贴图」shader（源码编译于 Rust 侧，无需 Godot 资源）
//!
//! 用途：为指定地形叠加一张平铺细节贴图（如草地的草纹）。
//! 由 GdQuickMap（挂每地形图层）与 GdDigLayer（挂根图层）共用，
//! 两个类的瓦片都画在单一 CanvasItem 上且「上层地形垫底/过渡片」
//! 会混入其他地形的像素，因此不能用材质直接区分地形——
//! 统一走「逐格遮罩贴图」：mask_tex 每世界格 1 像素，r/g/b 通道
//! 分别声明该格应用 tex0/tex1/tex2。
//!
//! 贴图采样用世界坐标 UV + repeat：相邻格子/过渡片之间纹丝对接，
//! 与 GdQuickMap.water_flow 同款约定（地图根位于原点）。
//!
//! uniform 契约（自定义 .gdshader 想吃贴图也按这个约定写）：
//!   mask_tex / map_cells / cell_px / tex0..2 / tex_size0..2

/// 内置 shader 注册名（GdQuickMap.terrain_shaders 可填此名）
pub const BUILTIN_NAME: &str = "terrain_tex";

/// 1×1 全透明纹理：占位未配置的 tex 槽位（tex1/tex2 等）。
/// 关键防御：gl_compatibility 下未设置的 sampler2D 会绑定引擎默认白色纹理，
/// 一旦遮罩通道误置位就会把整层刷成白色（xiaozhai 草地全白事故根因之一）
pub fn transparent_tex() -> godot::obj::Gd<godot::classes::ImageTexture> {
    use godot::classes::{Image, ImageTexture, image::Format};
    let mut img = Image::create_empty(1, 1, false, Format::RGBA8).expect("create 1x1 image");
    img.fill(godot::builtin::Color::from_rgba(0.0, 0.0, 0.0, 0.0));
    ImageTexture::create_from_image(&img).expect("create transparent texture")
}

pub const TERRAIN_TEX_SHADER_SRC: &str = r#"
shader_type canvas_item;

// 逐格遮罩：r/g/b = 该格应用 tex0/tex1/tex2（0.0 = 无）
uniform sampler2D mask_tex : filter_nearest, source_color;
uniform sampler2D tex0 : repeat_enable, filter_linear_mipmap, source_color;
uniform sampler2D tex1 : repeat_enable, filter_linear_mipmap, source_color;
uniform sampler2D tex2 : repeat_enable, filter_linear_mipmap, source_color;
// 各贴图的平铺尺寸（世界像素；默认 = 贴图原尺寸 1:1 平铺）
uniform vec2 tex_size0 = vec2(32.0, 32.0);
uniform vec2 tex_size1 = vec2(32.0, 32.0);
uniform vec2 tex_size2 = vec2(32.0, 32.0);
// 地图格数 / 格尺寸（世界像素）
uniform vec2 map_cells = vec2(1.0, 1.0);
uniform vec2 cell_px = vec2(32.0, 32.0);

varying vec2 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 0.0, 1.0)).xy;
}

void fragment() {
	vec4 base = texture(TEXTURE, UV);
	vec2 cell = floor(world_pos / cell_px);
	vec4 m = texture(mask_tex, (cell + 0.5) / map_cells);
	vec3 rgb = base.rgb;
	if (m.r > 0.5) {
		vec4 t = texture(tex0, world_pos / tex_size0);
		rgb = mix(rgb, t.rgb, t.a);
	}
	if (m.g > 0.5) {
		vec4 t = texture(tex1, world_pos / tex_size1);
		rgb = mix(rgb, t.rgb, t.a);
	}
	if (m.b > 0.5) {
		vec4 t = texture(tex2, world_pos / tex_size2);
		rgb = mix(rgb, t.rgb, t.a);
	}
	// 保留瓦片自身 alpha（双网格过渡片的圆角透明缺口不受影响）
	COLOR = vec4(rgb, base.a);
}
"#;
