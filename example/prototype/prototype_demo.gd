# prototype_demo - 原型脚手架能力演示
#
# 一屏演示框架补齐的原型层能力（对照 prototype_kit 设计）：
#   1. EventBus        击中/受伤事件广播（订阅方无需拿节点引用）
#   2. SpawnPool       子弹对象池（spawn/despawn 复用，观察 created_count 不增长）
#   3. Health/Hitbox   玩家血量 + 敌人攻击盒持续造成伤害（无敌帧可观察到掉血节奏）
#   4. DebugDraw       一键绘制视野圈/射线/连线（2 秒自动消失）
#   5. 全局暂停        暂停游戏世界，HUD 按钮仍可操作（CanvasLayer ALWAYS）
#
# 结构（index.tscn）：
#   GdSceneRoot
#   └─ Demo(Node2D, PAUSABLE)      游戏世界：玩家(GdHealth+GdHurtbox)、敌人(GdHitbox)
#      └─ HUD(CanvasLayer, ALWAYS)  按钮/血条/分数，暂停时仍可交互
extends Node2D

## 子弹场景路径
const BULLET_SCENE := "res://example/prototype/bullet.tscn"

var scene_root: GdSceneRoot
var player: Node2D
var enemy: Node2D
var health: GdHealth
var health_bar: ProgressBar
var status_label: Label
var score_label: Label
var god_btn: Button
var pause_btn: Button

var _score := 0


func _ready() -> void:
	process_mode = Node.PROCESS_MODE_PAUSABLE
	scene_root = get_parent()

	_build_world()
	_build_hud()

	# 对象池：注册并预热 8 发子弹
	var pool = Engine.get_singleton("GDSPAWNPOOL")
	pool.register("bullet", BULLET_SCENE, 8)

	# EventBus：血量组件发事件，HUD 订阅（双方互不持引用）
	var bus = Engine.get_singleton("GDEVENTBUS")
	bus.subscribe("player_damaged", _on_player_damaged)
	bus.subscribe("player_healed", _on_player_healed)
	bus.subscribe("player_died", _on_player_died)
	_refresh_health()


# ---------------------------------------------------------------- 世界

func _build_world() -> void:
	# 玩家：GdHealth + GdHurtbox（hurtbox 的父节点上有 GdHealth 即自动转发）
	player = Node2D.new()
	player.name = "Player"
	player.position = Vector2(420, 300)
	add_child(player)

	health = GdHealth.new()
	health.max_health = 100
	health.invincible_time = 0.8
	player.add_child(health)

	health.s_damaged.connect(func(amount: float, current: float):
		Engine.get_singleton("GDEVENTBUS").publish("player_damaged", [amount, current]))
	health.s_healed.connect(func(amount: float, current: float):
		Engine.get_singleton("GDEVENTBUS").publish("player_healed", [amount, current]))
	health.s_died.connect(func():
		Engine.get_singleton("GDEVENTBUS").publish("player_died", []))

	var hurt: GdHurtbox = GdHurtbox.new()
	var hurt_shape := CollisionShape2D.new()
	var hurt_circle := CircleShape2D.new()
	hurt_circle.radius = 36.0
	hurt_shape.shape = hurt_circle
	hurt.add_child(hurt_shape)
	player.add_child(hurt)

	# 玩家示意图形
	var body := Polygon2D.new()
	body.polygon = PackedVector2Array([Vector2(-24, -24), Vector2(24, -24), Vector2(24, 24), Vector2(-24, 24)])
	body.color = Color(0.35, 0.65, 1.0)
	player.add_child(body)

	# 敌人：GdHitbox 与玩家重叠，按冷却持续造成伤害
	enemy = Node2D.new()
	enemy.name = "Enemy"
	enemy.position = Vector2(500, 300)
	add_child(enemy)

	var hitbox: GdHitbox = GdHitbox.new()
	hitbox.damage = 10.0
	hitbox.cooldown = 1.0
	var hit_shape := CollisionShape2D.new()
	var hit_circle := CircleShape2D.new()
	hit_circle.radius = 80.0
	hit_shape.shape = hit_circle
	hitbox.add_child(hit_shape)
	enemy.add_child(hitbox)

	hitbox.s_hit.connect(func(_target: String):
		Engine.get_singleton("GDEVENTBUS").publish("enemy_hit", []))

	var enemy_body := Polygon2D.new()
	enemy_body.polygon = PackedVector2Array([Vector2(-18, -18), Vector2(18, -18), Vector2(18, 18), Vector2(-18, 18)])
	enemy_body.color = Color(1.0, 0.35, 0.3)
	enemy.add_child(enemy_body)


# ---------------------------------------------------------------- HUD

func _build_hud() -> void:
	var hud := CanvasLayer.new()
	hud.name = "HUD"
	hud.layer = 10
	hud.process_mode = Node.PROCESS_MODE_ALWAYS
	add_child(hud)

	score_label = _label(hud, Vector2(24, 16))
	status_label = _label(hud, Vector2(24, 48))
	health_bar = ProgressBar.new()
	health_bar.position = Vector2(24, 84)
	health_bar.size = Vector2(260, 22)
	health_bar.min_value = 0
	health_bar.max_value = 1.0
	hud.add_child(health_bar)

	var hint := _label(hud, Vector2(24, 120))
	hint.text = "敌人贴身每秒造成 10 伤害（受无敌帧节流）｜伤害数字经 EventBus 广播"

	var x := 24.0
	_button(hud, "伤害 -10", Vector2(x, 620), func(): health.take_damage(10.0)); x += 120
	_button(hud, "治疗 +20", Vector2(x, 620), func(): health.heal(20.0)); x += 120
	god_btn = _button(hud, "上帝模式:关", Vector2(x, 620), func():
		health.god_mode = not health.god_mode
		god_btn.text = "上帝模式:%s" % ("开" if health.god_mode else "关")); x += 150
	_button(hud, "发射子弹 x5", Vector2(x, 620), _fire_bullets); x += 150
	_button(hud, "绘制调试图形", Vector2(x, 620), _draw_debug); x += 160
	pause_btn = _button(hud, "暂停", Vector2(x, 620), func():
		var paused := not scene_root.is_game_paused()
		scene_root.set_game_paused(paused)
		pause_btn.text = "继续" if paused else "暂停")


func _label(parent: Node, pos: Vector2) -> Label:
	var l := Label.new()
	l.position = pos
	l.text = ""
	parent.add_child(l)
	return l


func _button(parent: Node, text: String, pos: Vector2, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.position = pos
	b.size = Vector2(110, 34)
	b.pressed.connect(on_press)
	parent.add_child(b)
	return b


# ---------------------------------------------------------------- 演示动作

func _fire_bullets() -> void:
	var pool = Engine.get_singleton("GDSPAWNPOOL")
	for i in 5:
		var pos := player.position + Vector2(40, -40 + i * 20.0)
		var bullet = pool.spawn("bullet", self, pos)
		if bullet != null:
			bullet.velocity = Vector2(randf_range(240, 420), 0)
	_set_status("池状态: 闲置=%d 累计实例化=%d" % [pool.idle_count("bullet"), pool.created_count("bullet")])


func _draw_debug() -> void:
	var dd = Engine.get_singleton("GDDEBUGDRAW")
	var vp := get_viewport_rect().size
	# 视野圈（实心浅红）
	dd.draw_circle(player.position, 120.0, true, Color(1, 0, 0, 0.12), 2.0)
	# 视野圈描边
	dd.draw_circle(player.position, 120.0, false, Color(1, 0.2, 0.2, 0.8), 2.0)
	# 朝向射线（带箭头）
	dd.draw_ray(player.position, Vector2(1, 0), 160.0, Color(1, 0.9, 0.2), 2.0)
	# 连线到敌人
	dd.draw_line(player.position, enemy.position, Color(0.4, 1, 0.4, 0.8), 2.0)
	# 边界框
	dd.draw_rect(Vector2.ZERO, Vector2(vp.x, vp.y), false, Color(0.5, 0.7, 1, 0.5), 2.0)


func _on_player_damaged(args: Array) -> void:
	_set_status("受伤 %.0f，剩余 %.0f" % [args[0], args[1]])
	_refresh_health()


func _on_player_healed(args: Array) -> void:
	_set_status("恢复 %.0f，当前 %.0f" % [args[0], args[1]])
	_refresh_health()


func _on_player_died(_args: Array) -> void:
	_set_status("玩家死亡！3 秒后复活")
	await get_tree().create_timer(3.0).timeout
	health.revive(health.max_health)
	_refresh_health()


func _refresh_health() -> void:
	if health_bar != null and health != null:
		health_bar.value = health.get_health_ratio()


func _set_status(text: String) -> void:
	status_label.text = text
