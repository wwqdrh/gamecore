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
