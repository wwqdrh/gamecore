// cardpile: 卡堆生成系统
//
// 管理卡堆的生成、内容分配、出口隐藏
// 卡堆数量不固定，每堆牌的内容由 EntityPool 过程化生成
// 类型权重可自定义扩展，不限于固定的怪物/武器/防具/道具

use crate::rogue::entity::{EntityPool, EntityStats, EntityType};
use crate::rogue::rng::GameRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: usize,
    pub entity: EntityStats,
    pub face_up: bool,
}

impl Card {
    pub fn new(id: usize, entity: EntityStats) -> Self {
        Card {
            id,
            entity,
            face_up: false,
        }
    }

    pub fn is_type(&self, type_name: &str) -> bool {
        self.entity.entity_type.as_str() == type_name
    }

    pub fn is_monster(&self) -> bool {
        self.is_type(EntityType::MONSTER)
    }

    pub fn is_weapon(&self) -> bool {
        self.is_type(EntityType::WEAPON)
    }

    pub fn is_armor(&self) -> bool {
        self.is_type(EntityType::ARMOR)
    }

    pub fn is_item(&self) -> bool {
        self.is_type(EntityType::ITEM)
    }

    pub fn is_exit(&self) -> bool {
        self.is_type(EntityType::EXIT)
    }

    pub fn reveal(&mut self) {
        self.face_up = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardPile {
    pub id: usize,
    pub cards: Vec<Card>,
    pub position: (f64, f64),
}

impl CardPile {
    pub fn new(id: usize, position: (f64, f64)) -> Self {
        CardPile {
            id,
            cards: Vec::new(),
            position,
        }
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn top(&self) -> Option<&Card> {
        self.cards.last()
    }

    pub fn top_mut(&mut self) -> Option<&mut Card> {
        self.cards.last_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn has_exit(&self) -> bool {
        self.cards.iter().any(|c| c.is_exit())
    }

    pub fn reveal_top(&mut self) {
        if let Some(card) = self.cards.last_mut() {
            card.face_up = true;
        }
    }

    pub fn reveal_all(&mut self) {
        for card in &mut self.cards {
            card.face_up = true;
        }
    }

    pub fn remove_card(&mut self, card_id: usize) -> Option<Card> {
        let pos = self.cards.iter().position(|c| c.id == card_id)?;
        Some(self.cards.remove(pos))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardPileConfig {
    pub pile_count: usize,
    pub cards_per_pile: std::ops::Range<usize>,
    pub type_weights: Vec<(String, f64)>,
    pub spacing: f64,
}

impl Default for CardPileConfig {
    fn default() -> Self {
        CardPileConfig {
            pile_count: 3,
            cards_per_pile: 3..6,
            type_weights: vec![
                (EntityType::MONSTER.to_string(), 5.0),
                (EntityType::WEAPON.to_string(), 2.0),
                (EntityType::ARMOR.to_string(), 2.0),
                (EntityType::ITEM.to_string(), 2.0),
            ],
            spacing: 200.0,
        }
    }
}

impl CardPileConfig {
    pub fn with_type_weight(mut self, type_name: &str, weight: f64) -> Self {
        if let Some(entry) = self.type_weights.iter_mut().find(|(t, _)| t == type_name) {
            entry.1 = weight;
        } else {
            self.type_weights.push((type_name.to_string(), weight));
        }
        self
    }

    pub fn get_type_weight(&self, type_name: &str) -> f64 {
        self.type_weights
            .iter()
            .find(|(t, _)| t == type_name)
            .map(|(_, w)| *w)
            .unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardPileLayout {
    pub piles: Vec<CardPile>,
    pub exit_pile_id: usize,
}

impl CardPileLayout {
    pub fn find_pile(&self, pile_id: usize) -> Option<&CardPile> {
        self.piles.iter().find(|p| p.id == pile_id)
    }

    pub fn find_pile_mut(&mut self, pile_id: usize) -> Option<&mut CardPile> {
        self.piles.iter_mut().find(|p| p.id == pile_id)
    }

    pub fn total_cards(&self) -> usize {
        self.piles.iter().map(|p| p.len()).sum()
    }

    pub fn move_card(&mut self, card_id: usize, from_pile: usize, to_pile: usize) -> bool {
        if from_pile == to_pile {
            return false;
        }

        let card = {
            let from = match self.find_pile_mut(from_pile) {
                Some(p) => p,
                None => return false,
            };
            match from.remove_card(card_id) {
                Some(c) => c,
                None => return false,
            }
        };

        if let Some(to) = self.find_pile_mut(to_pile) {
            to.push(card);
            true
        } else {
            false
        }
    }
}

pub fn generate_card_piles(
    rng: &mut GameRng,
    pool: &EntityPool,
    config: &CardPileConfig,
    depth: usize,
) -> CardPileLayout {
    let mut piles = Vec::new();
    let mut card_id_counter = 0usize;

    let exit_pile_idx = rng.next_range(0, config.pile_count as i32) as usize;

    for i in 0..config.pile_count {
        let x = i as f64 * config.spacing;
        let y = 0.0;
        let mut pile = CardPile::new(i, (x, y));

        let card_count = rng.next_range(
            config.cards_per_pile.start as i32,
            config.cards_per_pile.end as i32 + 1,
        ) as usize;

        for _ in 0..card_count {
            let entity = roll_card_entity(rng, pool, config, depth);
            let card = Card::new(card_id_counter, entity);
            card_id_counter += 1;
            pile.push(card);
        }

        piles.push(pile);
    }

    if let Some(exit_pile) = piles.get_mut(exit_pile_idx) {
        let exit_template = pool.get("exit");
        let exit_entity = if let Some(t) = exit_template {
            t.generate(depth, rng)
        } else {
            EntityStats {
                template_id: "exit".to_string(),
                name: "出口".to_string(),
                entity_type: EntityType::new(EntityType::EXIT),
                stats: Default::default(),
                depth,
            }
        };

        let exit_card = Card::new(card_id_counter, exit_entity);
        exit_pile.cards.insert(0, exit_card);
    }

    CardPileLayout {
        piles,
        exit_pile_id: exit_pile_idx,
    }
}

fn roll_card_entity(
    rng: &mut GameRng,
    pool: &EntityPool,
    config: &CardPileConfig,
    depth: usize,
) -> EntityStats {
    if config.type_weights.is_empty() {
        return EntityStats {
            template_id: "fallback".to_string(),
            name: "空牌".to_string(),
            entity_type: EntityType::new("none"),
            stats: Default::default(),
            depth,
        };
    }

    let weights: Vec<f64> = config.type_weights.iter().map(|(_, w)| *w).collect();
    let chosen_idx = rng.choose_weighted_idx(&weights).unwrap_or(0);
    let chosen_type = EntityType::new(&config.type_weights[chosen_idx].0);

    if let Some(entity) = pool.roll_random(&chosen_type, depth, rng) {
        return entity;
    }

    let mut fallback_order: Vec<(usize, f64)> = config
        .type_weights
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != chosen_idx)
        .map(|(i, (_, w))| (i, *w))
        .collect();
    fallback_order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (idx, _) in &fallback_order {
        let ft = EntityType::new(&config.type_weights[*idx].0);
        if let Some(entity) = pool.roll_random(&ft, depth, rng) {
            return entity;
        }
    }

    EntityStats {
        template_id: "fallback".to_string(),
        name: "空牌".to_string(),
        entity_type: EntityType::new(EntityType::ITEM),
        stats: Default::default(),
        depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            EntityTemplate::new("sword", "剑", EntityType::new(EntityType::WEAPON))
                .with_stat("atk", StatScale::linear(3.0, 1.0, 0.1))
                .with_weight(3.0),
        );

        pool.register(
            EntityTemplate::new("shield", "盾", EntityType::new(EntityType::ARMOR))
                .with_stat("def", StatScale::linear(3.0, 0.8, 0.1))
                .with_weight(2.0),
        );

        pool.register(
            EntityTemplate::new("potion", "药水", EntityType::new(EntityType::ITEM))
                .with_stat("heal", StatScale::linear(15.0, 3.0, 0.1))
                .with_weight(4.0),
        );

        pool
    }

    #[test]
    fn test_generate_card_piles() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);
        let config = CardPileConfig::default();
        let layout = generate_card_piles(&mut rng, &pool, &config, 1);

        assert_eq!(layout.piles.len(), config.pile_count);
        assert!(layout.total_cards() > 0);
    }

    #[test]
    fn test_exit_at_bottom() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);
        let config = CardPileConfig::default();
        let layout = generate_card_piles(&mut rng, &pool, &config, 1);

        let exit_pile = layout.find_pile(layout.exit_pile_id).unwrap();
        assert!(exit_pile.has_exit());

        let bottom_card = &exit_pile.cards[0];
        assert!(bottom_card.is_exit());
    }

    #[test]
    fn test_custom_pile_count() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);
        let config = CardPileConfig {
            pile_count: 5,
            cards_per_pile: 2..4,
            ..Default::default()
        };
        let layout = generate_card_piles(&mut rng, &pool, &config, 1);

        assert_eq!(layout.piles.len(), 5);
        for pile in &layout.piles {
            assert!(pile.len() >= 2);
        }
    }

    #[test]
    fn test_custom_type_weights() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);
        let config = CardPileConfig {
            type_weights: vec![
                ("monster".to_string(), 8.0),
                ("item".to_string(), 2.0),
            ],
            ..Default::default()
        };
        let layout = generate_card_piles(&mut rng, &pool, &config, 1);
        assert!(layout.total_cards() > 0);
    }

    #[test]
    fn test_with_type_weight() {
        let config = CardPileConfig::default()
            .with_type_weight("monster", 10.0)
            .with_type_weight("trap", 3.0);

        assert_eq!(config.get_type_weight("monster"), 10.0);
        assert_eq!(config.get_type_weight("trap"), 3.0);
        assert_eq!(config.get_type_weight("nonexistent"), 0.0);
    }

    #[test]
    fn test_card_pile_operations() {
        let mut pile = CardPile::new(0, (0.0, 0.0));
        let entity = EntityStats {
            template_id: "test".to_string(),
            name: "Test".to_string(),
            entity_type: EntityType::new(EntityType::MONSTER),
            stats: Default::default(),
            depth: 1,
        };

        pile.push(Card::new(0, entity));
        assert_eq!(pile.len(), 1);
        assert!(!pile.is_empty());

        let top = pile.top().unwrap();
        assert_eq!(top.id, 0);
        assert!(!top.face_up);

        pile.reveal_top();
        assert!(pile.top().unwrap().face_up);
    }

    #[test]
    fn test_move_card_between_piles() {
        let pool = make_test_pool();
        let mut rng = GameRng::from_seed(42);
        let config = CardPileConfig {
            pile_count: 2,
            cards_per_pile: 2..3,
            ..Default::default()
        };
        let mut layout = generate_card_piles(&mut rng, &pool, &config, 1);

        let from_len = layout.find_pile(0).unwrap().len();
        let to_len = layout.find_pile(1).unwrap().len();

        if from_len > 0 {
            let card_id = layout.find_pile(0).unwrap().cards[0].id;
            let ok = layout.move_card(card_id, 0, 1);
            assert!(ok);
            assert_eq!(layout.find_pile(0).unwrap().len(), from_len - 1);
            assert_eq!(layout.find_pile(1).unwrap().len(), to_len + 1);
        }
    }

    #[test]
    fn test_same_seed_same_layout() {
        let pool = make_test_pool();
        let config = CardPileConfig {
            pile_count: 3,
            cards_per_pile: 3..5,
            ..Default::default()
        };

        let mut rng1 = GameRng::from_seed(42);
        let layout1 = generate_card_piles(&mut rng1, &pool, &config, 1);

        let mut rng2 = GameRng::from_seed(42);
        let layout2 = generate_card_piles(&mut rng2, &pool, &config, 1);

        assert_eq!(layout1.piles.len(), layout2.piles.len());
        assert_eq!(layout1.exit_pile_id, layout2.exit_pile_id);
        for (p1, p2) in layout1.piles.iter().zip(layout2.piles.iter()) {
            assert_eq!(p1.len(), p2.len());
        }
    }

    #[test]
    fn test_card_type_checks() {
        let monster_card = Card::new(0, EntityStats {
            template_id: "goblin".to_string(),
            name: "哥布林".to_string(),
            entity_type: EntityType::new(EntityType::MONSTER),
            stats: Default::default(),
            depth: 1,
        });
        assert!(monster_card.is_monster());
        assert!(!monster_card.is_weapon());
        assert!(monster_card.is_type("monster"));

        let exit_card = Card::new(1, EntityStats {
            template_id: "exit".to_string(),
            name: "出口".to_string(),
            entity_type: EntityType::new(EntityType::EXIT),
            stats: Default::default(),
            depth: 1,
        });
        assert!(exit_card.is_exit());
    }

    #[test]
    fn test_custom_entity_type() {
        let trap_card = Card::new(2, EntityStats {
            template_id: "spike_trap".to_string(),
            name: "尖刺陷阱".to_string(),
            entity_type: EntityType::new("trap"),
            stats: Default::default(),
            depth: 1,
        });
        assert!(trap_card.is_type("trap"));
        assert!(!trap_card.is_monster());
    }
}
