// loot: 战利品/掉落表系统
//
// 支持权重随机、条件过滤、唯一物品追踪、层数限制
// 支持嵌套掉落表和组合条件逻辑

use super::rng::GameRng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LootCondition {
    Always,
    Chance(f64),
    OneIn(usize),
    IfFlag(String),
    MinDepth(usize),
    MaxDepth(usize),
    And(Box<LootCondition>, Box<LootCondition>),
    Or(Box<LootCondition>, Box<LootCondition>),
    Not(Box<LootCondition>),
}

impl LootCondition {
    pub fn evaluate(&self, context: &LootContext) -> bool {
        match self {
            LootCondition::Always => true,
            LootCondition::Chance(chance) => context._last_roll.map(|r| r < *chance).unwrap_or(false),
            LootCondition::OneIn(n) => context._last_roll.map(|r| r < 1.0 / *n as f64).unwrap_or(false),
            LootCondition::IfFlag(flag) => context.flags.contains(flag),
            LootCondition::MinDepth(d) => context.depth >= *d,
            LootCondition::MaxDepth(d) => context.depth <= *d,
            LootCondition::And(a, b) => a.evaluate(context) && b.evaluate(context),
            LootCondition::Or(a, b) => a.evaluate(context) || b.evaluate(context),
            LootCondition::Not(c) => !c.evaluate(context),
        }
    }

    pub fn chance(chance: f64) -> Self {
        LootCondition::Chance(chance)
    }

    pub fn one_in(n: usize) -> Self {
        LootCondition::OneIn(n)
    }

    pub fn if_flag(flag: &str) -> Self {
        LootCondition::IfFlag(flag.to_string())
    }

    pub fn min_depth(d: usize) -> Self {
        LootCondition::MinDepth(d)
    }

    pub fn max_depth(d: usize) -> Self {
        LootCondition::MaxDepth(d)
    }

    pub fn and(self, other: LootCondition) -> Self {
        LootCondition::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: LootCondition) -> Self {
        LootCondition::Or(Box::new(self), Box::new(other))
    }

    pub fn not(self) -> Self {
        LootCondition::Not(Box::new(self))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LootContext {
    pub depth: usize,
    pub flags: HashSet<String>,
    pub rolled_unique: HashSet<usize>,
    #[serde(skip)]
    pub _last_roll: Option<f64>,
}

impl LootContext {
    pub fn new(depth: usize) -> Self {
        LootContext {
            depth,
            flags: HashSet::new(),
            rolled_unique: HashSet::new(),
            _last_roll: None,
        }
    }

    pub fn with_flag(mut self, flag: &str) -> Self {
        self.flags.insert(flag.to_string());
        self
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootEntry<T: Clone> {
    pub item: T,
    pub weight: f64,
    pub condition: Option<LootCondition>,
    pub unique: bool,
    pub id: Option<usize>,
    pub min_depth: usize,
    pub max_depth: usize,
}

impl<T: Clone> LootEntry<T> {
    pub fn new(item: T, weight: f64) -> Self {
        LootEntry {
            item,
            weight,
            condition: None,
            unique: false,
            id: None,
            min_depth: 0,
            max_depth: usize::MAX,
        }
    }

    pub fn with_condition(mut self, condition: LootCondition) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn with_id(mut self, id: usize) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_depth_range(mut self, min: usize, max: usize) -> Self {
        self.min_depth = min;
        self.max_depth = max;
        self
    }

    fn is_available(&self, context: &LootContext) -> bool {
        if context.depth < self.min_depth || context.depth > self.max_depth {
            return false;
        }
        if self.unique {
            if let Some(id) = self.id {
                if context.rolled_unique.contains(&id) {
                    return false;
                }
            }
        }
        if let Some(ref condition) = self.condition {
            condition.evaluate(context)
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootTable<T: Clone> {
    pub entries: Vec<LootEntry<T>>,
    pub guaranteed: Vec<LootEntry<T>>,
    pub min_drops: usize,
    pub max_drops: usize,
}

impl<T: Clone> LootTable<T> {
    pub fn new() -> Self {
        LootTable {
            entries: Vec::new(),
            guaranteed: Vec::new(),
            min_drops: 1,
            max_drops: 1,
        }
    }

    pub fn with_drops(mut self, min: usize, max: usize) -> Self {
        self.min_drops = min;
        self.max_drops = max;
        self
    }

    pub fn add(&mut self, entry: LootEntry<T>) {
        self.entries.push(entry);
    }

    pub fn add_guaranteed(&mut self, entry: LootEntry<T>) {
        self.guaranteed.push(entry);
    }

    pub fn roll(&self, rng: &mut GameRng, context: &mut LootContext) -> Vec<T> {
        let mut results = Vec::new();

        for entry in &self.guaranteed {
            if entry.is_available(context) {
                results.push(entry.item.clone());
                if entry.unique {
                    if let Some(id) = entry.id {
                        context.rolled_unique.insert(id);
                    }
                }
            }
        }

        let available: Vec<&LootEntry<T>> = self
            .entries
            .iter()
            .filter(|e| {
                let mut ctx = context.clone();
                ctx._last_roll = Some(rng.next_float());
                e.is_available(&ctx)
            })
            .collect();

        if available.is_empty() {
            return results;
        }

        let drop_count = if self.min_drops == self.max_drops {
            self.min_drops
        } else {
            rng.next_range(self.min_drops as i32, self.max_drops as i32 + 1) as usize
        };

        let total_weight: f64 = available.iter().map(|e| e.weight).sum();
        if total_weight <= 0.0 {
            return results;
        }

        for _ in 0..drop_count {
            let mut roll = rng.next_float() * total_weight;
            let mut chosen: Option<&LootEntry<T>> = None;

            for entry in &available {
                roll -= entry.weight;
                if roll <= 0.0 {
                    chosen = Some(entry);
                    break;
                }
            }

            let chosen = chosen.unwrap_or_else(|| available.last().unwrap());

            if chosen.unique {
                if let Some(id) = chosen.id {
                    if context.rolled_unique.contains(&id) {
                        continue;
                    }
                    context.rolled_unique.insert(id);
                }
            }

            results.push(chosen.item.clone());
        }

        results
    }

    pub fn roll_one(&self, rng: &mut GameRng, context: &mut LootContext) -> Option<T> {
        let items = self.roll(rng, context);
        items.into_iter().next()
    }

    pub fn roll_n(&self, rng: &mut GameRng, context: &mut LootContext, count: usize) -> Vec<T> {
        let mut all = Vec::new();
        for _ in 0..count {
            let items = self.roll(rng, context);
            all.extend(items);
        }
        all
    }
}

impl<T: Clone> Default for LootTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedLootTable {
    pub tables: Vec<(String, LootTable<String>)>,
}

impl NestedLootTable {
    pub fn new() -> Self {
        NestedLootTable { tables: Vec::new() }
    }

    pub fn add_table(&mut self, name: String, table: LootTable<String>) {
        self.tables.push((name, table));
    }

    pub fn roll_table(
        &self,
        table_name: &str,
        rng: &mut GameRng,
        context: &mut LootContext,
    ) -> Option<Vec<String>> {
        self.tables
            .iter()
            .find(|(name, _)| name == table_name)
            .map(|(_, table)| table.roll(rng, context))
    }
}

impl Default for NestedLootTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_roll() {
        let mut table: LootTable<&str> = LootTable::new();
        table.add(LootEntry::new("sword", 1.0));
        table.add(LootEntry::new("shield", 1.0));
        table.add(LootEntry::new("potion", 3.0));

        let mut rng = GameRng::from_seed(42);
        let mut context = LootContext::new(1);
        let items = table.roll(&mut rng, &mut context);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_weighted_distribution() {
        let mut table: LootTable<&str> = LootTable::new();
        table.add(LootEntry::new("common", 9.0));
        table.add(LootEntry::new("rare", 1.0));

        let mut rng = GameRng::from_seed(42);
        let mut common_count = 0usize;
        let mut rare_count = 0usize;

        for _ in 0..1000 {
            let mut context = LootContext::new(1);
            let items = table.roll(&mut rng, &mut context);
            for item in items {
                match item {
                    "common" => common_count += 1,
                    "rare" => rare_count += 1,
                    _ => {}
                }
            }
        }

        assert!(common_count > rare_count * 3);
    }

    #[test]
    fn test_guaranteed_drops() {
        let mut table: LootTable<&str> = LootTable::new();
        table.add_guaranteed(LootEntry::new("gold", 1.0));
        table.add(LootEntry::new("gem", 1.0));

        let mut rng = GameRng::from_seed(42);
        let mut context = LootContext::new(1);
        let items = table.roll(&mut rng, &mut context);
        assert!(items.contains(&"gold"));
    }

    #[test]
    fn test_multiple_drops() {
        let mut table: LootTable<&str> = LootTable::new().with_drops(2, 4);
        table.add(LootEntry::new("item", 1.0));
        table.add(LootEntry::new("item2", 1.0));
        table.add(LootEntry::new("item3", 1.0));

        let mut rng = GameRng::from_seed(42);
        let mut context = LootContext::new(1);
        let items = table.roll(&mut rng, &mut context);
        assert!(items.len() >= 2 && items.len() <= 4);
    }

    #[test]
    fn test_depth_restriction() {
        let mut table: LootTable<&str> = LootTable::new();
        table.add(LootEntry::new("early_item", 1.0).with_depth_range(0, 3));
        table.add(LootEntry::new("late_item", 1.0).with_depth_range(5, 10));

        let mut rng = GameRng::from_seed(42);
        let mut context = LootContext::new(1);
        let items = table.roll(&mut rng, &mut context);
        assert!(items.contains(&"early_item"));
        assert!(!items.contains(&"late_item"));

        let mut context = LootContext::new(7);
        let items = table.roll(&mut rng, &mut context);
        assert!(!items.contains(&"early_item"));
        assert!(items.contains(&"late_item"));
    }

    #[test]
    fn test_unique_items() {
        let mut table: LootTable<&str> = LootTable::new().with_drops(3, 3);
        table.add(LootEntry::new("unique_sword", 1.0).unique().with_id(1));
        table.add(LootEntry::new("potion", 10.0));

        let mut rng = GameRng::from_seed(42);
        let mut context = LootContext::new(1);
        let items = table.roll(&mut rng, &mut context);
        let unique_count = items.iter().filter(|&&i| i == "unique_sword").count();
        assert!(unique_count <= 1);
    }

    #[test]
    fn test_condition_logic() {
        let cond = LootCondition::IfFlag("boss_killed".to_string());
        let ctx = LootContext::new(1).with_flag("boss_killed");
        assert!(cond.evaluate(&ctx));

        let ctx = LootContext::new(1);
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_condition_combination() {
        let cond = LootCondition::min_depth(3).and(LootCondition::if_flag("hard_mode"));
        let ctx = LootContext::new(5).with_flag("hard_mode");
        assert!(cond.evaluate(&ctx));

        let ctx = LootContext::new(5);
        assert!(!cond.evaluate(&ctx));

        let ctx = LootContext::new(1).with_flag("hard_mode");
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_nested_loot_table() {
        let mut nested = NestedLootTable::new();

        let mut weapon_table: LootTable<String> = LootTable::new();
        weapon_table.add(LootEntry::new("sword".to_string(), 1.0));
        weapon_table.add(LootEntry::new("bow".to_string(), 1.0));

        let mut armor_table: LootTable<String> = LootTable::new();
        armor_table.add(LootEntry::new("helmet".to_string(), 1.0));

        nested.add_table("weapons".to_string(), weapon_table);
        nested.add_table("armor".to_string(), armor_table);

        let mut rng = GameRng::from_seed(42);
        let mut context = LootContext::new(1);
        let result = nested.roll_table("weapons", &mut rng, &mut context);
        assert!(result.is_some());
        assert!(!result.unwrap().is_empty());
    }
}
