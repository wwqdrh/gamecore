# 技能格条目控制器 —— 由 skill_item.gml 的 <ui script="skill_item.gd"> 声明，
# 挂载到技能格根节点（SkillSlot），条目被 UIHList duplicate 时脚本随节点复制。
#
# 数据契约（@export = 条目接受的外部注入字段，声明了才注入，类型即文档）：
#   icon 技能图标（emoji 占位，正式版为贴图）
#   key  键位角标（"1"~"6"）
# 运行时状态（skillbar 控制器每帧驱动，非注入契约）：
#   selected     选中高亮（金色描边覆盖层；键位 1-6 与点选两路统一收敛）
#   set_cooldown 冷却灰罩进度条（灰罩盖住格子底部、清晰区自上而下推进，
#                走完恢复全亮=可再施放；冷却唯一真源 = player.gd SKILL_PATTERNS）
extends Panel

@export var icon: String = ""
@export var key: String = ""

## 选中态（唯一真源 = GdState mainhud.skill，控制器写这里）
var selected: bool = false:
	set(value):
		selected = value
		if is_node_ready():
			_apply_selected()

## 冷却运行时状态（剩余秒 / 总时长；skillbar set_cooldown 每帧同步）
var _cd_left := 0.0
var _cd_total := 0.0


func _ready() -> void:
	# 契约注入发生在条目挂树前（构建期），is_node_ready() 为 false 时 setter
	# 跳过 UI 联动——_ready 统一补偿应用（见 ui-gml.md 坑 7）
	_apply_selected()
	_apply_cooldown()


## 冷却同步入口（skillbar 控制器每帧调用）：
##   left > 0 且 total > 0 → 冷却中：灰罩盖住底部 frac 比例（清晰区自上而下推进）
##   left <= 0 / total <= 0 → 就绪：灰罩隐藏（total=0 = 该技能无冷却）
func set_cooldown(left: float, total: float) -> void:
	_cd_left = left
	_cd_total = total
	if is_node_ready():
		_apply_cooldown()


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


## 冷却灰罩：CdMask 锚定 bottom_wide（gml 静态声明），运行期只推 offset_top——
## 灰罩顶边从格子顶部（frac=1）降到格子底部（frac=0），图标自上而下逐渐清晰。
## 每帧按当前格子实际尺寸换算（条目 expand_fill，尺寸随布局变化）
func _apply_cooldown() -> void:
	var mask := find_child("CdMask", true, false) as Panel
	if mask == null:
		return
	var cooling := _cd_left > 0.0 and _cd_total > 0.0
	mask.visible = cooling
	if not cooling:
		return
	var frac: float = clampf(_cd_left / _cd_total, 0.0, 1.0)
	mask.anchor_top = 1.0
	mask.anchor_bottom = 1.0
	mask.offset_top = -size.y * frac
	mask.offset_bottom = 0.0
