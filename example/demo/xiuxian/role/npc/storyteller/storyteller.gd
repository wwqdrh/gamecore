# 贾先生 —— 茶棚说书先生：棚下站桩（AI_IDLE），只负责对话
extends XiuNpcBase


func _init() -> void:
	display_name = "说书先生"
	role_name = "贾先生"
	timeline_path = "res://example/demo/xiuxian/role/npc/storyteller/story_timeline.txt"
	entry_stage = "story_talk"
	npc_cell = Vector2i(18, 8)
	ai_behavior = 0
	body_color = Color(0.65, 0.6, 0.8)
