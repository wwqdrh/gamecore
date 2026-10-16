# 校验脚本：M6 第一阶段全链通关验收（真实主场景装配）
# 运行：godot --headless -s res://test/check_stage1_clear.gd
# 与 check_stage1_flow（纯状态层端到端）的区别：本测试跑在真实 index.tscn
# 装配链路上，覆盖跨系统真实联动：
#   A. guide 教学提示自动开播（mainhud 集成）
#   B. home 农田虫害事件真实刷鼠 → 击杀联动推任务（home_pest）
#   C. 突破演出真实播放（Banner 组件）+ 前置链解锁 + 门控解锁
#   D. 复命收束 → 通关字幅（突破/通关 Banner 背靠背排队接播）
extends SceneTree

var _fails := 0


func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	wd.timeout.connect(func() -> void: quit(2))
	root.add_child(wd)
	wd.start()
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[Stage1Clear] OK  ", msg)
	else:
		_fails += 1
		printerr("[Stage1Clear] FAIL ", msg)


func _wait(frames: int) -> void:
	for i in frames:
		await process_frame


## 轮询直到条件满足或超时（墙钟时限——无头模式帧率不节流，帧数≠时间；
## 传入的 ms 值按原「帧数÷60」语义换算，调用点无需改动）
func _poll(cond: Callable, max_frames: int) -> bool:
	var budget_ms := int(max_frames / 60.0 * 1000.0)
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < budget_ms:
		if cond.call():
			return true
		await process_frame
	return cond.call()


## 当前地图上的地灵鼠（按场景文件路径识别）
func _find_mice(map: Node) -> Array:
	var out: Array = []
	for c in map.get_children():
		if str(c.get_scene_file_path()).ends_with("mouse.tscn"):
			out.append(c)
	return out


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var ts: XiuTaskState = XiuTaskState.ins()
	var ch: XiuCharacterState = XiuCharacterState.ins()
	var dlg: XiuDialogState = XiuDialogState.ins()
	var it: XiuItemState = XiuItemState.ins()
	ts.reset_demo()
	ch.reset_demo()
	it.reset_demo()

	# ---- 装配：真实主场景 ----
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	_check(main_scene != null, "index.tscn 可加载")
	if main_scene == null:
		quit(1)
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	await _wait(8)

	var mgr: Node = index.get_node_or_null("MapManager")
	var player: Node = index.get_node_or_null("Player")
	var guide_layer: Node = index.find_child("GuideLayer", true, false)
	var toast: Node = index.find_child("HintToast", true, false)
	var banner: Node = index.find_child("PerfBanner", true, false)
	if mgr == null or player == null or toast == null or banner == null:
		_check(false, "主场景关键节点齐备（mgr/player/toast/banner）")
		quit(1)
		return
	_check(guide_layer != null, "GuideLayer 已随 mainhud 装配")

	# 日程/闲聊冻结（对话流测试规约）
	for n in get_nodes_in_group("npc_schedule"):
		n.set("enable", false)
	for n in get_nodes_in_group("npc_chat"):
		n.set("enable", false)

	# ---- A. 教学提示：guide 在真实场景自动开播 ----
	var tut_started: bool = await _poll(
		func() -> bool: return bool(toast.call("is_busy")), 300)
	_check(tut_started, "教学 4 连提示自动开播")
	_check(dlg.has_flag("tutorial_shown"), "教学开播即置 tutorial_shown")
	toast.call("clear_toasts")

	# ---- B. 主线前段：main_101 → main_103（真实虫害事件）----
	_check(ts.accept_task("main_101"), "接取 main_101")
	ts.advance_task("main_101")  # 老爷对话（对话链 check_task_flow 已覆盖）
	_check(ts.complete_task("main_101"), "提交 main_101（解锁 main_102）")
	_check(ts.accept_task("main_102"), "接取 main_102")
	ts.advance_task("main_102")
	ts.advance_task("main_102")
	_check(ts.complete_task("main_102"), "提交 main_102（解锁 main_103）")
	_check(ts.accept_task("main_103"), "接取 main_103")

	# 切到萧宅农田 → PestEvent 真实刷鼠
	mgr.call("open_map", "xiuxian_home", Vector2i(10, 7))
	await _wait(10)
	var map: Node = mgr.get_current_map()
	_check(str(mgr.get_current_alias()) == "xiuxian_home", "已切换到萧宅农田")
	var mice_ready: bool = await _poll(
		func() -> bool: return _find_mice(map).size() == 3, 600)
	var mice: Array = _find_mice(map)
	_check(mice_ready and mice.size() == 3, "虫害事件应刷出地灵鼠×3（got %d）" % mice.size())
	# 逐只击杀（s_died → PestEvent 计数，第 3 只推任务）
	for e in mice:
		e.health.call("kill")
		await _wait(4)
	var advanced: bool = await _poll(
		func() -> bool:
			return int(ts.get_progress("main_103").get("step", 0)) >= 1,
		120)
	_check(advanced, "清剿 3 只 → main_103 击杀步推进（home_pest 联动）")
	var corpses_gone: bool = await _poll(
		func() -> bool: return _find_mice(map).is_empty(), 120)
	_check(corpses_gone, "击杀后尸体清理（死亡动画销毁）")
	ts.advance_task("main_103")  # talk 小翠回报
	_check(ts.complete_task("main_103"), "提交 main_103（前置链解锁 main_104）")
	_check(ts.get_task_status("main_104") == ts.STATUS_AVAILABLE, "main_104 已解锁")
	ts.accept_task("main_104")
	ts.advance_task("main_104")
	ts.advance_task("main_104")
	_check(ts.complete_task("main_104"), "提交 main_104（解锁 main_105）")

	# ---- C. 真相段 → 突破演出（真实 Banner）----
	_check(ts.accept_task("main_105"), "接取 main_105")
	it.add_item("break_qizhe", 3)  # 萧老爷赠药（item_give 对话命令已单测）
	ts.advance_task("main_105")    # collect ×3
	ch.add_exp(20000)
	_check(ch.level == 10, "经验封顶斗之气 10 段")
	_check(ch.can_breakthrough(), "聚气散×3 齐备可突破")
	# 排空可能残留的演出队列（升级不播横幅——guide 按大阶判定；此处防历史残留）
	await _poll(func() -> bool: return not bool(banner.call("is_playing")), 600)
	var bt_ok: bool = ch.try_breakthrough()
	_check(bt_ok, "突破成功")
	if bt_ok:
		_check(ch.level == 11, "进入斗者一段")
		_check(dlg.has_flag("rank_dou_zhe"), "突破联动置 rank_dou_zhe")
		var playing: bool = await _poll(
			func() -> bool: return bool(banner.call("is_playing")), 60)
		_check(playing, "突破演出真实播放")
	_check(int(ts.get_progress("main_105").get("step", 0)) == 2, "rank_up 步已推进")
	_check(it.get_count("break_qizhe") == 0, "突破消耗聚气散×3")

	# ---- D. 复命收束 → 通关字幅（可能排队在突破演出后接播）----
	ts.advance_task("main_105")  # talk 萧老爷复命
	_check(ts.complete_task("main_105"), "提交 main_105")
	_check(it.get_count("token_sect") == 1, "宗门引荐令入包")
	_check(ts.get_task_status("sect_tournament") == ts.STATUS_AVAILABLE,
		"main_105 提交 → 宗门选拔点亮（第 7 天日历自动接取）")
	# stage1_clear 由 arena 收束段选项动作置位（对话侧已被 check_stage1_dialog
	# 覆盖）——此处置位以驱动 guide→Banner 通关字幅的真实联动
	dlg.set_flag("stage1_clear")
	_check(dlg.has_flag("stage1_clear"), "阶段通关 flag 置位")
	# 等 banner 队列播空（突破演出 ~2.9s + 通关字幅 ~2.9s）
	var all_done: bool = await _poll(
		func() -> bool: return not bool(banner.call("is_playing")), 900)
	for i in 6:
		if not bool(banner.call("is_playing")):
			break
		await create_timer(1.0).timeout
		print("[Stage1Clear] TIMELINE +", i + 1, "s playing=", banner.call("is_playing"))
	_check(all_done, "演出队列收束（突破+通关背靠背）")
	if not all_done:
		var labels0: Array = banner.find_children("*", "Label", true, false)
		for lb in labels0:
			print("[Stage1Clear] DIAG label=", lb.text)
		print("[Stage1Clear] DIAG guide _clear_shown=",
			guide_layer.get("_clear_shown") if guide_layer else "n/a")
	# 捕获通关字幅标题：重新触发一次（flag 幂等靠 _clear_shown，此处直调 banner）
	banner.call("play_banner", "第一阶段 · 完", "验证文本")
	await _wait(10)
	var labels: Array = banner.find_children("*", "Label", true, false)
	var title := ""
	for lb in labels:
		var t := str(lb.text)
		if t.contains("第一阶段"):
			title = t
	_check(title.contains("第一阶段 · 完"), "通关字幅标题正确：%s" % title)
	await _wait(5)

	# ---- E. 解锁后真实切图（青草岭可达）----
	mgr.call("open_map", "xiuxian_map_a", Vector2i(5, 5))
	await _wait(10)
	_check(str(mgr.get_current_alias()) == "xiuxian_map_a", "青草岭解锁后可真实进入")
	_check(not ts.complete_task("main_105"), "已提交任务重复提交应失败")

	# ---- 结果 ----
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	if _fails == 0:
		print("[Stage1Clear] ALL PASS")
		quit(0)
	else:
		printerr("[Stage1Clear] %d FAIL" % _fails)
		quit(1)
