# 挖地地形 + 农田状态持久化宿主（demo 侧）—— 挂在 GdDigLayer 地图场景内
# （home.tscn 的 DigSave 节点），锄过的地形与农田随档保存 / 进场自动恢复。
#
# 分工（框架零业务感知）：
#   - 框架 GdDigLayer：
#     · dig() 地形变化 + s_terrain_changed 通知 + get/set_terrain_grid
#       整网格 dump/restore（地图小，百格级整存最简）
#     · water/plant/harvest() 农田状态流转 + get/set_farm_state dump/restore
#   - demo（本脚本）：
#     · 进场：XiuMapState.terrain_grids[map_alias] → set_terrain_grid 覆盖
#       导出配置的初始补丁（无存档 = 初始地形）；farm_states → set_farm_state
#     · 地形变化：s_terrain_changed → 整网格写回 Bean（update 现值比对天然
#       去重，地形未变化不落盘不通知）
#     · 农田状态：1s 轮询 diff 写回（湿润计时/作物生长/干燥都是框架内部
#       tick，无逐事件信号——轮询全量 dump 后 dict 比对，无变化 update 短路
#       零 IO；写回前 wet 值统一转 1.0，规避 bool 落盘不确定性）
#
# 时序：本节点是地图根的子节点，子 ready 先于父 GdDigLayer ready（网格尚未
# 按导出配置初始化）→ 恢复走 call_deferred（父 ready 完成后执行，set_terrain_grid
# 全量重铺）。与 deco_harvest 的 spawn_deco 时序处理同款。
extends Node

## 地图别名（Bean 存储键；须与 MapManager.map_paths 注册的别名一致）
@export var map_alias := ""

## 农田状态轮询间隔秒（框架内部 tick 无逐事件信号，轮询 diff 兜底）
const FARM_POLL_SECS := 1.0

var _layer: Node = null
var _farm_cache: Dictionary = {}
var _poll: Timer = null


func _ready() -> void:
	_layer = get_parent()
	if _layer == null or not _layer.has_method("get_terrain_grid"):
		return
	_layer.s_terrain_changed.connect(_on_terrain_changed)
	_restore.call_deferred()


## 进场恢复：存档网格覆盖初始地形 + 农田状态注入
## （deferred——等父地图 ready 初始化完网格/农田数组）
func _restore() -> void:
	var grid: PackedInt32Array = XiuMapState.ins().load_terrain_grid(map_alias)
	if not grid.is_empty():
		_layer.call("set_terrain_grid", grid)
	# 农田层：恢复存档状态（无存档 = 全空），并以此为基线开轮询
	if _layer.has_method("get_farm_state") and _layer.has_method("set_farm_state"):
		var farm: Dictionary = XiuMapState.ins().load_farm_state(map_alias)
		if not farm.is_empty():
			_layer.call("set_farm_state", farm)
		_farm_cache = _layer.call("get_farm_state")
		_poll = Timer.new()
		_poll.wait_time = FARM_POLL_SECS
		_poll.timeout.connect(_poll_farm)
		add_child(_poll)
		_poll.start()


## 农田状态轮询：与上次写回快照比对，变化才整份写回（update 现值比对
## 天然去重——相同内容二次写回会被 GdBean 短路，不落盘不通知）
func _poll_farm() -> void:
	if _layer == null:
		return
	var farm: Dictionary = _layer.call("get_farm_state")
	if farm != _farm_cache:
		_farm_cache = farm
		XiuMapState.ins().save_farm_state(map_alias, farm)


## 任一格地形变化 → 整网格写回（同值 update 自动短路，锄挖瞬间落盘）
func _on_terrain_changed(_cell: Vector2i, _from: String, _to: String) -> void:
	XiuMapState.ins().save_terrain_grid(map_alias, _layer.call("get_terrain_grid"))
