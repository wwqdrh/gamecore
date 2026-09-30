// GdWeather - 天气管理节点
// 结构：自身 (Node2D) 负责刮风树叶的绘制（复用 drawer 模块的树叶轮廓与调色板），
//       子节点 WeatherOverlay (ColorRect) 挂动态创建的天气 shader 做全屏后期。
// 切换天气：set_weather(name) → shaders::builtin_weather_shader 查表 →
//       Shader::new_gd + set_code 动态创建并绑定，fade 从 0 渐入到 intensity。
// 天气名：clear / day / night / rain / snow / wind（未知名字按 clear 处理）。

use godot::prelude::*;
use godot::classes::{
    control::{LayoutPreset, MouseFilter},
    ColorRect, INode2D, Node2D, Shader, ShaderMaterial,
};
use godot::global::randf_range;

use super::shaders;
use crate::drawer::{colors, geometry};

/// 刮风树叶的运动状态
#[derive(Clone)]
struct LeafState {
    pos: Vector2,
    size: f32,
    rot: f32,
    rot_spd: f32,
    /// 水平风速（像素/秒）
    speed: f32,
    /// 下落速度（像素/秒）
    fall: f32,
    sway_amp: f32,
    sway_freq: f32,
    phase: f32,
    color: Color,
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdWeather {
    /// 当前天气名（clear/day/night/rain/snow/wind）
    weather: GString,
    /// 效果强度 0..1（渐入目标值）
    #[var(pub)]
    intensity: f64,
    /// 刮风时的树叶数量
    #[var(pub)]
    leaf_count: i64,
    /// 全屏后期覆盖层
    overlay: Option<Gd<ColorRect>>,
    /// 覆盖层的材质（shader 动态创建）
    mat: Option<Gd<ShaderMaterial>>,
    /// 渐入系数 0..1
    fade: f32,
    /// 刮风树叶
    leaves: Vec<LeafState>,
    /// 累计时间
    time: f64,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdWeather {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            weather: GString::from("clear"),
            intensity: 1.0,
            leaf_count: 40,
            overlay: None,
            mat: None,
            fade: 0.0,
            leaves: Vec::new(),
            time: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        // 覆盖层保证在最上方（Node2D 无布局，ColorRect 锚定视口）
        self.ensure_overlay();
        let name = self.weather.to_string();
        self.apply_internal(&name);
    }

    fn process(&mut self, delta: f64) {
        self.time += delta;

        // 覆盖层显式同步视口大小：Control 挂在 Node2D 下时锚点布局不生效，
        // size 会保持 0 导致 shader 覆盖层完全不可见，必须手动撑满
        let vp_rect = self.base().get_viewport_rect();
        if let Some(mut ov) = self.overlay.take() {
            if ov.get_size() != vp_rect.size {
                ov.set_position(Vector2::ZERO);
                ov.set_size(vp_rect.size);
            }
            self.overlay = Some(ov);
        }

        // 渐入：fade → 1，实时刷新 intensity uniform
        if self.fade < 1.0 {
            self.fade = (self.fade + (delta * 2.0) as f32).min(1.0);
        }
        let value = self.intensity as f32 * self.fade;
        if let Some(mut mat) = self.mat.take() {
            mat.set_shader_parameter(&StringName::from("intensity"), &value.to_variant());
            self.mat = Some(mat);
        }

        // 刮风：更新树叶运动
        if self.weather.to_string() == "wind" && !self.leaves.is_empty() {
            let vp = self.base().get_viewport_rect().size;
            let t = self.time as f32;
            let dt = delta as f32;
            for leaf in &mut self.leaves {
                leaf.rot += leaf.rot_spd * dt;
                leaf.pos.x +=
                    (leaf.speed + (t * leaf.sway_freq + leaf.phase).sin() * leaf.sway_amp) * dt;
                leaf.pos.y += leaf.fall * dt;
                // 越界回收（右侧/底部飘出后从左侧/顶部重新进入）
                if leaf.pos.x > vp.x + 40.0 {
                    leaf.pos.x = -40.0;
                    leaf.pos.y = randf_range(-40.0, (vp.y * 0.8) as f64) as f32;
                }
                if leaf.pos.y > vp.y + 40.0 {
                    leaf.pos.y = -40.0;
                    leaf.pos.x = randf_range(-40.0, (vp.x * 0.8) as f64) as f32;
                }
            }
            self.base_mut().queue_redraw();
        }
    }

    fn draw(&mut self) {
        // 刮风树叶画在天气节点自身画布上（在覆盖层之下，云影会扫过树叶）
        if self.weather.to_string() != "wind" {
            return;
        }
        // 克隆一份遍历，避免遍历 leaves 与 base_mut() 的借用冲突
        let leaves = self.leaves.clone();
        for leaf in &leaves {
            let outline = geometry::leaf_outline(leaf.size, 0.55);
            let pts = geometry::place(&outline, leaf.pos, leaf.rot);
            self.base_mut().draw_colored_polygon(&pts, leaf.color);
            // 叶脉
            let vein_color = colors::darkened(leaf.color, 0.4);
            let v0 = Vector2::new(0.0, leaf.size * 0.12);
            let v1 = Vector2::new(0.0, leaf.size * 0.88);
            let (s, c) = leaf.rot.sin_cos();
            let world = |p: Vector2| {
                Vector2::new(
                    p.x * c - p.y * s + leaf.pos.x,
                    p.x * s + p.y * c + leaf.pos.y,
                )
            };
            self.base_mut()
                .draw_line_ex(world(v0), world(v1), vein_color)
                .width(1.0)
                .done();
        }
    }
}

#[godot_api]
impl GdWeather {
    /// 切换天气（clear / day / night / rain / snow / wind，未知名字按 clear 处理）
    #[func]
    pub fn set_weather(&mut self, name: GString) {
        let name_s = name.to_string();
        self.apply_internal(&name_s);
        self.weather = name;
    }

    /// 当前天气名
    #[func]
    pub fn get_weather(&self) -> GString {
        self.weather.clone()
    }

    /// 获取全屏覆盖层（测试/扩展用）
    #[func]
    pub fn get_overlay(&self) -> Option<Gd<ColorRect>> {
        self.overlay.clone()
    }

    // ---------- 内部实现 ----------

    /// 应用天气（不修改 weather 字段，ready 与 set_weather 共用）
    fn apply_internal(&mut self, name: &str) {
        self.leaves.clear();
        match shaders::builtin_weather_shader(name) {
            Some(src) => {
                self.ensure_overlay();
                // 字符串动态创建 shader 并绑定
                let mut shader = Shader::new_gd();
                shader.set_code(&GString::from(src));
                if let Some(mut mat) = self.mat.take() {
                    mat.set_shader(&shader);
                    self.mat = Some(mat);
                }
                if let Some(mut ov) = self.overlay.take() {
                    ov.set_visible(true);
                    self.overlay = Some(ov);
                }
                // 从 0 渐入
                self.fade = 0.0;
                if name == "wind" {
                    self.spawn_leaves();
                }
            }
            None => {
                // clear 或未知天气：隐藏覆盖层、清理树叶
                if let Some(mut ov) = self.overlay.take() {
                    ov.set_visible(false);
                    self.overlay = Some(ov);
                }
            }
        }
        self.base_mut().queue_redraw();
    }

    /// 确保覆盖层存在（懒创建，full-rect 锚定视口，不拦截鼠标）
    fn ensure_overlay(&mut self) {
        let valid = self
            .overlay
            .as_ref()
            .map(|o| o.is_instance_valid())
            .unwrap_or(false);
        if valid {
            return;
        }
        let mut cr = ColorRect::new_alloc();
        cr.set_name(&StringName::from("WeatherOverlay"));
        cr.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
        cr.set_mouse_filter(MouseFilter::IGNORE);
        // Node2D 父节点下锚点布局不生效，创建时直接按视口撑满
        let vp_size = self.base().get_viewport_rect().size;
        cr.set_size(vp_size);
        let mut mat = ShaderMaterial::new_gd();
        cr.set_material(&mat);
        self.base_mut().add_child(&cr);
        self.overlay = Some(cr);
        self.mat = Some(mat);
    }

    /// 生成刮风树叶（位置/大小/颜色随机，秋季调色板）
    fn spawn_leaves(&mut self) {
        let vp = self.base().get_viewport_rect().size;
        let count = self.leaf_count.max(0) as usize;
        self.leaves.clear();
        for i in 0..count {
            self.leaves.push(LeafState {
                pos: Vector2::new(
                    randf_range(-40.0, vp.x as f64) as f32,
                    randf_range(-40.0, vp.y as f64) as f32,
                ),
                size: randf_range(10.0, 22.0) as f32,
                rot: randf_range(0.0, std::f64::consts::TAU) as f32,
                rot_spd: randf_range(-3.0, 3.0) as f32,
                speed: randf_range(60.0, 140.0) as f32,
                fall: randf_range(40.0, 90.0) as f32,
                sway_amp: randf_range(20.0, 60.0) as f32,
                sway_freq: randf_range(1.0, 3.0) as f32,
                phase: randf_range(0.0, std::f64::consts::TAU) as f32,
                color: colors::palette_color(i),
            });
        }
    }
}
