# 任务系统 + 对话进度端到端验收 —— flag 门控 / 最近触发 / 任务奖励闭环
#
# 运行：perl -e 'alarm 150; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_task_flow.gd
#
# 覆盖：
#   1. 最近触发：两个 NPC 触发圈重叠时，只有离玩家最近的触发器能启动对话
#   2. 对话进度：萧老爷初见段（置 met_xiao_master）只进一次，
#      再次对话自动跳到任务可接段（flag 门控 + start_from 回退选段）
#   3. 任务接取：选项「有什么能帮忙的吗」→ 交代差事 → @task_accept:side_003
#   4. 任务完成：与小翠对话进入任务段 → @task_complete:side_003 →
#      奖励 50 金币入账（XiuCharacterState.coins），状态 submitted
#   5. 任务关闭：完成后萧老爷对话不再出现任务可接段（日常再访段）
#   6. 幂等：已提交任务重复 complete 返回 false；重置后可重复演示
extends SceneTree

var ok := true


func fail(msg: String) -> void:
	push_error("[Task] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


## 推进一屏：选项等待时点含 pick 文本的按钮（空 = 首个），普通行走 _advance
func advance_once(dialogue: Node, dialog_box: Node, pick: String = "") -> void:
	await process_frame
	if not dialogue.is_playing():
		return
	if dialog_box.responding:
		var chosen := 0
		for i in dialog_box.choices_box.get_child_count():
			var b: Button = dialog_box.choices_box.get_child(i)
			if pick != "" and b.text.contains(pick):
				chosen = i
				break
		var btn: Button = dialog_box.choices_box.get_child(chosen)
		btn.pressed.emit()
	else:
		dialog_box._advance()
	await process_frame
	await process_frame


## 推进到对话结束（兜底）
func advance_to_end(dialogue: Node, dialog_box: Node) -> void:
	var guard := 0
	while dialogue.is_playing() and guard < 80:
		guard += 1
		await advance_once(dialogue, dialog_box)


func _run() -> void:
	# 0. 状态基线
	var tasks: XiuTaskState = XiuTaskState.ins()
	var ch: XiuCharacterState = XiuCharacterState.ins()
	var dlg: XiuDialogState = XiuDialogState.ins()
	tasks.reset_demo()
	ch.reset_demo()
	check(XiuTaskTable.count() == 9, "任务定义表应有 9 条（8 迁移 + side_003），实际 %d" % XiuTaskTable.count())

	# 1. 装配
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	for i in 6:
		await process_frame

	var mgr: Node = index.get_node_or_null("MapManager")
	var dialog_layer: Node = index.get_node_or_null("DialogLayer")
	var dialogue: Node = dialog_layer.get_node_or_null("Dialogue")
	var dialog_box: Node = dialog_layer.get_node_or_null("DialogBox")
	check(dialogue != null and dialog_box != null, "DialogLayer 应装配 Dialogue + DialogBox")
	if mgr == null or dialogue == null or dialog_box == null:
		_finish()
		return
	check(str(mgr.get_current_alias()) == "xiuxian_xiaozhai", "应默认加载萧宅")
	var map: Node = mgr.get_current_map()
	var player: Node = index.get_node_or_null("Player")

	var npcs := {}
	for child in map.get_children():
		if child.get_class() == "GdRoleMover":
			npcs[str(child.role_name)] = child
	for i in 5:
		await process_frame
	check(npcs.size() == 3, "萧宅应有 3 个 NPC，实际 %d" % npcs.size())

	var master: Node = npcs.get("萧老爷")
	var lady: Node = npcs.get("萧夫人")
	var maid: Node = npcs.get("小翠")
	if master == null or lady == null or maid == null or player == null:
		_finish()
		return
	var master_trigger: Node = master.get_node("Trigger")
	var lady_trigger: Node = lady.get_node("Trigger")
	var maid_trigger: Node = maid.get_node("Trigger")

	# ---- 2. 最近触发 ----
	var cs: int = map.get_cell_size_px()
	check(master_trigger.is_in_group("dialog_trigger"), "触发器应加入 dialog_trigger 分组")
	# 玩家放两者之间、离夫人更近：老爷(9,6) 夫人(12,7)，玩家 (11,6)
	player.global_position = Vector2(11.5, 6.5) * cs
	for i in 6:
		await physics_frame
	var started_far: bool = master_trigger.start_dialog()
	check(not started_far, "较远的萧老爷触发应让位给更近的萧夫人")
	var started_near: bool = lady_trigger.start_dialog()
	check(started_near, "更近的萧夫人触发应成功")
	if started_near:
		check(lady_trigger.is_dialog_active() and dialogue.is_playing(), "夫人对话应播放中")
		lady_trigger.cancel_dialog()
		check(not lady_trigger.is_dialog_active(), "cancel_dialog 应复位")
		check(not dialogue.is_playing(), "cancel_dialog 应停掉共享 Dialogue 播放")
	print("[Task] nearest: far_rejected=%s near_started=%s" % [not started_far, started_near])

	# ---- 3. 对话进度 + 任务接取（全新进度）----
	tasks.reset_demo()
	ch.reset_demo()
	check(not dlg.has_flag("met_xiao_master"), "重置后 met_xiao_master 应为 false")
	player.global_position = Vector2(11.5, 10.5) * cs
	for i in 3:
		await physics_frame
	check(master_trigger.start_dialog(), "萧老爷对话应可触发")
	check(dlg.has_flag("met_xiao_master"), "初见段应置 met_xiao_master（持久化 flag）")
	var intro_text := str(dialog_box.text_label.text)
	check(intro_text.contains("稳当"), "首次对话应进入初见段，实际：%s" % intro_text)
	# 选项：有什么能帮忙的吗 → 任务交代段
	await advance_once(dialogue, dialog_box, "帮忙")
	var offer_text := str(dialog_box.text_label.text)
	check(offer_text.contains("小翠"), "应进入任务交代段（提及小翠），实际：%s" % offer_text)
	await advance_once(dialogue, dialog_box)  # 交代段第 2 行（选项挂这行）
	# 选项：好，我去去就来 → master_task_ok（@task_accept:side_003 落在台词上）
	await advance_once(dialogue, dialog_box, "去去就来")
	check(str(dialog_box.text_label.text).contains("好孩子"), "应进入应承段，实际：%s" % str(dialog_box.text_label.text))
	check(tasks.get_task_status("side_003") == "accepted", "任务应已接取（accepted）")
	check(dlg.has_flag("task_side_003_accepted"), "应同步任务接取 flag")
	check(int(ch.coins) == 0, "接取任务不发奖励，金币应为 0")
	await advance_to_end(dialogue, dialog_box)
	check(not dialogue.is_playing(), "对话应播完")
	print("[Task] accepted: status=%s" % tasks.get_task_status("side_003"))

	# ---- 4. 任务完成（与小翠对话）----
	check(maid_trigger.start_dialog(), "小翠对话应可触发（直接调用）")
	var maid_text := str(dialog_box.text_label.text)
	check(maid_text.contains("陪我聊天"), "应进入任务对话段，实际：%s" % maid_text)
	await advance_once(dialogue, dialog_box)  # 第二行挂 @task_complete:side_003
	check(tasks.get_task_status("side_003") == "submitted", "任务应完成并提交（submitted）")
	check(int(ch.coins) == 50, "奖励应为 50 金币，实际 %d" % int(ch.coins))
	check(dlg.has_flag("task_side_003_done"), "应同步任务完成 flag")
	check(not tasks.complete_task("side_003"), "已提交任务重复完成应返回 false")
	await advance_once(dialogue, dialog_box, "聊痛快")
	await advance_to_end(dialogue, dialog_box)
	check(not dialogue.is_playing(), "小翠对话应播完")
	print("[Task] completed: coins=%s" % str(ch.coins))

	# ---- 5. 任务关闭：萧老爷再访不再出现任务段 ----
	check(master_trigger.start_dialog(), "萧老爷对话应可再次触发")
	var catchup_text := str(dialog_box.text_label.text)
	check(catchup_text.contains("投缘"), "应进入任务完成后的日常段，实际：%s" % catchup_text)
	check(not catchup_text.contains("晒药闷得慌"), "任务可接段不应再出现")
	await advance_to_end(dialogue, dialog_box)
	check(not dialogue.is_playing(), "再访对话应播完")
	print("[Task] closed: catchup_ok")

	# ---- 6. 重置后可重复演示 ----
	tasks.reset_demo()
	check(tasks.get_task_status("side_003") == "available", "重置后任务应回到可接取")
	check(not dlg.has_flag("met_xiao_master"), "重置应清空对话 flag")

	index.queue_free()
	_finish()


func _finish() -> void:
	if ok:
		print("[Task] RESULT=PASS")
	else:
		print("[Task] RESULT=FAIL")
	quit(0 if ok else 1)
