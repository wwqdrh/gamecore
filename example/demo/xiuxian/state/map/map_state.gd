# 地图资源状态 Bean（分类：地图资源）—— 仙途 demo 存档级地图状态层
#
# 职责：持有随存档持久化的地图资源采集状态。
# 当前字段：
#   · harvested —— 已采集「待再生」格子集合（Dictionary：键 = 格子 key 字符串，
#     值 = 采集时刻的游戏秒（XiuCalendarState.game_secs 时间轴）；
#     0.0 = 已再生弹出。key 编码 = (x << 32) | y
#     （与框架 GdDecoLayer.cell_to_key 一致；GJson 数值一律 float，
#     键走字符串避免大整数精度问题）。
#     存时间戳而非标记位：重开后由 deco_harvest 按「此刻 - 采集时刻」换算
#     已冷却时长——再生节拍跨重启延续，不会每次开游戏重新等满间隔；
#     旧格式值 1.0（标记位）会被视为早已到期的时刻 → 首次加载弹出（矿回一次）。
#
#   · terrain_grids —— 各地图挖地地形网格（Dictionary：键 = 地图别名，
#     值 = Array[float]（行主序地形下标，与 GdDigLayer.get_terrain_grid
#     对齐；GJson 数值一律 float）。dig_save.gd 进场恢复 / 变化写回。
#
#   · farm_states —— 各地图农田状态（Dictionary：键 = 地图别名，
#     值 = GdDigLayer.get_farm_state dump：{"wet": {"x,y": 1.0},
#     "crops": {"x,y": {"stage": float, "t": float}}}。wet 值落盘前统一
#     转 1.0（规避 bool 落盘不确定性；恢复侧只看键存在性不看值）。
#     dig_save.gd 轮询 diff 写回 / 进场恢复。
#
# 与框架 GdDecoLayer 的协作（框架零业务感知）：
#   进场：deco_harvest.gd（图内宿主）读本 Bean → layer.set_harvested_cells 注入
#         → spawn_deco 跳过这些格子（挖掉的矿不会重开游戏就全部长回）
#   采集：回调读矿石 deco_cell meta → add_harvested（框架同时已记录运行时队列）
#   再生：框架每 deco_regrow_secs 随机补种并发 s_regrowned → remove_harvested
#
# 持久化：走 GdBean 标准链路（user://coredata_{save_id}.data，scope "init"），
# 写法与 XiuItemState.bag 同款（duplicate 整字典替换，update 现值比对天然去重）。
class_name XiuMapState
extends GdBean

## 已采集待再生格子集合（键 "%d" % cell_key → 采集时刻游戏秒 / 0.0 已弹出）
var harvested: Dictionary = {}

## 各地图挖地地形网格（键 = 地图别名 → Array[float] 行主序地形下标）
var terrain_grids: Dictionary = {}

## 各地图农田状态（键 = 地图别名 → GdDigLayer.get_farm_state dump）
var farm_states: Dictionary = {}


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuMapState:
	return GdBean.bean("xiuxian_map", func(): return new())


## 记录一格已采集（进入再生冷却；at_secs = 采集时刻的游戏秒）
func add_harvested(cell_key: int, at_secs: float) -> void:
	var d := harvested.duplicate(true)
	d[str(cell_key)] = at_secs
	update("harvested", d, {}, false)


## 一格再生完成（弹出冷却队列）
func remove_harvested(cell_key: int) -> void:
	var d := harvested.duplicate(true)
	d[str(cell_key)] = 0.0
	update("harvested", d, {}, false)


## 当前冷却中的格子 key 列表（过滤掉已弹出的 0 值项）
func get_harvested_keys() -> PackedInt64Array:
	var out := PackedInt64Array()
	for k in harvested.keys():
		if float(harvested[k]) > 0.0:
			out.append(int(str(k)))
	return out


## 原始冷却记录（键 = 格子 key 字符串，值 = 采集时刻游戏秒；deco_harvest
## 进场恢复用：按时间换算已冷却时长，到期的弹出、未到期的注入 deco 层）
func get_harvested() -> Dictionary:
	return harvested


## 清空全部记录（测试用；逐项置 0 保持结构可去重）
func clear_harvested() -> void:
	var d := harvested.duplicate(true)
	for k in d.keys():
		d[k] = 0.0
	update("harvested", d, {}, true)


## 保存某地图的挖地地形整网格（值转 Array[float] 落盘，GJson 数值一律 float；
## 整表 duplicate 替换，update 现值比对天然去重——地形未变化不落盘不通知）
func save_terrain_grid(alias: String, grid: PackedInt32Array) -> void:
	var arr: Array = []
	for v in grid:
		arr.append(float(v))
	var d := terrain_grids.duplicate(true)
	d[alias] = arr
	update("terrain_grids", d, {}, false)


## 读取某地图的存档地形网格（无存档返回空——进场用初始补丁地形）
func load_terrain_grid(alias: String) -> PackedInt32Array:
	if not terrain_grids.has(alias):
		return PackedInt32Array()
	var out := PackedInt32Array()
	for v in terrain_grids[alias]:
		out.append(int(v))
	return out


## 清空全部地形网格（测试用；force 绕过同值短路）
func clear_terrain_grids() -> void:
	update("terrain_grids", {}, {}, true)


## 保存某地图农田状态（wet 的 true 统一转 1.0 float，规避 bool 落盘
## 不确定性——恢复侧只看键存在性不看值；整表 duplicate 替换，update
## 现值比对天然去重，状态未变化不落盘不通知）
func save_farm_state(alias: String, state: Dictionary) -> void:
	var norm: Dictionary = {}
	var wet_src: Dictionary = state.get("wet", {})
	var wet_d: Dictionary = {}
	for k in wet_src.keys():
		wet_d[str(k)] = 1.0
	norm["wet"] = wet_d
	var crops_src: Dictionary = state.get("crops", {})
	var crops_d: Dictionary = {}
	for k in crops_src.keys():
		var cd: Dictionary = crops_src[k]
		crops_d[str(k)] = {
			"stage": float(cd.get("stage", 0.0)),
			"t": float(cd.get("t", 0.0)),
			"id": float(cd.get("id", 0.0)),
		}
	norm["crops"] = crops_d
	var d := farm_states.duplicate(true)
	d[alias] = norm
	update("farm_states", d, {}, false)


## 读取某地图存档农田状态（无存档返回空——进场用初始空农田）
func load_farm_state(alias: String) -> Dictionary:
	return farm_states.get(alias, {})


## 清空全部农田状态（测试用；force 绕过同值短路）
func clear_farm_states() -> void:
	update("farm_states", {}, {}, true)
