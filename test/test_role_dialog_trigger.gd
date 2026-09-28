# suite: role-dialog - 对话触发器与角色绑定测试
# 覆盖: AUTO/PROXIMITY/INTERACT/MANUAL 四种触发模式、条件门控、
#        trigger_once、cooldown、对话期间暂停移动、结束后恢复
extends "res://test/test_case.gd"

const TIMELINE := """
[first]
(测试者)
第一句。
第二句。

[second]
(测试者)
第二段。
"""

const RESPONSE_TIMELINE := """
[first]
(测试者)
选一个。
- 去下一段 @goto:second
- 直接结束 @end

[second]
(测试者)
第二段。
"""

var ended_count := 0


func wait_phys(frames: int) -> void:
	var tree := Engine.get_main_loop() as SceneTree
	for i in frames:
		await tree.physics_frame


func wait_phys_until(cond: Callable, timeout_ticks: int = 600) -> bool:
	var tree := Engine.get_main_loop() as SceneTree
	var ticks := 0
	while ticks < timeout_ticks:
		if cond.call():
			return true
		await tree.physics_frame
		ticks += 1
	return cond.call()


func _on_dialog_ended() -> void:
	ended_count += 1


## 构建最小对话场景：Dialogue + DialogBox(control) + Host(角色+触发器) + Player
func _build(mode: int, cfg: Dictionary = {}) -> Node2D:
	ended_count = 0
	var tree := Engine.get_main_loop() as SceneTree
	var scene := Node2D.new()
	scene.name = "DlgScene"
	tree.root.add_child(scene)

	var dialogue := GdDialogue.new()
	dialogue.name = "Dialogue"
	dialogue.initial(TIMELINE)
	scene.add_child(dialogue)

	var control: CanvasLayer = load("res://example/role/dialog_box.gd").new()
	control.name = "Box"
	control.dialogue_path = NodePath("../Dialogue")
	scene.add_child(control)

	var host := GdRoleMover.new()
	host.name = "Host"
	host.control_mode = 3  # AI
	host.position = Vector2.ZERO
	scene.add_child(host)

	var speaker := GdRoleSpeaker.new()
	speaker.name = "Speaker"
	speaker.role_name = "测试者"
	host.add_child(speaker)

	var trigger := GdDialogTrigger.new()
	trigger.name = "Trigger"
	trigger.trigger_mode = mode
	trigger.dialogue_path = NodePath("../../Dialogue")
	trigger.trigger_radius = cfg.get("radius", 100.0)
	trigger.auto_delay = cfg.get("auto_delay", 0.05)
	trigger.trigger_once = cfg.get("once", false)
	trigger.cooldown = cfg.get("cooldown", 0.0)
	trigger.condition_fn = cfg.get("condition", "")
	trigger.entry_stage = cfg.get("entry", "")
	host.add_child(trigger)
	trigger.s_dialog_ended.connect(_on_dialog_ended)

	var player := GdRoleMover.new()
	player.name = "Player"
	player.control_mode = 0
	player.position = cfg.get("player_pos", Vector2(2000, 0))
	scene.add_child(player)

	return scene


func _finish_dialogue(dialogue: GdDialogue) -> void:
	# 播完所有行；最后一调使 has_next=false 并触发 s_finished
	var guard := 0
	while dialogue.has_next() and guard < 100:
		dialogue.next("")
		guard += 1
	dialogue.next("")


func test_auto_trigger_pauses_and_resumes() -> void:
	var scene := _build(2)  # AUTO
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var host: GdRoleMover = scene.get_node("Host")
	var player: GdRoleMover = scene.get_node("Player")

	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(started, "AUTO 模式应在延迟后自动开始对话")
	assert_true(host.is_paused(), "对话中宿主应被暂停")
	assert_true(player.is_paused(), "对话中玩家应被暂停")

	_finish_dialogue(dialogue)
	var resumed := await wait_phys_until(
		func(): return not host.is_paused() and not player.is_paused(), 300)
	assert_true(resumed, "对话结束后双方应恢复移动")
	assert_eq(ended_count, 1, "s_dialog_ended 应触发一次")
	scene.free()


func test_proximity_trigger() -> void:
	var scene := _build(0, {"radius": 100.0})
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var player: GdRoleMover = scene.get_node("Player")

	await wait_phys(5)
	assert_false(dialogue.is_playing(), "玩家在范围外不应触发")

	player.position = Vector2(60, 0)
	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(started, "玩家进入范围应触发对话")
	scene.free()


func test_condition_gate() -> void:
	var scene := _build(0, {"condition": "has_flag:nope"})
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var player: GdRoleMover = scene.get_node("Player")
	var box: CanvasLayer = scene.get_node("Box")

	player.position = Vector2(60, 0)
	await wait_phys(45)
	assert_false(dialogue.is_playing(), "条件不满足时不应触发")

	box.set_flag("nope")
	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(started, "条件满足后应自动重试并触发")
	scene.free()


func test_interact_trigger() -> void:
	if not InputMap.has_action("interact"):
		InputMap.add_action("interact")
	var scene := _build(1, {"radius": 100.0})
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var player: GdRoleMover = scene.get_node("Player")

	player.position = Vector2(60, 0)
	await wait_phys(2)
	Input.action_press("interact")
	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	Input.action_release("interact")
	assert_true(started, "范围内按交互键应触发对话")
	scene.free()


func test_trigger_once() -> void:
	var scene := _build(2, {"once": true})
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var trigger: GdDialogTrigger = scene.get_node("Host/Trigger")

	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(started, "首次应触发")

	_finish_dialogue(dialogue)
	await wait_phys(3)
	assert_false(trigger.start_dialog(), "trigger_once 触发过一次后不应再次触发")
	scene.free()


func test_cooldown() -> void:
	var scene := _build(2, {"cooldown": 0.4})
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var trigger: GdDialogTrigger = scene.get_node("Host/Trigger")

	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(started, "第一次应触发")

	_finish_dialogue(dialogue)
	await wait_phys(3)
	assert_false(trigger.start_dialog(), "冷却期内不应再次触发")

	var retriggered := await wait_phys_until(
		func(): return trigger.start_dialog(), 300)
	assert_true(retriggered, "冷却结束后应可再次触发")
	_finish_dialogue(dialogue)
	await wait_phys(3)
	scene.free()


func test_manual_mode() -> void:
	var scene := _build(3)  # MANUAL
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var trigger: GdDialogTrigger = scene.get_node("Host/Trigger")

	await wait_phys(10)
	assert_false(dialogue.is_playing(), "MANUAL 模式不应自动触发")

	assert_true(trigger.start_dialog(), "手动调用 start_dialog 应触发")
	assert_true(dialogue.is_playing(), "手动触发后对话应开始")
	assert_true(trigger.is_dialog_active(), "触发器应处于对话中状态")
	_finish_dialogue(dialogue)
	await wait_phys(3)
	assert_false(trigger.is_dialog_active(), "对话结束后触发器应复位")
	scene.free()


func test_ui_style_advance_resumes() -> void:
	# 回归：DialogBox 显示完最后一句后就隐藏面板（has_next 已为 false），
	# 若误以为已结束则 playing 永远为 true，触发器等不到收尾、双方冻住。
	# 正确流程：每次点击 next() 一次，用 is_playing 判断是否真正播完。
	var scene := _build(2)
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var host: GdRoleMover = scene.get_node("Host")
	var player: GdRoleMover = scene.get_node("Player")

	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(started, "对话应开始")

	# 模拟 DialogBox 逐次点击推进
	var guard := 0
	while dialogue.is_playing() and guard < 100:
		dialogue.next("")
		guard += 1
	assert_false(dialogue.is_playing(), "UI 式推进应能正常播完（playing 复位）")
	assert_true(guard >= 2, "UI 应消费完剩余台词并补上收尾一击（guard=%d）" % guard)

	var resumed := await wait_phys_until(
		func(): return not host.is_paused() and not player.is_paused(), 300)
	assert_true(resumed, "UI 式推进结束后双方应恢复移动")
	assert_eq(ended_count, 1, "s_dialog_ended 应触发一次")
	scene.free()


func test_response_choice_advances_and_resumes() -> void:
	# 回归：带选项的对话，选项按钮必须可见可点，选择后能续播并正常收尾
	var scene := _build(2)
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var host: GdRoleMover = scene.get_node("Host")
	var player: GdRoleMover = scene.get_node("Player")
	dialogue.initial(RESPONSE_TIMELINE)

	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(started, "带选项的对话应开始")

	var box: CanvasLayer = scene.get_node("Box")
	assert_true(box.responding, "第一句应处于选项等待状态")
	assert_true(box.choices_panel.visible, "选项面板应显示在屏幕中央")
	assert_eq(box.choices_box.get_child_count(), 2, "应有两个选项按钮")

	# 点击第一个选项按钮（goto:second → 续播）
	var btn: Button = box.choices_box.get_child(0)
	btn.pressed.emit()
	await wait_phys(2)
	assert_true(dialogue.is_playing(), "选择后应续播下一段")
	assert_false(box.responding, "续播行不应再处于选项状态")
	assert_false(box.choices_panel.visible, "续播时应隐藏选项面板")
	assert_true(box.panel.visible, "续播时对话框应保持显示")

	# 连续推进直到播完，验证收尾与恢复
	var guard := 0
	while dialogue.is_playing() and guard < 50:
		dialogue.next("")
		guard += 1
	assert_false(dialogue.is_playing(), "应能正常播完（playing 复位）")
	var resumed := await wait_phys_until(
		func(): return not host.is_paused() and not player.is_paused(), 300)
	assert_true(resumed, "对话结束后双方应恢复移动")
	assert_eq(ended_count, 1, "s_dialog_ended 应触发一次")
	scene.free()


func test_proximity_rearm_after_dialog() -> void:
	# 回归：PROXIMITY 对话结束后玩家仍在范围内时，不应立刻重触发（原地循环冻结）
	var scene := _build(0, {"radius": 100.0})
	var dialogue: GdDialogue = scene.get_node("Dialogue")
	var player: GdRoleMover = scene.get_node("Player")

	player.position = Vector2(60, 0)
	var started := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(started, "进入范围应触发对话")

	_finish_dialogue(dialogue)
	await wait_phys(30)
	assert_false(dialogue.is_playing(),
		"对话结束后玩家仍留在范围内不应立刻重触发")

	# 走出范围重新武装，再回来可再次触发
	player.position = Vector2(2000, 0)
	await wait_phys(10)
	player.position = Vector2(60, 0)
	var restarted := await wait_phys_until(
		func(): return dialogue.is_playing(), 300)
	assert_true(restarted, "离开后重新进入范围应能再次触发")
	_finish_dialogue(dialogue)
	await wait_phys(3)
	scene.free()
