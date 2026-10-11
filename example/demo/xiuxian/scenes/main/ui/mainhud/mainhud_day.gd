# 天数徽章控制器 —— 由 mainhud_day.gml 的 <ui script="mainhud_day.gd"> 声明，
# 挂载到徽章根节点（DayBadge）。
#
# 职责：直连 XiuCalendarState（持久化日历 Bean，随存档）：
#   · watch "day"（注册即回调当前值）→ 刷新「第 N 天」文本
#   · 天数由 CalendarManager 按 WeatherManager 游戏时长推算写入 Bean，
#     本徽章只读展示，不写状态
#
# 红线：watch 按回调哈希去重；回调内不得同步再调 GDSTATE/GdBean。
extends Panel

var _calendar_bean: XiuCalendarState


func _ready() -> void:
	_calendar_bean = XiuCalendarState.ins()
	_calendar_bean.watch("day", _on_day_changed)


func _on_day_changed(value: Variant = null, _metas: Variant = null) -> void:
	var lbl := find_child("DayText", true, false) as Label
	if lbl:
		lbl.text = "第 %d 天" % int(value)
