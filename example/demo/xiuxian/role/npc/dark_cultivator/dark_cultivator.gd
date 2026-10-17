# 玄鸦 —— 魔道蚀骨门使者：不落任何地图，仅作剧情演出 actor
# （movie.actors 注册 + 跨图 enter/exit；黑袍斗笠，声音沙哑如鸦）
extends XiuNpcBase


func _init() -> void:
	display_name = "黑袍人"
	role_name = "玄鸦"
	timeline_path = ""  # 不参与游戏内交互：只作 GdMoviePlayer 演出角色
	entry_stage = ""
	npc_cell = Vector2i(-1, -1)
	ai_behavior = -1  # 不挂 Brain：演出站位由剧本指令全权落位
	body_color = Color(0.12, 0.12, 0.16)
