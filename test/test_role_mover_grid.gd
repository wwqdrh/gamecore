# suite: role-grid - GdRoleMover 网格移动模式测试
# 覆盖: MODE_GRID 一次一格移动、格心对齐、禁止斜向、阻挡格不可进入、
#        按住连续走格、moving 信号
extends "res://test/test_case.gd"

const CELL := 32
const MAP_SEED := 7

var map: GdQuickMap
var player: GdRoleMover
var moving_events: Array[bool] = []


func wait_phys(frames: int) -> void:
	var tree := Engine.get_main_loop() as SceneTree
	for i in frames:
		await tree.physics_frame


func wait_phys_until(cond: Callable, timeout_ticks: int = 600) -> bool:
	var tree := Engine.get_main_loop() as SceneTree
	var ticks := 0
	while ticks < timeout_ticks:
		if cond.call():
			return true
		await tree.physics_frame
		ticks += 1
	return cond.call()


func _on_moving(m: bool) -> void:
	moving_events.append(m)


## 构建网格场景：8x8 地图 + AI 控制的网格移动角色
func _build() -> void:
	moving_events.clear()
	var tree := Engine.get_main_loop() as SceneTree
	var scene := Node2D.new()
	scene.name = "GridScene"
	tree.root.add_child(scene)

	map = GdQuickMap.new()
	map.name = "QuickMap"
	map.width = 8
	map.height = 8
	map.cell_size = CELL
	map.seed_value = MAP_SEED
	scene.add_child(map)
	map.generate(MAP_SEED)
	map.set_blocked_terrain_names(PackedStringArray(["water", "mountain"]))

	player = GdRoleMover.new()
	player.name = "Player"
	player.control_mode = 3  # AI（测试用 set_ai_direction 模拟输入）
	player.move_mode = 2     # MODE_GRID
	player.speed = CELL * 20.0  # 一格约 3 物理帧
	player.grid_cell_size = CELL
	player.grid_map_path = NodePath("../QuickMap")
	player.s_moving_changed.connect(_on_moving)
	scene.add_child(player)


func _place_at(cell: Vector2i) -> void:
	player.position = map.cell_to_center(cell)
	await wait_phys(2)


func _is_aligned(p: Vector2) -> bool:
	var cx := (p.x / CELL + 0.5)
	var cy := (p.y / CELL + 0.5)
	return absf(cx - roundf(cx)) < 0.01 and absf(cy - roundf(cy)) < 0.01


func test_grid_moves_one_cell() -> void:
	_build()
	await _place_at(Vector2i(2, 2))
	var start := player.position

	player.set_ai_direction(Vector2.RIGHT)
	var started := await wait_phys_until(
		func(): return player.is_moving(), 120)
	assert_true(started, "应开始移动")
	var arrived := await wait_phys_until(
		func(): return not player.is_moving(), 120)
	assert_true(arrived, "移动一格后应停下")
	player.set_ai_direction(Vector2.ZERO)
	await wait_phys(3)

	var delta := player.position - start
	assert_near(delta.x, float(CELL), 0.5, "应恰好右移一格")
	assert_near(delta.y, 0.0, 0.5, "纵向不应移动")
	assert_true(_is_aligned(player.position), "停止位置应对齐格心")
	assert_false(player.is_moving(), "停止后 moving 应为 false")
	player.get_parent().free()


func test_grid_no_diagonal() -> void:
	_build()
	await _place_at(Vector2i(2, 2))
	var start := player.position

	# 斜向输入：应只走主导轴（x 优先）
	player.set_ai_direction(Vector2(1, 1).normalized())
	var started := await wait_phys_until(
		func(): return player.is_moving(), 120)
	assert_true(started, "应开始移动")
	var arrived := await wait_phys_until(
		func(): return not player.is_moving(), 120)
	assert_true(arrived, "斜向输入应走一格后停下")
	player.set_ai_direction(Vector2.ZERO)
	await wait_phys(3)

	var delta := player.position - start
	assert_near(delta.x, float(CELL), 0.5, "主导轴 x 应移动一格")
	assert_near(delta.y, 0.0, 0.5, "禁止斜向：纵向不应移动")
	player.get_parent().free()


func test_grid_blocked_cell() -> void:
	_build()
	map.generate(MAP_SEED)
	map.set_blocked_terrain_names(PackedStringArray(["water", "mountain"]))

	# 找一个水格，且其左侧格可通行
	var target := Vector2i(-1, -1)
	for cy in range(8):
		for cx in range(1, 8):
			if map.get_terrain_name_at(Vector2i(cx, cy)) == "water" \
					and map.is_walkable(Vector2i(cx - 1, cy)):
				target = Vector2i(cx, cy)
				break
		if target.x >= 0:
			break
	assert_true(target.x >= 0, "地图中应存在左邻可通行阻挡格")

	await _place_at(Vector2i(target.x - 1, target.y))
	player.set_ai_direction(Vector2.RIGHT)
	await wait_phys(30)  # 足够走数格的时间

	var delta := player.position - map.cell_to_center(Vector2i(target.x - 1, target.y))
	assert_near(delta.x, 0.0, 0.5, "前方是阻挡格，不应移动")
	assert_near(delta.y, 0.0, 0.5, "纵向不应移动")
	player.get_parent().free()


func test_grid_continuous_hold() -> void:
	_build()
	await _place_at(Vector2i(1, 4))
	var start := player.position

	# 持续按住方向：应连续走多格且每格对齐
	player.set_ai_direction(Vector2.RIGHT)
	var moved := await wait_phys_until(
		func(): return absf(player.position.x - start.x) >= CELL * 3.0, 240)
	player.set_ai_direction(Vector2.ZERO)
	await wait_phys_until(func(): return not player.is_moving(), 120)
	await wait_phys(2)

	assert_true(moved, "按住方向应连续走格")
	var cells_moved := roundf((player.position.x - start.x) / CELL)
	assert_true(cells_moved >= 3.0, "至少连续移动 3 格 (moved=%d)" % int(cells_moved))
	assert_true(_is_aligned(player.position), "停止位置应对齐格心")
	player.get_parent().free()


func test_grid_moving_signal_sequence() -> void:
	_build()
	await _place_at(Vector2i(2, 2))
	moving_events.clear()

	player.set_ai_direction(Vector2.DOWN)
	var started := await wait_phys_until(
		func(): return player.is_moving(), 120)
	assert_true(started, "应开始移动")
	var arrived := await wait_phys_until(
		func(): return not player.is_moving(), 120)
	player.set_ai_direction(Vector2.ZERO)
	await wait_phys(3)
	assert_true(arrived, "应完成一格移动")
	assert_true(moving_events.size() >= 2, "应发出 moving true/false 事件对")
	assert_true(moving_events[0], "第一个事件应为 true")
	assert_false(moving_events[moving_events.size() - 1], "最后一个事件应为 false")
	var delta := player.position - map.cell_to_center(Vector2i(2, 2))
	assert_near(delta.y, float(CELL), 0.5, "应向下移动一格")
	player.get_parent().free()


func test_grid_path_following() -> void:
	_build()
	# 从某通行格出发，用 find_path 找一条可达路径
	var start := Vector2i(-1, -1)
	var path: PackedVector2Array
	for cy in range(8):
		for cx in range(8):
			if not map.is_walkable(Vector2i(cx, cy)):
				continue
			var p: PackedVector2Array = map.find_path(Vector2i(cx, cy), Vector2i(7, 7))
			if map.is_walkable(Vector2i(7, 7)) and p.size() >= 3:
				start = Vector2i(cx, cy)
				path = p
				break
		if start.x >= 0:
			break
	assert_true(start.x >= 0, "应找到 >=3 格的可达路径")
	if start.x < 0:
		player.get_parent().free()
		return

	var finished := [false]
	player.s_grid_path_finished.connect(func(): finished[0] = true)
	await _place_at(start)
	assert_true(player.is_grid_path_active() == false, "设置路径前不应处于跟随状态")

	player.set_grid_path(path)
	assert_true(player.is_grid_path_active(), "设置路径后应处于跟随状态")
	var ok := await wait_phys_until(func(): return finished[0], 1200)
	assert_true(ok, "路径应走完并发出 s_grid_path_finished")
	await wait_phys(2)

	assert_false(player.is_grid_path_active(), "走完后跟随状态应复位")
	assert_near(player.position.x, path[path.size() - 1].x, 0.5,
		"终点 x 应等于路径最后一点")
	assert_near(player.position.y, path[path.size() - 1].y, 0.5,
		"终点 y 应等于路径最后一点")
	assert_true(_is_aligned(player.position), "终点应对齐格心")
	assert_false(player.is_moving(), "走完后应停止")
	player.get_parent().free()


func test_grid_path_interrupt_by_input() -> void:
	_build()
	var start := Vector2i(-1, -1)
	var path: PackedVector2Array
	for cy in range(8):
		for cx in range(8):
			if not map.is_walkable(Vector2i(cx, cy)):
				continue
			var p: PackedVector2Array = map.find_path(Vector2i(cx, cy), Vector2i(7, 7))
			if map.is_walkable(Vector2i(7, 7)) and p.size() >= 3:
				start = Vector2i(cx, cy)
				path = p
				break
		if start.x >= 0:
			break
	assert_true(start.x >= 0, "应找到可达路径")
	if start.x < 0:
		player.get_parent().free()
		return

	await _place_at(start)
	player.set_grid_path(path)
	var started := await wait_phys_until(
		func(): return player.is_moving(), 120)
	assert_true(started, "设置路径后应开始移动")

	# AI 输入打断：路径应被清除
	player.set_ai_direction(Vector2.RIGHT)
	var cleared := await wait_phys_until(
		func(): return not player.is_grid_path_active(), 120)
	assert_true(cleared, "移动输入应打断路径跟随")
	# 打断后当前输入触发的那一格会走完，等它停下
	player.set_ai_direction(Vector2.ZERO)
	await wait_phys_until(func(): return not player.is_moving(), 120)
	await wait_phys(3)
	assert_false(player.is_moving(), "打断并清零输入后应停止")
	player.get_parent().free()
