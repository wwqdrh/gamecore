# suite: rogue - 肉鸽引擎测试
# 覆盖: 种子初始化、实体模板加载、按权重抽取可复现性、牌堆生成、快照/恢复
# 注意: RogueEngine 是 RefCounted，由引用计数释放，不要手动 free()
extends "res://test/test_case.gd"

const ENTITIES_JSON := """
{
  "entities": [
    { "id": "slime",   "name": "史莱姆", "type": "monster", "weight": 5,
      "stats": { "hp": { "scale": "fixed", "value": 30 } } },
    { "id": "bat",     "name": "蝙蝠",   "type": "monster", "weight": 3,
      "stats": { "hp": { "scale": "fixed", "value": 15 } } },
    { "id": "sword",   "name": "铁剑",   "type": "weapon",  "weight": 2,
      "stats": { "atk": { "scale": "fixed", "value": 10 } } }
  ]
}
"""

const PILES_JSON := """
{
  "pile_count": 4,
  "cards_per_pile_min": 2,
  "cards_per_pile_max": 3,
  "type_weights": { "monster": 6, "weapon": 2 }
}
"""


func _make_engine(seed: int) -> RogueEngine:
	var engine := RogueEngine.new()
	engine.init_with_seed(seed)
	assert_true(engine.load_entities_from_json(ENTITIES_JSON), "实体模板 JSON 应加载成功")
	return engine


func test_seed_and_depth() -> void:
	var engine := _make_engine(20260928)
	assert_eq(engine.get_seed(), 20260928, "get_seed 应返回初始化种子")
	# 深度从 1 开始（RogueContext::new 中 depth: 1）
	assert_eq(engine.get_depth(), 1, "初始深度应为 1")
	engine.advance_depth()
	assert_eq(engine.get_depth(), 2, "advance_depth 后深度应为 2")
	engine.set_depth(5)
	assert_eq(engine.get_depth(), 5, "set_depth 应生效")


func test_same_seed_same_result() -> void:
	# 种子可复现性: 相同种子 + 相同配置，抽取结果必须一致
	var a := _make_engine(42)
	var b := _make_engine(42)

	for i in 10:
		var ea: Dictionary = a.generate_entity("slime")
		var eb: Dictionary = b.generate_entity("slime")
		assert_eq(ea, eb, "第 %d 次生成: 相同种子结果应一致" % i)


func test_generate_entity_stats() -> void:
	var engine := _make_engine(7)
	var entity: Dictionary = engine.generate_entity("slime")
	assert_not_null(entity, "生成 slime 不应返回 null")
	assert_eq(String(entity.get("template_id", "")), "slime", "template_id 应为 slime")
	var stats: Dictionary = entity.get("stats", {})
	assert_true(stats.has("hp"), "slime 的 stats 应包含 hp")
	assert_near(float(stats.get("hp", 0.0)), 30.0, 0.001, "fixed 缩放 hp 应为 30")


func test_generate_piles_layout() -> void:
	var engine := _make_engine(1234)
	var layout: Dictionary = engine.generate_piles(PILES_JSON)
	assert_not_null(layout, "牌堆布局不应为 null")

	assert_true(layout.has("piles"), "布局应包含 piles")
	assert_true(layout.has("exit_pile_id"), "布局应包含 exit_pile_id")
	assert_eq(layout["piles"].size(), 4, "pile_count=4 应生成 4 个牌堆")

	# 每个牌堆应有卡牌；普通牌堆卡牌数在 [min, max]，出口牌堆会在头部插入 1 张出口卡，
	# 因此任意牌堆卡牌数区间为 [min, max+1]；卡牌结构: {id, face_up, entity}
	for pile in layout["piles"]:
		var cards: Array = pile["cards"]
		assert_between(cards.size(), 2, 4, "牌堆卡牌数应在 [2, 3+1出口卡]")
		for card in cards:
			var entity: Dictionary = card.get("entity", {})
			assert_not_null(entity.get("template_id"), "卡牌 entity 应有 template_id")

	# 出口牌堆的第一张卡应为 exit
	var exit_pile: Dictionary = layout["piles"][int(layout["exit_pile_id"])]
	var first_entity: Dictionary = exit_pile["cards"][0].get("entity", {})
	assert_eq(String(first_entity.get("template_id", "")), "exit", "出口牌堆首卡应为 exit")

	# exit_pile_id 应指向布局中的某个牌堆
	var found_exit := false
	for pile in layout["piles"]:
		if int(pile["id"]) == int(layout["exit_pile_id"]):
			found_exit = true
	assert_true(found_exit, "exit_pile_id 应指向布局中的某个牌堆")


func test_snapshot_restore() -> void:
	var a := _make_engine(999)
	a.set_depth(3)
	var snapshot: String = a.get_snapshot_json()
	assert_not_null(snapshot, "快照 JSON 不应为空")

	var b := _make_engine(0)
	assert_true(b.restore_from_json(snapshot), "快照应能成功恢复")
	assert_eq(b.get_depth(), 3, "恢复后深度应为 3")
	assert_eq(b.get_seed(), 999, "恢复后种子应为 999")

	# 恢复后随机流应与原引擎一致
	var ea: Dictionary = a.generate_entity("slime")
	var eb: Dictionary = b.generate_entity("slime")
	assert_eq(ea, eb, "恢复快照后随机结果应与原引擎一致")


func test_invalid_json_rejected() -> void:
	var engine := RogueEngine.new()
	engine.init_with_seed(1)
	assert_false(engine.load_entities_from_json("not a json"), "非法 JSON 应返回 false")
	assert_false(engine.load_entities_from_json("{\"no_entities\": 1}"), "缺少 entities 字段应返回 false")
