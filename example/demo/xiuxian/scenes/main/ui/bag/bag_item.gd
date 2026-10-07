# 背包格子条目控制器 —— 由 bag_item.gml 的 <ui script="bag_item.gd"> 声明，
# 挂载到格子根节点（BagCell），条目被 UIGrid duplicate 时脚本随节点复制。
#
# 数据契约（@export = 条目接受的外部注入字段，声明了才注入，类型即文档）：
#   icon     物品图标（emoji 占位，正式版为贴图）
#   count    数量角标文本（"x16"；空串隐藏）
#   quality  品质（"稀有" 紫底；空串普通米绿底）
#   locked   锁定格（格子未解锁：置灰 + 锁标）
#   selected 选中态（金色边框）
extends Panel

@export var icon: String = ""
@export var count: String = "":
	set(value):
		count = value
		if is_node_ready():
			var label := find_child("CountLabel", true, false) as Label
			if label:
				label.visible = value != ""
@export var quality: String = "":
	set(value):
		quality = value
		if is_node_ready():
			_apply_quality()
@export var locked: bool = false:
	set(value):
		locked = value
		if is_node_ready():
			_apply_locked()
@export var selected: bool = false:
	set(value):
		selected = value
		if is_node_ready():
			var frame := find_child("SelectedFrame", true, false) as Control
			if frame:
				frame.visible = value


func _ready() -> void:
	# 契约注入发生在条目挂树前（构建期），is_node_ready() 为 false 时 setter
	# 跳过 UI 联动——_ready 统一补偿应用（见 ui-gml.md 坑 7）
	var count_label := find_child("CountLabel", true, false) as Label
	if count_label:
		count_label.visible = count != ""
	_apply_quality()
	_apply_locked()
	var frame := find_child("SelectedFrame", true, false) as Control
	if frame:
		frame.visible = selected


func _apply_quality() -> void:
	var rare := find_child("RareFrame", true, false) as Control
	if rare:
		rare.visible = quality == "稀有"


func _apply_locked() -> void:
	var lock_icon := find_child("LockIcon", true, false) as Control
	if lock_icon:
		lock_icon.visible = locked
	# 锁定格整格置灰
	modulate = Color(1, 1, 1, 1) if not locked else Color(0.62, 0.62, 0.62, 1)
