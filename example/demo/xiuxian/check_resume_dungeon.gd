# 秘境存档恢复端到端验收 —— 秘境中关闭游戏重开，不自动重返秘境
#
# 运行：perl -e 'alarm 300; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_resume_dungeon.gd
#
# 背景：秘境战斗中不记录玩家位置（player._save_position 跳过秘境图；入场时
#       DungeonManager.start_run 强制落盘「进秘境前」的位置）。重开应用时
#       DungeonManager._wire 检测 running → 作废本次挑战，留在主场景，
#       map_restore 按存档位置恢复进秘境前的地图与落点。
#
# 覆盖：
#   A. 正常入场路径：status=running 时实例化 explore/index.tscn → 自动按层
#      种子开图（start_run → change_scene 后的 explore_manager._wire 链路）
#   B. 秘境中重开：running + 存档位置=进秘境前（青石镇）→ 全场景流
#      title → main 后停留主场景（不自动切探索场景）、挑战作废（status 空）、
#      开图=进秘境前的青石镇、玩家精确落位到存档位置
extends SceneTree

const APP_SCENE := "res://example/demo/xiuxian/index.tscn"
const EXPLORE_SCENE := "res://example/demo/xiuxian/scenes/explore/index.tscn"
const TOWN_SCENE := "res://example/demo/xiuxian/map/xiaozhen.tscn"

var ok := true


func _initialize() -> void:
	# 看门狗：任何环节卡死/脚本错误中止协程时，强制退出并留下输出
	#（否则进程挂到外部 alarm 超时，管道缓冲导致"零输出假死"观感）
	create_timer(180.0).timeout.connect(func() -> void:
		print("[Resume] WATCHDOG 超时——测试未完成，强制退出（可能卡死）")
		quit(2))
	_run()


func check(cond: bool, msg: String) -> void:
	if cond:
		print("[Resume] OK  ", msg)
	else:
		push_error("[Resume] FAIL  " + msg)
		ok = false


func _wait(n: int) -> void:
	for i in n:
		await process_frame


func _poll(cond: Callable, max_frames: int, msg: String) -> bool:
	for i in max_frames:
		if cond.call():
			return true
		await process_frame
	push_error("[Resume] FAIL  " + msg + "（超时 %d 帧）" % max_frames)
	ok = false
	return false


func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var dungeon: XiuDungeonState = XiuDungeonState.ins()
	dungeon.reset_demo()
	XiuPlayerState.ins().clear_pos()

	# ---- A. 正常入场路径：running → explore 场景自动按层开图 ----
	dungeon.start_run("xiuxian_xiaozhai", Vector2i(10, 8))
	dungeon.advance_floor()  # floor=2
	check(str(dungeon.status) == XiuDungeonState.STATUS_RUNNING and int(dungeon.floor) == 2,
		"预置秘境 running 第 2 层")
	var explore: Node = load(EXPLORE_SCENE).instantiate()
	root.add_child(explore)
	if not await _poll(func():
		var mm: Node = explore.get_node_or_null("MapManager")
		return mm != null and mm.get_current_map() != null, 300,
		"explore 直启应自动开出秘境地图"):
		_finish()
		return
	var mm: Node = explore.get_node("MapManager")
	check(str(mm.get_current_alias()) == XiuDungeonState.DUNGEON_ALIAS,
		"当前图应为随机秘境场景")
	root.remove_child(explore)
	explore.free()
	await _wait(4)  # call_deferred 注销组件表

	# ---- B. 秘境中重开：存档位置=进秘境前（青石镇）→ 停留主场景、作废进度 ----
	# 找青石镇一个可行走格，作为「进秘境前的退出位置」
	var town: Node = load(TOWN_SCENE).instantiate()
	root.add_child(town)
	await _wait(4)
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
	check(town_cell.x >= 0, "青石镇应存在可行走格")
	var town_world: Vector2 = town.cell_to_center(town_cell) + Vector2(3, 5)
	town.free()
	await _wait(2)
	if town_cell.x < 0:
		_finish()
		return
	# 模拟上局：在青石镇走到这里 → 点「秘境」入场（start_run 强制落盘当前
	# 位置）→ 秘境中推到第 2 层 → 直接关闭游戏（秘境内不记录位置）
	dungeon.finish_run()  # A 段遗留的 running 复位（start_run 运行中守卫会拒绝）
	dungeon.clear_entry()
	XiuPlayerState.ins().save_pos("xiuxian_xiaozhen", town_world)
	dungeon.start_run("xiuxian_xiaozhen", town_cell)
	dungeon.advance_floor()
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("_probe")  # 切走（写盘）
	core.set_save_id("_test")   # 切回（磁盘重读，模拟重开游戏）
	var pos: Dictionary = XiuPlayerState.ins().load_pos()
	check(not pos.is_empty() and str(pos["alias"]) == "xiuxian_xiaozhen"
		and (pos["pos"] as Vector2).is_equal_approx(town_world),
		"存档位置应为进秘境前的青石镇落点（秘境内不覆写）")
	check(str(dungeon.status) == XiuDungeonState.STATUS_RUNNING,
		"冷读后秘境状态仍为 running（待主场景 wire 作废）")

	var app: Node = load(APP_SCENE).instantiate()
	root.add_child(app)
	await _wait(8)
	if not await _poll(func(): return _scene_id(app) == "xiuxian_title", 300,
		"入口场景应为 title"):
		_finish()
		return
	app.call("change_scene", "xiuxian_main", {}, "", true, false)
	if not await _poll(func(): return _scene_id(app) == "xiuxian_main", 600,
		"应切到 main 场景"):
		_finish()
		return
	# 停留观察：DungeonManager._wire 已处理 running（作废），不得自动切探索场景
	await _wait(300)
	check(_scene_id(app) == "xiuxian_main",
		"重开后应停留主场景，不自动重返秘境（实际 %s）" % _scene_id(app))
	check(str(dungeon.status) != XiuDungeonState.STATUS_RUNNING,
		"未完成秘境应被作废（status 空）")
	check(str(dungeon.from_map) == "" and dungeon.from_cell == Vector2i(-1, -1),
		"入口记录应已清空")

	# ---- C. 恢复进秘境前的地图与落点 ----
	var cur: Node = app.call("get_current_scene")
	var mm2: Node = cur.get_node_or_null("MapManager") if cur != null else null
	check(mm2 != null and mm2.get_current_map() != null, "主场景地图应已打开")
	if mm2 != null and mm2.get_current_map() != null:
		check(str(mm2.get_current_alias()) == "xiuxian_xiaozhen",
			"应恢复进秘境前的青石镇，实际 %s" % mm2.get_current_alias())
		var player: Node2D = cur.get_node_or_null("Player")
		check(player != null, "主场景应有 Player")
		if player != null:
			print("[Resume] 玩家落位=%s（存档位置 %s）" % [player.global_position, town_world])
			check(player.global_position.is_equal_approx(town_world),
				"玩家应恢复到进秘境前位置 %s，实际 %s"
					% [town_world, player.global_position])

	_finish()


func _finish() -> void:
	XiuDungeonState.ins().finish_run()
	print("[Resume] RESULT=%s" % ("ALL GREEN" if ok else "FAILED"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
