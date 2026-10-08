# NPC 角色基类（XiuNpcBase）—— 对话三件套 + AI 行为装配
#
# 具体角色在各目录（elder/merchant/disciple/...）继承本类，用 _init 设置
# 默认导出值（名字/站位/AI 行为/timeline），并各自持有自己的 timeline 文件。
#
# 组合框架能力（零自写对话/AI 逻辑）：
#   - GdRoleMover：宿主（网格移动；对话期间被触发器 set_paused 暂停
#     并自动与玩家互相面向）
#   - GdRoleSpeaker：把本节点绑定到 timeline 角色名（触发器启动对话时自动注册）
#   - GdDialogTrigger：TRIGGER_INTERACT —— 玩家进入 trigger_radius 后按 E 触发
#     （interact_action 未注册 InputMap 动作时框架自动回退物理 E 键）；
#     timeline_path 指向本角色目录下的专属 timeline（触发时加载进共享 Dialogue）
#   - GdNpcBrain：AI 行为（ai_behavior：-1 不挂 / 0 IDLE 站桩 / 1 WANDER 游走 /
#     2 PATROL 巡逻）——网格地图下经 find_path BFS 逐格走，不穿不可行走地形
#   - 落位：父级 GdQuickMap 上 npc_cell 落在不可行走地形（水/山）时，
#     BFS 吸附到最近可行走格（噪声地图种子固定但格子随机，手填坐标不可靠）
#
# 对话节点解析：共享 GdDialogue 由主场景 DialogLayer 创建并加入
# "__gd_dialogue" 分组；本组件 _process 内重试接线（红线：禁止 call_deferred
# 自重试——deferred 队列 flush 到空才结束，自重试会卡死主循环）。
#
# 场景装配：NPC 场景作为地图场景（town.tscn 等）的子节点实例化——随地图
# 一起加载/释放；对话内容在本角色目录的 timeline 文件里维护。
class_name XiuNpcBase
extends GdRoleMover

const DIALOGUE_GROUP := "__gd_dialogue"
const TRIGGER_INTERACT := 1
const AI_NO_BRAIN := -1

## 头顶显示名
@export var display_name := "路人"
## 对话角色名（须与 timeline 中角色名一致）
@export var role_name := "路人"
## 本角色专属 timeline 文件路径（触发对话时加载进共享 GdDialogue）
@export var timeline_path := ""
## 对话入口舞台名（空 = timeline 开头）
@export var entry_stage := ""
## 触发条件门控："方法名[:参数]"（在触发器/宿主/对话 control 上查找）
@export var condition_fn := ""
## 站位格子（-1 = 用实例化时的坐标换算）
@export var npc_cell := Vector2i(-1, -1)
## AI 行为：-1 不挂 Brain / 0 IDLE 站桩 / 1 WANDER 游走 / 2 PATROL 巡逻
@export var ai_behavior := AI_NO_BRAIN
## 游走半径（像素，相对初始位置）
@export var wander_radius := 96.0
## 巡逻格列表（AI_PATROL 用；运行期逐格吸附到可行走地形后转世界坐标）
@export var patrol_cells: Array[Vector2i] = []
## 身体颜色（占位视觉，正式素材就位后替换）
@export var body_color := Color(0.9, 0.75, 0.35)

var trigger: Node = null
var brain: Node = null

var _snapped := false
var _wired := false
var _ai_applied := false
var _hint: Label


func _ready() -> void:
	control_mode = 0  # CONTROL_NONE：网格移动走 grid_path 队列，AI Brain 直接驱动
	_make_visual()
	_make_dialog_parts()
	_make_ai()
	# 接线由 _process 驱动（父地图地形数据与共享对话节点可能晚于本 ready）


func _process(_delta: float) -> void:
	if not _snapped:
		_snap_to_map()
	if not _wired:
		_wire_dialogue()
	if _snapped and _wired and _ai_applied:
		set_process(false)


# ---- 落位：吸附到父地图最近可行走格 ----

func _snap_to_map() -> void:
	var map := get_parent()
	if map == null or not map.has_method("is_walkable") \
			or not map.has_method("get_cell_size_px"):
		return
	var cs: int = map.get_cell_size_px()
	var cell := npc_cell
	if cell.x < 0:
		cell = map.world_to_cell(global_position)
	var target := _nearest_walkable(map, cell)
	global_position = Vector2((target.x + 0.5) * cs, (target.y + 0.5) * cs)
	if target != cell:
		push_warning("[Npc] %s 站位格 %s 不可行走，吸附到 %s" % [display_name, cell, target])
	_snapped = true
	# AI 基准点/巡逻点以吸附后的真实位置为准（Brain ready 时取到的是吸附前坐标）
	_apply_ai_points(map, cs)


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
	return cell  # 全图不可走（异常）：原样返回


# ---- AI 行为：Brain 装配 + 巡逻点吸附 ----

func _make_ai() -> void:
	if ai_behavior < 0:
		_ai_applied = true
		return
	brain = GdNpcBrain.new()
	brain.name = "Brain"
	brain.behavior = ai_behavior
	brain.wander_radius = wander_radius
	add_child(brain)


func _apply_ai_points(map: Node, cs: int) -> void:
	if brain == null:
		_ai_applied = true
		return
	if ai_behavior == 2 and not patrol_cells.is_empty():
		# 巡逻格逐个吸附到可行走地形，转格心世界坐标交给 Brain
		var points := PackedVector2Array()
		for cell in patrol_cells:
			var fixed: Vector2i = _nearest_walkable(map, cell)
			points.append(Vector2((fixed.x + 0.5) * cs, (fixed.y + 0.5) * cs))
		brain.set_patrol_points(points)
	brain.restart()  # 以吸附后位置为新家，巡逻从头开始
	_ai_applied = true


# ---- 对话节点接线：分组查找共享 GdDialogue ----

func _wire_dialogue() -> void:
	if trigger == null:
		return
	var dia: Node = get_tree().get_first_node_in_group(DIALOGUE_GROUP)
	if dia == null:
		return  # DialogLayer 尚未就绪，下帧重试
	trigger.dialogue_path = trigger.get_path_to(dia)
	_wired = true


# ---- 对话三件套装配 ----

func _make_dialog_parts() -> void:
	var speaker := GdRoleSpeaker.new()
	speaker.name = "Speaker"
	speaker.role_name = role_name
	add_child(speaker)

	trigger = GdDialogTrigger.new()
	trigger.name = "Trigger"
	trigger.trigger_mode = TRIGGER_INTERACT
	trigger.trigger_radius = 96.0  # 约三格
	trigger.pause_roles = true     # 对话期间暂停双方移动、互相面向
	trigger.entry_stage = entry_stage
	trigger.condition_fn = condition_fn
	if not timeline_path.is_empty():
		trigger.timeline_path = timeline_path
	add_child(trigger)
	trigger.s_trigger_enter.connect(func() -> void: _set_hint_visible(true))
	trigger.s_trigger_exit.connect(func() -> void: _set_hint_visible(false))


func _set_hint_visible(v: bool) -> void:
	if _hint != null:
		_hint.visible = v


# ---- 视觉：身体 + 头顶名字 + "E 对话"提示（正式素材就位后替换） ----

func _make_visual() -> void:
	var body := ColorRect.new()
	body.name = "Body"
	body.color = body_color
	body.size = Vector2(22, 22)
	body.position = -body.size / 2.0
	body.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(body)

	var name_label := Label.new()
	name_label.text = display_name
	name_label.add_theme_font_size_override("font_size", 12)
	name_label.add_theme_color_override("font_color", Color(1.0, 0.95, 0.8))
	name_label.add_theme_color_override("font_outline_color", Color(0.1, 0.08, 0.04))
	name_label.add_theme_constant_override("outline_size", 4)
	name_label.position = Vector2(-28, -34)
	name_label.size = Vector2(56, 16)
	name_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	name_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(name_label)

	_hint = Label.new()
	_hint.text = "E 对话"
	_hint.visible = false
	_hint.add_theme_font_size_override("font_size", 11)
	_hint.add_theme_color_override("font_color", Color(0.55, 0.9, 1.0))
	_hint.add_theme_color_override("font_outline_color", Color(0.05, 0.1, 0.15))
	_hint.add_theme_constant_override("outline_size", 4)
	_hint.position = Vector2(-28, -48)
	_hint.size = Vector2(56, 14)
	_hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_hint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_hint)
