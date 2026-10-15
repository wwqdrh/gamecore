# 挖矿动作光标实机探针：真实主场景 → 开萧宅 → 模拟镐模式 mine 光标 → 截图
# 用法：godot --path . --resolution 1920x1080 -s res://test/probe_mine_cursor.gd
extends SceneTree


func _initialize() -> void:
	_run()


func _run() -> void:
	var tsg: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	tsg.isolate()
	var scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	var main: Node = scene.instantiate()
	root.add_child(main)
	for i in 20:
		await process_frame
	var mgr: Node = main.get_node(^"MapManager")
	var opened: bool = mgr.call("open_map", "xiuxian_xiaozhai", Vector2i(8, 8))
	print("[ProbeMine] open_map=", opened)
	for i in 15:
		await process_frame
	var map: Node = mgr.call("get_current_map")

	# 找一个矿石格
	var layer: Node = map.get_node_or_null("DecoLayer")
	var ore_cell := Vector2i(-1, -1)
	if layer != null:
		for child in layer.get_children():
			if String(child.name).begins_with("Deco_") \
					and child.get_node_or_null("Health") != null:
				ore_cell = child.get_meta("deco_cell")
				break
	print("[ProbeMine] ore_cell=", ore_cell)
	# 玩家挪到矿石旁看效果；相机跟上后停掉玩家帧循环——
	# 空手 _update_dig_cursor 每帧 harvest 过滤会覆盖手动 mine 调用
	var player: Node = main.get_node(^"Player")
	var beside := ore_cell + Vector2i(0, 2)
	if not bool(map.call("is_walkable", beside)):
		beside = ore_cell + Vector2i(2, 0)
	player.set("global_position", map.call("cell_to_center", beside))
	for i in 40:
		await process_frame
	player.set_process(false)
	player.set_physics_process(false)
	# 驱动 mine 光标（等价玩家镐模式的每帧同步）
	map.call("set_cursor_for", ore_cell, "mine")
	for i in 3:
		await process_frame
	var cursor: Node2D = map.get_node_or_null("ActionCursor")
	print("[ProbeMine] cursor=", cursor, " visible=", cursor.is_visible() if cursor else false,
		" pos=", cursor.global_position if cursor else Vector2())
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png("/tmp/probe_mine_cursor.png")
	print("[ProbeMine] screenshot saved")
	quit(0)
