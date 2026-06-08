// progression: 局外成长/遗物系统
//
// 支持遗物效果注册、协同效果、互斥检查、解锁条件
// 与 stats 模块集成，遗物效果可直接产生 Modifier

use super::stats::{Modifier, ModifierOp};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelicTier {
    Common,
    Uncommon,
    Rare,
    Legendary,
    Boss,
}

impl RelicTier {
    pub fn weight(&self) -> f64 {
        match self {
            RelicTier::Common => 5.0,
            RelicTier::Uncommon => 3.0,
            RelicTier::Rare => 1.5,
            RelicTier::Legendary => 0.5,
            RelicTier::Boss => 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelicEffect {
    StatModifier {
        stat: String,
        operation: ModifierOp,
        value: f64,
    },
    OnEvent {
        event: String,
        action: String,
    },
    Passive {
        id: String,
        data: serde_json::Value,
    },
}

impl RelicEffect {
    pub fn stat_modifier(stat: &str, op: ModifierOp, value: f64) -> Self {
        RelicEffect::StatModifier {
            stat: stat.to_string(),
            operation: op,
            value,
        }
    }

    pub fn on_event(event: &str, action: &str) -> Self {
        RelicEffect::OnEvent {
            event: event.to_string(),
            action: action.to_string(),
        }
    }

    pub fn passive(id: &str, data: serde_json::Value) -> Self {
        RelicEffect::Passive {
            id: id.to_string(),
            data,
        }
    }

    pub fn to_modifier(&self, relic_id: &str) -> Option<Modifier> {
        match self {
            RelicEffect::StatModifier {
                stat,
                operation,
                value,
            } => Some(Modifier::relic(relic_id, stat, *operation, *value)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tier: RelicTier,
    pub effects: Vec<RelicEffect>,
    pub prerequisites: Vec<String>,
    pub conflicts: Vec<String>,
}

impl RelicDef {
    pub fn new(id: &str, name: &str, tier: RelicTier) -> Self {
        RelicDef {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            tier,
            effects: Vec::new(),
            prerequisites: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_effect(mut self, effect: RelicEffect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn with_prerequisite(mut self, relic_id: &str) -> Self {
        self.prerequisites.push(relic_id.to_string());
        self
    }

    pub fn with_conflict(mut self, relic_id: &str) -> Self {
        self.conflicts.push(relic_id.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synergy {
    pub id: String,
    pub name: String,
    pub relic_ids: Vec<String>,
    pub bonus_effects: Vec<RelicEffect>,
}

impl Synergy {
    pub fn new(id: &str, name: &str, relic_ids: Vec<String>, bonus_effects: Vec<RelicEffect>) -> Self {
        Synergy {
            id: id.to_string(),
            name: name.to_string(),
            relic_ids,
            bonus_effects,
        }
    }

    pub fn is_active(&self, owned: &[String]) -> bool {
        self.relic_ids.iter().all(|id| owned.contains(id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyGraph {
    synergies: Vec<Synergy>,
}

impl SynergyGraph {
    pub fn new() -> Self {
        SynergyGraph {
            synergies: Vec::new(),
        }
    }

    pub fn add(&mut self, synergy: Synergy) {
        self.synergies.push(synergy);
    }

    pub fn active_synergies(&self, owned: &[String]) -> Vec<&Synergy> {
        self.synergies.iter().filter(|s| s.is_active(owned)).collect()
    }

    pub fn potential_synergies(&self, owned: &[String]) -> Vec<(&Synergy, Vec<String>)> {
        self.synergies
            .iter()
            .filter(|s| !s.is_active(owned))
            .filter_map(|s| {
                let missing: Vec<String> = s
                    .relic_ids
                    .iter()
                    .filter(|id| !owned.contains(id))
                    .cloned()
                    .collect();
                if missing.len() == 1 {
                    Some((s, missing))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for SynergyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionSystem {
    relics: HashMap<String, RelicDef>,
    unlocks: HashSet<String>,
    synergy_graph: SynergyGraph,
    max_relics: usize,
}

impl ProgressionSystem {
    pub fn new() -> Self {
        ProgressionSystem {
            relics: HashMap::new(),
            unlocks: HashSet::new(),
            synergy_graph: SynergyGraph::new(),
            max_relics: 20,
        }
    }

    pub fn with_max_relics(mut self, max: usize) -> Self {
        self.max_relics = max;
        self
    }

    pub fn register_relic(&mut self, relic: RelicDef) {
        self.relics.insert(relic.id.clone(), relic);
    }

    pub fn register_synergy(&mut self, synergy: Synergy) {
        self.synergy_graph.add(synergy);
    }

    pub fn unlock(&mut self, relic_id: &str) {
        self.unlocks.insert(relic_id.to_string());
    }

    pub fn is_unlocked(&self, relic_id: &str) -> bool {
        self.unlocks.contains(relic_id)
    }

    pub fn can_acquire(&self, owned: &[String], relic_id: &str) -> bool {
        let relic = match self.relics.get(relic_id) {
            Some(r) => r,
            None => return false,
        };

        if owned.len() >= self.max_relics {
            return false;
        }

        if owned.contains(&relic_id.to_string()) {
            return false;
        }

        for prereq in &relic.prerequisites {
            if !owned.contains(prereq) {
                return false;
            }
        }

        for conflict in &relic.conflicts {
            if owned.contains(conflict) {
                return false;
            }
        }

        true
    }

    pub fn acquire(
        &mut self,
        owned: &mut Vec<String>,
        relic_id: &str,
    ) -> Result<Vec<RelicEffect>, String> {
        if !self.can_acquire(owned, relic_id) {
            return Err(format!("Cannot acquire relic: {}", relic_id));
        }

        let relic = self.relics.get(relic_id).unwrap().clone();
        owned.push(relic_id.to_string());

        let mut all_effects = relic.effects.clone();

        let active_synergies = self.synergy_graph.active_synergies(owned);
        for synergy in active_synergies {
            all_effects.extend(synergy.bonus_effects.clone());
        }

        Ok(all_effects)
    }

    pub fn remove(owned: &mut Vec<String>, relic_id: &str) -> bool {
        if let Some(pos) = owned.iter().position(|id| id == relic_id) {
            owned.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_active_effects(&self, owned: &[String]) -> Vec<&RelicEffect> {
        let mut effects = Vec::new();

        for relic_id in owned {
            if let Some(relic) = self.relics.get(relic_id) {
                effects.extend(relic.effects.iter());
            }
        }

        let active_synergies = self.synergy_graph.active_synergies(owned);
        for synergy in active_synergies {
            effects.extend(synergy.bonus_effects.iter());
        }

        effects
    }

    pub fn get_stat_modifiers(&self, owned: &[String]) -> Vec<Modifier> {
        self.get_active_effects(owned)
            .iter()
            .filter_map(|effect| {
                if let Some(relic_id) = owned.first() {
                    effect.to_modifier(relic_id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn check_synergies(&self, owned: &[String]) -> Vec<&Synergy> {
        self.synergy_graph.active_synergies(owned)
    }

    pub fn get_available_relics(&self, owned: &[String]) -> Vec<&RelicDef> {
        self.relics
            .values()
            .filter(|r| self.can_acquire(owned, &r.id))
            .collect()
    }

    pub fn get_available_by_tier(&self, owned: &[String], tier: RelicTier) -> Vec<&RelicDef> {
        self.get_available_relics(owned)
            .into_iter()
            .filter(|r| r.tier == tier)
            .collect()
    }

    pub fn get_relic(&self, relic_id: &str) -> Option<&RelicDef> {
        self.relics.get(relic_id)
    }

    pub fn all_relics(&self) -> &HashMap<String, RelicDef> {
        &self.relics
    }

    pub fn synergy_graph(&self) -> &SynergyGraph {
        &self.synergy_graph
    }
}

impl Default for ProgressionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_system() -> ProgressionSystem {
        let mut sys = ProgressionSystem::new();

        sys.register_relic(
            RelicDef::new("fire_sword", "Fire Sword", RelicTier::Common)
                .with_effect(RelicEffect::stat_modifier("attack", ModifierOp::Add, 5.0)),
        );

        sys.register_relic(
            RelicDef::new("ice_shield", "Ice Shield", RelicTier::Uncommon)
                .with_effect(RelicEffect::stat_modifier("defense", ModifierOp::Add, 3.0)),
        );

        sys.register_relic(
            RelicDef::new("dragon_heart", "Dragon Heart", RelicTier::Rare)
                .with_prerequisite("fire_sword")
                .with_effect(RelicEffect::stat_modifier("hp", ModifierOp::Multiply, 0.2)),
        );

        sys.register_relic(
            RelicDef::new("shadow_dagger", "Shadow Dagger", RelicTier::Common)
                .with_conflict("fire_sword")
                .with_effect(RelicEffect::stat_modifier("attack", ModifierOp::Add, 3.0)),
        );

        sys.register_synergy(Synergy::new(
            "frost_fire",
            "Frost & Fire",
            vec!["fire_sword".to_string(), "ice_shield".to_string()],
            vec![RelicEffect::stat_modifier("attack", ModifierOp::Multiply, 0.1)],
        ));

        sys
    }

    #[test]
    fn test_can_acquire_basic() {
        let sys = make_test_system();
        let owned = vec![];
        assert!(sys.can_acquire(&owned, "fire_sword"));
        assert!(sys.can_acquire(&owned, "ice_shield"));
    }

    #[test]
    fn test_cannot_acquire_duplicate() {
        let sys = make_test_system();
        let owned = vec!["fire_sword".to_string()];
        assert!(!sys.can_acquire(&owned, "fire_sword"));
    }

    #[test]
    fn test_prerequisite() {
        let sys = make_test_system();
        let owned = vec![];
        assert!(!sys.can_acquire(&owned, "dragon_heart"));

        let owned = vec!["fire_sword".to_string()];
        assert!(sys.can_acquire(&owned, "dragon_heart"));
    }

    #[test]
    fn test_conflict() {
        let sys = make_test_system();
        let owned = vec!["fire_sword".to_string()];
        assert!(!sys.can_acquire(&owned, "shadow_dagger"));

        let owned = vec![];
        assert!(sys.can_acquire(&owned, "shadow_dagger"));
    }

    #[test]
    fn test_acquire_effects() {
        let mut sys = make_test_system();
        let mut owned = vec![];
        let effects = sys.acquire(&mut owned, "fire_sword").unwrap();
        assert_eq!(effects.len(), 1);
        assert!(owned.contains(&"fire_sword".to_string()));
    }

    #[test]
    fn test_synergy_activation() {
        let mut sys = make_test_system();
        let mut owned = vec![];

        sys.acquire(&mut owned, "fire_sword").unwrap();
        let synergies = sys.check_synergies(&owned);
        assert!(synergies.is_empty());

        sys.acquire(&mut owned, "ice_shield").unwrap();
        let synergies = sys.check_synergies(&owned);
        assert_eq!(synergies.len(), 1);
        assert_eq!(synergies[0].id, "frost_fire");
    }

    #[test]
    fn test_active_effects_with_synergy() {
        let mut sys = make_test_system();
        let mut owned = vec![];

        sys.acquire(&mut owned, "fire_sword").unwrap();
        sys.acquire(&mut owned, "ice_shield").unwrap();

        let effects = sys.get_active_effects(&owned);
        assert!(effects.len() >= 3);
    }

    #[test]
    fn test_remove_relic() {
        let mut owned = vec!["fire_sword".to_string()];
        assert!(ProgressionSystem::remove(&mut owned, "fire_sword"));
        assert!(owned.is_empty());
    }

    #[test]
    fn test_max_relics() {
        let mut sys = ProgressionSystem::new().with_max_relics(2);
        sys.register_relic(RelicDef::new("a", "A", RelicTier::Common));
        sys.register_relic(RelicDef::new("b", "B", RelicTier::Common));
        sys.register_relic(RelicDef::new("c", "C", RelicTier::Common));

        let mut owned = vec![];
        sys.acquire(&mut owned, "a").unwrap();
        sys.acquire(&mut owned, "b").unwrap();
        assert!(!sys.can_acquire(&owned, "c"));
    }

    #[test]
    fn test_potential_synergies() {
        let sys = make_test_system();
        let owned = vec!["fire_sword".to_string()];
        let potential = sys.synergy_graph().potential_synergies(&owned);
        assert_eq!(potential.len(), 1);
        assert_eq!(potential[0].1, vec!["ice_shield".to_string()]);
    }

    #[test]
    fn test_tier_weights() {
        assert!(RelicTier::Common.weight() > RelicTier::Rare.weight());
        assert!(RelicTier::Rare.weight() > RelicTier::Legendary.weight());
    }
}
