# 演出期常驻 NPC 抑制 + 点击寻路关闭 验收（headless 手动跑）：
#   godot --headless --path . -s res://example/demo/xiuxian/check_movie_npc_dedup.gd
#
# 覆盖：
#   1. 演出 dedup：xiaozhai 内嵌萧家三口常驻 NPC，c01 剧本 actor 再实例化
#      同角色 → 播片期间常驻 NPC 自动隐藏（不重复出屏），actor 正常可见
#   2. stop() 后常驻 NPC 恢复可见（演出不永久改写场景）
#   3. click_to_move=false（产品默认）：点击不再生成寻路/预览线；
#      =true 时机制仍在（预览线回归见 check_path_preview.gd）
extends SceneTree

var ok := true


func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	wd.autostart = true
	wd.timeout.connect(func() -> void: quit(2))
	root.add_child(wd)

	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	XiuMovieState.ins().mark_played("prologue")  # 屏蔽 MovieTrigger 并发开播

	var app: Node = load("res://example/demo/xiuxian/index.tscn").instantiate()
	root.add_child(app)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_title", 300):
		_check(false, "未进入 title")
		_finish(1)
		return
	app.call("change_scene", "xiuxian_main", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_main", 600):
		_check(false, "未切到 main")
		await _finish(1)
		return

	var player: Node = get_first_node_in_group("player")
	if not await _poll(func() -> bool: return bool(player.get("_wired")), 600):
		_check(false, "player 接线完成")
		await _finish(1)
		return
	var map_mgr: Node = player.get("map_mgr")

	# ---- 1. 开萧宅图（内嵌萧家三口常驻 NPC）----
	_check(map_mgr.open_map("xiuxian_xiaozhai", Vector2i(10, 8)), "开萧宅图")
	if not await _poll(func() -> bool:
		return str(map_mgr.get_current_alias()) == "xiuxian_xiaozhai", 300):
		_check(false, "萧宅图已开")
		await _finish(1)
		return
	await _wait(10)
	var map: Node = map_mgr.get_current_map()
	var residents: Array = _npcs_in(map)
	_check(residents.size() == 3, "常驻 NPC 3 个（萧战/柳氏/小翠），实际 %d" % residents.size())
	_check(residents.all(func(n: Node) -> bool: return n.is_visible()),
			"演出前常驻 NPC 全部可见")

	# ---- 2. 播 c01（actor 与常驻 NPC 同角色）：播片中常驻 NPC 隐藏 ----
	var movie: Node = GdMoviePlayer.new()
	root.add_child(movie)
	movie.call("configure", "res://example/demo/xiuxian/movie/c01_farewell/c01_farewell.movie",
			NodePath(), movie.get_path_to(player))
	movie.call("play")
	var suppressed: bool = await _poll(func() -> bool:
		return residents.all(func(n: Node) -> bool: return not n.is_visible()), 120)
	_check(suppressed, "演出开播后常驻 NPC 全部隐藏（不与 actor 同屏重复）")

	# actor 出场在 wait 0.6 之后——真实时间流逝轮询（帧数≠墙钟时间）
	var actor: Node = null
	var deadline := Time.get_ticks_msec() + 8000
	while actor == null and Time.get_ticks_msec() < deadline:
		actor = movie.call("get_actor", "xiao_master")
		await process_frame
	_check(actor != null, "演出 actor xiao_master 已实例化")
	if actor != null:
		_check(actor.is_visible(), "演出 actor 正常可见")

	# ---- 3. stop()：常驻 NPC 恢复可见 ----
	movie.call("stop")
	await _wait(5)
	var restored: bool = residents.all(func(n: Node) -> bool: return n.is_visible())
	_check(restored, "演出停止后常驻 NPC 恢复可见")
	movie.queue_free()

	# ---- 4. 点击寻路关闭（产品默认）----
	var from: Vector2i = map.world_to_cell(player.global_position)
	var target := Vector2i(-1, -1)
	for dx in range(1, 12):
		var c := from + Vector2i(dx, 0)
		if bool(map.is_walkable(c)):
			target = c
			break
	_check(target.x > from.x, "找到东侧可行走目标格 %s" % str(target))

	player.call("_on_map_clicked_at", map, target)
	await _wait(3)
	_check(not bool(player.call("is_grid_path_active")), "click_to_move=false：点击不产生移动")
	_check(map.get_node_or_null("PathPreview") == null, "click_to_move=false：无预览线")

	# ---- 5. 开关打开后机制仍在（回归能力保留）----
	player.set("click_to_move", true)
	player.call("_on_map_clicked_at", map, target)
	await _wait(3)
	_check(bool(player.call("is_grid_path_active")), "click_to_move=true：点击寻路生效")
	_check(map.get_node_or_null("PathPreview") != null, "click_to_move=true：预览线存在")
	player.call("stop")

	print("[MovieNpcDedup] %s" % ("ALL PASS" if ok else "FAIL"))
	_finish(0 if ok else 1)


# ---- 工具 ----

## 收集地图子树中的常驻 NPC（XiuNpcBase 实例）
func _npcs_in(map: Node) -> Array:
	var out: Array = []
	var stack: Array = [map]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children():
			if c is XiuNpcBase:
				out.append(c)
			stack.append(c)
	return out


func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _check(cond: bool, what: String) -> void:
	if cond:
		print("[MovieNpcDedup] PASS: ", what)
	else:
		ok = false
		print("[MovieNpcDedup] FAIL: ", what)


func _finish(code: int) -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(code)


func _wait(frames: int) -> void:
	for i in frames:
		await process_frame


func _poll(cond: Callable, max_frames: int) -> bool:
	for i in max_frames:
		if cond.call():
			return true
		await process_frame
	return cond.call()
