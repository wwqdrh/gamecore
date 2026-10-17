# 端到端验收：重开游戏回到退出时所在地图 + deco 采集冷却跨重启延续
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_resume_map.gd
#
# 背景：用户实机反馈①在青石镇退出、重开却出生在萧宅（主场景永远开 initial_map，
#       存档位置被默认出生点覆写）；②挖掉的矿重开后很快回涨（再生节拍每次
#       重启清零重新计时）。修复：map_restore.gd 按存档 pos_alias 恢复所在地图；
#       deco_harvest 进场按「采集时刻游戏秒」换算已冷却时长并预推进再生时钟。
#
# 覆盖：
#   A. 主场景恢复上次所在地图：pos_alias=xiuxian_xiaozhen → 开青石镇 + 玩家
#      精确落位（真实 index.tscn 全链路）
#   B. 副本 alias 回退：pos_alias=xiuxian_dungeon（不在主场景注册表）→ 默认萧宅
#   C. deco 冷却跨重启：未到期格不铺矿、到期格立即长回且记录弹出（真实
#      xiaozhai.tscn + deco_harvest 恢复链路）
extends SceneTree

const MAIN_SCENE := "res://example/demo/xiuxian/scenes/main/index.tscn"
const MAP_SCENE := "res://example/demo/xiuxian/map/xiaozhai.tscn"
const TOWN_SCENE := "res://example/demo/xiuxian/map/xiaozhen.tscn"
const REGROW_SECS := 120.0  # 与 XiuGameConfigState 默认一致

var ok := true


func _initialize() -> void:
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[ResumeMap] OK  ", msg)
	else:
		push_error("[ResumeMap] FAIL  " + msg)
		ok = false


func _wait_frames(n: int) -> void:
	for i in n:
		await process_frame


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var core: Object = Engine.get_singleton("GDCORE")
	XiuPlayerState.ins().clear_pos()
	XiuMapState.ins().clear_harvested()

	# ---- A. 主场景恢复上次所在地图（青石镇）----
	# 先独立加载 xiaozhen 找一个可行走格，作为「退出时所在位置」
	var town: Node = load(TOWN_SCENE).instantiate()
	root.add_child(town)
	await _wait_frames(4)
	var town_cell := Vector2i(-1, -1)
	var w: int = town.get_map_width()
	var h: int = town.get_map_height()
	for y in h:
		for x in w:
			if town.is_walkable(Vector2i(x, y)):
				town_cell = Vector2i(x, y)
				break
		if town_cell.x >= 0:
			break
	_check(town_cell.x >= 0, "青石镇应存在可行走格")
	var town_world: Vector2 = town.cell_to_center(town_cell)
	town.queue_free()
	await _wait_frames(2)
	if town_cell.x < 0:
		_finish()
		return
	XiuPlayerState.ins().save_pos("xiuxian_xiaozhen", town_world)
	# 预设时间轴（上局已玩到游戏第 1000 秒）——验 CalendarManager 恢复时间基准
	XiuCalendarState.ins().set_game_secs(1000.0)
	core.set_save_id("_probe")   # 切走（写盘）
	core.set_save_id("_test")    # 切回（磁盘重读，模拟重开游戏）

	var index: Node = load(MAIN_SCENE).instantiate()
	root.add_child(index)
	await _wait_frames(30)  # MapRestore 开图 + player._process 驱动接线
	var mm: Node = index.get_node("MapManager")
	var player: Node2D = index.get_node_or_null("Player")
	var cur: String = str(mm.get_current_alias())
	print("[ResumeMap] main opened=%s saved=xiuxian_xiaozhen player=%s" % [cur, player.global_position if player else null])
	_check(cur == "xiuxian_xiaozhen", "主场景应恢复上次所在地图青石镇，实际 %s" % cur)
	if player != null:
		_check(player.global_position.is_equal_approx(town_world),
			"玩家应落位在退出时位置 %s，实际 %s" % [town_world, player.global_position])
	# D. 时间轴恢复：WeatherManager 应从存档 game_secs 接续（不恢复则每局从
	# start_time 重新累计，0.5s 轮询会把 Bean 覆写回 ~0 → 矿冷却误判到期）
	var secs: float = float(XiuCalendarState.ins().game_secs)
	print("[ResumeMap] game_secs after resume=%f (preset 1000)" % secs)
	_check(secs >= 1000.0, "时间轴应从存档值接续不倒退，实际 %f" % secs)
	# 停掉该实例的轮询/存位（保留在树上：主场景 ViewCamera 为全局常驻组件，
	# free 玩家会触发悬垂 panic；日历轮询会覆写 C 段的 game_secs 时间基准）
	index.get_node("Player").set_process(false)
	index.get_node("CalendarManager").set_process(false)

	# ---- B. 副本 alias 回退默认图 ----
	XiuPlayerState.ins().save_pos("xiuxian_dungeon", Vector2(100.0, 100.0))
	var index2: Node = load(MAIN_SCENE).instantiate()
	root.add_child(index2)
	await _wait_frames(30)
	var cur2: String = str(index2.get_node("MapManager").get_current_alias())
	_check(cur2 == "xiuxian_home", "副本 alias 不在主场景注册表应回退默认图（initial_map 家宅），实际 %s" % cur2)
	index2.get_node("Player").set_process(false)
	index2.get_node("CalendarManager").set_process(false)

	# ---- C. deco 冷却跨重启：未到期不回、到期立即回 ----
	# 时间轴定在 1000s；采集间隔默认 120s
	XiuCalendarState.ins().set_game_secs(1000.0)
	var ms := XiuMapState.ins()
	ms.clear_harvested()
	# 收集 xiaozhai 两个真实 deco 格
	var map1: Node = load(MAP_SCENE).instantiate()
	root.add_child(map1)
	await _wait_frames(4)
	var layer1: Node = map1.find_child("DecoLayer", true, false)
	var cells: Array = []
	for child in layer1.get_children():
		if child.has_meta("deco_cell"):
			cells.append(child.get_meta("deco_cell"))
	_check(cells.size() >= 2, "xiaozhai 应铺出至少两个矿石，实际 %d" % cells.size())
	if cells.size() < 2:
		_finish()
		return
	var key_of := func(c: Vector2i) -> int: return (maxi(c.x, 0) << 32) | maxi(c.y, 0)
	var key_cool: int = key_of.call(cells[0])   # 60s 前采集 → 仍在冷却
	var key_gone: int = key_of.call(cells[1])   # 500s 前采集 → 早已到期
	ms.add_harvested(key_cool, 1000.0 - 60.0)
	ms.add_harvested(key_gone, 1000.0 - 500.0)
	core.set_save_id("_probe")
	core.set_save_id("_test")    # 冷读（模拟重开游戏）
	var raw: Dictionary = XiuMapState.ins().get_harvested()
	print("[ResumeMap] harvested after reload: %s" % raw)
	_check(absf(float(raw.get(str(key_cool), 0.0)) - 940.0) < 0.001
		and absf(float(raw.get(str(key_gone), 0.0)) - 500.0) < 0.001,
		"采集时间戳应随档冷读（冷却进度跨重启延续的前提）")

	# 重开 xiaozhai：deco_harvest 应弹出到期记录、注入未到期格。
	# 注：spawn_deco 逐格消耗 RNG 且跳过冷却格不消耗 → 注入会使后续铺设位置
	# 漂移（框架既有语义，生长位置本就不固定），故不按坐标断言到期格有矿，
	# 而是断言 deco 层冷却队列内容（到期格不受铺设压制）。
	var map2: Node = load(MAP_SCENE).instantiate()
	root.add_child(map2)
	await _wait_frames(4)
	var layer2: Node = map2.find_child("DecoLayer", true, false)
	var has_ore := func(c: Vector2i) -> bool:
		for child in layer2.get_children():
			if child.has_meta("deco_cell") and child.get_meta("deco_cell") == c:
				return true
		return false
	var injected: PackedInt64Array = layer2.call("get_harvested_cells")
	var alive2 := 0
	for child in layer2.get_children():
		if child.has_meta("deco_item_id"):
			alive2 += 1
	var raw2: Dictionary = XiuMapState.ins().get_harvested()
	print("[ResumeMap] injected=%s alive=%d harvested=%s" % [injected, alive2, raw2])
	_check(not has_ore.call(cells[0]), "冷却中格子（60s 前）重开不应有矿")
	_check(injected.has(key_cool) and not injected.has(key_gone),
		"deco 层冷却队列应只含未到期格（到期格不再压制铺设）")
	_check(float(raw2.get(str(key_gone), 0.0)) == 0.0, "到期记录应被弹出（值归 0）")
	_check(absf(float(raw2.get(str(key_cool), 0.0)) - 940.0) < 0.001, "未到期记录应保留")
	await _wait_frames(2)

	_finish()


func _finish() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	print("[ResumeMap] RESULT=%s" % ("ALL GREEN" if ok else "FAILED"))
	quit(0 if ok else 1)
