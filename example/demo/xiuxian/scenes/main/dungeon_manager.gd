# 秘境入口/送回控制器 —— 挂主场景根 index.tscn（scenes/main/）
#
# 职责（跨场景分工：探索中的流转/结算在 scenes/explore/explore_manager.gd）：
#   1. 入口：mainhud「秘境」按钮 → start_run()：记录入口（固定地图+玩家所在格，
#      存入 XiuDungeonState.from_map/from_cell）→ change_scene("xiuxian_explore")
#      进入独立探索场景（随机地图/随机敌人/多层爬塔/结算都在那边）
#   2. 送回：从探索场景回来（通关/放弃）后本节点随主场景重建，wire 时检测
#      from_map 非空 → open_map 送回入口 → 清空入口记录
#   3. 存档恢复：重进应用时若秘境仍 running（上次退出停在秘境中）→ 作废本次
#      挑战、留在主场景——秘境内不记录玩家位置（player._save_position 跳过），
#      存档位置=进秘境前的地图与落点，map_restore 直接恢复，不自动重返秘境
#
# 组件查找约定：经 GdSceneRoot 组件注册表（register_component/get_component），
# mainhud 跨组件调用 start_run 走同一张表；本节点 _process 逐帧重试注册；
# 场景切换时本节点随场景销毁，_exit_tree 注销组件表（防悬空引用）。
extends CanvasLayer

## 组件表注册名（scene_root.get_component("DungeonManager")）
const COMPONENT_NAME := "DungeonManager"

var _map: Node = null
var _state: XiuDungeonState
var _wired := false
var _retry_enter := false  # 入场切换被转场防重入拒绝（is_changing）→ _process 逐帧重试
var _retry_left := 0


func _process(_delta: float) -> void:
	if not _wire():
		return
	if _retry_enter:
		_retry_left -= 1
		if _enter_explore():
			_retry_enter = false
			set_process(false)
		elif _retry_left <= 0:
			# 重试耗尽（异常情况）：作废本次进度，避免玩家卡在"running 但没进图"
			push_warning("DungeonManager: 入场重试耗尽，作废本次秘境进度")
			_retry_enter = false
			_state.finish_run()
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


## 一次副本挑战入口（mainhud「秘境」按钮调用）；进行中返回 false。
## 只记录入口并切换场景：随机生成/层数流转/结算全部由探索场景的 ExploreManager 负责
func start_run() -> bool:
	if not _wired or _state == null:
		return false
	if str(_state.status) == XiuDungeonState.STATUS_RUNNING:
		return false
	var from: String = str(_map.get_current_alias())
	# 入口图守卫：主场景注册表内任意地图均可开局（家宅/萧宅/青石镇/青云坊/
	# 探索图…——家宅是 initial_map 默认出生图，旧守卫只认 FIXED_MAPS 三图，
	# 从家宅/探索图点「秘境」被静默拒绝 = 按钮无反应）；仅排除随机秘境自身
	#（主场景注册表本就不含它，双保险）
	if from.is_empty() or from == XiuDungeonState.DUNGEON_ALIAS:
		return false
	# 入口记录：当前固定地图 + 玩家所在格（回主场景后原路送回）
	var cell := Vector2i(10, 8)
	var player: Node = get_tree().get_first_node_in_group("player")
	var cur_map: Object = _map.get_current_map()
	if player != null and cur_map != null and cur_map.has_method("world_to_cell"):
		cell = cur_map.world_to_cell(player.global_position)
	# 入场前强制落盘当前位置（秘境内 player 不再记录位置，2s 节流窗口内的
	# 最后位移在此补记）：关闭游戏重开，存档位置即「进秘境前」的地图与落点
	if player != null:
		XiuPlayerState.ins().save_pos(from, player.global_position)
	_state.start_run(from, cell)
	# 挑战已开启：场景切换若因转场动画防重入（is_changing）被拒，转 _process 重试
	# （真实场景：进主场景淡入未完时立刻点「秘境」按钮，同步调用必被拒一次）
	if not _enter_explore():
		_retry_enter = true
		_retry_left = 600
		set_process(true)  # wire 成功后 _process 已停帧，重试需重新驱动
	return true


# ------------------------------------------------------------------ 接线

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
	_state = XiuDungeonState.ins()
	# 秘境进行中且出现在主场景 wire = 存档恢复（重进应用时上次退出停在秘境；
	# 正常入场路径 start_run 后立即切场景离开 main，不会回到这里）→
	# 作废本次挑战、留在主场景：秘境内不记录玩家位置，存档位置即「进秘境前」
	# 的地图与落点，map_restore 已按它开图恢复，不自动重返秘境
	if str(_state.status) == XiuDungeonState.STATUS_RUNNING:
		print("[DungeonManager] 重开检测到未完成秘境：作废进度，回到进秘境前位置")
		_state.finish_run()
		_state.clear_entry()
		return true
	# 从探索场景回来（通关/放弃，from_map 由 ExploreManager 保留）：送回入口
	var from_map: String = str(_state.from_map)
	if from_map != "":
		var cell: Vector2i = _state.from_cell
		_state.clear_entry()
		if _map.get_current_alias() != from_map:
			_map.open_map(from_map, cell if cell.x >= 0 else Vector2i(10, 8))
	return true


## 切换到探索场景（沿父链向上找 GdScene 场景根的 change_scene，同 title_menu 约定）
func _enter_explore() -> bool:
	var node: Node = get_parent()
	while node != null:
		if node.has_method("change_scene"):
			return node.call("change_scene", "xiuxian_explore", {})
		node = node.get_parent()
	push_warning("DungeonManager: 未找到 GdScene 场景根，无法进入秘境")
	return false


## 场景根（GdSceneRoot）：经 GDCORE 全局节点表获取（同 player.gd）
func _find_scene_root() -> Node:
	if not Engine.has_singleton("GDCORE"):
		return null
	var node: Object = Engine.get_singleton("GDCORE").get_global_node("default")
	return node if node is Node else null
