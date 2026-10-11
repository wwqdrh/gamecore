# 主场景开图引导 —— 挂 index.tscn（MapManager 之后，ready 顺序先于玩家接线）
#
# 职责：决定主场景进入时加载哪张地图。默认 initial_map（萧宅）；
# 存档「上次位置」（XiuPlayerState.pos_alias）若为主场景注册地图，
# 则恢复该图——重开游戏回到退出时所在的青石镇/青云坊等，而不是永远回萧宅。
#
# 配对关系：
#   · player.gd _resolve_spawn_position 恢复图内精确位置（alias 匹配 + 可行走）；
#     本引导先保证「开对图」，位置恢复才不会因 alias 不匹配回退默认出生点、
#     更不会在错误图上覆写存档位置
#   · 副本 alias（xiuxian_dungeon 等）不在主场景注册表 → 自然回退默认图；
#     秘境中途退出的续玩由 dungeon_manager 的 running 恢复负责，互不干扰
extends Node

func _ready() -> void:
	var mgr: Node = get_node("../MapManager")
	var alias := ""
	var pos: Dictionary = XiuPlayerState.ins().load_pos()
	if not pos.is_empty():
		alias = str(pos["alias"])
	var registered: Array = mgr.get_registered_maps()
	if alias.is_empty() or not registered.has(alias):
		alias = str(mgr.initial_map)  # 无记录/上次在副本/未知图 → 默认入口
	if not mgr.open_map(alias, mgr.initial_spawn):
		push_error("[MapRestore] 开图失败: %s" % alias)
		if not mgr.open_map(str(mgr.initial_map), mgr.initial_spawn):
			push_error("[MapRestore] 默认图也开图失败: %s" % mgr.initial_map)
