# 地图系统端到端验收 —— GdMapManager 地图注册/加载/切换/传送 + GdMapMarker 高亮传送点
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_map_flow.gd
#
# 覆盖：
#   1. scenes/main/index.tscn 装配：MapManager 存在、注册表、initial_map 显式配置
#   2. 初始加载：open_map 自动进入主城 xiuxian_town，当前地图为 GdQuickMap
#   3. 传送点：当前地图上有 GdMapMarker，且格子已吸附到可行走地形
#   4. 命中判定：marker.get_center() 命中 contains_world_point；远处点不命中
#   5. 传送流转：try_teleport(传送点中心) 切到 xiuxian_map_a，再传到 map_b
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
	check(str(mgr.initial_map) == "xiuxian_town", "initial_map 应显式配置为 xiuxian_town（主城默认加载）")
	var registered = mgr.get_registered_maps()
	check(registered.has("xiuxian_town") and registered.has("xiuxian_map_a")
		and registered.has("xiuxian_map_b"),
		"注册表应含 xiuxian_town/map_a/map_b 三个别名，实际 %s" % [registered])

	# 2. 初始加载：ready 时自动进入 initial_map（主城）
	check(str(mgr.get_current_alias()) == "xiuxian_town", "初始应自动加载主城 xiuxian_town")
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

	# 3. 传送点：数量、分组收集、格子已吸附可行走（主城两个传送点）
	var markers: Array = mgr.get_markers()
	check(markers.size() == 2, "主城应有 2 个传送点（东→map_a / 北→map_b），实际 %d" % markers.size())
	if markers.is_empty():
		_finish()
		return
	var marker: Node2D = markers[0]
	check(marker.get_class() == "GdMapMarker", "传送点应为 GdMapMarker")
	check(str(marker.target_alias) == "xiuxian_map_a", "主城首个传送点应指向 xiuxian_map_a")
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
	check(str(mgr.get_current_alias()) == "xiuxian_town", "未命中不应切换地图")

	# 5. 信号 + 传送流转：主城 →(东门) map_a
	mgr.connect("s_map_changed", func(alias: String) -> void:
		map_changed_log.append(alias))
	mgr.connect("s_teleport_triggered", func(from: String, to: String, cell: Vector2i) -> void:
		teleport_log.append([from, to, cell]))

	var jumped: bool = mgr.try_teleport(center)
	check(jumped, "命中传送点的传送应返回 true")
	check(str(mgr.get_current_alias()) == "xiuxian_map_a", "传送后应切换到 xiuxian_map_a")
	check(map_changed_log == ["xiuxian_map_a"], "s_map_changed 应发出 xiuxian_map_a，实际 %s" % [map_changed_log])
	check(teleport_log.size() == 1 and teleport_log[0][0] == "xiuxian_town"
		and teleport_log[0][1] == "xiuxian_map_a",
		"s_teleport_triggered 应记录 town→a 流转，实际 %s" % [teleport_log])
	var new_map: Object = mgr.get_current_map()
	check(new_map.is_walkable(mgr.get_spawn_cell()),
		"传送后出生格 %s 应吸附在可行走地形上" % mgr.get_spawn_cell())
	print("[Map] teleport town->a: changed=%s teleports=%s" % [map_changed_log, teleport_log])

	# 6. 反向传送：map_a →(首传送点) map_b
	var markers_a: Array = mgr.get_markers()
	check(markers_a.size() == 2, "map_a 应有 2 个传送点（东→map_b / 西→主城）")
	var marker_a: Node2D = markers_a[0]
	check(str(marker_a.target_alias) == "xiuxian_map_b", "map_a 首传送点应指向 xiuxian_map_b")
	var map_a: Object = mgr.get_current_map()
	var a_cell: Vector2i = marker_a.get_cell()
	check(map_a.is_walkable(a_cell), "map_a 传送点格子也应吸附可行走")
	var back: bool = mgr.try_teleport(marker_a.get_center())
	check(back, "反向传送应成功")
	check(str(mgr.get_current_alias()) == "xiuxian_map_b", "反向传送后应到 xiuxian_map_b")
	check(map_changed_log == ["xiuxian_map_a", "xiuxian_map_b"], "s_map_changed 应记录 a→b")
	print("[Map] teleport a->b: changed=%s" % [map_changed_log])

	# 7. 未注册别名拒绝
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
