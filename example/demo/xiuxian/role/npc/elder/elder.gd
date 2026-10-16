# 执事长老 —— 站桩 NPC（AI_IDLE）：执事堂前不动，只负责对话
extends XiuNpcBase


func _init() -> void:
	display_name = "执事长老"
	role_name = "执事长老"
	timeline_path = "res://example/demo/xiuxian/role/npc/elder/elder_timeline.txt"
	entry_stage = ""  # 置空：入口选段交给 timeline gate 优先级扫描（entry 直达无条件兜底段会绕过门控）
	npc_cell = Vector2i(10, 9)
	ai_behavior = 0  # AI_IDLE 站桩（配置 schedule.json 后由日程系统接管）
	body_color = Color(0.85, 0.68, 0.25)
	# 重逢问候：首次对话后（timeline 置 met_elder flag）玩家再靠近主动打招呼
	remeet_flag = "met_elder"
	remeet_greeting = "来了？今日修行可有长进？"
