# 游戏日历管理器 —— 挂主场景根 index.tscn（与天气/副本管理器同层，不随换图销毁）
#
# 职责：按 WeatherManager 的游戏内时长推算「第 N 天」，写入 XiuCalendarState
#（mainhud 天数徽章 watch 展示），并在到日时驱动剧情解锁：
#   · 第 TOURNAMENT_DAY 天 → 解锁并自动接取「宗门大比」（sect_tournament），
#     置对话 flag day7_arrived（演武教头台词按此门控）+ toast 提示
#
# 时间流速（存档级游戏配置 XiuGameConfigState.day_length = 现实秒/游戏日）：
#   本管理器是「配置 → 天气」的下发者：_wire 时按配置换算并下发
#   WeatherManager.set_time_scale（= 86400 游戏秒 / day_length 现实秒），
#   并 watch 配置变化实时重下发（天气相位/降雨等全部基于游戏秒自动跟随）。
#
# 时间源：兄弟节点 WeatherManager（同为主场景根直挂，NodePath 引用，
# 与 player.gd 引用 ../MapManager/MapLayer/xxx 同模式）；
# 天数 = floor(get_game_time() / day_length) + 1（day_length 缺失回退 86400）。
# 轮询节流 0.5s（天数变化最频繁也就一天一次，无需每帧）。
extends CanvasLayer

## 组件表注册名（scene_root.get_component("CalendarManager")）
const COMPONENT_NAME := "CalendarManager"
## 天数轮询间隔（秒）
const POLL_INTERVAL := 0.5

var _weather: Node = null
var _state: XiuCalendarState
var _task_state: XiuTaskState
var _config: XiuGameConfigState
var _wired := false
var _acc := 0.0

var _toast: Label
var _toast_tween: Tween


func _ready() -> void:
	layer = XiuUILayers.FULLSCREEN
	_make_toast()
	# 场景装配即接线（含存档时间基准恢复）——严禁留给 0.5s 轮询首跑：
	# 期间任何对 WeatherManager 的 set_total_time（测试推时间/未来业务）都会
	# 被本管理器的恢复动作覆盖，产生难以排查的时序竞态
	_wire()


func _process(delta: float) -> void:
	_acc += delta
	if _acc < POLL_INTERVAL:
		return
	_acc = 0.0
	if _wire():
		_poll_day()


# ------------------------------------------------------------------ 天数

func _wire() -> bool:
	if _wired:
		return true
	_weather = get_node_or_null("../WeatherManager")
	if _weather == null:
		return false
	_wired = true
	_state = XiuCalendarState.ins()
	_task_state = XiuTaskState.ins()
	# 存档级时间流速：按配置下发天气流速（注册即回调当前值，无需手动首刷）；
	# 红线：watch 回调内不得同步再调 GDSTATE/GdBean——这里只调 Weather 节点方法
	_config = XiuGameConfigState.ins()
	_config.watch("day_length", func(_v = null, _m = null): _apply_time_speed.call_deferred())
	_apply_time_speed()
	# 存档恢复：时间基准对齐。game_secs 是绝对游戏秒历史值（含曾经的 start_time
	# 偏移）——先清 start_time 再按绝对值恢复，时间轴跨局/跨场景切换单调连续。
	# 不恢复的话 WeatherManager 每局从 start_time 重新累计：日历天数被静默重置、
	# deco 采集冷却的时间戳基准循环错乱 → 重开后挖掉的矿被误判"早已到期"立即长回
	_weather.set("start_time", 0.0)
	_weather.set_total_time(maxf(float(_state.game_secs), 0.0))
	# 存档恢复：天数直接对齐（老存档升到第 7 天后首次进本版本也补触发）
	_check_day(int(_state.day))
	return true


## 配置 → 天气：下发流速（游戏秒/现实秒 = 一昼夜游戏秒 / 一昼夜现实秒）
func _apply_time_speed() -> void:
	if _weather == null:
		return
	var dl: float = maxf(float(_config.day_length), XiuGameConfigState.MIN_DAY_LENGTH)
	_weather.set_time_scale(XiuGameConfigState.GAME_SECONDS_PER_DAY / dl)


func _poll_day() -> void:
	var t: float = float(_weather.get_game_time())
	var dl_v: Variant = _weather.get("day_length")
	var dl: float = 86400.0 if dl_v == null else float(dl_v)
	_check_day(int(t / maxf(dl, 1.0)) + 1)
	# 游戏时长随档落盘（deco 采集冷却的跨重启时钟基准；同值自动短路）
	_state.set_game_secs(t)


## 天数变化处理：写 Bean + 到日解锁
func _check_day(day: int) -> void:
	if day < 1 or day == int(_state.day):
		return
	_state.set_day(day)
	print("[Calendar] 第 %d 天" % day)
	if day >= XiuCalendarState.TOURNAMENT_DAY:
		_open_tournament()


## 宗门大比到日：解锁并自动接取任务 + 置对话 flag（教头台词门控）+ 提示
func _open_tournament() -> void:
	if _task_state.get_task_status("sect_tournament") != XiuTaskState.STATUS_LOCKED:
		return  # 已解锁/已接取/已完结：幂等
	_task_state.unlock_task("sect_tournament")
	_task_state.accept_task("sect_tournament")  # 自动接取：横幅直接显示 0/2
	XiuDialogState.ins().set_flag("day7_arrived")
	toast("第七天：宗门大比开始！前往青云坊演武堂")
	print("[Calendar] 宗门大比解锁（task_sect_tournament_accepted）")


# ------------------------------------------------------------------ 提示 UI

## 顶部横幅提示（同 DungeonManager 模式：上浮渐隐，重复调用覆盖前一条）
func toast(msg: String) -> void:
	_toast.text = msg
	_toast.modulate = Color(1, 1, 1, 0)
	if _toast_tween != null and _toast_tween.is_valid():
		_toast_tween.kill()
	_toast_tween = create_tween()
	_toast_tween.tween_property(_toast, "modulate:a", 1.0, 0.15)
	_toast_tween.tween_interval(1.6)
	_toast_tween.tween_property(_toast, "modulate:a", 0.0, 0.3)


func _make_toast() -> void:
	_toast = Label.new()
	_toast.name = "Toast"
	_toast.position = Vector2(0, 200)
	_toast.size = Vector2(1920, 44)
	_toast.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_toast.add_theme_font_size_override("font_size", 24)
	_toast.add_theme_color_override("font_color", Color(1.0, 0.88, 0.55))
	_toast.add_theme_color_override("font_outline_color", Color(0.12, 0.06, 0.02))
	_toast.add_theme_constant_override("outline_size", 6)
	_toast.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_toast.modulate = Color(1, 1, 1, 0)
	add_child(_toast)
