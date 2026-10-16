# 任务领奖弹窗验收（headless 手动跑）：
#   godot --headless --path . -s res://example/demo/xiuxian/check_reward_ui.gd
#
# 链路：complete_task 自动发奖写 last_reward → reward_panel watch 自动填充
# → TaskRewardModal 弹出（标题/任务名/奖励格子）→ 确认关闭 + 清空视图。
extends SceneTree

var ok := true


func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 60.0
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
	await _wait(10)

	var task := XiuTaskState.ins()
	task.reset_demo()

	_check(GdUIManager.has_ui("TaskRewardModal"), "TaskRewardModal 已注册在组合根")

	# ---- 完成任务（side_102：exp150 + 灵石20 + 雪参x2 = 3 格奖励）----
	_check(task.accept_task("side_102"), "接取 side_102")
	_check(task.complete_task("side_102"), "完成 side_102（自动发奖）")
	var items: Array = task.last_reward
	_check(items.size() == 3, "last_reward 3 条（修为/灵石/道具），实际 %d" % items.size())
	_check(str(task.last_reward_task).contains("采集") or task.last_reward_task != "",
			"last_reward_task 已写入: %s" % task.last_reward_task)
	_check(task.get_task_status("side_102") == "submitted", "状态直达 submitted")

	# ---- 弹窗自动弹出（watch deferred + 2 帧缓冲）----
	var modal: Node = GdUIManager.find_ui("TaskRewardModal")
	_check(modal != null, "find_ui 命中弹窗")
	var opened: bool = await _poll(func() -> bool:
		return modal != null and modal.is_modal_open(), 60)
	_check(opened, "complete_task 后弹窗自动打开")

	# ---- 奖励格子渲染 ----
	var panel: Node = _find_gml_panel(modal)
	_check(panel != null, "弹窗内容面板（reward_panel.gd）在树")
	if panel != null:
		var list: Node = _find_by_name(panel, "RewardList")
		_check(list != null, "奖励列表 RewardList 存在")
		var sub: Label = _find_by_name(panel, "SubtitleLabel") as Label
		_check(sub != null and str(sub.text) != "", "任务名副标题已填充: %s" % (sub.text if sub else ""))
		if list != null:
			# 条目 = RewardList 可见子节点（index 0 是 slot 模板，跳过；
			# duplicate 条目重名带 @ 后缀，不能按名找）
			var cells: Array = []
			var kids := list.get_children()
			for i in range(1, kids.size()):
				var c: Node = kids[i]
				if c is Control and (c as Control).visible:
					cells.append(c)
			_check(cells.size() == items.size(),
					"格子数 = 奖励数（%d/%d）" % [cells.size(), items.size()])
			var tips_bound := 0
			for c in cells:
				var label := _find_by_name(c, "CountLabel") as Label
				if label != null and str(label.text).begins_with("x"):
					tips_bound += 1
			_check(tips_bound == cells.size(), "数量角标全部渲染（x%d）" % tips_bound)
			# 悬停 tips：条目经 setup_hover 接到共享宿主（__GmlTipsHost）
			var host: Node = _find_by_name(panel, "__GmlTipsHost")
			_check(host != null, "共享 tips 宿主已创建（__GmlTipsHost）")
			# 修为格 tips 为固定说明
			var first: Dictionary = items[0]
			_check(str(first.get("tips", "")) != "", "奖励条目带用途说明")

	# ---- 确认关闭 ----
	var confirm: BaseButton = _find_by_name(panel, "ConfirmBtn") as BaseButton
	_check(confirm != null, "确认按钮存在")
	if confirm != null:
		confirm.pressed.emit()
		await _wait(3)
		_check(not modal.is_modal_open(), "确认后弹窗关闭")
		_check(task.last_reward.is_empty(), "确认后 last_reward 已清空")

	# ---- 幂等：重复 complete 同一任务不再弹（状态已 submitted 拒绝）----
	_check(not task.complete_task("side_102"), "重复 complete 被拒（防重弹）")

	# ---- 自动领奖：advance_task 终步达成 → 自动 complete_task（发奖+弹窗）----
	_check(task.accept_task("main_101"), "接取 main_101（steps=1）")
	var step: int = task.advance_task("main_101")
	_check(step == 1, "advance_task 推到终步（1/1）")
	var auto_done: bool = await _poll(func() -> bool:
		return task.get_task_status("main_101") == "submitted", 30)
	_check(auto_done, "终步达成自动领奖（无需手动点领取，直达 submitted）")
	var auto_opened: bool = await _poll(func() -> bool:
		return modal != null and modal.is_modal_open(), 60)
	_check(auto_opened, "自动领奖触发领奖弹窗再次打开")
	_check(not task.last_reward.is_empty(), "自动领奖后 last_reward 已写入（弹窗数据源）")

	print("[RewardUI] %s" % ("ALL PASS" if ok else "FAIL"))
	_finish(0 if ok else 1)


# ---- 工具 ----

func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _find_gml_panel(modal: Node) -> Node:
	# Modal ContentArea 下找挂 reward_panel.gd 的窗口根（class window-bg）
	var stack: Array = [modal]
	while not stack.is_empty():
		var n: Node = stack.pop_front()
		if n != modal and n is Panel and n.has_method("gml_tips_host"):
			return n
		for c in n.get_children():
			stack.append(c)
	return null


func _find_by_name(root: Node, target: String) -> Node:
	if root.name == target:
		return root
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_front()
		for c in n.get_children():
			if c.name == target:
				return c
			stack.append(c)
	return null


func _collect_cells(root: Node, cell_name: String) -> Array:
	var out: Array = []
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_front()
		if n != root and n.name == cell_name:
			out.append(n)
			continue
		for c in n.get_children():
			stack.append(c)
	return out


func _check(cond: bool, what: String) -> void:
	if cond:
		print("[RewardUI] PASS: ", what)
	else:
		ok = false
		print("[RewardUI] FAIL: ", what)


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
