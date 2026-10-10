# 秘境探索场景流程控制器 —— 挂 explore 场景根 index.tscn（scenes/explore/）
#
# 进入秘境 = 整个场景切换（main → explore，走 GdScene 转场遮罩）：
#   main 场景 DungeonManager.start_run() 记录入口（from_map/from_cell）→
#   change_scene("xiuxian_explore") → 本控制器 wire 后按 XiuDungeonState 自动开图。
#
# 流程编排（数据在 XiuDungeonState，随机生成在 dungeon_map.gd，本脚本只管流转）：
#   1. wire 完成 & status=running → open_map(xiuxian_dungeon)：随机场景按层种子出图
#   2. 接地图 Floor 控制器（组 "dungeon_floor"）的 s_floor_cleared：
#      · 非 boss 层 → toast「已扫荡」→ 1.2s 后 advance_floor 重开随机场景（下一层）
#      · boss 层   → settle()：发奖（修为/灵石/战利品/妖王首杀解锁秘境残图）→
#        打开结算弹窗（ui_id=DungeonSettleModal，explore.gml 装配）→
#        弹窗关闭（s_modal_closed）→ change_scene 回 xiuxian_main
#   3. 玩家中途放弃（explore HUD「离开秘境」按钮 → abandon()）：作废进度后回主场景
#      （from_map 保留，由 main 场景 DungeonManager 送回入口）
#
# 组件查找约定：经 GdSceneRoot 组件注册表（register_component/get_component）；
# 本节点 _process 逐帧重试注册（GdScene 管理器延迟创建，同 dungeon_manager 模式）；
# 场景切换时本节点随场景销毁，_exit_tree 注销组件表（防悬空引用）。
extends CanvasLayer

## 组件表注册名（scene_root.get_component("ExploreManager")）
const COMPONENT_NAME := "ExploreManager"
## 层间流转的停顿（扫荡提示展示时长）
const NEXT_FLOOR_DELAY := 1.2
## 第 1 层出生格（dungeon.tscn 生成器保证可行走）
const FIRST_SPAWN := Vector2i(10, 8)
## 第 2 层起的出生格（远离第 1 层入口，视觉上"深入"）
const NEXT_SPAWN := Vector2i(18, 13)

var _map: Node = null
var _state: XiuDungeonState
var _toast: Label
var _toast_tween: Tween
var _wired := false
var _advancing := false  # 换层/结算流转中（防清场信号重入）
var _leaving := false    # 回主场景流转中（防重复 change_scene）
var _retry_back := false  # 返回切换被转场防重入拒绝（is_changing）→ _process 逐帧重试
var _retry_left := 0


func _ready() -> void:
	layer = 10
	_make_toast()


func _process(_delta: float) -> void:
	if not _wire():
		return
	if _retry_back:
		_retry_left -= 1
		if _do_back_to_main():
			_retry_back = false
			set_process(false)
		elif _retry_left <= 0:
			push_warning("ExploreManager: 返回主场景重试耗尽")
			_retry_back = false
			set_process(false)
	else:
		set_process(false)


func _exit_tree() -> void:
	# 场景切换销毁本节点：注销组件表（GdSceneRoot 表跨场景共享，防悬空引用）。
	# 必须 call_deferred：场景替换发生在 GdSceneRoot 的 Rust 调用栈内，
	# 同步回调 unregister_component 会重入 bind_mut panic。
	if _wired:
		var scene_root: Node = _find_scene_root()
		if scene_root != null:
			scene_root.call_deferred("unregister_component", COMPONENT_NAME)


## 中途放弃挑战（explore HUD「离开秘境」按钮）：进度作废，回主场景
func abandon() -> bool:
	if _leaving:
		return false
	_leaving = true
	if str(_state.status) == XiuDungeonState.STATUS_RUNNING:
		_state.finish_run()  # from_map 保留 → main 场景送回入口
	return _back_to_main()


# ------------------------------------------------------------------ 流转

## 本层扫荡完成：boss 层直接结算；否则提示后进入下一层
func _on_floor_cleared() -> void:
	if _advancing or _leaving:
		return
	# 运行中守卫：结算（finish_run 置状态空）后迟到的清场信号直接忽略，
	# 防止同帧多杀导致的 s_floor_cleared 重放造成二次发奖
	if str(_state.status) != XiuDungeonState.STATUS_RUNNING:
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
	if _leaving or str(_state.status) != XiuDungeonState.STATUS_RUNNING:
		return
	_state.advance_floor()
	_map.open_map(XiuDungeonState.DUNGEON_ALIAS, NEXT_SPAWN)
	toast("进入第 %d 层%s" % [
		int(_state.floor), " · 妖王镇守！" if _state.is_boss_floor() else ""])


## 通关结算：发奖 → 写结算视图 → 状态回空 → 弹结算窗（关闭后回主场景）
func _settle() -> void:
	var view: Dictionary = _grant_rewards()
	toast("秘境通关！")
	_state.write_settle(view)
	_state.finish_run()
	# 结算弹窗（<Modal ui_id="DungeonSettleModal">，面板 watch settle_view 刷新）
	var modal: Object = GdUIManager.find_ui("DungeonSettleModal")
	if modal != null:
		if not modal.is_connected("s_modal_closed", _on_settle_closed):
			modal.connect("s_modal_closed", _on_settle_closed)
		modal.call("open")
	else:
		_back_to_main()  # 弹窗缺失兜底：直接回主场景


## 结算弹窗关闭 → 回主场景（main 的 DungeonManager 按 from_map 送回入口）
func _on_settle_closed() -> void:
	_back_to_main()


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
	# explore 场景 MapManager auto_load=false（仅注册不自动开图），无 current_map
	if _map == null or not _map.has_method("open_map"):
		return false
	_wired = true
	_map.connect("s_map_changed", _on_map_changed)
	_state = XiuDungeonState.ins()
	# 挑战进行中（正常进入路径）→ 按当前层数开图；层入口格按 floor 区分
	if str(_state.status) == XiuDungeonState.STATUS_RUNNING:
		var spawn := FIRST_SPAWN if int(_state.floor) <= 1 else NEXT_SPAWN
		if _map.open_map(XiuDungeonState.DUNGEON_ALIAS, spawn):
			toast("进入落霞秘境 · 第 %d 层%s" % [
				int(_state.floor), " · 妖王镇守！" if _state.is_boss_floor() else ""])
	return true


## 新地图实例上的 Floor 控制器（旧地图已释放，组内唯一）接清场信号
func _on_map_changed(alias: String) -> void:
	if alias != XiuDungeonState.DUNGEON_ALIAS:
		return
	var floor_node: Node = get_tree().get_first_node_in_group("dungeon_floor")
	if floor_node == null:
		return
	if not floor_node.s_floor_cleared.is_connected(_on_floor_cleared):
		floor_node.s_floor_cleared.connect(_on_floor_cleared)


## 回主场景（沿父链向上找 GdScene 场景根的 change_scene，同 title_menu 约定）；
## 切换被转场防重入拒绝时转 _process 逐帧重试（返回值只表示"流转已受理"）
func _back_to_main() -> bool:
	if _retry_back:
		return true
	_leaving = true
	if _do_back_to_main():
		return true
	_retry_back = true
	_retry_left = 600
	set_process(true)  # wire 成功后 _process 已停帧，重试需重新驱动
	return true


func _do_back_to_main() -> bool:
	var node: Node = get_parent()
	while node != null:
		if node.has_method("change_scene"):
			return node.call("change_scene", "xiuxian_main", {})
		node = node.get_parent()
	push_warning("ExploreManager: 未找到 GdScene 场景根，无法返回主场景")
	return false


## 场景根（GdSceneRoot）：经 GDCORE 全局节点表获取（同 dungeon_manager.gd）
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
