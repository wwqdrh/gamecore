# 路径预览残线修复验收（headless 手动跑）：
#   godot --headless --path . -s res://example/demo/xiuxian/check_path_preview.gd
#
# 背景 bug：预览折线曾挂在玩家身上且路径点存世界坐标——寻路被键盘打断时
# Rust 侧只清 grid_path_active 不发 s_grid_path_finished，残线永远跟着玩家走
# （用户截图中的"黄色边框跟着玩家动"）。
#
# 修复：预览线挂当前地图根（单位变换，世界坐标恒定）；打断/失效由
# _process 按 is_grid_path_active 兜底清除。
#
# headless 无鼠标：直调 _on_map_clicked_at 绕开鼠标（红线备忘）；
# stop() 与键盘打断的观测效果等价（路径失活且不发 finished 信号）。
extends SceneTree

var ok := true


func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 90.0
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
		await _finish(1)
		return
	app.call("change_scene", "xiuxian_main", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_main", 600):
		_check(false, "未切到 main")
		await _finish(1)
		return

	var player: Node = get_first_node_in_group("player")
	_check(player != null, "player 在场")
	if not await _poll(func() -> bool: return bool(player.get("_wired")), 600):
		_check(false, "player 接线完成")
		await _finish(1)
		return
	# 点击寻路产品级默认关闭（click_to_move=false）——本验收专测预览线机制，
	# 临时打开（不落盘，场景实例属性）
	player.set("click_to_move", true)
	var map_mgr: Node = player.get("map_mgr")

	# ---- 1. 切到小镇图（GdQuickMap，支持 find_path）----
	_check(map_mgr.open_map("xiuxian_xiaozhen", Vector2i(5, 5)), "开小镇图")
	if not await _poll(func() -> bool:
		return str(map_mgr.get_current_alias()) == "xiuxian_xiaozhen", 300):
		_check(false, "小镇图已开")
		await _finish(1)
		return
	await _wait(10)  # player._apply_map（s_map_changed → 重绑网格/相机/预览线置空）

	var map: Node = map_mgr.get_current_map()
	_check(map.has_method("find_path"), "小镇图为 QuickMap（支持 find_path）")

	# ---- 2. 直调点击寻路：预览线应挂地图根（世界坐标恒定）----
	var from: Vector2i = map.world_to_cell(player.global_position)
	var target := Vector2i(-1, -1)
	for dx in range(1, 12):
		var c := from + Vector2i(dx, 0)
		if bool(map.is_walkable(c)):
			target = c
			break
	_check(target.x > from.x, "找到东侧可行走目标格 %s" % str(target))
	player.call("_on_map_clicked_at", map, target)
	var line: Line2D = map.get_node_or_null("PathPreview")
	_check(line != null, "预览线挂在地图根下")
	if line != null:
		_check(line.get_parent() != player, "预览线不在玩家下（不随玩家漂移）")
		_check(line.get_point_count() > 0, "预览线含路径点")

	# ---- 3. 模拟打断（等价键盘打断：路径失活但不发 finished 信号）----
	player.call("stop")
	await _wait(3)
	if line != null:
		_check(line.get_point_count() == 0, "打断后残线被 _process 兜底清除")

	# ---- 4. 瞄准框场景策略：主场景/家园关闭（秘境 dungeon 图才开）----
	var shooter: Node = player.get("shooter")
	_check(shooter != null, "player 挂载 shooter")
	if shooter != null:
		_check(bool(shooter.get("show_aim_frame")) == false,
			"主场景（非秘境）瞄准框应关闭")

	print("[PathPreview] %s" % ("ALL PASS" if ok else "FAIL"))
	await _finish(0 if ok else 1)


# ---- 工具 ----

func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _check(cond: bool, what: String) -> void:
	if cond:
		print("[PathPreview] PASS: ", what)
	else:
		ok = false
		print("[PathPreview] FAIL: ", what)


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
