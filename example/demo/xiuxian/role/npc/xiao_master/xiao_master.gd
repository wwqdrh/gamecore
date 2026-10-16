# 萧远山（萧老爷）—— 萧宅家主：正房前站桩（AI_IDLE），只负责对话
extends XiuNpcBase


func _init() -> void:
	display_name = "萧老爷"
	role_name = "萧老爷"
	timeline_path = "res://example/demo/xiuxian/role/npc/xiao_master/master_timeline.txt"
	entry_stage = ""  # 置空：入口选段交给 timeline gate 优先级扫描（entry 直达无条件兜底段会绕过门控）
	npc_cell = Vector2i(9, 6)
	ai_behavior = 0
	body_color = Color(0.35, 0.42, 0.72)
