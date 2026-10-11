# 游戏日历状态 Bean（分类：日历）—— 仙途 demo 游戏状态层
#
# 职责：持有游戏内天数（随存档持久化）。天数由 CalendarManager 按
# WeatherManager 的游戏时长（get_game_time / day_length）推算后写入，
# mainhud 天数徽章与剧情解锁（宗门大比第 7 天开赛）都 watch 本 Bean。
class_name XiuCalendarState
extends GdBean

## 宗门大比开赛日（游戏第 N 天；CalendarManager 到日自动解锁任务）
const TOURNAMENT_DAY := 7

## 当前游戏天数（1 起；跨存档持久化）
var day: int = 1

## 当前游戏时长（秒；WeatherManager 游戏时间轴，CalendarManager 轮询写入）。
## 用途：deco 采集冷却的跨重启时钟基准——冷却记录存「采集时刻的游戏秒」，
## 重开后按 此刻 - 采集时刻 换算已冷却时长，再生节拍不因重启清零
var game_secs: float = 0.0


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuCalendarState:
	return GdBean.bean("xiuxian_calendar", func(): return new())


## 写入天数（同值幂等不落盘；红线：严禁先改成员再 update 同值——
## update 的去重读的是 get() 现值，先改成员会被去重短路，必须只传新值
## 由 update 内部 set 成员）
func set_day(v: int) -> void:
	if int(day) == v:
		return
	update("day", v, {}, false)


## 写入游戏时长（秒；0.5s 轮询粒度，暂停/未推进时同值 update 短路不落盘）
func set_game_secs(v: float) -> void:
	if absf(float(game_secs) - v) < 0.001:
		return
	update("game_secs", v, {}, false)


## 还原演示基线（跨运行确定性测试用）
func reset_demo() -> void:
	day = 1
	update("day", day, {}, true)
	game_secs = 0.0
	update("game_secs", game_secs, {}, true)
