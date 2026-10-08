// GdMapMarker - 地图传送点/标记（继承 Node2D，tool）
//
// 放置在地图场景中（建议作为 GdQuickMap 根的子节点），在指定格子上绘制
// 高亮呼吸亮框，用于标记传送门/出口/兴趣点：
//   - ready 时按 cell 自动定位到格子中心（地图根位于原点的约定）
//   - 运行时亮框呼吸脉冲（编辑器内静态预览）
//   - 携带 target_alias/target_cell：由 GdMapManager.try_teleport 消费，
//     后续人物网格走动到达该格时调用 try_teleport(角色世界坐标) 即可触发传送
//
// 与 GdMapManager 的协作约定：
//   - ready 时自动加入 "__gdmap_marker" 分组，管理器经分组收集当前地图的传送点
//   - contains_world_point(world_pos) 判定世界坐标是否落在本格内
//
// 导出字段（cell/cell_size/target_alias/target_cell 等）由 godot-rust 自动生成
// get_/set_ 方法，此处不再手写 getter。
//
// 视觉分层：半透明填充 → 外圈光晕环（呼吸）→ 亮框描边 → 四角高亮 → 中心光点

use godot::prelude::*;
use godot::builtin::{Color, GString, Rect2, Vector2, Vector2i};
use godot::classes::{Engine, INode2D, Node2D, ThemeDb};

/// 传送点分组名（GdMapManager 经此收集传送点）
pub const MAP_MARKER_GROUP: &str = "__gdmap_marker";

#[derive(GodotClass)]
#[class(base = Node2D, tool)]
pub struct GdMapMarker {
    /// 所在格子坐标（ready 时自动定位到格心）
    #[export]
    cell: Vector2i,
    /// 格子尺寸（像素，需与地图 cell_size 一致）
    #[export]
    cell_size: i32,
    /// 高亮颜色（默认传送门青色）
    #[export]
    color: Color,
    /// 运行时是否呼吸脉冲
    #[export]
    pulse: bool,
    /// 目标地图别名（空 = 仅标记不传送）
    #[export]
    target_alias: GString,
    /// 传送到目标地图的落点格子
    #[export]
    target_cell: Vector2i,
    /// 亮框上方显示的文字（空则不显示，如"前往幽竹林"）
    #[export]
    label: GString,

    /// 脉冲相位累计（秒）
    phase: f64,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdMapMarker {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            cell: Vector2i::ZERO,
            cell_size: 32,
            color: Color::from_rgb(0.35, 0.95, 1.0),
            pulse: true,
            target_alias: GString::new(),
            target_cell: Vector2i::ZERO,
            label: GString::new(),
            phase: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        // 按格子坐标自动定位到格心（约定：父级即地图根，位于原点）
        let cs = self.cell_size as f32;
        let pos = Vector2::new(
            (self.cell.x as f32 + 0.5) * cs,
            (self.cell.y as f32 + 0.5) * cs,
        );
        self.base_mut().set_position(pos);

        // 加入分组供 GdMapManager 收集（编辑器内也加入，便于场景内统计）
        self.base_mut().add_to_group(MAP_MARKER_GROUP);
        self.base_mut().queue_redraw();
    }

    fn process(&mut self, delta: f64) {
        // 编辑器内静态预览，运行时才呼吸
        if Engine::singleton().is_editor_hint() || !self.pulse {
            return;
        }
        self.phase += delta;
        self.base_mut().queue_redraw();
    }

    fn draw(&mut self) {
        let cs = self.cell_size as f32;
        if cs <= 0.0 {
            return;
        }
        let half = cs * 0.5;
        let rect = Rect2::new(Vector2::new(-half, -half), Vector2::new(cs, cs));
        // 呼吸系数 [0, 1]：sin 半周期归一化
        let t = ((self.phase * 2.2).sin() * 0.5 + 0.5) as f32;
        let c = self.color;

        // 1. 半透明填充
        let fill = Color::from_rgba(c.r, c.g, c.b, 0.16 + 0.10 * t);
        self.base_mut().draw_rect(rect, fill);

        // 2. 外圈光晕环（呼吸外扩 + 淡出）
        let grow = 3.0 + 5.0 * t;
        let glow = Rect2::new(
            Vector2::new(-half - grow, -half - grow),
            Vector2::new(cs + grow * 2.0, cs + grow * 2.0),
        );
        let glow_color = Color::from_rgba(c.r, c.g, c.b, 0.35 * (1.0 - t));
        self.base_mut()
            .draw_rect_ex(glow, glow_color)
            .filled(false)
            .width(2.0)
            .done();

        // 3. 亮框描边
        let frame = Color::from_rgba(c.r, c.g, c.b, 0.75 + 0.25 * t);
        self.base_mut()
            .draw_rect_ex(rect, frame)
            .filled(false)
            .width(2.0)
            .done();

        // 4. 四角高亮短边（L 形角标）
        let arm = cs * 0.28;
        let corners: [(Vector2, Vector2, Vector2, Vector2); 4] = [
            // 左上：竖边、横边
            (
                Vector2::new(-half, -half + arm),
                Vector2::new(-half, -half),
                Vector2::new(-half, -half),
                Vector2::new(-half + arm, -half),
            ),
            // 右上
            (
                Vector2::new(half - arm, -half),
                Vector2::new(half, -half),
                Vector2::new(half, -half),
                Vector2::new(half, -half + arm),
            ),
            // 右下
            (
                Vector2::new(half, half - arm),
                Vector2::new(half, half),
                Vector2::new(half, half),
                Vector2::new(half - arm, half),
            ),
            // 左下
            (
                Vector2::new(-half + arm, half),
                Vector2::new(-half, half),
                Vector2::new(-half, half),
                Vector2::new(-half, half - arm),
            ),
        ];
        for (a, b, cc, d) in corners {
            self.base_mut().draw_line(a, b, c);
            self.base_mut().draw_line(cc, d, c);
        }

        // 5. 中心光点（呼吸缩放）
        let dot_r = 3.0 + 2.0 * t;
        let dot = Color::from_rgba(c.r, c.g, c.b, 0.9);
        self.base_mut().draw_circle(Vector2::ZERO, dot_r, dot);

        // 6. 可选文字标签（亮框上方居中）
        let text = self.label.to_string();
        if !text.is_empty() {
            if let Some(font) = ThemeDb::singleton().get_fallback_font() {
                let fs = 13;
                let text_gd = GString::from(text.as_str());
                let size = font
                    .get_string_size_ex(&text_gd)
                    .width(-1.0)
                    .font_size(fs)
                    .done();
                let pos = Vector2::new(-size.x * 0.5, -half - 8.0);
                let label_color = Color::from_rgba(1.0, 1.0, 1.0, 0.9);
                self.base_mut()
                    .draw_string_ex(&font, pos, &text_gd)
                    .font_size(fs)
                    .modulate(label_color)
                    .done();
            }
        }
    }
}

#[godot_api]
impl GdMapMarker {
    /// 传送触发信号（GdMapManager 命中本格时发出）
    #[signal]
    fn s_triggered();

    /// 放置到指定格子（更新 cell 并同步位置；GdMapManager 吸附不可通行格子时调用）
    #[func]
    pub fn place_at(&mut self, cell: Vector2i) {
        self.cell = cell;
        let cs = self.cell_size as f32;
        let pos = Vector2::new((cell.x as f32 + 0.5) * cs, (cell.y as f32 + 0.5) * cs);
        self.base_mut().set_position(pos);
        self.base_mut().queue_redraw();
    }

    /// 本格中心的全局坐标
    #[func]
    fn get_center(&self) -> Vector2 {
        self.base().get_global_position()
    }

    /// 世界坐标是否落在本格矩形内（含边界）
    #[func]
    fn contains_world_point(&self, world_pos: Vector2) -> bool {
        let half = self.cell_size as f32 * 0.5;
        let gp = self.base().get_global_position();
        world_pos.x >= gp.x - half
            && world_pos.x <= gp.x + half
            && world_pos.y >= gp.y - half
            && world_pos.y <= gp.y + half
    }
}
