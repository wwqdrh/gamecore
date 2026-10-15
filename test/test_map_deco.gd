# suite: map-deco - GdDecoLayer 地图装饰层测试
# 覆盖: 地形匹配随机分布（频率/确定性）、可开采采集协议（受击 3 次 →
#        s_harvested → 按名回调 → 矿石移除）、碰撞层约定、频率 0 不生成、
#        幂等重铺
extends "res://test/test_case.gd"

const DECO_LAYER_BIT := 16  # layer5 可采集物受击盒


func _make_map(w: int = 12, h: int = 12, seed: int = 42) -> GdQuickMap:
	var tree := Engine.get_main_loop() as SceneTree
	var map := GdQuickMap.new()
	map.name = "QM"
	map.width = w
	map.height = h
	map.cell_size = 32
	map.seed_value = seed
	tree.root.add_child(map)
	map.generate(seed)
	# 屏蔽水/山，保证 grass/forest/sand 均可行走（deco 只铺可行走格）
	map.set_blocked_terrain_names(PackedStringArray(["water", "mountain"]))
	return map


func _make_layer(map: GdQuickMap, cfg: Dictionary) -> GdDecoLayer:
	var layer := GdDecoLayer.new()
	layer.name = "DecoLayer"
	layer.deco_terrains = PackedStringArray(cfg.get("terrains", ["grass"]))
	layer.deco_item_ids = PackedStringArray(cfg.get("items", ["ore_test"]))
	layer.deco_frequencies = PackedFloat64Array(cfg.get("freqs", [1.0]))
	layer.deco_mineable = PackedInt32Array(cfg.get("mineable", [0]))
	layer.deco_hits = PackedInt32Array(cfg.get("hits", [3]))
	layer.deco_callbacks = PackedStringArray(cfg.get("callbacks", ["on_harvest"]))
	layer.deco_seed = cfg.get("seed", 7)
	layer.deco_colors = PackedColorArray([Color(0.5, 0.7, 0.9)])
	map.add_child(layer)
	return layer


func _make_harvest_holder(map: GdQuickMap) -> Node:
	var src := GDScript.new()
	src.source_code = "var hits: Array = []\nfunc on_harvest(item_id, node):\n\thits.append(item_id)\n"
	src.reload()
	var holder := Node.new()
	holder.name = "HarvestHolder"
	holder.set_script(src)
	map.add_child(holder)
	return holder


func _deco_nodes(layer: GdDecoLayer) -> Array:
	var out: Array = []
	for child in layer.get_children():
		if String(child.name).begins_with("Deco_"):
			out.append(child)
	return out


func test_spawn_on_matching_terrain() -> void:
	var map := _make_map()
	var layer := _make_layer(map, {"terrains": ["grass"], "freqs": [1.0]})
	await wait_frames(2)  # ready 的 call_deferred spawn_deco
	var nodes := _deco_nodes(layer)
	assert_true(nodes.size() > 0, "频率 1.0 应在草地铺出 deco")
	for n in nodes:
		var cell: Vector2i = map.world_to_cell(n.position)
		assert_eq(String(map.get_terrain_name_at(cell)), "grass",
			"deco 应只落在目标地形上 (pos=%s)" % n.position)
		assert_true(map.is_walkable(cell), "deco 不应落在不可行走格")
	map.free()


func test_spawn_determinism() -> void:
	var a := _make_map(12, 12, 2026)
	var b := _make_map(12, 12, 2026)
	var la := _make_layer(a, {"terrains": ["grass"], "freqs": [0.3], "seed": 9})
	var lb := _make_layer(b, {"terrains": ["grass"], "freqs": [0.3], "seed": 9})
	await wait_frames(2)
	var pos_a: Array = _deco_nodes(la).map(func(n): return n.position)
	var pos_b: Array = _deco_nodes(lb).map(func(n): return n.position)
	assert_eq(pos_a.size(), pos_b.size(), "同种子同配置生成数量应一致")
	assert_eq(pos_a, pos_b, "同种子同配置生成位置应一致")
	a.free()
	b.free()


func test_frequency_zero() -> void:
	var map := _make_map()
	var layer := _make_layer(map, {"terrains": ["grass"], "freqs": [0.0]})
	await wait_frames(2)
	assert_eq(layer.get_spawned_count(), 0, "频率 0 不应生成任何 deco")
	assert_eq(_deco_nodes(layer).size(), 0, "频率 0 无 deco 节点")
	map.free()


func test_harvest_after_three_hits() -> void:
	var map := _make_map(8, 8, 3)
	# 整图铺水、只留一格草地 → 频率 1.0 恰好铺一个矿石（精准定位）
	var cell := Vector2i(4, 4)
	for cy in range(8):
		for cx in range(8):
			map.set_terrain_at(Vector2i(cx, cy), 0)  # water
	map.set_terrain_at(cell, 2)  # grass

	var holder := _make_harvest_holder(map)
	var layer := _make_layer(map, {
		"terrains": ["grass"], "freqs": [1.0],
		"mineable": [1], "hits": [3], "callbacks": ["on_harvest"],
	})
	layer.callback_target_path = NodePath("../HarvestHolder")
	await wait_frames(2)

	var nodes := _deco_nodes(layer)
	assert_eq(nodes.size(), 1, "频率 1.0 应恰好铺一个 deco（单草地格地图）")
	if nodes.is_empty():
		map.free()
		return
	var ore: Node = nodes[0]
	assert_eq(String(ore.get_meta("deco_item_id")), "ore_test", "矿石应携带物品 id meta")
	# 落点应在目标草地格
	var ore_cell: Vector2i = map.world_to_cell(ore.position)
	assert_eq(ore_cell, cell, "矿石应铺在配置的草地格")

	# 碰撞层约定：可采集物受击盒 layer5(16)，mask 0
	var hurt: Node = ore.get_node("Hurtbox")
	assert_eq(int(hurt.get("collision_layer")), DECO_LAYER_BIT,
		"可采集物受击盒应在 layer5(16)")
	assert_eq(int(hurt.get("collision_mask")), 0, "受击盒不应主动扫描")

	# 敲击 1、2 次：不采集
	hurt.call("take_damage", 1.0)
	hurt.call("take_damage", 1.0)
	assert_true(ore.is_inside_tree(), "未满敲击次数不应被移除")
	assert_eq(holder.hits.size(), 0, "未满次数不应触发采集回调")

	# 第 3 次：s_harvested + 按名回调 + 矿石移除
	var fired := [false, ""]
	layer.connect("s_harvested", func(item_id, _node) -> void:
		fired[0] = true
		fired[1] = str(item_id))
	hurt.call("take_damage", 1.0)
	assert_true(fired[0], "敲满 3 次应发出 s_harvested")
	assert_eq(fired[1], "ore_test", "s_harvested 应携带物品 id")
	assert_eq(holder.hits.size(), 1, "采集回调应被按名调用一次")
	assert_eq(String(holder.hits[0]), "ore_test", "回调参数应为物品 id")
	assert_true(ore.is_queued_for_deletion(), "采集完成后矿石应被移除")
	map.free()


func test_respawn_idempotent() -> void:
	var map := _make_map()
	var layer := _make_layer(map, {"terrains": ["grass"], "freqs": [0.5], "seed": 5})
	await wait_frames(2)
	var first := _deco_nodes(layer).size()
	assert_true(first > 0, "首次铺设应有产出")
	layer.spawn_deco()  # 幂等：清旧再铺，同种子数量不变
	await wait_frames(2)  # 旧节点 queue_free 需到帧末移除
	var second := _deco_nodes(layer).size()
	assert_eq(second, first, "重铺应幂等（同种子同数量）")
	map.free()


## 真实挥击全链：GdMelee.swing → 内嵌 GdHitbox 物理扫描 → GdHurtbox.take_damage
## → GdHealth 归零 → s_died → 采集回调（复现实机挖矿路径，区别于直接 take_damage）
func test_harvest_via_melee_swing() -> void:
	var map := _make_map(8, 8, 3)
	# 整图铺水、只留一格草地 → 频率 1.0 恰好一个矿石
	for cy in range(8):
		for cx in range(8):
			map.set_terrain_at(Vector2i(cx, cy), 0)  # water
	var cell := Vector2i(4, 4)
	map.set_terrain_at(cell, 2)  # grass

	var holder := _make_harvest_holder(map)
	var layer := _make_layer(map, {
		"terrains": ["grass"], "freqs": [1.0],
		"mineable": [1], "hits": [3], "callbacks": ["on_harvest"],
	})
	layer.callback_target_path = NodePath("../HarvestHolder")
	await wait_frames(2)
	var nodes := _deco_nodes(layer)
	assert_eq(nodes.size(), 1, "应恰好铺一个矿石")
	if nodes.is_empty():
		map.free()
		return
	var ore: Node = nodes[0]

	# 挥击者：Node2D 宿主（GdMelee 协议要求）+ GdMelee（mask=16 = 矿石受击盒层，
	# 单次伤害 1 = 一次命中一次敲击计数，与 demo 玄铁镐一致）
	var attacker := Node2D.new()
	attacker.name = "Attacker"
	attacker.position = ore.position
	map.add_child(attacker)
	var melee := GdMelee.new()
	melee.name = "Melee"
	melee.target_mask = DECO_LAYER_BIT
	melee.attack_damage = 1.0
	melee.attack_cooldown = 0.05
	melee.swing_window = 0.15
	melee.attack_range = 40.0
	melee.attack_width = 40.0
	attacker.add_child(melee)
	await wait_physics_frames(2)  # melee ready 初始化内嵌 SwingHitbox

	# 真实挥击 3 次（每挥等冷却与窗口走完，物理帧推进扫描）
	for i in 3:
		var ok: bool = melee.call("swing", Vector2.RIGHT)
		assert_true(ok, "第 %d 次挥击应被接受（未进冷却）" % (i + 1))
		await wait_physics_frames(8)

	assert_eq(holder.hits.size(), 1, "真实挥击满 3 次应触发采集回调（melee 全链）")
	assert_eq(String(holder.hits[0]), "ore_test", "回调参数应为物品 id")
	await wait_frames(2)
	assert_false(is_instance_valid(ore) and not ore.is_queued_for_deletion(),
		"采集完成后矿石应被移除")
	map.free()


## 已采集记忆：采集后格子进 harvested 队列 → 重铺（进场）跳过该格
## （挖掉的矿不会重开游戏就全部长回）；get/set_harvested_cells 往返一致
func test_harvest_memory_blocks_respawn() -> void:
	var map := _make_map(8, 8, 3)
	# 整图铺水、只留一格草地 → 恰好一个矿石
	for cy in range(8):
		for cx in range(8):
			map.set_terrain_at(Vector2i(cx, cy), 0)  # water
	var cell := Vector2i(4, 4)
	map.set_terrain_at(cell, 2)  # grass

	var layer := _make_layer(map, {
		"terrains": ["grass"], "freqs": [1.0],
		"mineable": [1], "hits": [3], "callbacks": ["on_harvest"],
	})
	await wait_frames(2)
	assert_eq(_deco_nodes(layer).size(), 1, "应恰好铺一个矿石")

	# 采集（直接敲满 3 次）
	var ore: Node = _deco_nodes(layer)[0]
	var hurt: Node = ore.get_node("Hurtbox")
	for i in 3:
		hurt.call("take_damage", 1.0)

	# 采集后：harvested 队列应记录该格
	var harvested: PackedInt64Array = layer.get_harvested_cells()
	assert_eq(harvested.size(), 1, "采集后 harvested 队列应记 1 条")
	var want_key: int = (cell.x << 32) | cell.y
	assert_eq(int(harvested[0]), want_key, "记录的格子 key 应与采集格一致")

	# 重铺（模拟重开游戏进场）：该格被跳过，不再长出
	layer.spawn_deco()
	await wait_frames(1)
	assert_eq(_deco_nodes(layer).size(), 0, "重铺应跳过已采集格（矿不复活）")

	# 注入往返：新层注入同一 key → 铺设同样跳过（进场恢复持久化）
	var layer2 := _make_layer(map, {
		"terrains": ["grass"], "freqs": [1.0],
		"mineable": [1], "hits": [3], "callbacks": ["on_harvest"],
	})
	layer2.set_harvested_cells(harvested)
	await wait_frames(2)  # layer2 ready 的 call_deferred spawn_deco
	assert_eq(_deco_nodes(layer2).size(), 0, "注入 harvested 后铺设应跳过该格")
	assert_eq(layer2.get_harvested_cells().size(), 1, "注入内容应可读回")
	map.free()


## 定时随机补种：regrow 间隔到点后缺口条目长出新矿（位置不固定），
## s_regrowned 信号发出，harvested 队列弹出（缺口记录守恒）
func test_regrow_after_interval() -> void:
	var map := _make_map(10, 10, 42)
	var layer := _make_layer(map, {
		"terrains": ["grass"], "freqs": [1.0],
		"mineable": [1], "hits": [1], "callbacks": ["on_harvest"],
	})
	layer.deco_regrow_interval = 0.05  # 测试用极短间隔（headless 帧快）
	await wait_frames(2)
	var initial := _deco_nodes(layer).size()
	assert_true(initial >= 2, "多格草地应铺出多个矿石（实际 %d）" % initial)

	# 采集一个 → harvested 1 条（queue_free 到帧末生效，先等一帧再数）
	var ore: Node = _deco_nodes(layer)[0]
	ore.get_node("Hurtbox").call("take_damage", 1.0)
	await wait_frames(2)
	assert_eq(layer.get_harvested_cells().size(), 1, "采集后应记 1 条缺口")
	var missing := _deco_nodes(layer).size()
	assert_eq(missing, initial - 1, "采集后存活数应减一")

	# 等 regrow 到点补种——墙钟时限（红线：无头帧率不节流，帧数≠墙钟时间，
	# 4000 帧固定帧数在 fps 过高时累计 delta 不足 0.05s 会假失败）
	var regrown := [false, ""]
	layer.connect("s_regrowned", func(item_id, _key) -> void:
		regrown[0] = true
		regrown[1] = str(item_id))
	var deadline := Time.get_ticks_msec() + 10000
	while Time.get_ticks_msec() < deadline:
		await wait_frames(1)
		if regrown[0]:
			break
	assert_true(regrown[0], "到间隔应发出 s_regrowned")
	assert_eq(regrown[1], "ore_test", "s_regrowned 应携带物品 id")
	assert_eq(_deco_nodes(layer).size(), initial, "补种后存活数应回到初始数量")
	assert_eq(layer.get_harvested_cells().size(), 0, "补种成功应弹出缺口记录")

	# 补种位置不固定：新矿不应在原采集格（随机挑格；非严格断言但极大概率成立）
	# ——不强行断言位置，只验证新格合法（地形匹配 + 可行走）
	for n in _deco_nodes(layer):
		var c: Vector2i = map.world_to_cell(n.position)
		assert_true(map.is_walkable(c) and String(map.get_terrain_name_at(c)) == "grass",
			"补种节点应落在可行走草地上")
	map.free()
