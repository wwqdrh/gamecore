// encounter: 遭遇/事件系统
//
// 支持房间遭遇分配、难度曲线、敌人配置生成
// 与 dungeon 和 loot 模块集成，根据房间类型和层数生成遭遇

use super::dungeon::{DungeonMap, RoomType};
use super::rng::GameRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncounterType {
    Combat,
    Elite,
    Boss,
    Treasure,
    Shop,
    Rest,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemySpawn {
    pub enemy_id: String,
    pub weight: f64,
    pub min_depth: usize,
    pub max_depth: usize,
    pub count: Range<usize>,
}

impl EnemySpawn {
    pub fn new(id: &str, weight: f64) -> Self {
        EnemySpawn {
            enemy_id: id.to_string(),
            weight,
            min_depth: 0,
            max_depth: usize::MAX,
            count: 1..2,
        }
    }

    pub fn with_depth(mut self, min: usize, max: usize) -> Self {
        self.min_depth = min;
        self.max_depth = max;
        self
    }

    pub fn with_count(mut self, min: usize, max: usize) -> Self {
        self.count = min..max;
        self
    }

    fn is_available(&self, depth: usize) -> bool {
        depth >= self.min_depth && depth <= self.max_depth
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Encounter {
    Combat {
        enemy_pool: Vec<EnemySpawn>,
        count: Range<usize>,
    },
    Elite {
        enemy: String,
    },
    Boss {
        boss_id: String,
    },
    Treasure {
        loot_table: String,
    },
    Shop {
        inventory: Vec<String>,
    },
    Event {
        event_id: String,
    },
    Rest {
        heal_percent: f64,
    },
    Custom {
        id: String,
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterEntry {
    pub encounter: Encounter,
    pub weight: f64,
    pub min_depth: usize,
    pub max_depth: usize,
    pub difficulty: f64,
}

impl EncounterEntry {
    pub fn new(encounter: Encounter, weight: f64) -> Self {
        EncounterEntry {
            encounter,
            weight,
            min_depth: 0,
            max_depth: usize::MAX,
            difficulty: 1.0,
        }
    }

    pub fn with_depth(mut self, min: usize, max: usize) -> Self {
        self.min_depth = min;
        self.max_depth = max;
        self
    }

    pub fn with_difficulty(mut self, difficulty: f64) -> Self {
        self.difficulty = difficulty;
        self
    }

    fn is_available(&self, depth: usize) -> bool {
        depth >= self.min_depth && depth <= self.max_depth
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterTable {
    pub entries: Vec<EncounterEntry>,
    pub difficulty_scale: f64,
}

impl EncounterTable {
    pub fn new() -> Self {
        EncounterTable {
            entries: Vec::new(),
            difficulty_scale: 0.1,
        }
    }

    pub fn add(&mut self, entry: EncounterEntry) {
        self.entries.push(entry);
    }

    pub fn roll(
        &self,
        rng: &mut GameRng,
        depth: usize,
    ) -> Option<Encounter> {
        let available: Vec<&EncounterEntry> = self
            .entries
            .iter()
            .filter(|e| e.is_available(depth))
            .collect();

        if available.is_empty() {
            return None;
        }

        let weights: Vec<f64> = available.iter().map(|e| e.weight).collect();
        let idx = rng.choose_weighted_idx(&weights)?;
        Some(available[idx].encounter.clone())
    }
}

impl Default for EncounterTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyCurve {
    pub base_difficulty: f64,
    pub per_depth: f64,
    pub per_depth_exponential: f64,
    pub max_difficulty: f64,
}

impl DifficultyCurve {
    pub fn new() -> Self {
        DifficultyCurve {
            base_difficulty: 1.0,
            per_depth: 0.5,
            per_depth_exponential: 1.05,
            max_difficulty: 20.0,
        }
    }

    pub fn difficulty_at(&self, depth: usize) -> f64 {
        let linear = self.base_difficulty + self.per_depth * depth as f64;
        let exponential = self.per_depth_exponential.powi(depth as i32);
        (linear * exponential).min(self.max_difficulty)
    }

    pub fn enemy_count_at(&self, depth: usize, base_count: usize) -> usize {
        let difficulty = self.difficulty_at(depth);
        let extra = (difficulty / 3.0).floor() as usize;
        base_count + extra
    }

    pub fn enemy_hp_multiplier(&self, depth: usize) -> f64 {
        1.0 + self.difficulty_at(depth) * 0.1
    }

    pub fn enemy_damage_multiplier(&self, depth: usize) -> f64 {
        1.0 + self.difficulty_at(depth) * 0.05
    }
}

impl Default for DifficultyCurve {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterSystem {
    tables: HashMap<EncounterType, EncounterTable>,
    pub difficulty_curve: DifficultyCurve,
}

impl EncounterSystem {
    pub fn new() -> Self {
        EncounterSystem {
            tables: HashMap::new(),
            difficulty_curve: DifficultyCurve::new(),
        }
    }

    pub fn add_table(&mut self, encounter_type: EncounterType, table: EncounterTable) {
        self.tables.insert(encounter_type, table);
    }

    pub fn generate_encounter(
        &self,
        rng: &mut GameRng,
        encounter_type: &EncounterType,
        depth: usize,
    ) -> Option<Encounter> {
        self.tables
            .get(encounter_type)
            .and_then(|table| table.roll(rng, depth))
    }

    pub fn generate_encounter_for_room_type(
        &self,
        rng: &mut GameRng,
        room_type: &RoomType,
        depth: usize,
    ) -> Option<Encounter> {
        let encounter_type = match room_type {
            RoomType::Combat => EncounterType::Combat,
            RoomType::Elite => EncounterType::Elite,
            RoomType::Boss => EncounterType::Boss,
            RoomType::Treasure => EncounterType::Treasure,
            RoomType::Shop => EncounterType::Shop,
            RoomType::Rest => EncounterType::Rest,
            RoomType::Start | RoomType::End => return None,
            RoomType::Custom(_) => {
                return self.generate_encounter(rng, &EncounterType::Event, depth);
            }
        };
        self.generate_encounter(rng, &encounter_type, depth)
    }

    pub fn assign_encounters(
        &self,
        rng: &mut GameRng,
        map: &DungeonMap,
        depth: usize,
    ) -> HashMap<usize, Encounter> {
        let mut encounters = HashMap::new();

        for room in &map.rooms {
            if let Some(encounter) =
                self.generate_encounter_for_room_type(rng, &room.room_type, depth)
            {
                encounters.insert(room.id, encounter);
            }
        }

        encounters
    }

    pub fn generate_enemy_group(
        &self,
        rng: &mut GameRng,
        enemy_pool: &[EnemySpawn],
        depth: usize,
    ) -> Vec<String> {
        let available: Vec<&EnemySpawn> = enemy_pool
            .iter()
            .filter(|e| e.is_available(depth))
            .collect();

        if available.is_empty() {
            return Vec::new();
        }

        let base_count = self.difficulty_curve.enemy_count_at(depth, 2);
        let mut enemies = Vec::new();

        for _ in 0..base_count {
            let weights: Vec<f64> = available.iter().map(|e| e.weight).collect();
            if let Some(idx) = rng.choose_weighted_idx(&weights) {
                let spawn = available[idx];
                let count = rng.next_range(spawn.count.start as i32, spawn.count.end as i32 + 1) as usize;
                for _ in 0..count {
                    enemies.push(spawn.enemy_id.clone());
                }
            }
        }

        enemies
    }
}

impl Default for EncounterSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rogue::dungeon::{DungeonConfig, DungeonGenerator, RoomCorridorGenerator};

    #[test]
    fn test_encounter_table_roll() {
        let mut table = EncounterTable::new();
        table.add(EncounterEntry::new(
            Encounter::Combat {
                enemy_pool: vec![EnemySpawn::new("goblin", 1.0)],
                count: 2..4,
            },
            3.0,
        ));
        table.add(EncounterEntry::new(
            Encounter::Rest { heal_percent: 0.3 },
            1.0,
        ));

        let mut rng = GameRng::from_seed(42);
        let encounter = table.roll(&mut rng, 1);
        assert!(encounter.is_some());
    }

    #[test]
    fn test_depth_restriction() {
        let mut table = EncounterTable::new();
        table.add(EncounterEntry::new(
            Encounter::Elite {
                enemy: "dragon".to_string(),
            },
            1.0,
        ).with_depth(5, 10));

        let mut rng = GameRng::from_seed(42);
        assert!(table.roll(&mut rng, 3).is_none());
        assert!(table.roll(&mut rng, 7).is_some());
    }

    #[test]
    fn test_difficulty_curve() {
        let curve = DifficultyCurve::new();
        let d1 = curve.difficulty_at(1);
        let d5 = curve.difficulty_at(5);
        let d10 = curve.difficulty_at(10);
        assert!(d1 < d5);
        assert!(d5 < d10);
    }

    #[test]
    fn test_difficulty_scaling() {
        let curve = DifficultyCurve::new();
        let hp_mult = curve.enemy_hp_multiplier(5);
        let dmg_mult = curve.enemy_damage_multiplier(5);
        assert!(hp_mult > 1.0);
        assert!(dmg_mult > 1.0);
    }

    #[test]
    fn test_encounter_system() {
        let mut sys = EncounterSystem::new();

        let mut combat_table = EncounterTable::new();
        combat_table.add(EncounterEntry::new(
            Encounter::Combat {
                enemy_pool: vec![
                    EnemySpawn::new("goblin", 3.0),
                    EnemySpawn::new("skeleton", 2.0),
                ],
                count: 2..5,
            },
            5.0,
        ));
        sys.add_table(EncounterType::Combat, combat_table);

        let mut rng = GameRng::from_seed(42);
        let encounter = sys.generate_encounter(&mut rng, &EncounterType::Combat, 1);
        assert!(encounter.is_some());
    }

    #[test]
    fn test_assign_encounters_to_map() {
        let mut sys = EncounterSystem::new();

        let mut combat_table = EncounterTable::new();
        combat_table.add(EncounterEntry::new(
            Encounter::Combat {
                enemy_pool: vec![EnemySpawn::new("goblin", 1.0)],
                count: 2..4,
            },
            5.0,
        ));
        sys.add_table(EncounterType::Combat, combat_table);

        let mut rng = GameRng::from_seed(42);
        let config = DungeonConfig {
            width: 40,
            height: 30,
            min_rooms: 4,
            max_rooms: 6,
            ..Default::default()
        };
        let map = RoomCorridorGenerator::new().generate(&mut rng, &config);

        let mut rng2 = GameRng::from_seed(123);
        let encounters = sys.assign_encounters(&mut rng2, &map, 1);
        assert!(!encounters.is_empty());
    }

    #[test]
    fn test_generate_enemy_group() {
        let sys = EncounterSystem::new();
        let pool = vec![
            EnemySpawn::new("goblin", 3.0).with_count(1, 2),
            EnemySpawn::new("orc", 1.0).with_count(1, 1),
        ];

        let mut rng = GameRng::from_seed(42);
        let enemies = sys.generate_enemy_group(&mut rng, &pool, 1);
        assert!(!enemies.is_empty());
    }
}
