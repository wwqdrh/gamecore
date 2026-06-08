extends Control

var engine: RogueEngine
var piles_data: Array = []
var exit_pile_id: int = -1

var player_hp: float = 100.0
var player_max_hp: float = 100.0
var player_atk: float = 10.0
var player_def: float = 5.0
var player_inventory: Array = []
var current_depth: int = 1
var game_over: bool = false

const ENTITIES_JSON := """
{
  "entities": [
    {
      "id": "goblin", "name": "哥布林", "type": "monster", "weight": 5.0,
      "stats": { "hp": {"scale":"linear","base":20,"per_level":8,"variance":0.1}, "atk": {"scale":"linear","base":5,"per_level":2,"variance":0.1}, "def": {"scale":"fixed","value":1} }
    },
    {
      "id": "skeleton", "name": "骷髅兵", "type": "monster", "weight": 3.0, "min_depth": 2,
      "stats": { "hp": {"scale":"linear","base":30,"per_level":10,"variance":0.1}, "atk": {"scale":"linear","base":8,"per_level":2.5,"variance":0.1}, "def": {"scale":"fixed","value":3} }
    },
    {
      "id": "orc", "name": "兽人", "type": "monster", "weight": 2.0, "min_depth": 3,
      "stats": { "hp": {"scale":"linear","base":50,"per_level":12,"variance":0.1}, "atk": {"scale":"linear","base":12,"per_level":3,"variance":0.1}, "def": {"scale":"fixed","value":5} }
    },
    {
      "id": "wooden_sword", "name": "木剑", "type": "weapon", "weight": 4.0,
      "stats": { "atk": {"scale":"linear","base":3,"per_level":1,"variance":0.1} }
    },
    {
      "id": "iron_sword", "name": "铁剑", "type": "weapon", "weight": 2.0, "min_depth": 2,
      "stats": { "atk": {"scale":"linear","base":6,"per_level":1.5,"variance":0.1} }
    },
    {
      "id": "wooden_shield", "name": "木盾", "type": "armor", "weight": 4.0,
      "stats": { "def": {"scale":"linear","base":2,"per_level":0.8,"variance":0.1} }
    },
    {
      "id": "iron_shield", "name": "铁盾", "type": "armor", "weight": 2.0, "min_depth": 2,
      "stats": { "def": {"scale":"linear","base":5,"per_level":1.2,"variance":0.1} }
    },
    {
      "id": "heal_potion", "name": "治疗药水", "type": "item", "weight": 5.0,
      "stats": { "heal": {"scale":"linear","base":15,"per_level":5,"variance":0.1} }
    },
    {
      "id": "big_potion", "name": "大治疗药水", "type": "item", "weight": 2.0, "min_depth": 3,
      "stats": { "heal": {"scale":"linear","base":30,"per_level":8,"variance":0.1} }
    }
  ]
}
"""

func _ready() -> void:
	engine = RogueEngine.new()
	engine.init_with_seed(42)
	engine.load_entities_from_json(ENTITIES_JSON)
	_start_new_floor()

func _start_new_floor() -> void:
	game_over = false
	player_hp = player_max_hp
	player_atk = 10.0 + current_depth * 2.0
	player_def = 5.0 + current_depth
	player_inventory.clear()
	engine.set_depth(current_depth)

	var config := """{
		"pile_count": %d,
		"cards_per_pile_min": %d,
		"cards_per_pile_max": %d,
		"type_weights": { "monster": 5.0, "weapon": 2.0, "armor": 1.5, "item": 1.5 }
	}""" % [3 + current_depth / 3, 3 + current_depth / 2, 5 + current_depth / 2]

	var result = engine.generate_piles(config)
	piles_data = []
	if result and result.has("piles"):
		for pile_dict in result["piles"]:
			piles_data.append(pile_dict)
		exit_pile_id = int(result.get("exit_pile_id", -1))
	queue_redraw()

func _draw() -> void:
	var screen_size = get_viewport_rect().size
	var font = ThemeDB.fallback_font
	var font_size = 16

	draw_string(font, Vector2(20, 30), "🏰 肉鸽卡牌冒险 - 第 %d 层" % current_depth, HORIZONTAL_ALIGNMENT_LEFT, -1, 24, Color.WHITE)
	draw_string(font, Vector2(20, 60), "❤️ HP: %d/%d  ⚔️ ATK: %d  🛡️ DEF: %d" % [int(player_hp), int(player_max_hp), int(player_atk), int(player_def)], HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color.GREEN)
	if player_inventory.size() > 0:
		draw_string(font, Vector2(20, 85), "🎒 " + " ".join(player_inventory), HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color.LIGHT_GRAY)

	if game_over:
		draw_string(font, Vector2(screen_size.x / 2 - 80, screen_size.y / 2), "💀 游戏结束！按 R 重新开始", HORIZONTAL_ALIGNMENT_LEFT, -1, 24, Color.RED)
		return

	var pile_count = piles_data.size()
	if pile_count == 0:
		return

	var pile_width = 180.0
	var gap = 30.0
	var total_width = pile_count * pile_width + (pile_count - 1) * gap
	var start_x = (screen_size.x - total_width) / 2.0
	var start_y = 140.0

	for i in range(pile_count):
		var pile = piles_data[i]
		var x = start_x + i * (pile_width + gap)
		var cards = pile.get("cards", [])

		var pile_color = Color(0.2, 0.25, 0.35)
		if pile.get("id", -1) == exit_pile_id:
			pile_color = Color(0.25, 0.2, 0.35)
		draw_rect(Rect2(x, start_y, pile_width, 280), pile_color, true, 8.0)
		draw_rect(Rect2(x, start_y, pile_width, 280), Color(0.4, 0.45, 0.55), false, 2.0)

		var pile_label = "牌堆 %d (%d张)" % [i + 1, cards.size()]
		draw_string(font, Vector2(x + 10, start_y + 25), pile_label, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color.LIGHT_GRAY)

		if cards.size() > 0:
			var top_card = cards[cards.size() - 1]
			var entity = top_card.get("entity", {})
			var type_str = str(entity.get("type", ""))
			var name_str = str(entity.get("name", "???"))
			var stats = entity.get("stats", {})

			var icon = _type_icon(type_str)
			var card_color = _type_color(type_str)
			draw_rect(Rect2(x + 10, start_y + 40, pile_width - 20, 80), card_color, true, 6.0)

			draw_string(font, Vector2(x + 20, start_y + 65), "%s %s" % [icon, name_str], HORIZONTAL_ALIGNMENT_LEFT, -1, 18, Color.WHITE)

			var stat_parts = []
			for key in stats:
				stat_parts.append("%s%d" % [key.to_upper(), int(float(str(stats[key])))] )
			draw_string(font, Vector2(x + 20, start_y + 100), " ".join(stat_parts), HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color.WHITE)

			if cards.size() > 1:
				draw_string(font, Vector2(x + 10, start_y + 140), "还有 %d 张牌" % (cards.size() - 1), HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color.GRAY)
		else:
			draw_string(font, Vector2(x + 20, start_y + 80), "(空)", HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color.GRAY)

	draw_string(font, Vector2(20, screen_size.y - 60), "点击牌堆拿取顶牌 | M+数字移动牌 | R重新开始 | N下一层", HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color.GRAY)

func _input(event: InputEvent) -> void:
	if game_over:
		if event is InputEventKey and event.pressed and event.keycode == KEY_R:
			current_depth = 1
			engine.init_with_seed(42)
			engine.load_entities_from_json(ENTITIES_JSON)
			_start_new_floor()
		return

	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_R:
			current_depth = 1
			engine.init_with_seed(42)
			engine.load_entities_from_json(ENTITIES_JSON)
			_start_new_floor()
		elif event.keycode == KEY_N:
			current_depth += 1
			_start_new_floor()
		return

	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var click_pos = event.position
		var pile_count = piles_data.size()
		var pile_width = 180.0
		var gap = 30.0
		var total_width = pile_count * pile_width + (pile_count - 1) * gap
		var start_x = (get_viewport_rect().size.x - total_width) / 2.0
		var start_y = 140.0

		for i in range(pile_count):
			var x = start_x + i * (pile_width + gap)
			if click_pos.x >= x and click_pos.x <= x + pile_width and click_pos.y >= start_y and click_pos.y <= start_y + 280:
				_pick_pile(i)
				return

func _pick_pile(pile_idx: int) -> void:
	if pile_idx < 0 or pile_idx >= piles_data.size():
		return

	var pile = piles_data[pile_idx]
	var cards = pile.get("cards", [])
	if cards.size() == 0:
		return

	var top_card = cards[cards.size() - 1]
	var entity = top_card.get("entity", {})
	var type_str = str(entity.get("type", ""))
	var name_str = str(entity.get("name", ""))
	var stats = entity.get("stats", {})

	match type_str:
		"monster":
			_combat(entity)
		"weapon":
			var atk_val = float(str(stats.get("atk", 0)))
			player_atk += atk_val
			player_inventory.append("🗡️%s" % name_str)
		"armor":
			var def_val = float(str(stats.get("def", 0)))
			player_def += def_val
			player_inventory.append("🛡️%s" % name_str)
		"item":
			var heal_val = float(str(stats.get("heal", 0)))
			player_hp = min(player_hp + heal_val, player_max_hp)
		"exit":
			current_depth += 1
			_start_new_floor()
			return

	cards.erase(top_card)
	queue_redraw()

func _combat(monster: Dictionary) -> void:
	var m_hp = float(str(monster.get("stats", {}).get("hp", 0)))
	var m_atk = float(str(monster.get("stats", {}).get("atk", 0)))
	var m_def = float(str(monster.get("stats", {}).get("def", 0)))
	var m_name = str(monster.get("name", "怪物"))

	while m_hp > 0 and player_hp > 0:
		var p_dmg = player_atk * (1.0 - m_def / (m_def + 100.0))
		m_hp -= p_dmg
		if m_hp <= 0:
			break
		var m_dmg = m_atk * (1.0 - player_def / (player_def + 100.0))
		player_hp -= m_dmg

	if player_hp <= 0:
		player_hp = 0
		game_over = true

	queue_redraw()

func _type_icon(type_str: String) -> String:
	match type_str:
		"monster": return "👹"
		"weapon": return "🗡️"
		"armor": return "🛡️"
		"item": return "💊"
		"exit": return "🚪"
		_: return "❓"

func _type_color(type_str: String) -> Color:
	match type_str:
		"monster": return Color(0.6, 0.15, 0.15)
		"weapon": return Color(0.15, 0.3, 0.6)
		"armor": return Color(0.15, 0.5, 0.3)
		"item": return Color(0.5, 0.4, 0.1)
		"exit": return Color(0.1, 0.5, 0.5)
		_: return Color(0.3, 0.3, 0.3)
