# 传送链路运行时闭环 —— 模拟玩家逐环踩传送点走完整条世界链
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://test/check_teleport_walk.gd
#
# 覆盖（复现并锁死历史 bug）：
#   1. 从萧宅农田出发，每次取「指向未访问图」的传送点格心调 try_teleport
#   2. 传送后玩家实际落格 == 传送点声明的 target_cell（而非地图中部）
#   3. 落点不在任何传送点本体上（否则下帧立即二次弹回——home↔xiaozhai 旧 bug）
#   4. 闭环覆盖全部 6 张图后回到起点
extends SceneTree

const START := "xiuxian_home"
const MAX_STEPS := 24

var ok := true
var main: Node
var mgr: Node


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[TeleportWalk] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _player_cell() -> Vector2i:
	var player: Node = main.get_node(^"Player")
	var map: Node = mgr.get_current_map()
	return map.call("world_to_cell", player.global_position)


func _run() -> void:
	var tsg: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	tsg.isolate()
	main = (load("res://example/demo/xiuxian/scenes/main/index.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for i in 20:
		await process_frame
	mgr = main.get_node(^"MapManager")
	check(mgr != null, "MapManager 应存在")
	if mgr == null:
		quit(1)
		return

	# 从萧宅农田开始（存档隔离后 map_restore 落默认图，显式 open 保证起点）
	check(mgr.call("open_map", START, Vector2i(10, 8)), "应能打开萧宅农田")
	for i in 6:
		await process_frame

	var visited: Array = [START]
	var step := 0
	var bounced := false
	while step < MAX_STEPS:
		step += 1
		var cur: String = mgr.call("get_current_alias")
		# 找一个指向「已访问序列外」或回到起点的传送点（优先未访问图）
		var target_marker: Node = null
		var markers: Array = mgr.call("get_markers")
		var best_score := -1
		for m in markers:
			var to: String = str(m.get("target_alias"))
			var score: int = 0 if visited.has(to) else 1
			if to == START:
				score = 0  # 回起点作为收尾
			if score > best_score:
				best_score = score
				target_marker = m
		if target_marker == null:
			break
		var to_alias: String = str(target_marker.get("target_alias"))
		var to_cell: Vector2i = target_marker.get("target_cell")
		# 格心世界坐标：直接取 marker global_position（ready 已定位格心，
		# 自动含 MapLayer 等父级偏移，与玩家 global_position 同一坐标系）
		var world: Vector2 = (target_marker as Node2D).global_position
		var changed: bool = mgr.call("try_teleport", world)
		check(changed, "第 %d 步踩传送点应触发传送（%s → %s）" % [step, cur, to_alias])
		if not changed:
			break
		for i in 6:
			await process_frame
		check(mgr.call("get_current_alias") == to_alias,
			"第 %d 步后应到达 %s（实际 %s）" % [step, to_alias, mgr.call("get_current_alias")])
		# 落点 = target_cell（而非被甩到地图中部）
		var landed: Vector2i = _player_cell()
		check(landed == to_cell,
			"第 %d 步玩家落格应 = %s（实际 %s）" % [step, str(to_cell), str(landed)])
		# 落点不得压在目标图任何传送点上（防落地即弹回）
		var new_markers: Array = mgr.call("get_markers")
		for m2 in new_markers:
			var mc: Vector2i = m2.get("cell")
			if mc == landed:
				bounced = true
				push_error("[TeleportWalk] 第 %d 步落点 %s 压在传送点上 → 会二次弹回" % [
					step, str(landed)])
		check(not bounced, "落点不得压在传送点本体上")
		if bounced:
			break
		if not visited.has(to_alias):
			visited.append(to_alias)
		# 回到起点 → 闭环完成
		if to_alias == START and visited.size() >= 6:
			break

	check(visited.size() == 6, "闭环应覆盖全部 6 图（实际 %d: %s）" % [visited.size(), str(visited)])
	print("[TeleportWalk] 路径: %s" % str(visited))
	print("[TeleportWalk] 结果: %s" % ("全部通过" if ok else "存在失败项"))
	tsg.restore()
	quit(0 if ok else 1)
