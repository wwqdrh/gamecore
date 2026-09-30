// 绘图模块
// geometry:      纯几何轮廓生成（树叶 / 圆 / 点集变换），无 Godot 类依赖
// colors:        调色板与颜色工具（hex 解析、插值、秋季叶色）
// GdCanvasDrawer: 队列式画布绘制节点（矩形 / 圆 / 线 / 多边形 / 树叶）
// GdColorUtil:   颜色工具的 Godot 静态方法接口
//
// 典型用法（GDScript）：
//   var d := GdCanvasDrawer.new()
//   add_child(d)
//   d.add_rect(Vector2.ZERO, Vector2(100, 50), Color.RED, true)
//   d.add_leaf(Vector2(50, 25), 20.0, 45.0, Color.GREEN)

pub mod geometry;
pub mod colors;
pub mod canvas_drawer;

pub use canvas_drawer::GdCanvasDrawer;
pub use colors::GdColorUtil;
