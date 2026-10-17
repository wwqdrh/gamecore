# 技能格条目控制器 —— 由 skill_item.gml 的 <ui script="skill_item.gd"> 声明，
# 挂载到技能格根节点（SkillSlot），条目被 UIHList duplicate 时脚本随节点复制。
#
# 数据契约（@export = 条目接受的外部注入字段，声明了才注入，类型即文档）：
#   icon 技能图标（emoji 占位，正式版为贴图）
#   key  键位角标（"1"~"6"）
#   cd   冷却文本（"8s"；非空显示冷却遮罩+倒计时，空串 = 技能就绪）
# 运行时状态（skillbar 控制器按状态键 mainhud.skill 同步，非注入契约）：
#   selected 选中高亮（金色描边覆盖层；键位 1-6 与点选两路统一收敛）
extends Panel

@export var icon: String = ""
@export var key: String = ""
@export var cd: String = "":
	set(value):
		cd = value
		if is_node_ready():
			_apply_cd()

## 选中态（唯一真源 = GdState mainhud.skill，控制器写这里）
var selected: bool = false:
	set(value):
		selected = value
		if is_node_ready():
			_apply_selected()


func _ready() -> void:
	# 契约注入发生在条目挂树前（构建期），is_node_ready() 为 false 时 setter
	# 跳过 UI 联动——_ready 统一补偿应用（见 ui-gml.md 坑 7）
	_apply_cd()
	_apply_selected()


func _apply_cd() -> void:
	var cooling := cd != ""
	var mask := find_child("CdMask", true, false) as Control
	if mask:
		mask.visible = cooling
	var text := find_child("CdText", true, false) as Label
	if text:
		text.visible = cooling


## 选中高亮覆盖层（懒创建：金色描边圆角框，置顶绘制、鼠标穿透）
func _apply_selected() -> void:
	var frame := find_child("SelFrame", true, false) as Panel
	if frame == null:
		frame = Panel.new()
		frame.name = "SelFrame"
		var sb := StyleBoxFlat.new()
		sb.bg_color = Color(0, 0, 0, 0)
		sb.border_color = Color(1.0, 0.82, 0.3)
		sb.set_border_width_all(3)
		sb.set_corner_radius_all(10)
		frame.add_theme_stylebox_override("panel", sb)
		frame.mouse_filter = Control.MOUSE_FILTER_IGNORE
		frame.set_anchors_preset(Control.PRESET_FULL_RECT)
		add_child(frame)
	frame.visible = selected
