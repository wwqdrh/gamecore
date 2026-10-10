// GdWeather - 天气视觉节点
//
// 双通道架构（v2，替代旧版"整体 set_weather 交叉渐变"）：
//   ① 色调通道（tint）：单一全屏覆盖层 + 通用 tint shader，承载
//      dawn/day/dusk/night 四相位。相位切换**不换 shader、不换层**，
//      只是调色板目标值（乘色/附加光/暗角）变化 → process 逐帧缓动插值。
//      亮度/色温全程连续，天亮天黑不再有跳变。
//   ② 效果通道（fx）：双层覆盖层交叉渐变，承载 rain/storm/snow/fog/wind。
//      shader 内部按 density 做哈希门控（雨柱/雪花逐个"点亮"），
//      density = fade × intensity → 雨滴数量由少到多/由多到少真实渐变。
//   通道独立驱动：夜晚下雨 = night 色调 + rain 效果叠加（各管各的）。
//
// API：
//   set_weather(name)  兼容入口，按名字自动路由到 tint/effect 通道
//   set_tint(name)     色调通道（day/night/dawn/dusk；其他名字 = 清除色调）
//   set_effect(name)   效果通道（rain/storm/snow/fog/wind；其他 = 清除效果）
//   set_intensity(v)   效果密度终值倍率（0..1）
//
// 性能要点：
//   · Shader 对象按名字缓存（切换不再重新创建/编译）
//   · uniform 去重：值稳定后不再每帧写 shader 参数
//   · 树叶绘制先收集后绘制，透明度跟随 wind 层密度
//
// 覆盖层锚定（重要）：覆盖层是挂在世界画布上的 ColorRect，若固定在世界
//   原点、按窗口像素尺寸摆放，则只有「地图在原点附近且不大于视口」时才
//   碰巧盖住屏幕。换地图后相机 zoom/边界变化，平滑滑行期间可视区还会越
//   过地图边界进入负坐标——覆盖层盖不住，屏幕出现一条无天气亮带（表现
//   为"换图后天气被重置"）。因此覆盖层每帧对齐「可视世界矩形」（视口
//   矩形经画布变换逆推），与相机位置/缩放彻底解耦；树叶的活动范围同样
//   跟随可视区。
//
// 天气名：clear / day / night / dawn / dusk / rain / storm / snow / fog / wind

use std::collections::HashMap;

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

/// 一层效果覆盖层（fx 通道双层交叉渐变：一层 density 渐增、另一层渐减）
#[derive(Default)]
struct FxLayer {
    overlay: Option<Gd<ColorRect>>,
    mat: Option<Gd<ShaderMaterial>>,
    /// 本层当前渲染的效果名（"" = 空闲层）
    name: String,
    /// density 渐变系数 0..1
    fade: f32,
    /// 渐变目标（1.0 渐入中 / 0.0 渐出中或已停）
    target: f32,
    /// 上次写入 uniform 的 density 值（去重，值稳定后不再写）
    last_value: f32,
}

impl FxLayer {
    fn is_free(&self) -> bool {
        self.name.is_empty() || (self.target <= 0.0 && self.fade <= 0.0)
    }
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdWeather {
    /// 当前目标天气名（get_weather 返回值；tint/effect 共用）
    weather: GString,
    /// 效果密度终值倍率 0..1（fx 通道）
    #[var(pub)]
    intensity: f64,
    /// 效果渐入时长（秒，0.05 下限防除零）
    #[var(pub)]
    fade_in_time: f64,
    /// 效果渐出时长（秒）
    #[var(pub)]
    fade_out_time: f64,
    /// 色调缓动时长（秒；相位切换的调色板插值时间）
    #[var(pub)]
    tint_fade_time: f64,
    /// 刮风时的树叶数量
    #[var(pub)]
    leaf_count: i64,

    // ---- fx 通道（双层交叉渐变）----
    layers: [FxLayer; 2],
    /// 当前效果所在层下标
    active: usize,
    /// shader 缓存（名字 → 已创建的 Shader；切换不再重新创建）
    shader_cache: HashMap<String, Gd<Shader>>,

    // ---- tint 通道（单覆盖层 + 调色板缓动）----
    tint_overlay: Option<Gd<ColorRect>>,
    tint_mat: Option<Gd<ShaderMaterial>>,
    /// 当前色调相位名（"" = 中性/清除）
    tint_name: String,
    /// 当前调色板（缓动中）
    cur_mul: [f32; 3],
    cur_add: [f32; 3],
    cur_grad: f32,
    cur_vig: f32,
    cur_strength: f32,
    /// 目标调色板
    tgt_mul: [f32; 3],
    tgt_add: [f32; 3],
    tgt_grad: f32,
    tgt_vig: f32,
    tgt_strength: f32,
    /// 缓动起点快照（相位切换时从当前显示值出发，中途切换也连续）
    from_mul: [f32; 3],
    from_add: [f32; 3],
    from_grad: f32,
    from_vig: f32,
    from_strength: f32,
    /// 缓动进度 0..1（线性推进 + smoothstep 施加，准时收敛无慢尾）
    tint_progress: f32,

    /// 刮风树叶
    leaves: Vec<LeafState>,
    /// wind 效果当前密度（树叶绘制透明度）
    wind_alpha: f32,
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
            fade_in_time: 2.0,
            fade_out_time: 3.0,
            tint_fade_time: 4.0,
            leaf_count: 40,
            layers: [FxLayer::default(), FxLayer::default()],
            active: 0,
            shader_cache: HashMap::new(),
            tint_overlay: None,
            tint_mat: None,
            tint_name: String::new(),
            cur_mul: [1.0, 1.0, 1.0],
            cur_add: [0.0, 0.0, 0.0],
            cur_grad: 0.0,
            cur_vig: 1.0,
            cur_strength: 0.0,
            tgt_mul: [1.0, 1.0, 1.0],
            tgt_add: [0.0, 0.0, 0.0],
            tgt_grad: 0.0,
            tgt_vig: 1.0,
            tgt_strength: 0.0,
            from_mul: [1.0, 1.0, 1.0],
            from_add: [0.0, 0.0, 0.0],
            from_grad: 0.0,
            from_vig: 1.0,
            from_strength: 0.0,
            tint_progress: 1.0,
            leaves: Vec::new(),
            wind_alpha: 0.0,
            time: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        // 覆盖层每帧对齐相机可视区：必须晚于相机/玩家等默认优先级节点处理，
        // 否则同帧内相机后动会让覆盖层滞后一帧（换图滑行期露出无天气亮带）
        self.base_mut().set_process_priority(4096);
        // 编辑器预置了初始天气则应用（weather 字段已是该名字，先清掉再去重）
        let initial = self.weather.to_string();
        if !initial.is_empty() && initial != "clear" {
            self.weather = GString::from("clear");
            self.set_weather(GString::from(initial.as_str()));
        }
    }

    fn process(&mut self, delta: f64) {
        self.time += delta;
        // 可视世界矩形外扩半屏：吸收同帧内相机后动的残余时序差。
        // fx/tint shader 全部基于 SCREEN_UV 屏幕坐标采样，外扩不影响视觉，
        // 只保证任何相机速度下覆盖层几何上始终盖满屏幕。
        let vis_raw = self.visible_world_rect();
        let pad = vis_raw.size.x.max(vis_raw.size.y) * 0.5;
        let vis = vis_raw.grow_individual(pad, pad, pad, pad);
        let dt = delta as f32;

        // ---- tint 通道：调色板缓动插值（亮度/色温连续变化的核心）----
        // 线性进度 + smoothstep：准时收敛（progress 到 1 即停在目标），
        // 且中途切换相位时从当前显示值出发，过渡始终连续
        if self.tint_progress < 1.0 {
            self.tint_progress =
                (self.tint_progress + dt / self.tint_fade_time.max(0.05) as f32).min(1.0);
            let p = self.tint_progress;
            let k = p * p * (3.0 - 2.0 * p);
            for i in 0..3 {
                self.cur_mul[i] = self.from_mul[i] + (self.tgt_mul[i] - self.from_mul[i]) * k;
                self.cur_add[i] = self.from_add[i] + (self.tgt_add[i] - self.from_add[i]) * k;
            }
            self.cur_grad = self.from_grad + (self.tgt_grad - self.from_grad) * k;
            self.cur_vig = self.from_vig + (self.tgt_vig - self.from_vig) * k;
            self.cur_strength =
                self.from_strength + (self.tgt_strength - self.from_strength) * k;
            self.push_tint_uniforms();
        }

        // ---- fx 通道：density 渐变推进 + uniform 去重写入 ----
        let intensity = self.intensity.max(0.0).min(1.0) as f32;
        let in_rate = (1.0 / self.fade_in_time.max(0.05)) as f32;
        let out_rate = (1.0 / self.fade_out_time.max(0.05)) as f32;
        let mut wind_alpha = 0.0f32;
        for i in 0..2 {
            let (fade, target) = (self.layers[i].fade, self.layers[i].target);
            if fade != target {
                let rate = if target > fade { in_rate } else { out_rate };
                let step = rate * dt;
                let new_fade = if target > fade {
                    (fade + step).min(target)
                } else {
                    (fade - step).max(target)
                };
                self.layers[i].fade = new_fade;
            }
            // density = fade × intensity（intensity 变化也会即时反映）
            let value = self.layers[i].fade * intensity;
            if (value - self.layers[i].last_value).abs() > 1e-4 {
                self.layers[i].last_value = value;
                if let Some(mut mat) = self.layers[i].mat.take() {
                    mat.set_shader_parameter(
                        &StringName::from("density"),
                        &value.to_variant(),
                    );
                    self.layers[i].mat = Some(mat);
                }
            }
            if self.layers[i].name == "wind" {
                wind_alpha = wind_alpha.max(value);
            }
            // 渐出完成：隐藏覆盖层（材质保留，下次切换复用）
            if target <= 0.0 && fade <= 0.0 && !self.layers[i].name.is_empty() {
                self.layers[i].name = String::new();
                if let Some(mut ov) = self.layers[i].overlay.take() {
                    ov.set_visible(false);
                    self.layers[i].overlay = Some(ov);
                }
            }
        }
        self.wind_alpha = wind_alpha;

        // ---- 覆盖层同步：每帧对齐可视世界矩形（与相机位置/缩放解耦）----
        let tint_vis = self.tint_visible();
        if let Some(ov) = self.tint_overlay.as_mut() {
            sync_overlay(ov, vis, tint_vis);
        }
        for i in 0..2 {
            if let Some(ov) = self.layers[i].overlay.as_mut() {
                sync_overlay(ov, vis, ov.is_visible());
            }
        }

        // ---- 刮风：更新树叶运动（密度 > 0 才有意义；活动范围跟随可视区）----
        if wind_alpha > 0.001 && !self.leaves.is_empty() {
            let vp = vis.size;
            let origin = vis.position;
            let t = self.time as f32;
            for leaf in &mut self.leaves {
                leaf.rot += leaf.rot_spd * dt;
                leaf.pos.x +=
                    (leaf.speed + (t * leaf.sway_freq + leaf.phase).sin() * leaf.sway_amp) * dt;
                leaf.pos.y += leaf.fall * dt;
                // 越界回收（右侧/底部飘出后从左侧/顶部重新进入）
                if leaf.pos.x > origin.x + vp.x + 40.0 {
                    leaf.pos.x = origin.x - 40.0;
                    leaf.pos.y =
                        randf_range((origin.y - 40.0) as f64, (origin.y + vp.y * 0.8) as f64)
                            as f32;
                }
                if leaf.pos.y > origin.y + vp.y + 40.0 {
                    leaf.pos.y = origin.y - 40.0;
                    leaf.pos.x =
                        randf_range((origin.x - 40.0) as f64, (origin.x + vp.x * 0.8) as f64)
                            as f32;
                }
            }
            self.base_mut().queue_redraw();
        }
    }

    fn draw(&mut self) {
        // 刮风树叶画在天气节点自身画布上（在覆盖层之下，云影会扫过树叶）。
        // 透明度跟随 wind 层密度——雨滴同理，起风/风停也是渐变。
        if self.wind_alpha <= 0.001 || self.leaves.is_empty() {
            return;
        }
        struct LeafDraw {
            pts: PackedVector2Array,
            color: Color,
            vein: (Vector2, Vector2),
            vein_color: Color,
        }
        let alpha = self.wind_alpha;
        let cmds: Vec<LeafDraw> = self
            .leaves
            .iter()
            .map(|leaf| {
                let outline = geometry::leaf_outline(leaf.size, 0.55);
                let pts = geometry::place(&outline, leaf.pos, leaf.rot);
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
                LeafDraw {
                    pts,
                    color: Color {
                        r: leaf.color.r,
                        g: leaf.color.g,
                        b: leaf.color.b,
                        a: leaf.color.a * alpha,
                    },
                    vein: (world(v0), world(v1)),
                    vein_color,
                }
            })
            .collect();
        for cmd in cmds {
            self.base_mut().draw_colored_polygon(&cmd.pts, cmd.color);
            self.base_mut()
                .draw_line_ex(cmd.vein.0, cmd.vein.1, cmd.vein_color)
                .width(1.0)
                .done();
        }
    }
}

#[godot_api]
impl GdWeather {
    /// 切换天气（兼容入口）：tint 相位名路由 set_tint + 清效果，
    /// fx 效果名路由 set_effect，clear 全清。同名重复调用无效果。
    #[func]
    pub fn set_weather(&mut self, name: GString) {
        let s = normalize_weather(&name.to_string());
        if self.weather.to_string() == s {
            return;
        }
        if shaders::is_fx_weather(&s) {
            self.set_effect(GString::from(s.as_str()));
        } else if s == "clear" {
            self.clear_tint();
            self.clear_fx();
            self.weather = GString::from("clear");
        } else {
            // tint 相位（set_tint 内部更新 weather）
            self.set_tint(GString::from(s.as_str()));
            self.clear_fx();
        }
    }

    /// 色调通道：day/night/dawn/dusk（其他名字 = 清除色调，回中性）。
    /// 相位切换只改调色板目标值，process 逐帧缓动——亮度连续无跳变。
    #[func]
    pub fn set_tint(&mut self, name: GString) {
        let s = name.to_string();
        if !shaders::is_tint_weather(&s) {
            self.clear_tint();
            return;
        }
        if self.tint_name == s && self.tgt_strength >= 0.999 && self.tint_progress >= 1.0 {
            return;
        }
        self.snapshot_tint_from();
        let pal = shaders::tint_palette(&s);
        self.tint_name = s.clone();
        self.tgt_mul = pal.mul;
        self.tgt_add = pal.add;
        self.tgt_grad = pal.grad;
        self.tgt_vig = pal.vig;
        self.tgt_strength = 1.0;
        self.ensure_tint_overlay();
        self.weather = GString::from(s.as_str());
    }

    /// 效果通道：rain/storm/snow/fog/wind（其他名字 = 清除效果）。
    /// 旧效果 density 渐出、新效果 density 渐入（雨滴数量真实增减）。
    #[func]
    pub fn set_effect(&mut self, name: GString) {
        let s = name.to_string();
        if !shaders::is_fx_weather(&s) {
            self.clear_fx();
            return;
        }
        // 同名且正在渐入/已稳定 → 去重
        if self.layers[self.active].name == s && self.layers[self.active].target > 0.0 {
            self.weather = GString::from(s.as_str());
            return;
        }
        let old = self.active;
        self.layers[old].target = 0.0;
        let next = 1 - old;
        self.setup_fx_layer(next, &s);
        self.active = next;
        self.weather = GString::from(s.as_str());
    }

    /// 当前天气名（最后下发的目标天气）
    #[func]
    pub fn get_weather(&self) -> GString {
        self.weather.clone()
    }

    /// 当前色调强度 0..1（tint 通道缓动值，测试/过渡监听用）
    #[func]
    pub fn get_tint_strength(&self) -> f64 {
        self.cur_strength as f64
    }

    /// 当前效果密度 0..1（fx 活动层 fade × intensity，测试用）
    #[func]
    pub fn get_fx_density(&self) -> f64 {
        (self.layers[self.active].fade * self.intensity as f32) as f64
    }

    /// 当前效果层渐变系数 0..1（兼容旧 API）
    #[func]
    pub fn get_fade(&self) -> f64 {
        self.layers[self.active].fade as f64
    }

    /// 当前可见覆盖层数量（tint + fx；测试用）
    #[func]
    pub fn get_visible_overlay_count(&self) -> i64 {
        let mut n = 0;
        if self.tint_visible() {
            n += 1;
        }
        n += self
            .layers
            .iter()
            .filter(|l| l.overlay.as_ref().map(|o| o.is_visible()).unwrap_or(false))
            .count();
        n as i64
    }

    /// 获取效果通道活动层覆盖层（测试/扩展用）
    #[func]
    pub fn get_overlay(&self) -> Option<Gd<ColorRect>> {
        self.layers[self.active].overlay.clone()
    }

    // ---------- tint 通道内部 ----------

    /// 当前可视世界矩形：视口矩形经画布变换逆推（含相机位置/缩放/旋转）。
    /// 覆盖层与树叶活动范围以此为基准——相机在哪，天气就铺到哪，
    /// 换图/滑行/越界期间也不会露出未覆盖区域。
    fn visible_world_rect(&self) -> Rect2 {
        let screen = self.base().get_viewport_rect();
        let Some(vp) = self.base().get_viewport() else {
            return screen;
        };
        let inv = vp.get_canvas_transform().affine_inverse();
        // 取四角包围盒（相机带旋转时轴对齐包围盒仍能完整盖住屏幕）
        let c0 = inv * screen.position;
        let c1 = inv * (screen.position + Vector2::new(screen.size.x, 0.0));
        let c2 = inv * (screen.position + Vector2::new(0.0, screen.size.y));
        let c3 = inv * (screen.position + screen.size);
        let min = Vector2::new(
            c0.x.min(c1.x).min(c2.x).min(c3.x),
            c0.y.min(c1.y).min(c2.y).min(c3.y),
        );
        let max = Vector2::new(
            c0.x.max(c1.x).max(c2.x).max(c3.x),
            c0.y.max(c1.y).max(c2.y).max(c3.y),
        );
        Rect2::new(min, max - min)
    }

    /// tint 覆盖层是否应可见（强度非零或正在进入）
    fn tint_visible(&self) -> bool {
        self.cur_strength > 0.002 || self.tgt_strength > 0.002
    }

    /// 清除色调（目标回中性、强度回 0，process 缓动渐出）
    fn clear_tint(&mut self) {
        if self.tint_name.is_empty() && self.tgt_strength <= 0.0 && self.tint_progress >= 1.0 {
            return;
        }
        self.snapshot_tint_from();
        self.tint_name = String::new();
        let pal = shaders::tint_palette("");
        self.tgt_mul = pal.mul;
        self.tgt_add = pal.add;
        self.tgt_grad = pal.grad;
        self.tgt_vig = pal.vig;
        self.tgt_strength = 0.0;
    }

    /// 快照当前显示调色板为缓动起点（并重置进度）
    fn snapshot_tint_from(&mut self) {
        self.from_mul = self.cur_mul;
        self.from_add = self.cur_add;
        self.from_grad = self.cur_grad;
        self.from_vig = self.cur_vig;
        self.from_strength = self.cur_strength;
        self.tint_progress = 0.0;
    }

    /// 确保 tint 覆盖层存在并绑定缓动 shader（懒创建）
    fn ensure_tint_overlay(&mut self) {
        let valid = self
            .tint_overlay
            .as_ref()
            .map(|o| o.is_instance_valid())
            .unwrap_or(false);
        if valid {
            return;
        }
        let mut cr = ColorRect::new_alloc();
        cr.set_name(&StringName::from("TintOverlay"));
        cr.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
        cr.set_mouse_filter(MouseFilter::IGNORE);
        let mut mat = ShaderMaterial::new_gd();
        let shader = self
            .shader_cache
            .entry("__tint__".to_string())
            .or_insert_with(|| {
                let mut s = Shader::new_gd();
                s.set_code(&GString::from(shaders::TINT_SHADER_SRC));
                s
            })
            .clone();
        mat.set_shader(&shader);
        cr.set_material(&mat);
        self.base_mut().add_child(&cr);
        let vis = self.visible_world_rect();
        cr.set_position(vis.position);
        cr.set_size(vis.size);
        self.tint_overlay = Some(cr);
        self.tint_mat = Some(mat);
        // 立即按当前调色板写一遍 uniform（首帧即正确）
        self.push_tint_uniforms();
    }

    /// 把当前调色板写入 tint shader uniforms
    fn push_tint_uniforms(&mut self) {
        let Some(mut mat) = self.tint_mat.take() else {
            return;
        };
        let mul = Color {
            r: self.cur_mul[0],
            g: self.cur_mul[1],
            b: self.cur_mul[2],
            a: 1.0,
        };
        let add = Color {
            r: self.cur_add[0],
            g: self.cur_add[1],
            b: self.cur_add[2],
            a: 1.0,
        };
        mat.set_shader_parameter(&StringName::from("mul_color"), &mul.to_variant());
        mat.set_shader_parameter(&StringName::from("add_color"), &add.to_variant());
        mat.set_shader_parameter(&StringName::from("add_gradient"), &self.cur_grad.to_variant());
        mat.set_shader_parameter(&StringName::from("vig"), &self.cur_vig.to_variant());
        mat.set_shader_parameter(
            &StringName::from("tint_strength"),
            &self.cur_strength.to_variant(),
        );
        self.tint_mat = Some(mat);
        let vis = self.tint_visible();
        if let Some(ov) = self.tint_overlay.as_mut() {
            if ov.is_visible() != vis {
                ov.set_visible(vis);
            }
        }
    }

    // ---------- fx 通道内部 ----------

    /// 把某层设置为目标效果：绑定缓存 shader、显示覆盖层、density 从 0 渐入
    fn setup_fx_layer(&mut self, idx: usize, name: &str) {
        {
            let layer = &mut self.layers[idx];
            layer.name = name.to_string();
            layer.fade = 0.0;
            layer.target = 1.0;
            layer.last_value = f32::NAN; // 强制首次写 uniform
        }
        match shaders::builtin_fx_shader(name) {
            Some(src) => {
                self.ensure_fx_overlay(idx);
                // shader 按名字缓存复用（切换不再重新创建）
                let shader = self
                    .shader_cache
                    .entry(name.to_string())
                    .or_insert_with(|| {
                        let mut s = Shader::new_gd();
                        s.set_code(&GString::from(src));
                        s
                    })
                    .clone();
                let (mat, ov) = {
                    let layer = &mut self.layers[idx];
                    (layer.mat.take(), layer.overlay.take())
                };
                let vis = self.visible_world_rect();
                if let Some(mut m) = mat {
                    m.set_shader(&shader);
                    self.layers[idx].mat = Some(m);
                }
                if let Some(mut o) = ov {
                    o.set_visible(true);
                    o.set_position(vis.position);
                    o.set_size(vis.size);
                    self.layers[idx].overlay = Some(o);
                }
                if name == "wind" {
                    self.spawn_leaves();
                }
            }
            None => {}
        }
        self.base_mut().queue_redraw();
    }

    /// 渐出全部效果（density 缓降到 0；不改 weather 名）
    fn clear_fx(&mut self) {
        let mut any = false;
        for layer in &mut self.layers {
            if !layer.name.is_empty() && layer.target > 0.0 {
                layer.target = 0.0;
                any = true;
            }
        }
        if !self.leaves.is_empty() {
            self.leaves.clear();
            any = true;
        }
        if any {
            self.base_mut().queue_redraw();
        }
    }

    /// 确保某层覆盖层存在（懒创建，不拦截鼠标）
    fn ensure_fx_overlay(&mut self, idx: usize) {
        let valid = self.layers[idx]
            .overlay
            .as_ref()
            .map(|o| o.is_instance_valid())
            .unwrap_or(false);
        if valid {
            return;
        }
        let mut cr = ColorRect::new_alloc();
        cr.set_name(&StringName::from(&format!("WeatherOverlay{}", idx)));
        cr.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
        cr.set_mouse_filter(MouseFilter::IGNORE);
        let mut mat = ShaderMaterial::new_gd();
        cr.set_material(&mat);
        self.base_mut().add_child(&cr);
        self.layers[idx].overlay = Some(cr);
        self.layers[idx].mat = Some(mat);
    }

    /// 生成刮风树叶（位置/大小/颜色随机，秋季调色板；范围跟随可视区）
    fn spawn_leaves(&mut self) {
        let vis = self.visible_world_rect();
        let count = self.leaf_count.max(0) as usize;
        self.leaves.clear();
        for i in 0..count {
            self.leaves.push(LeafState {
                pos: Vector2::new(
                    randf_range(
                        (vis.position.x - 40.0) as f64,
                        (vis.position.x + vis.size.x) as f64,
                    ) as f32,
                    randf_range(
                        (vis.position.y - 40.0) as f64,
                        (vis.position.y + vis.size.y) as f64,
                    ) as f32,
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

/// 同步覆盖层位置/尺寸到可视世界矩形与可见性
fn sync_overlay(ov: &mut Gd<ColorRect>, want: Rect2, visible: bool) {
    if ov.is_visible() != visible {
        ov.set_visible(visible);
    }
    if visible && (ov.get_size() != want.size || ov.get_position() != want.position) {
        ov.set_position(want.position);
        ov.set_size(want.size);
    }
}

/// 天气名归一化（未知名字按 clear 处理）
fn normalize_weather(name: &str) -> String {
    if shaders::WEATHERS.contains(&name) {
        name.to_string()
    } else {
        "clear".to_string()
    }
}
