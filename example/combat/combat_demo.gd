# combat_demo - 战斗场景演示
#
# 组件全部为 Rust 侧实现，本脚本只负责组装与 HUD：
#   玩家 = GdRoleMover（键盘四向移动）
#        + GdHealth（血量/无敌帧）
#        + GdHurtbox（受击盒，layer 2）
#        + GdShooter（按住左键朝鼠标方向射击，子弹经 GdSpawnPool 池化）
#   敌人 = GdRoleMover（AI 驱动）
#        + GdNpcBrain（HUNTER：游走，玩家进入视野追击，贴近进入攻击相位）
#        + GdHealth + GdHurtbox（layer 3，可被子弹命中）
#        + GdHitbox（攻击盒，mask 2，仅在 attack 相位启用，伤害玩家）
#
# 碰撞层约定：layer 2 = 玩家受击盒（值 2），layer 3 = 敌人受击盒（值 4）
# 运行: 打开 example/combat/index.tscn 直接 F6 运行
extends Node2D

const BULLET_SCENE := "res://example/combat/bullet.tscn"
const ROLE_SCENE := "res://example/role/index.tscn"

const ENEMY_COUNT := 4
const ENEMY_OFFSET := 420.0

var player: GdRoleMover
var player_health: GdHealth
var health_bar: ProgressBar
var status_label: Label
var score_label: Label
var enemies_alive := 0
var kills := 0


func _ready() -> void:
	process_mode = Node.PROCESS_MODE_PAUSABLE
	_make_arena()
	_make_player()
	for i in ENEMY_COUNT:
		_make_hunter(i, Vector2.from_angle(TAU * i / ENEMY_COUNT) * ENEMY_OFFSET)
	_make_aim_cursor()
	_make_hud()


# ---------------------------------------------------------------- 场地

func _make_arena() -> void:
	var bg := ColorRect.new()
	bg.name = "Background"
	bg.color = Color(0.1, 0.11, 0.14)
	bg.size = Vector2(3200, 2000)
	bg.position = -bg.size / 2.0
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(bg)
	move_child(bg, 0)


# ---------------------------------------------------------------- 玩家

func _make_player() -> void:
	player = GdRoleMover.new()
	player.name = "Player"
	player.control_mode = 1  # 键盘
	player.move_mode = 0     # 四向
	player.speed = 230.0
	_attach_visual(player, Vector2(34, 34), Color(0.35, 0.65, 1.0))

	# 血量 + 受击盒（layer 2）
	player_health = GdHealth.new()
	player_health.name = "Health"
	player_health.max_health = 100
	player_health.invincible_time = 0.6
	player.add_child(player_health)
	player_health.s_died.connect(_on_player_died)

	var hurt: GdHurtbox = GdHurtbox.new()
	hurt.name = "Hurtbox"
	hurt.collision_layer = 2
	hurt.collision_mask = 0
	_add_circle_shape(hurt, 22.0)
	player.add_child(hurt)

	# 射击组件（Rust 侧，按住左键朝鼠标射击）
	var shooter: GdShooter = GdShooter.new()
	shooter.name = "Shooter"
	shooter.auto_fire_mouse = true
	shooter.fire_cooldown = 0.22
	shooter.bullet_speed = 520.0
	shooter.bullet_damage = 20.0
	shooter.bullet_lifetime = 1.4
	shooter.muzzle_distance = 26.0
	shooter.bullet_alias = "combat_bullet"
	shooter.bullet_scene_path = BULLET_SCENE
	player.add_child(shooter)

	add_child(player)

	# 相机跟随玩家
	var cam := Camera2D.new()
	cam.name = "Camera"
	player.add_child(cam)
	cam.make_current()


# ---------------------------------------------------------------- 敌人

## 猎手敌人：游走为底，玩家进入视野追击，贴近攻击
func _make_hunter(index: int, pos: Vector2) -> void:
	var npc := GdRoleMover.new()
	npc.name = "Hunter%d" % index
	npc.control_mode = 3  # AI
	npc.move_mode = 0
	npc.speed = 120.0
	npc.position = pos
	_attach_visual(npc, Vector2(30, 30), Color(0.95, 0.38, 0.32))

	# 血量 + 受击盒（layer 3，可被子弹命中）
	var health := GdHealth.new()
	health.name = "Health"
	health.max_health = 60
	health.invincible_time = 0.3
	npc.add_child(health)
	health.s_died.connect(_on_enemy_died.bind(npc))

	var hurt: GdHurtbox = GdHurtbox.new()
	hurt.name = "Hurtbox"
	hurt.collision_layer = 4
	hurt.collision_mask = 0
	_add_circle_shape(hurt, 20.0)
	npc.add_child(hurt)

	# 攻击盒：仅在 AI 进入 attack 相位时启用
	var hitbox: GdHitbox = GdHitbox.new()
	hitbox.name = "Hitbox"
	hitbox.damage = 8.0
	hitbox.cooldown = 1.0
	hitbox.enabled = false
	hitbox.collision_layer = 0
	hitbox.collision_mask = 2  # 检测玩家受击盒
	_add_circle_shape(hitbox, 44.0)
	npc.add_child(hitbox)

	# 猎手 AI
	var brain := GdNpcBrain.new()
	brain.name = "Brain"
	brain.behavior = 5  # HUNTER
	brain.follow_target = player
	brain.sight_range = 300.0
	brain.chase_memory = 2.5
	brain.attack_range = 46.0
	brain.attack_cooldown = 1.0
	brain.wander_radius = 130.0
	npc.add_child(brain)
	brain.s_ai_state_changed.connect(_on_enemy_state.bind(hitbox))

	add_child(npc)
	enemies_alive += 1


func _on_enemy_state(state: String, hitbox: GdHitbox) -> void:
	hitbox.enabled = state == "attack"


func _on_enemy_died(npc: Node2D) -> void:
	kills += 1
	enemies_alive -= 1
	score_label.text = "击杀 %d ｜ 剩余敌人 %d" % [kills, enemies_alive]
	_set_status("敌人 %s 被消灭！" % npc.name)
	var tween := create_tween()
	tween.tween_property(npc, "modulate:a", 0.0, 0.4)
	tween.tween_callback(npc.queue_free)


func _on_player_died() -> void:
	_set_status("玩家阵亡！3 秒后复活")
	await get_tree().create_timer(3.0).timeout
	player_health.revive(player_health.max_health)
	_set_status("玩家已复活")


# ---------------------------------------------------------------- HUD

func _make_hud() -> void:
	var hud := CanvasLayer.new()
	hud.name = "HUD"
	hud.layer = 10
	hud.process_mode = Node.PROCESS_MODE_ALWAYS
	add_child(hud)

	score_label = _label(hud, Vector2(24, 16))
	score_label.text = "击杀 0 ｜ 剩余敌人 %d" % ENEMY_COUNT
	status_label = _label(hud, Vector2(24, 48))

	health_bar = ProgressBar.new()
	health_bar.position = Vector2(24, 84)
	health_bar.size = Vector2(260, 22)
	health_bar.min_value = 0
	health_bar.max_value = 1.0
	hud.add_child(health_bar)

	var hint := _label(hud, Vector2(24, 120))
	hint.text = "WASD 四向移动 ｜ 按住鼠标左键朝鼠标方向射击 ｜ 敌人进入视野会追击"

	var x := 24.0
	_button(hud, "回到角色场景", Vector2(x, 620), func():
		get_tree().change_scene_to_file(ROLE_SCENE)); x += 150
	_button(hud, "重开战斗", Vector2(x, 620), func():
		get_tree().reload_current_scene()); x += 120
	_button(hud, "暂停/继续", Vector2(x, 620), func():
		get_tree().paused = not get_tree().paused)


func _label(parent: Node, pos: Vector2) -> Label:
	var l := Label.new()
	l.position = pos
	parent.add_child(l)
	return l


func _button(parent: Node, text: String, pos: Vector2, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.position = pos
	b.size = Vector2(130, 34)
	b.pressed.connect(on_press)
	parent.add_child(b)
	return b


func _set_status(text: String) -> void:
	status_label.text = text


# ---------------------------------------------------------------- 视觉辅助

## 瞄准光标：鼠标十字圈 + 玩家到鼠标的辅助线
class AimCursor extends Node2D:
	var player: Node2D

	func _process(_delta: float) -> void:
		queue_redraw()

	func _draw() -> void:
		if player == null or not is_instance_valid(player):
			return
		var mouse := get_global_mouse_position()
		draw_line(player.global_position, mouse, Color(1, 0.9, 0.3, 0.22), 1.0)
		draw_arc(mouse, 12.0, 0.0, TAU, 24, Color(1, 0.9, 0.3, 0.9), 2.0)
		draw_line(mouse + Vector2(-18, 0), mouse + Vector2(-6, 0), Color(1, 0.9, 0.3, 0.9), 2.0)
		draw_line(mouse + Vector2(6, 0), mouse + Vector2(18, 0), Color(1, 0.9, 0.3, 0.9), 2.0)


func _make_aim_cursor() -> void:
	var cursor := AimCursor.new()
	cursor.name = "AimCursor"
	cursor.player = player
	add_child(cursor)


func _attach_visual(role: Node2D, size: Vector2, color: Color) -> void:
	var rect := ColorRect.new()
	rect.color = color
	rect.size = size
	rect.position = -size / 2.0
	rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	role.add_child(rect)


func _add_circle_shape(area: Area2D, radius: float) -> void:
	var cs := CollisionShape2D.new()
	var shape := CircleShape2D.new()
	shape.radius = radius
	cs.shape = shape
	area.add_child(cs)
