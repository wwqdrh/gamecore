# 背包分类列表控制器 —— 由 bag_categories.gml 的 <ui script="bag_categories.gd">
# 声明，挂载到分类面板根节点（CategoryPanel）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   1. 命令上行：点击分类 -> 写 KEY_CATEGORY（不直接操作网格）
#   2. 状态下行：watch KEY_CATEGORY -> 互斥高亮对应按钮
#
# 注意：watch 回调内不得同步再调用 GDSTATE（重入 panic）
extends Panel

## 与 bag_panel.gd 的 KEY_CATEGORY 一致（demo 各自声明，避免 class_name 缓存依赖）
const KEY_CATEGORY := "bag.category"

## 分类按钮名 -> 数据 category 值（空串 = 全部，不过滤）
const NAME_TO_CAT := {
	"CatAll": "",
	"CatTool": "tool",
	"CatPill": "pill",
	"CatMaterial": "material",
	"CatTalisman": "talisman",
}

## category 值 -> 分类按钮名（高亮用，NAME_TO_CAT 的反向表）
const CAT_TO_NAME := {
	"": "CatAll",
	"tool": "CatTool",
	"pill": "CatPill",
	"material": "CatMaterial",
	"talisman": "CatTalisman",
}

const COLOR_NORMAL := Color(1, 1, 1, 1)
const COLOR_SELECTED := Color(1.0, 0.85, 0.42, 1)

var _current: Control


func _ready() -> void:
	# 初始高亮也由状态驱动（面板 _ready 写入初始分类时触发通知）
	_state().watch(KEY_CATEGORY, _on_category_changed)


func _state():
	return Engine.get_singleton("GDSTATE")


## @pressed 回调（参数补绑发出按钮）：只上报，不高亮（高亮由状态下行驱动）
func _on_category(btn: Control) -> void:
	_state().set_state(KEY_CATEGORY, NAME_TO_CAT.get(String(btn.name), ""))


## 状态下行：互斥高亮当前分类按钮
func _on_category_changed(value: Variant) -> void:
	if value == null:
		return  # 尚无初始值，等面板写入
	_select(find_child(CAT_TO_NAME.get(str(value), "CatAll"), true, false))


func _select(btn: Control) -> void:
	if btn == null:
		return
	if _current != null and is_instance_valid(_current):
		_current.modulate = COLOR_NORMAL
	_current = btn
	btn.modulate = COLOR_SELECTED
