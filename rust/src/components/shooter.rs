// GdShooter - 射击组件
// 挂在角色（Node2D 宿主，如 GdRoleMover）的子节点上，负责生成子弹：
//   - fire(direction)          朝给定方向发射
//   - fire_at_point(target)    朝世界坐标点发射
//   - fire_toward_mouse()      朝鼠标位置发射
//   - auto_fire_mouse 开启时，框架自动处理开火输入（见下）
//
// 开火输入路由（设计约定：凡「点击世界」类动作一律走 unhandled 阶段）：
//   auto_fire_mouse = true 时，GdShooter 在 unhandled_input 接收鼠标按键——
//   被 UI 控件（mouse_filter=STOP）消费的点击到不了这里，引擎保证
//   「收到 = 点击落在世界，收不到 = 点击被 UI 吃掉」，无需 hover 猜测。
//   按下即开火并置 fire_held，_process 按住连射（冷却内置）；抬起复位。
//   防卡死兜底：release 若被 UI 吞掉（按住拖进控件再松开），process 中用
//   Input 轮询校准复位 fire_held。
//   auto_fire_mouse = false 时完全由外部脚本驱动（调 fire_toward_mouse）。
//
// 子弹经 GdSpawnPool 生成/回收（bullet_scene_path 首次射击时自动注册进池），
// 每发子弹的 速度/伤害/寿命/池别名 由本组件写入 GdBullet。
//
// 瞄准框（show_aim_frame，默认显示）：宿主周边 aim_distance 固定距离、
// 跟随鼠标方向的四角括线框（aim_frame_size 边长），懒创建挂宿主下的
// GdShooterAimFrame 节点，每帧定位——辅助瞄准纯视觉，不影响判定/开火。
//
// 信号：
//   s_fired(muzzle, direction)  每次成功发射

use godot::prelude::*;
use godot::builtin::{GString, StringName, Variant, Vector2};
use godot::classes::{
    Engine, INode, INode2D, Input, InputEvent, InputEventMouseButton, Node, Node2D, Window,
};
use godot::global::MouseButton;

use super::bullet::GdBullet;

// ==================== 瞄准框节点 ====================

/// GdShooterAimFrame - 瞄准框（四角括线 + 呼吸透明度）
/// 由 GdShooter 懒创建挂到宿主下（跟随宿主移动），每帧定位到
/// 宿主 -> 鼠标方向固定距离处。仅视觉指示，不参与任何判定。
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdShooterAimFrame {
    /// 框边长（像素，由 GdShooter.aim_frame_size 每帧同步）
    frame_size: f32,
    /// 呼吸相位
    phase: f64,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdShooterAimFrame {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            frame_size: 20.0,
            phase: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        // 盖在地图/角色之上（Node2D 无 mouse_filter，天然不挡输入）
        self.base_mut().set_z_index(100);
        self.base_mut().set_visible(false);
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
        let h = self.frame_size * 0.5;
        if h <= 0.0 {
            return;
        }
        let l = (self.frame_size * 0.36).max(2.0); // 括线臂长
        // 呼吸：透明度 0.65..0.95 缓动
        let a = 0.65 + 0.3 * (self.phase * 4.0).sin() as f32;
        let color = Color::from_rgba(1.0, 0.85, 0.35, a);
        // 正方形四角，各角两条臂沿两边收拢
        let corners = [
            Vector2::new(-h, -h),
            Vector2::new(h, -h),
            Vector2::new(h, h),
            Vector2::new(-h, h),
        ];
        let mut b = self.base_mut();
        for i in 0..4 {
            let c = corners[i];
            let prev = corners[(i + 3) % 4];
            let next = corners[(i + 1) % 4];
            b.draw_line(c, c + (prev - c).normalized() * l, color);
            b.draw_line(c, c + (next - c).normalized() * l, color);
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdShooter {
    /// 是否启用射击
    #[export]
    enabled: bool,

    /// 框架自动处理开火输入：unhandled 阶段路由（点击 UI 不开火，点击世界开火）
    #[export]
    auto_fire_mouse: bool,

    /// 开火按键：鼠标左键
    #[export]
    fire_button_left: bool,

    /// 开火按键：鼠标右键
    #[export]
    fire_button_right: bool,

    /// 射击冷却（秒）
    #[export]
    fire_cooldown: f64,

    /// 子弹初速（像素/秒）
    #[export]
    bullet_speed: f64,

    /// 子弹存活时长（秒）
    #[export]
    bullet_lifetime: f64,

    /// 单发伤害
    #[export]
    bullet_damage: f64,

    /// 枪口距宿主中心的偏移距离（像素）
    #[export]
    muzzle_distance: f64,

    /// 子弹最大射程（像素，飞行距离超限销毁）；0 = 不限
    #[export]
    max_distance: f64,

    /// 子弹池别名
    #[export]
    bullet_alias: GString,

    /// 子弹场景路径（首次射击时自动注册进 GdSpawnPool）
    #[export]
    bullet_scene_path: GString,

    /// 是否显示瞄准框（宿主周边固定距离、跟随鼠标方向的辅助瞄准指示；
    /// 编辑器可配，默认显示）
    #[export]
    show_aim_frame: bool,

    /// 瞄准框中心距宿主中心的固定距离（像素）
    #[export]
    aim_distance: f64,

    /// 瞄准框边长（像素）
    #[export]
    aim_frame_size: f64,

    // ---- 运行时状态 ----
    cooldown_timer: f64,
    pool_ready: bool,
    /// unhandled 阶段收到开火键按下 → 按住连射；抬起/校准复位
    fire_held: bool,
    /// 懒创建的瞄准框节点（挂宿主下，随宿主移动）
    aim_frame: Option<Gd<GdShooterAimFrame>>,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdShooter {
    fn init(base: Base<Node>) -> Self {
        Self {
            enabled: true,
            auto_fire_mouse: true,
            fire_button_left: true,
            fire_button_right: false,
            fire_cooldown: 0.25,
            bullet_speed: 480.0,
            bullet_lifetime: 1.5,
            bullet_damage: 20.0,
            muzzle_distance: 24.0,
            max_distance: 0.0,
            bullet_alias: GString::from("bullet"),
            bullet_scene_path: GString::new(),
            show_aim_frame: true,
            aim_distance: 64.0,
            aim_frame_size: 20.0,
            cooldown_timer: 0.0,
            pool_ready: false,
            fire_held: false,
            aim_frame: None,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer = (self.cooldown_timer - delta).max(0.0);
        }
        // 瞄准框每帧跟随（enabled/暂停/配置关 → 隐藏），与开火逻辑解耦
        self.update_aim_frame();
        if !self.enabled || !self.auto_fire_mouse || !self.fire_held {
            return;
        }
        // 防卡死校准：release 被控件吞掉（按住拖进 UI 再松开）时，Input 状态
        // 已复位但 fire_held 收不到 release——以 Input 实际状态为准复位
        let input = Input::singleton();
        let pressed = (self.fire_button_left && input.is_mouse_button_pressed(MouseButton::LEFT))
            || (self.fire_button_right && input.is_mouse_button_pressed(MouseButton::RIGHT));
        if !pressed {
            self.fire_held = false;
            return;
        }
        if self.host_paused() {
            return;
        }
        self.fire_toward_mouse();
    }

    /// 开火输入路由：unhandled 阶段 = 点击落在了世界上。
    /// 被 mouse_filter=STOP 的 UI 控件消费的点击到不了这里（引擎保证），
    /// 因此「点击 UI 不开火、点击地图/世界开火」无需任何 hover 猜测。
    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if !self.enabled || !self.auto_fire_mouse {
            return;
        }
        // 对话/剧情锁：宿主实现 is_paused()（如 GdRoleMover）时暂停期不接开火输入
        if self.host_paused() {
            return;
        }
        let Ok(btn) = event.try_cast::<InputEventMouseButton>() else {
            return;
        };
        let idx = btn.get_button_index();
        let is_fire_button = (idx == MouseButton::LEFT && self.fire_button_left)
            || (idx == MouseButton::RIGHT && self.fire_button_right);
        if !is_fire_button {
            return;
        }
        if btn.is_pressed() {
            self.fire_held = true;
            if self.fire_toward_mouse() {
                // 世界点击已被「开火」消费，不再下传（防止同时触发寻路等）
                if let Some(mut vp) = self.base().get_viewport() {
                    vp.set_input_as_handled();
                }
            }
        } else {
            self.fire_held = false;
        }
    }
}

#[godot_api]
impl GdShooter {
    /// 每次成功发射时触发（参数：枪口世界坐标、飞行方向）
    #[signal]
    fn s_fired(muzzle: Vector2, direction: Vector2);

    /// 剩余冷却时间（秒）
    #[func]
    pub fn get_cooldown_left(&self) -> f64 {
        self.cooldown_timer
    }

    /// 朝给定方向发射一发子弹，冷却中返回 false
    #[func]
    pub fn fire(&mut self, direction: Vector2) -> bool {
        if !self.enabled || self.cooldown_timer > 0.0 {
            return false;
        }
        if direction.length_squared() < 0.0001 {
            return false;
        }
        let dir = direction.normalized();
        let Some(origin) = self.host_position() else {
            godot_error!("[GdShooter] 宿主不是 Node2D，无法定位枪口");
            return false;
        };
        let muzzle = origin + dir * self.muzzle_distance as f32;

        let Some(bullet) = self.spawn_bullet(muzzle) else {
            return false;
        };

        // 朝向与参数写入子弹
        if let Ok(mut n2d) = bullet.clone().try_cast::<Node2D>() {
            n2d.set_rotation(dir.angle());
        }
        if let Ok(mut b) = bullet.try_cast::<GdBullet>() {
            let speed = self.bullet_speed;
            let damage = self.bullet_damage;
            let lifetime = self.bullet_lifetime;
            let alias = self.bullet_alias.clone();
            let max_distance = self.max_distance as f32;
            b.bind_mut()
                .setup(dir * speed as f32, damage, lifetime, alias, max_distance);
        }

        self.cooldown_timer = self.fire_cooldown;
        self.base_mut()
            .emit_signal("s_fired", &[muzzle.to_variant(), dir.to_variant()]);
        true
    }

    /// 朝世界坐标点发射
    #[func]
    pub fn fire_at_point(&mut self, target: Vector2) -> bool {
        let Some(origin) = self.host_position() else {
            return false;
        };
        self.fire(target - origin)
    }

    /// 朝鼠标当前位置发射
    #[func]
    pub fn fire_toward_mouse(&mut self) -> bool {
        let Some(dir) = self.aim_direction() else {
            return false;
        };
        self.fire(dir)
    }

    // ---- 内部实现 ----

    /// 瞄准框跟随：宿主 -> 鼠标方向固定距离处显示四角括线框（每帧刷新）。
    /// 关闭配置/组件禁用/宿主暂停/拿不到瞄准方向（headless 无鼠标且宿主在
    /// 原点等）时隐藏。仅视觉指示，不影响判定与开火。
    fn update_aim_frame(&mut self) {
        if !self.show_aim_frame || !self.enabled || self.host_paused() {
            self.hide_aim_frame();
            return;
        }
        let Some(host) = self.base().get_parent() else {
            return;
        };
        let Ok(mut host2d) = host.try_cast::<Node2D>() else {
            return; // 非 Node2D 宿主：无瞄准语义，静默跳过
        };
        let Some(dir) = self.aim_direction() else {
            self.hide_aim_frame();
            return;
        };
        // 懒创建：挂宿主下（跟随宿主移动），随宿主释放自动回收
        let need_create = match &self.aim_frame {
            Some(f) => !f.is_instance_valid(),
            None => true,
        };
        if need_create {
            let mut node = Gd::<GdShooterAimFrame>::from_init_fn(|base| GdShooterAimFrame::init(base));
            node.set_name(&StringName::from("AimFrame"));
            host2d.add_child(&node);
            self.aim_frame = Some(node);
        }
        if let Some(mut f) = self.aim_frame.clone() {
            f.set_visible(true);
            f.set_position(dir * self.aim_distance as f32);
            f.bind_mut().frame_size = self.aim_frame_size as f32;
        }
    }

    /// 隐藏瞄准框（懒创建前调用为空操作；宿主已释放时静默跳过）
    fn hide_aim_frame(&mut self) {
        if let Some(mut f) = self.aim_frame.clone() {
            if f.is_instance_valid() {
                f.set_visible(false);
            }
        }
    }

    /// 宿主（父节点）全局位置
    fn host_position(&self) -> Option<Vector2> {
        let host = self.base().get_parent()?;
        let host2d = host.try_cast::<Node2D>().ok()?;
        Some(host2d.get_global_position())
    }

    /// 宿主暂停判定（duck-type：宿主实现 is_paused() 即生效，如 GdRoleMover
    /// 的对话/剧情锁）。非 Node2D 宿主或未实现时不视为暂停。
    fn host_paused(&self) -> bool {
        let Some(host) = self.base().get_parent() else {
            return false;
        };
        let mut obj = host.upcast::<Object>();
        if obj.has_method(&StringName::from("is_paused")) {
            return obj
                .call(&StringName::from("is_paused"), &[])
                .try_to::<bool>()
                .unwrap_or(false);
        }
        false
    }

    /// 瞄准方向：宿主 -> 鼠标（经宿主 CanvasItem 换算，自动适配相机）
    fn aim_direction(&self) -> Option<Vector2> {
        let host = self.base().get_parent()?;
        let host2d = host.try_cast::<Node2D>().ok()?;
        let mouse = host2d.get_global_mouse_position();
        let diff = mouse - host2d.get_global_position();
        if diff.length_squared() < 0.0001 {
            return None;
        }
        Some(diff.normalized())
    }

    /// 经对象池生成子弹（首次自动注册池），失败返回 None
    fn spawn_bullet(&mut self, pos: Vector2) -> Option<Gd<Node>> {
        let engine = Engine::singleton();
        let Some(mut pool) = engine.get_singleton("GDSPAWNPOOL") else {
            godot_error!("[GdShooter] GDSPAWNPOOL 单例不存在");
            return None;
        };
        // 懒注册：首次射击把子弹场景注册进池并预热
        if !self.pool_ready {
            let alias = self.bullet_alias.clone();
            let path = self.bullet_scene_path.clone();
            if alias.is_empty() || path.is_empty() {
                godot_error!("[GdShooter] bullet_alias / bullet_scene_path 未配置");
                return None;
            }
            let ok = pool.call(
                &StringName::from("register"),
                &[alias.to_variant(), path.to_variant(), 12i64.to_variant()],
            );
            if !ok.try_to::<bool>().unwrap_or(false) {
                return None;
            }
            self.pool_ready = true;
        }
        // 父节点：优先当前场景，无当前场景（headless 测试等）时兜底挂到根
        let parent = self.spawn_parent();
        let ret = pool.call(
            &StringName::from("spawn"),
            &[
                self.bullet_alias.to_variant(),
                parent.map(|p| p.to_variant()).unwrap_or(Variant::nil()),
                pos.to_variant(),
            ],
        );
        ret.try_to::<Gd<Node>>().ok()
    }

    /// 子弹挂载父节点：当前场景 > 场景树根
    fn spawn_parent(&self) -> Option<Gd<Node>> {
        // 用 _or_null 版本：宿主尚未入树时 get_tree() 会 panic
        let Some(tree) = self.base().get_tree_or_null() else {
            godot_warn!("[GdShooter] 宿主不在场景树中，无法挂载子弹");
            return None;
        };
        if let Some(cs) = tree.get_current_scene() {
            return Some(cs);
        }
        let root: Gd<Window> = tree.get_root()?;
        Some(root.upcast::<Node>())
    }
}
