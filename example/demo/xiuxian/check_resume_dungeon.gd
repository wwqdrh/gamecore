# 秘境场景恢复端到端验收 —— 中途退出游戏后重进，应回到退出时的秘境层数与位置
#
# 运行：perl -e 'alarm 300; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_resume_dungeon.gd
#
# 背景：XiuDungeonState（run_seed/floor/status）与 XiuPlayerState（pos_alias+
#       世界坐标）均随档持久化；explore_manager._wire 天然按 running 状态续层
#       开图。本验收钉死全链：
#   A. 探索场景直接恢复：status=running floor=2 实例化 explore/index.tscn
#      → 自动按层种子开出第 2 层（不需要 main 场景参与）
#   B. main 场景自动续玩：main 场景 DungeonManager._wire 检测 running
#      → 自动 change_scene 到探索场景（原「作废进度」改为「续玩」）
#   C. 玩家位置恢复：XiuPlayerState.pos_alias=xiuxian_dungeon 与当前图匹配
#      + 落点格可行走（同 seed 地形复现）→ 原样落位（精确世界坐标）
extends SceneTree

const APP_SCENE := "res://example/demo/xiuxian/index.tscn"
const EXPLORE_SCENE := "res://example/demo/xiuxian/scenes/explore/index.tscn"

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

	# ---- 预置：一次进行中的秘境挑战，已推进到第 2 层 ----
	dungeon.start_run("xiuxian_xiaozhai", Vector2i(10, 8))
	dungeon.advance_floor()  # floor=2
	check(str(dungeon.status) == XiuDungeonState.STATUS_RUNNING and int(dungeon.floor) == 2,
		"预置秘境 running 第 2 层")

	# ---- A. 探索场景直接恢复：running → 自动按层开图（无需 main 参与）----
	var explore: Node = load(EXPLORE_SCENE).instantiate()
	root.add_child(explore)
	if not await _poll(func():
		var mm: Node = explore.get_node_or_null("MapManager")
		return mm != null and mm.get_current_map() != null, 300,
		"explore 直启应自动开出秘境地图"):
		_finish()
		return
	var mm: Node = explore.get_node("MapManager")
	var map: Node = mm.get_current_map()
	check(str(mm.get_current_alias()) == XiuDungeonState.DUNGEON_ALIAS,
		"当前图应为随机秘境场景")
	# 找一个可行走格，记下其中心+格内偏移的世界坐标（第 2 层同 seed 复现，
	# 地形确定 → B 段该格仍可行走）
	var spawn_cell: Vector2i = mm.get_spawn_cell()
	var target := Vector2i(-1, -1)
	for dy in range(0, 6):
		for dx in range(-6, 7):
			var c: Vector2i = spawn_cell + Vector2i(dx, dy)
			if c.x >= 0 and c.y >= 0 and c.x < int(map.get_map_width()) \
					and c.y < int(map.get_map_height()) and map.is_walkable(c):
				target = c
				break
		if target.x >= 0:
			break
	check(target.x >= 0 and map.is_walkable(target),
		"应找到可行走格（spawn=%s 附近）" % spawn_cell)
	var world: Vector2 = map.cell_to_center(target) + Vector2(3, 5)
	print("[Resume] 目标格=%s 世界坐标=%s" % [target, world])
	root.remove_child(explore)
	explore.free()
	await _wait(4)  # call_deferred 注销组件表

	# ---- 位置入档 + 冷读（模拟退出）----
	XiuPlayerState.ins().save_pos(XiuDungeonState.DUNGEON_ALIAS, world)
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("_probe")
	core.set_save_id("_test")
	var pos: Dictionary = XiuPlayerState.ins().load_pos()
	check(not pos.is_empty() and pos["pos"].is_equal_approx(world),
		"秘境内位置应随档冷读")

	# ---- B. 真实场景流：title → main → DungeonManager 检测 running 自动续玩 ----
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
	# main 场景 DungeonManager._wire 检测 running → 自动 change_scene 到探索场景
	if not await _poll(func(): return _scene_id(app) == "xiuxian_explore", 900,
		"main 装配后应自动续玩切到探索场景"):
		_finish()
		return
	await _wait(10)

	# ---- C. 续玩后：第 2 层已开 + 玩家恢复到退出时精确位置 ----
	var mm2: Node = app.get_node("SceneLayer").get_child(0).get_node_or_null("MapManager") \
		if app.get_node_or_null("SceneLayer") != null else null
	# 场景节点可能不是唯一子节点，稳妥起见按 current_scene 直取
	var cur: Node = app.call("get_current_scene")
	if mm2 == null and cur != null:
		mm2 = cur.get_node_or_null("MapManager")
	check(mm2 != null and mm2.get_current_map() != null, "续玩后秘境地图应已打开")
	if mm2 != null and mm2.get_current_map() != null:
		check(str(mm2.get_current_alias()) == XiuDungeonState.DUNGEON_ALIAS,
			"续玩后应仍在秘境场景")
		var map2: Node = mm2.get_current_map()
		check(int(dungeon.floor) == 2 and str(dungeon.status) == XiuDungeonState.STATUS_RUNNING,
			"进度应保持第 2 层 running")
		var player: Node2D = cur.get_node_or_null("Player")
		check(player != null, "探索场景应有 Player")
		if player != null:
			var cell: Vector2i = map2.world_to_cell(player.global_position)
			print("[Resume] 玩家落位=%s（目标世界坐标 %s）" % [player.global_position, world])
			check(player.global_position.is_equal_approx(world),
				"玩家应恢复到退出时精确世界坐标 %s，实际 %s" % [world, player.global_position])
			check(cell == target, "落点格应为 %s，实际 %s" % [target, cell])

	_finish()


func _finish() -> void:
	XiuDungeonState.ins().finish_run()
	print("[Resume] RESULT=%s" % ("ALL GREEN" if ok else "FAILED"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
