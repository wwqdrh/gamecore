# XiuMovieState + MovieTrigger headless 集成测试（新档自动播开场剧情）
#
# 运行：perl -e 'alarm 150; exec @ARGV' godot --headless --path . \
#   -s res://test/test_movie_trigger.gd
#
# 覆盖：
#   1. 进入主场景（播放记录为空）→ MovieTrigger 自动开播 prologue
#      （MoviePlayer 在场、is_playing、玩家锁定）
#   2. s_movie_finished → XiuMovieState.mark_played 落盘（is_played=true）
#   3. 离开再进入主场景 → 不再触发（无 MoviePlayer、玩家不被锁定）
extends SceneTree

const APP_SCENE := "res://example/demo/xiuxian/index.tscn"

var ok := true


func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	wd.autostart = true
	wd.timeout.connect(func() -> void: quit(2))
	root.add_child(wd)

	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	var app: Node = load(APP_SCENE).instantiate()
	root.add_child(app)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_title", 300):
		_fail("未进入 title")
		await _finish(1)
		return

	# 播放记录清零（跨运行确定性：模拟新档/未看过剧情的存档）
	XiuMovieState.ins().reset_demo()

	app.call("change_scene", "xiuxian_main", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_main", 600):
		_fail("未切到 main")
		await _finish(1)
		return

	# ---- 1. 自动开播 ----
	var cur: Node = app.call("get_current_scene")
	var movie_ready: bool = await _poll(func() -> bool:
		var m: Node = cur.get_node_or_null("MoviePlayer")
		return m != null and bool(m.call("is_playing")), 300)
	_check(movie_ready, "进入主场景后 MovieTrigger 自动开播 prologue")
	if not movie_ready:
		await _finish(1)
		return

	var player: Node = cur.get_node("Player")
	_check(bool(player.call("is_paused")), "演出期玩家被锁定")
	_check(not XiuMovieState.ins().is_played("prologue"),
		"播放中尚未置位 played（中途退出会重播，符合预期）")

	# ---- 2. 播完落盘（prologue 全定时指令，等自然播完）----
	var done: bool = await _poll(func() -> bool:
		return XiuMovieState.ins().is_played("prologue"), 100000)
	_check(done, "s_movie_finished 后 played 已落盘")
	_check(not bool(player.call("is_paused")), "演出结束后玩家已解锁")
	var mp: Node = cur.get_node_or_null("MoviePlayer")
	_check(mp == null or not bool(mp.call("is_playing")), "播完后播放器不在播放中")

	# ---- 3. 重进主场景不再播放 ----
	app.call("change_scene", "xiuxian_title", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_title", 600):
		_fail("未切回 title")
		await _finish(1)
		return
	app.call("change_scene", "xiuxian_main", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_main", 600):
		_fail("再次未切到 main")
		await _finish(1)
		return
	var cur2: Node = app.call("get_current_scene")
	# 装配 + 触发器 deferred 窗口内给足帧数，确认不触发
	for i in 120:
		await process_frame
	_check(cur2.get_node_or_null("MoviePlayer") == null, "重进后无 MoviePlayer（未触发）")
	_check(not bool(cur2.get_node("Player").call("is_paused")), "重进后玩家未被锁定")

	print("[MovieTriggerTest] %s" % ("ALL PASS" if ok else "FAIL"))
	await _finish(0 if ok else 1)


func _finish(code: int) -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(code)


func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _check(cond: bool, what: String) -> void:
	if not cond:
		ok = false
		print("[MovieTriggerTest] FAIL: ", what)
	else:
		print("[MovieTriggerTest] PASS: ", what)


func _fail(what: String) -> void:
	print("[MovieTriggerTest] FAIL: ", what)
	ok = false


func _poll(cond: Callable, max_frames: int) -> bool:
	for i in max_frames:
		if cond.call():
			return true
		await process_frame
	return cond.call()
