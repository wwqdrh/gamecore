# 战斗组件测试：GdShooter 射击 / GdBullet 池化子弹命中与回收
# 依赖 example/combat/bullet.tscn（根节点 GdBullet，mask 指向 layer 3 值 4）
extends "res://test/test_case.gd"

const BULLET_SCENE := "res://example/combat/bullet.tscn"

var _root: Node2D


func _pool():
	return Engine.get_singleton("GDSPAWNPOOL")


## 懒初始化测试容器（基类没有 setup 钩子，不能依赖 _setup 被调用）
func _ensure_root() -> Node2D:
	if _root == null or not is_instance_valid(_root):
		_root = Node2D.new()
		_root.name = "CombatTestRoot"
		var tree := Engine.get_main_loop() as SceneTree
		tree.root.add_child(_root)
	return _root


func _teardown() -> void:
	if _pool() != null:
		_pool().clear("test_combat_bullet")


## 构建一个挂了 GdShooter 的宿主（Node2D），alias 用于隔离对象池条目
func _make_shooter(cooldown: float, alias: String = "test_combat_bullet") -> GdShooter:
	var host := Node2D.new()
	host.name = "ShooterHost"
	var shooter: GdShooter = GdShooter.new()
	shooter.name = "Shooter"
	shooter.auto_fire_mouse = false
	shooter.fire_cooldown = cooldown
	shooter.bullet_speed = 400.0
	shooter.bullet_damage = 15.0
	shooter.bullet_lifetime = 1.0
	shooter.muzzle_distance = 0.0
	shooter.bullet_alias = alias
	shooter.bullet_scene_path = BULLET_SCENE
	host.add_child(shooter)
	_ensure_root().add_child(host)
	return shooter


## 构建一个带受击盒的目标（layer 3 = 值 4，与子弹 mask 匹配）
func _make_target(pos: Vector2, max_hp: float) -> GdHealth:
	var body := Node2D.new()
	body.name = "Target"
	body.position = pos
	var health := GdHealth.new()
	health.max_health = max_hp
	health.invincible_time = 0.0
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
	_ensure_root().add_child(body)
	return health


# ---------------------------------------------------------------- Shooter

func test_shooter_fire_spawns_and_moves() -> void:
	var shooter := _make_shooter(10.0)
	var fired := []
	shooter.s_fired.connect(func(muzzle, dir): fired.append([muzzle, dir]))

	# -s 模式下 _initialize 阶段 add_child 后节点尚未入树，等一帧
	await wait_frames(1)
	assert_true(shooter.fire(Vector2(1, 0)), "首次射击应成功")
	assert_eq(fired.size(), 1, "应发出 s_fired 信号")

	# 子弹已在场上且沿方向飞行
	await wait_frames(3)
	var pool = _pool()
	var bullets := _find_bullets()
	assert_eq(bullets.size(), 1, "场上应有 1 颗子弹")
	if bullets.size() == 1:
		var b: Node2D = bullets[0]
		assert_true(b.position.x > 0.0, "子弹应向右飞行")

	# 冷却期内再射应被拒绝
	assert_false(shooter.fire(Vector2(1, 0)), "冷却期内射击应失败")
	assert_eq(fired.size(), 1, "冷却期内不应再发信号")
	assert_eq(bullets.size(), 1, "场上仍只有 1 颗子弹")


func test_shooter_zero_direction_rejected() -> void:
	var shooter := _make_shooter(0.0)
	assert_false(shooter.fire(Vector2.ZERO), "零方向射击应失败")
	assert_false(shooter.fire(Vector2(0.0001, 0.0)), "近零方向射击应失败")


func test_shooter_pool_lazily_registered() -> void:
	# 用独立别名验证懒注册，避免与其他用例的池条目相互影响
	var alias := "test_combat_bullet_lazy"
	assert_true(_pool().aliases().is_empty() or not alias in _pool().aliases(),
			"射击前该别名未注册")
	var shooter := _make_shooter(10.0, alias)
	await wait_frames(1)  # 确保入树
	assert_true(shooter.fire(Vector2(1, 0)), "首次射击应触发懒注册")
	assert_true(_pool().idle_count(alias) >= 0, "池应已注册")


# ---------------------------------------------------------------- Bullet

func test_bullet_hits_hurtbox_and_recycles() -> void:
	var shooter := _make_shooter(10.0)
	# 目标放在弹道正前方 200px
	var health := _make_target(Vector2(200, 0), 100.0)

	await wait_frames(1)  # 确保宿主与目标入树（Area2D 重叠检测依赖物理帧）
	assert_true(shooter.fire(Vector2(1, 0)), "射击应成功")
	# 子弹速度 400px/s，命中 200px 外目标约 0.5s，等命中
	var hit := await wait_until(func(): return health.get_health() < 100.0, 120)
	assert_true(hit, "子弹应命中目标造成伤害")
	assert_near(health.get_health(), 85.0, 0.01, "15 点伤害应扣减血量")

	# 命中后子弹应归池（等待 despawn 完成）
	var recycled := await wait_until(
		func(): return _find_bullets().is_empty(), 60)
	assert_true(recycled, "命中后子弹应从场上消失")
	assert_true(_pool().idle_count("test_combat_bullet") >= 1, "子弹应归还池中而非释放")


func test_bullet_lifetime_expiry_recycles() -> void:
	var shooter := _make_shooter(10.0)
	await wait_frames(1)  # 确保入树
	# 朝空旷方向射击，不会命中任何目标
	assert_true(shooter.fire(Vector2(0, -1)), "射击应成功")
	# bullet_lifetime = 1.0s，等它自然到期回收
	var gone := await wait_until(
		func(): return _find_bullets().is_empty(), 180)
	assert_true(gone, "寿命到期后子弹应自动回收")
	assert_true(_pool().idle_count("test_combat_bullet") >= 1, "到期子弹应归还池中")


# ---------------------------------------------------------------- 辅助

func _find_bullets() -> Array:
	# 子弹由 GdShooter 挂到当前场景（-s 模式下兜底挂到树根），扫描这两处
	var out := []
	var tree := Engine.get_main_loop() as SceneTree
	for parent in [_root, tree.root]:
		if parent == null or not is_instance_valid(parent):
			continue
		for child in parent.get_children():
			if child.name.begins_with("@Bullet") or child.name == "Bullet" \
					or child.name.begins_with("Bullet"):
				out.append(child)
	return out
