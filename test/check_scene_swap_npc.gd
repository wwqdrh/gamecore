# 冒烟：经 GdSceneRoot.change_scene 进入主场景（实机 panic 路径——
# do_swap_scene 借用期内 map_restore→open_map→npc._ready 级联装配），
# 验证 NPC 日程时间源接线推迟后无 "already bound" 重入 panic。
extends SceneTree

const APP_SCENE := "res://example/demo/xiuxian/index.tscn"

func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 45.0
	wd.one_shot = true
	wd.timeout.connect(func() -> void: quit(2))
	root.add_child(wd)
	wd.start()

	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	var app: Node = load(APP_SCENE).instantiate()
	root.add_child(app)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_title", 300):
		print("[SmokeSwap] FAIL: 未进入 title")
		quit(2)
		return
	# 关键：这一次调用栈内会完成主场景装配（含 NPC _ready 级联）
	app.call("change_scene", "xiuxian_main", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_main", 600):
		print("[SmokeSwap] FAIL: 未切到 main")
		quit(2)
		return
	await _wait(10)  # 覆盖 call_deferred 的时间源接线帧

	var npcs := _find_npcs(app)
	print("[SmokeSwap] npc_count=", npcs.size())
	var wired := 0
	var scheds := 0
	var wm: Node = null
	for c in root.get_children():
		if c.get_class() == "GdWeatherManager":
			wm = c
	print("[SmokeSwap] wm_hour=", wm.get_hour() if wm != null else "<无 WeatherManager>")
	for npc in npcs:
		var sch: Node = npc.get_node_or_null("Schedule")
		if sch != null:
			scheds += 1
			var p: String = str(sch.get("time_source_path"))
			if not p.is_empty():
				wired += 1
			# 等过节流（0.25s）+ 若干 tick，再看实际应用状态
			await create_timer(1.2).timeout
			var st: Dictionary = sch.call("debug_state")
			print("[SmokeSwap] npc=%s path=%s state=%s pos=%s" % [
				npc.name, p, st, npc.global_position])
	print("[SmokeSwap] scheds=%d wired=%d" % [scheds, wired])
	var ok := scheds > 0 and wired == scheds
	print("[SmokeSwap] %s" % ("ALL PASS" if ok else "FAIL"))
	quit(0 if ok else 3)


func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _find_npcs(n: Node) -> Array:
	var out: Array = []
	if n.has_method("show_bubble"):
		out.append(n)
	for c in n.get_children():
		out.append_array(_find_npcs(c))
	return out


func _wait(frames: int) -> void:
	for i in frames:
		await process_frame


func _poll(cond: Callable, max_frames: int) -> bool:
	for i in max_frames:
		if cond.call():
			return true
		await process_frame
	return cond.call()
