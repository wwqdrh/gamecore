# 柳氏（萧夫人）—— 萧宅主母：东厢前站桩（AI_IDLE），只负责对话
extends XiuNpcBase


func _init() -> void:
	display_name = "萧夫人"
	role_name = "萧夫人"
	timeline_path = "res://example/demo/xiuxian/role/npc/xiao_lady/lady_timeline.txt"
	entry_stage = "lady_talk"
	npc_cell = Vector2i(12, 7)
	ai_behavior = 0
	body_color = Color(0.8, 0.55, 0.6)
