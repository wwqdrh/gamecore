# 赵擎 —— 外门师兄（反派）：外门场院游走（AI_WANDER），倨傲记仇
extends XiuNpcBase


func _init() -> void:
	display_name = "赵擎"
	role_name = "赵擎"
	timeline_path = "res://example/demo/xiuxian/role/npc/rival/rival_timeline.txt"
	entry_stage = ""
	npc_cell = Vector2i(9, 12)
	ai_behavior = 1
	wander_radius = 80.0
	body_color = Color(0.85, 0.6, 0.2)
	remeet_flag = "met_rival"
	remeet_greeting = "又是你。滚开点，别挡着本少爷的道。"
