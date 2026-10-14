# 修仙手册（docsbook）验收 —— 面板装配/菜单联动/富文本加载/tscn rebuild
#
# 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . \
#   -s res://test/check_docsbook.gd
#
# 覆盖：
#   1. docsbook.gml.tscn 直开：TreeMenu rebuild_from_rows（tscn 序列化行恢复）
#   2. RichText src 属性 ready 自动加载（basics.gdoc）
#   3. 菜单选中 → _load_doc 切换正文（ability.gdoc）
#   4. 未登记 id 容错
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[DocsBook] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	var scene: PackedScene = load(
		"res://example/demo/xiuxian/scenes/main/ui/docsbook/docsbook.gml.tscn")
	check(scene != null, "docsbook.gml.tscn 应存在（先跑 regen_gml_tscn）")
	if scene == null:
		quit(2)
		return
	var panel: Control = scene.instantiate()
	root.add_child(panel)
	await process_frame

	var menu: Control = panel.find_child("DocMenu", true, false)
	var view: Control = panel.find_child("DocView", true, false)
	check(menu != null, "DocMenu 应存在")
	check(view != null, "DocView 应存在")
	check(menu != null and menu.get_class() == "GdUITreeMenu", "目录应为 GdUITreeMenu")
	check(view != null and view.get_class() == "GdUIRichText", "正文应为 GdUIRichText")

	# ---- tscn rebuild：行恢复 + 占位符不残留 ----
	if menu:
		check(int(menu.call("get_item_count")) == 2, "目录应有 2 项（实际 %d）" % int(menu.call("get_item_count")))
		for i in menu.get_child_count():
			var c: Node = menu.get_child(i)
			if c is Control and c.has_meta("__menu_row"):
				check(not c.has_meta("__menu_node"), "占位符不应残留")

	# ---- ready 自动加载默认文档 ----
	var bb := String(view.call("get_bbcode")) if view else ""
	check(bb.contains("修行基础知识"), "默认应加载 basics.gdoc（实际前 60 字：%s）" % bb.substr(0, 60))
	check(String(menu.call("get_selected_id")) == "basics", "默认应选中 basics")

	# ---- 菜单切换正文 ----
	panel.call("_on_menu_selected", "ability")
	bb = String(view.call("get_bbcode"))
	check(bb.contains("境界与突破"), "切换 ability 后应加载能力体系章节")
	check(bb.contains("[table=3]"), "能力体系应含突破材料表格（围栏直通）")
	check(bb.contains("[table=2]"), "境界总览 markdown 表格应转 [table=2]")
	check(not bb.contains("| 阶位 |"), "境界总览不得残留管道原文")
	check(bb.contains("[cell][b]阶位[/b][/cell]"), "境界总览表头应加粗")
	check(bb.contains("[img width=560]"), "能力体系应含配图（img 直通）")
	check(not bb.contains("[li]"), "列表不得残留 [li] 字面标签")

	# 未登记 id 容错（不崩溃、不清空正文）
	panel.call("_on_menu_selected", "nope")
	bb = String(view.call("get_bbcode"))
	check(bb.contains("境界与突破"), "未登记 id 不应破坏当前正文")

	# select_item 反向校验
	check(menu.call("select_item", "basics", false), "select_item(basics) 应成功")
	check(String(menu.call("get_selected_id")) == "basics", "选中应回到 basics")

	print("[DocsBook] 全部通过" if ok else "[DocsBook] 存在失败项")
	quit(0 if ok else 2)
