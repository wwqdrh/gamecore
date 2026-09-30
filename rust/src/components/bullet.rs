// GdBullet - 池化子弹组件
// 挂在子弹场景根节点（Area2D）上，配合 GdSpawnPool + GdShooter 使用：
//   - 每帧沿 velocity 直线飞行
//   - 主动扫描重叠的 GdHurtbox 造成伤害，命中后回收
//   - lifetime 归零自动回收（经对象池归还，不 free；无池别名时 queue_free）
//
// 场景约定：根节点 type="GdBullet"，子节点挂 CollisionShape2D，
// collision_mask 指向目标方 GdHurtbox 所在 layer，collision_layer 保持 0。
//
// 信号：
//   s_hit(target)  命中一个受击盒（参数：受击节点名）

use godot::prelude::*;
use godot::builtin::{GString, StringName, Vector2};
use godot::classes::{Area2D, Engine, IArea2D};

use super::combat::GdHurtbox;

#[derive(GodotClass)]
#[class(base = Area2D)]
pub struct GdBullet {
    /// 飞行速度向量（像素/秒）
    #[export]
    velocity: Vector2,
    /// 命中造成的伤害
    #[export]
    damage: f64,
    /// 存活时长（秒），归零自动回收
    #[export]
    lifetime: f64,
    /// 对象池别名（由 GdShooter 填写；空 = 生命周期结束时直接 queue_free）
    #[export]
    pool_alias: GString,

    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for GdBullet {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            velocity: Vector2::ZERO,
            damage: 10.0,
            lifetime: 1.5,
            pool_alias: GString::new(),
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // 直线飞行
        let step = self.velocity * delta as f32;
        let new_pos = self.base().get_position() + step;
        self.base_mut().set_position(new_pos);

        // 生命到期回收
        self.lifetime -= delta;
        if self.lifetime <= 0.0 {
            self.recycle();
            return;
        }

        // 主动扫描重叠区域，命中受击盒即造成伤害并回收（单发命中即毁）
        let areas = self.base_mut().get_overlapping_areas();
        for i in 0..areas.len() {
            let Some(area) = areas.get(i) else { continue; };
            let Ok(mut hurt) = area.clone().try_cast::<GdHurtbox>() else {
                continue;
            };
            if hurt.bind_mut().take_damage(self.damage) {
                let target = area.get_name();
                self.base_mut()
                    .emit_signal("s_hit", &[target.to_variant()]);
                self.recycle();
                return;
            }
        }
    }
}

#[godot_api]
impl GdBullet {
    /// 命中一个受击盒时触发（参数：受击节点名）
    #[signal]
    fn s_hit(target: GString);

    /// 一次性配置子弹（由 GdShooter 在每次发射时调用）
    #[func]
    pub fn setup(&mut self, velocity: Vector2, damage: f64, lifetime: f64, pool_alias: GString) {
        self.velocity = velocity;
        self.damage = damage;
        self.lifetime = lifetime;
        self.pool_alias = pool_alias;
    }

    // ---- 内部实现 ----

    /// 回收：有池别名走对象池 despawn，失败兜底 queue_free
    fn recycle(&mut self) {
        let alias = self.pool_alias.clone();
        if alias.is_empty() {
            self.base_mut().queue_free();
            return;
        }
        let engine = Engine::singleton();
        let Some(mut pool) = engine.get_singleton("GDSPAWNPOOL") else {
            self.base_mut().queue_free();
            return;
        };
        let instance_id = self.base().instance_id();
        let Ok(self_gd) = Gd::<Area2D>::try_from_instance_id(instance_id) else {
            self.base_mut().queue_free();
            return;
        };
        let ok = pool.call(
            &StringName::from("despawn"),
            &[self_gd.to_variant()],
        );
        if !ok.try_to::<bool>().unwrap_or(false) {
            self.base_mut().queue_free();
        }
    }
}
