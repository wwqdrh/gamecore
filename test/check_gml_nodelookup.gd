# GML name 直取节点验收 —— GdUIManager.get_gml_node（rust 维护 name→节点映射）
#
# 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . \
#   -s res://test/check_gml_nodelookup.gd
#
# 覆盖：
#   1. tscn 直开路径：从树根/从深层子节点按 name 命中（免长路径）
#   2. 缓存 O(1) 命中：二次查找与首查结果一致
#   3. 未登记 name 返回 null（不崩、不误报）
#   4. 嵌套 <Gml> 作用域隔离：外层树查不到内层文件节点，内层自查命中
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[NodeLookup] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	# ---- 1/2. tscn 直开：树根与深层子节点均可按 name 直取 ----
	var scene: PackedScene = load(
		"res://example/demo/xiuxian/scenes/main/ui/docsbook/docsbook.gml.tscn")
	check(scene != null, "docsbook.gml.tscn 应存在")
	if scene == null:
		quit(2)
		return
	var panel: Control = scene.instantiate()
	root.add_child(panel)
	await process_frame

	var menu_ref: Control = panel.find_child("DocMenu", true, false)
	var view_ref: Control = panel.find_child("DocView", true, false)
	check(menu_ref != null and view_ref != null, "参照节点应存在（find_child 基线）")

	# 从树根直取（@onready 典型用法）
	var menu: Control = GdUIManager.get_gml_node(panel, "DocMenu")
	check(menu != null, "树根按 name 应命中 DocMenu")
	check(menu == menu_ref, "命中节点应与 find_child 基线一致")

	# 从深层子节点直取（自动向上定位所属 gml 树根）
	var deep: Control = panel.find_child("MenuScroll", true, false)
	check(deep != null, "深层参照节点应存在")
	var menu2: Control = GdUIManager.get_gml_node(deep, "DocMenu")
	check(menu2 == menu_ref, "从子节点按 name 应同样命中")

	# 缓存 O(1) 命中：二次查找结果一致
	var menu3: Control = GdUIManager.get_gml_node(panel, "DocMenu")
	check(menu3 == menu_ref, "二次查找（缓存路径）应结果一致")

	# ---- 3. 未登记 name → null ----
	var missing: Control = GdUIManager.get_gml_node(panel, "NoSuchNode")
	check(missing == null, "未登记 name 应返回 null")

	# ---- 4. 嵌套 <Gml> 作用域隔离（main.gml.tscn → DocsModal → docsbook）----
	var guard: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	guard.isolate()
	var main_scene: PackedScene = load(
		"res://example/demo/xiuxian/scenes/main/ui/main.gml.tscn")
	var main: Node = main_scene.instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame

	# 外层树查内层文件节点 → null（collect 停在嵌套 __gml_root）
	var outer_hit: Control = GdUIManager.get_gml_node(main, "DocMenu")
	check(outer_hit == null, "外层树不应查到内层 <Gml> 文件的 DocMenu（作用域隔离）")

	# 内层文件自查 → 命中
	var modal: Control = main.get_node_or_null(^"DocsModal")
	check(modal != null, "DocsModal 应存在")
	if modal:
		modal.call("open")
		for i in 12:
			await process_frame
		var inner_menu: Control = modal.find_child("DocMenu", true, false)
		check(inner_menu != null, "内层 DocMenu 应存在（装配完成基线）")
		var scoped: Control = GdUIManager.get_gml_node(inner_menu, "DocView")
		var view_ref2: Control = modal.find_child("DocView", true, false)
		check(scoped == view_ref2, "内层树内按 name 应命中 DocView（就近作用域）")

	guard.restore()
	print("[NodeLookup] 全部通过" if ok else "[NodeLookup] 存在失败项")
	quit(0 if ok else 2)
