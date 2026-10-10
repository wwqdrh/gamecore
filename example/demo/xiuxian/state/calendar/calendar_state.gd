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


## 还原演示基线（跨运行确定性测试用）
func reset_demo() -> void:
	day = 1
	update("day", day, {}, true)
