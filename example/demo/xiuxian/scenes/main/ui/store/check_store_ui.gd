# 校验脚本：修仙坊市 GML 结构验证
# 运行：godot --headless --path . -s res://example/demo/xiuxian/scenes/main/ui/store/check_store_ui.gd
# 职责：1) 全部 gml 构建校验并重生成同名 tscn（等效编辑器插件产物）
#       2) 实例化主面板验证 <Gml> 组合 / Tab 页内 HBox（左 3x2 网格 + 右推荐卡）/
#          分类过滤 / 条目契约注入（品质角标/倒计时/售罄态）/ 布局比例
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/main/ui/store/"
const GML_FILES := [
	"store_panel.gml", "store_topbar.gml", "store_tabs.gml",
	"store_goods.gml", "store_item.gml", "store_featured.gml", "store_footer.gml",
]


func _initialize() -> void:
	_run()


func _run() -> void:
	var builder := GdUiBuilder.new()
	var ok := true

	# 1. 全部 gml 可构建（语法 / <Gml> 引用 / <script> 数据块校验）+ 重生成 tscn
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
	var panel_scene: PackedScene = load(DIR + "store_panel.gml.tscn")
	if panel_scene == null:
		push_error("[Check] store_panel.gml.tscn 加载失败")
		quit(1)
		return
	var holder := Control.new()
	holder.size = Vector2(1152, 648)
	root.add_child(holder)
	var panel: Control = panel_scene.instantiate()
	holder.add_child(panel)
	for i in 5:
		await process_frame

	# <Gml> 引用的子视图（构建期嫁接，无需脚本挂载）
	for view_name in ["StoreTopBar", "StoreTabs", "StoreFooter"]:
		if panel.find_child(view_name, true, false) == null:
			push_error("[Check] 子视图 %s 不存在" % view_name)
			ok = false
		else:
			print("[Check] view %s OK" % view_name)

	# 4 个页签：每页 TabBody（HBox）内 左网格 + 右推荐卡
	var tabs: TabContainer = panel.find_child("StoreTabs", true, false)
	if tabs == null:
		push_error("[Check] StoreTabs 不存在")
		quit(1)
		return
	var bodies := panel.find_children("TabBody", "HBoxContainer", true, false)
	var grids := panel.find_children("GoodsGrid", "GdUIGrid", true, false)
	var featured_list := panel.find_children("FeaturedPanel", "Panel", true, false)
	print("[Check] tab_bodies=%d grids=%d featured=%d" % [
		bodies.size(), grids.size(), featured_list.size()])
	if bodies.size() != 4 or grids.size() != 4 or featured_list.size() != 4:
		push_error("[Check] 期望 4 页签 × (TabBody + GoodsGrid + FeaturedPanel)")
		ok = false

	# 商品网格：灵药页 6 件（3x2 满格，分类过滤后），模板渲染 + 契约注入
	var grid: GdUIGrid = grids[0]
	var count: int = grid.get_child_count() - 1
	print("[Check] goods items=%d" % count)
	if count != 6:
		push_error("[Check] 灵药页期望 6 件商品（3x2），实际 %d" % count)
		ok = false

	# 3x2 两行布局：条目 0-2 同行（y 相同），条目 3-5 第二行
	for i in 6:
		var it: Control = grid.get_at(i)
		if it == null or not it.visible or it.size.x <= 0:
			push_error("[Check] 条目 %d 不可见或尺寸为 0（网格空格子？）" % i)
			ok = false
	if ok and count == 6:
		var y0: float = grid.get_at(0).position.y
		var same_row := true
		for i in 3:
			if absf(grid.get_at(i).position.y - y0) > 1.0:
				same_row = false
		var second_row := absf(grid.get_at(3).position.y - y0) > 1.0
		print("[Check] grid row0 y=%.0f same_row=%s second_row=%s" % [y0, same_row, second_row])
		if not same_row or not second_row:
			push_error("[Check] 网格不是 3x2 两行布局")
			ok = false
		# 网格填满（条目 expand_fill）：末列右缘≈网格宽，末行下缘≈网格高
		var gw: float = grid.size.x
		var gh: float = grid.size.y
		var r_edge: float = grid.get_at(2).position.x + grid.get_at(2).size.x
		var b_edge: float = grid.get_at(5).position.y + grid.get_at(5).size.y
		print("[Check] grid fill right=%.0f/%.0f bottom=%.0f/%.0f" % [r_edge, gw, b_edge, gh])
		if r_edge < gw - 20.0 or b_edge < gh - 20.0:
			push_error("[Check] 网格未填满可用空间（右侧/下方留空）")
			ok = false

	# 条目 0：千年灵芝（珍品角标 + 价格模板）
	var item0: Control = grid.get_at(0)
	var title0: Label = item0.find_child("GoodsTitle", true, false)
	var quality0: Label = item0.find_child("QualityText", true, false)
	var tag0: Panel = item0.find_child("QualityTag", true, false)
	print("[Check] item0 title=%s quality=%s tag_visible=%s" % [
		title0.text, quality0.text, tag0.visible])
	if title0.text != "千年灵芝" or not quality0.text.contains("珍") or not tag0.visible:
		push_error("[Check] 商品条目模板渲染/品质角标失败")
		ok = false

	# 条目 3：天雷符箓（限时倒计时 + 划线原价）
	var item3: Control = grid.get_at(3)
	var cd3: Label = item3.find_child("CountdownLabel", true, false)
	var old3: Label = item3.find_child("PriceOld", true, false)
	print("[Check] item3 countdown_visible=%s text=%s old=%s" % [
		cd3.visible, cd3.text, old3.text])
	if not cd3.visible or not cd3.text.contains("04:32:10") or old3.text != "¥1200":
		push_error("[Check] 限时商品倒计时/划线原价失败")
		ok = false

	# 条目 5：玄冰雪莲（售罄态：印章 + 按钮禁用 + 置灰）
	var item5: Control = grid.get_at(5)
	var stamp5: Control = item5.find_child("SoldOutStamp", true, false)
	var btn5: Button = item5.find_child("BuyBtn", true, false)
	print("[Check] item5 sold_out stamp=%s btn_disabled=%s modulate_a=%.2f" % [
		stamp5.visible, btn5.disabled, item5.modulate.a])
	if not stamp5.visible or not btn5.disabled or item5.modulate.a >= 1.0:
		push_error("[Check] 售罄态联动失败")
		ok = false

	# 正常条目按钮可用
	var btn0: Button = item0.find_child("BuyBtn", true, false)
	if btn0.disabled:
		push_error("[Check] 正常条目按钮不应禁用")
		ok = false

	# 底部进度条 + 推荐卡极品角标
	var bar: ProgressBar = panel.find_child("PurchaseBar", true, false)
	if bar == null or absf(bar.value - 40.0) > 0.01:
		push_error("[Check] 限购进度条缺失或数值错误")
		ok = false
	else:
		print("[Check] purchase_bar value=%s" % bar.value)

	# 每页签 TabBody 内 网格:推荐卡 ≈ 2:1（stretch_ratio 静态生效）。
	# 非当前页签未显示时 size 为 0（TabContainer 惰性布局），比例只查可见页签
	await process_frame
	var visible_checked := 0
	for b in bodies:
		var body: HBoxContainer = b
		if body.get_child_count() != 3:
			push_error("[Check] TabBody 应有 3 个子节点（网格/间隔/推荐卡）")
			ok = false
			continue
		if not body.is_visible_in_tree():
			continue
		visible_checked += 1
		var g: Control = body.get_child(0)
		var f: Control = body.get_child(2)
		var total := body.size.x
		var r_g := g.size.x / total
		var r_f := f.size.x / total
		print("[Check] visible tabbody grid=%.2f featured=%.2f" % [r_g, r_f])
		if absf(r_g - 0.655) > 0.06 or absf(r_f - 0.32) > 0.05:
			push_error("[Check] 可见 TabBody 比例偏离 2:1")
			ok = false
	if visible_checked != 1:
		push_error("[Check] 应恰好 1 个可见页签，实际 %d" % visible_checked)
		ok = false

	print("[Check] %s" % ("ALL GREEN" if ok else "FAILED"))
	quit(0 if ok else 1)
