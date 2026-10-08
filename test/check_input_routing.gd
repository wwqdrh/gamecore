# 鼠标点击事件路由验收 —— 「点击世界才开火，点击 UI 不开火」
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://test/check_input_routing.gd
#
# 设计约定（框架红线）：凡「点击世界」类玩法输入（开火/点击寻路/点击移动）
# 一律走 unhandled 阶段事件路由——被 mouse_filter=STOP 的 UI 控件消费的
# 点击到不了 unhandled_input，引擎保证「收到 = 点击落在世界」，
# 严禁 Input 轮询 + gui_get_hovered_control 猜测。
#
# 覆盖：
#   1. 点击 UI 按钮（STOP）→ GdShooter 不开火
#   2. 点击空白（世界）→ 开火（s_fired）
#   3. 按住连射：fire_held 置位后按冷却持续开火，抬起复位
#   4. 宿主 is_paused()（对话/剧情锁 duck-type）→ 不接开火输入
#   5. 未配置的开火按键（默认右键）不开火
extends SceneTree

var ok := true


class PausableHost extends Node2D:
	var paused := false
	func is_paused() -> bool:
		return paused


func fail(msg: String) -> void:
	push_error("[Routing] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _mk_motion(pos: Vector2) -> InputEventMouseMotion:
	var ev := InputEventMouseMotion.new()
	ev.position = pos
	ev.global_position = pos
	return ev


func _mk_click(pos: Vector2, pressed: bool, button: MouseButton = MOUSE_BUTTON_LEFT) -> InputEventMouseButton:
	var ev := InputEventMouseButton.new()
	ev.position = pos
	ev.global_position = pos
	ev.button_index = button
	ev.pressed = pressed
	return ev


## 经 Input.parse_input_event 派发（真实 OS 事件路径：同步更新 Input 按键
## 状态 + 完整走 GUI → unhandled 路由；push_input 不更新 Input 状态）
func _send(ev: InputEvent) -> void:
	Input.parse_input_event(ev)
	Input.flush_buffered_events()


func _run() -> void:
	# ---- 装配：世界宿主(48,32) + GdShooter + 全屏半边 STOP 按钮 ----
	var host := PausableHost.new()
	host.name = "Host"
	host.position = Vector2(48, 12)  # 与点击点错开（同点瞄准为零向量，组件拒发）
	root.add_child(host)
	for i in 3:
		await process_frame

	var shooter := GdShooter.new()
	shooter.name = "Shooter"
	shooter.auto_fire_mouse = true  # 框架 unhandled 路由（默认开）
	shooter.fire_button_left = true
	shooter.fire_button_right = false
	shooter.fire_cooldown = 0.25
	shooter.bullet_alias = "routing_test_bullet"
	shooter.bullet_scene_path = "res://example/demo/xiuxian/role/player/bullet.tscn"
	host.add_child(shooter)
	for i in 3:
		await process_frame

	var fired := [0]
	shooter.connect("s_fired", func(_m, _d) -> void: fired[0] += 1)

	var btn := Button.new()
	btn.name = "UIBtn"
	btn.position = Vector2.ZERO
	btn.size = Vector2(24, 64)
	btn.text = "UI"
	root.add_child(btn)
	for i in 3:
		await process_frame

	# ---- 1. 点击 UI 按钮：GUI 消费，unhandled 收不到 → 不开火 ----
	_send(_mk_motion(Vector2(12, 32)))
	await physics_frame
	_send(_mk_click(Vector2(12, 32), true))
	_send(_mk_click(Vector2(12, 32), false))
	await physics_frame
	check(fired[0] == 0, "点击 UI 按钮（STOP）不应开火，实际 %d 发" % fired[0])

	# ---- 2. 点击空白（世界）：unhandled 路由 → 开火 ----
	_send(_mk_motion(Vector2(48, 48)))
	await physics_frame
	_send(_mk_click(Vector2(48, 48), true))
	await physics_frame
	check(fired[0] >= 1, "点击世界空白处应开火，实际 %d 发" % fired[0])
	# ---- 3. 按住连射：fire_held 置位，跨过冷却后自动续射 ----
	await create_timer(0.35).timeout
	await physics_frame
	check(fired[0] >= 2, "按住左键应跨冷却连射，实际 %d 发" % fired[0])
	_send(_mk_click(Vector2(48, 48), false))
	await physics_frame
	var after_release: int = fired[0]
	await create_timer(0.35).timeout
	await physics_frame
	check(fired[0] == after_release, "松开后应停止连射（fire_held 复位）")

	# ---- 4. 宿主 is_paused()（对话/剧情锁）→ 不接开火输入 ----
	host.paused = true
	_send(_mk_click(Vector2(48, 48), true))
	await physics_frame
	_send(_mk_click(Vector2(48, 48), false))
	await physics_frame
	check(fired[0] == after_release, "宿主 is_paused 时点击世界不应开火")
	host.paused = false

	# ---- 5. 未配置的开火按键（右键默认关）不开火 ----
	_send(_mk_click(Vector2(48, 48), true, MOUSE_BUTTON_RIGHT))
	await physics_frame
	_send(_mk_click(Vector2(48, 48), false, MOUSE_BUTTON_RIGHT))
	await physics_frame
	check(fired[0] == after_release, "未配置的开火按键（右键）不应开火")

	if ok:
		print("[Routing] RESULT=PASS")
	else:
		print("[Routing] RESULT=FAIL")
	quit(0 if ok else 1)
