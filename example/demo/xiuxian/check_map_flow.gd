# 地图系统端到端验收 —— GdMapManager 地图注册/加载/切换/传送 + GdMapMarker 高亮传送点
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_map_flow.gd
#
# 覆盖：
#   1. scenes/main/index.tscn 装配：MapManager 存在、注册表、initial_map 显式配置
#   2. 初始加载：open_map 自动进入萧宅 xiuxian_xiaozhai（默认入口），
#      paint_rects 手绘建筑块生效（山地=建筑：正房/厢房/南墙不可行走、大门可通行）
#   3. 传送点：当前地图上有 GdMapMarker，且格子已吸附到可行走地形
#   4. 命中判定：marker.get_center() 命中 contains_world_point；远处点不命中
#   5. 传送流转：萧宅 →(南门) 青石镇 →(西门) 返回萧宅，
#      再 open_map 青云坊 →(东门) map_a →(传送点) map_b
#   6. 信号：s_map_changed / s_teleport_triggered 均发出且参数正确
#   7. 出生点：open_map 记录 spawn_cell，get_spawn_point 为格心世界坐标
extends SceneTree

var ok := true
var map_changed_log: Array = []
var teleport_log: Array = []


func fail(msg: String) -> void:
	push_error("[Map] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _run() -> void:
	# 0. GDCORE 单例（管理器注册后端）
	var core: Object = Engine.get_singleton("GDCORE")
	check(core != null, "GDCORE 单例应已注册")

	# 1. 主场景装配断言：MapManager 节点 + 显式初始地图配置
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "scenes/main/index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)

	# 等待 ready 链路（-s 模式 ready 延迟到首帧）
	for i in 4:
		await process_frame

	var mgr: Node = index.get_node_or_null("MapManager")
	check(mgr != null, "主场景应挂载 MapManager 节点")
	if mgr == null:
		_finish()
		return
	check(mgr.get_class() == "GdMapManager", "MapManager 应为 GdMapManager 类型")
	check(str(mgr.initial_map) == "xiuxian_xiaozhai", "initial_map 应显式配置为 xiuxian_xiaozhai（默认进萧宅）")
	var registered = mgr.get_registered_maps()
	check(registered.has("xiuxian_xiaozhai") and registered.has("xiuxian_xiaozhen")
		and registered.has("xiuxian_town") and registered.has("xiuxian_map_a")
		and registered.has("xiuxian_map_b"),
		"注册表应含萧宅/青石镇/青云坊/map_a/map_b 五个别名，实际 %s" % [registered])

	# 2. 初始加载：ready 时自动进入萧宅（默认入口）
	check(str(mgr.get_current_alias()) == "xiuxian_xiaozhai", "初始应自动加载萧宅 xiuxian_xiaozhai")
	var cur_map: Object = mgr.get_current_map()
	check(cur_map != null, "当前地图实例不应为空")
	if cur_map == null:
		_finish()
		return
	check(cur_map.get_class() == "GdQuickMap", "地图根应为 GdQuickMap")
	check(mgr.get_current_cell_size() == 32, "当前地图格子尺寸应为 32")
	var init_spawn: Vector2i = mgr.get_spawn_cell()
	check(cur_map.is_walkable(init_spawn),
		"出生格 %s 应吸附在可行走地形上" % init_spawn)
	var spawn_pt: Vector2 = mgr.get_spawn_point()
	check(spawn_pt == cur_map.cell_to_center(init_spawn),
		"出生点应为出生格中心世界坐标，实际 %s" % spawn_pt)
	print("[Map] initial: alias=%s map=%s cell_size=%d spawn=%s" % [
		mgr.get_current_alias(), cur_map.get_class(), mgr.get_current_cell_size(), spawn_pt])

	# 2b. 手绘建筑块（paint_rects，山地=建筑）：正房/厢房/南墙不可行走，大门可通行
	check(str(cur_map.get_terrain_name_at(Vector2i(9, 3))) == "mountain",
		"正房 (9,3) 应为手绘山地（建筑）")
	check(not cur_map.is_walkable(Vector2i(9, 3)), "正房格应不可通行")
	check(not cur_map.is_walkable(Vector2i(14, 6)), "东厢格应不可通行")
	check(not cur_map.is_walkable(Vector2i(5, 10)), "南墙西段应不可通行")
	check(not cur_map.is_walkable(Vector2i(13, 10)), "南墙东段应不可通行")
	check(cur_map.is_walkable(Vector2i(9, 10)) and cur_map.is_walkable(Vector2i(10, 10)),
		"南墙大门 (9..10,10) 应可通行")
	check(cur_map.is_walkable(Vector2i(10, 8)), "庭院 (10,8) 应可通行")

	# 3. 传送点：萧宅 1 个（南门→青石镇），格子已吸附可行走
	var markers: Array = mgr.get_markers()
	check(markers.size() == 1, "萧宅应有 1 个传送点（南门→青石镇），实际 %d" % markers.size())
	if markers.is_empty():
		_finish()
		return
	var marker: Node2D = markers[0]
	check(marker.get_class() == "GdMapMarker", "传送点应为 GdMapMarker")
	check(str(marker.target_alias) == "xiuxian_xiaozhen", "萧宅传送点应指向 xiuxian_xiaozhen")
	var m_cell: Vector2i = marker.get_cell()
	check(cur_map.is_walkable(m_cell),
		"传送点格子 %s 应吸附在可行走地形上" % m_cell)
	print("[Map] markers: count=%d cell=%s target=%s" % [
		markers.size(), m_cell, marker.target_alias])

	# 4. 命中判定
	var center: Vector2 = marker.get_center()
	check(marker.contains_world_point(center), "传送点中心应命中自身格子")
	check(not marker.contains_world_point(center + Vector2(500, 500)),
		"远处坐标不应命中传送点")
	check(not mgr.try_teleport(center + Vector2(500, 500)),
		"未命中传送点的传送应返回 false")
	check(str(mgr.get_current_alias()) == "xiuxian_xiaozhai", "未命中不应切换地图")

	# 5. 信号 + 传送流转：萧宅 →(南门) 青石镇
	mgr.connect("s_map_changed", func(alias: String) -> void:
		map_changed_log.append(alias))
	mgr.connect("s_teleport_triggered", func(from: String, to: String, cell: Vector2i) -> void:
		teleport_log.append([from, to, cell]))

	var jumped: bool = mgr.try_teleport(center)
	check(jumped, "命中传送点的传送应返回 true")
	check(str(mgr.get_current_alias()) == "xiuxian_xiaozhen", "传送后应切换到 xiuxian_xiaozhen")
	check(map_changed_log == ["xiuxian_xiaozhen"], "s_map_changed 应发出 xiuxian_xiaozhen，实际 %s" % [map_changed_log])
	check(teleport_log.size() == 1 and teleport_log[0][0] == "xiuxian_xiaozhai"
		and teleport_log[0][1] == "xiuxian_xiaozhen",
		"s_teleport_triggered 应记录 萧宅→青石镇 流转，实际 %s" % [teleport_log])
	var new_map: Object = mgr.get_current_map()
	check(new_map.is_walkable(mgr.get_spawn_cell()),
		"传送后出生格 %s 应吸附在可行走地形上" % mgr.get_spawn_cell())
	check(new_map.is_walkable(Vector2i(5, 8)), "青石镇药铺门前 (5,8) 应可通行")
	check(not new_map.is_walkable(Vector2i(5, 5)), "青石镇药铺建筑 (5,5) 应不可通行")
	print("[Map] teleport xiaozhai->xiaozhen: changed=%s teleports=%s" % [map_changed_log, teleport_log])

	# 6. 反向传送：青石镇 →(西门) 返回萧宅
	var markers_z: Array = mgr.get_markers()
	check(markers_z.size() == 2, "青石镇应有 2 个传送点（西门→萧宅 / 东门→青云坊）")
	var marker_z: Node2D = markers_z[0]
	check(str(marker_z.target_alias) == "xiuxian_xiaozhai", "青石镇首传送点应指向 xiuxian_xiaozhai")
	var back: bool = mgr.try_teleport(marker_z.get_center())
	check(back, "反向传送应成功")
	check(str(mgr.get_current_alias()) == "xiuxian_xiaozhai", "反向传送后应回到萧宅")

	# 7. 青云坊 → map_a → map_b 老流转（显式 open_map 进入主城）
	mgr.open_map("xiuxian_town", Vector2i(6, 10))
	check(str(mgr.get_current_alias()) == "xiuxian_town", "open_map 应切换到青云坊")
	var markers_t: Array = mgr.get_markers()
	check(markers_t.size() == 2, "青云坊应有 2 个传送点（东→map_a / 北→map_b）")
	var marker_t: Node2D = markers_t[0]
	check(str(marker_t.target_alias) == "xiuxian_map_a", "青云坊首传送点应指向 xiuxian_map_a")
	var to_a: bool = mgr.try_teleport(marker_t.get_center())
	check(to_a and str(mgr.get_current_alias()) == "xiuxian_map_a", "青云坊→青草岭传送应成功")
	var markers_a: Array = mgr.get_markers()
	check(markers_a.size() == 2, "map_a 应有 2 个传送点")
	var marker_a: Node2D = markers_a[0]
	var to_b: bool = mgr.try_teleport(marker_a.get_center())
	check(to_b and str(mgr.get_current_alias()) == "xiuxian_map_b", "青草岭→幽竹林传送应成功")
	check(map_changed_log.size() == 5, "s_map_changed 应累计 5 次流转，实际 %s" % [map_changed_log])
	check(map_changed_log.slice(2) == ["xiuxian_town", "xiuxian_map_a", "xiuxian_map_b"],
		"后三段流转应为 town→a→b，实际 %s" % [map_changed_log.slice(2)])
	print("[Map] teleport town->a->b: changed=%s" % [map_changed_log])

	# 8. 未注册别名拒绝
	check(not mgr.open_map("no_such_map", Vector2i.ZERO), "未注册别名应返回 false")
	check(str(mgr.get_current_alias()) == "xiuxian_map_b", "失败切换不应改变当前地图")

	# 清理
	index.queue_free()
	_finish()


func _finish() -> void:
	if ok:
		print("[Map] RESULT=PASS")
	else:
		print("[Map] RESULT=FAIL")
	quit(0 if ok else 1)
