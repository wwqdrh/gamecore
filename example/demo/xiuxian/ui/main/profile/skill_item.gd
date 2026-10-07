# 功法条目控制器 —— 由 skill_item.gml 的 <ui script="skill_item.gd"> 自动挂载
# 到条目根节点（SkillItemRoot），随列表 duplicate 复制到每个条目。
#
# 数据契约：@export 变量 = 条目接受的外部注入字段（列表 update 时同名 key 自动
# 写入脚本实例），编辑 skill_item.gml 时对照本列表即知会传入哪些数据。
# locked 为带 setter 的契约字段：注入时同步置灰条目并禁用升级按钮。
extends Panel

@export var icon: String = ""
@export var title: String = ""
@export var level: String = ""
@export var btn_text: String = ""
@export var locked: bool = false:
	set(value):
		locked = value
		if is_node_ready():
			_apply_locked()

func _ready() -> void:
	_apply_locked()

## 未习得（锁定）状态：整条目半透明 + 按钮禁用
func _apply_locked() -> void:
	modulate = Color(1, 1, 1, 0.65) if locked else Color.WHITE
	var btn := find_child("SkillBtn", true, false) as Button
	if btn:
		btn.disabled = locked

## 升级按钮回调（@pressed 声明在 skill_item.gml，参数自动补绑发出按钮）
func _on_skill_action(_btn: Control) -> void:
	print("[SkillItem] %s %s" % [title, btn_text])
