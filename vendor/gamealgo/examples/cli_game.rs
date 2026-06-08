// cli_game: 肉鸽卡牌游戏 CLI 示例
//
// 玩法：多堆牌，所有牌默认翻开，只能看到每堆顶牌
// 玩家选择牌堆拿取顶牌（战斗/获取道具），或拖动牌堆叠到其他堆
// 找到出口即通关

use gamealgo::*;
use std::io::{self, Write};

struct Player {
    hp: f64,
    max_hp: f64,
    atk: f64,
    def: f64,
    inventory: Vec<String>,
}

impl Player {
    fn new(hp: f64, atk: f64, def: f64) -> Self {
        Player {
            hp,
            max_hp: hp,
            atk,
            def,
            inventory: Vec::new(),
        }
    }

    fn is_alive(&self) -> bool {
        self.hp > 0.0
    }

    fn heal(&mut self, amount: f64) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    fn take_damage(&mut self, raw_damage: f64) -> f64 {
        let reduction = self.def / (self.def + 100.0);
        let actual = raw_damage * (1.0 - reduction);
        self.hp = (self.hp - actual).max(0.0);
        actual
    }

    fn add_weapon(&mut self, atk: f64, name: &str) {
        self.atk += atk;
        self.inventory.push(format!("武器: {} (atk+{})", name, atk as i32));
    }

    fn add_armor(&mut self, def: f64, name: &str) {
        self.def += def;
        self.inventory.push(format!("防具: {} (def+{})", name, def as i32));
    }
}

fn build_entity_pool() -> EntityPool {
    let mut pool = EntityPool::new();

    pool.register(
        EntityTemplate::new("goblin", "哥布林", EntityType::new(EntityType::MONSTER))
            .with_stat("hp", StatScale::linear(20.0, 8.0, 0.1))
            .with_stat("atk", StatScale::linear(5.0, 2.0, 0.1))
            .with_stat("def", StatScale::fixed(1.0))
            .with_weight(5.0),
    );
    pool.register(
        EntityTemplate::new("skeleton", "骷髅兵", EntityType::new(EntityType::MONSTER))
            .with_stat("hp", StatScale::linear(30.0, 10.0, 0.1))
            .with_stat("atk", StatScale::linear(8.0, 2.5, 0.1))
            .with_stat("def", StatScale::fixed(3.0))
            .with_weight(3.0)
            .with_depth_range(2, usize::MAX),
    );
    pool.register(
        EntityTemplate::new("orc", "兽人", EntityType::new(EntityType::MONSTER))
            .with_stat("hp", StatScale::linear(50.0, 12.0, 0.1))
            .with_stat("atk", StatScale::linear(12.0, 3.0, 0.1))
            .with_stat("def", StatScale::fixed(5.0))
            .with_weight(2.0)
            .with_depth_range(3, usize::MAX),
    );
    pool.register(
        EntityTemplate::new("dragon", "巨龙", EntityType::new(EntityType::MONSTER))
            .with_stat("hp", StatScale::exponential(80.0, 1.15, 0.05))
            .with_stat("atk", StatScale::exponential(15.0, 1.1, 0.05))
            .with_stat("def", StatScale::fixed(8.0))
            .with_tag("boss")
            .with_weight(0.5)
            .with_depth_range(5, usize::MAX),
    );

    pool.register(
        EntityTemplate::new("wooden_sword", "木剑", EntityType::new(EntityType::WEAPON))
            .with_stat("atk", StatScale::linear(3.0, 1.0, 0.1))
            .with_weight(4.0),
    );
    pool.register(
        EntityTemplate::new("iron_sword", "铁剑", EntityType::new(EntityType::WEAPON))
            .with_stat("atk", StatScale::linear(6.0, 1.5, 0.1))
            .with_weight(2.0)
            .with_depth_range(2, usize::MAX),
    );
    pool.register(
        EntityTemplate::new("magic_staff", "法杖", EntityType::new(EntityType::WEAPON))
            .with_stat("atk", StatScale::linear(10.0, 2.0, 0.1))
            .with_weight(1.0)
            .with_depth_range(4, usize::MAX),
    );

    pool.register(
        EntityTemplate::new("wooden_shield", "木盾", EntityType::new(EntityType::ARMOR))
            .with_stat("def", StatScale::linear(2.0, 0.8, 0.1))
            .with_weight(4.0),
    );
    pool.register(
        EntityTemplate::new("iron_shield", "铁盾", EntityType::new(EntityType::ARMOR))
            .with_stat("def", StatScale::linear(5.0, 1.2, 0.1))
            .with_weight(2.0)
            .with_depth_range(2, usize::MAX),
    );

    pool.register(
        EntityTemplate::new("heal_potion", "治疗药水", EntityType::new(EntityType::ITEM))
            .with_stat("heal", StatScale::linear(15.0, 5.0, 0.1))
            .with_weight(5.0),
    );
    pool.register(
        EntityTemplate::new("big_potion", "大治疗药水", EntityType::new(EntityType::ITEM))
            .with_stat("heal", StatScale::linear(30.0, 8.0, 0.1))
            .with_weight(2.0)
            .with_depth_range(3, usize::MAX),
    );

    pool
}

fn print_separator() {
    println!("{}", "─".repeat(60));
}

fn print_status(player: &Player, depth: usize) {
    println!(
        "  ❤️ HP: {}/{}  ⚔️ ATK: {}  🛡️ DEF: {}  📦 深度: {}",
        player.hp as i32,
        player.max_hp as i32,
        player.atk as i32,
        player.def as i32,
        depth
    );
    if !player.inventory.is_empty() {
        println!("  🎒 背包: {}", player.inventory.join(", "));
    }
}

fn entity_icon(entity_type: &EntityType) -> &'static str {
    match entity_type.as_str() {
        EntityType::MONSTER => "👹",
        EntityType::WEAPON => "🗡️",
        EntityType::ARMOR => "🛡️",
        EntityType::ITEM => "💊",
        EntityType::EXIT => "🚪",
        _ => "❓",
    }
}

fn format_entity_stats(entity: &EntityStats) -> String {
    let mut parts = Vec::new();
    if let Some(&v) = entity.stats.get("hp") {
        parts.push(format!("HP{}", v as i32));
    }
    if let Some(&v) = entity.stats.get("atk") {
        parts.push(format!("ATK{}", v as i32));
    }
    if let Some(&v) = entity.stats.get("def") {
        parts.push(format!("DEF{}", v as i32));
    }
    if let Some(&v) = entity.stats.get("heal") {
        parts.push(format!("回血{}", v as i32));
    }
    parts.join(" ")
}

fn print_piles(layout: &CardPileLayout) {
    println!();
    for pile in &layout.piles {
        if pile.is_empty() {
            println!(
                "  [{}] 牌堆 {}  (空)",
                pile.id + 1,
                pile.id + 1,
            );
        } else {
            let top = pile.top().unwrap();
            let icon = entity_icon(&top.entity.entity_type);
            let remaining = if pile.len() > 1 {
                format!(" (+{}张)", pile.len() - 1)
            } else {
                String::new()
            };
            println!(
                "  [{}] 牌堆 {}  {} {} {} {}",
                pile.id + 1,
                pile.id + 1,
                icon,
                top.entity.name,
                format_entity_stats(&top.entity),
                remaining,
            );
        }
    }
    println!();
}

fn combat(player: &mut Player, monster: &EntityStats) -> bool {
    let m_hp = monster.get("hp");
    let m_atk = monster.get("atk");
    let m_def = monster.get("def");

    println!("\n  ⚔️ 遭遇 {}！ HP:{} ATK:{} DEF:{}",
        monster.name, m_hp as i32, m_atk as i32, m_def as i32);

    let mut current_m_hp = m_hp;

    loop {
        let p_damage = DamageFormula::physical_damage(player.atk, m_def);
        current_m_hp -= p_damage;
        println!(
            "    你攻击造成 {} 伤害，怪物剩余 HP {}",
            p_damage as i32,
            current_m_hp.max(0.0) as i32
        );

        if current_m_hp <= 0.0 {
            println!("  ✅ 击败了 {}！", monster.name);
            return true;
        }

        let m_damage = DamageFormula::physical_damage(m_atk, player.def);
        let actual = player.take_damage(m_damage);
        println!(
            "    {} 攻击造成 {} 伤害，你剩余 HP {}",
            monster.name,
            actual as i32,
            player.hp as i32
        );

        if !player.is_alive() {
            println!("  💀 你被 {} 击败了...", monster.name);
            return false;
        }
    }
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn read_number(prompt: &str, min: i32, max: i32) -> i32 {
    loop {
        let input = read_input(prompt);
        match input.parse::<i32>() {
            Ok(n) if n >= min && n <= max => return n,
            _ => println!("  请输入 {}-{} 之间的数字", min, max),
        }
    }
}

fn play_round(ctx: &mut RogueContext, pool: &EntityPool, depth: usize) -> bool {
    let config = CardPileConfig {
        pile_count: 3 + (depth / 3).min(3),
        cards_per_pile: (3 + depth / 2)..(5 + depth / 2),
        type_weights: vec![
            (EntityType::MONSTER.to_string(), 5.0),
            (EntityType::WEAPON.to_string(), 2.0),
            (EntityType::ARMOR.to_string(), 1.5),
            (EntityType::ITEM.to_string(), 1.5),
        ],
        spacing: 200.0,
    };

    let mut layout = ctx.generate_card_piles(pool, &config);

    for pile in &mut layout.piles {
        pile.reveal_all();
    }

    let mut player = Player::new(100.0 + depth as f64 * 10.0, 10.0, 5.0);

    println!("\n{}", "═".repeat(60));
    println!("  🏰 第 {} 层", depth);
    println!("  选择牌堆拿取顶牌，找到出口 🚪 即可通关！");
    println!("  出口隐藏在某堆牌的下方，需要先拿走上方的牌");
    println!("{}", "═".repeat(60));

    loop {
        print_separator();
        print_status(&player, depth);
        print_piles(&layout);

        if !player.is_alive() {
            println!("\n  💀 游戏结束！你倒在了第 {} 层", depth);
            return false;
        }

        let all_empty = layout.piles.iter().all(|p| p.is_empty());
        if all_empty {
            println!("\n  所有牌堆已空，但未找到出口...");
            return false;
        }

        println!("  操作:");
        println!("    1-{}: 拿取对应牌堆的顶牌", layout.piles.len());
        println!("    m:   移动牌堆顶牌到其他牌堆");
        println!("    q:   退出游戏");

        let input = read_input("  > ");

        if input == "q" {
            println!("  退出游戏");
            return false;
        }

        if input == "m" {
            let from = read_number(
                &format!("  从哪堆移？(1-{}): ", layout.piles.len()),
                1,
                layout.piles.len() as i32,
            ) as usize - 1;

            let to = read_number(
                &format!("  移到哪堆？(1-{}): ", layout.piles.len()),
                1,
                layout.piles.len() as i32,
            ) as usize - 1;

            if from == to {
                println!("  不能移到同一堆");
                continue;
            }

            let pile = &layout.piles[from];
            if pile.is_empty() {
                println!("  牌堆 {} 是空的", from + 1);
                continue;
            }

            let card_id = pile.top().unwrap().id;
            if layout.move_card(card_id, from, to) {
                let moved_name = layout.find_pile(to).unwrap().top().unwrap().entity.name.clone();
                println!("  ✅ 将 {} 从牌堆 {} 移到牌堆 {}", moved_name, from + 1, to + 1);
            } else {
                println!("  ❌ 移动失败");
            }
            continue;
        }

        if let Ok(pile_idx) = input.parse::<usize>() {
            if pile_idx < 1 || pile_idx > layout.piles.len() {
                println!("  请输入 1-{}", layout.piles.len());
                continue;
            }

            let pile_idx = pile_idx - 1;
            let pile = match layout.find_pile_mut(pile_idx) {
                Some(p) => p,
                None => continue,
            };

            if pile.is_empty() {
                println!("  牌堆 {} 已空", pile_idx + 1);
                continue;
            }

            let card = pile.top().unwrap().clone();

            match card.entity.entity_type.as_str() {
                EntityType::MONSTER => {
                    let monster = card.entity.clone();
                    let won = combat(&mut player, &monster);
                    layout.find_pile_mut(pile_idx).unwrap().pop();
                    if !won {
                        return false;
                    }
                }
                EntityType::WEAPON => {
                    let atk = card.entity.get("atk");
                    let name = &card.entity.name;
                    player.add_weapon(atk, name);
                    println!("  🗡️ 获得 {}！ATK +{}", name, atk as i32);
                    layout.find_pile_mut(pile_idx).unwrap().pop();
                }
                EntityType::ARMOR => {
                    let def = card.entity.get("def");
                    let name = &card.entity.name;
                    player.add_armor(def, name);
                    println!("  🛡️ 获得 {}！DEF +{}", name, def as i32);
                    layout.find_pile_mut(pile_idx).unwrap().pop();
                }
                EntityType::ITEM => {
                    let heal = card.entity.get("heal");
                    let name = &card.entity.name;
                    player.heal(heal);
                    println!("  💊 使用 {}！恢复 {} HP (当前 HP {})", name, heal as i32, player.hp as i32);
                    layout.find_pile_mut(pile_idx).unwrap().pop();
                }
                EntityType::EXIT => {
                    println!("\n  🎉 找到出口！第 {} 层通关！", depth);
                    layout.find_pile_mut(pile_idx).unwrap().pop();
                    return true;
                }
                _ => {
                    println!("  ❓ 未知卡片类型");
                    layout.find_pile_mut(pile_idx).unwrap().pop();
                }
            }
        } else {
            println!("  无效输入");
        }
    }
}

fn main() {
    println!();
    println!("  ╔══════════════════════════════════════╗");
    println!("  ║     🃏 肉鸽卡牌冒险 🃏              ║");
    println!("  ║                                      ║");
    println!("  ║  选择牌堆拿取顶牌，战斗怪物         ║");
    println!("  ║  收集装备，找到出口通往下一层！     ║");
    println!("  ╚══════════════════════════════════════╝");
    println!();

    let seed_str = read_input("  输入种子（留空随机）: ");
    let seed = if seed_str.is_empty() {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    } else {
        seed_str.parse::<u64>().unwrap_or(42)
    };

    println!("  种子: {}", seed);

    let mut ctx = RogueContext::new(seed);
    let pool = build_entity_pool();

    let mut depth = 1;
    let mut max_depth = 0;

    loop {
        ctx.set_depth(depth);
        let passed = play_round(&mut ctx, &pool, depth);

        if !passed {
            break;
        }

        max_depth = depth;
        depth += 1;

        let choice = read_input("  继续下一层？(y/n): ");
        if choice != "y" && choice != "Y" {
            break;
        }
    }

    println!("\n{}", "═".repeat(60));
    if max_depth > 0 {
        println!("  🏆 通关层数: {}", max_depth);
    }
    println!("  种子: {}", seed);
    println!("  感谢游玩！");
    println!("{}", "═".repeat(60));
}
