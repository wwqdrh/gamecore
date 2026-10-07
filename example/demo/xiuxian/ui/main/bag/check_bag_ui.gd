# 校验脚本：修仙储物袋 GML 结构验证
# 运行：godot --headless --path . -s res://example/demo/xiuxian/ui/main/bag/check_bag_ui.gd
# 职责：1) 全部 gml 构建校验并重生成同名 tscn（等效编辑器插件产物）
#       2) 实例化主面板验证 <Gml> 组合 / 8x6 网格 / 条目契约（数量/品质/锁定/选中）
#          / 分类点击联动过滤（含 @pressed gui_input 点击回退）
#          / 物品点选联动详情（选中互斥 + 详情卡填充）/ 三列布局比例
extends SceneTree

const DIR := "res://example/demo/xiuxian/ui/main/bag/"
const GML_FILES := [
	"bag_panel.gml", "bag_topbar.gml", "bag_categories.gml",
	"bag_grid.gml", "bag_item.gml", "bag_detail.gml",
]


func _initialize() -> void:
	_run()


func _run() -> void:
	var builder := GdUiBuilder.new()
	var ok := true

	# 1. 全部 gml 可构建 + 重生成 tscn
	for f in GML_FILES:
		var scene: PackedScene = builder.build_scene_file(DIR + f)
		if scene == null:
			push_error("[Check] %s 构建失败: %s" % [f, builder.last_error()])
			ok = false
			continue
		var err := ResourceSaver.save(scene, DIR + f + ".tscn")
		print("[Check] build %s save_err=%d" % [f, err])
		if err != OK:
			ok = false

	# 2. 实例化主面板验证结构
	var panel_scene: PackedScene = load(DIR + "bag_panel.gml.tscn")
	if panel_scene == null:
		push_error("[Check] bag_panel.gml.tscn 加载失败")
		quit(1)
		return
	var holder := Control.new()
	holder.size = Vector2(1152, 648)
	root.add_child(holder)
	var panel: Control = panel_scene.instantiate()
	holder.add_child(panel)
	for i in 5:
		await process_frame

	# 信号绑定自动连接（无包装层自举）：gml 根 <ui script> 挂 __gml_root 标记，
	# 挂树时由 GDCORE node_added 钩子自动 connect_signals——后续点击/联动断言即验证本机制

	# <Gml> 引用的子视图
	for view_name in ["BagTopBar", "CategoryPanel", "GoodsPanel", "DetailPanel"]:
		if panel.find_child(view_name, true, false) == null:
			push_error("[Check] 子视图 %s 不存在" % view_name)
			ok = false
		else:
			print("[Check] view %s OK" % view_name)

	# 背包网格：8 列 x 48 格
	var grid: GdUIGrid = panel.find_child("GoodsGrid", true, false)
	if grid == null:
		push_error("[Check] GoodsGrid 不存在")
		quit(1)
		return
	var count: int = grid.get_child_count() - 1
	print("[Check] grid columns=%d items=%d" % [grid.columns, count])
	if grid.columns != 8 or count != 48:
		push_error("[Check] 期望 8 列 48 格，实际 columns=%d items=%d" % [grid.columns, count])
		ok = false

	# 8 列两行以上布局：条目 0-7 同行（y 相同），8 换行 + 填满断言
	var y0: float = grid.get_at(0).position.y
	var same_row := true
	for i in 8:
		var it: Control = grid.get_at(i)
		if it == null or not it.visible or it.size.x <= 0:
			push_error("[Check] 条目 %d 不可见或尺寸为 0" % i)
			ok = false
			continue
		if absf(it.position.y - y0) > 1.0:
			same_row = false
	var second_row := absf(grid.get_at(8).position.y - y0) > 1.0
	print("[Check] grid row0 y=%.0f same_row=%s second_row=%s" % [y0, same_row, second_row])
	if not same_row or not second_row:
		push_error("[Check] 网格不是 8 列布局")
		ok = false
	var gw: float = grid.size.x
	var gh: float = grid.size.y
	var r_edge: float = grid.get_at(7).position.x + grid.get_at(7).size.x
	var b_edge: float = grid.get_at(47).position.y + grid.get_at(47).size.y
	print("[Check] grid fill right=%.0f/%.0f bottom=%.0f/%.0f" % [r_edge, gw, b_edge, gh])
	if r_edge < gw - 20.0 or b_edge < gh - 20.0:
		push_error("[Check] 网格未填满可用空间")
		ok = false

	# 条目 0：选中态（金边框）+ 数量角标 x1
	var item0: Control = grid.get_at(0)
	var sel0: Control = item0.find_child("SelectedFrame", true, false)
	var cnt0: Label = item0.find_child("CountLabel", true, false)
	var rare0: Control = item0.find_child("RareFrame", true, false)
	print("[Check] item0 selected=%s count=%s rare=%s" % [
		sel0.visible, cnt0.text, rare0.visible])
	if not sel0.visible or cnt0.text != "x1" or rare0.visible:
		push_error("[Check] 条目 0 选中态/数量角标失败")
		ok = false

	# 条目 1：稀有紫底
	var rare1: Control = grid.get_at(1).find_child("RareFrame", true, false)
	if rare1 == null or not rare1.visible:
		push_error("[Check] 条目 1 稀有紫底失败")
		ok = false
	else:
		print("[Check] item1 rare OK")

	# 条目 47：锁定格（锁标 + 置灰）
	var item47: Control = grid.get_at(47)
	var lock47: Control = item47.find_child("LockIcon", true, false)
	print("[Check] item47 locked=%s modulate_a=%.2f" % [
		lock47.visible, item47.modulate.r])
	if not lock47.visible or item47.modulate.r >= 1.0:
		push_error("[Check] 锁定格联动失败")
		ok = false

	# 分类点击联动过滤：法器(0 件) -> 全部(48) -> 丹药(48)
	var cats: Control = panel.find_child("CategoryPanel", true, false)
	if cats == null:
		push_error("[Check] CategoryPanel 不存在")
		ok = false
	else:
		cats.call("_on_category", cats.find_child("CatTool", true, false))
		var n_tool: int = grid.get_child_count() - 1
		cats.call("_on_category", cats.find_child("CatAll", true, false))
		var n_all: int = grid.get_child_count() - 1
		cats.call("_on_category", cats.find_child("CatPill", true, false))
		var n_pill: int = grid.get_child_count() - 1
		print("[Check] filter tool=%d all=%d pill=%d" % [n_tool, n_all, n_pill])
		if n_tool != 0 or n_all != 48 or n_pill != 48:
			push_error("[Check] 分类联动过滤失败")
			ok = false
		# 选中态互斥：最后点击的 CatPill 应为金色，CatAll 应回白
		var pill: Control = cats.find_child("CatPill", true, false)
		if pill.modulate.r < 0.99 or pill.modulate.g < 0.8:
			push_error("[Check] 分类选中态高亮失败")
			ok = false
		else:
			print("[Check] category highlight OK")

	# @pressed gui_input 点击回退（Panel 无原生 pressed 信号，框架监听
	# gui_input 模拟 Button）：合成左键按下事件发到 CatTool，验证真实点击链路
	var cat_tool: Control = cats.find_child("CatTool", true, false)
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = true
	cat_tool.emit_signal("gui_input", ev)
	var n_fallback: int = grid.get_child_count() - 1
	var ev2 := InputEventMouseButton.new()
	ev2.button_index = MOUSE_BUTTON_LEFT
	ev2.pressed = true
	cats.find_child("CatPill", true, false).emit_signal("gui_input", ev2)
	var n_back: int = grid.get_child_count() - 1
	print("[Check] gui_input fallback tool=%d pill=%d" % [n_fallback, n_back])
	if n_fallback != 0 or n_back != 48:
		push_error("[Check] @pressed gui_input 点击回退失败")
		ok = false

	# 详情卡：初始展示预选中条目（数据 selected: true 的首件 = 聚气丹）
	var detail: Control = panel.find_child("DetailPanel", true, false)
	if detail == null:
		push_error("[Check] DetailPanel 不存在")
		ok = false
	else:
		var d_name: Label = detail.find_child("DetailName", true, false)
		var d_quality: Label = detail.find_child("QualityText", true, false)
		var d_icon: Label = detail.find_child("DetailIcon", true, false)
		print("[Check] detail initial: %s %s %s" % [d_icon.text, d_name.text, d_quality.text])
		if d_name.text != "聚气丹" or d_icon.text != "🏺":
			push_error("[Check] 详情初始展示失败: %s" % d_name.text)
			ok = false

	# 物品点选联动详情：点击条目 2（清心草）-> 金边互斥 + 详情填充
	var item2: Control = grid.get_at(2)
	grid.emit_signal("s_click_item", item2)
	var sel2: Control = item2.find_child("SelectedFrame", true, false)
	var sel0_after: Control = grid.get_at(0).find_child("SelectedFrame", true, false)
	var d_name2: Label = detail.find_child("DetailName", true, false)
	var d_quality2: Label = detail.find_child("QualityText", true, false)
	var d_icon2: Label = detail.find_child("DetailIcon", true, false)
	print("[Check] click item2: sel=%s sel0=%s detail=%s %s %s" % [
		sel2.visible, sel0_after.visible, d_icon2.text, d_name2.text, d_quality2.text])
	if not sel2.visible or sel0_after.visible:
		push_error("[Check] 点选金边互斥失败")
		ok = false
	if d_name2.text != "清心草" or d_icon2.text != "🌿" or d_quality2.text != "普通":
		push_error("[Check] 点选联动详情失败")
		ok = false

	# 锁定格点选：详情显示封印态
	var item_locked: Control = grid.get_at(47)
	grid.emit_signal("s_click_item", item_locked)
	var d_name47: Label = detail.find_child("DetailName", true, false)
	var d_quality47: Label = detail.find_child("QualityText", true, false)
	print("[Check] click item47: %s %s" % [d_name47.text, d_quality47.text])
	if d_name47.text != "封印格" or d_quality47.text != "封印":
		push_error("[Check] 锁定格详情失败")
		ok = false

	# 详情卡按钮
	for btn_name in ["UseBtn", "SellBtn", "ComposeBtn"]:
		if panel.find_child(btn_name, true, false) == null:
			push_error("[Check] 详情按钮 %s 不存在" % btn_name)
			ok = false

	# 三列布局比例 1:4:2.3（stretch_ratio 静态生效）
	await process_frame
	var body_row: HBoxContainer = panel.find_child("BodyRow", true, false)
	if body_row == null:
		push_error("[Check] BodyRow 不存在")
		ok = false
	else:
		var kids := body_row.get_children()
		var total := body_row.size.x
		var r_cat: float = kids[0].size.x / total
		var r_grid: float = kids[2].size.x / total
		var r_detail: float = kids[4].size.x / total
		print("[Check] body cat=%.2f grid=%.2f detail=%.2f" % [r_cat, r_grid, r_detail])
		if absf(r_cat - 0.14) > 0.04 or absf(r_grid - 0.55) > 0.05 \
				or absf(r_detail - 0.315) > 0.05:
			push_error("[Check] 三列比例偏离 1:4:2.3")
			ok = false

	print("[Check] %s" % ("ALL GREEN" if ok else "FAILED"))
	quit(0 if ok else 1)
