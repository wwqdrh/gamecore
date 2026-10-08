// GdBullet - 池化子弹组件
// 挂在子弹场景根节点（Area2D）上，配合 GdSpawnPool + GdShooter 使用：
//   - 每帧沿 velocity 直线飞行
//   - 主动扫描重叠的 GdHurtbox 造成伤害，命中后回收
//   - 射程 max_distance 超限回收（0 = 不限，由 GdShooter 每发写入）
//   - 地形高度检测：沿运动线段采样 terrain_provider 的高度场，
//     地形海拔 > fly_height 视为撞山，回收（check_terrain 关闭可跳过）
//   - lifetime 归零自动回收（经对象池归还，不 free；无池别名时 queue_free）
//
// 场景约定：根节点 type="GdBullet"，子节点挂 CollisionShape2D，
// collision_mask 指向目标方 GdHurtbox 所在 layer，collision_layer 保持 0。
//
// 地形协议（duck-type，零耦合）：场景树内 "terrain_provider" 分组的节点
// 实现 get_height_at_world(world: Vector2) -> i32 即可被查询（GdQuickMap
// 内置支持；GDScript 自定义地图同样可入组）。子弹不认识地图、地图不认识
// 子弹；换图后旧 provider 失效自动重查当前地图。
//
// 信号：
//   s_hit(target)            命中一个受击盒（参数：受击节点名）
//   s_destroyed(reason)      非命中销毁：range（超射程）/ terrain（撞山）/
//                            lifetime（寿命到期）——特效/统计挂这里

use godot::prelude::*;
use godot::builtin::{GString, StringName, Vector2};
use godot::classes::{Area2D, Engine, IArea2D};

use super::combat::GdHurtbox;

/// 地形采样步长（像素）：小于最常见格子尺寸的一半，高速子弹不会穿透一格
const TERRAIN_SAMPLE_STEP: f32 = 8.0;
/// 单帧最大采样数（防御异常高速）
const TERRAIN_MAX_SAMPLES: i32 = 64;

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
    /// 最大射程（像素，飞行距离）；0 = 不限（由 GdShooter 每发写入）
    #[export]
    max_distance: f32,
    /// 飞行海拔（格）：地形海拔高于该值时撞毁；调高可飞越山地
    /// （子弹场景级配置，同类武器共用）
    #[export]
    fly_height: i32,
    /// 是否检测地形高度（无 terrain_provider 时自动跳过）
    #[export]
    check_terrain: bool,

    // ---- 运行时状态 ----
    /// 已飞行距离（像素）
    traveled: f32,
    /// 高度场提供者缓存（换图失效自动重查）
    terrain_provider: Option<Gd<Node>>,

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
            max_distance: 0.0,
            fly_height: 0,
            check_terrain: true,
            traveled: 0.0,
            terrain_provider: None,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // 直线飞行（记录本帧线段，供射程与地形采样）
        let from = self.base().get_position();
        let step = self.velocity * delta as f32;
        let to = from + step;
        self.base_mut().set_position(to);

        // 射程超限回收
        self.traveled += step.length();
        if self.max_distance > 0.0 && self.traveled >= self.max_distance {
            self.destroy("range");
            return;
        }

        // 地形高度采样：沿本帧线段按固定步长取样，防高速穿透一格
        if self.check_terrain {
            if let Some(mut provider) = self.resolve_provider() {
                let dist = step.length();
                if dist > 0.0 {
                    let n = ((dist / TERRAIN_SAMPLE_STEP).ceil() as i32)
                        .clamp(1, TERRAIN_MAX_SAMPLES);
                    for i in 1..=n {
                        let p = from.lerp(to, i as f32 / n as f32);
                        let h = provider
                            .call(
                                &StringName::from("get_height_at_world"),
                                &[p.to_variant()],
                            )
                            .try_to::<i64>()
                            .unwrap_or(0);
                        if h as i32 > self.fly_height {
                            self.destroy("terrain");
                            return;
                        }
                    }
                }
            }
        }

        // 生命到期回收
        self.lifetime -= delta;
        if self.lifetime <= 0.0 {
            self.destroy("lifetime");
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

    /// 非命中销毁时触发（参数：range / terrain / lifetime）
    #[signal]
    fn s_destroyed(reason: GString);

    /// 一次性配置子弹（由 GdShooter 在每次发射时调用）
    #[func]
    pub fn setup(
        &mut self,
        velocity: Vector2,
        damage: f64,
        lifetime: f64,
        pool_alias: GString,
        max_distance: f32,
    ) {
        self.velocity = velocity;
        self.damage = damage;
        self.lifetime = lifetime;
        self.pool_alias = pool_alias;
        self.max_distance = max_distance;
        // 池化复用：运行时状态必须重置
        self.traveled = 0.0;
        self.terrain_provider = None;
    }

    // ---- 内部实现 ----

    /// 解析高度场提供者（terrain_provider 分组首个节点），缓存 + 失效重查。
    /// 无提供者（无地图/headless）返回 None，调用方跳过地形检测。
    fn resolve_provider(&mut self) -> Option<Gd<Node>> {
        if let Some(ref p) = self.terrain_provider {
            if p.is_instance_valid() {
                return Some(p.clone());
            }
            self.terrain_provider = None;
        }
        let Some(tree) = self.base().get_tree_or_null() else {
            return None;
        };
        let found = tree.get_first_node_in_group("terrain_provider")?;
        self.terrain_provider = Some(found.clone());
        Some(found)
    }

    /// 统一销毁出口：先发 s_destroyed(reason) 再回收（池化 / queue_free）
    fn destroy(&mut self, reason: &str) {
        self.base_mut()
            .emit_signal("s_destroyed", &[GString::from(reason).to_variant()]);
        self.recycle();
    }

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
