# 原型脚手架能力测试：EventBus / SpawnPool / Health / Hurtbox / Hitbox / AttributeSet / InputBuffer / DebugDraw / 全局暂停
extends "res://test/test_case.gd"


func _bus():
	return Engine.get_singleton("GDEVENTBUS")


func _pool():
	return Engine.get_singleton("GDSPAWNPOOL")


func _dd():
	return Engine.get_singleton("GDDEBUGDRAW")


func _make_health(max_hp: float, invincible: float) -> GdHealth:
	var h := GdHealth.new()
	h.max_health = max_hp
	h.invincible_time = invincible
	h.set_health(max_hp)
	return h


# ---------------------------------------------------------------- EventBus

func test_eventbus_pubsub() -> void:
	var bus = _bus()
	bus.clear("test_ev")
	var hits := []
	var id1: int = bus.subscribe("test_ev", func(a): hits.append(["s1", a]))
	bus.subscribe_once("test_ev", func(a): hits.append(["once", a]))
	assert_eq(bus.subscriber_count("test_ev"), 2, "订阅后应有 2 个监听者")

	bus.publish("test_ev", [7])
	assert_eq(hits.size(), 2, "首次发布应触发两个订阅")
	assert_eq(hits[0][1], 7, "参数应透传")
	assert_eq(bus.subscriber_count("test_ev"), 1, "once 订阅触发后应移除")

	bus.publish("test_ev", [8])
	assert_eq(hits.size(), 3, "二次发布只剩普通订阅")

	assert_true(bus.unsubscribe("test_ev", id1), "按 id 取消订阅应成功")
	assert_eq(bus.subscriber_count("test_ev"), 0, "取消后应为 0")
	bus.clear("test_ev")


func test_eventbus_isolated_events() -> void:
	var bus = _bus()
	bus.clear("ev_a")
	bus.clear("ev_b")
	var a_hits := []
	bus.subscribe("ev_a", func(): a_hits.append(1))
	bus.publish("ev_b", [])
	assert_eq(a_hits.size(), 0, "不同事件不应互相触发")
	bus.clear("ev_a")
	bus.clear("ev_b")


# ---------------------------------------------------------------- SpawnPool

func test_spawnpool_reuse() -> void:
	var pool = _pool()
	var scene: PackedScene = load("res://example/prototype/bullet.tscn")
	assert_true(pool.register_scene("test_bullet", scene, 3), "注册池应成功")
	assert_eq(pool.idle_count("test_bullet"), 3, "预热 3 个闲置实例")

	var root: Node = (Engine.get_main_loop() as SceneTree).root
	var b1 = pool.spawn("test_bullet", root, Vector2(10, 20))
	assert_true(b1 != null, "spawn 应返回节点")
	assert_eq(b1.position, Vector2(10, 20), "Node2D 位置应被设置")
	assert_eq(pool.idle_count("test_bullet"), 2, "取走一个闲置实例")
	assert_eq(pool.created_count("test_bullet"), 3, "未超发时不应新增实例化")

	# 取空后继续 spawn 会新实例化
	var b2 = pool.spawn("test_bullet", root, Vector2.ZERO)
	var b3 = pool.spawn("test_bullet", root, Vector2.ZERO)
	var b4 = pool.spawn("test_bullet", root, Vector2.ZERO)
	assert_eq(pool.created_count("test_bullet"), 4, "取空后再取应实例化 1 个新节点")

	# 回收复用
	assert_true(pool.despawn(b1), "回收应成功")
	assert_eq(pool.idle_count("test_bullet"), 1, "回收后闲置 +1")
	var created_before: int = pool.created_count("test_bullet")
	var b5 = pool.spawn("test_bullet", root, Vector2.ZERO)
	assert_eq(pool.created_count("test_bullet"), created_before, "回收节点应被复用而非新实例化")

	# 清理
	for n in [b2, b3, b4, b5]:
		pool.despawn(n)
	pool.clear("test_bullet")


# ---------------------------------------------------------------- Health

func test_health_damage_heal_death() -> void:
	var h := _make_health(100.0, 0.0)
	var died_count := []
	h.s_died.connect(func(): died_count.append(1))

	assert_true(h.take_damage(30.0), "受伤应生效")
	assert_eq(h.get_health(), 70.0, "血量应扣减")
	assert_true(h.heal(20.0), "治疗应生效")
	assert_eq(h.get_health(), 90.0, "血量应恢复")

	# 超量治疗钳制
	h.heal(999.0)
	assert_eq(h.get_health(), 100.0, "治疗不应超过最大血量")

	# 死亡信号
	h.take_damage(150.0)
	assert_eq(h.get_health(), 0.0, "血量归零")
	assert_eq(died_count.size(), 1, "应触发一次死亡信号")
	assert_false(h.is_alive(), "应处于死亡状态")
	assert_false(h.take_damage(10.0), "死亡后不再受伤")

	# 复活
	h.revive(50.0)
	assert_true(h.is_alive(), "复活后应存活")
	assert_eq(h.get_health(), 50.0, "复活到指定血量")
	h.free()


func test_health_invincible_and_god_mode() -> void:
	# 无敌帧
	var h := _make_health(100.0, 10.0)
	assert_true(h.take_damage(10.0), "首次受伤应生效")
	assert_true(h.is_invincible(), "受伤后应处于无敌帧")
	assert_false(h.take_damage(10.0), "无敌帧内受伤应无效")
	assert_eq(h.get_health(), 90.0, "无敌帧内血量不变")
	h.free()

	# 上帝模式（无无敌帧干扰）
	var g := _make_health(100.0, 0.0)
	g.set_god_mode(true)
	assert_false(g.take_damage(10.0), "上帝模式下受伤应无效")
	assert_eq(g.get_health(), 100.0, "上帝模式血量不变")
	g.free()


# ---------------------------------------------------------------- Hurtbox 转发

func test_hurtbox_forwards_to_health() -> void:
	var tree := Engine.get_main_loop() as SceneTree
	# 结构：Holder(Node2D) + GdHealth + GdHurtbox（自动找父节点血量）
	var holder := Node2D.new()
	var h := _make_health(100.0, 0.0)
	holder.add_child(h)
	var hurt: GdHurtbox = GdHurtbox.new()
	holder.add_child(hurt)
	tree.root.add_child(holder)
	await wait_frames(1)  # 确保入树（_initialize 中 add_child 的入树是延迟的）

	var hurt_hits := []
	hurt.s_hurt.connect(func(amount): hurt_hits.append(amount))

	assert_true(hurt.take_damage(25.0), "受击盒转发应生效")
	assert_eq(h.get_health(), 75.0, "血量组件应扣减")
	assert_eq(hurt_hits.size(), 1, "应触发 s_hurt 信号")
	assert_eq(hurt_hits[0], 25.0, "信号应携带伤害值")

	# health_path 显式指向（挂在别的节点下）
	var hurt2: GdHurtbox = GdHurtbox.new()
	var container := Node2D.new()
	container.add_child(hurt2)
	tree.root.add_child(container)
	hurt2.health_path = h.get_path() as NodePath
	assert_true(hurt2.take_damage(5.0), "显式路径转发应生效")
	assert_eq(h.get_health(), 70.0, "显式路径血量扣减")

	holder.free()
	container.free()


# ---------------------------------------------------------------- AttributeSet

func test_attribute_set() -> void:
	var s := GdAttributeSet.new()
	var changes := []
	s.s_attribute_changed.connect(func(key, old_v, new_v): changes.append([key, old_v, new_v]))

	s.set_attribute("strength", 10)
	assert_eq(s.get_attribute_float("strength", 0), 10.0, "数值属性应可读写")
	s.set_attribute("strength", 15)
	assert_eq(s.get_attribute_float("strength", 0), 15.0, "覆盖后应为新值")
	assert_eq(changes.size(), 2, "变更信号应触发两次")
	assert_eq(changes[1][1], 10, "信号应携带旧值")
	assert_eq(changes[1][2], 15, "信号应携带新值")

	assert_true(s.has_attribute("strength"), "应存在属性")
	assert_eq(s.attribute_count(), 1, "属性数量")
	s.erase_attribute("strength")
	assert_false(s.has_attribute("strength"), "删除后不应存在")

	# 相同值不触发信号
	s.set_attribute("agi", 5)
	var before: int = changes.size()
	s.set_attribute("agi", 5)
	assert_eq(changes.size(), before, "相同值赋值不应触发信号")
	s.free()


# ---------------------------------------------------------------- InputBuffer

func test_input_buffer_window() -> void:
	var buf := GdInputBuffer.new()
	buf.buffer_window = 0.15
	buf.watch_action("ui_accept")
	assert_false(buf.is_buffered("ui_accept"), "初始无缓冲")
	assert_false(buf.consume("ui_accept"), "无缓冲消费应失败")

	# 注入按下（等效真实 just_pressed）
	buf.notify_pressed("ui_accept")
	assert_true(buf.is_buffered("ui_accept"), "注入后应有缓冲")
	assert_true(buf.consume("ui_accept"), "窗口内消费应成功")
	assert_false(buf.is_buffered("ui_accept"), "消费后应清空（一次性）")
	assert_false(buf.consume("ui_accept"), "重复消费应失败")

	buf.clear_all()
	buf.free()


# ---------------------------------------------------------------- DebugDraw

func test_debugdraw_shapes() -> void:
	var dd = _dd()
	dd.clear()
	assert_eq(dd.shape_count(), 0, "初始应为空")

	dd.draw_line(Vector2.ZERO, Vector2(100, 0), Color.RED, 0.0)
	dd.draw_circle(Vector2(50, 50), 32.0, false, Color.BLUE, 5.0)
	dd.draw_ray(Vector2.ZERO, Vector2(1, 1), 80.0, Color.YELLOW, 5.0)
	assert_eq(dd.shape_count(), 3, "绘制后应有 3 条图形")

	dd.clear()
	assert_eq(dd.shape_count(), 0, "清空后应为 0")


# ---------------------------------------------------------------- 全局暂停

func test_global_pause() -> void:
	var tree := Engine.get_main_loop() as SceneTree
	var root_node := GdSceneRoot.new()
	tree.root.add_child(root_node)
	# 等一帧确保节点完成入树（_initialize 中 add_child 的入树是延迟的）
	await wait_frames(1)

	root_node.set_game_paused(true)
	assert_true(tree.is_paused(), "暂停后场景树应处于暂停状态")
	assert_true(root_node.is_game_paused(), "is_game_paused 应返回 true")

	root_node.set_game_paused(false)
	assert_false(tree.is_paused(), "恢复后场景树应继续运行")

	root_node.free()
