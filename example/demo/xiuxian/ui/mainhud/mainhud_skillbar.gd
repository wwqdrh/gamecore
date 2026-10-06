# 主界面技能栏控制器 —— 由 mainhud_skillbar.gml 的 <ui script="mainhud_skillbar.gd">
# 声明，挂载到技能栏根节点（SkillBar）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   命令上行：点选技能格 -> 读条目 __item_data 取键位 -> 写 KEY_SKILL
#   （技能释放等下游逻辑由游戏侧 watch KEY_SKILL 响应，本控制器不感知）
extends VBoxContainer

## 与 mainhud.gd 的 KEY_SKILL 一致（demo 各自声明，避免 class_name 缓存依赖）
const KEY_SKILL := "mainhud.skill"


func _state():
	return Engine.get_singleton("GDSTATE")


## UIGrid/UIHList 原生点击信号回调（参数 = 条目根节点）
func _on_skill_clicked(item: Control) -> void:
	var data: Dictionary = item.get_meta("__item_data", {})
	_state().set_state(KEY_SKILL, str(data.get("key", "")))
