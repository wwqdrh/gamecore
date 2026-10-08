# 外门弟子 —— 巡逻 NPC（AI_PATROL）：沿坊市东街来回溜达
extends XiuNpcBase


func _init() -> void:
	display_name = "外门弟子"
	role_name = "外门弟子"
	timeline_path = "res://example/demo/xiuxian/role/disciple/disciple_timeline.txt"
	entry_stage = "disciple_talk"
	npc_cell = Vector2i(23, 7)
	ai_behavior = 2  # AI_PATROL 路径巡逻
	patrol_cells = [
		Vector2i(23, 7),   # 坊市东街北口
		Vector2i(25, 12),  # 街角
		Vector2i(20, 14),  # 南段
		Vector2i(19, 9),   # 回程中段
	]
	body_color = Color(0.55, 0.62, 0.9)
