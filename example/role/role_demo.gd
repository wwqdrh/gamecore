# role 模块示例：键盘四向玩家 + 三种 NPC 行为
#
# 场景布局（相机居中于原点）:
#   - Player        蓝色，WASD/方向键四向移动（GdRoleMover 键盘控制，零配置）
#   - NpcTopRight   黄色，右上角静止（GdNpcBrain IDLE）
#   - NpcBottomLeft 绿色，左下角静止（GdNpcBrain IDLE）
#   - NpcPatrol     红色，右下角在固定范围内横向来回巡逻（GdNpcBrain PATROL）
#   - NpcFollow     紫色，跟随玩家但保持安全距离不重叠（GdNpcBrain FOLLOW）
#
# 运行: 打开 example/role/index.tscn 直接 F6 运行
extends Node2D

# 行为常量（与 GdNpcBrain 中定义一致）
const AI_IDLE := 0
const AI_PATROL := 2
const AI_FOLLOW := 3

# 控制方式常量（与 GdRoleMover 中定义一致）
const CONTROL_NONE := 0
const CONTROL_KEYBOARD := 1
const CONTROL_AI := 3

const VIEW_HALF := Vector2(460, 260)  # 各 NPC 相对屏幕中心的基准位置

var player: GdRoleMover


func _ready() -> void:
	_make_background()
	player = _make_player()
	_make_static_npc("NpcTopRight", Vector2(VIEW_HALF.x, -VIEW_HALF.y), Color(0.95, 0.78, 0.25))
	_make_static_npc("NpcBottomLeft", Vector2(-VIEW_HALF.x, VIEW_HALF.y), Color(0.45, 0.8, 0.5))
	_make_patrol_npc(Vector2(VIEW_HALF.x, VIEW_HALF.y), Color(0.9, 0.42, 0.35))
	_make_follow_npc(Vector2(-VIEW_HALF.x, 0), Color(0.62, 0.48, 0.9))
	_make_camera()
	_claim_ownership()


# ---------------------------------------------------------------------------
# 玩家
# ---------------------------------------------------------------------------

func _make_player() -> GdRoleMover:
	player = GdRoleMover.new()

	player.name = "Player"
	player.control_mode = CONTROL_KEYBOARD
	player.move_mode = 0  # 四向
	player.speed = 220.0
	player.position = Vector2.ZERO
	_attach_visual(player, Vector2(36, 36), Color(0.3, 0.6, 1.0))
	add_child(player)

	# 键盘提示
	_make_label("Player  WASD/方向键移动", Vector2(-110, -60), Color(0.7, 0.85, 1.0))
	return player


# ---------------------------------------------------------------------------
# NPC
# ---------------------------------------------------------------------------

## 静止 NPC：IDLE 行为，只站桩
func _make_static_npc(npc_name: String, pos: Vector2, color: Color) -> void:
	var npc := GdRoleMover.new()
	npc.name = npc_name
	npc.control_mode = CONTROL_NONE  # 不响应任何输入
	npc.position = pos
	_attach_visual(npc, Vector2(32, 32), color)
	add_child(npc)

	var brain := GdNpcBrain.new()
	brain.name = npc_name + "Brain"
	brain.behavior = AI_IDLE
	npc.add_child(brain)


## 巡逻 NPC：在出生点两侧 120px 范围内横向来回走动
func _make_patrol_npc(pos: Vector2, color: Color) -> void:
	var npc := GdRoleMover.new()
	npc.name = "NpcPatrol"
	npc.position = pos
	_attach_visual(npc, Vector2(32, 32), color)
	add_child(npc)

	var brain := GdNpcBrain.new()
	brain.name = "NpcPatrolBrain"
	brain.behavior = AI_PATROL
	brain.patrol_points = PackedVector2Array([
		pos + Vector2(-120, 0),
		pos + Vector2(120, 0),
	])
	brain.patrol_loop = true
	brain.patrol_pause = 0.8  # 在端点稍作停留，更像人
	brain.arrival_distance = 6.0
	npc.add_child(brain)


## 跟随 NPC：保持 follow_stop_distance 的安全距离，不与玩家重叠
func _make_follow_npc(pos: Vector2, color: Color) -> void:
	var npc := GdRoleMover.new()
	npc.name = "NpcFollow"
	npc.position = pos
	_attach_visual(npc, Vector2(32, 32), color)
	add_child(npc)

	var brain := GdNpcBrain.new()
	brain.name = "NpcFollowBrain"
	brain.behavior = AI_FOLLOW
	brain.follow_target = player
	brain.follow_stop_distance = 80.0  # 安全距离，避免重叠
	brain.arrival_distance = 6.0
	npc.add_child(brain)

	# 玩家移动后跟随目标引用不变，无需额外处理
	_make_label("Follow", Vector2(-24, -52), color)


# ---------------------------------------------------------------------------
# 视觉辅助
# ---------------------------------------------------------------------------

## 给角色挂一个纯色方块占位贴图
func _attach_visual(role: Node2D, size: Vector2, color: Color) -> void:
	var rect := ColorRect.new()
	rect.color = color
	rect.size = size
	rect.position = -size / 2.0
	rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	role.add_child(rect)


func _make_label(text: String, pos: Vector2, color: Color) -> void:
	var label := Label.new()
	label.text = text
	label.position = pos
	label.modulate = color
	add_child(label)


func _make_background() -> void:
	var bg := ColorRect.new()
	bg.name = "Background"
	bg.color = Color(0.12, 0.13, 0.16)
	bg.size = Vector2(1600, 1000)
	bg.position = -bg.size / 2.0
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(bg)
	move_child(bg, 0)  # 背景垫底


func _make_camera() -> void:
	var cam := Camera2D.new()
	cam.name = "Camera"
	cam.position = Vector2.ZERO
	add_child(cam)
	cam.make_current()

## pack() 只保存 owner 为根节点的节点 —— 运行时创建的节点必须补设 owner
## SaveButton 是工具按钮，跳过它，使其不被写入文件
func _claim_ownership() -> void:
	for child in get_children():
		if child.name == "SaveButton":
			continue
		_set_owner_recursive(child)


func _set_owner_recursive(node: Node) -> void:
	node.owner = self
	for child in node.get_children():
		_set_owner_recursive(child)


func _show_toast(text: String) -> void:
	var label := Label.new()
	label.text = text
	label.position = Vector2(170, 22)
	label.modulate = Color(0.7, 1.0, 0.7)
	add_child(label)
	get_tree().create_timer(2.0).timeout.connect(label.queue_free)
