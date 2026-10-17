# 楚天阔 —— 内门大师兄：演武场游走（AI_WANDER），爽朗护短
extends XiuNpcBase


func _init() -> void:
	display_name = "楚天阔"
	role_name = "楚天阔"
	timeline_path = "res://example/demo/xiuxian/role/npc/senior_brother/senior_timeline.txt"
	entry_stage = ""
	npc_cell = Vector2i(22, 10)
	ai_behavior = 1  # AI_WANDER 演武场小范围走动
	wander_radius = 96.0
	body_color = Color(0.92, 0.92, 0.88)
	remeet_flag = "met_senior"
	remeet_greeting = "练过了？剑要日日磨，人也一样。"
