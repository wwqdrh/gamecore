# 弹幕系统测试：GdDanmaku 阵型编排 / GdBullet 运动行为（波动/穿透/追踪）
# 依赖 example/combat/bullet.tscn（根节点 GdBullet，mask 指向敌受击盒 layer3 值 4）
#
# 隔离约定：runner 对整个套件只建一个实例（无每用例钩子），本套件每个用例
# 开头 _case_begin() 释放上一用例容器（宿主/靶子/发射器整体销毁）并清场残弹
# ——残靶挡弹道、在飞弹污染计数都是真实踩过的坑。
extends "res://test/test_case.gd"

const BULLET_SCENE := "res://example/combat/bullet.tscn"
const ALIAS := "test_danmaku"

## 当前用例容器（每用例重建，前一用例的随 _case_begin 释放）
var _root: Node2D
## 调试：观测到的死亡/收尾事件（pierce 用例打印用）
var _deaths: Array = []


func _pool():
	return Engine.get_singleton("GDSPAWNPOOL")


## 用例开头调用：立即销毁上一用例容器 + 清池清残弹 + 建新容器。
## 清池安全性：每用例的发射器都是新建实例（pool_ready=false，会重新懒注册）。
## 必须用 free() 而非 queue_free：延迟销毁会让旧 dm 的残余 pending 在清池
## **之后**才发射，产生幽灵弹污染本用例计数（实测 ring 多 1 发的根因）
func _case_begin() -> void:
	var tree := Engine.get_main_loop() as SceneTree
	if _root != null and is_instance_valid(_root):
		_root.free()
		_root = null
	# 清池 + 强杀残弹：在飞弹/池内弹全部作废，杜绝跨用例污染
	if _pool() != null:
		_pool().clear(ALIAS)
		# 显式预注册：清池后各用例 dm 的懒注册存在全量跑下的时序竞态
		# （实测出现过 spawn 时"未注册的池别名"），测试内不依赖懒注册
		_pool().register(ALIAS, BULLET_SCENE, 24)
	for b in _find_bullets_unsafe():
		(b as Node).free()
	await wait_frames(2)
	_root = Node2D.new()
	_root.name = "DanmakuCaseRoot"
	tree.root.add_child(_root)
	await wait_frames(1)


## 构建挂了 GdDanmaku 的宿主（Node2D）
func _make_danmaku(alias: String = ALIAS, cooldown: float = 10.0) -> GdDanmaku:
	var host := Node2D.new()
	host.name = "DanmakuHost"
	var dm: GdDanmaku = GdDanmaku.new()
	dm.name = "Danmaku"
	dm.cast_cooldown = cooldown
	dm.bullet_speed = 400.0
	dm.bullet_damage = 15.0
	dm.bullet_lifetime = 1.0
	dm.muzzle_distance = 0.0
	dm.bullet_alias = alias
	dm.bullet_scene_path = BULLET_SCENE
	host.add_child(dm)
	_root.add_child(host)
	return dm


## 构建带受击盒的目标（layer 3 = 值 4，与子弹 mask 匹配）
## invincible 无敌帧：穿透用例须 >0——子弹在重叠帧间反复 take_damage，
## 无敌帧返回 false 不耗穿透数（与真实战斗语义一致）
func _make_target(pos: Vector2, max_hp: float, invincible: float = 0.0) -> GdHealth:
	var body := Node2D.new()
	body.name = "Target"
	body.position = pos
	var health := GdHealth.new()
	health.max_health = max_hp
	health.invincible_time = invincible
	health.set_health(max_hp)
	body.add_child(health)

	var hurt: GdHurtbox = GdHurtbox.new()
	hurt.collision_layer = 4
	hurt.collision_mask = 0
	var cs := CollisionShape2D.new()
	var shape := CircleShape2D.new()
	shape.radius = 20.0
	cs.shape = shape
	hurt.add_child(cs)
	body.add_child(hurt)
	_root.add_child(body)
	return health


## 扫描场上子弹（发射器挂当前场景 / -s 模式兜底挂树根；不等帧，供清场轮询）
func _find_bullets_unsafe() -> Array:
	var out := []
	var tree := Engine.get_main_loop() as SceneTree
	for parent in [_root, tree.root]:
		if parent == null or not is_instance_valid(parent):
			continue
		for child in parent.get_children():
			if child.get_class() == "GdBullet":
				out.append(child)
	return out


func _find_bullets() -> Array:
	return _find_bullets_unsafe()


# ---------------------------------------------------------------- 阵型编排

func test_danmaku_single_spawns_one() -> void:
	await _case_begin()
	var dm := _make_danmaku()
	var counts := []
	dm.s_cast.connect(func(shape, count): counts.append([str(shape), count]))
	assert_true(dm.cast("single", Vector2(1, 0), {}), "single 施放应成功")
	await wait_frames(2)
	assert_eq(_find_bullets().size(), 1, "single 应发射 1 发")
	assert_eq(counts.size(), 1, "应发 1 次 s_cast")
	if counts.size() == 1:
		assert_eq(counts[0][0], "single", "s_cast 阵型名")
		assert_eq(int(counts[0][1]), 1, "s_cast 子弹总数")


func test_danmaku_fan_spreads_bullets() -> void:
	await _case_begin()
	var dm := _make_danmaku()
	var opts := {"count": 7, "spread": 70.0}
	var casts := []
	dm.s_cast.connect(func(_shape, count): casts.append(int(count)))
	assert_true(dm.cast("fan", Vector2(1, 0), opts), "fan 施放应成功")
	await wait_frames(4)
	# 总数以 s_cast 为准（场上扫描会混入池复用/边缘时序节点，只设下限）
	assert_eq(casts, [7], "fan 应编排 7 发")
	assert_true(_find_bullets().size() >= 7, "场上弹数应不少于编排数")
	# 边缘弹横向展开（正负 y 各有）；阈值放宽防物理帧时序抖动
	var has_up := false
	var has_down := false
	for b in _find_bullets():
		if b.position.y > 3.0:
			has_up = true
		elif b.position.y < -3.0:
			has_down = true
	assert_true(has_up and has_down, "扇形应向 aim 两侧展开")


func test_danmaku_ring_full_circle() -> void:
	await _case_begin()
	var dm := _make_danmaku()
	assert_true(dm.cast("ring", Vector2(1, 0), {"count": 12}), "ring 施放应成功")
	await wait_frames(4)
	var bullets := _find_bullets()
	assert_eq(bullets.size(), 12, "ring 应发射 12 发")
	if bullets.size() == 12:
		# 四象限都应有弹（360° 均布）
		var q := [false, false, false, false]
		for b in bullets:
			if b.position.x > 0 and b.position.y >= 0: q[0] = true
			elif b.position.x <= 0 and b.position.y > 0: q[1] = true
			elif b.position.x < 0: q[2] = true
			else: q[3] = true
		assert_true(q[0] and q[1] and q[2] and q[3], "环形应覆盖四象限")


func test_danmaku_burst_delayed_emission() -> void:
	await _case_begin()
	var dm := _make_danmaku()
	var opts := {"count": 3, "interval": 0.15}
	assert_true(dm.cast("burst", Vector2(1, 0), opts), "burst 施放应成功")
	await wait_frames(2)
	assert_eq(_find_bullets().size(), 1, "首弹应立即发出")
	var done := await wait_until(func(): return _find_bullets().size() >= 3, 120)
	assert_true(done, "连发应在 interval 间隔后补齐 3 发")


func test_danmaku_spiral_volley_finished_signal() -> void:
	await _case_begin()
	var dm := _make_danmaku()
	# spiral：0.3s 内每 0.1s 发 2 臂 → 3 轮 6 发（总数走 s_cast 信号断言，
	# 不数场上弹——短寿命弹在计数前可能已到期回收）
	var opts := {"duration": 0.3, "step": 0.1, "arms": 2, "rotate": 30.0,
		"bullet": {"lifetime": 0.4}}
	var casts := []
	dm.s_cast.connect(func(_shape, count): casts.append(int(count)))
	var finished := [false]
	dm.s_volley_finished.connect(func(): finished[0] = true)
	assert_true(dm.cast("spiral", Vector2(1, 0), opts), "spiral 施放应成功")
	await wait_frames(2)
	assert_eq(_find_bullets().size(), 2, "首轮应发 2 臂")
	var done := await wait_until(func(): return finished[0], 240)
	assert_true(done, "队列清空后应发 s_volley_finished")
	assert_eq(casts.size(), 1, "s_cast 仅施放时发一次")
	if casts.size() == 1:
		assert_eq(casts[0], 6, "s_cast 总数 = 3 轮 × 2 臂")


# ---------------------------------------------------------------- 冷却

func test_danmaku_cooldown_rejects_second_cast() -> void:
	await _case_begin()
	var dm := _make_danmaku(ALIAS, 5.0)
	assert_true(dm.cast("single", Vector2(1, 0), {}), "首次施放应成功")
	assert_false(dm.cast("single", Vector2(1, 0), {}), "冷却期内施放应拒绝")
	assert_near(dm.get_cooldown_left(), 5.0, 0.2, "冷却剩余时间应正确")


func test_danmaku_per_cast_cooldown_override() -> void:
	await _case_begin()
	var dm := _make_danmaku(ALIAS, 10.0)
	assert_true(dm.cast("single", Vector2(1, 0), {"cooldown": 0.1}),
		"首放应成功（冷却按次覆盖）")
	var ok := await wait_until(func(): return dm.cast("single", Vector2(1, 0), {}), 60)
	assert_true(ok, "覆盖冷却 0.1s 后应可再次施放（组件 10s 冷却被覆盖）")


# ---------------------------------------------------------------- 子弹运动行为
# 行为用例手动构造 GdBullet（直接调 setup/setup_motion 公开 API，确定性 100%）；
# danmaku 全链路（池/编排/冷却）已由上方用例覆盖

## 手动构造一颗已配置的子弹（alias 空 = 生命周期结束 queue_free，不经池）
func _make_bullet(pos: Vector2, vel: Vector2, motion: Dictionary) -> Node:
	var b: Node = load(BULLET_SCENE).instantiate()
	b.position = pos
	_root.add_child(b)
	b.setup(vel, 15.0, 3.0, "", 0.0)
	b.setup_motion(
		float(motion.get("angular_speed", 0.0)),
		float(motion.get("wave_amplitude", 0.0)),
		float(motion.get("wave_frequency", 0.0)),
		str(motion.get("homing_group", "")),
		float(motion.get("homing_turn_rate", 0.0)),
		int(motion.get("pierce", 0)))
	return b


func test_bullet_wave_oscillates_laterally() -> void:
	await _case_begin()
	var b := _make_bullet(Vector2(0, 0), Vector2(300, 0),
		{"wave_amplitude": 40.0, "wave_frequency": 4.0})
	var ys := []
	for i in 40:
		await wait_physics_frames(1)
		if not is_instance_valid(b):
			break
		ys.append((b as Node2D).position.y)
	assert_true(ys.size() >= 10, "波动弹应存活足够帧数")
	if ys.size() >= 10:
		var spread: float = ys.max() - ys.min()
		assert_true(spread > 10.0,
			"正弦横摆应有可观测的横向振荡（实测 %f px）" % spread)


func test_bullet_pierce_passes_through_targets() -> void:
	await _case_begin()
	var hp1 := _make_target(Vector2(60, 0), 100.0, 0.6)
	var hp2 := _make_target(Vector2(120, 0), 100.0, 0.6)
	var b := _make_bullet(Vector2(0, 0), Vector2(200, 0), {"pierce": 1})
	var both_hit := await wait_until(
		func(): return hp1.get_health() < 100.0 and hp2.get_health() < 100.0, 240)
	assert_true(both_hit, "穿透 1 应连续命中两个目标")
	var gone := await wait_until(func(): return not is_instance_valid(b), 120)
	assert_true(gone, "穿透数耗尽后子弹应回收")


func test_bullet_no_pierce_stops_at_first_target() -> void:
	await _case_begin()
	var hp1 := _make_target(Vector2(60, 0), 100.0)
	var hp2 := _make_target(Vector2(120, 0), 100.0)
	var b := _make_bullet(Vector2(0, 0), Vector2(200, 0), {})
	var first_hit := await wait_until(func(): return hp1.get_health() < 100.0, 240)
	assert_true(first_hit, "首目标应命中")
	var gone := await wait_until(func(): return not is_instance_valid(b), 60)
	assert_true(gone, "无穿透命中即回收")
	assert_near(hp2.get_health(), 100.0, 0.01, "后续目标不应被命中")


func test_bullet_homing_curves_toward_target() -> void:
	await _case_begin()
	# 目标在弹道右上侧（纯直线永远不会命中）
	var target := Node2D.new()
	target.name = "HomingTarget"
	target.position = Vector2(200, 160)
	target.add_to_group("test_homing_targets")
	_root.add_child(target)
	var b := _make_bullet(Vector2(0, 0), Vector2(280, 0),
		{"homing_group": "test_homing_targets", "homing_turn_rate": 6.0})
	var curved_cond := func() -> bool:
		return is_instance_valid(b) and (b as Node2D).position.y > 20.0
	var curved := await wait_until(curved_cond, 240)
	assert_true(curved, "追踪弹应朝目标方向转弯（y 持续增大）")
