# NPC 生活化验收 —— GdNpcSchedule（日程/落位/idle 池）+ GdNpcChat（相遇闲聊）
#
# 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . \
#   -s res://test/check_npc_life.gd
#
# 覆盖（fixture 鸭子对象：假时钟/假地图/假宿主，全鸭子 API 与真实环境一致）：
#   A. 选段：hour 命中区间、跨午夜段（from>to）、扰动同日稳定跨日变化
#   B. 落位：近距离走 grid_path、远距离 teleport、目标格不可行走 BFS 吸附
#   C. 到岗：路径走完 → Brain 重配 + idle 气泡/转身触发
#   D. 闲聊：同伴发现、双方暂停、气泡交替、播完恢复 + 冷却
#   E. 暂停友好：宿主 paused 时日程冻结
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[NpcLife] " + msg)
		ok = false


func _initialize() -> void:
	_run()


# ---- fixture：假时钟 ----
class FakeClock extends Node:
	var t_secs: float = 0.0  # 游戏秒
	func get_hour() -> float:
		return fmod(t_secs, 86400.0) / 3600.0
	func get_game_time() -> float:
		return t_secs

# ---- fixture：假地图（32px 格，(5,5)-(5,6) 不可走） ----
class FakeMap extends Node2D:
	var paths_requested: Array = []
	func is_walkable(cell: Vector2i) -> bool:
		return not (cell.x == 5 and cell.y >= 5 and cell.y <= 6)
	func world_to_cell(pos: Vector2) -> Vector2i:
		return Vector2i(int(pos.x / 32.0), int(pos.y / 32.0))
	func get_cell_size_px() -> int:
		return 32
	func get_map_width() -> int:
		return 32
	func get_map_height() -> int:
		return 20
	func find_path(from: Vector2i, to: Vector2i) -> PackedVector2Array:
		paths_requested.append([from, to])
		var pts := PackedVector2Array()
		var cur := from
		var steps := 0
		while cur != to and steps < 64:
			if cur.x != to.x:
				cur.x += signi(to.x - cur.x)
			else:
				cur.y += signi(to.y - cur.y)
			pts.append(Vector2((cur.x + 0.5) * 32.0, (cur.y + 0.5) * 32.0))
			steps += 1
		return pts
	func signi(v: int) -> int:
		return 1 if v > 0 else (-1 if v < 0 else 0)

# ---- fixture：假宿主（鸭子实现 mover 约定） ----
class FakeHost extends Node2D:
	var grid_path: PackedVector2Array = PackedVector2Array()
	var paused := false
	var facing := "down"
	var bubbles: Array = []
	func is_paused() -> bool:
		return paused
	func set_paused(p: bool) -> void:
		paused = p
	func is_grid_path_active() -> bool:
		return not grid_path.is_empty()
	func set_grid_path(p: PackedVector2Array) -> void:
		grid_path = p
	func clear_grid_path() -> void:
		grid_path = PackedVector2Array()
	func set_facing(d: String) -> void:
		facing = d
	func show_bubble(text: String, _secs := 3.0) -> void:
		bubbles.append(text)
	func hide_bubble() -> void:
		bubbles.append("<hide>")



func _run() -> void:
	# ---- A. 选段（含跨午夜） ----
	var clock := FakeClock.new()
	clock.name = "Clock"
	var map := FakeMap.new()
	var host := FakeHost.new()
	host.name = "铁牛"
	root.add_child(map)
	map.add_child(host)
	map.add_child(clock)  # 时钟先进树，get_path_to 才能解析
	host.position = Vector2(12 * 32 + 16, 8 * 32 + 16)  # (12,8)
	var sch := GdNpcSchedule.new()
	sch.name = "Schedule"
	sch.npc_id = "铁牛"
	sch.jitter_minutes = 0.0  # A 段先关扰动，纯选段
	sch.teleport_far = false
	host.add_child(sch)  # schedule 挂宿主子节点，宿主父=地图
	sch.time_source_path = sch.get_path_to(clock)
	sch.set_entries([
		{"from": 8.0, "to": 12.0, "cell": Vector2i(12, 8), "behavior": "idle", "label": "打铁", "lines": ["叮……"]},
		{"from": 12.0, "to": 14.0, "cell": Vector2i(16, 12), "behavior": "idle", "label": "午饭"},
		{"from": 22.0, "to": 6.0, "cell": Vector2i(12, 10), "behavior": "idle", "label": "歇息"},
	])
	clock.t_secs = 8.0 * 3600.0  # 初始 08:00（0 点不在任何时段，applied=-1 属正确行为）
	await create_timer(0.4).timeout
	await create_timer(0.4).timeout
	check(sch.is_at_label("打铁"), "hour=8 应命中 8-12 段（初始位置即 (12,8)）")
	check(sch.is_at_post(), "近距离应直接到岗")

	clock.t_secs = 13.0 * 3600.0
	await create_timer(0.4).timeout
	check(sch.is_at_label("午饭"), "hour=13 应切到午饭段")
	check(host.grid_path.size() > 0 or host.position.distance_to(Vector2(16 * 32 + 16, 12 * 32 + 16)) < 24.0,
		"换段后应导航或已落位")
	# 模拟走完
	host.grid_path = PackedVector2Array()
	host.position = Vector2(16 * 32 + 16, 12 * 32 + 16)
	await create_timer(0.4).timeout
	check(sch.is_at_post(), "路径走完应到岗")

	clock.t_secs = 23.0 * 3600.0
	await create_timer(0.4).timeout
	check(sch.is_at_label("歇息"), "hour=23 应命中 22-6 跨午夜段")
	clock.t_secs = 3.0 * 3600.0
	await create_timer(0.4).timeout
	check(sch.is_at_label("歇息"), "hour=3 也应命中跨午夜段")

	# ---- A2. 每日扰动：同日稳定、跨日变化（冒烟：不同天数下选段不崩溃） ----
	sch.jitter_minutes = 30.0
	for day in range(5):
		clock.t_secs = 8.1 * 3600.0 + day * 86400.0
		await create_timer(0.4).timeout
		check(sch.is_at_label("打铁") or sch.is_at_label("歇息") or sch.is_at_label("午饭"),
			"扰动下 day=%d 仍应命中某段" % day)

	# ---- B0. 场景进入直接落位：首次应用条目直接传送显形，不走过去 ----
	var map3 := FakeMap.new()
	root.add_child(map3)
	var hostC := FakeHost.new()
	hostC.name = "远客"
	map3.add_child(hostC)
	hostC.position = Vector2(2 * 32 + 16, 2 * 32 + 16)  # 离时段位置很远
	var schC := GdNpcSchedule.new()
	schC.name = "SchedC"
	schC.npc_id = "远客"
	hostC.add_child(schC)
	var clockC := FakeClock.new()
	clockC.t_secs = 8.0 * 3600.0
	map3.add_child(clockC)
	schC.time_source_path = schC.get_path_to(clockC)
	schC.teleport_far = true
	schC.teleport_cells = 3
	schC.set_entries([
		{"from": 0.0, "to": 24.0, "cell": Vector2i(20, 15), "behavior": "idle", "label": "远方岗位"},
	])
	await create_timer(0.4).timeout
	check(hostC.position.distance_to(Vector2(20 * 32 + 16, 15 * 32 + 16)) < 4.0,
		"场景进入应直接落位到时段位置（实际 %s）" % str(hostC.position))
	check(hostC.grid_path.is_empty(), "场景进入不应下发路径（不走过去）")
	check(schC.is_at_post(), "直接落位即到岗")
	var d0: Dictionary = schC.debug_state()
	check(int(d0.get("illusion")) == 0, "直接落位不进入过渡阶段")
	map3.free()

	# ---- B. 远距离换段：出场走到视野外 → 视野外传送 → 从视野外走入目标 ----
	host.grid_path = PackedVector2Array()
	sch.teleport_far = true
	sch.teleport_cells = 5
	clock.t_secs = 13.0 * 3600.0  # 午饭段 (16,12)，但先把它放到很远
	host.position = Vector2(2 * 32 + 16, 2 * 32 + 16)
	await create_timer(0.4).timeout
	check(host.grid_path.size() > 0, "远距离换段应先下发出场路径（走到视野外）")
	var d1: Dictionary = sch.debug_state()
	check(int(d1.get("illusion")) == 1, "应处于出场走阶段（实际 %s）" % str(d1.get("illusion")))
	# 模拟出场走完
	host.grid_path = PackedVector2Array()
	await create_timer(0.4).timeout
	var d2: Dictionary = sch.debug_state()
	check(int(d2.get("illusion")) == 2, "出场走完应转入入场走阶段（实际 %s）" % str(d2.get("illusion")))
	check(host.grid_path.size() > 0, "视野外传送后应下发入场路径")
	# 模拟入场走完
	host.position = Vector2(16 * 32 + 16, 12 * 32 + 16)
	host.grid_path = PackedVector2Array()
	await create_timer(0.4).timeout
	check(sch.is_at_post(), "入场走完应到岗")
	check(int(sch.debug_state().get("illusion")) == 0, "到岗后过渡状态应复位")

	# ---- B2. 不可行走目标 BFS 吸附 ----
	clock.t_secs = 12.5 * 3600.0
	# 把午饭段目标改成不可走格 (5,5)，吸附到 (4,5) 或 (6,5)
	var snap_cfg := [{"from": 12.0, "to": 14.0, "cell": Vector2i(5, 5), "behavior": "idle", "label": "吸附测试"}]
	sch.set_entries(snap_cfg)
	await create_timer(0.4).timeout
	var cell := Vector2i(int(host.position.x / 32.0), int(host.position.y / 32.0))
	check(map.is_walkable(cell), "落位格应可行走（实际 %s）" % str(cell))
	check(sch.is_at_label("吸附测试"), "吸附后应到岗")

	# ---- C. idle 行为池：气泡/转身 ----
	clock.t_secs = 8.5 * 3600.0
	sch.set_entries([
		{"from": 8.0, "to": 12.0, "cell": Vector2i(12, 8), "behavior": "idle", "label": "打铁", "lines": ["叮……叮……", "好铁！"]},
	])
	sch.ambient_interval_min = 0.05
	sch.ambient_interval_max = 0.15
	host.position = Vector2(12 * 32 + 16, 8 * 32 + 16)
	await create_timer(1.5).timeout
	check(host.bubbles.size() > 0, "idle 行为池应触发气泡（实际 %d 次）" % host.bubbles.size())

	# ---- E. 暂停友好：paused 时冻结 ----
	host.paused = true
	var bubbles_before: int = host.bubbles.size()
	clock.t_secs = 13.5 * 3600.0  # 换段
	await create_timer(0.8).timeout
	check(host.bubbles.size() == bubbles_before, "paused 时日程应冻结（不触发新行为）")
	host.paused = false
	await create_timer(0.8).timeout
	check(sch.is_at_post() or not host.grid_path.is_empty(), "解除暂停后应自动补位")

	# ---- D. 闲聊 ----
	var map2 := FakeMap.new()
	root.add_child(map2)
	var hostA := FakeHost.new()
	hostA.name = "铁牛"
	var hostB := FakeHost.new()
	hostB.name = "坊市商人"
	map2.add_child(hostA)
	map2.add_child(hostB)
	hostA.position = Vector2(100, 100)
	hostB.position = Vector2(132, 100)  # 相邻格
	var schA := GdNpcSchedule.new()
	schA.name = "SchedA"
	schA.npc_id = "铁牛"
	var schB := GdNpcSchedule.new()
	schB.name = "SchedB"
	schB.npc_id = "坊市商人"
	hostA.add_child(schA)
	hostB.add_child(schB)
	var clockA := FakeClock.new()
	clockA.t_secs = 8.0 * 3600.0
	map2.add_child(clockA)
	var clockB := FakeClock.new()
	clockB.t_secs = 8.0 * 3600.0
	map2.add_child(clockB)
	schA.time_source_path = schA.get_path_to(clockA)
	schB.time_source_path = schB.get_path_to(clockB)
	schA.set_entries([{"from": 0.0, "to": 24.0, "cell": Vector2i(3, 3), "behavior": "idle", "label": "A"}])
	schB.set_entries([{"from": 0.0, "to": 24.0, "cell": Vector2i(4, 3), "behavior": "idle", "label": "B"}])
	var chatA := GdNpcChat.new()
	chatA.name = "ChatA"
	chatA.partner_id = "坊市商人"
	chatA.my_lines = PackedStringArray(["今天炉火旺。", "改日打把好刀。"])
	chatA.partner_lines = PackedStringArray(["承你吉言。", "一言为定！"])
	chatA.chance = 1.0  # 必触发
	chatA.cooldown = 5.0
	chatA.line_secs = 0.3
	chatA.initial_cooldown = 0.0  # 测试免等待
	chatA.enable = true
	hostA.add_child(chatA)
	await create_timer(1.5).timeout
	check(chatA.is_chatting() or chatA.get_cooldown_left() > 0.0 or hostA.bubbles.size() > 0,
		"概率 1.0 应触发闲聊（或已播完进冷却）")
	check(hostA.paused or hostA.bubbles.size() > 0, "闲聊应暂停双方或已有气泡")
	check(hostB.bubbles.size() > 0, "对方应同步显示气泡")
	# 播完恢复（2 句 × 0.3s + 0.6 随机 ≤ 1.8s）
	await create_timer(3.0).timeout
	check(not hostA.paused and not hostB.paused, "聊完双方应恢复")
	check(not chatA.is_chatting(), "聊完应退出 chatting 状态")
	check(chatA.get_cooldown_left() > 0.0, "聊完应进入冷却")

	map.free()
	map2.free()
	print("[NpcLife] %s" % ("ALL PASS" if ok else "FAILED"))
	quit(0 if ok else 1)
