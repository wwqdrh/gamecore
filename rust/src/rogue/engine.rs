// RogueEngine - 肉鸽引擎 Godot 单例
//
// 管理 RogueContext（种子/深度）和 EntityPool（实体模板池）
// 支持通过 JSON 初始化实体模板和卡堆配置
// 提供 generate_piles / generate_entity 等方法供 GDScript 调用

use godot::prelude::*;
use godot::builtin::VarDictionary;
use godot::classes::IRefCounted;
use gamealgo::{
    RogueContext, EntityPool, EntityTemplate, EntityType, StatScale,
    CardPileConfig, CardPileLayout, Card, EntityStats,
};

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct RogueEngine {
    context: Option<RogueContext>,
    pool: EntityPool,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for RogueEngine {
    fn init(base: Base<RefCounted>) -> Self {
        RogueEngine {
            context: None,
            pool: EntityPool::new(),
            base,
        }
    }
}

#[godot_api]
impl RogueEngine {
    #[func]
    fn init_with_seed(&mut self, seed: i64) {
        self.context = Some(RogueContext::new(seed as u64));
    }

    #[func]
    fn get_seed(&self) -> i64 {
        self.context.as_ref().map(|c| c.seed() as i64).unwrap_or(0)
    }

    #[func]
    fn get_depth(&self) -> i64 {
        self.context.as_ref().map(|c| c.depth() as i64).unwrap_or(0)
    }

    #[func]
    fn set_depth(&mut self, depth: i64) {
        if let Some(ref mut ctx) = self.context {
            ctx.set_depth(depth as usize);
        }
    }

    #[func]
    fn advance_depth(&mut self) {
        if let Some(ref mut ctx) = self.context {
            ctx.advance_depth();
        }
    }

    #[func]
    fn load_entities_from_json(&mut self, json: GString) -> bool {
        let json_str = json.to_string();
        let config: serde_json::Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let entities = match config.get("entities") {
            Some(v) => v,
            None => return false,
        };

        let entities_arr = match entities.as_array() {
            Some(a) => a,
            None => return false,
        };

        for entity_val in entities_arr {
            if let Some(template) = parse_entity_template(entity_val) {
                self.pool.register(template);
            }
        }

        true
    }

    #[func]
    fn generate_piles(&mut self, json: GString) -> Variant {
        let ctx = match self.context.as_mut() {
            Some(c) => c,
            None => return Variant::nil(),
        };

        let config = parse_card_pile_config(&json.to_string());
        let layout = ctx.generate_card_piles(&self.pool, &config);
        let gd_piles = pack_layout_to_gd(&layout);
        gd_piles.to_variant()
    }

    #[func]
    fn generate_entity(&mut self, template_id: GString) -> Variant {
        let ctx = match self.context.as_mut() {
            Some(c) => c,
            None => return Variant::nil(),
        };

        let id = template_id.to_string();
        match ctx.generate_entity(&self.pool, &id) {
            Some(stats) => pack_entity_to_gd(&stats).to_variant(),
            None => Variant::nil(),
        }
    }

    #[func]
    fn roll_entity(&mut self, type_name: GString) -> Variant {
        let ctx = match self.context.as_mut() {
            Some(c) => c,
            None => return Variant::nil(),
        };

        let et = EntityType::new(&type_name.to_string());
        match ctx.roll_entity(&self.pool, &et) {
            Some(stats) => pack_entity_to_gd(&stats).to_variant(),
            None => Variant::nil(),
        }
    }

    #[func]
    fn get_snapshot_json(&self) -> GString {
        match self.context.as_ref() {
            Some(ctx) => {
                let snapshot = ctx.snapshot();
                match serde_json::to_string(&snapshot) {
                    Ok(s) => GString::from(&s),
                    Err(_) => GString::new(),
                }
            }
            None => GString::new(),
        }
    }

    #[func]
    fn restore_from_json(&mut self, json: GString) -> bool {
        let snapshot: gamealgo::rogue::context::RogueSnapshot = match serde_json::from_str(&json.to_string()) {
            Ok(s) => s,
            Err(_) => return false,
        };
        self.context = Some(RogueContext::restore(snapshot));
        true
    }
}

fn parse_entity_template(val: &serde_json::Value) -> Option<EntityTemplate> {
    let id = val.get("id")?.as_str()?;
    let name = val.get("name")?.as_str().unwrap_or(id);
    let type_str = val.get("type")?.as_str()?;
    let entity_type = EntityType::new(type_str);

    let mut template = EntityTemplate::new(id, name, entity_type);

    if let Some(weight) = val.get("weight").and_then(|v| v.as_f64()) {
        template = template.with_weight(weight);
    }
    if let Some(min) = val.get("min_depth").and_then(|v| v.as_u64()) {
        let max = val.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(usize::MAX as u64) as usize;
        template = template.with_depth_range(min as usize, max);
    }

    if let Some(stats) = val.get("stats").and_then(|v| v.as_object()) {
        for (stat_name, stat_val) in stats {
            let scale = parse_stat_scale(stat_val)?;
            template = template.with_stat(stat_name, scale);
        }
    }

    if let Some(tags) = val.get("tags").and_then(|v| v.as_array()) {
        for tag in tags {
            if let Some(tag_str) = tag.as_str() {
                template = template.with_tag(tag_str);
            }
        }
    }

    Some(template)
}

fn parse_stat_scale(val: &serde_json::Value) -> Option<StatScale> {
    let scale_type = val.get("scale")?.as_str().unwrap_or("linear");

    match scale_type {
        "fixed" => {
            let value = val.get("value")?.as_f64()?;
            Some(StatScale::fixed(value))
        }
        "linear" => {
            let base = val.get("base")?.as_f64()?;
            let per_level = val.get("per_level")?.as_f64()?;
            let variance = val.get("variance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Some(StatScale::linear(base, per_level, variance))
        }
        "exponential" => {
            let base = val.get("base")?.as_f64()?;
            let growth = val.get("growth")?.as_f64()?;
            let variance = val.get("variance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Some(StatScale::exponential(base, growth, variance))
        }
        _ => None,
    }
}

fn parse_card_pile_config(json_str: &str) -> CardPileConfig {
    let val: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    let mut config = CardPileConfig::default();

    if let Some(count) = val.get("pile_count").and_then(|v| v.as_u64()) {
        config.pile_count = count as usize;
    }
    if let Some(min) = val.get("cards_per_pile_min").and_then(|v| v.as_u64()) {
        let max = val.get("cards_per_pile_max").and_then(|v| v.as_u64()).unwrap_or(min + 3);
        config.cards_per_pile = min as usize..max as usize;
    }
    if let Some(weights) = val.get("type_weights").and_then(|v| v.as_object()) {
        config.type_weights = weights
            .iter()
            .map(|(k, v)| (k.clone(), v.as_f64().unwrap_or(1.0)))
            .collect();
    }

    config
}

fn pack_layout_to_gd(layout: &CardPileLayout) -> VarDictionary {
    let mut dict = VarDictionary::new();

    dict.set("exit_pile_id", layout.exit_pile_id as i64);

    let mut piles_arr = Array::<VarDictionary>::new();
    for pile in &layout.piles {
        let mut pile_dict = VarDictionary::new();
        pile_dict.set("id", pile.id as i64);
        pile_dict.set("position_x", pile.position.0 as f64);
        pile_dict.set("position_y", pile.position.1 as f64);

        let mut cards_arr = Array::<VarDictionary>::new();
        for card in &pile.cards {
            cards_arr.push(&pack_card_to_gd(card));
        }
        pile_dict.set("cards", &cards_arr);
        piles_arr.push(&pile_dict);
    }
    dict.set("piles", &piles_arr);

    dict
}

fn pack_card_to_gd(card: &Card) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("id", card.id as i64);
    dict.set("face_up", card.face_up);
    dict.set("entity", &pack_entity_to_gd(&card.entity));
    dict
}

fn pack_entity_to_gd(entity: &EntityStats) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("template_id", entity.template_id.clone());
    dict.set("name", entity.name.clone());
    dict.set("type", entity.entity_type.as_str().to_string());
    dict.set("depth", entity.depth as i64);

    let mut stats_dict = VarDictionary::new();
    for (k, v) in &entity.stats {
        stats_dict.set(k.clone(), *v as f64);
    }
    dict.set("stats", &stats_dict);

    dict
}
