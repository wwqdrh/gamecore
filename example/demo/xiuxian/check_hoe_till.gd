# 锄地端到端验收 —— 4 号槽玄铁锄 → till 模式 → GdDigLayer dig 协议
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_hoe_till.gd
#
# 覆盖：
#   A. 静态配置：4 号槽物品/图标/定义表（item.gjson 产物链）
#   B. 装备联动：mainhud.equip=tool_hoe → 玩家 till 模式（不走判定盒 mask=0，
#      挥击路由开启，WeaponHoe 图形装配）
#   B2. 挖地光标：GdDigLayer cursor_texture → DigCursor 精灵，可锄格显示/
#      不可锄格隐藏（所见即所锄）
#   C. 真实挥击锄地：GdMelee.swing → s_swing → 前方一格 dig —— dig_rects 内
#      grass→sand，目标格精度（非整片），s_dug 信号参数正确
#   D. 边界：dig_rects 外四周装饰草不可锄（地形不变）；三块田地均可锄；
#      无 dig 协议的地图静默跳过
#   E. 持久化：dig_save 进场自动恢复锄过的地形（XiuMapState.terrain_grids）
extends SceneTree

const MAP_SCENE := "res://example/demo/xiuxian/map/home.tscn"
const PLAYER_SCRIPT := "res://example/demo/xiuxian/role/player/player.gd"
const EQUIP_SCRIPT := "res://example/demo/xiuxian/scenes/main/ui/mainhud/mainhud_equipbar.gd"

## 桩 MapManager：player._try_till 仅依赖 get_current_map（鸭子类型）
class FakeMgr extends Node:
	var map: Node = null
	func get_current_map() -> Node:
		return map
	func get_current_alias() -> String:
		return "xiuxian_home"
	func try_teleport(_pos: Vector2) -> void:
		pass

var ok := true


func _initialize() -> void:
	# 看门狗：脚本错误中止协程时强制退出（防零输出假死）
	create_timer(90.0).timeout.connect(func() -> void:
		print("[Till] WATCHDOG 超时——测试未完成，强制退出")
		quit(2))
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[Till] OK  ", msg)
	else:
		push_error("[Till] FAIL  " + msg)
		ok = false


func _wait_frames(n: int) -> void:
	for i in n:
		await process_frame


func _state():
	return Engine.get_singleton("GDSTATE")


func _finish() -> void:
	print("[Till] RESULT: ", "ALL GREEN" if ok else "HAS FAILURES")
	quit(0 if ok else 1)


func _run() -> void:
	# 存档隔离：测试全程跑专用档，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var it := XiuItemState.ins()
	it.reset_demo()
	# 清空存档地形网格（前次运行的锄地结果不残留，从初始配置起步）
	XiuMapState.ins().clear_terrain_grids()

	# ---- A. 静态配置 ----
	var slots: Array = load(EQUIP_SCRIPT).SLOT_ITEMS
	_check(slots.size() >= 7 and str(slots[0]) == "" and str(slots[4]) == "tool_hoe",
		"1 号空手槽 + 5 号槽 tool_hoe（%s）" % str(slots))
	var info: Dictionary = it.get_item("tool_hoe")
	_check(str(info.get("name", "")) == "玄铁锄" and str(info.get("type", "")) == "装备",
		"定义表应有玄铁锄（装备类），实际 %s" % str(info))
	_check(str(info.get("icon", "")) == "img_tool_hoe", "icon 应为逻辑图片 id img_tool_hoe")
	_check(XiuIconMap.resolve("img_tool_hoe") == "🔨", "图标注册表应解析为 🔨")

	# ---- B. 装备联动 + C. 真实挥击锄地 ----
	var map: Node = load(MAP_SCENE).instantiate()
	root.add_child(map)
	await _wait_frames(4)

	var player: Node = load(PLAYER_SCRIPT).new()
	root.add_child(player)
	var fake := FakeMgr.new()
	fake.map = map
	root.add_child(fake)
	player.set("map_mgr", fake)
	# 站到 dig_rects(Rect2i(8,5,4,3)) 西侧相邻格，面向右挥击目标 (9,6)
	player.set("global_position", map.call("cell_to_center", Vector2i(8, 6)))
	await _wait_frames(3)  # player/melee ready（hitbox 初始化）

	_state().set_state("mainhud.equip", "tool_hoe")
	await _wait_frames(1)
	_check(str(player.get("attack_mode")) == "till",
		"装备锄头应进入 till 模式，实际 %s" % str(player.get("attack_mode")))
	var melee: Node = player.get_node("Melee")
	_check(int(melee.get("target_mask")) == 0, "锄地模式不走判定盒（mask=0）")
	_check(bool(melee.get("auto_attack_mouse")), "锄地模式挥击路由应开启")
	_check(player.get_node("WeaponMount").get_node_or_null("WeaponHoe") != null,
		"WeaponMount 应装配锄图形（WeaponHoe）")

	_check(map.call("get_terrain_name_at", Vector2i(9, 6)) == "grass", "挥击前 (9,6) 应为 grass")

	# 光标：配置 cursor_texture 后应创建 DigCursor 精灵，可锄格显示/不可锄格隐藏
	var cursor: Node2D = map.get_node_or_null("DigCursor")
	_check(cursor != null, "配置 cursor_texture 后应创建 DigCursor 精灵")
	if cursor != null:
		map.call("set_cursor_cell", Vector2i(9, 6))
		_check(cursor.visible, "可锄格 set_cursor_cell 应显示光标")
		# 双网格根带半格偏移+缩放：世界语义格心必须经 to_local 落位，
		# 全局坐标应精确等于格心（局部坐标直接赋值会 2 倍飘移）
		_check(cursor.global_position == map.call("cell_to_center", Vector2i(9, 6)),
			"光标世界坐标应等于目标格中心（%s vs %s）"
				% [cursor.global_position, map.call("cell_to_center", Vector2i(9, 6))])
		var sprite := cursor as Sprite2D
		var xform: Transform2D = sprite.get_global_transform()
		var fitted_w: float = sprite.texture.get_width() * xform.get_scale().x
		var fitted_h: float = sprite.texture.get_height() * xform.get_scale().y
		_check(absf(maxf(fitted_w, fitted_h) - 32.0) < 1.0,
			"光标世界尺寸应缩放到单格 32px（长边 %.1f px）" % maxf(fitted_w, fitted_h))
		map.call("set_cursor_cell", Vector2i(0, 0))
		_check(not cursor.visible, "dig_rects 外装饰草应隐藏光标")
		map.call("clear_cursor")
		_check(not cursor.visible, "clear_cursor 应隐藏光标")
	var dug: Array = []
	map.connect("s_dug", func(cell, from_t, to_t) -> void:
		dug.append([cell, str(from_t), str(to_t)]))
	var swung: bool = melee.call("swing", Vector2.RIGHT)
	_check(swung, "挥击应被接受（冷却外）")
	await _wait_frames(2)
	_check(str(map.call("get_terrain_name_at", Vector2i(9, 6))) == "sand",
		"挥击方向前方一格应被锄为 sand")
	_check(str(map.call("get_terrain_name_at", Vector2i(8, 6))) == "grass",
		"脚下格不应被误锄（单格精度）")
	_check(dug.size() == 1 and dug[0][0] == Vector2i(9, 6) and dug[0][1] == "grass"
		and dug[0][2] == "sand", "s_dug 应触发一次且参数正确（%s）" % str(dug))

	# 对角扇区：玩家 (8,6) 朝右下 45° 挥 → 右下邻格 (9,7)（8 邻格扇区选择）
	player.set("global_position", map.call("cell_to_center", Vector2i(8, 6)))
	await create_timer(0.5).timeout
	melee.call("swing", Vector2(1, 1).normalized())
	await _wait_frames(2)
	_check(str(map.call("get_terrain_name_at", Vector2i(9, 7))) == "sand",
		"右下 45° 扇区应锄右下邻格 (9,7)")
	_check(str(map.call("get_terrain_name_at", Vector2i(9, 6))) == "sand"
		and str(map.call("get_terrain_name_at", Vector2i(8, 7))) == "grass",
		"对角挥击不应误锄相邻的非扇区格")

	# ---- D. 边界：dig_rects 外四周装饰草不可锄 ----
	player.set("global_position", map.call("cell_to_center", Vector2i(0, 0)))
	await create_timer(0.5).timeout  # 挥击冷却走完
	melee.call("swing", Vector2.RIGHT)
	await _wait_frames(2)
	_check(str(map.call("get_terrain_name_at", Vector2i(1, 0))) == "grass",
		"四周装饰草不可锄（(1,0) 应保持 grass）")
	_check(dug.size() == 2, "rects 外挥击不应触发 s_dug")

	# 第二块田地（dig_rects patch2）同样可锄
	player.set("global_position", map.call("cell_to_center", Vector2i(15, 4)))
	await create_timer(0.5).timeout
	melee.call("swing", Vector2.RIGHT)
	await _wait_frames(2)
	_check(str(map.call("get_terrain_name_at", Vector2i(16, 4))) == "sand",
		"第二块田地内可锄（(16,4) grass→sand）")
	_check(dug.size() == 3, "三次成功锄地应触发三次 s_dug（%d）" % dug.size())

	# 无 dig 协议的地图（null）静默跳过，不报错
	fake.map = null
	await create_timer(0.5).timeout
	melee.call("swing", Vector2.RIGHT)
	await _wait_frames(2)
	_check(true, "无 dig 协议地图挥击不报错（静默跳过）")

	# ---- E. 持久化：dig_save 进场自动恢复锄过的地形 ----
	map.queue_free()
	await _wait_frames(2)
	var map2: Node = load(MAP_SCENE).instantiate()
	root.add_child(map2)
	await _wait_frames(4)  # DigSave deferred restore 需等一帧
	_check(str(map2.call("get_terrain_name_at", Vector2i(9, 6))) == "sand",
		"新实例进场应自动恢复锄过的地块（(9,6) sand）")
	_check(str(map2.call("get_terrain_name_at", Vector2i(8, 6))) == "grass",
		"未锄格应保持初始地形（(8,6) grass）")
	var saved: PackedInt32Array = XiuMapState.ins().load_terrain_grid("xiuxian_home")
	_check(saved.size() == 20 * 14, "Bean 应存有 home 整网格（280 格），实际 %d" % saved.size())
	map2.queue_free()
	await _wait_frames(2)

	player.queue_free()
	fake.queue_free()
	# map 已在 E 节释放（此处不得重复 queue_free——freed 实例调用会中止协程）
	await _wait_frames(2)
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	_finish()
