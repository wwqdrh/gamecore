# 任务面板「可接取」目录视觉探针：真实渲染器截图（任务目录页签 + HUD 横幅指引）
# 用法：godot --path . --resolution 1280x720 -s res://test/probe_task_catalog.gd
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/main/ui/task/"

func _initialize() -> void:
	_run()

func _shot(path: String) -> void:
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png(path)
	print("[ProbeVisual] saved ", path)

func _run() -> void:
	# 测试档隔离
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	XiuTaskState.ins().reset_demo()

	# gml 重建产物（等效编辑器插件）
	var builder := GdUiBuilder.new()
	for f in ["task_drawer.gml", "task_tabs.gml", "task_list.gml", "task_item.gml"]:
		var scene: PackedScene = builder.build_scene_file(DIR + f)
		ResourceSaver.save(scene, DIR + f + ".tscn")

	await process_frame
	await process_frame

	# 打开任务抽屉（GdUIManager 注册组件）
	var drawer_scene: PackedScene = load(DIR + "task_drawer.gml.tscn")
	var drawer: Node = drawer_scene.instantiate()
	root.add_child(drawer)
	await process_frame
	await process_frame
	drawer.call("open")
	for i in 6:
		await process_frame
	_shot("/tmp/task_catalog.png")
	print("[ProbeVisual] abs path: /tmp/task_catalog.png")

	# 清理
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0)
