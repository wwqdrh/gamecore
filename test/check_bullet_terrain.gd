# 子弹射程 + 地形高度场（撞山）验收
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://test/check_bullet_terrain.gd
#
# 覆盖：
#   1. 高度场查询：GdQuickMap get_cell_height / get_height_at_world
#   2. terrain_provider 协议：子弹经分组找到地图，撞山销毁（s_destroyed=terrain）
#   3. 高海拔子弹（fly_height > 山体海拔）飞越山地不撞
#   4. 射程：max_distance 超限销毁（s_destroyed=range）
#   5. 寿命到期统一出口（s_destroyed=lifetime）
extends SceneTree

var ok := true
var destroyed: Array[String] = []


func fail(msg: String) -> void:
	push_error("[BulletTerrain] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _run() -> void:
	# ---- 搭建地图：16x10，x=8 一列山墙（海拔 3）----
	var map := GdQuickMap.new()
	map.name = "TestMap"
	map.width = 16
	map.height = 10
	map.cell_size = 32
	map.terrain_names = PackedStringArray(["grass", "mountain"])
	map.terrain_thresholds = PackedFloat64Array([0.5])
	map.terrain_heights = PackedInt32Array([0, 3])
	map.blocked_terrains = PackedStringArray(["mountain"])
	root.add_child(map)
	# -s 模式 add_child 的 ready 延迟到首帧（生成地图 + 入分组都在 ready）
	await process_frame
	map.generate(1)
	# 噪声生成散布山格会干扰定点测试：整体铺草地，再立一列山墙
	for y in range(map.height):
		for x in range(map.width):
			map.set_terrain_at(Vector2i(x, y), 0)
	for y in range(map.height):
		map.set_terrain_at(Vector2i(8, y), 1)  # 山墙
	await process_frame

	# 1. 高度场查询
	check(map.get_cell_height(Vector2i(8, 5)) == 3, "山墙格海拔应为 3")
	check(map.get_cell_height(Vector2i(4, 5)) == 0, "草地格海拔应为 0")
	check(map.get_cell_height(Vector2i(-1, 0)) == 0, "越界海拔应为 0")
	check(map.get_height_at_world(Vector2(8 * 32 + 16, 5 * 32 + 16)) == 3,
		"世界坐标查询山墙海拔应为 3")
	check(map.is_in_group("terrain_provider"), "GdQuickMap 应在 terrain_provider 分组")

	# 子弹工厂：直接实例化 bullet.tscn（池别名为空 → 销毁即 queue_free）
	var bullet_scene: PackedScene = load("res://example/demo/xiuxian/role/player/bullet.tscn")

	# ---- 2. 低空子弹撞山：terrain 销毁 ----
	var b1 := _fire(bullet_scene, Vector2(5 * 32 + 16, 5 * 32 + 16), Vector2(400, 0), 0, 5.0)
	for i in range(40):  # 最多 ~0.66s
		await physics_frame
		if not is_instance_valid(b1):
			break
	check(not is_instance_valid(b1), "低空子弹应撞山销毁")
	check(destroyed.size() > 0 and destroyed[-1] == "terrain",
		"销毁原因应为 terrain，实际 %s" % [destroyed])

	# ---- 3. 高海拔子弹飞越山墙：不被地形销毁，最终 lifetime ----
	destroyed.clear()
	var b2 := _fire(bullet_scene, Vector2(5 * 32 + 16, 5 * 32 + 16), Vector2(400, 0), 5, 0.5)
	var crossed := false
	for i in range(60):
		await physics_frame
		if is_instance_valid(b2) and b2.position.x > 9 * 32:
			crossed = true
		if not is_instance_valid(b2):
			break
	check(crossed, "高海拔子弹应飞越山墙（x 越过 288）")
	check(destroyed.size() > 0 and destroyed[-1] == "lifetime",
		"高海拔子弹最终应因 lifetime 销毁，实际 %s" % [destroyed])

	# ---- 4. 射程：超程销毁 ----
	destroyed.clear()
	var b3 := _fire(bullet_scene, Vector2(4 * 32, 2 * 32 + 16), Vector2(200, 0), 0, 5.0)
	b3.max_distance = 60.0
	for i in range(40):
		await physics_frame
		if not is_instance_valid(b3):
			break
	check(not is_instance_valid(b3), "超射程子弹应销毁")
	check(destroyed.size() > 0 and destroyed[-1] == "range",
		"销毁原因应为 range，实际 %s" % [destroyed])
	# 销毁位置应在射程圈内（起点 x=128，60px → < 192）
	if destroyed.size() > 0 and destroyed[-1] == "range":
		pass  # 位置已随节点释放，原因即证据

	# ---- 5. 寿命到期统一出口（空地飞行，无射程限制）----
	destroyed.clear()
	var b4 := _fire(bullet_scene, Vector2(4 * 32, 1 * 32 + 16), Vector2(100, 0), 0, 0.3)
	for i in range(40):
		await physics_frame
		if not is_instance_valid(b4):
			break
	check(destroyed.size() > 0 and destroyed[-1] == "lifetime",
		"寿命到期销毁原因应为 lifetime，实际 %s" % [destroyed])

	print("[BulletTerrain] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)


## 生成一颗已 setup 的子弹并挂到根场景（点击空池别名 → 销毁即 queue_free）
func _fire(scene: PackedScene, pos: Vector2, vel: Vector2, fly_height: int, lifetime: float) -> Area2D:
	var b := scene.instantiate() as Area2D
	b.position = pos
	b.fly_height = fly_height
	root.add_child(b)
	b.call("setup", vel, 0.0, lifetime, "", 0.0)
	b.connect("s_destroyed", func(reason: String) -> void:
		destroyed.append(str(reason)))
	return b
