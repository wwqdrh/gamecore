# 玩家角色 —— 四向网格移动 + 地图传送联动 + 相机跟随 + 战斗（受击/远程射击）
#
# 组合框架能力（零自绘移动/战斗逻辑）：
#   - GdRoleMover MODE_GRID：WASD/方向键逐格四向移动（框架内禁止斜向），
#     grid_map_path 绑定当前地图（duck-type is_walkable 判定通行）
#   - 鼠标左键点击地面：GdQuickMap.find_path BFS（四方向最短路）→ set_grid_path 逐格走
#   - GdMapManager：落格后 try_teleport(世界坐标) 命中传送点即切图；
#     s_map_changed 后重定位出生点、重绑地图、刷新相机边界
#   - GdViewCamera（GdSceneRoot 自动挂载并注册进组件表）：follow 跟随玩家 +
#     update_limit 限制相机边界在地图矩形内 + 地图贴合自适应缩放（不露空白）
#   - 战斗：GdHealth（受击无敌帧）+ GdHurtbox（layer2 玩家受击盒）+
#     GdShooter（发射 GdBullet，mask=4 打敌受击盒 layer3）+
#     GdMelee（挥击判定盒，内嵌 GdHitbox 扫描，mask=4 打敌受击盒 layer3）
#   - 装备联动（GdState 状态总线）：mainhud 装备栏 Hotbar 选中槽位 → 写
#     mainhud.equip（物品 id）；本脚本 watch 同名键——
#     枪支 → 射击模式（左键/右键按住朝鼠标连射）；近战武器 → 近战模式
#     （点击朝鼠标挥击）；未装备武器 → 左键恢复点击寻路
#   - 武器装配（WeaponMount）：装备变化时把对应武器图形节点挂到人物身上
#
# 组件查找：一律经 GdSceneRoot 组件表（get_component），禁止 get_parent
# 链遍历 / find_child 魔法查找——场景根经 GDCORE 全局节点表获取。
#
# 场景装配（scenes/main/index.tscn）：Player 与 MapManager 平级，玩家不在
# MapLayer 内——地图切换只替换 MapLayer 下的地图实例，玩家常驻。
extends GdRoleMover

## 最大血量
@export var max_health := 100.0
## 攻击力（子弹单发伤害）
@export var attack := 25.0

## 状态键：当前装备物品 id（与 mainhud_equipbar.gd 一致，各自声明避免缓存依赖）
const KEY_EQUIP := "mainhud.equip"

var map_mgr: Node = null
var camera: Camera2D = null
var scene_root: Node = null
var health: Node = null
var shooter: Node = null
var melee: Node = null
var weapon_mount: Node2D = null
## 当前攻击模式："gun" = 射击 / "melee" = 近战 / "" = 徒手（左键点击寻路）
var attack_mode := ""
## 兼容字段：持枪状态（射击模式下 true）
var gun_equipped: bool:
	get:
		return attack_mode == "gun"

# 上次落格判定用格子（-9999 = 未初始化，首帧强制判定一次）
var last_cell := Vector2i(-9999, -9999)
var path_line: Line2D
var _wired := false


func _ready() -> void:
	# 加入 "player" 分组：GdDialogTrigger / 敌人 AI 经分组解析玩家节点
	add_to_group("player")
	_make_visual()
	_make_combat()
	# 装备栏联动（状态下行）：watch 注册即回调当前值——HUD 后 ready 也会同步
	if Engine.has_singleton("GDSTATE"):
		Engine.get_singleton("GDSTATE").watch(KEY_EQUIP, _on_equip_changed)
	# 路径预览折线（黄色），走完清除
	path_line = Line2D.new()
	path_line.name = "PathPreview"
	path_line.width = 2.0
	path_line.default_color = Color(1.0, 0.85, 0.25, 0.75)
	path_line.z_index = 1
	add_child(path_line)
	s_grid_path_finished.connect(func() -> void: path_line.clear_points())
	# 接线由 _process 驱动（每帧重试直到完成）。
	# 红线：禁止 call_deferred 自重试——deferred 队列 flush 到空才结束，
	# 自重试会让队列永不为空，主循环卡死在 flush 内（process 永不执行）。


# ---- 战斗装配：Health / Hurtbox / Shooter ----

func _make_combat() -> void:
	health = GdHealth.new()
	health.name = "Health"
	health.max_health = max_health
	health.invincible_time = 0.6
	add_child(health)
	health.s_damaged.connect(_on_damaged)
	# demo 不做死亡结算：倒地立即原地复活（正式版接存档/回城逻辑）
	health.s_died.connect(func() -> void:
		print("[Player] 被击倒，原地复活")
		health.revive(max_health))

	var hurt := GdHurtbox.new()
	hurt.name = "Hurtbox"
	hurt.collision_layer = 2  # 玩家受击盒 layer2（敌人攻击盒 mask 指向这里）
	hurt.collision_mask = 0
	var hurt_shape := CollisionShape2D.new()
	var hurt_rect := RectangleShape2D.new()
	hurt_rect.size = Vector2(18, 18)
	hurt_shape.shape = hurt_rect
	hurt.add_child(hurt_shape)
	add_child(hurt)

	shooter = GdShooter.new()
	shooter.name = "Shooter"
	# 开火输入由框架 unhandled 路由自动处理（点击 UI 不开火、点击世界开火），
	# 持枪与否只切换该开关（装备联动 watch 回调）；右键也开火
	shooter.auto_fire_mouse = false
	shooter.fire_button_right = true
	shooter.fire_cooldown = 0.35
	# 射程 10 格（cell 32px）；子弹超程自动销毁（0 = 不限）
	shooter.max_distance = 320.0
	shooter.bullet_damage = attack
	shooter.bullet_speed = 520.0
	shooter.bullet_lifetime = 1.2
	shooter.muzzle_distance = 18.0
	shooter.bullet_alias = "player_bullet"
	shooter.bullet_scene_path = "res://example/demo/xiuxian/role/player/bullet.tscn"
	add_child(shooter)

	# 近战组件：内嵌 GdHitbox 挥击盒（mask=4 打敌受击盒 layer3）。
	# 默认关闭 auto_attack_mouse（徒手模式），装备近战武器时经装备联动开启
	melee = GdMelee.new()
	melee.name = "Melee"
	melee.auto_attack_mouse = false
	melee.attack_button_left = true
	melee.attack_damage = attack * 1.6  # 近战伤害更高（贴脸风险换收益）
	melee.attack_cooldown = 0.45
	melee.attack_range = 56.0
	melee.attack_width = 48.0
	melee.swing_window = 0.12
	melee.target_mask = 4
	add_child(melee)
	melee.s_swing.connect(func(_dir: Vector2) -> void:
		# 挥击反馈：武器挂点转向挥击方向并回弹（简单图形下的动感占位）
		if weapon_mount != null:
			var tw := create_tween()
			weapon_mount.rotation = _dir.angle()
			tw.tween_property(weapon_mount, "rotation", _dir.angle() * 0.5, 0.18))

	# 武器挂点：装备变化时把对应武器图形装配到这里
	weapon_mount = Node2D.new()
	weapon_mount.name = "WeaponMount"
	weapon_mount.position = Vector2(10, 6)
	add_child(weapon_mount)


func _on_damaged(_amount: float, _current: float) -> void:
	# 受击闪红
	modulate = Color(1.6, 0.7, 0.7)
	var tw := create_tween()
	tw.tween_property(self, "modulate", Color.WHITE, 0.2)
	print("[Player] 受击 -%.0f → %.0f/%.0f"
		% [_amount, health.get_health(), max_health])


## 装备栏选择联动（watch 注册即回调当前值；value=nil = 尚未初始化，跳过）
## 红线：watch 回调内禁止同步调用 GDSTATE 任何方法（重入 panic）
## 攻击模式互斥：gun=射击 / 近战武器=挥击 / 其他=徒手（左键留给点击寻路）
func _on_equip_changed(value: Variant = null, _key: Variant = null) -> void:
	if value == null:
		return
	var item := str(value)
	# 攻击模式归一化：gun=射击 / melee=近战武器 / ""=徒手（左键留给点击寻路）
	if item == "gun":
		attack_mode = "gun"
	elif WEAPON_MELEE.has(item):
		attack_mode = "melee"
	else:
		attack_mode = ""
	# 射击模式 → 框架开火路由开启；近战模式 → 挥击路由开启；
	# 徒手 → 两者全关，左键留给点击寻路（本脚本 _unhandled_input）
	if shooter != null:
		shooter.auto_fire_mouse = attack_mode == "gun"
	if melee != null:
		melee.auto_attack_mouse = attack_mode == "melee"
		if attack_mode != "melee":
			melee.stop()
	_apply_weapon_visual(item)
	print("[Player] 装备: %s → 攻击模式 %s" % [value, attack_mode if attack_mode != "" else "徒手"])


# ---- 武器装配： WeaponMount 挂简单基础图形（正式素材就位后替换） ----

## 近战武器 id 集（与 item.json 装备类道具对应）
const WEAPON_MELEE := ["sword_qingfeng"]


## 装配武器图形：清空挂点 → 按物品 id 挂对应节点（徒手=空）
func _apply_weapon_visual(item: String) -> void:
	if weapon_mount == null:
		return
	for child in weapon_mount.get_children():
		child.queue_free()
	match item:
		"gun":
			weapon_mount.add_child(_make_gun_visual())
		_:
			if WEAPON_MELEE.has(item):
				weapon_mount.add_child(_make_sword_visual())


## 枪：灰色矩形枪身 + 深色枪口（横向持握，朝向由挥击/开火反馈旋转）
func _make_gun_visual() -> Node2D:
	var gun := Node2D.new()
	gun.name = "WeaponGun"
	var body := ColorRect.new()
	body.name = "GunBody"
	body.color = Color(0.45, 0.48, 0.55)
	body.size = Vector2(18, 6)
	body.position = Vector2(0, -3)
	body.mouse_filter = Control.MOUSE_FILTER_IGNORE
	gun.add_child(body)
	var muzzle := ColorRect.new()
	muzzle.name = "GunMuzzle"
	muzzle.color = Color(0.2, 0.22, 0.26)
	muzzle.size = Vector2(5, 4)
	muzzle.position = Vector2(18, -2)
	muzzle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	gun.add_child(muzzle)
	return gun


## 剑：多边形剑刃 + 十字护手 + 握柄（斜上持握）
func _make_sword_visual() -> Node2D:
	var sword := Node2D.new()
	sword.name = "WeaponSword"
	sword.rotation = -0.6  # 斜上持握
	var blade := Polygon2D.new()
	blade.name = "Blade"
	blade.color = Color(0.82, 0.88, 0.95)
	blade.polygon = PackedVector2Array([
		Vector2(0, -2), Vector2(26, -1), Vector2(30, 0),
		Vector2(26, 1), Vector2(0, 2),
	])
	sword.add_child(blade)
	var guard := ColorRect.new()
	guard.name = "Guard"
	guard.color = Color(0.72, 0.55, 0.2)
	guard.size = Vector2(3, 10)
	guard.position = Vector2(-1, -5)
	guard.mouse_filter = Control.MOUSE_FILTER_IGNORE
	sword.add_child(guard)
	var hilt := ColorRect.new()
	hilt.name = "Hilt"
	hilt.color = Color(0.4, 0.28, 0.15)
	hilt.size = Vector2(8, 3)
	hilt.position = Vector2(-9, -1.5)
	hilt.mouse_filter = Control.MOUSE_FILTER_IGNORE
	sword.add_child(hilt)
	return sword


# ---- 接线 ----

func _wire() -> void:
	if _wired:
		return
	scene_root = _find_scene_root()
	if scene_root == null:
		return
	# 组件表查询：MapManager 由管理器自动注册，ViewCamera 由 GdSceneRoot 自动注册
	map_mgr = scene_root.get_component("MapManager")
	if map_mgr == null or not map_mgr.has_method("get_current_map"):
		return
	if map_mgr.get_current_map() == null:
		# 初始地图尚未加载完（同帧级联顺序问题），下帧再试
		return
	_wired = true
	map_mgr.connect("s_map_changed", _on_map_changed)
	camera = scene_root.get_component("ViewCamera")
	_apply_map(map_mgr.get_current_map(), true)


## 场景根（GdSceneRoot）：经 GDCORE 全局节点表获取（manager_id 默认 "default"）
func _find_scene_root() -> Node:
	if not Engine.has_singleton("GDCORE"):
		return null
	var gdcore: Object = Engine.get_singleton("GDCORE")
	var node = gdcore.get_global_node("default")
	return node if node is Node else null


# ---- 地图应用：网格绑定 + 相机跟随/边界 + 出生点 ----

func _apply_map(map: Node, snap: bool) -> void:
	if map == null:
		return
	# 清掉旧地图遗留的路径指令（下一格在新图上无意义）
	stop()
	path_line.clear_points()
	# 网格绑定：指向当前地图实例（换图后 grid_map 缓存失效自动重解析）
	var alias: String = map_mgr.get_current_alias()
	grid_map_path = NodePath("../MapManager/MapLayer/" + alias)
	# 相机：跟随玩家 + 边界限制在地图矩形内
	_apply_camera(map, snap)
	# 出生点（MapManager 已把不可行走出生格吸附到可行走格）
	if snap:
		global_position = map_mgr.get_spawn_point()
	last_cell = Vector2i(-9999, -9999)


func _apply_camera(map: Node, snap: bool) -> void:
	if camera == null and scene_root != null:
		# 相机可能晚于玩家接线创建（默认 GdSceneRoot 延迟添加），组件表重查
		camera = scene_root.get_component("ViewCamera")
	if camera == null or not camera.has_method("follow"):
		return  # 相机缺失，_process 中重试
	# 边界限制在地图矩形内；缩放交给相机"地图贴合"自适应——
	# update_limit 内部按 视口/地图 算出恰好铺满屏幕的 zoom（不露空白），
	# 窗口缩放/全屏切换（viewport size_changed）自动重算，无需硬编码档位
	camera.follow(self, snap, true)
	var w: int = map.get_map_width()
	var h: int = map.get_map_height()
	var cs: int = map.get_cell_size_px()
	camera.update_limit(Vector4(0, w * cs, 0, h * cs))


func _on_map_changed(_alias: String) -> void:
	_apply_map(map_mgr.get_current_map(), true)


# ---- 传送判定：落格（含点击寻路途经）后查询地图管理器 ----

func _process(_delta: float) -> void:
	# 注意：不覆写 _physics_process（会遮蔽 GdRoleMover 的 Rust 侧网格驱动）
	# 接线重试在 process 阶段驱动（MapManager 组件注册也在其 process 中完成，
	# 同为 process 阶段即可收敛，且不会卡死 deferred flush）
	# 远程攻击：开火输入由 GdShooter 自身的 unhandled_input 事件路由驱动
	# （auto_fire_mouse=true 时框架自动处理：点击 UI 不开火、点击世界开火，
	# 见 shooter.rs；此处严禁再用 Input 轮询 + gui_get_hovered_control 猜测）
	if not _wired:
		_wire()
		if not _wired:
			return
	if map_mgr == null:
		return
	# 相机缺失重试：默认 GdSceneRoot 延迟添加，找到后补一次跟随/边界绑定
	if camera == null:
		_apply_camera(map_mgr.get_current_map(), true)
	if map_mgr.get_current_map() == null:
		return
	if is_moving():
		return
	var map: Node = map_mgr.get_current_map()
	if map == null or not map.has_method("world_to_cell"):
		return
	var cell: Vector2i = map.world_to_cell(global_position)
	if cell != last_cell:
		last_cell = cell
		# 命中传送点：管理器切图并经 s_map_changed 回调重定位玩家
		map_mgr.try_teleport(global_position)


# ---- 点击寻路 ----

func _unhandled_input(event: InputEvent) -> void:
	# 本回调只在「点击未被 UI 消费」时收到（事件路由，引擎保证）。
	# 未装备武器：左键点击地面 → BFS 寻路；射击/近战模式时左键已被
	# GdShooter / GdMelee 的 unhandled 路由消费（set_input_as_handled），
	# 此处按 attack_mode 分流双保险。
	if event is InputEventMouseButton and event.pressed \
			and event.button_index == MOUSE_BUTTON_LEFT:
		# 装备武器时左键 = 攻击输入（框架 unhandled 路由驱动），不再触发寻路
		if attack_mode != "":
			return
		_on_map_clicked()


func _on_map_clicked() -> void:
	if not _wired or map_mgr == null:
		return
	var map: Node = map_mgr.get_current_map()
	if map == null or not map.has_method("find_path"):
		return
	var target: Vector2i = map.world_to_cell(map.get_global_mouse_position())
	if not map.is_walkable(target):
		return
	var from: Vector2i = map.world_to_cell(global_position)
	if from == target:
		return
	var path: PackedVector2Array = map.find_path(from, target)
	if path.is_empty():
		return
	set_grid_path(path)
	# 预览线：当前位置 → 依次各格中心
	var pts := PackedVector2Array([global_position])
	pts.append_array(path)
	path_line.points = pts


# ---- 视觉：身体 + 朝向指示（正式素材就位后替换） ----

func _make_visual() -> void:
	var body := ColorRect.new()
	body.name = "Body"
	body.color = Color(0.3, 0.6, 1.0)
	body.size = Vector2(22, 22)
	body.position = -body.size / 2.0
	body.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(body)
	var eye := ColorRect.new()
	eye.name = "Eye"
	eye.color = Color(0.95, 0.98, 1.0)
	eye.size = Vector2(6, 6)
	eye.position = Vector2(2, -2)
	eye.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(eye)
	s_facing_changed.connect(_on_facing_changed)


func _on_facing_changed(facing: String) -> void:
	var eye: ColorRect = get_node_or_null("Eye")
	if eye == null:
		return
	match facing:
		"left":
			eye.position = Vector2(-8, -2)
		"right":
			eye.position = Vector2(2, -2)
		"up":
			eye.position = Vector2(-2, -8)
		_:
			eye.position = Vector2(-2, 2)
