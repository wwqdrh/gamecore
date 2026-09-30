// GdNpcBrain - NPC AI 行为大脑
// 挂在角色（GdRoleMover）的子节点上，根据行为配置每帧驱动移动器的 AI 移动接口：
//   - AI_IDLE    待机：原地不动
//   - AI_WANDER  随机游走：在家附近随机取点行走，走完随机休息
//   - AI_PATROL  路径巡逻：沿 patrol_points 循环或往返移动，可在每个点停留
//   - AI_FOLLOW  跟随目标：保持 follow_stop_distance 距离跟随 follow_target
//   - AI_FLEE    逃离目标：进入 flee_radius 后向远离方向移动
// 落点选择、停留时长均带随机抖动，让 NPC 表现更自然
//
// 信号：
//   s_ai_state_changed(state)  AI 状态切换 idle/walk/pause/flee

use rand::Rng;

use godot::prelude::*;
use godot::builtin::{GString, PackedVector2Array, Vector2};
use godot::classes::{INode, Node, Node2D};

use super::movement::GdRoleMover;

/// 行为：待机
pub const AI_IDLE: i64 = 0;
/// 行为：随机游走
pub const AI_WANDER: i64 = 1;
/// 行为：路径巡逻
pub const AI_PATROL: i64 = 2;
/// 行为：跟随目标
pub const AI_FOLLOW: i64 = 3;
/// 行为：逃离目标
pub const AI_FLEE: i64 = 4;
/// 行为：猎手（游走为底，目标进入视野追击，进入攻击距离后攻击）
pub const AI_HUNTER: i64 = 5;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdNpcBrain {
    /// 是否启用 AI
    #[export]
    enable: bool,

    /// 行为类型 AI_IDLE / AI_WANDER / AI_PATROL / AI_FOLLOW / AI_FLEE
    #[export]
    behavior: i64,

    /// 随机游走半径（相对初始位置，像素）
    #[export]
    wander_radius: f64,

    /// 游走/巡逻停留最短时长（秒）
    #[export]
    idle_min: f64,

    /// 游走停留最长时长（秒）
    #[export]
    idle_max: f64,

    /// 单段游走最短行走时长（秒）
    #[export]
    walk_min: f64,

    /// 单段游走最长行走时长（秒）
    #[export]
    walk_max: f64,

    /// 巡逻点列表（世界坐标）
    #[export]
    patrol_points: PackedVector2Array,

    /// true 循环巡逻，false 到端点后往返
    #[export]
    patrol_loop: bool,

    /// 到达巡逻点后的停留时长（0 = 不停留）
    #[export]
    patrol_pause: f64,

    /// 跟随/逃离的目标节点
    #[export]
    #[var(get = get_follow_target, set = set_follow_target)]
    follow_target: Option<Gd<Node2D>>,

    /// 跟随保持距离（像素）
    #[export]
    follow_stop_distance: f64,

    /// 逃离触发距离（像素）
    #[export]
    flee_radius: f64,

    /// 游走/巡逻到达判定距离（像素）
    #[export]
    arrival_distance: f64,

    /// [HUNTER] 视野半径（像素）：目标进入后从游走切换追击
    #[export]
    sight_range: f64,

    /// [HUNTER] 失去视野后继续追击的持续时间（秒），期间再次进入视野则刷新
    #[export]
    chase_memory: f64,

    /// [HUNTER] 攻击距离（像素）：追到该距离内进入攻击相位
    #[export]
    attack_range: f64,

    /// [HUNTER] 攻击冷却（秒）
    #[export]
    attack_cooldown: f64,

    // ---- 运行时状态 ----
    mover: Option<Gd<GdRoleMover>>,
    /// 初始位置（游走中心）
    home: Vector2,
    /// 当前 AI 状态 idle/walk/pause/flee/chase/attack
    phase: GString,
    /// 当前状态已持续时长
    phase_timer: f64,
    /// 当前状态时长上限（idle 随机停留 / walk 随机行走上限）
    phase_limit: f64,
    /// 游走目标点
    current_target: Vector2,
    /// 巡逻索引
    patrol_index: i64,
    /// 巡逻方向（往返模式 1/-1）
    patrol_dir: i64,
    /// [HUNTER] 追击记忆倒计时（失去视野后递减）
    chase_timer: f64,
    /// [HUNTER] 攻击冷却倒计时
    attack_timer: f64,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdNpcBrain {
    fn init(base: Base<Node>) -> Self {
        Self {
            enable: true,
            behavior: AI_WANDER,
            wander_radius: 100.0,
            idle_min: 0.5,
            idle_max: 2.0,
            walk_min: 0.8,
            walk_max: 2.5,
            patrol_points: PackedVector2Array::new(),
            patrol_loop: true,
            patrol_pause: 0.0,
            follow_target: None,
            follow_stop_distance: 40.0,
            flee_radius: 120.0,
            arrival_distance: 6.0,
            sight_range: 220.0,
            chase_memory: 3.0,
            attack_range: 48.0,
            attack_cooldown: 1.0,
            mover: None,
            home: Vector2::ZERO,
            phase: GString::from("idle"),
            phase_timer: 0.0,
            phase_limit: 1.0,
            current_target: Vector2::ZERO,
            patrol_index: 0,
            patrol_dir: 1,
            chase_timer: 0.0,
            attack_timer: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        // 解析移动器与初始位置
        if let Some(parent) = self.base().get_parent() {
            self.mover = parent.try_cast::<GdRoleMover>().ok();
            if let Some(ref mover) = self.mover {
                self.home = mover.bind().base().get_global_position();
            }
        }
        self.phase_limit = self.rand_range(self.idle_min, self.idle_max);
    }

    fn physics_process(&mut self, delta: f64) {
        if !self.enable {
            return;
        }
        let Some(mover) = self.mover.clone() else {
            return;
        };
        if !mover.is_instance_valid() {
            return;
        }
        let mut mover = mover;
        match self.behavior {
            AI_WANDER => self.tick_wander(delta, &mut mover),
            AI_PATROL => self.tick_patrol(delta, &mut mover),
            AI_FOLLOW => self.tick_follow(&mut mover),
            AI_FLEE => self.tick_flee(&mut mover),
            AI_HUNTER => self.tick_hunter(delta, &mut mover),
            _ => self.set_phase(&mut mover, "idle"),
        }
    }
}

#[godot_api]
impl GdNpcBrain {
    /// AI 状态变化信号
    #[signal]
    fn s_ai_state_changed(state: GString);

    /// [HUNTER] 攻击触发信号（进入攻击相位及冷却结束再次命中时发出）
    #[signal]
    fn s_attack();

    #[func]
    fn get_follow_target(&self) -> Option<Gd<Node2D>> {
        self.follow_target.clone()
    }

    #[func]
    fn set_follow_target(&mut self, target: Option<Gd<Node2D>>) {
        self.follow_target = target;
    }

    /// 当前 AI 状态（idle/walk/pause/flee）
    #[func]
    fn get_ai_state(&self) -> GString {
        self.phase.clone()
    }

    /// 重置 AI：以当前位置为新家，巡逻从头开始，状态回到 idle
    #[func]
    pub fn restart(&mut self) {
        self.patrol_index = 0;
        self.patrol_dir = 1;
        self.phase_timer = 0.0;
        self.current_target = Vector2::ZERO;
        if let Some(mover) = self.mover.clone() {
            if mover.is_instance_valid() {
                self.home = mover.bind().base().get_global_position();
                let mut mover = mover;
                self.set_phase(&mut mover, "idle");
            }
        }
    }

    // ---- 内部实现 ----

    /// 随机游走：随机点行走 <-> 随机停留
    fn tick_wander(&mut self, delta: f64, mover: &mut Gd<GdRoleMover>) {
        self.phase_timer += delta;
        if self.phase_is("walk") {
            let arrived = self.drive_to(mover, self.current_target, self.arrival_distance);
            if arrived || self.phase_timer >= self.phase_limit {
                self.enter_idle(mover);
            }
        } else if self.phase_timer >= self.phase_limit {
            // 停留结束：取随机点开始行走
            self.current_target = self.random_wander_point();
            self.phase_limit = self.rand_range(self.walk_min, self.walk_max);
            self.set_phase(mover, "walk");
        }
    }

    /// 路径巡逻
    fn tick_patrol(&mut self, delta: f64, mover: &mut Gd<GdRoleMover>) {
        if self.patrol_points.len() == 0 {
            self.set_phase(mover, "idle");
            return;
        }

        // 停留阶段
        if self.phase_is("pause") {
            self.phase_timer += delta;
            if self.phase_timer >= self.patrol_pause {
                self.advance_patrol();
                self.set_phase(mover, "walk");
            }
            return;
        }

        if !self.phase_is("walk") {
            self.set_phase(mover, "walk");
            return;
        }

        self.phase_timer += delta;
        let Some(target) = self.patrol_points.get(self.patrol_index as usize) else {
            self.set_phase(mover, "idle");
            return;
        };
        if self.drive_to(mover, target, self.arrival_distance) {
            if self.patrol_pause > 0.0 {
                self.phase_timer = 0.0;
                self.set_phase(mover, "pause");
            } else {
                self.advance_patrol();
            }
        }
    }

    /// 跟随目标
    fn tick_follow(&mut self, mover: &mut Gd<GdRoleMover>) {
        let tpos = match self.follow_target.as_ref() {
            Some(t) if t.is_instance_valid() => t.get_global_position(),
            _ => {
                self.set_phase(mover, "idle");
                return;
            }
        };
        let mpos = mover.bind().base().get_global_position();
        if mpos.distance_to(tpos) > self.follow_stop_distance as f32 {
            self.set_phase(mover, "walk");
            self.drive_to(mover, tpos, self.follow_stop_distance);
        } else {
            self.set_phase(mover, "idle");
        }
    }

    /// 逃离目标
    fn tick_flee(&mut self, mover: &mut Gd<GdRoleMover>) {
        let tpos = match self.follow_target.as_ref() {
            Some(t) if t.is_instance_valid() => t.get_global_position(),
            _ => {
                self.set_phase(mover, "idle");
                return;
            }
        };
        let mpos = mover.bind().base().get_global_position();
        let dist = mpos.distance_to(tpos);
        if dist < self.flee_radius as f32 && dist > 0.001 {
            let dir = (mpos - tpos).normalized();
            mover.bind_mut().set_ai_direction(dir);
            self.set_phase(mover, "flee");
        } else {
            self.set_phase(mover, "idle");
        }
    }

    /// 猎手：以随机游走为底行为，目标进入视野 → 追击 → 进入攻击距离 → 攻击
    /// （视野丢失后 chase_memory 秒内继续追，超时回到游走）
    fn tick_hunter(&mut self, delta: f64, mover: &mut Gd<GdRoleMover>) {
        // 攻击冷却推进
        if self.attack_timer > 0.0 {
            self.attack_timer = (self.attack_timer - delta).max(0.0);
        }
        let tpos = match self.follow_target.as_ref() {
            Some(t) if t.is_instance_valid() => Some(t.get_global_position()),
            _ => None,
        };
        let mpos = mover.bind().base().get_global_position();

        // 判断目标可见性，刷新追击记忆
        let mut in_sight = false;
        if let Some(tp) = tpos {
            if mpos.distance_to(tp) <= self.sight_range as f32 {
                in_sight = true;
                self.chase_timer = self.chase_memory;
            }
        }

        let dist = tpos.map_or(f32::INFINITY, |tp| mpos.distance_to(tp));

        // 相位机：attack > chase > wander
        if self.phase_is("attack") {
            // 攻击中：面向目标原地不动，脱离攻击距离则继续追
            mover.bind_mut().stop();
            if let Some(tp) = tpos {
                if dist > self.attack_range as f32 * 1.5 {
                    self.set_phase(mover, "chase");
                    return;
                }
                let _ = tp;
            } else {
                self.set_phase(mover, "idle");
            }
            // 冷却好了再触发一次攻击信号
            if self.attack_timer <= 0.0 && dist <= self.attack_range as f32 {
                self.attack_timer = self.attack_cooldown;
                self.base_mut().emit_signal("s_attack", &[]);
            }
            return;
        }

        if in_sight && dist <= self.attack_range as f32 && self.attack_timer <= 0.0 {
            self.chase_timer = self.chase_memory;
            self.attack_timer = self.attack_cooldown;
            self.set_phase(mover, "attack");
            self.base_mut().emit_signal("s_attack", &[]);
            return;
        }

        let chasing = in_sight || self.chase_timer > 0.0;
        if chasing {
            if let Some(tp) = tpos {
                self.chase_timer = if in_sight { self.chase_memory } else { self.chase_timer - delta };
                if self.chase_timer <= 0.0 {
                    self.enter_idle(mover);
                    return;
                }
                self.set_phase(mover, "chase");
                self.drive_to(mover, tp, self.attack_range);
                return;
            }
        }

        // 无目标：随机游走
        self.tick_wander(delta, mover);
    }

    /// 驱动移动器走向目标点，返回是否已到达
    fn drive_to(&self, mover: &mut Gd<GdRoleMover>, target: Vector2, stop_distance: f64) -> bool {
        let mpos = mover.bind().base().get_global_position();
        if mpos.distance_to(target) <= stop_distance.max(self.arrival_distance) as f32 {
            mover.bind_mut().stop();
            return true;
        }
        mover.bind_mut().set_ai_target_position(target, stop_distance);
        false
    }

    /// 切换 AI 状态（idle/pause 时同时停住移动器）
    fn set_phase(&mut self, mover: &mut Gd<GdRoleMover>, phase: &str) {
        let gs = GString::from(phase);
        if self.phase != gs {
            self.phase = gs;
            self.phase_timer = 0.0;
            let var = self.phase.to_variant();
            self.base_mut().emit_signal("s_ai_state_changed", &[var]);
        }
        if phase == "idle" || phase == "pause" {
            mover.bind_mut().stop();
        }
    }

    fn enter_idle(&mut self, mover: &mut Gd<GdRoleMover>) {
        self.phase_limit = self.rand_range(self.idle_min, self.idle_max);
        self.set_phase(mover, "idle");
    }

    fn phase_is(&self, s: &str) -> bool {
        self.phase == GString::from(s)
    }

    /// 推进巡逻索引（循环或往返）
    fn advance_patrol(&mut self) {
        let count = self.patrol_points.len() as i64;
        if count == 0 {
            return;
        }
        if self.patrol_loop {
            self.patrol_index = (self.patrol_index + 1) % count;
        } else {
            let next = self.patrol_index + self.patrol_dir;
            if next < 0 || next >= count {
                self.patrol_dir = -self.patrol_dir;
                self.patrol_index =
                    (self.patrol_index + self.patrol_dir).clamp(0, count - 1);
            } else {
                self.patrol_index = next;
            }
        }
    }

    /// 在家周围随机取点（均匀圆盘分布）
    fn random_wander_point(&self) -> Vector2 {
        let mut rng = rand::thread_rng();
        let angle: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
        let t: f64 = rng.gen_range(0.0..1.0);
        let r = self.wander_radius * t.sqrt();
        self.home
            + Vector2::new((angle.cos() * r) as f32, (angle.sin() * r) as f32)
    }

    /// [min, max) 随机数
    fn rand_range(&self, min: f64, max: f64) -> f64 {
        if max <= min {
            return min;
        }
        rand::thread_rng().gen_range(min..max)
    }
}
