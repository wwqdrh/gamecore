// stats: 数值系统
//
// 支持属性计算、Buff/Debuff 叠加、伤害公式、成长曲线
// Modifier 支持 Add/Multiply/Override/Min/Max 五种运算，按优先级计算

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type StatId = String;
pub type ModifierId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ModifierOp {
    Add,
    Multiply,
    Override,
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierSource {
    Equipment(String),
    Buff(String),
    Relic(String),
    Temporary(String),
}

impl ModifierSource {
    pub fn priority(&self) -> u32 {
        match self {
            ModifierSource::Equipment(_) => 0,
            ModifierSource::Buff(_) => 1,
            ModifierSource::Relic(_) => 2,
            ModifierSource::Temporary(_) => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub id: ModifierId,
    pub stat: StatId,
    pub operation: ModifierOp,
    pub value: f64,
    pub source: ModifierSource,
    pub duration: Option<f64>,
    pub stacks: usize,
    pub max_stacks: usize,
}

impl Modifier {
    pub fn new(id: &str, stat: &str, op: ModifierOp, value: f64) -> Self {
        Modifier {
            id: id.to_string(),
            stat: stat.to_string(),
            operation: op,
            value,
            source: ModifierSource::Buff(id.to_string()),
            duration: None,
            stacks: 1,
            max_stacks: 1,
        }
    }

    pub fn buff(id: &str, stat: &str, op: ModifierOp, value: f64) -> Self {
        Modifier {
            id: id.to_string(),
            stat: stat.to_string(),
            operation: op,
            value,
            source: ModifierSource::Buff(id.to_string()),
            duration: None,
            stacks: 1,
            max_stacks: 1,
        }
    }

    pub fn equipment(id: &str, stat: &str, op: ModifierOp, value: f64) -> Self {
        Modifier {
            id: id.to_string(),
            stat: stat.to_string(),
            operation: op,
            value,
            source: ModifierSource::Equipment(id.to_string()),
            duration: None,
            stacks: 1,
            max_stacks: 1,
        }
    }

    pub fn relic(id: &str, stat: &str, op: ModifierOp, value: f64) -> Self {
        Modifier {
            id: id.to_string(),
            stat: stat.to_string(),
            operation: op,
            value,
            source: ModifierSource::Relic(id.to_string()),
            duration: None,
            stacks: 1,
            max_stacks: 1,
        }
    }

    pub fn temporary(id: &str, stat: &str, op: ModifierOp, value: f64, duration: f64) -> Self {
        Modifier {
            id: id.to_string(),
            stat: stat.to_string(),
            operation: op,
            value,
            source: ModifierSource::Temporary(id.to_string()),
            duration: Some(duration),
            stacks: 1,
            max_stacks: 1,
        }
    }

    pub fn with_stacks(mut self, stacks: usize, max_stacks: usize) -> Self {
        self.stacks = stacks.min(max_stacks);
        self.max_stacks = max_stacks;
        self
    }

    pub fn with_duration(mut self, duration: f64) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_source(mut self, source: ModifierSource) -> Self {
        self.source = source;
        self
    }

    pub fn effective_value(&self) -> f64 {
        self.value * self.stacks as f64
    }

    pub fn is_expired(&self) -> bool {
        match self.duration {
            Some(d) => d <= 0.0,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatSystem {
    base_stats: HashMap<StatId, f64>,
    min_values: HashMap<StatId, f64>,
    max_values: HashMap<StatId, f64>,
    modifiers: Vec<Modifier>,
}

impl StatSystem {
    pub fn new() -> Self {
        StatSystem {
            base_stats: HashMap::new(),
            min_values: HashMap::new(),
            max_values: HashMap::new(),
            modifiers: Vec::new(),
        }
    }

    pub fn set_base(&mut self, stat: StatId, value: f64) {
        self.base_stats.insert(stat, value);
    }

    pub fn set_min(&mut self, stat: StatId, value: f64) {
        self.min_values.insert(stat, value);
    }

    pub fn set_max(&mut self, stat: StatId, value: f64) {
        self.max_values.insert(stat, value);
    }

    pub fn base(&self, stat: &StatId) -> f64 {
        self.base_stats.get(stat).copied().unwrap_or(0.0)
    }

    pub fn add_modifier(&mut self, modifier: Modifier) {
        if let Some(existing) = self.modifiers.iter_mut().find(|m| m.id == modifier.id) {
            if existing.max_stacks > 1 && existing.stacks < existing.max_stacks {
                existing.stacks = existing.stacks.saturating_add(1).min(existing.max_stacks);
                return;
            }
        }
        self.modifiers.push(modifier);
    }

    pub fn remove_modifier(&mut self, id: &ModifierId) {
        self.modifiers.retain(|m| &m.id != id);
    }

    pub fn remove_modifiers_by_source(&mut self, source: &ModifierSource) {
        self.modifiers.retain(|m| &m.source != source);
    }

    pub fn remove_modifiers_by_stat(&mut self, stat: &StatId) {
        self.modifiers.retain(|m| m.stat != *stat);
    }

    pub fn get(&self, stat: &StatId) -> f64 {
        let base = self.base(stat);

        let mut add_sum = 0.0;
        let mut multiply_product = 1.0;
        let mut override_value: Option<f64> = None;
        let mut min_value: Option<f64> = None;
        let mut max_value: Option<f64> = None;

        for modifier in &self.modifiers {
            if &modifier.stat != stat {
                continue;
            }

            match modifier.operation {
                ModifierOp::Add => {
                    add_sum += modifier.effective_value();
                }
                ModifierOp::Multiply => {
                    multiply_product *= 1.0 + modifier.effective_value();
                }
                ModifierOp::Override => {
                    match override_value {
                        None => override_value = Some(modifier.effective_value()),
                        Some(v) if modifier.effective_value() > v => {
                            override_value = Some(modifier.effective_value());
                        }
                        _ => {}
                    }
                }
                ModifierOp::Min => {
                    let v = modifier.effective_value();
                    min_value = Some(match min_value {
                        None => v,
                        Some(mv) => mv.max(v),
                    });
                }
                ModifierOp::Max => {
                    let v = modifier.effective_value();
                    max_value = Some(match max_value {
                        None => v,
                        Some(mv) => mv.min(v),
                    });
                }
            }
        }

        let mut result = if let Some(ov) = override_value {
            ov
        } else {
            (base + add_sum) * multiply_product
        };

        result = match min_value {
            Some(mv) => result.max(mv),
            None => result,
        };

        result = match max_value {
            Some(mv) => result.min(mv),
            None => result,
        };

        if let Some(&min) = self.min_values.get(stat) {
            result = result.max(min);
        }
        if let Some(&max) = self.max_values.get(stat) {
            result = result.min(max);
        }
        result
    }

    pub fn tick(&mut self, dt: f64) {
        for modifier in &mut self.modifiers {
            if let Some(ref mut duration) = modifier.duration {
                *duration -= dt;
            }
        }
        self.modifiers.retain(|m| !m.is_expired());
    }

    pub fn modifiers_for_stat(&self, stat: &StatId) -> Vec<&Modifier> {
        self.modifiers
            .iter()
            .filter(|m| m.stat == *stat)
            .collect()
    }

    pub fn all_modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub fn clear_temporary(&mut self) {
        self.modifiers
            .retain(|m| !matches!(m.source, ModifierSource::Temporary(_)));
    }
}

impl Default for StatSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrowthCurve {
    Linear { base: f64, per_level: f64 },
    Exponential { base: f64, growth: f64 },
    Logarithmic { base: f64, scale: f64 },
    Step { thresholds: Vec<(usize, f64)> },
}

impl GrowthCurve {
    pub fn value_at(&self, level: usize) -> f64 {
        match self {
            GrowthCurve::Linear { base, per_level } => {
                base + per_level * (level as f64 - 1.0).max(0.0)
            }
            GrowthCurve::Exponential { base, growth } => {
                base * growth.powi(level as i32 - 1)
            }
            GrowthCurve::Logarithmic { base, scale } => {
                base + scale * (level as f64).ln()
            }
            GrowthCurve::Step { thresholds } => {
                let mut result = 0.0;
                for &(threshold_level, value) in thresholds {
                    if level >= threshold_level {
                        result = value;
                    } else {
                        break;
                    }
                }
                result
            }
        }
    }

    pub fn linear(base: f64, per_level: f64) -> Self {
        GrowthCurve::Linear { base, per_level }
    }

    pub fn exponential(base: f64, growth: f64) -> Self {
        GrowthCurve::Exponential { base, growth }
    }

    pub fn logarithmic(base: f64, scale: f64) -> Self {
        GrowthCurve::Logarithmic { base, scale }
    }

    pub fn step(thresholds: Vec<(usize, f64)>) -> Self {
        GrowthCurve::Step { thresholds }
    }
}

pub struct DamageFormula;

impl DamageFormula {
    pub fn physical_damage(attack: f64, defense: f64) -> f64 {
        let reduction = defense / (defense + 100.0);
        attack * (1.0 - reduction)
    }

    pub fn flat_damage(base: f64, defense: f64) -> f64 {
        (base - defense).max(1.0)
    }

    pub fn percent_damage(current_hp: f64, percent: f64) -> f64 {
        current_hp * percent
    }

    pub fn flat_then_percent(base: f64, attack: f64, defense: f64, amp: f64) -> f64 {
        let raw = base + attack;
        let reduction = defense / (defense + 100.0);
        raw * (1.0 - reduction) * (1.0 + amp)
    }

    pub fn critical_damage(damage: f64, crit_multiplier: f64) -> f64 {
        damage * crit_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_stat() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        assert_eq!(sys.get(&"hp".to_string()), 100.0);
    }

    #[test]
    fn test_add_modifier() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        sys.add_modifier(Modifier::buff("hp_up", "hp", ModifierOp::Add, 20.0));
        assert_eq!(sys.get(&"hp".to_string()), 120.0);
    }

    #[test]
    fn test_multiply_modifier() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        sys.add_modifier(Modifier::buff("hp_mult", "hp", ModifierOp::Multiply, 0.5));
        assert_eq!(sys.get(&"hp".to_string()), 150.0);
    }

    #[test]
    fn test_add_then_multiply() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        sys.add_modifier(Modifier::buff("hp_up", "hp", ModifierOp::Add, 20.0));
        sys.add_modifier(Modifier::buff("hp_mult", "hp", ModifierOp::Multiply, 0.5));
        assert_eq!(sys.get(&"hp".to_string()), 180.0);
    }

    #[test]
    fn test_override_modifier() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        sys.add_modifier(Modifier::buff("hp_override", "hp", ModifierOp::Override, 50.0));
        assert_eq!(sys.get(&"hp".to_string()), 50.0);
    }

    #[test]
    fn test_min_max_clamp() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        sys.set_min("hp".to_string(), 1.0);
        sys.set_max("hp".to_string(), 200.0);
        sys.add_modifier(Modifier::buff("hp_up", "hp", ModifierOp::Add, 150.0));
        assert_eq!(sys.get(&"hp".to_string()), 200.0);
    }

    #[test]
    fn test_modifier_stacking() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        sys.add_modifier(
            Modifier::buff("stack", "hp", ModifierOp::Add, 10.0).with_stacks(1, 5),
        );
        sys.add_modifier(
            Modifier::buff("stack", "hp", ModifierOp::Add, 10.0).with_stacks(1, 5),
        );
        let mods = sys.modifiers_for_stat(&"hp".to_string());
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].stacks, 2);
        assert_eq!(sys.get(&"hp".to_string()), 120.0);
    }

    #[test]
    fn test_temporary_modifier_expiry() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        sys.add_modifier(Modifier::temporary("temp_hp", "hp", ModifierOp::Add, 30.0, 2.0));
        assert_eq!(sys.get(&"hp".to_string()), 130.0);

        sys.tick(1.0);
        assert_eq!(sys.get(&"hp".to_string()), 130.0);

        sys.tick(1.0);
        assert_eq!(sys.get(&"hp".to_string()), 100.0);
    }

    #[test]
    fn test_remove_modifier() {
        let mut sys = StatSystem::new();
        sys.set_base("hp".to_string(), 100.0);
        sys.add_modifier(Modifier::buff("hp_up", "hp", ModifierOp::Add, 20.0));
        assert_eq!(sys.get(&"hp".to_string()), 120.0);

        sys.remove_modifier(&"hp_up".to_string());
        assert_eq!(sys.get(&"hp".to_string()), 100.0);
    }

    #[test]
    fn test_growth_curves() {
        let linear = GrowthCurve::linear(100.0, 10.0);
        assert_eq!(linear.value_at(1), 100.0);
        assert_eq!(linear.value_at(5), 140.0);

        let exp = GrowthCurve::exponential(100.0, 1.1);
        assert!((exp.value_at(1) - 100.0).abs() < 0.001);
        assert!((exp.value_at(5) - 100.0 * 1.1_f64.powi(4)).abs() < 0.001);

        let log = GrowthCurve::logarithmic(50.0, 20.0);
        assert!((log.value_at(1) - 50.0).abs() < 0.001);

        let step = GrowthCurve::step(vec![(1, 10.0), (5, 20.0), (10, 30.0)]);
        assert_eq!(step.value_at(1), 10.0);
        assert_eq!(step.value_at(5), 20.0);
        assert_eq!(step.value_at(10), 30.0);
    }

    #[test]
    fn test_damage_formula() {
        let dmg = DamageFormula::physical_damage(100.0, 50.0);
        assert!(dmg > 0.0 && dmg < 100.0);

        let dmg = DamageFormula::flat_damage(100.0, 30.0);
        assert_eq!(dmg, 70.0);

        let dmg = DamageFormula::flat_damage(10.0, 30.0);
        assert_eq!(dmg, 1.0);

        let dmg = DamageFormula::percent_damage(200.0, 0.3);
        assert_eq!(dmg, 60.0);

        let crit = DamageFormula::critical_damage(100.0, 2.0);
        assert_eq!(crit, 200.0);
    }
}
