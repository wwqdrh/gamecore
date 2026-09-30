// GdDebugDraw - 运行时 2D 调试绘制单例
// 注册为 Engine singleton "GDDEBUGDRAW"
//
// 原型迭代时可视化：碰撞范围、巡逻点、射线检测、寻路路径……
// 第一次调用绘制接口时，自动在场景根下创建画布节点（z_index 很高，
// 始终盖在游戏画面上，mouse_filter 忽略不影响交互）。
//
// 每条图形带存活时长（默认 1 秒），到点自动消失；life<=0 表示常驻直到 clear。
//
// GDScript 用法：
//   var dd = Engine.get_singleton("GDDEBUGDRAW")
//   dd.draw_line(Vector2.ZERO, Vector2(100, 0), Color.RED, 0.0)
//   dd.draw_circle(enemy.position, 64.0, Color(1,0,0,0.3), true, 2.0)
//   dd.draw_ray(player.position, player.velocity, 80.0, Color.YELLOW, 0.0)

use godot::prelude::*;
use godot::builtin::{GString, StringName, Variant};
use godot::classes::{Engine, INode2D, Node2D, Object, SceneTree};

/// 单条调试图形
#[derive(Clone, Copy)]
enum Shape {
    Line(Vector2, Vector2),
    Rect(Rect2, bool),
    Circle(Vector2, f32, bool),
    /// 射线：起点 + 方向 + 长度（画线 + 端头箭头）
    Ray(Vector2, Vector2, f32),
}

#[derive(Clone)]
struct ShapeEntry {
    shape: Shape,
    color: Color,
    /// 存活剩余秒数（INFINITY = 常驻）
    life: f64,
}

/// 画布节点：进树后 _draw 绘制队列、_process 倒计时清理
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdDebugDrawNode {
    shapes: Vec<ShapeEntry>,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdDebugDrawNode {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            shapes: Vec::new(),
            base,
        }
    }

    fn ready(&mut self) {
        // 盖在一切之上、忽略鼠标
        self.base_mut().set_z_index(4096);
        self.base_mut().set_process_input(false);
    }

    fn process(&mut self, delta: f64) {
        if self.shapes.is_empty() {
            return;
        }
        let before = self.shapes.len();
        for s in &mut self.shapes {
            if s.life.is_finite() {
                s.life -= delta;
            }
        }
        self.shapes.retain(|s| s.life > 0.0);
        if self.shapes.len() != before {
            self.base_mut().queue_redraw();
        }
    }

    fn draw(&mut self) {
        let shapes = self.shapes.clone();
        for entry in shapes {
            let ShapeEntry { shape, color, .. } = entry;
            match shape {
                Shape::Line(from, to) => {
                    self.base_mut()
                        .draw_line_ex(from, to, color)
                        .width(1.5)
                        .done();
                }
                Shape::Rect(rect, filled) => {
                    self.base_mut()
                        .draw_rect_ex(rect, color)
                        .filled(filled)
                        .done();
                }
                Shape::Circle(center, radius, filled) => {
                    if filled {
                        self.base_mut()
                            .draw_circle_ex(center, radius, color)
                            .done();
                    } else {
                        self.base_mut().draw_arc(
                            center,
                            radius,
                            0.0,
                            std::f32::consts::TAU,
                            48,
                            color,
                        );
                    }
                }
                Shape::Ray(from, dir, len) => {
                    let n = dir.normalized_or_zero();
                    let to = from + n * len;
                    self.base_mut()
                        .draw_line_ex(from, to, color)
                        .width(1.5)
                        .done();
                    // 端头小箭头
                    let perp = Vector2::new(-n.y, n.x);
                    let tip = to;
                    let wing = n * -8.0 + perp * 4.0;
                    self.base_mut()
                        .draw_line_ex(tip, tip + wing, color)
                        .width(1.5)
                        .done();
                    let wing2 = n * -8.0 - perp * 4.0;
                    self.base_mut()
                        .draw_line_ex(tip, tip + wing2, color)
                        .width(1.5)
                        .done();
                }
            }
        }
    }
}

#[godot_api]
impl GdDebugDrawNode {
    /// 清空全部调试图形
    #[func]
    pub fn clear_all(&mut self) {
        self.shapes.clear();
        self.base_mut().queue_redraw();
    }

    /// 当前图形数量
    #[func]
    pub fn shape_count(&self) -> i64 {
        self.shapes.len() as i64
    }

    // ---- 由单例转发的内部入口（参数经 Variant 打包，减少绑定样板） ----

    #[func]
    pub fn add_line(&mut self, from: Vector2, to: Vector2, color: Color, life: f64) {
        self.push(Shape::Line(from, to), color, life);
    }

    #[func]
    pub fn add_rect(
        &mut self,
        pos: Vector2,
        size: Vector2,
        filled: bool,
        color: Color,
        life: f64,
    ) {
        self.push(Shape::Rect(Rect2::new(pos, size), filled), color, life);
    }

    #[func]
    pub fn add_circle(&mut self, center: Vector2, radius: f64, filled: bool, color: Color, life: f64) {
        self.push(Shape::Circle(center, radius as f32, filled), color, life);
    }

    #[func]
    pub fn add_ray(&mut self, from: Vector2, dir: Vector2, len: f64, color: Color, life: f64) {
        self.push(Shape::Ray(from, dir, len as f32), color, life);
    }

    fn push(&mut self, shape: Shape, color: Color, life: f64) {
        // life <= 0 = 常驻（直到 clear）
        let life = if life.is_finite() && life <= 0.0 {
            f64::INFINITY
        } else {
            life
        };
        self.shapes.push(ShapeEntry { shape, color, life });
        self.base_mut().queue_redraw();
    }
}

// ==================== Object 单例门面（手动内存） ====================

#[derive(GodotClass)]
#[class(base = Object)]
pub struct GdDebugDraw {
    /// 懒创建的画布节点
    canvas: Option<Gd<GdDebugDrawNode>>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GdDebugDraw {
    fn init(base: Base<Object>) -> Self {
        Self {
            canvas: None,
            base,
        }
    }
}

#[godot_api]
impl GdDebugDraw {
    /// 画线段
    #[func]
    pub fn draw_line(&mut self, from: Vector2, to: Vector2, color: Color, life: f64) {
        if let Some(mut c) = self.ensure_canvas() {
            c.bind_mut().add_line(from, to, color, life);
        }
    }

    /// 画矩形（左上角 + 尺寸）
    #[func]
    pub fn draw_rect(
        &mut self,
        pos: Vector2,
        size: Vector2,
        filled: bool,
        color: Color,
        life: f64,
    ) {
        if let Some(mut c) = self.ensure_canvas() {
            c.bind_mut().add_rect(pos, size, filled, color, life);
        }
    }

    /// 画圆（圆心 + 半径）
    #[func]
    pub fn draw_circle(
        &mut self,
        center: Vector2,
        radius: f64,
        filled: bool,
        color: Color,
        life: f64,
    ) {
        if let Some(mut c) = self.ensure_canvas() {
            c.bind_mut().add_circle(center, radius, filled, color, life);
        }
    }

    /// 画射线（起点 + 方向 + 长度，带箭头）
    #[func]
    pub fn draw_ray(&mut self, from: Vector2, dir: Vector2, len: f64, color: Color, life: f64) {
        if let Some(mut c) = self.ensure_canvas() {
            c.bind_mut().add_ray(from, dir, len, color, life);
        }
    }

    /// 清空全部
    #[func]
    pub fn clear(&mut self) {
        if let Some(c) = &mut self.canvas {
            if c.is_instance_valid() {
                c.bind_mut().clear_all();
            }
        }
    }

    /// 当前图形数量
    #[func]
    pub fn shape_count(&self) -> i64 {
        match &self.canvas {
            Some(c) if c.is_instance_valid() => c.bind().shape_count(),
            _ => 0,
        }
    }

    // ---- 内部实现 ----

    /// 确保画布节点已挂到场景根
    fn ensure_canvas(&mut self) -> Option<Gd<GdDebugDrawNode>> {
        if let Some(c) = &self.canvas {
            if c.is_instance_valid() {
                return Some(c.clone());
            }
        }
        // 找场景根
        let main_loop = Engine::singleton().get_main_loop()?;
        let tree: Gd<SceneTree> = main_loop.try_cast().ok()?;
        let mut root = tree.get_root()?;
        let mut node = Gd::<GdDebugDrawNode>::from_init_fn(|base| GdDebugDrawNode::init(base));
        node.set_name(&StringName::from("GdDebugDrawCanvas"));
        root.add_child(&node);
        self.canvas = Some(node.clone());
        Some(node)
    }
}

/// 注册为 Engine singleton "GDDEBUGDRAW"
pub fn register_gddebugdraw_singleton() {
    let instance = Gd::<GdDebugDraw>::from_init_fn(|base| GdDebugDraw::init(base));
    let name = StringName::from("GDDEBUGDRAW");
    Engine::singleton().register_singleton(&name, &instance);
    std::mem::forget(instance);
}

pub fn unregister_gddebugdraw_singleton() {
    let name = StringName::from("GDDEBUGDRAW");
    Engine::singleton().unregister_singleton(&name);
}
