# 玄机老人 —— 藏经阁扫地老人：阁侧站桩（AI_IDLE），疯言疯语藏着锋芒
extends XiuNpcBase


func _init() -> void:
	display_name = "玄机老人"
	role_name = "玄机老人"
	timeline_path = "res://example/demo/xiuxian/role/npc/sweep_elder/sweep_timeline.txt"
	entry_stage = ""
	npc_cell = Vector2i(26, 6)
	ai_behavior = 0  # AI_IDLE 站桩（扫帚不离手）
	body_color = Color(0.55, 0.5, 0.45)
	remeet_flag = "met_sweep"
	remeet_greeting = "扫地，扫地。心不净，地也扫不净哟。"
