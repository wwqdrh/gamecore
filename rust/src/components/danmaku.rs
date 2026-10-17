// GdDanmaku - 弹幕发射器（俯视角即时战斗 · 弹幕射击编排组件）
// 挂在角色（Node2D 宿主，如 GdRoleMover）的子节点上，一次 cast 按「阵型
// 描述」编排一组子弹（复用 GdBullet + GdSpawnPool，与 GdShooter 同源）。
//
// 阵型（shape）：
//   single  单发直射（1 发，朝 aim）
//   fan     扇形（count 发，spread 张角，以 aim 为中心对称展开）
//   ring    环形（count 发，360° 均布）
//   burst   连发（count 发，间隔 interval，同方向）
//   spiral  螺旋（持续 duration，每 step 秒发 arms 臂，每轮偏转 rotate 度）
//
// 描述字典（opts，全部可选）：
//   count/spread/interval/duration/step/arms/rotate  阵型参数
//   cooldown        本次施放的冷却覆盖（秒；缺省用组件 cast_cooldown）
//   bullet          每发子弹覆盖 {speed, damage, lifetime, max_distance,
//                   wave_amplitude, wave_frequency, angular_speed,
//                   homing_group, homing_turn_rate, pierce}
//   —— 新弹幕 = 新描述数据，不改本组件（可扩展点）
//
// 输入约定：与 GdShooter 一致，本组件不接输入；demo 侧由玩家脚本
// watch 技能状态键 / 键盘按键后调 cast_toward_mouse / cast。
//
// 信号：
//   s_cast(shape, count)      施放成功（阵型名、本阵型子弹总数）
//   s_volley_finished         编排队列清空（全部弹发完，特效/音效挂点）

use godot::prelude::*;
use godot::builtin::{Dictionary, GString, PackedStringArray, StringName, Variant, Vector2};
use godot::classes::{Engine, INode, Node, Node2D, Window};

use super::bullet::GdBullet;

/// 弹幕描述字典（GDScript 侧传入的扁平结构）
type VarDict = Dictionary<Variant, Variant>;

/// 每发子弹的运动/数值覆盖（None = 用组件默认值）
#[derive(Default, Clone)]
struct BulletOpts {
    speed: Option<f64>,
    damage: Option<f64>,
    lifetime: Option<f64>,
    max_distance: Option<f32>,
    wave_amplitude: f32,
    wave_frequency: f32,
    angular_speed: f32,
    homing_group: String,
    homing_turn_rate: f32,
    pierce: i32,
}

/// 一次待发射：延迟秒数 + 绝对角度列表 + 子弹覆盖
struct PendingEmission {
    delay: f64,
    angles: Vec<f32>,
    opts: BulletOpts,
}

/// 把 opts 字典解析为 BulletOpts（demo 侧传描述字典，缺省全兜底）
fn parse_bullet_opts(dict: &VarDict) -> BulletOpts {
    let get_f = |k: &str| dict.get(&Variant::from(k)).and_then(|v| v.try_to::<f64>().ok());
    BulletOpts {
        speed: get_f("speed"),
        damage: get_f("damage"),
        lifetime: get_f("lifetime"),
        max_distance: get_f("max_distance").map(|v| v as f32),
        wave_amplitude: get_f("wave_amplitude").unwrap_or(0.0) as f32,
        wave_frequency: get_f("wave_frequency").unwrap_or(0.0) as f32,
        angular_speed: get_f("angular_speed").unwrap_or(0.0) as f32,
        homing_group: dict
            .get("homing_group")
            .and_then(|v| v.try_to::<GString>().ok())
            .map(|s| s.to_string())
            .unwrap_or_default(),
        homing_turn_rate: get_f("homing_turn_rate").unwrap_or(0.0) as f32,
        pierce: dict
            .get("pierce")
            .and_then(|v| v.try_to::<i64>().ok())
            .unwrap_or(0) as i32,
    }
}

/// 扇形角度表（相对角度，弧度；以 0 为中心对称展开，count=1 时仅 0）
/// 纯函数，单元测试覆盖
fn fan_angles(count: i32, spread_deg: f32) -> Vec<f32> {
    if count <= 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![0.0];
    }
    let spread = spread_deg.to_radians();
    let step = spread / (count - 1) as f32;
    let start = -spread / 2.0;
    (0..count).map(|i| start + step * i as f32).collect()
}

/// 环形角度表（相对角度，弧度；360° 均布，首角 0）
/// 纯函数，单元测试覆盖
fn ring_angles(count: i32) -> Vec<f32> {
    if count <= 0 {
        return Vec::new();
    }
    let step = std::f32::consts::TAU / count as f32;
    (0..count).map(|i| step * i as f32).collect()
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdDanmaku {
    /// 是否启用施放
    #[export]
    enabled: bool,
    /// 施放冷却（秒；opts.cooldown 可按次覆盖）
    #[export]
    cast_cooldown: f64,
    /// 子弹初速（像素/秒；opts.bullet.speed 可覆盖）
    #[export]
    bullet_speed: f64,
    /// 单发伤害（opts.bullet.damage 可覆盖）
    #[export]
    bullet_damage: f64,
    /// 子弹存活时长（秒）
    #[export]
    bullet_lifetime: f64,
    /// 子弹最大射程（像素，飞行距离超限销毁）；0 = 不限
    #[export]
    max_distance: f64,
    /// 枪口距宿主中心的偏移距离（像素）
    #[export]
    muzzle_distance: f64,
    /// 子弹池别名（弹幕建议独立别名，避免与 GdShooter 池互相重注册冲刷在飞子弹）
    #[export]
    bullet_alias: GString,
    /// 子弹场景路径（首次施放时自动注册进 GdSpawnPool）
    #[export]
    bullet_scene_path: GString,

    // ---- 运行时状态 ----
    cooldown_timer: f64,
    pool_ready: bool,
    /// 待发射队列（burst/spiral 的分时弹）
    pending: Vec<PendingEmission>,
    /// 队列计时（秒）
    volley_acc: f64,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdDanmaku {
    fn init(base: Base<Node>) -> Self {
        Self {
            enabled: true,
            cast_cooldown: 0.5,
            bullet_speed: 480.0,
            bullet_damage: 10.0,
            bullet_lifetime: 1.2,
            max_distance: 0.0,
            muzzle_distance: 24.0,
            bullet_alias: GString::from("danmaku"),
            bullet_scene_path: GString::new(),
            cooldown_timer: 0.0,
            pool_ready: false,
            pending: Vec::new(),
            volley_acc: 0.0,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer = (self.cooldown_timer - delta).max(0.0);
        }
        if self.pending.is_empty() {
            return;
        }
        // 宿主暂停（对话/剧情锁）时冻结编排队列计时
        if self.host_paused() {
            return;
        }
        self.volley_acc += delta;
        // 入口已保证 pending 非空；全部到期弹发完后队列清空 → 发完成信号
        while let Some(idx) = self.pending.iter().position(|e| e.delay <= self.volley_acc) {
            let mut emission = self.pending.swap_remove(idx);
            for angle in emission.angles.drain(..) {
                self.spawn_one(angle, &emission.opts);
            }
        }
        if self.pending.is_empty() {
            self.volley_acc = 0.0;
            self.base_mut().emit_signal("s_volley_finished", &[]);
        }
    }
}

#[godot_api]
impl GdDanmaku {
    /// 施放成功（参数：阵型名、本阵型子弹总数）
    #[signal]
    fn s_cast(shape: GString, count: i64);

    /// 编排队列清空（全部弹发完）
    #[signal]
    fn s_volley_finished();

    /// 剩余冷却时间（秒）
    #[func]
    pub fn get_cooldown_left(&self) -> f64 {
        self.cooldown_timer
    }

    /// 待发射队列长度（调试/测试观测用）
    #[func]
    pub fn get_pending_count(&self) -> i64 {
        self.pending.len() as i64
    }

    /// 按阵型施放弹幕。aim = 飞行基准方向（任意长度，内部归一化）；
    /// opts 描述字典见文件头。冷却中返回 false（队列不受影响继续发完）。
    #[func]
    pub fn cast(&mut self, shape: GString, aim: Vector2, opts: VarDict) -> bool {
        if !self.enabled {
            return false;
        }
        if self.cooldown_timer > 0.0 {
            return false;
        }
        if aim.length_squared() < 0.0001 {
            return false;
        }
        let base_angle = aim.normalized().angle();
        let get_i = |k: &str, d: i64| {
            opts.get(k)
                .and_then(|v| v.try_to::<i64>().ok())
                .unwrap_or(d)
        };
        let get_f = |k: &str, d: f64| {
            opts.get(k)
                .and_then(|v| v.try_to::<f64>().ok())
                .unwrap_or(d)
        };
        let bullet = opts
            .get("bullet")
            .and_then(|v| v.try_to::<VarDict>().ok())
            .unwrap_or_default();
        let bopts = parse_bullet_opts(&bullet);

        // 阵型 → 分时发射队列（角度已换算为绝对角）
        let shape_name = shape.to_string();
        let mut emissions: Vec<PendingEmission> = Vec::new();
        let mut total: i64 = 0;
        match shape_name.as_str() {
            "single" => {
                emissions.push(PendingEmission {
                    delay: 0.0,
                    angles: vec![base_angle],
                    opts: bopts,
                });
                total = 1;
            }
            "fan" => {
                let count = get_i("count", 5) as i32;
                let spread = get_f("spread", 60.0) as f32;
                let angles: Vec<f32> = fan_angles(count, spread)
                    .iter()
                    .map(|a| base_angle + a)
                    .collect();
                total = angles.len() as i64;
                emissions.push(PendingEmission { delay: 0.0, angles, opts: bopts });
            }
            "ring" => {
                let count = get_i("count", 12) as i32;
                let angles: Vec<f32> = ring_angles(count)
                    .iter()
                    .map(|a| base_angle + a)
                    .collect();
                total = angles.len() as i64;
                emissions.push(PendingEmission { delay: 0.0, angles, opts: bopts });
            }
            "burst" => {
                let count = get_i("count", 3) as i32;
                let interval = get_f("interval", 0.1);
                for i in 0..count {
                    emissions.push(PendingEmission {
                        delay: interval * i as f64,
                        angles: vec![base_angle],
                        opts: bopts.clone(),
                    });
                }
                total = count as i64;
            }
            "spiral" => {
                let duration = get_f("duration", 1.0);
                let step = get_f("step", 0.08).max(0.01);
                let arms = get_i("arms", 3) as i32;
                let rotate = get_f("rotate", 24.0) as f32;
                let mut k = 0i64;
                let mut t = 0.0f64;
                while t < duration {
                    let base = base_angle + rotate.to_radians() * k as f32;
                    let ring: Vec<f32> = ring_angles(arms)
                        .iter()
                        .map(|a| base + a)
                        .collect();
                    total += ring.len() as i64;
                    emissions.push(PendingEmission {
                        delay: t,
                        angles: ring,
                        opts: bopts.clone(),
                    });
                    k += 1;
                    t += step;
                }
            }
            other => {
                godot_warn!("[GdDanmaku] 未知阵型 '{}'", other);
                return false;
            }
        }

        self.cooldown_timer = opts
            .get("cooldown")
            .and_then(|v| v.try_to::<f64>().ok())
            .unwrap_or(self.cast_cooldown);
        // 连 cast 时序基准：旧队列未发完时新弹延迟顺延（delay 相对统一计时轴），
        // 否则 volley_acc 归零会让旧弹立即倾泻
        let base_t = if self.pending.is_empty() {
            self.volley_acc = 0.0;
            0.0
        } else {
            self.volley_acc
        };
        self.pending.extend(emissions.into_iter().map(|mut e| {
            e.delay += base_t;
            e
        }));
        self.base_mut().emit_signal(
            "s_cast",
            &[GString::from(shape_name.as_str()).to_variant(), total.to_variant()],
        );
        true
    }

    /// 朝鼠标当前位置施放（宿主 → 鼠标方向）
    #[func]
    pub fn cast_toward_mouse(&mut self, shape: GString, opts: VarDict) -> bool {
        let Some(aim) = self.aim_direction() else {
            return false;
        };
        self.cast(shape, aim, opts)
    }

    /// 描述字典一步施放（数据驱动入口）：
    /// {shape, aim?(Vector2 缺省朝鼠标), cooldown?, count?..., bullet?{}}
    #[func]
    pub fn cast_desc(&mut self, desc: VarDict) -> bool {
        let Some(shape) = desc.get("shape").and_then(|v| v.try_to::<GString>().ok()) else {
            godot_warn!("[GdDanmaku] cast_desc 缺少 shape");
            return false;
        };
        // 把 desc 除 shape/aim 外整体作为 opts 传递（扁平结构，双层 bullet 透传）
        let mut opts = VarDict::new();
        for (k, v) in desc.iter_shared() {
            let key = k.to_string();
            if key == "shape" || key == "aim" {
                continue;
            }
            opts.set(&k, &v);
        }
        match desc.get("aim").and_then(|v| v.try_to::<Vector2>().ok()) {
            Some(aim) => self.cast(shape, aim, opts),
            None => self.cast_toward_mouse(shape, opts),
        }
    }

    // ---- 内部实现 ----

    /// 宿主（父节点）全局位置
    fn host_position(&self) -> Option<Vector2> {
        let host = self.base().get_parent()?;
        let host2d = host.try_cast::<Node2D>().ok()?;
        Some(host2d.get_global_position())
    }

    /// 瞄准方向：宿主 → 鼠标
    fn aim_direction(&self) -> Option<Vector2> {
        let host = self.base().get_parent()?;
        let host2d = host.try_cast::<Node2D>().ok()?;
        let diff = host2d.get_global_mouse_position() - host2d.get_global_position();
        if diff.length_squared() < 0.0001 {
            return None;
        }
        Some(diff.normalized())
    }

    /// 宿主暂停判定（duck-type is_paused，同 GdShooter）
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

    /// 发射一发：角度为绝对角；数值缺省回落组件导出值
    fn spawn_one(&mut self, angle: f32, o: &BulletOpts) {
        let Some(origin) = self.host_position() else {
            return;
        };
        let dir = Vector2::from_angle(angle);
        let muzzle = origin + dir * self.muzzle_distance as f32;
        let Some(bullet) = self.spawn_bullet(muzzle) else {
            return;
        };
        if let Ok(mut n2d) = bullet.clone().try_cast::<Node2D>() {
            n2d.set_rotation(angle);
        }
        if let Ok(mut b) = bullet.try_cast::<GdBullet>() {
            let speed = o.speed.unwrap_or(self.bullet_speed);
            let damage = o.damage.unwrap_or(self.bullet_damage);
            let lifetime = o.lifetime.unwrap_or(self.bullet_lifetime);
            let max_distance = o.max_distance.unwrap_or(self.max_distance as f32);
            let alias = self.bullet_alias.clone();
            b.bind_mut()
                .setup(dir * speed as f32, damage, lifetime, alias, max_distance);
            b.bind_mut().setup_motion(
                o.angular_speed,
                o.wave_amplitude,
                o.wave_frequency,
                GString::from(o.homing_group.as_str()),
                o.homing_turn_rate,
                o.pierce,
            );
        }
    }

    /// 经对象池生成子弹（首次自动注册池），失败返回 None（同 GdShooter 范式）
    fn spawn_bullet(&mut self, pos: Vector2) -> Option<Gd<Node>> {
        let engine = Engine::singleton();
        let Some(mut pool) = engine.get_singleton("GDSPAWNPOOL") else {
            godot_error!("[GdDanmaku] GDSPAWNPOOL 单例不存在");
            return None;
        };
        if !self.pool_ready {
            let alias = self.bullet_alias.clone();
            let path = self.bullet_scene_path.clone();
            if alias.is_empty() || path.is_empty() {
                godot_error!("[GdDanmaku] bullet_alias / bullet_scene_path 未配置");
                return None;
            }
            let ok = pool.call(
                &StringName::from("register"),
                &[alias.to_variant(), path.to_variant(), 24i64.to_variant()],
            );
            if !ok.try_to::<bool>().unwrap_or(false) {
                godot_error!("[GdDanmaku] 池注册失败 alias={}", alias);
                return None;
            }
            self.pool_ready = true;
        } else {
            // 防御：外部清池（测试/热更）后本组件毫不知情，spawn 必失败——
            // 显式告警并自愈重注册（幂等，replace prewarm）
            let registered: bool = pool
                .call(&StringName::from("aliases"), &[])
                .try_to::<PackedStringArray>()
                .map(|names| names.to_vec().iter().any(|n| *n == self.bullet_alias))
                .unwrap_or(true);
            if !registered {
                godot_warn!("[GdDanmaku] 池别名 '{}' 已被外部清除，重注册", self.bullet_alias);
                let alias = self.bullet_alias.clone();
                let path = self.bullet_scene_path.clone();
                let ok = pool.call(
                    &StringName::from("register"),
                    &[alias.to_variant(), path.to_variant(), 24i64.to_variant()],
                );
                if !ok.try_to::<bool>().unwrap_or(false) {
                    return None;
                }
            }
        }
        // 父节点：优先当前场景，无当前场景（headless 测试等）时兜底挂到根
        let parent = self.spawn_parent();
        if parent.is_none() {
            godot_warn!("[GdDanmaku] spawn_parent 解析失败（宿主不在树中？）");
            return None;
        }
        let ret = pool.call(
            &StringName::from("spawn"),
            &[
                self.bullet_alias.to_variant(),
                parent.map(|p| p.to_variant()).unwrap_or(Variant::nil()),
                pos.to_variant(),
            ],
        );
        let out = ret.try_to::<Gd<Node>>().ok();
        if out.is_none() {
            godot_warn!("[GdDanmaku] pool.spawn 未返回节点（alias={}）", self.bullet_alias);
        }
        out
    }

    /// 子弹挂载父节点：当前场景 > 场景树根
    fn spawn_parent(&self) -> Option<Gd<Node>> {
        let Some(tree) = self.base().get_tree_or_null() else {
            godot_warn!("[GdDanmaku] 宿主不在场景树中，无法挂载子弹");
            return None;
        };
        if let Some(cs) = tree.get_current_scene() {
            return Some(cs);
        }
        let root: Gd<Window> = tree.get_root()?;
        Some(root.upcast::<Node>())
    }
}

/// 阵型角度纯函数单元测试（发射编排的数学核心）
#[cfg(test)]
mod tests {
    use super::*;

    const PI: f32 = std::f32::consts::PI;

    #[test]
    fn fan_single_and_empty() {
        assert!(fan_angles(0, 60.0).is_empty());
        assert_eq!(fan_angles(1, 60.0), vec![0.0]);
    }

    #[test]
    fn fan_symmetric_and_bounds() {
        let angles = fan_angles(7, 70.0);
        assert_eq!(angles.len(), 7);
        assert!((angles[0] + angles[6]).abs() < 1e-6, "扇形应以中心对称（首尾和为 0）");
        assert!((angles[3]).abs() < 1e-6, "中位角应为 0（瞄准方向）");
        let half = 35.0f32.to_radians();
        assert!((angles[0] + half).abs() < 1e-5, "首角 = -spread/2");
        assert!((angles[6] - half).abs() < 1e-5, "尾角 = +spread/2");
    }

    #[test]
    fn fan_two_is_half_spread() {
        let angles = fan_angles(2, 90.0);
        assert_eq!(angles.len(), 2);
        assert!((angles[0] + PI / 4.0).abs() < 1e-5);
        assert!((angles[1] - PI / 4.0).abs() < 1e-5);
    }

    #[test]
    fn ring_even_spacing_and_wrap() {
        let angles = ring_angles(12);
        assert_eq!(angles.len(), 12);
        let step = std::f32::consts::TAU / 12.0;
        for (i, a) in angles.iter().enumerate() {
            assert!((*a - step * i as f32).abs() < 1e-6, "均布间隔");
        }
        assert!(angles[0].abs() < 1e-6, "首角 0");
        assert!((angles.last().unwrap() - (std::f32::consts::TAU - step)).abs() < 1e-6);
    }

    #[test]
    fn ring_single_and_empty() {
        assert!(ring_angles(0).is_empty());
        assert_eq!(ring_angles(1), vec![0.0]);
    }
}
