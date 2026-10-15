# 校验脚本：M5 引导与演出（Toast 提示条 / Banner 演出字幅 / mainhud 集成）
# 运行：godot --headless -s res://test/check_stage1_guide.gd
# 覆盖：
#   1) GdUIToast：队列逐条播放 / s_toast_shown×N / s_toast_done 恰一次 /
#      播完隐藏 / clear_toasts 立即收束
#   2) GdUIBanner：播放中可见+STOP 挡输入 / 收束信号 / 回隐藏+IGNORE /
#      播放中重复调用忽略
#   3) mainhud 集成：GuideLayer 装配（Guide 脚本取到组件）/ 教学 4 连提示
#      门控（tutorial_shown flag）/ _on_portal_locked → Toast
extends SceneTree

var _fails := 0
var _deadline := 0


func _initialize() -> void:
	_deadline = Time.get_ticks_msec() + 30000
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[Guide] OK  ", msg)
	else:
		_fails += 1
		printerr("[Guide] FAIL ", msg)


func _wait_until(busy_cb: Callable, timeout_ms: int) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < timeout_ms:
		if not busy_cb.call():
			return true
		if Time.get_ticks_msec() > _deadline:
			return false
		await process_frame
	return not busy_cb.call()


func _run() -> void:
	# 存档隔离
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	# ---- 1. Toast 组件 ----
	var toast: Control = GdUIToast.new()
	root.add_child(toast)
	for i in 3:
		await process_frame
	var shown: Array = []
	var done_arr: Array = []  # GDScript lambda 按值捕获标量——用数组计数
	toast.connect("s_toast_shown", func(t: String) -> void: shown.append(t))
	toast.connect("s_toast_done", func() -> void: done_arr.append(1))
	toast.set("fade_secs", 0.05)
	toast.call("show_toast_secs", "甲", 0.1)
	toast.call("show_toast_secs", "乙", 0.1)
	toast.call("show_toast_secs", "丙", 0.1)
	_check(toast.call("is_busy"), "入队 3 条后 is_busy")
	_check(toast.visible, "播放中可见")
	var ok := await _wait_until(func() -> bool: return toast.call("is_busy"), 5000)
	_check(ok, "队列在超时前播完")
	_check(shown.size() == 3, "逐条 s_toast_shown ×3（got %d）" % shown.size())
	_check(shown[0] == "甲" and shown[2] == "丙", "按入队顺序播放")
	_check(done_arr.size() == 1, "队列播空 s_toast_done 恰一次（got %d）" % done_arr.size())
	_check(not toast.visible, "播完自动隐藏")

	# clear_toasts 立即收束
	toast.call("show_toast", "长提示")
	toast.call("clear_toasts")
	_check(not toast.call("is_busy"), "clear 后 is_busy=false")
	_check(not toast.visible, "clear 后立即隐藏")

	# ---- 2. Banner 组件 ----
	var banner: Control = GdUIBanner.new()
	banner.set("hold_secs", 0.1)
	banner.set("anim_secs", 0.1)
	root.add_child(banner)
	for i in 3:
		await process_frame
	var fin_arr: Array = []
	banner.connect("s_banner_finished", func() -> void: fin_arr.append(1))
	banner.call("play_banner", "突破成功", "气感圆融")
	_check(banner.call("is_playing"), "播放中 is_playing")
	_check(banner.visible, "播放中可见")
	_check(int(banner.get("mouse_filter")) == 0, "播放期 STOP 挡输入")
	# 播放中再次调用 → 排队，收束后无缝接播
	banner.call("play_banner", "第二幕", "排队播放")
	ok = await _wait_until(func() -> bool: return banner.call("is_playing"), 5000)
	_check(ok, "两条演出在超时前收束")
	_check(fin_arr.size() == 2, "排队接播，s_banner_finished 各发一次（got %d）" % fin_arr.size())
	_check(not banner.visible, "收束后隐藏")
	_check(int(banner.get("mouse_filter")) == 2, "收束后回 IGNORE 穿透")

	# ---- 3. mainhud 集成 ----
	XiuCharacterState.ins().reset_demo()
	XiuDialogState.ins().reset_demo()
	var hud_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/ui/mainhud/mainhud.gml.tscn")
	_check(hud_scene != null, "mainhud.gml.tscn 可加载")
	if hud_scene != null:
		var holder := Control.new()
		holder.size = Vector2(1152, 648)
		root.add_child(holder)
		var hud: Control = hud_scene.instantiate()
		holder.add_child(hud)
		for i in 8:
			await process_frame
		var layer: Control = hud.find_child("GuideLayer", true, false)
		_check(layer != null, "GuideLayer 已装配")
		var guide: Control = null
		var t_node: Control = hud.find_child("HintToast", true, false)
		var b_node: Control = hud.find_child("PerfBanner", true, false)
		_check(t_node != null and t_node.get_class() == "GdUIToast", "HintToast 为 GdUIToast")
		_check(b_node != null and b_node.get_class() == "GdUIBanner", "PerfBanner 为 GdUIBanner")
		if layer != null:
			guide = layer
			_check(guide.get("_toast") == t_node, "Guide 脚本经 get_gml_node 取到 Toast")
			_check(guide.get("_banner") == b_node, "Guide 脚本经 get_gml_node 取到 Banner")
		# 教学：装配后自动触发（无 met flag）→ 4 条入队 + flag 置位
		var dlg := XiuDialogState.ins()
		_check(dlg.has_flag("tutorial_shown"), "教学开播即置 tutorial_shown")
		_check(t_node.call("is_busy"), "教学 4 连提示入队播放中")
		# 门控提示：直调 handler（真实信号接线在进游戏后由 player 触发）
		t_node.call("clear_toasts")
		if guide != null:
			guide.call("_on_portal_locked", "xiuxian_map_a")
			_check(t_node.call("is_busy"), "portal_locked → Toast 提示")
			t_node.call("clear_toasts")
		# 重播守卫：flag 已置，再触发教学不重复
		var shown_before := 0
		t_node.connect("s_toast_shown", func(_t: String) -> void: shown_before += 1)
		if guide != null:
			guide.call("_start_tutorial")
		for i in 3:
			await process_frame
		_check(shown_before == 0, "flag 已置时教学不重放")
		holder.queue_free()
		for i in 3:
			await process_frame

	# ---- 结果 ----
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	if _fails == 0:
		print("[Guide] ALL PASS")
		quit(0)
	else:
		printerr("[Guide] %d FAIL" % _fails)
		quit(1)
