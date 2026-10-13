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
#           耕地 → 湿润，作物只在湿润时生长），7 号 = 种子（多作物轮换槽：
#           选中后**再按 7 / 再点击该格**轮换 SEED_IDS 里下一种种子，
#           图标同步更新；播种时 player 按当前种子 id 匹配作物 def）
extends HBoxContainer

## 与 player.gd 的 KEY_EQUIP 一致（demo 各自声明，避免 class_name 缓存依赖）
const KEY_EQUIP := "mainhud.equip"

## 槽位 → 物品 id（"" = 空槽 / 卸下）。
## 1 空手 2 枪支 3 近战 4 镐 5 锄 6 水壶 7 种子（轮换）
const SLOT_ITEMS := ["", "gun", "sword_qingfeng", "tool_pickaxe", "tool_hoe",
	"tool_watering_can", "seeds_veg"]

## 槽位图标占位（emoji；正式素材就位后换 TextureRect）
## 5 号锄头暂用锤 emoji 占位（Unicode 无锄头字符），同 icon_map img_tool_hoe
const SLOT_GLYPHS := ["✋", "🔫", "🗡️", "⛏️", "🔨", "🪣", "🌱"]

## 种子槽（7 号）轮换的种子 id 集（与 player.gd SEED_IDS 一致）与对应图标
const SEED_SLOT := 6
const SEED_IDS := ["seeds_veg", "seeds_radish", "seeds_melon"]
const SEED_GLYPHS := ["🌱", "🥕", "🍉"]

## 当前轮换到的种子下标（SLOT_ITEMS[SEED_SLOT] 与之同步）
var _seed_idx := 0

## 上一帧 Hotbar 选中槽位（「再按 7 = 轮换」的判定基准：输入事件阶段
## Hotbar（子节点）先处理——首次按 7 时本节点看到 selected 已变 6，
## 无法据此区分「刚选中」还是「再按」；用帧末快照对齐语义）
var _sel_prev := -1


func _ready() -> void:
	var hotbar: Control = find_child("EquipSlots", true, false)
	if hotbar == null:
		return
	for i in SLOT_GLYPHS.size():
		if str(SLOT_GLYPHS[i]) != "":
			hotbar.call("set_slot_glyph", i, str(SLOT_GLYPHS[i]))
	# 种子轮换入口 1：点击已选中的种子格（Hotbar 同格 select_slot 会 no-op
	# 不再发 s_selected，此处直接监听槽位 Panel 的 gui_input 补一刀）
	for i in hotbar.get_child_count():
		var slot := hotbar.get_child(i)
		if slot is Panel:
			slot.gui_input.connect(_on_slot_gui_input.bind(i))
	set_process(true)
	# 初始上报：Hotbar ready 已应用 selected_index 初值（不发信号），此处读取补报
	_on_equip_selected(int(hotbar.call("get_selected")))


## 帧末快照选中槽位（输入事件先于本帧 _process——事件里比对的是上一帧值）
func _process(_delta: float) -> void:
	var hotbar: Control = find_child("EquipSlots", true, false)
	if hotbar != null:
		_sel_prev = int(hotbar.call("get_selected"))


## Hotbar @s_selected 回调：槽位索引 → 物品 id 写状态总线
func _on_equip_selected(index: int) -> void:
	var item := ""
	if index >= 0 and index < SLOT_ITEMS.size():
		item = str(SLOT_ITEMS[index])
	# 种子槽：保持当前轮换选择（切走再切回不重置种子），图标同步
	if index == SEED_SLOT:
		item = str(SEED_IDS[_seed_idx])
		var hotbar: Control = find_child("EquipSlots", true, false)
		if hotbar != null:
			hotbar.call("set_slot_glyph", SEED_SLOT, str(SEED_GLYPHS[_seed_idx]))
	_state().set_state(KEY_EQUIP, item)


## 种子轮换入口 2：数字键再按。Hotbar 的 unhandled_key_input 不消费事件
## （无 set_input_as_handled），同格 select_slot no-op 后事件继续传播到本节点；
## **只有上一帧就已选中种子格才算「再按」**（否则首次选中会被误判轮换一次）
func _unhandled_key_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.is_echo():
		if event.keycode != KEY_7 or _sel_prev != SEED_SLOT:
			return
		var hotbar: Control = find_child("EquipSlots", true, false)
		if hotbar != null and int(hotbar.call("get_selected")) == SEED_SLOT:
			_cycle_seed()


## 点击槽位（gui_input 转发）：点的是已选中的种子格 → 轮换
## （_sel_prev 语义同上：本帧刚从别的格点过来 = 选中，不轮换）
func _on_slot_gui_input(event: InputEvent, index: int) -> void:
	if index != SEED_SLOT or _sel_prev != SEED_SLOT:
		return
	if event is InputEventMouseButton and event.pressed \
			and event.button_index == MOUSE_BUTTON_LEFT:
		_cycle_seed()


## 轮换到下一种种子：槽位图标同步 + 直接写状态总线
## （玩家 watch mainhud.equip 收到新种子 id → plant_seed 匹配对应作物 def）
func _cycle_seed() -> void:
	_seed_idx = (_seed_idx + 1) % SEED_IDS.size()
	var hotbar: Control = find_child("EquipSlots", true, false)
	if hotbar != null:
		hotbar.call("set_slot_glyph", SEED_SLOT, str(SEED_GLYPHS[_seed_idx]))
	_state().set_state(KEY_EQUIP, str(SEED_IDS[_seed_idx]))
	print("[EquipBar] 种子轮换 → %s" % str(SEED_IDS[_seed_idx]))


func _state():
	return Engine.get_singleton("GDSTATE")
