// GdCanvasDrawer - 队列式画布绘制节点
// add_* 方法把图形加入绘制队列并触发重绘，_draw 时统一渲染；
// 树叶轮廓来自 geometry，颜色加深/调色板来自 colors。
// 与"每帧改队列再重绘"配合即可做简单动画（参考 GdWeather 的刮风树叶）。

use godot::prelude::*;
use godot::classes::{INode2D, Node2D};

use super::{colors, geometry};

#[derive(Clone)]
enum Shape {
    /// 矩形 (Rect2, 颜色, 是否填充)
    Rect(Rect2, Color, bool),
    /// 圆 (圆心, 半径, 颜色, 是否填充)
    Circle(Vector2, f32, Color, bool),
    /// 粗线段 (起点, 终点, 颜色, 线宽)
    Line(Vector2, Vector2, Color, f32),
    /// 多边形 (点集, 颜色, 是否填充)
    Poly(PackedVector2Array, Color, bool),
    /// 树叶 (叶心, 叶长, 旋转弧度, 颜色)
    Leaf(Vector2, f32, f32, Color),
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdCanvasDrawer {
    shapes: Vec<Shape>,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdCanvasDrawer {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            shapes: Vec::new(),
            base,
        }
    }

    fn draw(&mut self) {
        // 克隆一份遍历，避免遍历 shapes 与 base_mut() 的借用冲突
        let shapes = self.shapes.clone();
        for shape in &shapes {
            match shape {
                Shape::Rect(rect, color, filled) => {
                    self.base_mut()
                        .draw_rect_ex(*rect, *color)
                        .filled(*filled)
                        .done();
                }
                Shape::Circle(center, radius, color, filled) => {
                    if *filled {
                        self.base_mut()
                            .draw_circle_ex(*center, *radius, *color)
                            .done();
                    } else {
                        self.base_mut().draw_arc(
                            *center,
                            *radius,
                            0.0,
                            std::f32::consts::TAU,
                            48,
                            *color,
                        );
                    }
                }
                Shape::Line(from, to, color, width) => {
                    self.base_mut()
                        .draw_line_ex(*from, *to, *color)
                        .width(*width)
                        .done();
                }
                Shape::Poly(points, color, filled) => {
                    if *filled {
                        self.base_mut().draw_colored_polygon(points, *color);
                    } else {
                        self.base_mut()
                            .draw_polyline_ex(points, *color)
                            .width(1.5)
                            .done();
                    }
                }
                Shape::Leaf(center, length, rotation, color) => {
                    let outline = geometry::leaf_outline(*length, 0.55);
                    let pts = geometry::place(&outline, *center, *rotation);
                    self.base_mut().draw_colored_polygon(&pts, *color);
                    // 叶脉：叶尖 → 叶基的细分线
                    let vein_color = colors::darkened(*color, 0.4);
                    let v0 = Vector2::new(0.0, length * 0.12);
                    let v1 = Vector2::new(0.0, length * 0.88);
                    let (s, c) = rotation.sin_cos();
                    let world = |p: Vector2| {
                        Vector2::new(
                            p.x * c - p.y * s + center.x,
                            p.x * s + p.y * c + center.y,
                        )
                    };
                    self.base_mut()
                        .draw_line_ex(world(v0), world(v1), vein_color)
                        .width(1.0)
                        .done();
                }
            }
        }
    }
}

#[godot_api]
impl GdCanvasDrawer {
    /// 添加矩形（pos 为左上角）
    #[func]
    pub fn add_rect(&mut self, pos: Vector2, size: Vector2, color: Color, filled: bool) {
        self.push(Shape::Rect(Rect2::new(pos, size), color, filled));
    }

    /// 添加圆
    #[func]
    pub fn add_circle(&mut self, center: Vector2, radius: f32, color: Color, filled: bool) {
        self.push(Shape::Circle(center, radius, color, filled));
    }

    /// 添加线段
    #[func]
    pub fn add_line(&mut self, from: Vector2, to: Vector2, color: Color, width: f32) {
        self.push(Shape::Line(from, to, color, width.max(0.5)));
    }

    /// 添加多边形（点集为世界/节点局部坐标）
    #[func]
    pub fn add_polygon(&mut self, points: PackedVector2Array, color: Color, filled: bool) {
        self.push(Shape::Poly(points, color, filled));
    }

    /// 添加树叶（叶心为中心点，rotation_deg 为旋转角度）
    #[func]
    pub fn add_leaf(&mut self, center: Vector2, length: f32, rotation_deg: f32, color: Color) {
        self.push(Shape::Leaf(
            center,
            length.max(2.0),
            rotation_deg.to_radians(),
            color,
        ));
    }

    /// 清空绘制队列
    #[func]
    pub fn clear_shapes(&mut self) {
        self.shapes.clear();
        self.base_mut().queue_redraw();
    }

    /// 当前队列中的图形数量
    #[func]
    pub fn shape_count(&self) -> i32 {
        self.shapes.len() as i32
    }

    /// 树叶轮廓点集（叶尖在原点、沿 +Y 延伸；静态方法，供外部生成自定义叶形）
    #[func]
    pub fn leaf_outline_points(length: f32, fatness: f32) -> PackedVector2Array {
        geometry::leaf_outline(length, fatness)
    }

    fn push(&mut self, shape: Shape) {
        self.shapes.push(shape);
        self.base_mut().queue_redraw();
    }
}
