# 挖地地形持久化宿主（demo 侧）—— 挂在 GdDigLayer 地图场景内（home.tscn 的
# DigSave 节点），锄过的地形随档保存 / 进场自动恢复。
#
# 分工（框架零业务感知）：
#   - 框架 GdDigLayer：dig() 地形变化 + s_terrain_changed 通知 +
#     get/set_terrain_grid 整网格 dump/restore（地图小，百格级整存最简）
#   - demo（本脚本）：
#     · 进场：XiuMapState.terrain_grids[map_alias] → set_terrain_grid 覆盖
#       导出配置的初始补丁（无存档 = 初始地形）
#     · 变化：s_terrain_changed → 整网格写回 Bean（update 现值比对天然去重，
#       地形未变化不落盘不通知）
#
# 时序：本节点是地图根的子节点，子 ready 先于父 GdDigLayer ready（网格尚未
# 按导出配置初始化）→ 恢复走 call_deferred（父 ready 完成后执行，set_terrain_grid
# 全量重铺）。与 deco_harvest 的 spawn_deco 时序处理同款。
extends Node

## 地图别名（Bean 存储键；须与 MapManager.map_paths 注册的别名一致）
@export var map_alias := ""

var _layer: Node = null


func _ready() -> void:
	_layer = get_parent()
	if _layer == null or not _layer.has_method("get_terrain_grid"):
		return
	_layer.s_terrain_changed.connect(_on_terrain_changed)
	_restore.call_deferred()


## 进场恢复：存档网格覆盖初始地形（deferred——等父地图 ready 初始化完网格）
func _restore() -> void:
	var grid: PackedInt32Array = XiuMapState.ins().load_terrain_grid(map_alias)
	if not grid.is_empty():
		_layer.call("set_terrain_grid", grid)


## 任一格地形变化 → 整网格写回（同值 update 自动短路，锄挖瞬间落盘）
func _on_terrain_changed(_cell: Vector2i, _from: String, _to: String) -> void:
	XiuMapState.ins().save_terrain_grid(map_alias, _layer.call("get_terrain_grid"))
