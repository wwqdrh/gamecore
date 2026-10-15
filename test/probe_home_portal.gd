# home 地图传送点可见性探针：真实主场景 → 开萧宅农田 → 玩家挪到传送点旁 → 截图
# 用法：godot --path . --resolution 1920x1080 -s res://test/probe_home_portal.gd
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
	# 直接打开萧宅农田，出生点图中央
	var opened: bool = mgr.call("open_map", "xiuxian_home", Vector2i(10, 8))
	print("[ProbeHome] open_map=", opened)
	for i in 10:
		await process_frame
	var map: Node = mgr.call("get_current_map")
	print("[ProbeHome] alias=", mgr.call("get_current_alias"),
		" map_class=", map.get_class(),
		" size=", map.call("get_map_width"), "x", map.call("get_map_height"))
	# 传送点清单
	var markers: Array = mgr.call("get_markers")
	for m in markers:
		var c: Vector2i = m.get("cell")
		print("[ProbeHome] marker=", m.name,
			" cell=", c, " pos=", (m as Node2D).global_position,
			" walkable=", map.call("is_walkable", c),
			" visible=", (m as Node2D).is_visible(),
			" target=", m.get("target_alias"))
	# 玩家挪到传送点旁一格 (17,12)，等相机跟上
	var player: Node = main.get_node(^"Player")
	player.set("global_position", map.call("cell_to_center", Vector2i(17, 12)))
	for i in 40:
		await process_frame
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png("/tmp/probe_home_portal.png")
	print("[ProbeHome] screenshot saved /tmp/probe_home_portal.png")
	# 再截一张全景：相机锁定地图中心
	player.set("global_position", map.call("cell_to_center", Vector2i(10, 8)))
	for i in 40:
		await process_frame
	var img2: Image = root.get_viewport().get_texture().get_image()
	img2.save_png("/tmp/probe_home_full.png")
	print("[ProbeHome] screenshot saved /tmp/probe_home_full.png")
	tsg.restore()
	quit(0)
