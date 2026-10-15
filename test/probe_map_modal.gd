# 世界舆图实机探针：真实主场景链路 → MapManager 开图 → 打开 MapModal → 截图
# 用法：godot --path . --resolution 1920x1080 -s res://test/probe_map_modal.gd
extends SceneTree

func _initialize() -> void:
	_run()


func _run() -> void:
	# 存档隔离（测试不写玩家活档）
	var tsg: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	tsg.isolate()
	# 真实主场景（MapManager 开图 + mainhud + Modal 全链路）
	var scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	var main: Node = scene.instantiate()
	root.add_child(main)
	for i in 20:
		await process_frame
	# 打开地图 Modal（走统一 UI 管理层，同 mainhud 按钮动作）
	var modal: Node = GdUIManager.find_ui("MapModal")
	print("[ProbeMap] modal=", modal)
	if modal == null:
		print("[ProbeMap] FAIL: MapModal 未注册")
		quit(2)
		return
	modal.call("open")
	for i in 20:
		await process_frame
	var world: Control = modal.find_child("WorldMap", true, false)
	print("[ProbeMap] world=", world,
		" content=", world.call("get_content_size") if world else "n/a")
	# 玩家光点应有格子（主场景已开图、Player 已就绪）
	var panel: Node = modal.find_child("MapWindow", true, false)
	print("[ProbeMap] player_cell=",
		panel.call("_resolve_player_cell", "xiuxian_xiaozhai") if panel else "no panel")
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png("/tmp/probe_map_modal.png")
	print("[ProbeMap] screenshot saved /tmp/probe_map_modal.png")
	tsg.restore()
	quit(0)
