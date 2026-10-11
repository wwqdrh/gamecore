# 复现实机「出生在上次位置」恢复链路 —— 真实主场景 index.tscn 全链路
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_pos_restore.gd
#
# 背景：用户实机反馈位置未恢复。探针证明 pos 已落盘（xiuxian_xiaozhai@(10,9)），
#       本脚本复现真实启动链路：预置位置 → 冷读确认 → 实例化真实主场景
#       （Player _process 驱动 _wire → _apply_map(snap=true) → _resolve_spawn_position）
#       → 断言玩家实际落格 == 上次位置。
extends SceneTree

const MAIN_SCENE := "res://example/demo/xiuxian/scenes/main/index.tscn"

var ok := true


func _initialize() -> void:
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[PosRestore] OK  ", msg)
	else:
		push_error("[PosRestore] FAIL  " + msg)
		ok = false


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	# ---- 1. 预置「上次位置」（模拟上一局退出时已保存；世界坐标故意偏离格心，
	#         验证精确像素恢复而非吸附格心）----
	var world := Vector2(10.5 * 32 + 7.0, 9.5 * 32 + 11.0)  # 格 (10,9) 内偏移点
	XiuPlayerState.ins().save_pos("xiuxian_xiaozhai", world)
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("_probe")   # 切走（写盘）
	core.set_save_id("_test")    # 切回（磁盘重读）
	var pos: Dictionary = XiuPlayerState.ins().load_pos()
	print("[PosRestore] pos after reload: %s" % pos)
	_check(not pos.is_empty() and pos["pos"].is_equal_approx(world),
		"预置世界坐标应随档冷读（模拟退出再进）")

	# ---- 2. 实例化真实主场景（与实机同一装配）----
	var scene: Node = load(MAIN_SCENE).instantiate()
	root.add_child(scene)
	# player._wire 由 _process 驱动（等 MapManager ready 开图），多等几帧
	for i in 30:
		await process_frame

	var player: Node2D = scene.get_node_or_null("Player")
	_check(player != null, "主场景应有 Player")
	if player == null:
		_finish()
		return
	var mm: Node = scene.get_node("MapManager")
	var map: Node = mm.get_current_map()
	_check(map != null, "初始地图应已加载")
	if map == null:
		_finish()
		return
	var cell: Vector2i = map.world_to_cell(player.global_position)
	var spawn_cell: Vector2i = mm.get_spawn_cell()
	print("[PosRestore] player cell=%s pos=%s  saved=%s  default_spawn=%s  walkable(10,9)=%s" % [
		cell, player.global_position, world, spawn_cell, map.is_walkable(Vector2i(10, 9))])
	_check(player.global_position.is_equal_approx(world),
		"玩家应原样落位在上次世界坐标 %s，实际 %s" % [world, player.global_position])
	_check(cell == Vector2i(10, 9),
		"落点格应为 (10,9)，实际 %s" % cell)

	_finish()


func _finish() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	print("[PosRestore] RESULT=%s" % ("ALL GREEN" if ok else "FAILED"))
	quit(0 if ok else 1)
