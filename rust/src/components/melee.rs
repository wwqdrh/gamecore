// GdMelee - 近战攻击组件
// 挂在角色（Node2D 宿主，如 GdRoleMover）的子节点上，负责近战挥击判定：
//   - swing(direction)         朝给定方向挥击
//   - swing_at_point(target)   朝世界坐标点挥击
//   - swing_toward_mouse()     朝鼠标位置挥击
//   - auto_attack_mouse 开启时，框架自动处理攻击输入（见下）
//
// 攻击输入路由（与 GdShooter 同套约定：凡「点击世界」类动作一律走 unhandled 阶段）：
//   auto_attack_mouse = true 时，GdMelee 在 unhandled_input 接收鼠标按键——
//   被 UI 控件（mouse_filter=STOP）消费的点击到不了这里，引擎保证
//   「收到 = 点击落在世界」，无需 hover 猜测。按下即挥击并置 attack_held，
//   _process 按住连击（冷却内置）；抬起复位。release 被吞时以 Input 实际
//   状态校准复位（防卡死，同 GdShooter）。
//
// 判定结构：ready 时创建内嵌 GdHitbox 子节点（SwingHitbox，Area2D）——
//   collision_layer=0 / monitorable=false（纯攻击方），collision_mask=target_mask
//   （默认 4 = 敌受击盒 layer3，与 GdBullet 一致）。挥击 = 把 SwingHitbox
//   定位到 dir * attack_offset（rotation = dir.angle()）并使能 swing_window
//   秒（重叠扫描与伤害由 GdHitbox 的 physics_process 完成），窗口结束复位。
//   窗口期命中冷却由 GdHitbox 自身 cooldown 控制（hit_cooldown）。
//
// 信号：
//   s_swing(direction)  每次成功挥击

use godot::prelude::*;
use godot::builtin::{StringName, Vector2};
use godot::classes::{
    CollisionShape2D, INode2D, Input, InputEvent, InputEventMouseButton, Node, Node2D,
    RectangleShape2D,
};
use godot::global::MouseButton;

use super::combat::GdHitbox;

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdMelee {
    /// 是否启用近战攻击
    #[export]
    enabled: bool,

    /// 框架自动处理攻击输入：unhandled 阶段路由（点击 UI 不攻击，点击世界攻击）
    #[export]
    auto_attack_mouse: bool,

    /// 攻击按键：鼠标左键
    #[export]
    attack_button_left: bool,

    /// 攻击按键：鼠标右键
    #[export]
    attack_button_right: bool,

    /// 单次挥击伤害（写入内嵌 GdHitbox.damage）
    #[export]
    attack_damage: f64,

    /// 攻击冷却（秒，两次挥击间隔）
    #[export]
    attack_cooldown: f64,

    /// 挥击判定盒长度（像素，沿攻击方向）
    #[export]
    attack_range: f64,

    /// 挥击判定盒宽度（像素，垂直攻击方向）
    #[export]
    attack_width: f64,

    /// 判定盒中心距宿主偏移（像素；默认 range/2 即贴着身体前方）
    #[export]
    attack_offset: f64,

    /// 挥击窗口（秒）：判定盒使能时长
    #[export]
    swing_window: f64,

    /// 目标受击盒碰撞掩码（默认 4 = 敌受击盒 layer3）
    #[export]
    target_mask: i64,

    /// 窗口期命中冷却（秒，写入 GdHitbox.cooldown；影响一次挥击最多命中几个目标）
    #[export]
    hit_cooldown: f64,

    // ---- 运行时状态 ----
    cooldown_timer: f64,
    /// > 0 = 挥击窗口中（判定盒使能），递减到 0 复位
    swing_timer: f64,
    /// unhandled 阶段收到攻击键按下 → 按住连击；抬起/校准复位
    attack_held: bool,
    /// ready 时创建的内嵌挥击盒
    hitbox: Option<Gd<GdHitbox>>,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdMelee {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            enabled: true,
            auto_attack_mouse: true,
            attack_button_left: true,
            attack_button_right: false,
            attack_damage: 15.0,
            attack_cooldown: 0.5,
            attack_range: 56.0,
            attack_width: 48.0,
            attack_offset: 0.0,
            swing_window: 0.12,
            target_mask: 4,
            hit_cooldown: 0.1,
            cooldown_timer: 0.0,
            swing_timer: 0.0,
            attack_held: false,
            hitbox: None,
            base,
        }
    }

    fn ready(&mut self) {
        // 内嵌挥击盒：复用 GdHitbox（伤害扫描/命中冷却），纯攻击方配置
        let mut hb = GdHitbox::new_alloc();
        {
            let mut h = hb.bind_mut();
            h.set_damage(self.attack_damage);
            h.set_cooldown(self.hit_cooldown);
            h.set_enabled(false);
        }
        hb.set_collision_layer(0);
        hb.set_collision_mask(self.target_mask as u32);
        hb.set_monitorable(false);
        // 判定形状：矩形（长边 = 攻击方向）
        let mut rect = RectangleShape2D::new_gd();
        rect.set_size(Vector2::new(self.attack_range as f32, self.attack_width as f32));
        let mut shape = CollisionShape2D::new_alloc();
        shape.set_shape(&rect);
        hb.add_child(&shape.upcast::<Node>());
        // 挥击盒随方向移动（宿主子节点，local 变换即可）
        let mut hb_node: Gd<Node> = hb.clone().upcast::<Node>();
        hb_node.set_name("SwingHitbox");
        self.base_mut().add_child(&hb_node);
        self.hitbox = Some(hb);
        // attack_offset 缺省 = range/2（判定盒贴着身体前方），负值按 0 处理
        if self.attack_offset <= 0.0 {
            self.attack_offset = self.attack_range / 2.0;
        }
    }

    fn process(&mut self, delta: f64) {
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer = (self.cooldown_timer - delta).max(0.0);
        }
        // 挥击窗口结束 → 复位判定盒
        if self.swing_timer > 0.0 {
            self.swing_timer = (self.swing_timer - delta).max(0.0);
            if self.swing_timer <= 0.0 {
                self.disable_hitbox();
            }
        }
        if !self.enabled || !self.auto_attack_mouse || !self.attack_held {
            return;
        }
        // 防卡死校准：release 被控件吞掉（按住拖进 UI 再松开）时，Input 状态
        // 已复位但 attack_held 收不到 release——以 Input 实际状态为准复位
        let input = Input::singleton();
        let pressed = (self.attack_button_left && input.is_mouse_button_pressed(MouseButton::LEFT))
            || (self.attack_button_right && input.is_mouse_button_pressed(MouseButton::RIGHT));
        if !pressed {
            self.attack_held = false;
            return;
        }
        if self.host_paused() {
            return;
        }
        self.swing_toward_mouse();
    }

    /// 攻击输入路由：unhandled 阶段 = 点击落在了世界上。
    /// 被 mouse_filter=STOP 的 UI 控件消费的点击到不了这里（引擎保证），
    /// 因此「点击 UI 不攻击、点击地图/世界攻击」无需任何 hover 猜测。
    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if !self.enabled || !self.auto_attack_mouse {
            return;
        }
        // 对话/剧情锁：宿主实现 is_paused()（如 GdRoleMover）时暂停期不接攻击输入
        if self.host_paused() {
            return;
        }
        let Ok(btn) = event.try_cast::<InputEventMouseButton>() else {
            return;
        };
        let idx = btn.get_button_index();
        let is_attack_button = (idx == MouseButton::LEFT && self.attack_button_left)
            || (idx == MouseButton::RIGHT && self.attack_button_right);
        if !is_attack_button {
            return;
        }
        if btn.is_pressed() {
            self.attack_held = true;
            if self.swing_toward_mouse() {
                // 世界点击已被「攻击」消费，不再下传（防止同时触发寻路等）
                if let Some(mut vp) = self.base().get_viewport() {
                    vp.set_input_as_handled();
                }
            }
        } else {
            self.attack_held = false;
        }
    }
}

#[godot_api]
impl GdMelee {
    /// 每次成功挥击时触发（参数：攻击方向）
    #[signal]
    fn s_swing(direction: Vector2);

    /// 剩余冷却时间（秒）
    #[func]
    pub fn get_cooldown_left(&self) -> f64 {
        self.cooldown_timer
    }

    /// 当前是否在挥击窗口中（判定盒使能）
    #[func]
    pub fn is_swinging(&self) -> bool {
        self.swing_timer > 0.0
    }

    /// 朝给定方向挥击一次，冷却中返回 false
    #[func]
    pub fn swing(&mut self, direction: Vector2) -> bool {
        if !self.enabled || self.cooldown_timer > 0.0 {
            return false;
        }
        if direction.length_squared() < 0.0001 {
            return false;
        }
        let dir = direction.normalized();
        let Some(mut hb) = self.hitbox.clone() else {
            godot_error!("[GdMelee] SwingHitbox 未初始化（宿主未 ready？）");
            return false;
        };
        // 判定盒定位：宿主前方 dir * offset，随方向旋转（矩形长边 = 攻击方向）
        hb.set_position(dir * self.attack_offset as f32);
        hb.set_rotation(dir.angle());
        // 窗口参数变化（damage/hit_cooldown 运行时调整）同步进 GdHitbox
        hb.bind_mut().set_damage(self.attack_damage);
        hb.bind_mut().set_cooldown(self.hit_cooldown);
        hb.bind_mut().set_enabled(true);
        self.swing_timer = self.swing_window;
        self.cooldown_timer = self.attack_cooldown;
        self.base_mut()
            .emit_signal("s_swing", &[dir.to_variant()]);
        true
    }

    /// 朝世界坐标点挥击
    #[func]
    pub fn swing_at_point(&mut self, target: Vector2) -> bool {
        let Some(origin) = self.host_position() else {
            return false;
        };
        self.swing(target - origin)
    }

    /// 朝鼠标当前位置挥击
    #[func]
    pub fn swing_toward_mouse(&mut self) -> bool {
        let Some(dir) = self.aim_direction() else {
            return false;
        };
        self.swing(dir)
    }

    /// 立即终止当前挥击（装备切换/宿主暂停时调用）
    #[func]
    pub fn stop(&mut self) {
        self.attack_held = false;
        self.swing_timer = 0.0;
        self.disable_hitbox();
    }

    // ---- 内部实现 ----

    /// 复位判定盒（窗口结束/stop）
    fn disable_hitbox(&mut self) {
        if let Some(mut hb) = self.hitbox.clone() {
            hb.bind_mut().set_enabled(false);
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
}
