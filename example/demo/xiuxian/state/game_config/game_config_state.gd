# 游戏配置状态 Bean（分类：游戏配置）—— 仙途 demo 存档级游戏配置层
#
# 职责：持有随存档持久化的玩法参数（而非全局 game_config.json 静态表）。
# 当前字段：
#   · day_length —— 一昼夜对应的「现实秒数」（时间流速：越小越快）。
#     天气系统（WeatherManager）与游戏日历（CalendarManager）共用同一时间轴：
#     天气内部按「游戏秒」流转，由 CalendarManager 按本配置换算下发流速：
#       time_scale（游戏秒/现实秒）= 86400（游戏秒/昼夜） / day_length
#     天数推算 = game_time / 86400 + 1，不受流速影响（同一时间轴）。
#
# 持久化：走 GdBean 标准链路（user://coredata_{save_id}.data，scope "init"），
# 每个存档独立一份流速配置。
class_name XiuGameConfigState
extends GdBean

## 一昼夜对应的现实秒数（默认 1200 = 现实 20 分钟一个游戏日，配合 NPC 日程
## 观感；时间流速 = 86400/1200 = 72 游戏秒/现实秒）
var day_length: float = 1200.0

## 地图 deco 资源再生间隔（现实秒；0 = 禁用再生）。
## 每到间隔地图 deco 层随机挑空格长出一批缺口资源（生长位置不固定）。
## demo 默认 120 = 2 分钟长出下一批（用户配置）。
var deco_regrow_secs: float = 120.0

## 游戏秒/昼夜（一昼夜的游戏内秒数，与 WeatherManager.day_length 默认一致；
## 日历与流速换算共同依赖的常量）
const GAME_SECONDS_PER_DAY := 86400.0

## day_length 允许的最小值（现实秒）——防误配出「一天几秒」的疯-speed
const MIN_DAY_LENGTH := 30.0


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuGameConfigState:
	return GdBean.bean("xiuxian_gameconfig", func(): return new())


## 写入一昼夜现实秒数（会触发 watch；红线条目：只传新值表达式，
## 严禁先改成员再 update 同值——去重比对 get() 现值会短路）
func set_day_length(v: float) -> void:
	var clamped: float = maxf(v, MIN_DAY_LENGTH)
	update("day_length", clamped, {}, false)


## 写入 deco 再生间隔（现实秒；0 = 禁用再生）
func set_deco_regrow_secs(v: float) -> void:
	update("deco_regrow_secs", maxf(v, 0.0), {}, false)


## 还原演示基线（跨运行确定性测试用）
func reset_demo() -> void:
	day_length = 600.0
	update("day_length", day_length, {}, true)
	deco_regrow_secs = 120.0
	update("deco_regrow_secs", deco_regrow_secs, {}, true)
