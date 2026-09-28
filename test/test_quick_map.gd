# suite: quick-map - GdQuickMap 快速地图生成器测试
# 覆盖: 种子确定性、阈值地形划分、通行查询与越界、格子坐标换算、
#        blocked 地形设置
extends "res://test/test_case.gd"


func _make_map(w: int = 8, h: int = 8, seed: int = 42) -> GdQuickMap:
	var tree := Engine.get_main_loop() as SceneTree
	var map := GdQuickMap.new()
	map.name = "QM"
	map.width = w
	map.height = h
	map.cell_size = 32
	map.seed_value = seed
	tree.root.add_child(map)
	map.generate(seed)
	return map


func test_seed_determinism() -> void:
	var a := _make_map(16, 16, 2026)
	var b := _make_map(16, 16, 2026)
	for cy in range(16):
		for cx in range(16):
			var cell := Vector2i(cx, cy)
			assert_eq(a.get_terrain_at(cell), b.get_terrain_at(cell),
				"同一种子应生成相同地形 (%d,%d)" % [cx, cy])
	a.free()
	b.free()


func test_all_terrains_present() -> void:
	# 16x16 的噪声图应能划分出所有 5 种默认地形
	var map := _make_map(24, 24, 99)
	var seen := {}
	for cy in range(24):
		for cx in range(24):
			var name := String(map.get_terrain_name_at(Vector2i(cx, cy)))
			seen[name] = true
	for expected in ["water", "sand", "grass", "forest", "mountain"]:
		assert_true(seen.has(expected), "应包含地形 %s" % expected)
	map.free()


func test_walkable_and_bounds() -> void:
	var map := _make_map(8, 8, 7)
	map.set_blocked_terrain_names(PackedStringArray(["water"]))

	# 有格子的地方通行性 = 地形不在 blocked 中
	for cy in range(8):
		for cx in range(8):
			var cell := Vector2i(cx, cy)
			var blocked := String(map.get_terrain_name_at(cell)) == "water"
			assert_eq(map.is_walkable(cell), not blocked,
				"(%d,%d) 通行性应与地形一致" % [cx, cy])

	# 越界不可通行
	assert_false(map.is_walkable(Vector2i(-1, 0)), "越界不可通行")
	assert_false(map.is_walkable(Vector2i(0, -1)), "越界不可通行")
	assert_false(map.is_walkable(Vector2i(8, 0)), "越界不可通行")
	assert_false(map.is_walkable(Vector2i(0, 8)), "越界不可通行")
	assert_eq(map.get_terrain_at(Vector2i(99, 99)), -1, "越界地形查询返回 -1")
	map.free()


func test_cell_coord_conversion() -> void:
	var map := _make_map(4, 4, 1)
	var center := map.cell_to_center(Vector2i(2, 3))
	assert_near(center.x, 2.5 * 32.0, 0.01, "格心 x = (cx+0.5)*cell")
	assert_near(center.y, 3.5 * 32.0, 0.01, "格心 y = (cy+0.5)*cell")

	var cell := map.world_to_cell(Vector2(70.0, 5.0))
	assert_eq(cell, Vector2i(2, 0), "世界坐标应换算到正确格子")
	map.free()


func test_set_terrain_at() -> void:
	var map := _make_map(4, 4, 1)
	map.set_terrain_at(Vector2i(1, 1), 2)  # grass
	assert_eq(String(map.get_terrain_name_at(Vector2i(1, 1))), "grass",
		"手动设置地形应生效")
	# 越界设置不崩溃
	map.set_terrain_at(Vector2i(-1, 0), 2)
	map.set_terrain_at(Vector2i(10, 10), 2)
	map.free()


func test_find_path() -> void:
	var map := _make_map(16, 16, 2026)
	map.set_blocked_terrain_names(PackedStringArray(["water", "mountain"]))

	# 找一对可达的通行格
	var start := Vector2i(-1, -1)
	var goal := Vector2i(-1, -1)
	var path: PackedVector2Array
	for cy in range(16):
		for cx in range(16):
			if not map.is_walkable(Vector2i(cx, cy)):
				continue
			var p: PackedVector2Array = map.find_path(Vector2i(cx, cy), Vector2i(15, 15))
			if map.is_walkable(Vector2i(15, 15)) and p.size() > 0:
				start = Vector2i(cx, cy)
				goal = Vector2i(15, 15)
				path = p
				break
		if start.x >= 0:
			break
	assert_true(start.x >= 0, "应找到可达路径对")

	if start.x >= 0:
		assert_eq(path[path.size() - 1], map.cell_to_center(goal),
			"路径最后一点应为终点格心")
		# 相邻路径点必须恰好相差一格（四方向，无斜跳）
		for i in range(path.size() - 1):
			var step: Vector2 = path[i + 1] - path[i]
			assert_true(absf(step.x) + absf(step.y) <= 32.5,
				"相邻路径点应相差一格 (i=%d, step=%s)" % [i, step])

	# 同种子重建地图：路径应完全一致（确定性）
	var map2 := _make_map(16, 16, 2026)
	map2.set_blocked_terrain_names(PackedStringArray(["water", "mountain"]))
	var path2: PackedVector2Array = map2.find_path(start, goal)
	assert_eq(path2.size(), path.size(), "同种子地图路径长度应一致")
	map2.free()

	# 终点为阻挡格 → 空
	var blocked := Vector2i(-1, -1)
	for cy in range(16):
		for cx in range(16):
			if not map.is_walkable(Vector2i(cx, cy)):
				blocked = Vector2i(cx, cy)
				break
		if blocked.x >= 0:
			break
	if blocked.x >= 0:
		assert_eq(map.find_path(start, blocked).size(), 0, "终点不可通行应返回空路径")
	# 越界终点 → 空
	assert_eq(map.find_path(start, Vector2i(99, 99)).size(), 0, "越界终点应返回空路径")
	# 起点 == 终点 → 空
	assert_eq(map.find_path(start, start).size(), 0, "起点等于终点应返回空路径")
	map.free()
