# 秘境技能栏控制器 —— 由 explore_skillbar.gml 的 <ui script="explore_skillbar.gd">
# 声明，挂载到技能栏根节点（SkillBar）。自 mainhud_skillbar 迁移（仅探索场景显示）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   命令上行：点选技能格 -> 读条目 __item_data 取键位 -> 写 KEY_SKILL
#   （再点已选中格 = 取消选中；按键 1-6 与点选同走状态键，见 player.gd）
#   状态下行：watch KEY_SKILL 同步选中高亮（选中态唯一真源 = 状态键）
#   状态下行：灵气经验条 watch 状态层 XiuCharacterState（bean: xiuxian_character）
#   经验变化 → 填充宽度 + 文本（exp/exp_to_next）
extends VBoxContainer
## 键名沿用 mainhud.skill（玩家侧消费方 watch 同名键，跨场景兼容）
const KEY_SKILL := "mainhud.skill"

var _char_bean: GdBean
## 当前选中键位本地镜像（watch 回调维护；点选切换用它判取/舍）
var _sel := ""


func _ready() -> void:
	_char_bean = XiuCharacterState.ins()
	_char_bean.watch("exp", _on_exp_changed)
	# watch 注册即回调当前值：进场景时恢复既有选中态（重开/回切场景不丢高亮）
	_state().watch(KEY_SKILL, _on_skill_state)


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


## 状态下行：选中键位变化 → 各条目高亮（键位 1-6 与点选两路统一收敛）
func _on_skill_state(value: Variant = null, _key: Variant = null) -> void:
	_sel = str(value) if value != null else ""
	var list: Node = find_child("SkillList", true, false)
	if list == null:
		return
	# duplicate 条目重名带 @ 后缀，按子节点序遍历（勿按名找）
	for item in list.get_children():
		if "selected" in item:
			item.set("selected", str(item.get("key")) == _sel)


## UIGrid/UIHList 原生点击信号回调（参数 = 条目根节点）：
## 再点已选中格 = 取消选中（toggle）；状态键 update 同值自动短路去重
func _on_skill_clicked(item: Control) -> void:
	var data: Dictionary = item.get_meta("__item_data", {})
	var key := str(data.get("key", ""))
	_state().set_state(KEY_SKILL, "" if _sel == key else key)
