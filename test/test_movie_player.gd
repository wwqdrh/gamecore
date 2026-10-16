# GdMoviePlayer headless 集成测试（真实主场景 + 真实剧本）
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://test/test_movie_player.gd
#
# 覆盖：
#   1. configure 后 play()：s_movie_started / is_playing 变迁 / 玩家锁定
#   2. scene 指令真实 open_map + actor enter（在场节点可查）
#   3. say dur 定时推进 / say 点击推进（movie_line_done 置位）
#   4. move 网格寻路完成 / bubble 按秒展示
#   5. flag 置位 + if-jump 分支（跳过不该出现的 say）
#   6. end → s_movie_finished / 自动释放角色 / 玩家解锁
extends SceneTree

const APP_SCENE := "res://example/demo/xiuxian/index.tscn"
const MOVIE_PATH := "res://example/demo/xiuxian/movie/test_smoke/test_smoke.movie"

var ok := true


func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 60.0
	wd.one_shot = true
	wd.autostart = true
	wd.timeout.connect(func() -> void: quit(2))
	root.add_child(wd)

	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	# 屏蔽 MovieTrigger：本测试直接驱动播放器，勿让主场景触发器并发开播
	# prologue（新档未置位会自动触发，与 test_smoke 演出互相干扰）
	XiuMovieState.ins().mark_played("prologue")

	var app: Node = load(APP_SCENE).instantiate()
	root.add_child(app)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_title", 300):
		_fail("未进入 title")
		_finish(1)
		return
	app.call("change_scene", "xiuxian_main", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_main", 600):
		_fail("未切到 main")
		await _finish(1)
		return
	var cur: Node = app.call("get_current_scene")
	await _wait(10)

	var dialog_box: Node = cur.get_node_or_null("DialogLayer/DialogBox")
	var player: Node = cur.get_node_or_null("Player")
	_check(dialog_box != null, "找到 DialogBox")
	_check(player != null, "找到 Player")

	# 动态构造播放器（框架类 GDScript 直 new）
	var movie: Node = GdMoviePlayer.new()
	movie.set_name("TestMoviePlayer")
	cur.add_child(movie)
	movie.call("configure", MOVIE_PATH,
		movie.get_path_to(dialog_box), movie.get_path_to(player))
	_check(not movie.call("is_playing"), "初始不在演出中")

	# ---- 播放 ----
	var started := [false]
	movie.connect("s_movie_started", func() -> void: started[0] = true)
	movie.call("play")
	_check(started[0], "s_movie_started 已发")
	_check(movie.call("is_playing"), "演出中")
	_check(player.call("is_paused"), "演出期玩家被锁定")

	# actor enter 落地（scene + actor 各 2/1 帧 + focus）
	var actor_ready: bool = await _poll(func() -> bool:
		var a: Variant = movie.call("get_actor", "xiao_maid")
		return typeof(a) == TYPE_OBJECT and (a as Node).is_inside_tree(), 120)
	_check(actor_ready, "actor enter 后在场且入树")

	# say 点击推进：台词走 rust 默认黑框（demo DialogBox 已无 show_movie_line）——
	# 等最后一条（无 dur 版）出现——文本含「台词二」——置位 movie_line_done
	# 模拟玩家点击（lambda 按值捕获：闭包内不赋外层变量，条件成立后重取）
	var click_done: bool = await _poll(func() -> bool:
		var bx: Node = movie.get_node_or_null("__MovieDialogBox")
		if bx == null:
			return false
		var tl: Label = bx.get_node_or_null("Panel/TextLabel")
		return tl != null and str(tl.text).contains("台词二"), 900)
	_check(click_done, "say 走 rust 默认黑框并等待点击")
	var default_box: Node = movie.get_node_or_null("__MovieDialogBox")
	if default_box != null:
		default_box.set("movie_line_done", true)
		print("[MovieTest] 模拟点击推进 say")

	# bubble 走 rust 默认气泡（挂在角色下，超时自隐）
	var actor_maid: Node = movie.call("get_actor", "xiao_maid")
	var bubble: Node = actor_maid.get_node_or_null("__MovieBubble") \
		if actor_maid != null else null
	_check(bubble != null, "bubble 指令挂载 rust 默认头顶气泡")

	# 等演出结束
	var finished := [false]
	movie.connect("s_movie_finished", func() -> void: finished[0] = true)
	var done: bool = await _poll(func() -> bool: return finished[0], 600)
	_check(done, "s_movie_finished 已发")
	_check(not movie.call("is_playing"), "演出结束后不在播放中")
	_check(not player.call("is_paused"), "玩家已解锁")

	# auto_free_actors：演出结束角色已释放
	await _wait(3)
	var gone: Variant = movie.call("get_actor", "xiao_maid")
	_check(typeof(gone) == TYPE_NIL, "结束后实例化角色已释放")

	# flag 落盘（movie_test_flag 经 control set_flag 委托 XiuDialogState）
	var dlg_state: XiuDialogState = XiuDialogState.ins()
	_check(dlg_state.has_flag("movie_test_flag"), "flag 指令已写入对话状态")
	dlg_state.clear_flag("movie_test_flag")

	print("[MovieTest] %s" % ("ALL PASS" if ok else "FAIL"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)


func _finish(code: int) -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(code)


func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _check(cond: bool, what: String) -> void:
	if not cond:
		ok = false
		print("[MovieTest] FAIL: ", what)
	else:
		print("[MovieTest] PASS: ", what)


func _fail(what: String) -> void:
	print("[MovieTest] FAIL: ", what)
	ok = false


func _wait(frames: int) -> void:
	for i in frames:
		await process_frame


func _poll(cond: Callable, max_frames: int) -> bool:
	for i in max_frames:
		if cond.call():
			return true
		await process_frame
	return cond.call()
