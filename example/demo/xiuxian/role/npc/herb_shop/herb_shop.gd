# 白芷 —— 回春堂药铺掌柜：铺面前站桩（AI_IDLE），只负责对话
extends XiuNpcBase


func _init() -> void:
	display_name = "药铺掌柜"
	role_name = "白芷"
	timeline_path = "res://example/demo/xiuxian/role/npc/herb_shop/herb_timeline.txt"
	entry_stage = "herb_talk"
	npc_cell = Vector2i(5, 8)
	ai_behavior = 0
	body_color = Color(0.55, 0.8, 0.5)
