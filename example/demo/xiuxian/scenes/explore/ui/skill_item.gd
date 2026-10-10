# 技能格条目控制器 —— 由 skill_item.gml 的 <ui script="skill_item.gd"> 声明，
# 挂载到技能格根节点（SkillSlot），条目被 UIHList duplicate 时脚本随节点复制。
#
# 数据契约（@export = 条目接受的外部注入字段，声明了才注入，类型即文档）：
#   icon 技能图标（emoji 占位，正式版为贴图）
#   key  键位角标（"1"~"6"）
#   cd   冷却文本（"8s"；非空显示冷却遮罩+倒计时，空串 = 技能就绪）
extends Panel

@export var icon: String = ""
@export var key: String = ""
@export var cd: String = "":
	set(value):
		cd = value
		if is_node_ready():
			_apply_cd()


func _ready() -> void:
	# 契约注入发生在条目挂树前（构建期），is_node_ready() 为 false 时 setter
	# 跳过 UI 联动——_ready 统一补偿应用（见 ui-gml.md 坑 7）
	_apply_cd()


func _apply_cd() -> void:
	var cooling := cd != ""
	var mask := find_child("CdMask", true, false) as Control
	if mask:
		mask.visible = cooling
	var text := find_child("CdText", true, false) as Label
	if text:
		text.visible = cooling
