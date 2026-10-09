# 敌人小怪基类（XiuEnemyBase）—— 属性 + 视野追击 + 接触伤害 + 受击
#
# 组合框架能力（零自写 AI/战斗逻辑）：
#   - GdRoleMover MODE_GRID：网格移动（Brain 驱动 grid_path 队列，禁斜线）
#   - GdNpcBrain AI_HUNTER：游走为底，玩家进 sight_range 追击（经地图 BFS
#     逐格走，不穿不可行走地形），进 attack_range 停下面向玩家（attack 相位）
#   - GdHealth：血量（受击无敌帧、s_died 死亡）
#   - GdHurtbox：受击盒（collision_layer=4 敌方 layer3；defense 减伤，
#     实际伤害 = max(1, 伤害 - 防御)）
#   - GdHitbox：攻击盒（collision_mask=2 玩家受击 layer2，常开接触伤害，
#     cooldown + 玩家无敌帧防连扣）
#
# 碰撞层约定：layer2(值2)=玩家受击盒、layer3(值4)=敌受击盒。
# 玩家远程：GdShooter 发 GdBullet（mask=4 打敌受击盒），见 role/player/。
#
# 变体小怪：不需要子脚本——不同颜色的 tscn 直接覆写导出值
# （max_health/attack/defense/body_color/speed/sight_range/exp_reward/...）。
# 子类扩展（新行为/新技能）时 extends XiuEnemyBase。
class_name XiuEnemyBase
extends GdRoleMover

## 头顶显示名
@export var display_name := "小怪"
## 最大血量
@export var max_health := 40.0
## 攻击力（Hitbox.damage，接触玩家时扣除）
@export var attack := 6.0
## 防御力（受击减伤，实际伤害 = max(1, 伤害 - 防御)）
@export var defense := 0.0
## 移动速度（像素/秒）
@export var move_speed := 80.0
## 视野半径（像素）：玩家进入即追击
@export var sight_range := 256.0
## 攻击距离（像素）：追到该距离内停下面向玩家（接触由攻击盒判定）
@export var attack_range := 28.0
## 无目标时的游走半径（像素）
@export var wander_radius := 64.0
## 站位格子（-1 = 用实例化坐标换算；落在不可行走地形 BFS 吸附）
@export var enemy_cell := Vector2i(-1, -1)
## 身体颜色（不同小怪不同色）
@export var body_color := Color(0.45, 0.8, 0.4)
## 击败经验（玩家击败本敌人获得，写入 XiuCharacterState 修为）
@export var exp_reward := 20

const AI_HUNTER := 5

var brain: Node = null
var health: Node = null

var _snapped := false
var _target_wired := false
var _dying := false


func _ready() -> void:
	control_mode = 0  # CONTROL_NONE：Brain 经 grid_path 队列驱动
	move_mode = 2     # MODE_GRID
	speed = move_speed
	if grid_cell_size <= 0.0:
		grid_cell_size = 32.0
	if grid_map_path.is_empty():
		grid_map_path = NodePath("..")  # 敌人作为地图场景子节点实例化
	_make_combat()
	_make_ai()
	_make_visual()


func _process(_delta: float) -> void:
	if _dying:
		return
	if not _snapped:
		_snap_to_map()
	if not _target_wired:
		_wire_target()
	if _snapped and _target_wired:
		set_process(false)


# ---- 战斗三件套：Health / Hurtbox / Hitbox ----

func _make_combat() -> void:
	health = GdHealth.new()
	health.name = "Health"
	health.max_health = max_health
	health.invincible_time = 0.4
	add_child(health)
	health.s_died.connect(_on_died)
	health.s_damaged.connect(_on_damaged)

	var hurt := GdHurtbox.new()
	hurt.name = "Hurtbox"
	hurt.collision_layer = 4   # 敌受击盒 layer3
	hurt.collision_mask = 0
	hurt.defense = defense
	hurt.add_child(_make_shape(18.0))
	add_child(hurt)

	var hit := GdHitbox.new()
	hit.name = "Hitbox"
	hit.damage = attack
	hit.cooldown = 1.0         # 接触伤害冷却（玩家侧另有无敌帧）
	hit.enabled = true
	hit.collision_layer = 0
	hit.collision_mask = 2     # 玩家受击盒 layer2
	hit.add_child(_make_shape(20.0))
	add_child(hit)


func _make_ai() -> void:
	brain = GdNpcBrain.new()
	brain.name = "Brain"
	brain.behavior = AI_HUNTER
	brain.sight_range = sight_range
	brain.attack_range = attack_range
	brain.wander_radius = wander_radius
	brain.chase_memory = 2.5   # 失去视野后追击记忆
	brain.attack_cooldown = 1.0
	add_child(brain)


## 绑定追击目标（"player" 分组；分组节点可能晚于本节点 ready，_process 重试）
func _wire_target() -> void:
	var player: Node = get_tree().get_first_node_in_group("player")
	if player == null:
		return
	brain.follow_target = player
	_target_wired = true


# ---- 落位：吸附到父地图最近可行走格（同 NPC 传送点规则） ----

func _snap_to_map() -> void:
	var map := get_parent()
	if map == null or not map.has_method("is_walkable") \
			or not map.has_method("get_cell_size_px"):
		return
	var cs: int = map.get_cell_size_px()
	var cell := enemy_cell
	if cell.x < 0:
		cell = map.world_to_cell(global_position)
	var target := _nearest_walkable(map, cell)
	global_position = Vector2((target.x + 0.5) * cs, (target.y + 0.5) * cs)
	if target != cell:
		push_warning("[Enemy] %s 站位格 %s 不可行走，吸附到 %s" % [display_name, cell, target])
	_snapped = true
	brain.restart()  # 游走中心以吸附后位置为准


func _nearest_walkable(map: Node, cell: Vector2i) -> Vector2i:
	if map.is_walkable(cell):
		return cell
	var w: int = map.get_map_width()
	var h: int = map.get_map_height()
	var queue: Array = [cell]
	var seen := {cell: true}
	while not queue.is_empty():
		var cur: Vector2i = queue.pop_front()
		if map.is_walkable(cur):
			return cur
		for off in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
			var nxt: Vector2i = cur + off
			if nxt.x < 0 or nxt.y < 0 or nxt.x >= w or nxt.y >= h or seen.has(nxt):
				continue
			seen[nxt] = true
			queue.append(nxt)
	return cell


# ---- 受击 / 死亡表现 ----

func _on_damaged(_amount: float, _current: float) -> void:
	# 受击闪白 + 血条刷新
	modulate = Color(2.0, 2.0, 2.0)
	var tw := create_tween()
	tw.tween_property(self, "modulate", Color.WHITE, 0.18)
	_update_hp_bar()


func _on_died() -> void:
	_dying = true
	if brain != null:
		brain.enable = false
	set_process(false)
	_award_exp()
	# 倒地淡出后移除
	var tw := create_tween()
	tw.tween_property(self, "modulate", Color(1, 1, 1, 0), 0.35)
	tw.tween_callback(queue_free)


## 击败奖励：exp_reward 写入角色状态修为（阶段内升级由 add_exp 自动处理），
## 并在尸体位置弹出「+N 修为」飘字
func _award_exp() -> void:
	if exp_reward <= 0:
		return
	var state := XiuCharacterState.ins()
	var gained: int = state.add_exp(exp_reward)
	print("[Enemy] 击败 %s：+%d 修为（升 %d 级 → %s）" % [
		display_name, exp_reward, gained, state.get_realm_title()])
	_spawn_exp_popup(exp_reward)


## 飘字：+N 修为（上浮渐隐，随尸体一起释放）
func _spawn_exp_popup(amount: int) -> void:
	var label := Label.new()
	label.text = "+%d 修为" % amount
	label.add_theme_font_size_override("font_size", 13)
	label.add_theme_color_override("font_color", Color(1.0, 0.85, 0.4))
	label.add_theme_color_override("font_outline_color", Color(0.1, 0.05, 0.02))
	label.add_theme_constant_override("outline_size", 4)
	label.position = Vector2(-30, -46)
	label.size = Vector2(60, 16)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(label)
	var tw := create_tween()
	tw.set_parallel(true)
	tw.tween_property(label, "position:y", label.position.y - 28.0, 0.7) \
		.set_ease(Tween.EASE_OUT)
	tw.tween_property(label, "modulate:a", 0.0, 0.7).set_ease(Tween.EASE_IN)


# ---- 视觉：身体 + 头顶名 + 血条（正式素材就位后替换） ----

var _hp_fg: ColorRect


func _make_visual() -> void:
	var body := ColorRect.new()
	body.name = "Body"
	body.color = body_color
	body.size = Vector2(20, 20)
	body.position = -body.size / 2.0
	body.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(body)

	var eye := ColorRect.new()
	eye.name = "Eye"
	eye.color = Color(0.15, 0.1, 0.1)
	eye.size = Vector2(10, 3)
	eye.position = Vector2(-5, -4)
	eye.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(eye)

	var name_label := Label.new()
	name_label.text = display_name
	name_label.add_theme_font_size_override("font_size", 11)
	name_label.add_theme_color_override("font_color", Color(1.0, 0.9, 0.85))
	name_label.add_theme_color_override("font_outline_color", Color(0.1, 0.05, 0.02))
	name_label.add_theme_constant_override("outline_size", 4)
	name_label.position = Vector2(-28, -34)
	name_label.size = Vector2(56, 14)
	name_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	name_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(name_label)

	# 迷你血条：底黑面红，宽度 = 20 * 血量比例
	var hp_bg := ColorRect.new()
	hp_bg.name = "HpBg"
	hp_bg.color = Color(0.1, 0.08, 0.08, 0.85)
	hp_bg.size = Vector2(22, 4)
	hp_bg.position = Vector2(-11, -26)
	hp_bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(hp_bg)
	_hp_fg = ColorRect.new()
	_hp_fg.name = "HpFg"
	_hp_fg.color = Color(0.9, 0.25, 0.2)
	_hp_fg.size = Vector2(20, 2)
	_hp_fg.position = Vector2(-10, -25)
	_hp_fg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_hp_fg)


func _update_hp_bar() -> void:
	if _hp_fg == null or health == null:
		return
	var ratio: float = clampf(health.get_health_ratio(), 0.0, 1.0)
	_hp_fg.size = Vector2(20.0 * ratio, 2)


func _make_shape(half: float) -> CollisionShape2D:
	var shape := CollisionShape2D.new()
	var rect := RectangleShape2D.new()
	rect.size = Vector2(half * 2.0, half * 2.0)
	shape.shape = rect
	return shape
