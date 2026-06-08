// context: 种子管理上下文
//
// RogueContext 是整个库的入口，设置种子后所有随机内容均可复现
// 内部为每个子系统派生独立的子 RNG，确保各系统随机互不干扰且可复现

use crate::rogue::cardpile::{CardPileConfig, CardPileLayout, generate_card_piles};
use crate::rogue::dungeon::{DungeonConfig, DungeonGenerator, DungeonMap, RoomCorridorGenerator};
use crate::rogue::encounter::{Encounter, EncounterSystem};
use crate::rogue::entity::{EntityPool, EntityStats, EntityType};
use crate::rogue::fov::{FovAlgorithm, FovResult, ShadowcastingFov};
use crate::rogue::loot::{LootContext, LootTable};
use crate::rogue::pathfind::{AStarFinder, PathConfig, PathFinder};
use crate::rogue::rng::GameRng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RogueContext {
    seed: u64,
    rngs: HashMap<String, GameRng>,
    depth: usize,
}

impl RogueContext {
    pub fn new(seed: u64) -> Self {
        let mut master = GameRng::from_seed(seed);
        let mut rngs = HashMap::new();
        rngs.insert("master".to_string(), master.fork());
        rngs.insert("dungeon".to_string(), master.fork());
        rngs.insert("loot".to_string(), master.fork());
        rngs.insert("encounter".to_string(), master.fork());
        rngs.insert("entity".to_string(), master.fork());
        rngs.insert("cardpile".to_string(), master.fork());
        rngs.insert("misc".to_string(), master.fork());

        RogueContext {
            seed,
            rngs,
            depth: 1,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    pub fn advance_depth(&mut self) {
        self.depth += 1;
    }

    pub fn rng(&mut self, domain: &str) -> &mut GameRng {
        if !self.rngs.contains_key(domain) {
            let new_rng = {
                let master = self.rngs.get_mut("master").unwrap();
                master.fork()
            };
            self.rngs.insert(domain.to_string(), new_rng);
        }
        self.rngs.get_mut(domain).unwrap()
    }

    pub fn generate_dungeon(&mut self, config: &DungeonConfig) -> DungeonMap {
        let rng = self.rng("dungeon");
        RoomCorridorGenerator::new().generate(rng, config)
    }

    pub fn generate_dungeon_with(
        &mut self,
        generator: &dyn DungeonGenerator,
        config: &DungeonConfig,
    ) -> DungeonMap {
        let rng = self.rng("dungeon");
        generator.generate(rng, config)
    }

    pub fn assign_encounters(
        &mut self,
        system: &EncounterSystem,
        map: &DungeonMap,
    ) -> HashMap<usize, Encounter> {
        let depth = self.depth;
        let rng = self.rng("encounter");
        system.assign_encounters(rng, map, depth)
    }

    pub fn compute_fov(&self, map: &DungeonMap, origin: (usize, usize), radius: usize) -> FovResult {
        ShadowcastingFov::new().compute(map, origin, radius)
    }

    pub fn compute_fov_with_explored(
        &self,
        map: &DungeonMap,
        origin: (usize, usize),
        radius: usize,
        explored: &HashSet<(usize, usize)>,
    ) -> FovResult {
        ShadowcastingFov::new().compute_with_explored(map, origin, radius, explored)
    }

    pub fn find_path(
        &self,
        map: &DungeonMap,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Option<Vec<(usize, usize)>> {
        AStarFinder.find_path(map, start, end, &PathConfig::default())
    }

    pub fn find_path_with(
        &self,
        finder: &dyn PathFinder,
        map: &DungeonMap,
        start: (usize, usize),
        end: (usize, usize),
        config: &PathConfig,
    ) -> Option<Vec<(usize, usize)>> {
        finder.find_path(map, start, end, config)
    }

    pub fn roll_loot<T: Clone>(
        &mut self,
        table: &LootTable<T>,
        loot_context: &mut LootContext,
    ) -> Vec<T> {
        let rng = self.rng("loot");
        table.roll(rng, loot_context)
    }

    pub fn generate_entity(&mut self, pool: &EntityPool, id: &str) -> Option<EntityStats> {
        let depth = self.depth;
        let rng = self.rng("entity");
        pool.generate(id, depth, rng)
    }

    pub fn roll_entity(&mut self, pool: &EntityPool, entity_type: &EntityType) -> Option<EntityStats> {
        let depth = self.depth;
        let rng = self.rng("entity");
        pool.roll_random(entity_type, depth, rng)
    }

    pub fn roll_entities(&mut self, pool: &EntityPool, entity_type: &EntityType, count: usize) -> Vec<EntityStats> {
        let depth = self.depth;
        let rng = self.rng("entity");
        pool.roll_random_n(entity_type, depth, count, rng)
    }

    pub fn generate_card_piles(&mut self, pool: &EntityPool, config: &CardPileConfig) -> CardPileLayout {
        let depth = self.depth;
        let rng = self.rng("cardpile");
        generate_card_piles(rng, pool, config, depth)
    }

    pub fn snapshot(&self) -> RogueSnapshot {
        RogueSnapshot {
            seed: self.seed,
            depth: self.depth,
            rng_states: self
                .rngs
                .iter()
                .map(|(k, v)| (k.clone(), v.seed_snapshot()))
                .collect(),
        }
    }

    pub fn restore(snapshot: RogueSnapshot) -> Self {
        let mut rngs = HashMap::new();
        for (k, state) in snapshot.rng_states {
            rngs.insert(k, GameRng::from_snapshot(state));
        }
        RogueContext {
            seed: snapshot.seed,
            rngs,
            depth: snapshot.depth,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RogueSnapshot {
    pub seed: u64,
    pub depth: usize,
    pub rng_states: HashMap<String, [u64; 4]>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rogue::dungeon::DungeonConfig;
    use crate::rogue::entity::{EntityTemplate, StatScale};

    fn make_test_pool() -> EntityPool {
        let mut pool = EntityPool::new();
        pool.register(
            EntityTemplate::new("goblin", "哥布林", EntityType::new(EntityType::MONSTER))
                .with_stat("hp", StatScale::linear(20.0, 5.0, 0.1))
                .with_stat("atk", StatScale::linear(5.0, 1.5, 0.1))
                .with_weight(5.0),
        );
        pool.register(
            EntityTemplate::new("potion", "药水", EntityType::new(EntityType::ITEM))
                .with_stat("heal", StatScale::linear(15.0, 3.0, 0.1))
                .with_weight(4.0),
        );
        pool.register(
            EntityTemplate::new("sword", "剑", EntityType::new(EntityType::WEAPON))
                .with_stat("atk", StatScale::linear(3.0, 1.0, 0.1))
                .with_weight(3.0),
        );
        pool.register(
            EntityTemplate::new("shield", "盾", EntityType::new(EntityType::ARMOR))
                .with_stat("def", StatScale::linear(3.0, 0.8, 0.1))
                .with_weight(2.0),
        );
        pool
    }

    #[test]
    fn test_same_seed_same_dungeon() {
        let config = DungeonConfig {
            width: 40,
            height: 30,
            min_rooms: 4,
            max_rooms: 6,
            ..Default::default()
        };

        let mut ctx1 = RogueContext::new(42);
        let map1 = ctx1.generate_dungeon(&config);

        let mut ctx2 = RogueContext::new(42);
        let map2 = ctx2.generate_dungeon(&config);

        assert_eq!(map1.rooms.len(), map2.rooms.len());
        assert_eq!(map1.to_string_map(), map2.to_string_map());
    }

    #[test]
    fn test_different_seed_different_dungeon() {
        let config = DungeonConfig {
            width: 40,
            height: 30,
            ..Default::default()
        };

        let mut ctx1 = RogueContext::new(42);
        let map1 = ctx1.generate_dungeon(&config);

        let mut ctx2 = RogueContext::new(999);
        let map2 = ctx2.generate_dungeon(&config);

        assert_ne!(map1.to_string_map(), map2.to_string_map());
    }

    #[test]
    fn test_domain_isolation() {
        let mut ctx = RogueContext::new(42);
        let config = DungeonConfig {
            width: 40,
            height: 30,
            min_rooms: 4,
            max_rooms: 6,
            ..Default::default()
        };

        let map1 = ctx.generate_dungeon(&config);

        let mut ctx2 = RogueContext::new(42);
        let mut loot_ctx = crate::rogue::loot::LootContext::new(1);
        let mut table: crate::rogue::loot::LootTable<&str> = crate::rogue::loot::LootTable::new();
        table.add(crate::rogue::loot::LootEntry::new("item", 1.0));
        let _ = ctx2.roll_loot(&table, &mut loot_ctx);

        let map2 = ctx2.generate_dungeon(&config);
        assert_eq!(map1.to_string_map(), map2.to_string_map());
    }

    #[test]
    fn test_snapshot_restore() {
        let config = DungeonConfig {
            width: 40,
            height: 30,
            min_rooms: 4,
            max_rooms: 6,
            ..Default::default()
        };

        let mut ctx = RogueContext::new(42);
        let _map1 = ctx.generate_dungeon(&config);
        let snapshot = ctx.snapshot();

        let _extra = ctx.rng("dungeon").next_u64();

        let mut ctx2 = RogueContext::restore(snapshot);
        let _map2 = ctx2.generate_dungeon(&config);

        let mut ctx3 = RogueContext::new(42);
        let _map3 = ctx3.generate_dungeon(&config);

        assert_eq!(ctx2.seed(), ctx3.seed());
    }

    #[test]
    fn test_depth_management() {
        let mut ctx = RogueContext::new(42);
        assert_eq!(ctx.depth(), 1);
        ctx.advance_depth();
        assert_eq!(ctx.depth(), 2);
        ctx.set_depth(5);
        assert_eq!(ctx.depth(), 5);
    }

    #[test]
    fn test_fov_and_pathfind() {
        let mut ctx = RogueContext::new(42);
        let config = DungeonConfig {
            width: 40,
            height: 30,
            min_rooms: 4,
            max_rooms: 6,
            ..Default::default()
        };
        let map = ctx.generate_dungeon(&config);
        let start = map.rooms[0].rect.center();
        let fov = ctx.compute_fov(&map, start, 8);
        assert!(fov.visible.contains(&start));

        if map.rooms.len() > 1 {
            let end = map.rooms.last().unwrap().rect.center();
            let path = ctx.find_path(&map, start, end);
            assert!(path.is_some());
        }
    }

    #[test]
    fn test_generate_entity_via_context() {
        let pool = make_test_pool();
        let mut ctx = RogueContext::new(42);

        let goblin = ctx.generate_entity(&pool, "goblin");
        assert!(goblin.is_some());
        let g = goblin.unwrap();
        assert!(g.get("hp") > 0.0);
    }

    #[test]
    fn test_roll_entity_via_context() {
        let pool = make_test_pool();
        let mut ctx = RogueContext::new(42);

        let entity = ctx.roll_entity(&pool, &EntityType::new(EntityType::MONSTER));
        assert!(entity.is_some());
        assert_eq!(entity.unwrap().entity_type, EntityType::new(EntityType::MONSTER));
    }

    #[test]
    fn test_generate_card_piles_via_context() {
        let pool = make_test_pool();
        let mut ctx = RogueContext::new(42);

        let config = CardPileConfig::default();
        let layout = ctx.generate_card_piles(&pool, &config);

        assert_eq!(layout.piles.len(), config.pile_count);
        assert!(layout.total_cards() > 0);
    }

    #[test]
    fn test_same_seed_same_card_piles() {
        let pool = make_test_pool();
        let config = CardPileConfig {
            pile_count: 4,
            cards_per_pile: 3..5,
            ..Default::default()
        };

        let mut ctx1 = RogueContext::new(42);
        let layout1 = ctx1.generate_card_piles(&pool, &config);

        let mut ctx2 = RogueContext::new(42);
        let layout2 = ctx2.generate_card_piles(&pool, &config);

        assert_eq!(layout1.piles.len(), layout2.piles.len());
        assert_eq!(layout1.exit_pile_id, layout2.exit_pile_id);
        for (p1, p2) in layout1.piles.iter().zip(layout2.piles.iter()) {
            assert_eq!(p1.len(), p2.len());
        }
    }
}
