# 凌霄真人 —— 青云宗宗主：大殿前站桩（AI_IDLE），威严少言
extends XiuNpcBase


func _init() -> void:
	display_name = "凌霄真人"
	role_name = "凌霄真人"
	timeline_path = "res://example/demo/xiuxian/role/npc/sect_master/master_sect_timeline.txt"
	entry_stage = ""  # 置空：入口选段交给 timeline gate 优先级扫描
	npc_cell = Vector2i(16, 7)
	ai_behavior = 0  # AI_IDLE 站桩
	body_color = Color(0.3, 0.55, 0.85)
	remeet_flag = "met_sect_master"
	remeet_greeting = "外门弟子，晨钟即起，莫误了功课。"
