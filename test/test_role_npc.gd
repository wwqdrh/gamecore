# suite: role - GdNpcBrain NPC 行为 AI 测试
# 覆盖: 待机不动、随机游走状态切换与位移、路径巡逻(循环/往返)、
#        跟随目标保持距离、逃离目标、s_ai_state_changed 信号
# 依赖: GDExtension 类 GdRoleMover / GdNpcBrain
# 行为常量: 0=IDLE 1=WANDER 2=PATROL 3=FOLLOW 4=FLEE
extends "res://test/test_case.gd"


func _tree() -> SceneTree:
	return Engine.get_main_loop() as SceneTree


func wait_phys(frames: int) -> void:
	var t := _tree()
	for i in frames:
		await t.physics_frame


func wait_phys_until(cond: Callable, timeout_ticks: int = 900) -> bool:
	var t := _tree()
	var n := 0
	while n < timeout_ticks:
		if cond.call():
			return true
		await t.physics_frame
		n += 1
	return cond.call()


## 组装 mover + brain（brain 缩短状态时长让测试更快）
func _make_npc(nm: String, behavior: int) -> Dictionary:
	var mover := GdRoleMover.new()
	mover.name = nm + "Mover"
	mover.speed = 300.0
	var brain := GdNpcBrain.new()
	brain.name = nm + "Brain"
	brain.behavior = behavior
	brain.idle_min = 0.05
	brain.idle_max = 0.1
	brain.walk_min = 0.05
	brain.walk_max = 0.1
	mover.add_child(brain)
	_tree().root.add_child(mover)
	return {"mover": mover, "brain": brain}


func test_idle_behavior_stays_put() -> void:
	var rig := _make_npc("NpcIdle", 0)
	var mover: GdRoleMover = rig["mover"]
	var brain: GdNpcBrain = rig["brain"]

	await wait_phys(60)
	assert_eq(brain.get_ai_state(), "idle", "待机行为状态应一直为 idle")
	assert_near(mover.position.x, 0.0, 0.5, "待机不应产生位移 x")
	assert_near(mover.position.y, 0.0, 0.5, "待机不应产生位移 y")
	mover.free()


func test_wander_moves_and_toggles_state() -> void:
	var rig := _make_npc("NpcWander", 1)
	var mover: GdRoleMover = rig["mover"]
	var brain: GdNpcBrain = rig["brain"]
	brain.wander_radius = 80.0

	var states := {}
	brain.s_ai_state_changed.connect(func(s): states[s] = true)

	var home := mover.position
	var walked := await wait_phys_until(
		func(): return mover.position.distance_to(home) > 10.0, 900)
	assert_true(walked, "游走应产生位移 (pos=%s)" % mover.position)

	var saw_walk := await wait_phys_until(func(): return states.has("walk"), 900)
	assert_true(saw_walk, "游走应进入 walk 状态 (states=%s)" % [states])

	# walk 结束后应回到 idle（再触发一次信号）
	var saw_idle := await wait_phys_until(
		func(): return states.has("idle") and brain.get_ai_state() == "idle", 900)
	assert_true(saw_idle, "游走间隙应回到 idle 状态 (states=%s)" % [states])

	# 始终不远离家超过 wander_radius + 余量
	assert_true(mover.position.distance_to(home) <= 120.0,
		"游走不应远离家超过半径余量 (dist=%s)" % mover.position.distance_to(home))
	mover.free()


func test_patrol_loop_visits_points() -> void:
	var rig := _make_npc("NpcPatrol", 2)
	var mover: GdRoleMover = rig["mover"]
	var brain: GdNpcBrain = rig["brain"]
	var point_a := Vector2(120, 0)
	var point_b := Vector2(120, 120)
	brain.patrol_points = PackedVector2Array([point_a, point_b])
	brain.patrol_loop = true
	brain.patrol_pause = 0.0

	var hit_a := await wait_phys_until(
		func(): return mover.position.distance_to(point_a) <= 12.0, 900)
	assert_true(hit_a, "循环巡逻应到达点 A (pos=%s)" % mover.position)

	var hit_b := await wait_phys_until(
		func(): return mover.position.distance_to(point_b) <= 12.0, 900)
	assert_true(hit_b, "巡逻应到达点 B (pos=%s)" % mover.position)

	# 循环模式应再次回到 A
	var hit_a_again := await wait_phys_until(
		func(): return mover.position.distance_to(point_a) <= 12.0, 900)
	assert_true(hit_a_again, "循环巡逻应回到点 A (pos=%s)" % mover.position)
	mover.free()


func test_patrol_pingpong_reverses() -> void:
	var rig := _make_npc("NpcPingPong", 2)
	var mover: GdRoleMover = rig["mover"]
	var brain: GdNpcBrain = rig["brain"]
	var point_a := Vector2(120, 0)
	var point_b := Vector2(240, 0)
	brain.patrol_points = PackedVector2Array([point_a, point_b])
	brain.patrol_loop = false
	brain.patrol_pause = 0.0

	var hit_b := await wait_phys_until(
		func(): return mover.position.distance_to(point_b) <= 12.0, 900)
	assert_true(hit_b, "往返巡逻应到达端点 B (pos=%s)" % mover.position)

	# 往返: 从 B 折回应先经过 A 而不是继续远离
	var back_near_a := await wait_phys_until(
		func(): return mover.position.x < point_a.x + 20.0, 900)
	assert_true(back_near_a, "往返模式到达端点后应折返 (pos=%s)" % mover.position)
	mover.free()


func test_patrol_pause_at_points() -> void:
	var rig := _make_npc("NpcPatrolPause", 2)
	var mover: GdRoleMover = rig["mover"]
	var brain: GdNpcBrain = rig["brain"]
	var point_a := Vector2(120, 0)
	var point_b := Vector2(240, 0)
	brain.patrol_points = PackedVector2Array([point_a, point_b])
	brain.patrol_loop = true
	brain.patrol_pause = 0.5

	var hit_a := await wait_phys_until(
		func(): return mover.position.distance_to(point_a) <= 12.0, 900)
	assert_true(hit_a, "应到达点 A")

	# 到点后应进入 pause 状态并停留
	var paused := await wait_phys_until(func(): return brain.get_ai_state() == "pause", 120)
	assert_true(paused, "patrol_pause > 0 时到点应进入 pause 状态")
	await wait_phys(5)
	assert_near(mover.position.distance_to(point_a), 0.0, 13.0, "停留期间不应离开点 A 太远")
	mover.free()


func test_follow_keeps_distance() -> void:
	var rig := _make_npc("NpcFollow", 3)
	var mover: GdRoleMover = rig["mover"]
	var brain: GdNpcBrain = rig["brain"]
	brain.follow_stop_distance = 40.0

	var target := Node2D.new()
	target.name = "FollowTarget"
	target.position = Vector2(0, 300)
	_tree().root.add_child(target)
	brain.follow_target = target

	var close := await wait_phys_until(
		func(): return mover.position.distance_to(target.position) <= 48.0, 900)
	assert_true(close, "跟随应逼近目标至保持距离内 (dist=%s)"
		% mover.position.distance_to(target.position))

	# 目标再跑远，应继续追
	target.position = Vector2(0, 600)
	close = await wait_phys_until(
		func(): return mover.position.distance_to(target.position) <= 48.0, 900)
	assert_true(close, "目标移动后应继续跟随 (dist=%s)"
		% mover.position.distance_to(target.position))
	mover.free()
	target.free()


func test_flee_moves_away() -> void:
	var rig := _make_npc("NpcFlee", 4)
	var mover: GdRoleMover = rig["mover"]
	var brain: GdNpcBrain = rig["brain"]
	brain.flee_radius = 150.0

	var threat := Node2D.new()
	threat.name = "Threat"
	threat.position = Vector2(0, 50)
	_tree().root.add_child(threat)
	brain.follow_target = threat

	var initial_dist := mover.position.distance_to(threat.position)
	# 逃离至 flee_radius 边缘即停止（无摩擦时精确停在半径处）
	var escaped := await wait_phys_until(
		func(): return mover.position.distance_to(threat.position) >= brain.flee_radius - 10.0, 900)
	assert_true(escaped, "逃离应使距离达到 flee_radius 附近 (dist=%s)"
		% mover.position.distance_to(threat.position))
	assert_true(mover.position.distance_to(threat.position) > initial_dist,
		"逃离过程中距离应持续增大")
	mover.free()
	threat.free()


func test_restart_resets() -> void:
	var rig := _make_npc("NpcRestart", 2)
	var mover: GdRoleMover = rig["mover"]
	var brain: GdNpcBrain = rig["brain"]
	brain.patrol_points = PackedVector2Array([Vector2(120, 0), Vector2(240, 0)])
	brain.patrol_loop = true

	# 先走一段
	await wait_phys(60)
	brain.restart()
	assert_eq(brain.get_ai_state(), "idle", "restart 后状态应为 idle")
	# 等待重新开始巡逻仍能正常工作
	var walking := await wait_phys_until(
		func(): return brain.get_ai_state() == "walk", 900)
	assert_true(walking, "restart 后应能重新开始巡逻")
	mover.free()
