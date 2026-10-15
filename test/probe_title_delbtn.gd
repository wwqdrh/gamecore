# 标题页删除存档按钮视觉探针：真实渲染器截图存档弹窗（有档槽 = 继续游戏 + 删除存档）
# 用法：godot --path . --resolution 1280x720 -s res://test/probe_title_delbtn.gd
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/title/ui/"

func _initialize() -> void:
	_run()

func _shot(path: String) -> void:
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png(path)
	print("[ProbeVisual] saved ", path)

func _run() -> void:
	# 环境清场 + 槽二造受控有档样本（level=9 视觉更像真实进度）
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("")
	for id in ["2", "3"]:
		core.delete_save(id)
	var ch := XiuCharacterState.ins()
	core.set_save_id("2")
	ch.update("level", 9, {}, true)

	# gml 重建产物（等效编辑器插件）
	var builder := GdUiBuilder.new()
	for f in ["title_menu.gml", "save_modal.gml"]:
		var scene: PackedScene = builder.build_scene_file(DIR + f)
		ResourceSaver.save(scene, DIR + f + ".tscn")

	await process_frame
	await process_frame

	var title_scene: PackedScene = load("res://example/demo/xiuxian/scenes/title/index.tscn")
	var title_root: Node = title_scene.instantiate()
	root.add_child(title_root)
	await process_frame
	await process_frame

	var start_btn: Button = title_root.find_child("BtnStart", true, false)
	if start_btn:
		start_btn.emit_signal("pressed")
	await process_frame
	await process_frame
	_shot("/tmp/title_save_delete.png")
	print("[ProbeVisual] abs path: /tmp/title_save_delete.png")

	# 清理：切回默认档、删测试槽位
	core.set_save_id("")
	for id in ["2", "3"]:
		core.delete_save(id)
	quit(0)
