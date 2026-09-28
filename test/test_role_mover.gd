# suite: role - GdRoleMover 移动控制测试
# 覆盖: 键盘四向/横向移动、鼠标点移动、AI 方向驱动与停止、
#        重力/落地信号、跳跃、加减速平滑
# 依赖: GDExtension 类 GdRoleMover，headless 下物理帧正常迭代
extends "res://test/test_case.gd"

const CONTROL_NONE := 0
const CONTROL_KEYBOARD := 1
const CONTROL_MOUSE := 2
const CONTROL_AI := 3
const MODE_FOUR_WAY := 0
const MODE_HORIZONTAL := 1


func _tree() -> SceneTree:
	return Engine.get_main_loop() as SceneTree


func wait_phys(frames: int) -> void:
	var t := _tree()
	for i in frames:
		await t.physics_frame


## 等待条件成立（物理帧轮询），返回是否满足
func wait_phys_until(cond: Callable, timeout_ticks: int = 600) -> bool:
	var t := _tree()
	var n := 0
	while n < timeout_ticks:
		if cond.call():
			return true
		await t.physics_frame
		n += 1
	return cond.call()


func _ensure_action(action: String) -> void:
	if not InputMap.has_action(action):
		InputMap.add_action(action)


func _cleanup_actions() -> void:
	for a in ["move_left", "move_right", "move_up", "move_down", "jump"]:
		if InputMap.has_action(a):
			Input.action_release(a)
			InputMap.erase_action(a)


## 创建无碰撞体的移动器（纯运动学测试用）并挂到树根
func _make_mover(nm: String, mode: int = MODE_FOUR_WAY) -> GdRoleMover:
	var mover := GdRoleMover.new()
	mover.name = nm
	mover.move_mode = mode
	_tree().root.add_child(mover)
	return mover


## 创建带碰撞体 + 地板的重力场景，返回 { mover, floor }
func _make_gravity_scene(nm: String) -> Dictionary:
	var tree := _tree()
	var mover := GdRoleMover.new()
	mover.name = nm
	mover.move_mode = MODE_HORIZONTAL
	mover.control_mode = CONTROL_NONE
	mover.gravity_enabled = true
	mover.position = Vector2(0, -100)

	var body_shape := CollisionShape2D.new()
	var circle := CircleShape2D.new()
	circle.radius = 10.0
	body_shape.shape = circle
	mover.add_child(body_shape)

	var floor_body := StaticBody2D.new()
	floor_body.name = nm + "Floor"
	var floor_shape := CollisionShape2D.new()
	var rect := RectangleShape2D.new()
	rect.size = Vector2(400, 20)
	floor_shape.shape = rect
	floor_body.add_child(floor_shape)
	floor_body.position = Vector2.ZERO

	tree.root.add_child(floor_body)
	tree.root.add_child(mover)
	return {"mover": mover, "floor": floor_body}


func test_keyboard_four_way_movement() -> void:
	_cleanup_actions()
	_ensure_action("move_left")
	var mover := _make_mover("MvKb4Way")
	var flags := {"moving": false, "facing": ""}
	mover.s_moving_changed.connect(func(m): flags["moving"] = true)
	mover.s_facing_changed.connect(func(f): flags["facing"] = f)

	Input.action_press("move_left")
	var moved := await wait_phys_until(func(): return mover.position.x < -10.0, 300)
	assert_true(moved, "按住 move_left 应向左移动 (x=%s)" % mover.position.x)
	assert_eq(mover.get_facing(), "left", "向左移动朝向应为 left")
	assert_true(flags["moving"], "应触发 s_moving_changed 信号")
	assert_eq(flags["facing"], "left", "应触发 s_facing_changed(left)")
	assert_true(mover.is_moving(), "移动中 is_moving 应为 true")

	Input.action_release("move_left")
	var stopped := await wait_phys_until(func(): return not mover.is_moving(), 60)
	assert_true(stopped, "松开按键后应停止移动")
	assert_near(mover.get_velocity().x, 0.0, 0.5, "松开后水平速度应归零")
	mover.free()
	_cleanup_actions()


func test_horizontal_mode_no_vertical() -> void:
	_cleanup_actions()
	_ensure_action("move_up")
	var mover := _make_mover("MvHorizontal", MODE_HORIZONTAL)

	Input.action_press("move_up")
	await wait_phys(30)
	assert_near(mover.position.y, 0.0, 0.01, "横向模式下上键不应产生垂直位移")
	assert_near(mover.position.x, 0.0, 0.01, "无水平输入时 x 不应变化")
	Input.action_release("move_up")

	_ensure_action("move_right")
	Input.action_press("move_right")
	var moved := await wait_phys_until(func(): return mover.position.x > 10.0, 300)
	assert_true(moved, "横向模式下右键应正常移动")
	Input.action_release("move_right")
	mover.free()
	_cleanup_actions()


func test_mouse_move_to_point() -> void:
	var mover := _make_mover("MvMouse")
	mover.control_mode = CONTROL_MOUSE
	var target := Vector2(200, 0)
	mover.move_to_point(target)

	var arrived := await wait_phys_until(
		func(): return mover.position.distance_to(target) <= 8.0, 600)
	assert_true(arrived, "move_to_point 应到达目标点附近 (pos=%s)" % mover.position)
	var stopped := await wait_phys_until(func(): return not mover.is_moving(), 60)
	assert_true(stopped, "到达后应自动停止")
	mover.free()


func test_ai_direction_and_stop() -> void:
	var mover := _make_mover("MvAiDir")
	# set_ai_direction 会自动切换为 AI 控制方式
	mover.set_ai_direction(Vector2.RIGHT)
	assert_eq(mover.control_mode, CONTROL_AI, "set_ai_direction 应切换为 AI 控制")

	var moved := await wait_phys_until(func(): return mover.position.x > 20.0, 300)
	assert_true(moved, "AI 方向驱动应向右移动 (pos=%s)" % mover.position)
	assert_eq(mover.get_facing(), "right", "AI 向右移动朝向应为 right")

	mover.stop()
	var stopped := await wait_phys_until(func(): return mover.get_velocity().length() < 0.5, 60)
	assert_true(stopped, "stop() 后应停止")
	mover.free()


func test_ai_target_position() -> void:
	var mover := _make_mover("MvAiTarget")
	mover.set_ai_target_position(Vector2(-150, 80), 6.0)
	var arrived := await wait_phys_until(
		func(): return mover.position.distance_to(Vector2(-150, 80)) <= 10.0, 600)
	assert_true(arrived, "AI 目标点驱动应到达目标 (pos=%s)" % mover.position)
	var stopped := await wait_phys_until(func(): return not mover.is_moving(), 60)
	assert_true(stopped, "AI 到达目标后应停止")
	mover.free()


func test_gravity_fall_and_landing() -> void:
	var scene := _make_gravity_scene("MvGravity")
	var mover: GdRoleMover = scene["mover"]
	var landed := {"fired": false}
	mover.s_landed.connect(func(): landed["fired"] = true)

	# 初始下落: y 向速度应增大
	await wait_phys(5)
	assert_true(mover.get_velocity().y > 0.0, "重力应产生向下速度 (vy=%s)" % mover.get_velocity().y)

	var on_floor := await wait_phys_until(func(): return mover.is_on_floor(), 600)
	assert_true(on_floor, "应落到地板上 (y=%s)" % mover.position.y)
	await wait_phys(3)
	assert_true(landed["fired"], "落地应触发 s_landed 信号")
	assert_near(mover.get_velocity().y, 0.0, 1.0, "落地后垂直速度应归零")
	mover.free()
	scene["floor"].free()


func test_jump() -> void:
	var scene := _make_gravity_scene("MvJump")
	var mover: GdRoleMover = scene["mover"]
	_cleanup_actions()
	_ensure_action("jump")

	# 先落到地面
	var on_floor := await wait_phys_until(func(): return mover.is_on_floor(), 600)
	assert_true(on_floor, "跳跃测试前置: 应先落地")

	var floor_y := mover.position.y
	mover.s_jumped.connect(func(): pass)  # 仅验证可连接不报错
	Input.action_press("jump")
	var rising := await wait_phys_until(func(): return mover.position.y < floor_y - 4.0, 120)
	assert_true(rising, "按跳跃键应向上起跳 (y=%s, floor_y=%s)" % [mover.position.y, floor_y])
	Input.action_release("jump")

	# 释放后应再次落回地面
	var landed_again := await wait_phys_until(func(): return mover.is_on_floor(), 600)
	assert_true(landed_again, "跳跃后应落回地面")
	assert_near(mover.position.y, floor_y, 2.0, "落回高度应与起跳前一致")
	mover.free()
	scene["floor"].free()
	_cleanup_actions()


func test_acceleration_and_friction() -> void:
	var mover := _make_mover("MvAccel")
	mover.speed = 150.0
	mover.acceleration = 300.0
	mover.friction = 300.0

	# 加速: 300/s 上限，每帧 +5，首帧观测值应远小于满速（渐变而非瞬发）
	mover.set_ai_direction(Vector2.RIGHT)
	var started := await wait_phys_until(
		func(): return mover.get_velocity().x > 0.5, 300)
	assert_true(started, "加速阶段应产生速度")
	var first_vx := mover.get_velocity().x
	assert_between(first_vx, 0.5, 15.0, "加速应从低速渐增 (首帧 vx=%s)" % first_vx)

	var full_speed := await wait_phys_until(
		func(): return mover.get_velocity().x >= 149.0, 300)
	assert_true(full_speed, "加速后应达到满速 150 (vx=%s)" % mover.get_velocity().x)

	# 减速: 摩擦渐降，不会瞬间归零
	mover.set_ai_direction(Vector2.ZERO)
	var decaying := await wait_phys_until(
		func(): return mover.get_velocity().x < 149.0, 300)
	assert_true(decaying, "松开方向后应开始减速")
	var decay_vx := mover.get_velocity().x
	assert_between(decay_vx, 130.0, 149.0, "减速应平滑而非瞬间归零 (vx=%s)" % decay_vx)
	var stopped := await wait_phys_until(
		func(): return mover.get_velocity().x <= 0.5, 300)
	assert_true(stopped, "摩擦应使速度衰减到 0")
	mover.free()


func test_max_fall_speed_clamp() -> void:
	var mover := _make_mover("MvMaxFall", MODE_HORIZONTAL)
	mover.gravity_enabled = true
	mover.gravity = 100000.0
	mover.max_fall_speed = 500.0

	await wait_phys(30)
	assert_true(mover.get_velocity().y <= 500.0,
		"下落速度应被 max_fall_speed 限制 (vy=%s)" % mover.get_velocity().y)
	mover.free()
