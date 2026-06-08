// entity: 过程化实体生成系统
//
// 通过实体模板（EntityTemplate）+ 缩放曲线 + 深度/难度，自动生成具体数值
// 无需为每个难度逐级定义怪物/道具数值，只需定义基础值和缩放规则
//
// 核心概念：
// - EntityTemplate: 实体模板，定义基础数值和缩放曲线
// - EntityStats: 生成的具体数值实例
// - EntityPool: 实体池，管理所有模板，按深度/标签筛选并随机选取
// - StatScale: 单个属性的缩放规则（基础值 + 缩放曲线 + 随机波动）

use crate::rogue::rng::GameRng;
use crate::rogue::stats::GrowthCurve;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatScale {
    pub base: f64,
    pub curve: GrowthCurve,
    pub variance: f64,
}

impl StatScale {
    pub fn fixed(value: f64) -> Self {
        StatScale {
            base: value,
            curve: GrowthCurve::Linear {
                base: value,
                per_level: 0.0,
            },
            variance: 0.0,
        }
    }

    pub fn linear(base: f64, per_level: f64, variance: f64) -> Self {
        StatScale {
            base,
            curve: GrowthCurve::Linear { base, per_level },
            variance,
        }
    }

    pub fn exponential(base: f64, growth: f64, variance: f64) -> Self {
        StatScale {
            base,
            curve: GrowthCurve::Exponential { base, growth },
            variance,
        }
    }

    pub fn scale(&self, depth: usize, rng: &mut GameRng) -> f64 {
        let base_value = self.curve.value_at(depth);
        if self.variance > 0.0 {
            let noise = (rng.next_float() - 0.5) * 2.0 * self.variance * base_value;
            (base_value + noise).max(1.0)
        } else {
            base_value.max(1.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityType(pub String);

impl EntityType {
    pub fn new(s: &str) -> Self {
        EntityType(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const MONSTER: &'static str = "monster";
    pub const WEAPON: &'static str = "weapon";
    pub const ARMOR: &'static str = "armor";
    pub const ITEM: &'static str = "item";
    pub const EXIT: &'static str = "exit";
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for EntityType {
    fn from(s: &str) -> Self {
        EntityType(s.to_string())
    }
}

impl From<String> for EntityType {
    fn from(s: String) -> Self {
        EntityType(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTemplate {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub stats: HashMap<String, StatScale>,
    pub tags: Vec<String>,
    pub weight: f64,
    pub min_depth: usize,
    pub max_depth: usize,
}

impl EntityTemplate {
    pub fn new(id: &str, name: &str, entity_type: EntityType) -> Self {
        EntityTemplate {
            id: id.to_string(),
            name: name.to_string(),
            entity_type,
            stats: HashMap::new(),
            tags: Vec::new(),
            weight: 1.0,
            min_depth: 0,
            max_depth: usize::MAX,
        }
    }

    pub fn with_stat(mut self, stat_name: &str, scale: StatScale) -> Self {
        self.stats.insert(stat_name.to_string(), scale);
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_depth_range(mut self, min: usize, max: usize) -> Self {
        self.min_depth = min;
        self.max_depth = max;
        self
    }

    pub fn is_available(&self, depth: usize) -> bool {
        depth >= self.min_depth && depth <= self.max_depth
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    pub fn generate(&self, depth: usize, rng: &mut GameRng) -> EntityStats {
        let mut stats = HashMap::new();
        for (stat_name, scale) in &self.stats {
            stats.insert(stat_name.clone(), scale.scale(depth, rng));
        }
        EntityStats {
            template_id: self.id.clone(),
            name: self.name.clone(),
            entity_type: self.entity_type.clone(),
            stats,
            depth,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityStats {
    pub template_id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub stats: HashMap<String, f64>,
    pub depth: usize,
}

impl EntityStats {
    pub fn get(&self, stat: &str) -> f64 {
        self.stats.get(stat).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, stat: &str, value: f64) {
        self.stats.insert(stat.to_string(), value);
    }

    pub fn modify(&mut self, stat: &str, delta: f64) {
        let current = self.get(stat);
        self.stats.insert(stat.to_string(), current + delta);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPool {
    templates: HashMap<String, EntityTemplate>,
}

impl EntityPool {
    pub fn new() -> Self {
        EntityPool {
            templates: HashMap::new(),
        }
    }

    pub fn register(&mut self, template: EntityTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn get(&self, id: &str) -> Option<&EntityTemplate> {
        self.templates.get(id)
    }

    pub fn generate(&self, id: &str, depth: usize, rng: &mut GameRng) -> Option<EntityStats> {
        self.templates.get(id).map(|t| t.generate(depth, rng))
    }

    pub fn available_at(&self, depth: usize) -> Vec<&EntityTemplate> {
        self.templates
            .values()
            .filter(|t| t.is_available(depth))
            .collect()
    }

    pub fn available_by_type(&self, entity_type: &EntityType, depth: usize) -> Vec<&EntityTemplate> {
        self.templates
            .values()
            .filter(|t| t.is_available(depth) && &t.entity_type == entity_type)
            .collect()
    }

    pub fn available_by_tag(&self, tag: &str, depth: usize) -> Vec<&EntityTemplate> {
        self.templates
            .values()
            .filter(|t| t.is_available(depth) && t.has_tag(tag))
            .collect()
    }

    pub fn roll_random(
        &self,
        entity_type: &EntityType,
        depth: usize,
        rng: &mut GameRng,
    ) -> Option<EntityStats> {
        let available = self.available_by_type(entity_type, depth);
        if available.is_empty() {
            return None;
        }
        let weights: Vec<f64> = available.iter().map(|t| t.weight).collect();
        let idx = rng.choose_weighted_idx(&weights)?;
        Some(available[idx].generate(depth, rng))
    }

    pub fn roll_random_n(
        &self,
        entity_type: &EntityType,
        depth: usize,
        count: usize,
        rng: &mut GameRng,
    ) -> Vec<EntityStats> {
        let mut results = Vec::new();
        for _ in 0..count {
            if let Some(entity) = self.roll_random(entity_type, depth, rng) {
                results.push(entity);
            }
        }
        results
    }

    pub fn roll_from_tags(
        &self,
        tags: &[String],
        depth: usize,
        rng: &mut GameRng,
    ) -> Option<EntityStats> {
        let available: Vec<&EntityTemplate> = self
            .templates
            .values()
            .filter(|t| {
                t.is_available(depth) && tags.iter().any(|tag| t.has_tag(tag))
            })
            .collect();
        if available.is_empty() {
            return None;
        }
        let weights: Vec<f64> = available.iter().map(|t| t.weight).collect();
        let idx = rng.choose_weighted_idx(&weights)?;
        Some(available[idx].generate(depth, rng))
    }

    pub fn all_templates(&self) -> &HashMap<String, EntityTemplate> {
        &self.templates
    }
}

impl Default for EntityPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_pool() -> EntityPool {
        let mut pool = EntityPool::new();

        pool.register(
            EntityTemplate::new("goblin", "哥布林", EntityType::new(EntityType::MONSTER))
                .with_stat("hp", StatScale::linear(20.0, 5.0, 0.1))
                .with_stat("atk", StatScale::linear(5.0, 1.5, 0.1))
                .with_stat("def", StatScale::fixed(2.0))
                .with_tag("melee")
                .with_weight(5.0),
        );

        pool.register(
            EntityTemplate::new("skeleton", "骷髅兵", EntityType::new(EntityType::MONSTER))
                .with_stat("hp", StatScale::linear(30.0, 8.0, 0.1))
                .with_stat("atk", StatScale::linear(8.0, 2.0, 0.1))
                .with_stat("def", StatScale::fixed(5.0))
                .with_tag("melee")
                .with_tag("undead")
                .with_weight(3.0)
                .with_depth_range(2, usize::MAX),
        );

        pool.register(
            EntityTemplate::new("dragon", "巨龙", EntityType::new(EntityType::MONSTER))
                .with_stat("hp", StatScale::exponential(100.0, 1.2, 0.05))
                .with_stat("atk", StatScale::exponential(20.0, 1.15, 0.05))
                .with_tag("boss")
                .with_weight(0.5)
                .with_depth_range(5, usize::MAX),
        );

        pool.register(
            EntityTemplate::new("wooden_sword", "木剑", EntityType::new(EntityType::WEAPON))
                .with_stat("atk", StatScale::linear(3.0, 1.0, 0.1))
                .with_tag("weapon")
                .with_weight(4.0),
        );

        pool.register(
            EntityTemplate::new("iron_shield", "铁盾", EntityType::new(EntityType::ARMOR))
                .with_stat("def", StatScale::linear(3.0, 0.8, 0.1))
                .with_tag("armor")
                .with_weight(3.0),
        );

        pool.register(
            EntityTemplate::new("heal_potion", "治疗药水", EntityType::new(EntityType::ITEM))
                .with_stat("heal", StatScale::linear(15.0, 3.0, 0.1))
                .with_tag("consumable")
                .with_weight(5.0),
        );

        pool
    }

    #[test]
    fn test_stat_scale_fixed() {
        let scale = StatScale::fixed(10.0);
        let mut rng = GameRng::from_seed(42);
        assert_eq!(scale.scale(1, &mut rng), 10.0);
        assert_eq!(scale.scale(10, &mut rng), 10.0);
    }

    #[test]
    fn test_stat_scale_linear() {
        let scale = StatScale::linear(20.0, 5.0, 0.0);
        let mut rng = GameRng::from_seed(42);
        let v1 = scale.scale(1, &mut rng);
        let v5 = scale.scale(5, &mut rng);
        assert!((v1 - 20.0).abs() < 0.01);
        assert!(v5 > v1);
    }

    #[test]
    fn test_stat_scale_variance() {
        let scale = StatScale::linear(100.0, 0.0, 0.2);
        let mut rng = GameRng::from_seed(42);
        let mut values = Vec::new();
        for _ in 0..100 {
            values.push(scale.scale(1, &mut rng));
        }
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(min < 100.0);
        assert!(max > 100.0);
    }

    #[test]
    fn test_entity_generation() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);

        let goblin_d1 = pool.generate("goblin", 1, &mut rng).unwrap();
        let goblin_d5 = pool.generate("goblin", 5, &mut rng).unwrap();

        assert!(goblin_d5.get("hp") > goblin_d1.get("hp"));
        assert!(goblin_d5.get("atk") > goblin_d1.get("atk"));
        assert_eq!(goblin_d1.get("def"), 2.0);
        assert_eq!(goblin_d5.get("def"), 2.0);
    }

    #[test]
    fn test_depth_restriction() {
        let pool = make_test_pool();
        assert!(pool.get("skeleton").unwrap().is_available(1) == false);
        assert!(pool.get("skeleton").unwrap().is_available(2) == true);
        assert!(pool.get("dragon").unwrap().is_available(3) == false);
        assert!(pool.get("dragon").unwrap().is_available(5) == true);
    }

    #[test]
    fn test_roll_random_by_type() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);

        let monster = pool.roll_random(&EntityType::new(EntityType::MONSTER), 1, &mut rng);
        assert!(monster.is_some());
        let m = monster.unwrap();
        assert_eq!(m.entity_type, EntityType::new(EntityType::MONSTER));

        let weapon = pool.roll_random(&EntityType::new(EntityType::WEAPON), 1, &mut rng);
        assert!(weapon.is_some());
        assert_eq!(weapon.unwrap().entity_type, EntityType::new(EntityType::WEAPON));
    }

    #[test]
    fn test_roll_random_n() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);
        let items = pool.roll_random_n(&EntityType::new(EntityType::ITEM), 1, 3, &mut rng);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_available_by_type() {
        let pool = make_test_pool();
        let monsters_d1 = pool.available_by_type(&EntityType::new(EntityType::MONSTER), 1);
        assert_eq!(monsters_d1.len(), 1);

        let monsters_d5 = pool.available_by_type(&EntityType::new(EntityType::MONSTER), 5);
        assert_eq!(monsters_d5.len(), 3);
    }

    #[test]
    fn test_roll_from_tags() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);
        let entity = pool.roll_from_tags(&["undead".to_string()], 3, &mut rng);
        assert!(entity.is_some());
        assert_eq!(entity.unwrap().template_id, "skeleton");
    }

    #[test]
    fn test_same_seed_same_entity() {
        let pool = make_test_pool();
        let mut rng1 = GameRng::from_seed(42);
        let mut rng2 = GameRng::from_seed(42);

        let e1 = pool.roll_random(&EntityType::new(EntityType::MONSTER), 3, &mut rng1).unwrap();
        let e2 = pool.roll_random(&EntityType::new(EntityType::MONSTER), 3, &mut rng2).unwrap();

        assert_eq!(e1.template_id, e2.template_id);
        assert!((e1.get("hp") - e2.get("hp")).abs() < 0.001);
    }

    #[test]
    fn test_entity_stats_modify() {
        let mut stats = EntityStats {
            template_id: "test".to_string(),
            name: "Test".to_string(),
            entity_type: EntityType::new(EntityType::MONSTER),
            stats: HashMap::new(),
            depth: 1,
        };
        stats.set("hp", 100.0);
        assert_eq!(stats.get("hp"), 100.0);
        stats.modify("hp", -30.0);
        assert_eq!(stats.get("hp"), 70.0);
    }
}
