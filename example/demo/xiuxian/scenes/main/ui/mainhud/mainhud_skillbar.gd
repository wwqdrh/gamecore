# 主界面技能栏控制器 —— 由 mainhud_skillbar.gml 的 <ui script="mainhud_skillbar.gd">
# 声明，挂载到技能栏根节点（SkillBar）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   命令上行：点选技能格 -> 读条目 __item_data 取键位 -> 写 KEY_SKILL
#   （技能释放等下游逻辑由游戏侧 watch KEY_SKILL 响应，本控制器不感知）
#   状态下行：灵气经验条 watch 状态层 XiuCharacterState（bean: xiuxian_character）
#   经验变化 → 填充宽度 + 文本（exp/exp_to_next）
extends VBoxContainer
## 与 mainhud.gd 的 KEY_SKILL 一致（demo 各自声明，避免 class_name 缓存依赖）
const KEY_SKILL := "mainhud.skill"

var _char_bean: GdBean


func _ready() -> void:
	_char_bean = XiuCharacterState.ins()
	_char_bean.watch("exp", _on_exp_changed)


func _state():
	return Engine.get_singleton("GDSTATE")


## 状态下行：经验变化 → 经验条填充 + 文本（watch 注册即回调当前值）
func _on_exp_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var track: Control = find_child("XpTrack", true, false)
	var fill: Control = find_child("XpFill", true, false)
	var text: Label = find_child("XpText", true, false)
	if track == null or fill == null:
		return
	var pct: float = _char_bean.get_exp_progress()
	fill.custom_minimum_size.x = track.size.x * pct
	if text:
		text.text = "%d/%d" % [int(_char_bean.exp), int(_char_bean.exp_to_next(_char_bean.level))]


## UIGrid/UIHList 原生点击信号回调（参数 = 条目根节点）
func _on_skill_clicked(item: Control) -> void:
	var data: Dictionary = item.get_meta("__item_data", {})
	_state().set_state(KEY_SKILL, str(data.get("key", "")))
