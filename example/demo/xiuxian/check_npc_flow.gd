# NPC 对话系统端到端验收 —— 主城默认加载 + NPC 落位 + E 键触发对话
#
# 运行：perl -e 'alarm 180; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_npc_flow.gd
#
# 覆盖：
#   1. 装配：MapManager initial_map=xiuxian_xiaozhai；DialogLayer（GdDialogue
#      分组 + DialogBox）挂载
#   2. 初始加载：默认进萧宅，3 个 NPC 站位可行走且落在格心
#   3. NPC 接线：trigger.dialogue_path 解析到共享 GdDialogue，Speaker/entry_stage 正确
#   4. 玩家：player 分组 + Speaker("云舟")（触发器经分组解析玩家）
#   5. 范围边沿：玩家进入半径 → s_trigger_enter + 头顶提示显示
#   6. E 键触发：parse_input_event(E) → 对话激活、对话框显示、双方暂停
#   7. 推进到结束：s_finished → 面板隐藏、触发器复位、玩家解除暂停
#   8. 转场青云坊：商人「让我看看货品」→ @open_ui:StoreModal → 商店打开、对话收尾
#   9. 角色目录拆分：各 NPC 自载专属 timeline（trigger.timeline_path）+
#      AI 行为（萧宅：老爷/夫人 IDLE、小翠 WANDER）网格游走不穿不可行走格
extends SceneTree

const DIALOGUE_GROUP := "__gd_dialogue"

var ok := true
var enter_count := 0
var finished_count := 0


func fail(msg: String) -> void:
	push_error("[Npc] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _run() -> void:
	# 1. 装配
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "scenes/main/index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	for i in 6:
		await process_frame

	var mgr: Node = index.get_node_or_null("MapManager")
	var dialog_layer: Node = index.get_node_or_null("DialogLayer")
	check(mgr != null, "主场景应挂载 MapManager")
	check(dialog_layer != null, "主场景应挂载 DialogLayer")
	if mgr == null or dialog_layer == null:
		_finish()
		return
	check(str(mgr.initial_map) == "xiuxian_xiaozhai", "initial_map 应为 xiuxian_xiaozhai（默认进萧宅）")
	var dialogue: Node = dialog_layer.get_node_or_null("Dialogue")
	var dialog_box: Node = dialog_layer.get_node_or_null("DialogBox")
	check(dialogue != null and dialogue.is_in_group(DIALOGUE_GROUP),
		"DialogLayer 下应有共享 GdDialogue 且加入 %s 分组" % DIALOGUE_GROUP)
	check(dialog_box != null and dialog_box.has_method("handle_line"),
		"DialogLayer 下应有实现 handle_line 的 DialogBox")
	# 共享 Dialogue 不再预载 timeline：每个 NPC 触发时经 trigger.timeline_path 自载
	check(str(dialogue.get_timeline_path()).is_empty(),
		"共享 Dialogue 初始不应预载 timeline（timeline 拆到各角色目录）")
	if dialogue == null or dialog_box == null:
		_finish()
		return

	# 2. 初始加载 + NPC 落位（萧宅）
	check(str(mgr.get_current_alias()) == "xiuxian_xiaozhai",
		"初始应自动加载萧宅，实际 %s" % mgr.get_current_alias())
	var map: Node = mgr.get_current_map()
	check(map != null, "萧宅地图实例应已加载")
	if map == null:
		_finish()
		return
	var npcs: Array = []
	for child in map.get_children():
		if child.has_method("get_component") or child.get_class() == "GdMapMarker":
			continue
		if child.get_class() == "GdRoleMover":
			npcs.append(child)
	check(npcs.size() == 3, "主城应有 3 个 NPC，实际 %d" % npcs.size())
	if npcs.size() != 3:
		_finish()
		return
	# 等 NPC 的 _process 吸附/接线收敛
	for i in 5:
		await process_frame
	# 各角色独立目录期望：timeline 归属 + AI 行为（老爷/夫人站桩、小翠游走）
	var expect_behavior := {"萧老爷": 0, "萧夫人": 0, "小翠": 1}
	var expect_timeline := {
		"萧老爷": "role/npc/xiao_master/master_timeline.txt",
		"萧夫人": "role/npc/xiao_lady/lady_timeline.txt",
		"小翠": "role/npc/xiao_maid/maid_timeline.txt",
	}
	for npc in npcs:
		var cell: Vector2i = map.world_to_cell(npc.global_position)
		check(map.is_walkable(cell),
			"NPC %s 站位格 %s 应可行走" % [npc.display_name, cell])
		var cs: int = map.get_cell_size_px()
		# 出生落位校验：AI 基准点 home 记录吸附后的真实位置（应为可行走格心）。
		# 游走/巡逻 NPC 此刻可能已离位，不能拿当前坐标断定格心
		if npc.brain != null:
			var home: Vector2 = npc.brain.get_home()
			var home_cell: Vector2i = map.world_to_cell(home)
			check(map.is_walkable(home_cell),
				"NPC %s 出生格 %s 应可行走" % [npc.display_name, home_cell])
			var home_center := Vector2((home_cell.x + 0.5) * cs, (home_cell.y + 0.5) * cs)
			check(home == home_center,
				"NPC %s 应落在格心 %s，实际 %s" % [npc.display_name, home_center, home])
		# 站桩 NPC 不会移动：当前位置仍应精确等于格心
		if npc.brain != null and int(npc.brain.behavior) == 0:
			var center := Vector2((cell.x + 0.5) * cs, (cell.y + 0.5) * cs)
			check(npc.global_position == center,
				"站桩 NPC %s 应停在格心 %s，实际 %s" % [npc.display_name, center, npc.global_position])
		# 3. 接线：trigger 对话路径可解析到共享 Dialogue
		var trigger: Node = npc.get_node_or_null("Trigger")
		check(trigger != null, "NPC %s 应挂 GdDialogTrigger" % npc.display_name)
		if trigger == null:
			continue
		var resolved: Node = trigger.get_node_or_null(trigger.dialogue_path)
		check(resolved != null and resolved.get_instance_id() == dialogue.get_instance_id(),
			"NPC %s 的 trigger.dialogue_path 应解析到共享 Dialogue，实际 %s"
				% [npc.display_name, resolved])
		check(int(trigger.trigger_mode) == 1, "触发模式应为 INTERACT(1)（E 键）")
		check(not str(trigger.entry_stage).is_empty(),
			"NPC %s 应配置 entry_stage" % npc.display_name)
		# 3b. timeline 拆分到角色目录：trigger 携带本角色专属 timeline
		check(str(trigger.timeline_path).ends_with(expect_timeline[str(npc.role_name)]),
			"NPC %s 的 timeline 应为 %s，实际 %s"
				% [npc.display_name, expect_timeline[str(npc.role_name)], trigger.timeline_path])
		# 3c. AI 行为：Brain 已装配且行为类型正确
		check(npc.brain != null, "NPC %s 应挂 GdNpcBrain" % npc.display_name)
		if npc.brain != null:
			check(int(npc.brain.behavior) == int(expect_behavior[str(npc.role_name)]),
				"NPC %s 的 AI 行为应为 %d，实际 %d"
					% [npc.display_name, expect_behavior[str(npc.role_name)], npc.brain.behavior])
		var speaker: Node = npc.get_node_or_null("Speaker")
		check(speaker != null and str(speaker.role_name) == str(npc.role_name),
			"NPC %s 的 Speaker 角色名应一致" % npc.display_name)
	print("[Npc] setup: npcs=%d wired" % npcs.size())

	# 2b. AI 网格游走合法性：跑一段时间，游走/巡逻中的 NPC 始终站在可行走格
	# （GdNpcBrain 网格适配：经 find_path BFS 逐格走，不穿水域/山地）
	var mover_npcs: Array = []
	for npc in npcs:
		if npc.brain != null and int(npc.brain.behavior) > 0:
			mover_npcs.append(npc)
	check(mover_npcs.size() == 1, "游走 NPC 应共 1 个（小翠），实际 %d" % mover_npcs.size())
	var bad_cell := false
	for i in 120:
		await process_frame
		for npc in mover_npcs:
			var c: Vector2i = map.world_to_cell(npc.global_position)
			if not map.is_walkable(c):
				bad_cell = true
	check(not bad_cell, "游走/巡逻期间 NPC 不应踏入不可行走格（网格寻路适配）")
	for npc in mover_npcs:
		print("[Npc] ai: %s state=%s" % [npc.display_name, npc.brain.get_ai_state()])

	# 4. 玩家分组 + Speaker
	var player: Node = index.get_node_or_null("Player")
	check(player != null, "主场景应挂载 Player")
	if player == null:
		_finish()
		return
	check(player.is_in_group("player"), "Player 应加入 player 分组（触发器解析玩家）")
	var p_speaker: Node = player.get_node_or_null("Speaker")
	check(p_speaker != null and str(p_speaker.role_name) == "云舟",
		"Player 应挂 Speaker 且角色名为 云舟")

	# 5. 范围边沿：把玩家放到首个 NPC 旁一格（32px < 半径 96px）
	# 注意：出生点 (10,8) 与萧老爷 (9,6) 相距不足半径，s_trigger_enter 早已
	# 发出——先把玩家挪远、等半径退出，再连信号构造"由远及近"的完整边沿
	var npc0: Node = npcs[0]
	var trigger0: Node = npc0.get_node_or_null("Trigger")
	var cs: int = map.get_cell_size_px()
	player.global_position = Vector2(16, 12) * cs + Vector2(16, 16)
	for i in 8:
		await process_frame
	trigger0.connect("s_trigger_enter", func() -> void: enter_count += 1)
	var near: Vector2 = npc0.global_position + Vector2(cs, 0)
	var near_cell: Vector2i = map.world_to_cell(near)
	if not map.is_walkable(near_cell):
		near = npc0.global_position - Vector2(cs, 0)
	player.global_position = near
	for i in 8:
		await process_frame
	check(enter_count >= 1, "玩家进入半径应发出 s_trigger_enter，实际 %d" % enter_count)
	check(npc0._hint != null and npc0._hint.visible,
		"进入半径后 NPC 头顶应显示 E 对话提示")
	print("[Npc] range: enter=%d hint=%s" % [enter_count, npc0._hint.visible])

	# 6. E 键触发（GdDialogTrigger INTERACT 模式：InputMap 无 interact 动作时回退物理 E 键）
	var ev := InputEventKey.new()
	ev.physical_keycode = KEY_E
	ev.keycode = KEY_E
	ev.pressed = true
	Input.parse_input_event(ev)
	for i in 4:
		await process_frame
	check(trigger0.is_dialog_active(), "玩家在半径内按 E 应触发对话")
	check(dialogue.is_playing(), "对话应处于播放状态")
	check(str(dialogue.get_timeline_path()).ends_with("master_timeline.txt"),
		"萧老爷对话应自载 master_timeline.txt，实际 %s" % dialogue.get_timeline_path())
	check(dialog_box.panel != null and dialog_box.panel.visible, "对话框面板应显示")
	check(player.is_paused(), "对话期间玩家移动应被暂停")
	check(npc0.is_paused(), "对话期间 NPC 也应被暂停")
	print("[Npc] dialog started: active=%s playing=%s box=%s"
		% [trigger0.is_dialog_active(), dialogue.is_playing(), dialog_box.panel.visible])

	# 7. 推进到结束（走 DialogBox 自身入口：普通行 _advance / 选项点按钮，
	#    播完时由 box._close 收尾——绕过 box 推进会让面板失去关闭时机）
	dialogue.connect("s_finished", func() -> void: finished_count += 1)
	var guard := 0
	while dialogue.is_playing() and guard < 60:
		guard += 1
		await process_frame
		if not dialogue.is_playing():
			break
		if dialog_box.responding:
			# 分支选项等待中：点击首个选项按钮（dialog_box._choose → exec_response）
			var btn: Button = dialog_box.choices_box.get_child(0)
			btn.pressed.emit()
		else:
			dialog_box._advance()
		await process_frame
	for i in 6:
		await process_frame
	check(finished_count >= 1, "对话播完应发出 s_finished")
	check(not trigger0.is_dialog_active(), "对话结束后触发器应复位")
	check(not player.is_paused(), "对话结束后玩家应解除暂停")
	check(not npc0.is_paused(), "对话结束后 NPC 应解除暂停")
	check(dialog_box.panel == null or not dialog_box.panel.visible, "对话框面板应隐藏")
	print("[Npc] dialog finished: finished=%d guard=%d" % [finished_count, guard])

	# 8. 转场青云坊，选项触发 UI 命令：商人选「看看货品」→ open_ui:StoreModal
	mgr.open_map("xiuxian_town", Vector2i(6, 10))
	check(str(mgr.get_current_alias()) == "xiuxian_town", "open_map 应切换到青云坊")
	map = mgr.get_current_map()
	npcs = []
	for child in map.get_children():
		if child.has_method("get_component") or child.get_class() == "GdMapMarker":
			continue
		if child.get_class() == "GdRoleMover":
			npcs.append(child)
	for i in 5:
		await process_frame
	var merchant: Node = null
	for npc in npcs:
		if str(npc.role_name) == "坊市商人":
			merchant = npc
			break
	check(merchant != null, "青云坊应有坊市商人（timeline merchant_talk）")
	if merchant != null:
		var mtrigger: Node = merchant.get_node_or_null("Trigger")
		var near2: Vector2 = merchant.global_position + Vector2(cs, 0)
		if not map.is_walkable(map.world_to_cell(near2)):
			near2 = merchant.global_position - Vector2(cs, 0)
		player.global_position = near2
		for i in 8:
			await process_frame
		# 先补一个 E 键释放（清掉 step6 遗留的按下沿），等物理帧消费释放状态，
		# 再触发新按下沿（release/press 同物理帧会丢边沿 → 触发偶发失效）
		var rel := InputEventKey.new()
		rel.physical_keycode = KEY_E
		rel.keycode = KEY_E
		Input.parse_input_event(rel)
		for i in 4:
			await physics_frame
		var ev2 := InputEventKey.new()
		ev2.physical_keycode = KEY_E
		ev2.keycode = KEY_E
		ev2.pressed = true
		Input.parse_input_event(ev2)
		for i in 4:
			await physics_frame
		for i in 4:
			await process_frame
		check(mtrigger.is_dialog_active(), "商人对话应被 E 键触发")
		check(dialog_box.responding, "商人开场应出现分支选项")
		check(str(dialogue.get_timeline_path()).ends_with("merchant_timeline.txt"),
			"商人对话应自载 merchant_timeline.txt，实际 %s" % dialogue.get_timeline_path())
		var store_btn: Button = null
		for child in dialog_box.choices_box.get_children():
			if child is Button and str(child.text).contains("货品"):
				store_btn = child
				break
		check(store_btn != null, "商人选项应包含「让我看看货品」入口")
		if store_btn != null:
			store_btn.pressed.emit()
			for i in 6:
				await process_frame
			check(GdUIManager.has_ui("StoreModal"), "StoreModal 应经组合根注册（ui_id）")
			var modal: Control = GdUIManager.find_ui("StoreModal")
			check(modal != null and modal.visible, "选「让我看看货品」后 StoreModal 应打开")
			check(not dialogue.is_playing(), "打开商店后对话应结束（end）")
			check(not player.is_paused(), "对话结束后玩家应解除暂停")
			print("[Npc] ui command: store_open=%s dialog_playing=%s"
				% [modal != null and modal.visible, dialogue.is_playing()])
			if modal != null:
				modal.call("close")
				check(not modal.call("is_modal_open"), "close 后 Modal 状态应立即翻转为关闭")
				for i in 40:
					await process_frame
				check(not modal.visible, "Modal 关闭动画结束后应整体隐藏")

	index.queue_free()
	_finish()


func _finish() -> void:
	if ok:
		print("[Npc] RESULT=PASS")
	else:
		print("[Npc] RESULT=FAIL")
	quit(0 if ok else 1)
