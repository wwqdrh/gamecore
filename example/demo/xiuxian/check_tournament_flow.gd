# 宗门大比主线端到端验收 —— 天数推进 / 任务自动解锁 / 演武堂比试 / 领奖
#
# 运行：perl -e 'alarm 240; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_tournament_flow.gd
#
# 覆盖：
#   1. 装配：CalendarManager 挂主场景根；天数徽章 DayText 在 mainhud；
#      sect_tournament 在任务总表且初始 locked
#   2. 天数：初始第 1 天；WeatherManager.set_total_time 推到第 6.5 天 →
#      日历到第 7 天（跨过 TOURNAMENT_DAY）
#   3. 自动解锁：任务 locked → accepted（自动接取）、flag day7_arrived、
#      天数徽章文案「第 7 天」
#   4. 演武堂：town 有演武教头；start_match() 置 match_started + 任务 1/2
#      + 比武弟子登台（town 地图子节点）
#   5. 夺魁：对手死亡 → tournament_won + 任务 2/2 completed
#   6. 领奖：complete_task → submitted，修为/灵石/青云剑到账
#   7. 幂等：重复 start_match 不重复刷怪；重复到日不重复解锁
extends SceneTree

var ok := true

const TID := "sect_tournament"
const DAY_LEN := 86400.0


func fail(msg: String) -> void:
	push_error("[Tournament] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _wait_frames(n: int) -> void:
	for i in n:
		await process_frame


func _poll(cond: Callable, max_frames: int, msg: String) -> bool:
	for i in max_frames:
		if cond.call():
			return true
		await process_frame
	fail(msg + "（超时 %d 帧）" % max_frames)
	return false


func _finish() -> void:
	print("[Tournament] RESULT: %s" % ("PASS" if ok else "FAIL"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)


func _run() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	# 0. 状态复位（跨运行确定性）
	var calendar: XiuCalendarState = XiuCalendarState.ins()
	var tasks: XiuTaskState = XiuTaskState.ins()
	var dialog: XiuDialogState = XiuDialogState.ins()
	var items: XiuItemState = XiuItemState.ins()
	var char_state: XiuCharacterState = XiuCharacterState.ins()
	calendar.reset_demo()
	tasks.reset_demo()
	dialog.reset_demo()
	items.reset_demo()
	char_state.reset_demo()
	var exp0: int = int(char_state.exp)

	# 1. 装配
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	await _wait_frames(4)

	var cal_mgr: Node = index.get_node_or_null("CalendarManager")
	check(cal_mgr != null, "主场景应挂载 CalendarManager")
	check(cal_mgr.get_class() == "CanvasLayer" and cal_mgr.has_method("_poll_day"),
		"CalendarManager 应为日历管理器脚本")
	var weather: Node = index.get_node_or_null("WeatherManager")
	check(weather != null, "主场景应挂载 WeatherManager")
	var day_label: Label = _find_label(index, "DayText")
	check(day_label != null, "mainhud 应有天数徽章 DayText")

	check(str(tasks.get_task(TID).get("name", "")) == "宗门大比", "任务总表应含宗门大比")
	check(tasks.get_task_status(TID) == XiuTaskState.STATUS_LOCKED,
		"开赛前任务应为 locked")
	check(int(calendar.day) == 1, "初始应为第 1 天")

	# 2. 时间推进到第 7 天（6.5 天处 set_total_time，等日历轮询追上）
	weather.set_total_time(6.5 * DAY_LEN)
	if not await _poll(func(): return int(calendar.day) == 7, 600,
		"推到第 6.5 天后日历应到第 7 天"):
		_finish()
		return

	# 3. 自动解锁 + 接取 + flag + 徽章文案（watch 刷新 UI 在 update 之后同帧/次帧，等两帧）
	await _wait_frames(2)
	check(tasks.get_task_status(TID) == XiuTaskState.STATUS_ACCEPTED,
		"第 7 天任务应自动解锁并接取")
	check(dialog.has_flag("day7_arrived"), "应置 flag day7_arrived")
	check(day_label != null and day_label.text == "第 7 天",
		"天数徽章应显示第 7 天，实际「%s」" % (day_label.text if day_label else ""))

	# 4. 演武堂开赛（经 s_map_changed 重定位玩家，多等几帧）
	var map_mgr: Node = index.get_node_or_null("MapManager")
	map_mgr.open_map("xiuxian_town", Vector2i(16, 9))
	await _wait_frames(8)
	var master: Node = map_mgr.get_current_map().get_node_or_null("NpcArenaMaster")
	check(master != null and master.has_method("start_match"),
		"青云坊应有演武教头（NpcArenaMaster）")
	if master == null:
		_finish()
		return
	master.start_match()
	await _wait_frames(4)  # 对手 _process 吸附落位
	check(dialog.has_flag("match_started"), "开赛应置 flag match_started")
	check(int(tasks.get_progress(TID).get("step", 0)) == 1, "开赛后任务应推进一步（1/2）")
	var opp: Node = master._opponent
	check(opp != null and is_instance_valid(opp), "开赛应刷出比武弟子")
	if opp != null:
		check(str(opp.display_name) == "比武弟子·武痴", "对手应为比武弟子·武痴")
		master.start_match()  # 重复开赛：应幂等不新刷
		await _wait_frames(2)
		check(master._opponent == opp, "重复 start_match 不应新刷对手")

	# 5. 夺魁
	if opp != null and is_instance_valid(opp):
		opp.health.kill()
	await _wait_frames(2)
	check(dialog.has_flag("tournament_won"), "对手倒下应置 flag tournament_won")
	check(tasks.get_task_status(TID) == XiuTaskState.STATUS_COMPLETED,
		"夺魁后任务应为 completed（待领奖）")

	# 6. 领奖（对话 @task_complete:sect_tournament 的落点方法）
	check(tasks.complete_task(TID), "complete_task 应成功")
	check(tasks.get_task_status(TID) == XiuTaskState.STATUS_SUBMITTED, "领奖后应为 submitted")
	check(int(char_state.level) > 1 or int(char_state.exp) > exp0,
		"奖励修为应到账（exp 为级内经验，整除升级时归零，以等级兜底）")
	check(int(char_state.spirit_stones) >= 120, "奖励灵石应到账")
	check(int(items.get_count("sword_qingfeng")) == 1, "青云剑应入包")

	# 7. 幂等：再次到日（第 8 天）不重复解锁/接取
	weather.set_total_time(7.5 * DAY_LEN)
	await _poll(func(): return int(calendar.day) == 8, 600, "第 8 天日历应推进")
	check(tasks.get_task_status(TID) == XiuTaskState.STATUS_SUBMITTED,
		"第 8 天不应改变已领奖状态")

	# 还原基线，避免污染其他测试
	calendar.reset_demo()
	tasks.reset_demo()
	dialog.reset_demo()
	items.reset_demo()
	index.queue_free()
	_finish()


## 跨 UI 层找节点（mainhud 在 CanvasLayer 下，find_child 递归）
func _find_label(from: Node, name_: String) -> Label:
	if from == null:
		return null
	if from.name == StringName(name_):
		return from as Label
	for c in from.get_children():
		var r := _find_label(c, name_)
		if r != null:
			return r
	return null
