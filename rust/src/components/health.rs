// GdHealth - 血量组件
// 挂到任何角色/怪物/可破坏物上即可获得：
//   - 血量、最大血量、无敌帧（受伤后短时间免疫）、上帝模式
//   - 信号：s_damaged(amount, current) / s_healed(amount, current) / s_died
// 配合 GdHurtbox 可自动接收攻击盒伤害；也可直接 take_damage()。

use godot::prelude::*;
use godot::builtin::GString;
use godot::classes::{INode, Node};

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdHealth {
    /// 最大血量
    #[export]
    max_health: f64,
    /// 受伤后的无敌时长（秒），0 = 无无敌帧
    #[export]
    invincible_time: f64,
    /// 上帝模式：免疫一切伤害
    #[export]
    god_mode: bool,

    // ---- 运行时状态 ----
    health: f64,
    invincible_timer: f64,
    dead: bool,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdHealth {
    fn init(base: Base<Node>) -> Self {
        Self {
            max_health: 100.0,
            invincible_time: 0.5,
            god_mode: false,
            health: 100.0,
            invincible_timer: 0.0,
            dead: false,
            base,
        }
    }

    fn ready(&mut self) {
        self.health = self.health.min(self.max_health);
    }

    fn process(&mut self, delta: f64) {
        if self.invincible_timer > 0.0 {
            self.invincible_timer = (self.invincible_timer - delta).max(0.0);
        }
    }
}

#[godot_api]
impl GdHealth {
    /// 受到伤害（触发 s_damaged；无敌帧内或 god_mode 下无效；血量归零触发 s_died）
    #[signal]
    fn s_damaged(amount: f64, current: f64);

    /// 治疗恢复（触发 s_healed）
    #[signal]
    fn s_healed(amount: f64, current: f64);

    /// 死亡（血量归零时触发一次）
    #[signal]
    fn s_died();

    /// 受到伤害，返回是否实际生效
    #[func]
    pub fn take_damage(&mut self, amount: f64) -> bool {
        if self.dead || amount <= 0.0 {
            return false;
        }
        if self.god_mode || self.invincible_timer > 0.0 {
            return false;
        }
        self.health = (self.health - amount).max(0.0);
        if self.invincible_time > 0.0 {
            self.invincible_timer = self.invincible_time;
        }
        let (a, h) = (amount, self.health);
        self.base_mut().emit_signal("s_damaged", &[a.to_variant(), h.to_variant()]);
        if self.health <= 0.0 {
            self.dead = true;
            self.base_mut().emit_signal("s_died", &[]);
        }
        true
    }

    /// 治疗，返回是否实际生效
    #[func]
    pub fn heal(&mut self, amount: f64) -> bool {
        if self.dead || amount <= 0.0 || self.health >= self.max_health {
            return false;
        }
        self.health = (self.health + amount).min(self.max_health);
        let (a, h) = (amount, self.health);
        self.base_mut().emit_signal("s_healed", &[a.to_variant(), h.to_variant()]);
        true
    }

    /// 直接击杀（无视无敌帧，god_mode 下也生效）
    #[func]
    pub fn kill(&mut self) {
        if self.dead {
            return;
        }
        self.health = 0.0;
        self.dead = true;
        self.base_mut().emit_signal("s_died", &[]);
    }

    /// 复活到指定血量（默认满血），并清除死亡与无敌状态
    #[func]
    pub fn revive(&mut self, hp: f64) {
        self.dead = false;
        self.invincible_timer = 0.0;
        self.health = hp.max(1.0).min(self.max_health);
    }

    /// 当前血量
    #[func]
    pub fn get_health(&self) -> f64 {
        self.health
    }

    /// 设置当前血量（0..max，不触发信号）
    #[func]
    pub fn set_health(&mut self, value: f64) {
        self.health = value.clamp(0.0, self.max_health);
        self.dead = self.health <= 0.0;
    }

    /// 血量比例 0..1（HUD 血条直连）
    #[func]
    pub fn get_health_ratio(&self) -> f64 {
        self.health / self.max_health
    }

    /// 是否存活
    #[func]
    pub fn is_alive(&self) -> bool {
        !self.dead
    }

    /// 是否处于无敌帧
    #[func]
    pub fn is_invincible(&self) -> bool {
        self.invincible_timer > 0.0
    }

    /// 剩余无敌时间（秒）
    #[func]
    pub fn get_invincible_left(&self) -> f64 {
        self.invincible_timer
    }
}
