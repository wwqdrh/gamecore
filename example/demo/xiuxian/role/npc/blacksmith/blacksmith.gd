# 铁牛 —— 铁匠铺师傅：炉前站桩（AI_IDLE），只负责对话
extends XiuNpcBase


func _init() -> void:
	display_name = "铁匠铁牛"
	role_name = "铁牛"
	timeline_path = "res://example/demo/xiuxian/role/npc/blacksmith/smith_timeline.txt"
	entry_stage = "smith_talk"
	npc_cell = Vector2i(12, 8)
	ai_behavior = 0
	body_color = Color(0.7, 0.5, 0.35)
