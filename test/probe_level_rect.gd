# 临时探针：量取功法面板在真实 Modal 链路下的布局矩形 + 截图
# 运行：perl -e 'alarm 90; exec @ARGV' godot --path . --resolution 1920x1080 \
#   -s res://test/probe_level_rect.gd
extends SceneTree

func _initialize() -> void:
	_run()

func _dump(c: Control, tag: String) -> void:
	print("[Probe] ", tag,
		" gp=", c.get_global_position(),
		" size=", c.get_size(),
		" vis=", c.is_visible_in_tree())

func _run() -> void:
	var guard: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	guard.isolate()

	var scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/ui/main.gml.tscn")
	if scene == null:
		push_error("[Probe] main.gml.tscn 加载失败")
		quit(2)
		return
	var main: Node = scene.instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame

	var modal: Control = main.get_node_or_null(^"LevelModal")
	if modal == null:
		push_error("[Probe] 找不到 LevelModal")
		guard.restore()
		quit(2)
		return
	print("[Probe] viewport size = ", root.get_visible_rect().size)
	modal.call("open")
	for i in 20:
		await process_frame

	var base: String = "ContentArea/LevelPanel/"
	_dump(modal, "LevelModal")
	for c in modal.get_children():
		print("[Probe] modal> ", c.get_name(), " (", c.get_class(), ") is_ctrl=", c is Control)
		for cc in c.get_children():
			print("[Probe] modal>> ", cc.get_name(), " (", cc.get_class(), ") is_ctrl=", cc is Control)
	for n in ["ContentArea", "LevelPanel", "MainMargin", "MainColumn",
			"TopBar", "ExpBar", "TreeScroll", "GongfaTree", "HintLabel"]:
		var c: Control = modal.get_node_or_null(NodePath(base + n))
		if c == null:
			print("[Probe] ", n, " : <不存在>")
			continue
		_dump(c, n)
	var tree_c: Control = modal.get_node_or_null(NodePath(base + "GongfaTree"))
	if tree_c:
		var btns: Array = tree_c.get_children()
		for i in mini(2, btns.size()):
			var b: Control = btns[i]
			_dump(b, "row[%d]" % i)
	# 截图
	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png("/tmp/probe_level.png")
	print("[Probe] screenshot saved /tmp/probe_level.png")
	guard.restore()
	quit(0)
