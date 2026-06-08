// RogueCard - 卡牌 Godot 包装类
//
// 将 gamealgo Card 数据暴露给 Godot
// 支持在 GDScript 中获取卡牌信息

use godot::prelude::*;
use godot::builtin::VarDictionary;

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct RogueCard {
    card_id: i64,
    template_id: GString,
    name: GString,
    pub(crate) entity_type: GString,
    stats: VarDictionary,
    face_up: bool,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for RogueCard {
    fn init(base: Base<RefCounted>) -> Self {
        RogueCard {
            card_id: 0,
            template_id: GString::new(),
            name: GString::new(),
            entity_type: GString::new(),
            stats: VarDictionary::new(),
            face_up: true,
            base,
        }
    }
}

#[godot_api]
impl RogueCard {
    #[func]
    fn get_card_id(&self) -> i64 {
        self.card_id
    }

    #[func]
    fn get_template_id(&self) -> GString {
        self.template_id.clone()
    }

    #[func]
    fn get_name(&self) -> GString {
        self.name.clone()
    }

    #[func]
    fn get_entity_type(&self) -> GString {
        self.entity_type.clone()
    }

    #[func]
    fn get_stats(&self) -> VarDictionary {
        self.stats.clone()
    }

    #[func]
    fn get_stat(&self, stat_name: GString) -> f64 {
        let s = stat_name.to_string();
        self.stats.get(s.as_str()).and_then(|v| v.try_to::<f64>().ok()).unwrap_or(0.0)
    }

    #[func]
    fn is_face_up(&self) -> bool {
        self.face_up
    }

    #[func]
    fn is_monster(&self) -> bool {
        self.entity_type.to_string() == "monster"
    }

    #[func]
    fn is_weapon(&self) -> bool {
        self.entity_type.to_string() == "weapon"
    }

    #[func]
    fn is_armor(&self) -> bool {
        self.entity_type.to_string() == "armor"
    }

    #[func]
    fn is_item(&self) -> bool {
        self.entity_type.to_string() == "item"
    }

    #[func]
    fn is_exit(&self) -> bool {
        self.entity_type.to_string() == "exit"
    }
}

impl RogueCard {
    pub fn from_dict(dict: &VarDictionary) -> Option<Gd<Self>> {
        let mut card = Gd::<Self>::from_init_fn(|base| RogueCard {
            card_id: 0,
            template_id: GString::new(),
            name: GString::new(),
            entity_type: GString::new(),
            stats: VarDictionary::new(),
            face_up: true,
            base,
        });

        card.bind_mut().card_id = dict.get("id").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0);
        card.bind_mut().face_up = dict.get("face_up").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false);

        if let Some(entity) = dict.get("entity").and_then(|v| v.try_to::<VarDictionary>().ok()) {
            card.bind_mut().template_id = entity.get("template_id").and_then(|v| v.try_to::<GString>().ok()).unwrap_or_default();
            card.bind_mut().name = entity.get("name").and_then(|v| v.try_to::<GString>().ok()).unwrap_or_default();
            card.bind_mut().entity_type = entity.get("type").and_then(|v| v.try_to::<GString>().ok()).unwrap_or_default();
            if let Some(stats) = entity.get("stats").and_then(|v| v.try_to::<VarDictionary>().ok()) {
                card.bind_mut().stats = stats;
            }
        }

        Some(card)
    }
}
