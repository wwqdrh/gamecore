// RogueCardPile - 牌堆 Godot 包装类
//
// 将 gamealgo CardPile 数据暴露给 Godot
// 支持在 GDScript 中获取牌堆信息和顶牌

use godot::prelude::*;
use godot::builtin::VarDictionary;
use super::card::RogueCard;

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct RogueCardPile {
    pile_id: i64,
    cards: Array<Gd<RogueCard>>,
    has_exit: bool,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for RogueCardPile {
    fn init(base: Base<RefCounted>) -> Self {
        RogueCardPile {
            pile_id: 0,
            cards: Array::new(),
            has_exit: false,
            base,
        }
    }
}

#[godot_api]
impl RogueCardPile {
    #[func]
    fn get_pile_id(&self) -> i64 {
        self.pile_id
    }

    #[func]
    fn get_card_count(&self) -> i64 {
        self.cards.len() as i64
    }

    #[func]
    fn get_top_card(&self) -> Variant {
        match self.cards.back() {
            Some(card) => card.to_variant(),
            None => Variant::nil(),
        }
    }

    #[func]
    fn has_exit_card(&self) -> bool {
        self.has_exit
    }

    #[func]
    fn get_all_cards(&self) -> Array<Gd<RogueCard>> {
        self.cards.clone()
    }
}

impl RogueCardPile {
    pub fn from_dict(dict: &VarDictionary) -> Option<Gd<Self>> {
        let pile_id = dict.get("id").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0);

        let cards_dict_arr = dict.get("cards").and_then(|v| v.try_to::<Array::<VarDictionary>>().ok())?;
        let mut cards = Array::<Gd<RogueCard>>::new();
        let mut has_exit = false;

        for card_dict in cards_dict_arr.iter_shared() {
            if let Some(gd_card) = RogueCard::from_dict(&card_dict) {
                if gd_card.bind().entity_type.to_string() == "exit" {
                    has_exit = true;
                }
                cards.push(&gd_card);
            }
        }

        let pile = Gd::<Self>::from_init_fn(|base| RogueCardPile {
            pile_id,
            cards,
            has_exit,
            base,
        });

        Some(pile)
    }
}
