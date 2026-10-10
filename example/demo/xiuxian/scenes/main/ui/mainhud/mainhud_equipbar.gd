# 主界面装备栏控制器 —— 由 mainhud_equipbar.gml 的 <ui script="mainhud_equipbar.gd">
# 声明，挂载到装备栏根节点（EquipBar）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   命令上行：Hotbar 选中（数字键 1~4 / 点击）→ 读槽位物品表 → 写 KEY_EQUIP
#   （mainhud.equip，值为物品 id 字符串；空串 = 空槽 / 卸下）
#   消费方 player.gd watch 同名键：枪支 → 射击模式；近战武器 → 挥击模式
#   （状态下行，本控制器不感知）
#
# 槽位配置：1 号 = 枪支（demo 初始 selected_index=0，进场即持枪可射击），
#           2 号 = 近战武器（青锋剑，item.json 装备类），3 号 = 丹药
extends HBoxContainer

## 与 player.gd 的 KEY_EQUIP 一致（demo 各自声明，避免 class_name 缓存依赖）
const KEY_EQUIP := "mainhud.equip"

## 槽位 → 物品 id（"" = 空槽 / 卸下）。1 号 = 枪支，2 号 = 近战武器
const SLOT_ITEMS := ["gun", "sword_qingfeng", "pill_hp", ""]

## 槽位图标占位（emoji；正式素材就位后换 TextureRect）
const SLOT_GLYPHS := ["🔫", "🗡️", "💊", ""]


func _ready() -> void:
	var hotbar: Control = find_child("EquipSlots", true, false)
	if hotbar == null:
		return
	for i in SLOT_GLYPHS.size():
		if str(SLOT_GLYPHS[i]) != "":
			hotbar.call("set_slot_glyph", i, str(SLOT_GLYPHS[i]))
	# 初始上报：Hotbar ready 已应用 selected_index 初值（不发信号），此处读取补报
	_on_equip_selected(int(hotbar.call("get_selected")))


## Hotbar @s_selected 回调：槽位索引 → 物品 id 写状态总线
func _on_equip_selected(index: int) -> void:
	var item := ""
	if index >= 0 and index < SLOT_ITEMS.size():
		item = str(SLOT_ITEMS[index])
	_state().set_state(KEY_EQUIP, item)


func _state():
	return Engine.get_singleton("GDSTATE")
