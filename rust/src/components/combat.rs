// GdHitbox / GdHurtbox - 攻击盒与受击盒组件
//
// 攻击判定数据流：
//   GdHitbox(Area2D) --area_entered--> GdHurtbox(Area2D) --take_damage--> GdHealth
//
// 用法：
//   - 攻击方：挂 GdHitbox + CollisionShape2D，设 damage/cooldown，
//     碰撞掩码指向受击方 layer；攻击瞬间可 set_enabled(true)（如近战挥击窗口）
//   - 受击方：挂 GdHurtbox + CollisionShape2D（layer 与攻击方 mask 对应），
//     health_path 指向同树上的 GdHealth（默认自动在父节点上找）

use godot::prelude::*;
use godot::builtin::{GString, NodePath};
use godot::classes::{Area2D, IArea2D};

use super::health::GdHealth;

// ==================== GdHitbox ====================

#[derive(GodotClass)]
#[class(base = Area2D)]
pub struct GdHitbox {
    /// 每次命中造成的伤害
    #[export]
    damage: f64,
    /// 命中冷却（秒）：冷却期间不再触发新的命中，防止穿透刷伤害
    #[export]
    cooldown: f64,
    /// 是否启用（近战挥击窗口 / 远程常开）
    #[export]
    enabled: bool,

    // ---- 运行时状态 ----
    cooldown_timer: f64,

    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for GdHitbox {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            damage: 10.0,
            cooldown: 0.3,
            enabled: true,
            cooldown_timer: 0.0,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer = (self.cooldown_timer - delta).max(0.0);
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        if !self.enabled || self.cooldown_timer > 0.0 {
            return;
        }
        // 主动扫描重叠区域（不依赖信号，命中时机由攻击方控制）
        let areas = self.base_mut().get_overlapping_areas();
        for i in 0..areas.len() {
            let Some(area) = areas.get(i) else { continue; };
            let Ok(mut hurt) = area.clone().try_cast::<GdHurtbox>() else {
                continue;
            };
            let applied = hurt.bind_mut().take_damage(self.damage);
            if applied {
                self.cooldown_timer = self.cooldown;
                let target = area.get_name();
                self.base_mut()
                    .emit_signal("s_hit", &[target.to_variant()]);
                break;
            }
        }
    }
}

#[godot_api]
impl GdHitbox {
    /// 命中一个受击盒时触发（参数：受击节点名）
    #[signal]
    fn s_hit(target: GString);

}

// ==================== GdHurtbox ====================

#[derive(GodotClass)]
#[class(base = Area2D)]
pub struct GdHurtbox {
    /// 血量组件路径（默认空 = 自动在父节点上找 GdHealth）
    #[export]
    health_path: NodePath,
    /// 受击后无敌时间交给 GdHealth 处理，此处仅转发

    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for GdHurtbox {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            health_path: NodePath::default(),
            base,
        }
    }
}

#[godot_api]
impl GdHurtbox {
    /// 受到伤害时触发（参数：伤害值），无论是否致死
    #[signal]
    fn s_hurt(amount: f64);

    /// 接收伤害并转发给血量组件，返回是否生效
    #[func]
    pub fn take_damage(&mut self, amount: f64) -> bool {
        let Some(mut health) = self.resolve_health() else {
            godot_error!(
                "[GdHurtbox] 未找到 GdHealth（节点 {}，health_path=\"{}\"）",
                self.base().get_name(),
                self.health_path
            );
            return false;
        };
        let applied = health.bind_mut().take_damage(amount);
        if applied {
            let a = amount;
            self.base_mut().emit_signal("s_hurt", &[a.to_variant()]);
        }
        applied
    }

    // ---- 内部实现 ----

    /// 解析血量组件：显式路径优先，其次父节点自身，最后父节点的子节点（兄弟）
    fn resolve_health(&self) -> Option<Gd<GdHealth>> {
        if !self.health_path.is_empty() {
            let path = self.health_path.clone();
            if let Some(node) = self.base().get_node_or_null(&path) {
                return node.try_cast::<GdHealth>().ok();
            }
        }
        let parent = self.base().get_parent()?;
        // 父节点自身
        if let Ok(h) = parent.clone().try_cast::<GdHealth>() {
            return Some(h);
        }
        // 兄弟节点（挂在同一父级下的 GdHealth）
        let children = parent.get_children();
        for i in 0..children.len() {
            let Some(child) = children.get(i) else { continue; };
            if let Ok(h) = child.try_cast::<GdHealth>() {
                return Some(h);
            }
        }
        None
    }
}
