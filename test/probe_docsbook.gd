# 临时探针：打开修仙手册 Modal → 切到「能力体系」→ 截图
# 运行：perl -e 'alarm 120; exec @ARGV' godot --path . --resolution 1920x1080 \
#   -s res://test/probe_docsbook.gd
extends SceneTree

func _initialize() -> void:
	_run()

func _run() -> void:
	var guard: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	guard.isolate()

	var scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/ui/main.gml.tscn")
	var main: Node = scene.instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame

	var modal: Control = main.get_node_or_null(^"DocsModal")
	if modal == null:
		push_error("[Probe] 找不到 DocsModal")
		guard.restore()
		quit(2)
		return
	modal.call("open")
	for i in 12:
		await process_frame

	var view: Control = modal.find_child("DocView", true, false)
	var menu: Control = modal.find_child("DocMenu", true, false)
	print("[Probe] view=", view, " menu=", menu)
	if view:
		print("[Probe] 默认文档 bbcode 长度=", String(view.call("get_bbcode")).length())
	if menu:
		print("[Probe] selected=", String(menu.call("get_selected_id")))
		# 切到「能力体系」：select_item 会发 s_menu_selected，实机接线直达面板
		menu.call("select_item", "ability", true)
		# 探针兜底：若信号未接线（Modal 内 Gml 内联装配时机差异），直调面板
		var panel: Control = modal.find_child("*", true, false)
		var host: Control = null
		for c in [modal, panel]:
			if c and c.has_method("_on_menu_selected"):
				host = c
				break
		if host == null:
			# 逐个找挂脚本的面板节点
			for i in modal.get_child_count():
				var ch: Node = modal.get_child(i)
				if ch is Control and ch.has_method("_on_menu_selected"):
					host = ch
					break
		if host and String(menu.call("get_selected_id")) == "ability":
			var bb_now := String(view.call("get_bbcode")) if view else ""
			if not bb_now.contains("境界与突破"):
				host.call("_on_menu_selected", "ability")
				print("[Probe] 兜底直调 _on_menu_selected(ability)")
	for i in 6:
		await process_frame
	if view:
		var bb := String(view.call("get_bbcode"))
		print("[Probe] 切换后正文含「境界与突破」=", bb.contains("境界与突破"),
			" 含表格=", bb.contains("[table=3]"), " 含配图=", bb.contains("[img"))

	var img: Image = root.get_viewport().get_texture().get_image()
	img.save_png("/tmp/probe_docsbook.png")
	print("[Probe] screenshot saved /tmp/probe_docsbook.png")

	# 滚动到正文底部，截突破材料表格
	if view:
		var sb: ScrollBar = view.get_v_scroll_bar()
		sb.set_value(sb.get_max())
		for i in 4:
			await process_frame
		var img2: Image = root.get_viewport().get_texture().get_image()
		img2.save_png("/tmp/probe_docsbook2.png")
		print("[Probe] screenshot saved /tmp/probe_docsbook2.png")
	guard.restore()
	quit(0)
