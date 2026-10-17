# 控制台连播验收（headless 手动跑）：
#   godot --headless --path . -s res://example/demo/xiuxian/check_console_index.gd
#
# 链路：index.tscn 根挂 index.gd → _ready 注册 GdConsole 命令
#   · play_all_movies() / stop_all_movies() 已注册（list_commands 可见）
#   · GdConsole.console_print（rust 新增）经 console_output 信号显示到面板
#   · _scan_movie_scripts 扫描 movie/ 目录：23 段、叙事排序、排除 test_smoke
#   · title 场景无玩家 → 命令优雅拒绝
#   · 主场景内启动连播 → GdMoviePlayer 开播 → stop_all_movies 中止清理
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
	# 屏蔽 MovieTrigger 自动补播 prologue，避免与连播测试并发
	XiuMovieState.ins().mark_played("prologue")

	var console: Object = Engine.get_singleton("GdConsole")

	# ---- 进面板前：console_print 信号链路（rust 新增能力） ----
	var outputs: Array = []
	console.console_output.connect(func(text: String) -> void: outputs.append(text))

	var app: Node = load("res://example/demo/xiuxian/index.tscn").instantiate()
	root.add_child(app)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_title", 300):
		_check(false, "未进入 title")
		_finish(1)
		return
	await _wait(5)

	# ---- 命令注册 ----
	var listed: PackedStringArray = console.list_commands()
	_check(str(listed).contains("play_all_movies"), "play_all_movies 已注册")
	_check(str(listed).contains("stop_all_movies"), "stop_all_movies 已注册")

	# ---- console_print：GDScript → 控制台面板输出（deferred，等到达）----
	app.call("_console_print", "probe_console_print_xyz")
	var probe: bool = await _poll(func() -> bool:
		return outputs.any(func(t: String) -> bool: return t.contains("probe_console_print_xyz")), 30)
	_check(probe, "console_print 经 console_output 信号送达")

	# ---- 剧本扫描与排序 ----
	var scripts: Array = app.call("_scan_movie_scripts")
	_check(scripts.size() == 23, "扫描到 23 段剧本（prologue+c01..c20+breakthrough+selection_eve），实际 %d" % scripts.size())
	if scripts.size() > 0:
		_check(str(scripts[0]).contains("prologue"), "首段为 prologue")
	if scripts.size() > 1:
		_check(str(scripts[1]).contains("c01_farewell"), "第二段为 c01_farewell")
	if scripts.size() > 2:
		_check(str(scripts[2]).contains("c02_gate_arrival"), "第三段为 c02_gate_arrival")
	_check(scripts.all(func(p: String) -> bool: return not p.contains("test_smoke")),
			"test_smoke 已排除")
	_check(scripts.any(func(p: String) -> bool: return p.contains("selection_eve")),
			"selection_eve 在列（排尾）")

	# ---- title 场景无玩家：命令优雅拒绝（走 execute 真实链路，
	#      覆盖「execute 同步回调内再触 GdConsole」的借用重入路径）----
	var refused: String = console.execute("play_all_movies()")
	_check(str(refused).contains("玩家节点"), "title 场景无玩家 → 拒绝连播: %s" % refused)
	_check(not app.get("_playing_all"), "拒绝后未进入连播态")

	# ---- 切主场景：execute 启动连播 → 开播 → execute 中止 ----
	app.call("change_scene", "xiuxian_main", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_main", 600):
		_check(false, "未切到 main")
		await _finish(1)
		return
	await _wait(10)

	var started: String = console.execute("play_all_movies()")
	_check(str(started).contains("开始连播"), "主场景内启动连播: %s" % started)
	var playing: bool = await _poll(func() -> bool:
		var cur = app.call("_get_current_movie")
		return cur != null and cur.call("is_playing"), 600)
	_check(playing, "连播第一段 GdMoviePlayer 已开播")
	# deferred console_print 应在 execute 返回后送达（直调会借用重入 panic）
	var progress: bool = await _poll(func() -> bool:
		return outputs.any(func(t: String) -> bool: return t.contains("播放 prologue")), 120)
	_check(progress, "连播进度经 deferred console_print 送达面板")

	var stopped: String = console.execute("stop_all_movies()")
	_check(str(stopped).contains("已停止"), "stop_all_movies 中止: %s" % stopped)
	_check(not app.get("_playing_all"), "中止后退出连播态")
	_check(app.call("_get_current_movie") == null, "当前播放器引用已清空")
	await _wait(5)

	# ---- 未污染播放记录（连播不写 XiuMovieState） ----
	_check(not XiuMovieState.ins().is_played("c01_farewell"), "连播不写播放记录")

	print("[ConsoleIndex] %s" % ("ALL PASS" if ok else "FAIL"))
	_finish(0 if ok else 1)


# ---- 工具 ----

func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _check(cond: bool, what: String) -> void:
	if cond:
		print("[ConsoleIndex] PASS: ", what)
	else:
		ok = false
		print("[ConsoleIndex] FAIL: ", what)


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
