// 纯几何轮廓生成：树叶、圆、点集变换
// 输出均为"局部坐标"点集（叶尖/圆心在原点），用 place() 变换到目标位置，
// 供 GdCanvasDrawer 与环境系统（刮风树叶）共用

use godot::prelude::*;

/// 树叶轮廓：叶尖在原点 (0,0)，沿 +Y 方向延伸 length，
/// 由两段二次贝塞尔（右叶缘 + 左叶缘镜像）合成闭合多边形。
/// fatness: 叶片胖瘦系数（1.0 = 标准叶形，越大越宽）
pub fn leaf_outline(length: f32, fatness: f32) -> PackedVector2Array {
    let n = 12; // 每侧采样点数
    let w = length * 0.5 * fatness.clamp(0.1, 2.0);
    let half = length * 0.5;
    let mut pts = PackedVector2Array::new();
    // 右叶缘：P0=(0,0) P1=(w, half) P2=(0, length)
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let it = 1.0 - t;
        pts.push(Vector2::new(
            2.0 * it * t * w,
            2.0 * it * t * half + t * t * length,
        ));
    }
    // 左叶缘（跳过两端重复点，倒序走回叶尖）
    for i in (1..n).rev() {
        let t = i as f32 / n as f32;
        let it = 1.0 - t;
        pts.push(Vector2::new(
            -2.0 * it * t * w,
            2.0 * it * t * half + t * t * length,
        ));
    }
    pts
}

/// 圆轮廓（segments 边近似，segments < 3 时按 3 处理）
pub fn circle_outline(radius: f32, segments: i32) -> PackedVector2Array {
    let n = segments.max(3);
    let mut pts = PackedVector2Array::new();
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        pts.push(Vector2::new(a.cos() * radius, a.sin() * radius));
    }
    pts
}

/// 将局部轮廓旋转（弧度）后平移到 origin
pub fn place(points: &PackedVector2Array, origin: Vector2, rotation: f32) -> PackedVector2Array {
    let (s, c) = rotation.sin_cos();
    let mut out = PackedVector2Array::new();
    for p in points.as_slice() {
        out.push(Vector2::new(
            p.x * c - p.y * s + origin.x,
            p.x * s + p.y * c + origin.y,
        ));
    }
    out
}
