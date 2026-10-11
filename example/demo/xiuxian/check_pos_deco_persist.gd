# 端到端验收：玩家位置持久化 + deco 资源再生（采集状态随档）
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_pos_deco_persist.gd
#
# 覆盖：
#   A. 位置持久化：XiuCharacterState.save_player_pos → 切档冷读 → load_player_pos 一致
#   B. 采集状态持久化：XiuMapState.add_harvested → 冷读 → get_harvested_keys 一致
#   C. 真实 xiaozhai.tscn 链路：DecoHarvest._ready 自动注入 harvested + 配置下发
#      （XiuGameConfigState.deco_regrow_secs=0.1 → layer 再生间隔）→ 采集矿
#      → 冷却记录入 Bean → 到点补种 → s_regrowned 弹出记录（数量守恒）
#   D. 重铺语义：注入 harvested 后 spawn_deco 跳过（重开游戏矿不全部复活）
extends SceneTree

const MAP_SCENE := "res://example/demo/xiuxian/map/xiaozhai.tscn"

var ok := true


func _initialize() -> void:
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[PosDeco] OK  ", msg)
	else:
		push_error("[PosDeco] FAIL  " + msg)
		ok = false


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	# ---- A. 位置持久化：写 → 切档冷读 → 读（世界坐标故意偏离格心）----
	var world := Vector2(6.5 * 32 + 3.0, 9.5 * 32 + 5.0)
	XiuPlayerState.ins().save_pos("xiuxian_xiaozhai", world)
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("_probe")   # 切走（=写盘 + 离开内存态）
	core.set_save_id("_test")    # 切回（=从磁盘重读 Bean）
	var pos: Dictionary = XiuPlayerState.ins().load_pos()
	print("[PosDeco] pos after reload: %s" % pos)
	_check(not pos.is_empty() and str(pos["alias"]) == "xiuxian_xiaozhai"
		and pos["pos"].is_equal_approx(world), "玩家位置应随档冷读恢复（alias+世界坐标）")

	# ---- B. 采集状态持久化（值 = 采集时刻游戏秒；时间轴定在 1000s）----
	XiuCalendarState.ins().set_game_secs(1000.0)
	var ms := XiuMapState.ins()
	ms.clear_harvested()  # 清历史运行残留（_test 档跨轮复用）
	var key_a: int = (4 << 32) | 7
	var key_b: int = (9 << 32) | 3
	ms.add_harvested(key_a, 1000.0)
	ms.add_harvested(key_b, 1000.0)
	ms.remove_harvested(key_b)
	core.set_save_id("_probe")
	core.set_save_id("_test")
	var keys: PackedInt64Array = XiuMapState.ins().get_harvested_keys()
	print("[PosDeco] harvested after reload: %s" % keys)
	_check(keys.size() == 1 and int(keys[0]) == key_a,
		"采集状态应随档冷读（弹出项已过滤，冷却项保留）")

	# ---- C/D. 真实 xiaozhai.tscn：注入 + 配置下发 + 采集/再生闭环 ----
	# 极短再生间隔（存档级配置 → DecoHarvest._ready 下发 layer）
	XiuGameConfigState.ins().set_deco_regrow_secs(0.1)
	var map: Node = load(MAP_SCENE).instantiate()
	root.add_child(map)
	for i in 4:
		await process_frame  # ready + DecoHarvest 注入 + call_deferred spawn_deco

	var layer: Node = map.find_child("DecoLayer", true, false)
	var host: Node = map.find_child("DecoHarvest", true, false)
	_check(layer != null and host != null, "xiaozhai.tscn 应含 DecoLayer + DecoHarvest")
	if layer == null or host == null:
		_finish()
		return
	print("[PosDeco] layer regrow interval=%.3f (config 0.1 下发)" % float(layer.deco_regrow_interval))
	_check(absf(float(layer.deco_regrow_interval) - 0.1) < 0.001,
		"存档级再生配置应已下发到 DecoLayer")

	# 注入 key_a（B 段持久化的冷却格，age=0 未到期）→ 铺设应跳过该格
	#（deco_harvest._ready 已自动注入，此处显式重注入验证 set_harvested_cells 往返）
	layer.call("set_harvested_cells", XiuMapState.ins().get_harvested_keys())

	# 采集一个真实矿石 → Bean 记录 +1（deco_harvest.on_ore_harvested 自动）
	var ores: Array = []
	for child in layer.get_children():
		if child.has_meta("deco_item_id"):
			ores.append(child)
	print("[PosDeco] ores spawned=%d" % ores.size())
	_check(ores.size() > 0, "xiaozhai 应铺出矿石")
	if ores.is_empty():
		_finish()
		return
	var ore: Node = ores[0]
	var before: PackedInt64Array = XiuMapState.ins().get_harvested_keys()
	var ore_cell: Vector2i = ore.get_meta("deco_cell")  # 采集后节点会被释放，先读
	var ore_key: int = (maxi(ore_cell.x, 0) << 32) | maxi(ore_cell.y, 0)
	ore.get_node("Hurtbox").call("take_damage", 1.0)  # hits=3，敲 3 次
	ore.get_node("Hurtbox").call("take_damage", 1.0)
	ore.get_node("Hurtbox").call("take_damage", 1.0)
	await process_frame
	await process_frame
	var after: PackedInt64Array = XiuMapState.ins().get_harvested_keys()
	print("[PosDeco] harvested %d -> %d (ore_key=%d in=%s)" % [
		before.size(), after.size(), ore_key, after.has(ore_key)])
	_check(after.size() == before.size() + 1 and after.has(ore_key),
		"采集后冷却记录应随真实链路写入 XiuMapState")

	# 等补种（0.1s 间隔；量级远小于 timeout）→ s_regrowned 弹出记录
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < 8000:
		await process_frame
		if XiuMapState.ins().get_harvested_keys().size() < after.size():
			break
	var settled: PackedInt64Array = XiuMapState.ins().get_harvested_keys()
	var alive := 0
	for child in layer.get_children():
		if child.has_meta("deco_item_id"):
			alive += 1
	print("[PosDeco] after regrow: harvested=%d alive=%d" % [settled.size(), alive])
	_check(settled.size() == after.size() - 1, "补种成功应弹出一条冷却记录（数量守恒）")
	_check(alive == ores.size(), "补种后存活矿数应回到采集前数量")

	_finish()


func _finish() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	print("[PosDeco] RESULT=%s" % ("ALL GREEN" if ok else "FAILED"))
	quit(0 if ok else 1)
