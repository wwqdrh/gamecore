# 任务系统 + 对话进度端到端验收 —— flag 门控 / 最近触发 / 任务奖励闭环
#
# 运行：perl -e 'alarm 150; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_task_flow.gd
#
# 覆盖：
#   1. 最近触发：两个 NPC 触发圈重叠时，只有离玩家最近的触发器能启动对话
#   2. 对话进度：萧老爷初见段（置 met_xiao_master）只进一次，
#      再次对话自动跳到任务可接段（flag 门控 + start_from 回退选段）
#   3. 主线接取：选项「拜见萧伯伯」→ @task_accept:main_101 + 推进直达
#      completed（steps=1，列表领奖语义）
#   4. 支线接取：side_101 家主嘱托（前置 main_101 提交后解锁）→
#      @task_accept:side_101 → 与小翠对话 @task_complete → 50 金币
#   5. 任务关闭：完成后萧老爷对话回日常兜底段（任务可接段不再出现）
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
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	# 0. 状态基线
	var tasks: XiuTaskState = XiuTaskState.ins()
	var ch: XiuCharacterState = XiuCharacterState.ins()
	var dlg: XiuDialogState = XiuDialogState.ins()
	tasks.reset_demo()
	ch.reset_demo()
	check(XiuTaskTable.count() == 15, "任务定义表应有 15 条（stage1 家族篇 10 + stage2 宗门篇 5），实际 %d" % XiuTaskTable.count())
	check(str(tasks.get_task("main_101").get("name")) == "初来乍到", "main_101 应为家族篇首任务")
	check(str(tasks.get_task("sect_tournament").get("category")) == "主线", "sect_tournament 应为主线（宗门选拔）")
	check(str(tasks.get_task("daily_201").get("giver")) == "执事长老", "stage2 daily_201 应在总表中")

	# 0b. 任务目录视图：仅 available 进"可接取"组并带接取指引；
	# 前置未满足（locked）不进目录——前置没完成后续任务不显示
	var v_main1 := {}
	for v in tasks.views:
		if str(v.get("id", "")) == "main_101":
			v_main1 = v
	check(str(v_main1.get("status_group", "")) == "avail", "可接取任务应入 avail 状态组（任务目录页签）")
	check(str(v_main1.get("hint", "")) == "找「萧老爷」接取", "可接取任务应带 giver 接取指引，实际 %s" % str(v_main1.get("hint", "")))
	check(str(v_main1.get("btn_state", "")) == "find", "可接取任务按钮应为 find（去接取）")
	var v_main2 := {}
	for v in tasks.views:
		if str(v.get("id", "")) == "main_102":
			v_main2 = v
	check(str(v_main2.get("status_group", "")) == "", "前置未满足的任务不应进任何页签（status_group 空）")
	check(str(v_main2.get("btn_state", "")) == "locked", "未解锁任务按钮应为 locked（不展示，仅视图字段完整）")
	check(tasks.get_task_status("sect_tournament") == "locked",
		"sect_tournament 前置 main_105 未完成应推导为 locked（定义表已无 unlock 字段）")
	check(tasks.get_task_status("side_102") == "available",
		"无前置任务应推导为 available（unlock 状态由前置链推导而非定义表）")

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
	# 本测试的对话 NPC 都在萧宅：出生点已改为 home 农田（beat 0），显式切图
	mgr.open_map("xiuxian_xiaozhai", Vector2i(10, 8))
	for i in 6:
		await process_frame
	check(str(mgr.get_current_alias()) == "xiuxian_xiaozhai", "切图后应加载萧宅")
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
	# 日程/闲聊冻结：schedule 会按当前小时走位/teleport，写死站位假设必失效
	# （对话流测试冻结规约，见 NPC_LIFE.md ④）
	for n in get_nodes_in_group("npc_schedule"):
		n.set("enable", false)
	for n in get_nodes_in_group("npc_chat"):
		n.set("enable", false)
	for i in 3:
		await process_frame
	var master_trigger: Node = master.get_node("Trigger")
	var lady_trigger: Node = lady.get_node("Trigger")
	var maid_trigger: Node = maid.get_node("Trigger")

	# ---- 2. 最近触发（站位由 NPC 实际位置推导，日程冻结后位置稳定）----
	var cs: int = map.get_cell_size_px()
	var mpos: Vector2 = master.global_position
	var lpos: Vector2 = lady.global_position
	check(master_trigger.is_in_group("dialog_trigger"), "触发器应加入 dialog_trigger 分组")
	# 玩家放两者连线靠夫人一侧：夫人更近 → 老爷让位
	player.global_position = lpos + (mpos - lpos) * 0.25
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

	# ---- 3. 对话进度 + 主线接取（main_101：初见段接取+推进）----
	tasks.reset_demo()
	ch.reset_demo()
	check(not dlg.has_flag("met_xiao_master"), "重置后 met_xiao_master 应为 false")
	# 玩家贴到老爷身边（老爷必然是最近触发器）
	player.global_position = mpos + Vector2(cs * 0.5, 0)
	for i in 3:
		await physics_frame
	check(master_trigger.start_dialog(), "萧老爷对话应可触发")
	var intro_text := str(dialog_box.text_label.text)
	check(intro_text.contains("稳当"), "首次对话应进入初见段，实际：%s" % intro_text)
	await advance_once(dialogue, dialog_box)  # 第 2 行
	await advance_once(dialogue, dialog_box)  # 第 3 行
	# 选项：拜见萧伯伯 → @task_accept:main_101;task_advance:main_101;goto:master_father
	await advance_once(dialogue, dialog_box, "拜见")
	check(dlg.has_flag("met_xiao_master"), "初见选项应置 met_xiao_master（持久化 flag）")
	check(tasks.get_task_status("main_101") == "submitted",
		"main_101 接取即推进终步（steps=1 → 自动领奖直达 submitted）")
	check(dlg.has_flag("task_main_101_accepted"), "应同步任务接取 flag")
	check(dlg.has_flag("task_main_101_step1"), "应同步任务步级 flag（step1）")
	check(int(ch.coins) == 30, "终步自动领奖应到账 30 金币，实际 %d" % int(ch.coins))
	check(str(dialog_box.text_label.text).contains("何止认得"), "应进入父亲旧事段，实际：%s" % str(dialog_box.text_label.text))
	await advance_to_end(dialogue, dialog_box)
	check(not dialogue.is_playing(), "对话应播完")

	# 列表领奖：main_101 已由 advance_task 终步自动领奖 → 后继点亮
	check(tasks.get_task_status("main_101") == "submitted", "main_101 应为 submitted")
	check(tasks.get_task_status("side_101") == "available", "side_101 前置 main_101 提交后应解锁")
	check(tasks.get_task_status("main_102") == "available", "main_102 前置 main_101 提交后应解锁")
	print("[Task] main_101 submitted, successors unlocked")

	# ---- 4. 支线接取（side_101 嘱托段）+ 完成（与小翠对话）----
	check(master_trigger.start_dialog(), "萧老爷对话应可再次触发")
	var offer_text := str(dialog_box.text_label.text)
	check(offer_text.contains("晒药闷得慌"), "应进入 side_101 嘱托段，实际：%s" % offer_text)
	await advance_once(dialogue, dialog_box)  # 第 2 行（选项挂这行）
	# 选项：好，我去去就来 → @task_accept:side_101;goto:master_end
	await advance_once(dialogue, dialog_box, "去去就来")
	check(str(dialog_box.text_label.text).contains("南门出去"), "应进入收尾段，实际：%s" % str(dialog_box.text_label.text))
	check(tasks.get_task_status("side_101") == "accepted", "任务应已接取（accepted）")
	check(dlg.has_flag("task_side_101_accepted"), "应同步任务接取 flag")
	await advance_to_end(dialogue, dialog_box)

	# 玩家挪到小翠身边（否则小翠触发器让位给更近的老爷）
	player.global_position = maid.global_position + Vector2(cs * 0.5, 0)
	for i in 3:
		await physics_frame
	check(maid_trigger.start_dialog(), "小翠对话应可触发（直接调用）")
	var maid_text := str(dialog_box.text_label.text)
	check(maid_text.contains("陪我聊天"), "应进入任务对话段，实际：%s" % maid_text)
	await advance_once(dialogue, dialog_box)  # 第 2 行（八卦）
	await advance_once(dialogue, dialog_box)  # 第 3 行（选拔伏笔）
	await advance_once(dialogue, dialog_box)  # 第 4 行（选项挂这行）
	check(dialog_box.responding, "任务段末行应出现「知道啦」选项")
	# 选项动作 @task_complete:side_101;goto:maid_end（完成位在选项动作里）
	await advance_once(dialogue, dialog_box, "知道啦")
	check(tasks.get_task_status("side_101") == "submitted", "任务应完成并提交（submitted）")
	check(int(ch.coins) == 80, "累计奖励应为 80 金币（main_101 的 30 + side_101 的 50），实际 %d" % int(ch.coins))
	check(dlg.has_flag("task_side_101_done"), "应同步任务完成 flag")
	check(not tasks.complete_task("side_101"), "已提交任务重复完成应返回 false")
	await advance_to_end(dialogue, dialog_box)
	check(not dialogue.is_playing(), "小翠对话应播完")
	print("[Task] completed: coins=%s" % str(ch.coins))

	# ---- 5. 任务关闭：萧老爷再访回日常兜底段 ----
	# side_101 已提交 → 嘱托段失配；main_102 未提交 → 灵田风波段失配 → 兜底
	player.global_position = mpos + Vector2(cs * 0.5, 0)
	for i in 3:
		await physics_frame
	check(master_trigger.start_dialog(), "萧老爷对话应可再次触发")
	var catchup_text := str(dialog_box.text_label.text)
	check(catchup_text.contains("有事？说吧"), "应进入日常兜底段，实际：%s" % catchup_text)
	check(not catchup_text.contains("晒药闷得慌"), "任务可接段不应再出现")
	await advance_to_end(dialogue, dialog_box)
	check(not dialogue.is_playing(), "再访对话应播完")
	print("[Task] closed: catchup_ok")

	# ---- 6. 重置后可重复演示 ----
	tasks.reset_demo()
	check(tasks.get_task_status("main_101") == "available", "重置后任务应回到可接取")
	check(not dlg.has_flag("met_xiao_master"), "重置应清空对话 flag")

	index.queue_free()
	_finish()


func _finish() -> void:
	if ok:
		print("[Task] RESULT=PASS")
	else:
		print("[Task] RESULT=FAIL")
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
