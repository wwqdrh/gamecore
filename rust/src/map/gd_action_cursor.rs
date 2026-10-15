// GdActionCursor - 地图动作预览光标（继承 Node2D）
//
// 通用的「下一击将处理的格子」高亮组件：由地图根（GdQuickMap / GdDigLayer /
// GdMapBasic 或任何实现了 cell_to_center 的 Node2D）在 ready 创建为子节点，
// 经 set_cursor_for / set_cursor_cell / clear_cursor 统一协议驱动。
//
// 职责单一：显示（定位到格心 + 按动作着色 + 可选贴图/默认框 + 呼吸动画）。
// 「这个格子能不能执行这个动作」的谓词判定留在地图根——不同地图语义不同
// （农田层看地形状态，随机地图查 deco 矿石），光标组件零业务感知。
//
// 定位约定（全项目唯一）：逻辑格心 = (cell + 0.5) * cell_size（世界语义）。
// 父级是地图根（根恒单位变换，见 map.md 架构红线）；show_at 经父级
// cell_to_center + to_local 换算，与 GdMapMarker.snap_to_cell 同款自愈防御
// （父级万一带变换也不会飘出地图）。
//
// 样式：texture 配置 → 贴图缩放到单格；未配置 → 动作色半透明填充 + 实线边框。
// 动作色约定：dig=白 / water=蓝 / plant=绿 / harvest=金 / mine=橙（矿镐）。

use godot::prelude::*;

use godot::classes::{INode2D, Node2D, Texture2D};

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdActionCursor {
    /// 格子尺寸（像素，世界语义；随宿主地图 cell_size 配置）
    #[export]
    cell_size: i32,
    /// 光标贴图（可选；配置后缩放到单格铺满，未配置画动作色框）
    #[export]
    texture: Option<Gd<Texture2D>>,

    /// 当前动作（决定框色；"dig"/"water"/"plant"/"harvest"/"mine"）
    action: GString,
    /// 呼吸动画相位
    phase: f64,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdActionCursor {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            cell_size: 32,
            texture: None,
            action: GString::new(),
            phase: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_visible(false);
        self.base_mut().set_process(true);
    }

    fn process(&mut self, delta: f64) {
        // 仅可见时呼吸（隐藏态零重绘开销）
        if !self.base().is_visible() {
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
        let color = action_color(&self.action.to_string());
        // 呼吸：透明度 0.55..1.0 缓动（同一节奏，颜色不变）
        let a = 0.775 + 0.225 * (self.phase * 4.0).sin() as f32;
        let rect = Rect2::new(Vector2::new(-cs / 2.0, -cs / 2.0), Vector2::new(cs, cs));

        let tex = self.texture.clone();
        if let Some(t) = tex.filter(|t| t.is_instance_valid()) {
            self.base_mut()
                .draw_texture_rect_ex(&t, rect, false)
                .modulate(Color::from_rgba(1.0, 1.0, 1.0, a))
                .done();
            return;
        }
        // 默认样式：动作色半透明填充 + 实线边框
        let mut fill = color;
        fill.a = 0.16 * a;
        self.base_mut().draw_rect_ex(rect, fill).filled(true).done();
        let mut line = color;
        line.a = a;
        self.base_mut()
            .draw_rect_ex(rect, line)
            .filled(false)
            .width(2.0)
            .done();
    }
}

#[godot_api]
impl GdActionCursor {
    /// 显示光标于 cell（世界语义格心 = (cell + 0.5) * cell_size）。
    /// 父级是地图根——按架构红线恒单位变换（map.md），格心即局部坐标；
    /// **严禁经父级 cell_to_center 回调定位**：地图根在 set_cursor_for
    /// 持有可变借用期间驱动本组件，回调会单借用重入 panic
    #[func]
    pub fn show_at(&mut self, cell: Vector2i, action: GString) {
        self.action = action;
        self.phase = 0.0;
        let cs = self.cell_size.max(1) as f32;
        let local = Vector2::new(
            (cell.x as f32 + 0.5) * cs,
            (cell.y as f32 + 0.5) * cs,
        );
        self.base_mut().set_position(local);
        self.base_mut().set_visible(true);
        self.base_mut().queue_redraw();
    }

    /// 隐藏光标
    #[func]
    pub fn hide_cursor(&mut self) {
        self.base_mut().set_visible(false);
    }
}

/// 动作 → 框色（未识别动作回退白色）
fn action_color(action: &str) -> Color {
    match action {
        "water" => Color::from_rgba(0.45, 0.75, 1.0, 1.0),
        "plant" => Color::from_rgba(0.45, 0.9, 0.45, 1.0),
        "harvest" => Color::from_rgba(1.0, 0.85, 0.35, 1.0),
        "mine" => Color::from_rgba(1.0, 0.6, 0.3, 1.0),
        _ => Color::from_rgba(1.0, 1.0, 1.0, 1.0),
    }
}
