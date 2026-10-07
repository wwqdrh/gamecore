# 主界面功能按钮列控制器 —— 由 mainhud_menus.gml 的 <ui script="mainhud_menus.gd">
# 声明，挂载到功能按钮列根节点（MenuColumn）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   1. 命令上行：点击功能按钮 -> 写 KEY_MENU（不直接操作其他区块）
#   2. 状态下行：watch KEY_MENU -> 金色高亮当前功能按钮（含初始高亮）
#
# 注意：watch 回调内不得同步再调用 GDSTATE（重入 panic）
extends VBoxContainer

## 与 mainhud.gd 的 KEY_MENU 一致（demo 各自声明，避免 class_name 缓存依赖）
const KEY_MENU := "mainhud.menu"

## 按钮名 -> 功能界面 id
const NAME_TO_ID := {
	"MenuCultivate": "cultivate",
	"MenuGongfa": "gongfa",
	"MenuBag": "bag",
	"MenuMarket": "market",
}

## 功能界面 id -> 按钮名（高亮用，NAME_TO_ID 的反向表）
const ID_TO_NAME := {
	"cultivate": "MenuCultivate",
	"gongfa": "MenuGongfa",
	"bag": "MenuBag",
	"market": "MenuMarket",
}

## 文本子节点名后缀（高亮切换用：按钮名去 Menu 前缀 + Text）
const COLOR_NORMAL := Color(1, 1, 1, 1)
const COLOR_ACTIVE := Color(1.0, 0.85, 0.42, 1)

var _current: Control


func _ready() -> void:
	# 初始高亮也由状态驱动（mainhud.gd _ready 写入初始功能页时触发通知）
	_state().watch(KEY_MENU, _on_menu_changed)


func _state():
	return Engine.get_singleton("GDSTATE")


## @pressed 回调（参数补绑发出按钮）：只上报，不高亮（高亮由状态下行驱动）
func _on_menu_pressed(btn: Control) -> void:
	_state().set_state(KEY_MENU, NAME_TO_ID.get(String(btn.name), ""))


## 状态下行：互斥高亮当前功能按钮
func _on_menu_changed(value: Variant) -> void:
	if value == null:
		return  # 尚无初始值，等面板写入
	_select(find_child(ID_TO_NAME.get(str(value), "MenuCultivate"), true, false))


func _select(btn: Control) -> void:
	if btn == null:
		return
	if _current == btn:
		return
	if _current != null and is_instance_valid(_current):
		_current.modulate = COLOR_NORMAL
	_current = btn
	btn.modulate = COLOR_ACTIVE
