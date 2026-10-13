# 端到端验收：GdDigLayer 挖地地图系统 + home.tscn
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_home_dig.gd
#
# 覆盖：
#   A. 初始地形：铺底 sand 填满 + init_rects 草地补丁；全显示格有图块（双网格渲染）
#   B. 挖地限制：dig_rects 外不可挖；rects 内 grass → sand，s_dug 信号触发
#   C. 视觉刷新：挖掘后受影响显示格图块变化（过渡片重算）
#   D. 持久化：get_terrain_grid dump → 新实例 set_terrain_grid restore 一致
#   E. 地图根协议：MapManager/player 鸭子类型接口齐备（is_walkable/world_to_cell/...）
extends SceneTree

const MAP_SCENE := "res://example/demo/xiuxian/map/home.tscn"

var ok := true


func _initialize() -> void:
	# 看门狗：脚本错误中止协程时强制退出（防零输出假死）
	create_timer(90.0).timeout.connect(func() -> void:
		print("[HomeDig] WATCHDOG 超时——测试未完成，强制退出")
		quit(2))
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[HomeDig] OK  ", msg)
	else:
		push_error("[HomeDig] FAIL  " + msg)
		ok = false


func _wait_frames(n: int) -> void:
	for i in n:
		await process_frame


func _finish() -> void:
	print("[HomeDig] RESULT: ", "ALL GREEN" if ok else "HAS FAILURES")
	quit(0 if ok else 1)


func _run() -> void:
	# ---- A. 初始地形 + 渲染 ----
	var map: Node = load(MAP_SCENE).instantiate()
	root.add_child(map)
	await _wait_frames(4)

	_check(map.get_terrain_name_at(Vector2i(0, 0)) == "sand", "铺底地形应为 sand（格 0,0）")
	_check(map.get_terrain_name_at(Vector2i(3, 3)) == "grass", "草地补丁内应为 grass（格 3,3）")
	_check(map.get_terrain_name_at(Vector2i(16, 1)) == "sand", "补丁外应为 sand（格 16,1）")
	_check(map.is_walkable(Vector2i(3, 3)) and map.is_walkable(Vector2i(0, 0)),
		"sand/grass 均可行走（blocked_terrains 空）")
	# 双网格渲染：显示格 (0..=w, 0..=h) 全部有图块
	var missing := 0
	for dy in range(0, 15):
		for dx in range(0, 21):
			if map.get_cell_source_id(Vector2i(dx, dy)) == -1:
				missing += 1
	_check(missing == 0, "全部显示格应有图块（缺 %d 个）" % missing)
	var tile_before: Vector2i = map.get_cell_atlas_coords(Vector2i(9, 6))
	_check(tile_before == Vector2i(2, 1), "纯草四角显示格应为整草图块 (2,1)，实际 %s" % tile_before)

	# ---- B. 挖地限制 + 信号 ----
	var dug_cells: Array = []
	map.connect("s_dug", func(cell, from_t, to_t) -> void:
		dug_cells.append([cell, str(from_t), str(to_t)]))
	_check(map.is_diggable(Vector2i(3, 3)) == false, "dig_rects 外草地不可挖（3,3）")
	_check(map.dig(Vector2i(3, 3)) == false, "rects 外 dig 应失败且地形不变（%s）" % map.get_terrain_name_at(Vector2i(3, 3)))
	_check(map.is_diggable(Vector2i(9, 6)) == true, "dig_rects 内草地可挖（9,6）")
	_check(map.dig(Vector2i(9, 6)) == true, "rects 内 dig 应成功")
	_check(map.get_terrain_name_at(Vector2i(9, 6)) == "sand", "挖后地形应变 sand")
	_check(dug_cells.size() == 1 and dug_cells[0][0] == Vector2i(9, 6)
		and dug_cells[0][1] == "grass" and dug_cells[0][2] == "sand",
		"s_dug 信号应触发一次且参数正确（%s）" % str(dug_cells))

	# ---- C. 视觉刷新 ----
	var tile_after: Vector2i = map.get_cell_atlas_coords(Vector2i(9, 6))
	_check(tile_after != tile_before, "挖掘后显示格 (9,6) 图块应变化（%s → %s）" % [tile_before, tile_after])
	var edge_tile: Vector2i = map.get_cell_atlas_coords(Vector2i(10, 6))
	_check(edge_tile != Vector2i(2, 1), "相邻显示格 (10,6) 应出现过渡片（%s）" % edge_tile)

	# ---- D. 持久化 roundtrip ----
	var grid: PackedInt32Array = map.get_terrain_grid()
	_check(grid.size() == 20 * 14, "网格 dump 长度应为 280")
	map.queue_free()
	await _wait_frames(2)
	var map2: Node = load(MAP_SCENE).instantiate()
	root.add_child(map2)
	await _wait_frames(4)
	_check(map2.set_terrain_grid(grid), "restore 应成功")
	_check(map2.get_terrain_name_at(Vector2i(9, 6)) == "sand", "restore 后 (9,6) 应保持 sand")
	_check(map2.get_terrain_name_at(Vector2i(3, 3)) == "grass", "restore 后 (3,3) 应保持 grass")
	_check(map2.dig(Vector2i(9, 6)) == false, "restore 后 (9,6) 已是 sand 不可再挖")

	# ---- E. 地图根协议（MapManager/player 鸭子类型）----
	for m in ["is_walkable", "world_to_cell", "cell_to_center", "get_map_width", "get_map_height", "get_cell_size_px"]:
		_check(map2.has_method(m), "地图根协议方法应存在: %s" % m)
	var c := Vector2i(5, 7)
	_check(map2.world_to_cell(map2.cell_to_center(c)) == c, "cell_to_center ↔ world_to_cell 往返一致")

	map2.queue_free()
	await _wait_frames(2)
	await _run_manager_smoke()


## F. MapManager 集成冒烟：home 别名注册 → open_map → 当前图正确 + 出生吸附可行走
func _run_manager_smoke() -> void:
	var mgr := GdMapManager.new()
	mgr.manager_id = "_home_smoke"
	root.add_child(mgr)
	mgr.register_map("xiuxian_home", MAP_SCENE)
	_check(mgr.open_map("xiuxian_home", Vector2i(9, 6)), "MapManager.open_map(home) 应成功")
	var cur: Node = mgr.get_current_map()
	_check(cur != null and str(cur.get("width")) == "20", "当前地图应为 home 实例")
	var alias: String = str(mgr.get_current_alias())
	_check(alias == "xiuxian_home", "current_alias 应为 xiuxian_home，实际 %s" % alias)
	# 出生格吸附：initial_spawn(9,6) 本身可行走 → get_spawn_cell 保持原格
	_check(mgr.get_spawn_cell() == Vector2i(9, 6), "可行走出生格应保持不变（%s）" % mgr.get_spawn_cell())
	mgr.queue_free()
	await _wait_frames(2)
	_finish()
