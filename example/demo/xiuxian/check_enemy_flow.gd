# 敌人战斗系统端到端验收 —— 小怪装配 / 视野追击 / 接触伤害 / 远程射击 / 连通地图
#
# 运行：perl -e 'alarm 240; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_enemy_flow.gd
#
# 覆盖：
#   1. 装配：切到青草岭（map_a），3 只小怪组件齐全
#      （Health / Hurtbox layer3+defense / Hitbox mask2+attack / Brain AI_HUNTER）
#   2. 视野追击：玩家出现在视野内 → Brain 进入 chase，距离持续缩短（网格 BFS 移动）
#   3. 接触伤害：贴近后玩家掉血（attack - 玩家防御）
#   4. 远程攻击：GdShooter.fire_at_point → 子弹命中 → 小怪掉血 = max(1, attack - defense)
#   5. 连续命中至死亡：s_died → 节点回收
#   6. 地图连通：map_a 可行走区域单一连通分量（connected_terrain 挖桥保证）
extends SceneTree

var ok := true


func fail(msg: String) -> void:
	push_error("[Enemy] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _run() -> void:
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "scenes/main/index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	for i in 8:
		await physics_frame

	var mgr: Node = index.get_node_or_null("MapManager")
	var player: Node = index.get_node_or_null("Player")
	check(mgr != null and player != null, "主场景应有 MapManager 与 Player")
	if mgr == null or player == null:
		_finish()
		return
	check(player.health != null and player.shooter != null,
		"玩家应装配 Health 与 Shooter")

	# 1. 切到青草岭（出生格避开传送点——落格判定 try_teleport 会被弹回主城）
	mgr.open_map("xiuxian_map_a", Vector2i(15, 9))
	for i in 10:
		await physics_frame
	var map: Node = mgr.get_current_map()
	check(map != null and str(mgr.get_current_alias()) == "xiuxian_map_a",
		"应成功切到 map_a")
	if map == null:
		_finish()
		return
	var enemies: Array = []
	for child in map.get_children():
		if child.get("max_health") != null and child.get("brain") != null:
			enemies.append(child)
	check(enemies.size() == 3, "map_a 应有 3 只小怪，实际 %d" % enemies.size())
	if enemies.size() != 3:
		_finish()
		return
	for e in enemies:
		check(e.health != null, "%s 应挂 GdHealth" % e.display_name)
		check(int(e.brain.behavior) == 5, "%s 应为 AI_HUNTER" % e.display_name)
		var hurt: Node = e.get_node_or_null("Hurtbox")
		var hit: Node = e.get_node_or_null("Hitbox")
		check(hurt != null and int(hurt.collision_layer) == 4,
			"%s 受击盒应在 layer3(值4)" % e.display_name)
		check(hit != null and int(hit.collision_mask) == 2,
			"%s 攻击盒应指向玩家受击盒 layer2(值2)" % e.display_name)
		check(absf(float(hit.damage) - float(e.attack)) < 0.001,
			"%s 攻击盒伤害应等于 attack" % e.display_name)
		check(absf(float(hurt.defense) - float(e.defense)) < 0.001,
			"%s 受击盒防御应等于 defense" % e.display_name)
	print("[Enemy] setup: enemies=%d wired" % enemies.size())

	# 2. 视野追击：玩家放到史莱姆视野内（间隔约 3 格），应进入 chase 且距离缩短
	var slime: Node = enemies[0]
	var cs: int = map.get_cell_size_px()
	var start_cell: Vector2i = map.world_to_cell(slime.global_position)
	var player_cell := start_cell + Vector2i(3, 0)
	if not map.is_walkable(player_cell):
		player_cell = start_cell - Vector2i(3, 0)
	player.global_position = Vector2((player_cell.x + 0.5) * cs, (player_cell.y + 0.5) * cs)
	var d0: float = player.global_position.distance_to(slime.global_position)
	var saw_chase := false
	for i in 180:
		await physics_frame
		if str(slime.brain.get_ai_state()) == "chase":
			saw_chase = true
			break
	check(saw_chase, "玩家进入视野后小怪应进入 chase 状态")
	# chase 只是状态切换瞬间：再等小怪实际走几格再量距离
	for i in 60:
		await physics_frame
	var d1: float = player.global_position.distance_to(slime.global_position)
	check(d1 < d0, "追击后距离应缩短（%0.f → %0.f）" % [d0, d1])
	print("[Enemy] chase: state=%s dist=%.0f->%.0f"
		% [slime.brain.get_ai_state(), d0, d1])

	# 3. 接触伤害：等小怪贴近到攻击盒重叠 → 玩家掉血（史莱姆 attack 6，玩家防御 0）
	var hp0: float = player.health.get_health()
	var hurt := false
	for i in 240:
		await physics_frame
		if player.health.get_health() < hp0:
			hurt = true
			break
	check(hurt, "小怪贴近后应碰到玩家造成伤害")
	check(player.health.get_health() > 0.0, "玩家不应被一只小怪击倒")
	print("[Enemy] contact: player_hp=%.0f" % player.health.get_health())

	# 4. 远程攻击：朝妖狐发射一发子弹，命中掉血 = max(1, attack - defense)
	var fox: Node = enemies[1]
	var fox_hp0: float = fox.health.get_health()
	var fired: bool = player.shooter.fire_at_point(fox.global_position)
	check(fired, "GdShooter.fire_at_point 应成功发射")
	var fox_hurt := false
	for i in 180:
		await physics_frame
		if not is_instance_valid(fox) or not fox.health.is_alive():
			fox_hurt = true
			break
		if fox.health.get_health() < fox_hp0:
			fox_hurt = true
			break
	check(fox_hurt, "子弹应命中妖狐造成掉血")
	if fox_hurt and is_instance_valid(fox) and fox.health.is_alive():
		var expect: float = maxf(1.0, 25.0 - float(fox.defense))
		check(absf((fox_hp0 - fox.health.get_health()) - expect) < 0.001,
			"妖狐掉血应 = max(1, 玩家攻击 - 防御) = %0.f，实际 %0.f"
				% [expect, fox_hp0 - fox.health.get_health()])
		print("[Enemy] shot: fox_hp=%.0f (-%0.f, defense=%d)"
			% [fox.health.get_health(), fox_hp0 - fox.health.get_health(), int(fox.defense)])

	# 5. 连续命中至死亡：先关 AI 并停住（清掉 mover 残留 grid_path，否则目标
	#    还在走队列，直线子弹会脱靶），逐发命中至血量归零 → s_died → 节点回收
	for e in [slime, fox]:
		e.brain.enable = false
		e.stop()
	for i in 12:
		if not is_instance_valid(fox) or not fox.health.is_alive():
			break
		player.shooter.fire_at_point(fox.global_position)
		for f in 40:
			await physics_frame
			if not is_instance_valid(fox) or not fox.health.is_alive():
				break
	check(not is_instance_valid(fox) or not fox.health.is_alive(),
		"连续命中后妖狐应死亡")
	for i in 40:
		await physics_frame
	check(not is_instance_valid(fox), "死亡后小怪节点应回收")
	print("[Enemy] kill: fox removed")

	# 6. 地图连通：map_a 可行走区域应为单一连通分量（connected_terrain 挖桥）
	_assert_connected(map, "xiuxian_map_a")

	index.queue_free()
	_finish()


## 所有可行走格应属于同一连通分量（connected_terrain 保证）
func _assert_connected(map: Node, alias: String) -> void:
	var w: int = map.get_map_width()
	var h: int = map.get_map_height()
	var total := 0
	var start := Vector2i(-1, -1)
	for y in h:
		for x in w:
			var c := Vector2i(x, y)
			if map.is_walkable(c):
				total += 1
				if start.x < 0:
					start = c
	if total == 0:
		return
	var seen := {start: true}
	var queue: Array = [start]
	var count := 0
	while not queue.is_empty():
		var cur: Vector2i = queue.pop_front()
		count += 1
		for off in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
			var nxt: Vector2i = cur + off
			if nxt.x < 0 or nxt.y < 0 or nxt.x >= w or nxt.y >= h:
				continue
			if not seen.has(nxt) and map.is_walkable(nxt):
				seen[nxt] = true
				queue.append(nxt)
	check(count == total,
		"%s 可行走区域应全连通：可达 %d / 总计 %d" % [alias, count, total])
	print("[Enemy] connectivity: %s walkable=%d reachable=%d" % [alias, total, count])


func _finish() -> void:
	if ok:
		print("[Enemy] RESULT=PASS")
	else:
		print("[Enemy] RESULT=FAIL")
	quit(0 if ok else 1)
