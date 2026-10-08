# 执事长老 —— 站桩 NPC（AI_IDLE）：执事堂前不动，只负责对话
extends XiuNpcBase


func _init() -> void:
	display_name = "执事长老"
	role_name = "执事长老"
	timeline_path = "res://example/demo/xiuxian/role/npc/elder/elder_timeline.txt"
	entry_stage = "elder_talk"
	npc_cell = Vector2i(10, 9)
	ai_behavior = 0  # AI_IDLE 站桩
	body_color = Color(0.85, 0.68, 0.25)
