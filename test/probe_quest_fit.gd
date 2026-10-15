# 任务横幅宽度自适应视觉探针：真实渲染器截图（长文案不再溢出面板）
# 用法：godot --path . --resolution 1280x720 -s res://test/probe_quest_fit.gd
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/main/ui/mainhud/"

func _initialize() -> void:
	_run()

func _shot(path: String) -> void:
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png(path)
	print("[ProbeVisual] saved ", path)

func _run() -> void:
	# gml 重建产物（等效编辑器插件）
	var builder := GdUiBuilder.new()
	var scene: PackedScene = builder.build_scene_file(DIR + "mainhud_quest.gml")
	ResourceSaver.save(scene, DIR + "mainhud_quest.gml.tscn")

	await process_frame
	await process_frame

	var banner: Control = load(DIR + "mainhud_quest.gml.tscn").instantiate()
	root.add_child(banner)
	# 放到画布左中位置（贴近 mainhud 实际摆放）
	banner.position = Vector2(20, 150)
	for i in 4:
		await process_frame
	_shot("/tmp/quest_fit_short.png")

	# 模拟 mainhud.gd 写入长指引文案
	var text: Label = banner.find_child("QuestText", true, false)
	if text:
		text.text = "可接取：日常·每日签到 · 找「执事长老」接取"
	for i in 4:
		await process_frame
	_shot("/tmp/quest_fit_long.png")
	print("[ProbeVisual] panel size: ", banner.find_child("QuestText", true, false).get_parent().get_parent().size)
	quit(0)
