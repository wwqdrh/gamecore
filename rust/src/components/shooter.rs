// GdShooter - 射击组件
// 挂在角色（Node2D 宿主，如 GdRoleMover）的子节点上，负责生成子弹：
//   - fire(direction)          朝给定方向发射
//   - fire_at_point(target)    朝世界坐标点发射
//   - fire_toward_mouse()      朝鼠标位置发射
//   - auto_fire_mouse 开启时，按住鼠标左键持续朝鼠标方向射击
// 子弹经 GdSpawnPool 生成/回收（bullet_scene_path 首次射击时自动注册进池），
// 每发子弹的 速度/伤害/寿命/池别名 由本组件写入 GdBullet。
//
// 信号：
//   s_fired(muzzle, direction)  每次成功发射

use godot::prelude::*;
use godot::builtin::{GString, StringName, Variant, Vector2};
use godot::classes::{Engine, INode, Input, Node, Node2D, Window};
use godot::global::MouseButton;

use super::bullet::GdBullet;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdShooter {
    /// 是否启用射击
    #[export]
    enabled: bool,

    /// 按住鼠标左键持续朝鼠标方向射击
    #[export]
    auto_fire_mouse: bool,

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

    /// 子弹池别名
    #[export]
    bullet_alias: GString,

    /// 子弹场景路径（首次射击时自动注册进 GdSpawnPool）
    #[export]
    bullet_scene_path: GString,

    // ---- 运行时状态 ----
    cooldown_timer: f64,
    pool_ready: bool,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdShooter {
    fn init(base: Base<Node>) -> Self {
        Self {
            enabled: true,
            auto_fire_mouse: true,
            fire_cooldown: 0.25,
            bullet_speed: 480.0,
            bullet_lifetime: 1.5,
            bullet_damage: 20.0,
            muzzle_distance: 24.0,
            bullet_alias: GString::from("bullet"),
            bullet_scene_path: GString::new(),
            cooldown_timer: 0.0,
            pool_ready: false,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer = (self.cooldown_timer - delta).max(0.0);
        }
        if !self.enabled || !self.auto_fire_mouse {
            return;
        }
        // 鼠标悬停在 UI 控件（按钮等）上时不自动开火
        if let Some(viewport) = self.base().get_viewport() {
            if viewport.gui_get_hovered_control().is_some() {
                return;
            }
        }
        if Input::singleton().is_mouse_button_pressed(MouseButton::LEFT) {
            self.fire_toward_mouse();
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
            b.bind_mut().setup(dir * speed as f32, damage, lifetime, alias);
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

    /// 宿主（父节点）全局位置
    fn host_position(&self) -> Option<Vector2> {
        let host = self.base().get_parent()?;
        let host2d = host.try_cast::<Node2D>().ok()?;
        Some(host2d.get_global_position())
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
