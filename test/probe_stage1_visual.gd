# M5 引导演出视觉探针：真实渲染器截图 Toast 提示条 / Banner 演出字幅
# 用法：godot --path . --resolution 1920x1080 -s res://test/probe_stage1_visual.gd
extends SceneTree

func _initialize() -> void:
	_run()

func _shot(path: String) -> void:
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png(path)
	print("[ProbeVisual] saved ", path)

func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	for i in 30:
		await process_frame
	var toast: Node = index.find_child("HintToast", true, false)
	var banner: Node = index.find_child("PerfBanner", true, false)
	if toast == null or banner == null:
		printerr("[ProbeVisual] FAIL: 组件缺失")
		quit(1)
		return
	# 1) Toast：教学已自动开播（清掉换两条可控的），淡入完成后截
	toast.call("clear_toasts")
	toast.call("show_toast", "欢迎来到青云坊，少年")
	toast.call("show_toast", "按提示逐步修行，先去见萧老爷")
	await create_timer(0.8).timeout
	_shot("/tmp/stage1_toast.png")
	# 2) Banner：播放中段（暗幕+大字）截图
	banner.call("play_banner", "突破成功", "气感圆融，迈入斗者一段")
	await create_timer(1.4).timeout
	_shot("/tmp/stage1_banner.png")
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0)
