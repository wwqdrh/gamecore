# 玩家端到端验收 —— GdRoleMover 网格四向移动 + 地图传送联动 + 相机跟随/边界
#
# 运行：perl -e 'alarm 180; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_player_flow.gd
#
# 覆盖：
#   1. 装配：scenes/main/index.tscn 挂载 Player（GdRoleMover 网格模式），
#      落在出生格中心，grid_map_path 指向当前地图实例
#   2. 相机：GdSceneRoot 的 ViewCamera follow 玩家，边界限制在地图矩形内，
#      zoom 放大到视野小于地图
#   3. 网格移动：点击寻路（BFS 四方向最短路）逐格走完，落点=目标格中心，
#      路径逐段轴向对齐（不允许斜线）
#   4. 传送联动：寻路走到传送点格，落格后自动切图，玩家重定位到新图出生点，
#      相机边界同步刷新为新图矩形，grid_map_path 重绑新图
#   5. 反向传送：从 map_b 走回传送点，回到 map_a
extends SceneTree

var ok := true


func fail(msg: String) -> void:
	push_error("[Player] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _run() -> void:
	# 1. 装配
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "scenes/main/index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	for i in 6:
		await process_frame

	var player: Node = index.get_node_or_null("Player")
	var mgr: Node = index.get_node_or_null("MapManager")
	check(player != null, "主场景应挂载 Player")
	check(mgr != null, "主场景应挂载 MapManager")
	if player == null or mgr == null:
		_finish()
		return
	# 装备栏默认选中枪支（左键=开火）；本脚本验的是移动/传送链路，先卸下装备
	# 恢复左键点击寻路（装备/开火链路由 check_equip_flow.gd 专项覆盖）
	Engine.get_singleton("GDSTATE").set_state("mainhud.equip", "")
	check(not bool(player.gun_equipped), "卸下装备后玩家不应持枪")
	check(player.get_class() == "GdRoleMover", "Player 应为 GdRoleMover")
	check(int(player.move_mode) == 2, "移动模式应为 MODE_GRID(2)")
	check(int(player.control_mode) == 1, "控制方式应为键盘")
	check(absf(float(player.grid_cell_size) - 32.0) < 0.01, "格子尺寸应为 32")

	# 1.5 组件表：场景根 + MapManager/ViewCamera 自动注册 + 玩家接线
	check(Engine.has_singleton("GDCORE"), "应有 GDCORE 单例")
	var gdcore: Object = Engine.get_singleton("GDCORE")
	var sroot: Node = gdcore.get_global_node("default")
	check(sroot != null, "GDCORE 全局节点表应有场景根（manager_id=default）")
	check(sroot.has_method("get_component"), "场景根应提供组件表查询")
	if sroot == null:
		_finish()
		return
	var comp_mgr: Node = sroot.get_component("MapManager")
	check(comp_mgr != null and comp_mgr.get_instance_id() == mgr.get_instance_id(),
		"组件表 MapManager 应为场景中的管理器实例（自动注册）")
	var comp_names: PackedStringArray = sroot.get_component_names()
	check("ViewCamera" in comp_names, "组件表应含 ViewCamera，实际 %s" % [comp_names])
	check(player.scene_root != null and player.map_mgr != null and player.camera != null,
		"player 应经组件表完成接线（scene_root/map_mgr/camera）")
	print("[Player] components: names=%s wired=%s" % [comp_names,
		player.scene_root != null and player.map_mgr != null])

	var map: Node = mgr.get_current_map()
	check(map != null, "初始地图应已加载")
	if map == null:
		_finish()
		return
	var spawn_pt: Vector2 = mgr.get_spawn_point()
	check(player.global_position == spawn_pt,
		"玩家应落在出生格中心 %s，实际 %s" % [spawn_pt, player.global_position])
	var cur_cell: Vector2i = map.world_to_cell(player.global_position)
	check(map.is_walkable(cur_cell), "玩家所在格应可行走")
	# 网格绑定：grid_map_path 指向的节点应为当前地图实例
	var bound: Node = player.get_node_or_null(player.grid_map_path)
	check(bound != null and bound.get_instance_id() == map.get_instance_id(),
		"grid_map_path 应绑定当前地图实例")
	print("[Player] setup: cell=%s spawn=%s bound=%s" % [cur_cell, spawn_pt, bound != null])

	# 2. 相机：经组件表查询 GdSceneRoot 的 ViewCamera，跟随玩家 + 地图边界
	var camera: Camera2D = sroot.get_component("ViewCamera")
	check(camera != null and camera is Camera2D, "组件表 ViewCamera 应为 Camera2D")
	if camera == null:
		_finish()
		return
	check(camera.follow_node != null
		and camera.follow_node.get_instance_id() == player.get_instance_id(),
		"相机应跟随 Player")
	var w: int = map.get_map_width()
	var h: int = map.get_map_height()
	check(camera.get_limit(SIDE_LEFT) == 0 and camera.get_limit(SIDE_TOP) == 0,
		"相机左/上边界应为 0")
	check(camera.get_limit(SIDE_RIGHT) == w * 32 and camera.get_limit(SIDE_BOTTOM) == h * 32,
		"相机右/下边界应为地图矩形 (%d, %d)" % [w * 32, h * 32])
	# 等缩放 tween 完成（start_zoom 平滑过渡到 zoom_max）
	var zi := 0
	while camera.zoom.x < 2.49 and zi < 120:
		await process_frame
		zi += 1
	check(absf(camera.zoom.x - 2.5) < 0.01,
		"相机应放大到 zoom 2.5（视野小于地图边界限制才生效），实际 %s" % camera.zoom)
	print("[Player] camera: follow=%s limits=(%d,%d,%d,%d) zoom=%s" % [
		camera.follow_node != null,
		camera.get_limit(SIDE_LEFT), camera.get_limit(SIDE_RIGHT),
		camera.get_limit(SIDE_TOP), camera.get_limit(SIDE_BOTTOM), camera.zoom])

	# 3. 网格移动：点击寻路逐格走完（无斜线段）
	var path: PackedVector2Array = _find_reachable_path(map, cur_cell)
	check(path.size() > 0, "应能从出生点 BFS 到某个相邻可行走格")
	if path.is_empty():
		_finish()
		return
	# 路径轴向对齐断言：相邻点必须只有一轴变化（BFS 四方向保证无斜线）
	for i in range(1, path.size()):
		var seg: Vector2 = path[i] - path[i - 1]
		check(seg.x == 0.0 or seg.y == 0.0,
			"路径第 %d 段存在斜线 %s" % [i, seg])
	player.set_grid_path(path)
	var done: bool = await _await_path_done(player)
	check(done, "寻路应在时限内走完")
	await _settle(player)
	var end_cell: Vector2i = map.world_to_cell(player.global_position)
	check(player.global_position == path[path.size() - 1],
		"落点应为路径终点格心 %s，实际 %s" % [path[path.size() - 1], player.global_position])
	check(end_cell.x != cur_cell.x or end_cell.y != cur_cell.y, "玩家应已离开出生格")
	print("[Player] walk: %d 格 → cell=%s" % [path.size(), end_cell])

	# 4. 传送联动：落位到传送点格（水域可能阻断跨图寻路，走格能力已由第 3 步
	#    覆盖），玩家 _process 落格判定应触发自动切图
	var markers: Array = mgr.get_markers()
	check(markers.size() >= 1, "当前地图应有传送点")
	if markers.is_empty():
		_finish()
		return
	var marker: Node2D = markers[0]
	var expect_alias: String = str(marker.target_alias)
	player.global_position = marker.get_center()
	# 等传送判定（_process 落格查询）与切图重定位
	for i in 10:
		await process_frame
	check(str(mgr.get_current_alias()) == expect_alias,
		"走到传送点后应自动切到 %s，实际 %s" % [expect_alias, mgr.get_current_alias()])
	var map_b: Node = mgr.get_current_map()
	check(map_b != null and map_b.get_class() == "GdQuickMap", "新地图应为 GdQuickMap")
	var spawn_b: Vector2 = mgr.get_spawn_point()
	check(player.global_position == spawn_b,
		"切图后玩家应重定位到新图出生点 %s，实际 %s" % [spawn_b, player.global_position])
	var bound_b: Node = player.get_node_or_null(player.grid_map_path)
	check(bound_b != null and bound_b.get_instance_id() == map_b.get_instance_id(),
		"grid_map_path 应重绑到新地图")
	check(camera.get_limit(SIDE_RIGHT) == int(map_b.get_map_width()) * 32
		and camera.get_limit(SIDE_BOTTOM) == int(map_b.get_map_height()) * 32,
		"相机边界应刷新为新图矩形 (%d, %d)" % [
			int(map_b.get_map_width()) * 32, int(map_b.get_map_height()) * 32])
	print("[Player] teleport a->b: spawn=%s" % spawn_b)

	# 5. 反向传送：新图 →(首传送点目标)
	var marker_b: Node2D = mgr.get_markers()[0]
	var expect_back: String = str(marker_b.target_alias)
	player.global_position = marker_b.get_center()
	for i in 10:
		await process_frame
	check(str(mgr.get_current_alias()) == expect_back,
		"回程后应切到 %s，实际 %s" % [expect_back, mgr.get_current_alias()])
	check(player.global_position == mgr.get_spawn_point(),
		"回程后玩家应在新出生点")
	print("[Player] teleport back: alias=%s" % mgr.get_current_alias())

	index.queue_free()
	_finish()


# 从 from_cell 找一个 BFS 可达且非当前的可行走格，返回完整路径
func _find_reachable_path(map: Node, from_cell: Vector2i) -> PackedVector2Array:
	var w: int = map.get_map_width()
	var h: int = map.get_map_height()
	var offsets := [
		Vector2i(2, 0), Vector2i(-2, 0), Vector2i(0, 2), Vector2i(0, -2),
		Vector2i(3, 1), Vector2i(-3, -1), Vector2i(1, 3), Vector2i(-1, -3),
		Vector2i(4, 0), Vector2i(0, 4),
	]
	for off in offsets:
		var t := Vector2i(from_cell.x + off.x, from_cell.y + off.y)
		if t.x < 0 or t.y < 0 or t.x >= w or t.y >= h:
			continue
		if not map.is_walkable(t):
			continue
		var p: PackedVector2Array = map.find_path(from_cell, t)
		if not p.is_empty():
			return p
	return PackedVector2Array()


# 等待网格路径走完（超时 1500 物理帧 ≈ 25s）
func _await_path_done(player: Node) -> bool:
	var i := 0
	while player.is_grid_path_active() and i < 1500:
		await physics_frame
		i += 1
	return not player.is_grid_path_active()


# 等待玩家静止在格心（最多 30 帧）
func _settle(player: Node) -> void:
	var i := 0
	while player.is_moving() and i < 30:
		await physics_frame
		i += 1


func _finish() -> void:
	if ok:
		print("[Player] RESULT=PASS")
	else:
		print("[Player] RESULT=FAIL")
	quit(0 if ok else 1)
