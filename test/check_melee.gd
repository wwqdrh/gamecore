# GdMelee 近战组件验收 —— 挥击判定 / 窗口复位 / 冷却 / 命中伤害
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://test/check_melee.gd
#
# 覆盖：
#   1. 装配：ready 自动创建内嵌 SwingHitbox（GdHitbox 子节点 + 矩形判定形状），
#      layer=0 / monitorable=false / mask=target_mask
#   2. 挥击：swing(direction) → 判定盒定位到 dir*offset、is_swinging=true
#   3. 冷却：窗口期/冷却期再次 swing 返回 false
#   4. 窗口复位：swing_window 后 is_swinging=false（判定盒复位）
#   5. 命中：范围外敌人不掉血 → 范围内敌人经 GdHurtbox 掉血（防御减伤链路）
#   6. stop()：立即终止挥击
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[Melee] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	# ---- 装配：宿主 + 近战组件 + 敌人（Health + Hurtbox layer3） ----
	var host := Node2D.new()
	host.name = "MeleeHost"
	root.add_child(host)
	host.position = Vector2(100, 100)

	var melee := GdMelee.new()
	melee.name = "Melee"
	melee.auto_attack_mouse = false  # 单元测试直调 swing，不走输入路由
	melee.attack_damage = 10.0
	melee.attack_cooldown = 0.3
	melee.attack_range = 56.0
	melee.attack_width = 48.0
	melee.swing_window = 0.3  # 拉长窗口便于断言
	host.add_child(melee)
	await physics_frame
	await physics_frame

	# 1. 内嵌挥击盒
	var hb: Node = melee.get_node_or_null("SwingHitbox")
	check(hb != null, "ready 应自动创建 SwingHitbox（内嵌 GdHitbox）")
	if hb == null:
		_finish()
		return
	check(int(hb.get("collision_layer")) == 0, "SwingHitbox 应为纯攻击方 layer=0")
	check(int(hb.get("collision_mask")) == 4, "SwingHitbox mask 应默认 4（敌受击盒 layer3）")
	check(not bool(hb.get("monitorable")), "SwingHitbox 应 monitorable=false")
	check(hb.get_child_count() >= 1, "SwingHitbox 应带矩形判定形状")

	# 敌人：范围内（host 右侧 50px，判定盒中心 x=128 半长 28 → 覆盖到 x=156）
	var enemy := Node2D.new()
	enemy.name = "Enemy"
	root.add_child(enemy)
	enemy.position = Vector2(150, 100)
	var ehealth: Node = GdHealth.new()
	ehealth.set("max_health", 100.0)
	ehealth.set("invincible_time", 0.0)
	enemy.add_child(ehealth)
	var hurt: Node = GdHurtbox.new()
	hurt.set("collision_layer", 4)
	hurt.set("collision_mask", 0)
	var shape := CollisionShape2D.new()
	var rect := RectangleShape2D.new()
	rect.size = Vector2(16, 16)
	shape.shape = rect
	hurt.add_child(shape)
	enemy.add_child(hurt)

	# 范围外敌人（y 偏移 200，不在判定盒内）
	var far := Node2D.new()
	far.name = "FarEnemy"
	root.add_child(far)
	far.position = Vector2(150, 300)
	var fhealth: Node = GdHealth.new()
	fhealth.set("max_health", 100.0)
	fhealth.set("invincible_time", 0.0)
	far.add_child(fhealth)
	var fhurt: Node = GdHurtbox.new()
	fhurt.set("collision_layer", 4)
	fhurt.set("collision_mask", 0)
	var fshape := CollisionShape2D.new()
	fshape.shape = rect
	fhurt.add_child(fshape)
	far.add_child(fhurt)

	await physics_frame
	await physics_frame

	# ---- 2. 挥击 + 5. 命中 ----
	var swings := [0]
	melee.connect("s_swing", func(_d) -> void: swings[0] += 1)
	check(melee.swing(Vector2(1, 0)), "swing(→) 应成功")
	check(bool(melee.is_swinging()), "挥击后应处于挥击窗口中")
	check(int(hb.get("position").x) > 0, "判定盒应定位到宿主前方（x>0）")
	await physics_frame
	await physics_frame
	check(swings[0] == 1, "挥击应触发 s_swing 一次，实际 %d" % swings[0])
	var ehp: float = ehealth.call("get_health")
	var fhp: float = fhealth.call("get_health")
	check(ehp < 100.0, "范围内敌人应被命中掉血，实际 %s" % str(ehp))
	check(fhp == 100.0, "范围外敌人不应掉血")

	# ---- 3. 冷却：冷却期内再次挥击应被拒 ----
	check(not melee.swing(Vector2(1, 0)), "冷却期内 swing 应返回 false")

	# ---- 4. 窗口复位 ----
	await create_timer(0.4).timeout
	await physics_frame
	check(not bool(melee.is_swinging()), "窗口结束后应复位 is_swinging=false")
	check(not bool(hb.get("enabled")), "窗口结束后判定盒应复位 enabled=false")

	# 冷却结束恢复
	await create_timer(0.1).timeout
	check(melee.swing(Vector2(0, 1)), "冷却结束后 swing 应恢复可用")
	await create_timer(0.4).timeout
	await physics_frame

	# ---- 6. stop() 立即终止 ----
	melee.swing(Vector2(1, 0))
	await physics_frame
	melee.stop()
	check(not bool(melee.is_swinging()), "stop() 应立即终止挥击")
	check(not bool(hb.get("enabled")), "stop() 后判定盒应复位")

	if ok:
		print("[Melee] RESULT=PASS")
	else:
		print("[Melee] RESULT=FAIL")
	_finish()


func _finish() -> void:
	quit(0 if ok else 1)
