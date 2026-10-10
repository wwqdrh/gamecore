# 秘境（随机副本）流程控制器 —— 挂主场景根 index.tscn（与地图/天气同层，不随换图销毁）
#
# 流程编排（数据在 XiuDungeonState，随机生成在 dungeon_map.gd，本脚本只管流转）：
#   1. mainhud「秘境」按钮 → start_run()：记录入口（固定地图+玩家所在格）→
#      open_map(xiuxian_dungeon)：随机场景按 run_seed 出图 + 随机刷怪
#   2. 接地图 Floor 控制器（组 "dungeon_floor"）的 s_floor_cleared：
#      · 非 boss 层 → toast「已扫荡」→ 1.2s 后 advance_floor 重开随机场景（下一层）
#      · boss 层   → settle()：发奖（修为/灵石/战利品/妖王首杀解锁秘境残图）→
#        open_map 回入口 → 打开结算弹窗（ui_id=DungeonSettleModal）
#   3. 存档恢复残留 running 状态（重进应用时副本未重建）→ _wire 里作废本次进度
#
# 组件查找约定：经 GdSceneRoot 组件注册表（register_component/get_component），
# mainhud 跨组件调用 start_run 走同一张表；本节点 _process 逐帧重试注册
# （GdScene 默认管理器延迟创建，同 player.gd/_wire 模式）。
extends CanvasLayer

## 组件表注册名（scene_root.get_component("DungeonManager")）
const COMPONENT_NAME := "DungeonManager"
## 层间流转的停顿（扫荡提示展示时长）
const NEXT_FLOOR_DELAY := 1.2

var _map: Node = null
var _state: XiuDungeonState
var _toast: Label
var _toast_tween: Tween
var _wired := false
var _advancing := false  # 换层/结算流转中（防清场信号重入）


func _ready() -> void:
	layer = 10
	_make_toast()


func _process(_delta: float) -> void:
	if _wire():
		set_process(false)


## 一次副本挑战入口（mainhud「秘境」按钮调用）；进行中/流转中返回 false
func start_run() -> bool:
	if not _wired or _advancing:
		return false
	if str(_state.status) == XiuDungeonState.STATUS_RUNNING:
		return false
	var from: String = str(_map.get_current_alias())
	if from == XiuDungeonState.DUNGEON_ALIAS:
		return false
	# 入口记录：当前固定地图 + 玩家所在格（结算后原路送回）
	var cell := Vector2i(10, 8)
	var player: Node = get_tree().get_first_node_in_group("player")
	var cur_map: Object = _map.get_current_map()
	if player != null and cur_map != null and cur_map.has_method("world_to_cell"):
		cell = cur_map.world_to_cell(player.global_position)
	_state.start_run(from, cell)
	var ok: bool = _map.open_map(XiuDungeonState.DUNGEON_ALIAS, cell)
	if ok:
		toast("进入落霞秘境 · 第 1 层")
	return ok


# ------------------------------------------------------------------ 流转

func _on_map_changed(alias: String) -> void:
	if alias != XiuDungeonState.DUNGEON_ALIAS:
		return
	# 新地图实例上的 Floor 控制器（旧地图已释放，组内唯一）接清场信号
	var floor_node: Node = get_tree().get_first_node_in_group("dungeon_floor")
	if floor_node == null:
		return
	if not floor_node.s_floor_cleared.is_connected(_on_floor_cleared):
		floor_node.s_floor_cleared.connect(_on_floor_cleared)


## 本层扫荡完成：boss 层直接结算；否则提示后进入下一层
func _on_floor_cleared() -> void:
	if _advancing:
		return
	_advancing = true
	if _state.is_boss_floor():
		_settle()
		_advancing = false
		return
	toast("第 %d 层已扫荡，正在进入下一层…" % int(_state.floor))
	get_tree().create_timer(NEXT_FLOOR_DELAY).timeout.connect(_goto_next_floor)


func _goto_next_floor() -> void:
	_advancing = false
	if str(_state.status) != XiuDungeonState.STATUS_RUNNING:
		return
	_state.advance_floor()
	_map.open_map(XiuDungeonState.DUNGEON_ALIAS, Vector2i(18, 13))
	toast("进入第 %d 层%s" % [
		int(_state.floor), " · 妖王镇守！" if _state.is_boss_floor() else ""])


## 通关结算：发奖 → 写结算视图 → 回入口地图 → 弹结算窗
func _settle() -> void:
	var view: Dictionary = _grant_rewards()
	toast("秘境通关！")
	_state.write_settle(view)
	_state.finish_run()
	_map.open_map(str(_state.from_map), _state.from_cell)
	# 结算弹窗（<Modal ui_id="DungeonSettleModal">，面板 watch settle_view 刷新）
	var modal: Object = GdUIManager.find_ui("DungeonSettleModal")
	if modal != null:
		modal.call("open")


## 奖励发放：通关修为 + 灵石 + 随机战利品（2~4 条）+ 妖王首杀解锁秘境残图
func _grant_rewards() -> Dictionary:
	var char_state := XiuCharacterState.ins()
	var item_state := XiuItemState.ins()
	var floors := int(_state.total_floors)
	var exp_bonus := 100 * floors
	var stones := 60 * floors
	char_state.add_exp(exp_bonus)
	char_state.add_spirit_stones(stones)

	var rng := RandomNumberGenerator.new()
	rng.seed = int(_state.run_seed)  # 结算随 run 种子：同一次挑战结果可追溯
	var items: Array = []
	var drops := 2 + rng.randi_range(0, 2)
	for i in drops:
		var id: String = XiuDungeonState.LOOT_POOL[rng.randi_range(0, XiuDungeonState.LOOT_POOL.size() - 1)]
		var n := rng.randi_range(1, 2)
		item_state.add_item(id, n)
		items.append({
			"icon": "🧪",
			"name": str(item_state.get_item(id).get("name", id)),
			"count": "x%d" % n,
			"note": "",
		})
	# 妖王首杀：解锁并发放「秘境残图」（一次性剧情奖励）
	if _state.mark_boss_cleared():
		item_state.unlock_item("map_secret")
		item_state.add_item("map_secret", 1)
		items.append({"icon": "📜", "name": "秘境残图", "count": "x1", "note": "首杀奖励"})
	return {
		"title": "秘境通关",
		"floors": floors,
		"exp": exp_bonus,
		"stones": stones,
		"items": items,
	}


# ------------------------------------------------------------------ 接线 / 提示

func _wire() -> bool:
	if _wired:
		return true
	var scene_root: Node = _find_scene_root()
	if scene_root == null:
		return false
	scene_root.register_component(COMPONENT_NAME, self)
	_map = scene_root.get_component("MapManager")
	if _map == null or not _map.has_method("open_map") or _map.get_current_map() == null:
		return false
	_wired = true
	_map.connect("s_map_changed", _on_map_changed)
	_state = XiuDungeonState.ins()
	# 存档恢复残留的 running（重进应用时副本未重建）：作废本次进度，玩家在固定场景
	if str(_state.status) == XiuDungeonState.STATUS_RUNNING:
		_state.finish_run()
	return true


## 场景根（GdSceneRoot）：经 GDCORE 全局节点表获取（同 player.gd）
func _find_scene_root() -> Node:
	if not Engine.has_singleton("GDCORE"):
		return null
	var node: Object = Engine.get_singleton("GDCORE").get_global_node("default")
	return node if node is Node else null


## 顶部横幅提示（1.8s 上浮渐隐；重复调用覆盖前一条）
func toast(msg: String) -> void:
	_toast.text = msg
	_toast.modulate = Color(1, 1, 1, 0)
	if _toast_tween != null and _toast_tween.is_valid():
		_toast_tween.kill()
	_toast_tween = create_tween()
	_toast_tween.tween_property(_toast, "modulate:a", 1.0, 0.15)
	_toast_tween.tween_interval(1.4)
	_toast_tween.tween_property(_toast, "modulate:a", 0.0, 0.3)


func _make_toast() -> void:
	_toast = Label.new()
	_toast.name = "Toast"
	_toast.position = Vector2(0, 150)
	_toast.size = Vector2(1920, 44)
	_toast.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_toast.add_theme_font_size_override("font_size", 24)
	_toast.add_theme_color_override("font_color", Color(1.0, 0.88, 0.55))
	_toast.add_theme_color_override("font_outline_color", Color(0.12, 0.06, 0.02))
	_toast.add_theme_constant_override("outline_size", 6)
	_toast.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_toast.modulate = Color(1, 1, 1, 0)
	add_child(_toast)
