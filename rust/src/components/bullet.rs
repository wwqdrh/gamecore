// GdBullet - 池化子弹组件
// 挂在子弹场景根节点（Area2D）上，配合 GdSpawnPool + GdShooter/GdDanmaku 使用：
//   - 每帧沿 velocity 飞行（默认直线；可选运动行为见下）
//   - 主动扫描重叠的 GdHurtbox 造成伤害，命中后回收
//   - 射程 max_distance 超限回收（0 = 不限，由发射器每发写入）
//   - 地形高度检测：沿运动线段采样 terrain_provider 的高度场，
//     地形海拔 > fly_height 视为撞山，回收（check_terrain 关闭可跳过）
//   - lifetime 归零自动回收（经对象池归还，不 free；无池别名时 queue_free）
//
// 可选运动行为（默认全关，直线弹零影响；由 GdDanmaku 等发射器按需开启）：
//   - angular_speed   角速度（弧度/秒）：velocity 逐帧旋转 → 曲线弹/回旋弹
//   - wave_amplitude + wave_frequency   横向正弦摆动（波纹弹，垂直飞行方向）
//   - homing_group + homing_turn_rate   追踪：朝组内最近目标转向（弧度/秒上限）
//   - pierce          穿透数：命中 N 个受击盒后才回收（0 = 单发即毁，默认）
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
/// 追踪目标重查间隔（秒）：避免每帧全组遍历，目标死亡后最多延迟该时长换目标
const HOMING_RESEARCH_INTERVAL: f32 = 0.2;

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

    // ---- 可选运动行为（默认全关 = 直线弹）----
    /// 角速度（弧度/秒）：velocity 逐帧旋转，0 = 不旋转
    #[export]
    angular_speed: f32,
    /// 正弦横摆振幅（像素）：0 = 无摆动
    #[export]
    wave_amplitude: f32,
    /// 正弦横摆频率（Hz）
    #[export]
    wave_frequency: f32,
    /// 追踪目标所在分组（空 = 不追踪）
    #[export]
    homing_group: GString,
    /// 追踪转向速率上限（弧度/秒）
    #[export]
    homing_turn_rate: f32,
    /// 追踪索敌半径（像素；组内超出该距离的目标不锁定）
    #[export]
    homing_range: f32,
    /// 穿透数：命中 N 个受击盒后才回收（0 = 单发即毁）
    #[export]
    pierce: i32,

    // ---- 运行时状态 ----
    /// 已飞行距离（像素，沿基准弹道累计）
    traveled: f32,
    /// 高度场提供者缓存（换图失效自动重查）
    terrain_provider: Option<Gd<Node>>,
    /// 基准弹道位置（不含横摆偏移；实际 position = base_pos + 横移量）
    base_pos: Vector2,
    /// 横摆相位（弧度）
    wave_phase: f32,
    /// 追踪目标缓存（定期重查）
    homing_target: Option<Gd<Node>>,
    homing_research_acc: f32,
    /// 已命中数（穿透计数）
    hit_count: i32,

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
            angular_speed: 0.0,
            wave_amplitude: 0.0,
            wave_frequency: 0.0,
            homing_group: GString::new(),
            homing_turn_rate: 0.0,
            homing_range: 512.0,
            pierce: 0,
            traveled: 0.0,
            terrain_provider: None,
            base_pos: Vector2::ZERO,
            wave_phase: 0.0,
            homing_target: None,
            homing_research_acc: 0.0,
            hit_count: 0,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let dt = delta as f32;

        // 1. 转向：追踪（受转向率限制）优先，其次固定角速度
        if !self.homing_group.is_empty() && self.homing_turn_rate > 0.0 {
            if let Some(target_pos) = self.homing_target_pos(dt) {
                let pos = self.base().get_position();
                let desired = (target_pos - pos).angle();
                let cur = self.velocity.angle();
                let max_turn = self.homing_turn_rate * dt;
                let diff = wrap_angle(desired - cur);
                let turn = diff.clamp(-max_turn, max_turn);
                self.velocity = self.velocity.rotated(turn);
            }
        } else if self.angular_speed != 0.0 {
            self.velocity = self.velocity.rotated(self.angular_speed * dt);
        }

        // 2. 基准弹道位移（直线分量）
        let from = self.base().get_position();
        let step = self.velocity * delta as f32;
        self.base_pos += step;
        self.traveled += step.length();

        // 3. 正弦横摆：垂直飞行方向的偏移，叠加在基准位置上
        let new_pos = if self.wave_amplitude > 0.0 && self.wave_frequency > 0.0 {
            let perp = self.velocity.normalized().orthogonal();
            self.base_pos + perp * (self.wave_amplitude * self.wave_phase.sin())
        } else {
            self.base_pos
        };
        self.wave_phase += std::f32::consts::TAU * self.wave_frequency * dt;
        self.base_mut().set_position(new_pos);

        // 射程超限回收（沿基准弹道累计）
        if self.max_distance > 0.0 && self.traveled >= self.max_distance {
            self.destroy("range");
            return;
        }

        // 地形高度采样：沿本帧实际线段按固定步长取样，防高速穿透一格
        if self.check_terrain {
            if let Some(mut provider) = self.resolve_provider() {
                let seg = new_pos - from;
                let dist = seg.length();
                if dist > 0.0 {
                    let n = ((dist / TERRAIN_SAMPLE_STEP).ceil() as i32)
                        .clamp(1, TERRAIN_MAX_SAMPLES);
                    for i in 1..=n {
                        let p = from.lerp(new_pos, i as f32 / n as f32);
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

        // 主动扫描重叠区域：命中受击盒造成伤害；穿透数未满继续飞，满了回收
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
                self.hit_count += 1;
                if self.hit_count > self.pierce {
                    // 穿透耗尽销毁也走统一出口（与 range/terrain/lifetime 一致，
                    // 特效/统计可监听；单发即毁的直射弹同样发此原因）
                    self.destroy("pierce_exhausted");
                    return;
                }
            }
        }
    }
}

/// 角度归一化到 [-π, π]
fn wrap_angle(a: f32) -> f32 {
    const PI: f32 = std::f32::consts::PI;
    const TAU: f32 = std::f32::consts::TAU;
    (a + PI).rem_euclid(TAU) - PI
}

#[godot_api]
impl GdBullet {
    /// 命中一个受击盒时触发（参数：受击节点名）
    #[signal]
    fn s_hit(target: GString);

    /// 非命中销毁时触发（参数：range / terrain / lifetime）
    #[signal]
    fn s_destroyed(reason: GString);

    /// 一次性配置子弹（由 GdShooter/GdDanmaku 在每次发射时调用）
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
        self.base_pos = self.base().get_position();
        self.wave_phase = 0.0;
        self.homing_target = None;
        self.homing_research_acc = 0.0;
        self.hit_count = 0;
    }

    /// 可选运动行为配置（默认全关 = 直线弹；由弹幕发射器按描述字典写入）。
    /// 参数顺序：角速度 / 摆幅 / 摆频 / 追踪组 / 转向率 / 穿透数
    #[func]
    pub fn setup_motion(
        &mut self,
        angular_speed: f32,
        wave_amplitude: f32,
        wave_frequency: f32,
        homing_group: GString,
        homing_turn_rate: f32,
        pierce: i32,
    ) {
        self.angular_speed = angular_speed;
        self.wave_amplitude = wave_amplitude;
        self.wave_frequency = wave_frequency;
        self.homing_group = homing_group;
        self.homing_turn_rate = homing_turn_rate;
        self.pierce = pierce;
    }

    // ---- 内部实现 ----

    /// 追踪目标位置：缓存目标定期重查（间隔 HOMING_RESEARCH_INTERVAL），
    /// 目标失效（死亡/离树）立即换；组内取索敌半径内最近者
    fn homing_target_pos(&mut self, dt: f32) -> Option<Vector2> {
        self.homing_research_acc += dt;
        let need_search = self.homing_research_acc >= HOMING_RESEARCH_INTERVAL
            || self
                .homing_target
                .as_ref()
                .is_none_or(|t| !t.is_instance_valid());
        if need_search {
            self.homing_research_acc = 0.0;
            self.homing_target = self.search_homing_target();
        }
        let target = self.homing_target.as_ref()?;
        if !target.is_instance_valid() {
            return None;
        }
        let n2d = target.clone().try_cast::<Node2D>().ok()?;
        Some(n2d.get_global_position())
    }

    /// 组内索敌：距子弹最近的 Node2D（超出 homing_range 不锁）
    fn search_homing_target(&mut self) -> Option<Gd<Node>> {
        let group = self.homing_group.to_string();
        if group.is_empty() {
            return None;
        }
        let Some(tree) = self.base().get_tree_or_null() else {
            return None;
        };
        let pos = self.base().get_global_position();
        let nodes = tree.get_nodes_in_group(group.as_str());
        let mut best: Option<(f32, Gd<Node>)> = None;
        for i in 0..nodes.len() {
            let Some(node) = nodes.get(i) else { continue; };
            let Ok(n2d) = node.clone().try_cast::<Node2D>() else {
                continue;
            };
            let d = pos.distance_to(n2d.get_global_position());
            if d > self.homing_range {
                continue;
            }
            if best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                best = Some((d, node.clone()));
            }
        }
        best.map(|(_, n)| n)
    }

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
