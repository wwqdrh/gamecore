# 右上角小地图实机探针：真实主场景 → HUD 小地图聚焦当前图 → 截图
# 用法：godot --path . --resolution 1920x1080 -s res://test/probe_hud_minimap.gd
extends SceneTree

func _initialize() -> void:
	_run()


func _run() -> void:
	# 存档隔离（测试不写玩家活档）
	var tsg: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	tsg.isolate()
	# 真实主场景（MapManager 开图 + mainhud + 小地图脚本全链路）
	var scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	var main: Node = scene.instantiate()
	root.add_child(main)
	for i in 30:
		await process_frame

	var view: Control = main.find_child("MiniMapView", true, false)
	var name_label: Label = main.find_child("MapName", true, false)
	var player: Node = main.find_child("Player", true, false)
	print("[ProbeHud] view=", view, " fit=", view.get("fit_current") if view else "n/a")
	print("[ProbeHud] name_label=", name_label.text if name_label else "null",
		" player_cell=", view.call("get_meta") if false else "see-draw")
	# 玩家格子（经 MapManager 当前地图换算，与脚本刷新同路径）
	var mgr: Node = main.get_tree().root.find_child("*", true, false)
	var hud_label := name_label.text if name_label else "?"
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png("/tmp/probe_hud_minimap.png")
	print("[ProbeHud] screenshot saved /tmp/probe_hud_minimap.png map=", hud_label,
		" player=", "ok" if player != null else "missing")

	# 传送跟随：open_map 切图 → s_map_changed → 小地图聚焦新图 + 名字更新
	var scene_root: Node = Engine.get_singleton("GDCORE").get_global_node("default")
	var map_mgr: Node = scene_root.get_component("MapManager")
	if map_mgr != null:
		map_mgr.call("open_map", "xiuxian_town", Vector2i(6, 10))
		for i in 40:
			await process_frame
		print("[ProbeHud] after_tp name=", name_label.text if name_label else "null")
		var img2: Image = root.get_viewport().get_texture().get_image()
		img2.save_png("/tmp/probe_hud_minimap_town.png")
		print("[ProbeHud] town screenshot saved")
	tsg.restore()
	quit(0)
