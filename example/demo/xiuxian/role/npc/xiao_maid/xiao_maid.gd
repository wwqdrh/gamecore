# 丫鬟小翠 —— 萧宅丫鬟：西厢晒药处游走（AI_WANDER），小半径不离宅
extends XiuNpcBase


func _init() -> void:
	display_name = "丫鬟小翠"
	role_name = "小翠"
	timeline_path = "res://example/demo/xiuxian/role/npc/xiao_maid/maid_timeline.txt"
	entry_stage = ""  # 置空：入口选段交给 timeline gate 优先级扫描（entry 直达无条件兜底段会绕过门控）
	npc_cell = Vector2i(6, 7)
	ai_behavior = 1
	wander_radius = 64.0  # 约2格，不离本处太远
	body_color = Color(0.5, 0.75, 0.7)
