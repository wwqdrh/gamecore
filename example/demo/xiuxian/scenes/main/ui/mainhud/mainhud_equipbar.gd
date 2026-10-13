# 主界面装备栏控制器 —— 由 mainhud_equipbar.gml 的 <ui script="mainhud_equipbar.gd">
# 声明，挂载到装备栏根节点（EquipBar）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   命令上行：Hotbar 选中（数字键 1~7 / 点击）→ 读槽位物品表 → 写 KEY_EQUIP
#   （mainhud.equip，值为物品 id 字符串；空串 = 空槽 / 卸下）
#   消费方 player.gd watch 同名键：枪支 → 射击模式；近战武器 → 挥击模式
#   （状态下行，本控制器不感知）
#
# 槽位配置：1 号 = 空手（equip 上报空串，点击相邻成熟作物可采摘入包，
#           远处点击仍是寻路；默认选中——开局手上什么都不拿），
#           2 号 = 枪支，3 号 = 近战武器（青锋剑，item.json 装备类），
#           4 号 = 挖矿工具（玄铁镐，可开采地图 deco 矿石——碰撞 layer5/16，
#           见 gd_map_deco.rs），5 号 = 锄地工具（玄铁锄，可锄开支持 dig 协议
#           的地图地形——如 home 的 GdDigLayer 草地 → 沙地，见 gd_dig_layer.rs），
#           6 号 = 浇灌工具（青铜水壶，对已开垦耕地浇水——农田状态流转
#           耕地 → 湿润，作物只在湿润时生长），7 号 = 种子（青菜种子，
#           播种在湿润耕地上，经 crop_stages 个阶段长到成熟）
extends HBoxContainer

## 与 player.gd 的 KEY_EQUIP 一致（demo 各自声明，避免 class_name 缓存依赖）
const KEY_EQUIP := "mainhud.equip"

## 槽位 → 物品 id（"" = 空槽 / 卸下）。
## 1 空手 2 枪支 3 近战 4 镐 5 锄 6 水壶 7 种子
const SLOT_ITEMS := ["", "gun", "sword_qingfeng", "tool_pickaxe", "tool_hoe",
	"tool_watering_can", "seeds_veg"]

## 槽位图标占位（emoji；正式素材就位后换 TextureRect）
## 5 号锄头暂用锤 emoji 占位（Unicode 无锄头字符），同 icon_map img_tool_hoe
const SLOT_GLYPHS := ["✋", "🔫", "🗡️", "⛏️", "🔨", "🪣", "🌱"]


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
