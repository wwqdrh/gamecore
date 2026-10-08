# 坊市商人 —— 游走 NPC（AI_WANDER）：在摊位附近随机溜达吆喝
extends XiuNpcBase


func _init() -> void:
	display_name = "坊市商人"
	role_name = "坊市商人"
	timeline_path = "res://example/demo/xiuxian/role/merchant/merchant_timeline.txt"
	entry_stage = "merchant_talk"
	npc_cell = Vector2i(17, 13)
	ai_behavior = 1  # AI_WANDER 随机游走
	wander_radius = 96.0  # 约三格，不离摊位太远
	body_color = Color(0.4, 0.75, 0.55)
