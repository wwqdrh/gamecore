# role 模块示例：键盘四向玩家 + NPC 行为 + 对话系统集成
#
# 场景布局（相机居中于原点）:
#   - Player         蓝色，WASD/方向键四向移动，speaker 角色"旅人"
#   - Dialogue       共享 GdDialogue（加载 demo_timeline.txt）
#   - DialogBox      对话框 UI（dialog_box.gd，同时充当对话状态存储）
#   - NpcTopRight    黄色"长老"，IDLE，交互键(E)触发对话
#   - NpcBottomLeft  绿色"小孩"，IDLE，靠近触发对话
#   - NpcPatrol      红色"守卫"，横向巡逻，靠近 + 条件触发（需先见过长老）
#   - NpcFollow      紫色"猫"，跟随玩家，3 秒后自动触发自言自语
#
# 对话期间双方自动暂停移动、互相面向，结束后恢复。
# 运行: 打开 example/role/index.tscn 直接 F6 运行
extends Node2D

# 触发模式常量（与 GdDialogTrigger 一致）
const TRIGGER_PROXIMITY := 0
const TRIGGER_INTERACT := 1
const TRIGGER_AUTO := 2

const VIEW_HALF := Vector2(460, 260)  # 各 NPC 相对屏幕中心的基准位置

var player: GdRoleMover
var dialogue: GdDialogue


func _ready() -> void:
	_make_background()
	_make_dialog_system()
	player = _make_player()
	_make_static_npc("NpcTopRight", Vector2(VIEW_HALF.x, -VIEW_HALF.y),
			Color(0.95, 0.78, 0.25), "长老", TRIGGER_INTERACT, "elder_first", "")
	_make_static_npc("NpcBottomLeft", Vector2(-VIEW_HALF.x, VIEW_HALF.y),
			Color(0.45, 0.8, 0.5), "小孩", TRIGGER_PROXIMITY, "kid_chat", "")
	_make_patrol_npc(Vector2(VIEW_HALF.x, VIEW_HALF.y),
			Color(0.9, 0.42, 0.35), "守卫", "guard_talk", "has_flag:met_elder")
	_make_follow_npc(Vector2(-VIEW_HALF.x, 0),
			Color(0.62, 0.48, 0.9), "猫", "cat_mind")
	_make_camera()


# ---------------------------------------------------------------------------
# 对话系统
# ---------------------------------------------------------------------------

## 共享 GdDialogue + DialogBox：所有 NPC 的触发器指向同一个 Dialogue，
## 借助 is_playing 天然互斥，避免两场对话抢占 UI
func _make_dialog_system() -> void:
	dialogue = GdDialogue.new()
	dialogue.name = "Dialogue"
	dialogue.set_timeline_path("res://example/role/demo_timeline.txt")
	add_child(dialogue)

	var box: CanvasLayer = load("res://example/role/dialog_box.gd").new()
	box.name = "DialogBox"
	box.dialogue_path = NodePath("../Dialogue")
	add_child(box)


## 给 NPC 挂对话绑定与触发器
func _add_dialog_parts(npc: GdRoleMover, role_name: String, mode: int,
		entry: String, condition: String) -> void:
	var speaker := GdRoleSpeaker.new()
	speaker.name = "Speaker"
	speaker.role_name = role_name
	npc.add_child(speaker)

	var trigger := GdDialogTrigger.new()
	trigger.name = "Trigger"
	trigger.trigger_mode = mode
	trigger.dialogue_path = NodePath("../../Dialogue")
	trigger.entry_stage = entry
	trigger.condition_fn = condition
	trigger.trigger_radius = 100.0
	npc.add_child(trigger)


# ---------------------------------------------------------------------------
# 玩家
# ---------------------------------------------------------------------------

func _make_player() -> GdRoleMover:
	player = GdRoleMover.new()
	player.name = "Player"
	player.control_mode = 1  # 键盘
	player.move_mode = 0     # 四向
	player.speed = 220.0
	player.position = Vector2.ZERO
	_attach_visual(player, Vector2(36, 36), Color(0.3, 0.6, 1.0))

	var speaker := GdRoleSpeaker.new()
	speaker.name = "Speaker"
	speaker.role_name = "旅人"
	player.add_child(speaker)

	add_child(player)
	_make_label("WASD 移动  E 对话", Vector2(-80, -64), Color(0.7, 0.85, 1.0))
	return player


# ---------------------------------------------------------------------------
# NPC
# ---------------------------------------------------------------------------

## 静止 NPC：IDLE 站桩
func _make_static_npc(npc_name: String, pos: Vector2, color: Color,
		role_name: String, mode: int, entry: String, condition: String) -> void:
	var npc := GdRoleMover.new()
	npc.name = npc_name
	npc.control_mode = 0  # 不响应输入
	npc.position = pos
	_attach_visual(npc, Vector2(32, 32), color)
	add_child(npc)

	var brain := GdNpcBrain.new()
	brain.name = npc_name + "Brain"
	brain.behavior = 0  # IDLE
	npc.add_child(brain)

	_add_dialog_parts(npc, role_name, mode, entry, condition)


## 巡逻 NPC：在出生点两侧 120px 范围内横向来回走动
func _make_patrol_npc(pos: Vector2, color: Color, role_name: String,
		entry: String, condition: String) -> void:
	var npc := GdRoleMover.new()
	npc.name = "NpcPatrol"
	npc.position = pos
	_attach_visual(npc, Vector2(32, 32), color)
	add_child(npc)

	var brain := GdNpcBrain.new()
	brain.name = "NpcPatrolBrain"
	brain.behavior = 2  # PATROL
	brain.patrol_points = PackedVector2Array([
		pos + Vector2(-120, 0),
		pos + Vector2(120, 0),
	])
	brain.patrol_loop = true
	brain.patrol_pause = 0.8
	brain.arrival_distance = 6.0
	npc.add_child(brain)

	_add_dialog_parts(npc, role_name, TRIGGER_PROXIMITY, entry, condition)


## 跟随 NPC：保持安全距离跟随玩家，auto_delay 秒后自动开口
func _make_follow_npc(pos: Vector2, color: Color, role_name: String,
		entry: String) -> void:
	var npc := GdRoleMover.new()
	npc.name = "NpcFollow"
	npc.position = pos
	_attach_visual(npc, Vector2(32, 32), color)
	add_child(npc)

	var brain := GdNpcBrain.new()
	brain.name = "NpcFollowBrain"
	brain.behavior = 3  # FOLLOW
	brain.follow_target = player
	brain.follow_stop_distance = 80.0
	brain.arrival_distance = 6.0
	npc.add_child(brain)

	_add_dialog_parts(npc, role_name, TRIGGER_AUTO, entry, "")
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
