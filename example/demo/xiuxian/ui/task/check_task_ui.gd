# 校验脚本：验证宗门任务面板 Drawer 抽屉改造 + 多 GML 组合 + 任务列表数据加载
# 运行：godot --headless --path . -s res://example/demo/xiuxian/ui/task/check_task_ui.gd
# 职责：1) 全部 task gml 构建校验并重生成同名 tscn（等效编辑器插件产物，
#          防止磁盘上的过期 tscn 缺失 slot 模板导致列表无法建条目）
#       2) 实例化主面板验证 Drawer 抽屉（方向/半屏宽/open 按钮/开关动画/内容嫁接）
#       3) 4 页签 / bean:task_list:tasks 兜底绑定建条目 / filter 分类过滤 /
#          条目信号连通 / Bean 动态插入响应式刷新
extends SceneTree

const DIR := "res://example/demo/xiuxian/ui/task/"
const GML_FILES := [
	"task_panel.gml", "task_topbar.gml", "task_tabs.gml",
	"task_list.gml", "task_item.gml", "task_activity.gml",
]


func _initialize() -> void:
	_run()


func _run() -> void:
	# 1. 全部 gml 可构建（语法 / <Gml> 引用 / <script> 数据块校验）+ 重生成 tscn
	var builder := GdUiBuilder.new()
	var ok := true
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
	var panel_scene: PackedScene = load(DIR + "task_panel.gml.tscn")
	if panel_scene == null:
		push_error("[Check] task_panel.gml.tscn 加载失败")
		quit(1)
		return
	# -s 模式下根窗口尺寸不可控（headless 默认 64x64），显式扩到项目窗口尺寸；
	# 抽屉 ready 时监听了 size_changed，扩窗后 width_ratio 会自动重算
	var holder: Control = Control.new()
	holder.size = Vector2(1152, 648)
	root.size = Vector2(1152, 648)
	root.add_child(holder)
	await process_frame
	var panel: Control = panel_scene.instantiate()
	holder.add_child(panel)
	for i in 5:
		await process_frame

	# 3. Drawer 抽屉：构建期嫁接 + 属性声明 + 序列化存活
	var drawer: Control = panel.find_child("TaskDrawer", true, false)
	if drawer == null:
		push_error("[Check] TaskDrawer 抽屉不存在")
		quit(1)
		return
	print("[Check] drawer direction=%s width_ratio=%s slide_width=%s" % [
		drawer.get("direction"), drawer.get("width_ratio"), drawer.get("slide_width")])
	if int(drawer.get("direction")) != 1:
		push_error("[Check] 抽屉方向应为 left(1)")
		ok = false
	if absf(float(drawer.get("width_ratio")) - 0.5) > 0.001:
		push_error("[Check] slide_width=\"50%%\" 未换算为 width_ratio=0.5（tscn 序列化失效？）")
		ok = false
	# 打开前隐藏；内容区已嫁接 TaskTabs（构建期 add_content_child）
	if drawer.is_visible_in_tree():
		push_error("[Check] 抽屉初始应为隐藏")
		ok = false
	var tabs_in_drawer: Control = drawer.find_child("TaskTabs", true, false)
	if tabs_in_drawer == null:
		push_error("[Check] 抽屉内容区未嫁接 TaskTabs")
		ok = false

	# 4. 打开按钮内部动作（@pressed="show:TaskDrawer" → Drawer.open）
	var open_btn: Button = panel.find_child("OpenTasksBtn", true, false)
	if open_btn == null:
		push_error("[Check] OpenTasksBtn 打开按钮不存在")
		ok = false
	else:
		var conns: int = open_btn.pressed.get_connections().size()
		print("[Check] open btn connections=%d" % conns)
		if conns == 0:
			push_error("[Check] 打开按钮信号未连接到抽屉")
			ok = false

	# 5. 打开抽屉（动画 0.25s），验证锚点比例宽度
	# 新实现：width_ratio>0 时宽度由锚点表达（DIR_LEFT: anchor_right=0.5），
	# 序列化进 tscn、随父级尺寸自适应，无需像素换算——断言锚点即可
	var dpanel_early: Control = drawer.find_child("DrawerPanel", true, false)
	var a_right: float = dpanel_early.anchor_right if dpanel_early else -1.0
	print("[Check] drawer panel anchor_right=%.3f (expect 0.5)" % a_right)
	if absf(a_right - 0.5) > 0.001:
		push_error("[Check] DrawerPanel.anchor_right 应为 0.5（width_ratio 锚点化）")
		ok = false
	drawer.set("slide_width", 576)
	drawer.call("update_layout")
	drawer.call("open")
	await create_timer(0.5).timeout
	print("[Check] drawer open visible=%s is_open=%s" % [
		drawer.is_visible_in_tree(), drawer.call("is_drawer_open")])
	if not drawer.is_visible_in_tree() or not drawer.call("is_drawer_open"):
		push_error("[Check] 抽屉打开失败")
		ok = false
	var dpanel: Control = drawer.find_child("DrawerPanel", true, false)
	if dpanel:
		print("[Check] drawer panel width=%.1f expect=576.0" % dpanel.size.x)
		if absf(dpanel.size.x - 576.0) > 2.0:
			push_error("[Check] 抽屉面板宽度应为 576（半屏）")
			ok = false
	else:
		push_error("[Check] DrawerPanel 不存在")
		ok = false

	# 5b. 标题栏关闭按钮回调（tscn 直开场景由 ready 复用路径重连）
	var tlabel: Label = drawer.find_children("*", "Label", true, false).front()
	if tlabel:
		var close_btn: Button = null
		for c in tlabel.get_parent().get_children():
			if c is Button:
				close_btn = c
				break
		if close_btn:
			var cconn: int = close_btn.pressed.get_connections().size()
			print("[Check] close btn connections=%d" % cconn)
			if cconn == 0:
				push_error("[Check] 关闭按钮信号未连接")
				ok = false
			close_btn.emit_signal("pressed")
			await create_timer(0.4).timeout
			print("[Check] drawer after close-btn is_open=%s" % drawer.call("is_drawer_open"))
			if drawer.call("is_drawer_open"):
				push_error("[Check] 点击关闭按钮未收起抽屉")
				ok = false
			drawer.call("open")
			await create_timer(0.4).timeout
		else:
			push_error("[Check] 标题栏关闭按钮不存在")
			ok = false
	else:
		push_error("[Check] 抽屉标题 Label 不存在")
		ok = false

	# 5c. 遮罩点击关闭：向 Overlay 注入左键按下事件
	var overlay: ColorRect = drawer.find_child("Overlay", true, false)
	if overlay:
		var ev: InputEventMouseButton = InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = true
		overlay.emit_signal("gui_input", ev)
		await create_timer(0.4).timeout
		print("[Check] drawer after overlay-click is_open=%s" % drawer.call("is_drawer_open"))
		if drawer.call("is_drawer_open"):
			push_error("[Check] 点击遮罩未收起抽屉")
			ok = false
		drawer.call("open")
		await create_timer(0.4).timeout
	else:
		push_error("[Check] Overlay 不存在")
		ok = false

	# 6. <Gml> 引用的子视图直接成为骨架子节点（构建期嫁接，无需脚本挂载）
	for view_name in ["TopBar", "TaskTabs", "ActivityRoot"]:
		var view: Control = panel.find_child(view_name, true, false)
		if view == null:
			push_error("[Check] <Gml> 引用子视图 %s 不存在" % view_name)
			ok = false
		else:
			print("[Check] gml include=%s size=%s" % [view_name, view.get_size()])

	# 7. TabContainer：4 个页签 + 原生切换（抽屉已打开，页签参与布局）
	var tabs: TabContainer = panel.find_child("TaskTabs", true, false)
	if tabs == null:
		push_error("[Check] TaskTabs 不存在")
		quit(1)
		return
	print("[Check] tab_count=%d" % tabs.get_tab_count())
	if tabs.get_tab_count() != 4:
		push_error("[Check] 期望 4 个页签")
		ok = false

	# 8. 每页列表条目数（bean:task_list:tasks 兜底绑定建条目 + filter 分类过滤）
	#    默认数据分类分布：daily×2 / main×1 / guild×1 / bounty×1
	var expect := [2, 1, 1, 1]
	var lists: Array = []
	for i in range(4):
		var page: Control = tabs.get_tab_control(i)
		var list: GdUIVList = page.find_child("TaskList", true, false)
		if list == null:
			push_error("[Check] tab[%d] TaskList 不存在" % i)
			ok = false
			lists.append(null)
			continue
		lists.append(list)
		var count: int = list.get_child_count() - 1
		var item0: Control = list.get_at(0)
		var size0: Vector2 = item0.get_size() if item0 else Vector2()
		print("[Check] tab[%d] items=%d item0.size=%s" % [i, count, size0])
		if count != expect[i]:
			push_error("[Check] tab[%d] 期望 %d 条，实际 %d（列表数据未正确加载？）" % [i, expect[i], count])
			ok = false
		# 宽度只查可见页签：TabContainer 惰性布局，隐藏页条目 size 为 0 属正常
		if page.is_visible_in_tree() and (item0 == null or size0.x < 200.0):
			push_error("[Check] tab[%d] 条目宽度塌陷: %s" % [i, size0])
			ok = false

	# 9. 原生页签切换（切换后该页参与布局，验宽度）
	tabs.current_tab = 2
	await process_frame
	print("[Check] current_tab=%d visible_page=%s" % [tabs.current_tab, tabs.get_tab_control(2).is_visible_in_tree()])
	if tabs.current_tab != 2:
		push_error("[Check] 页签切换失败")
		ok = false
	var list2: GdUIVList = lists[2]
	if list2:
		var item2: Control = list2.get_at(0)
		var size2: Vector2 = item2.get_size() if item2 else Vector2()
		print("[Check] tab[2] after switch item0.size=%s" % size2)
		if item2 == null or size2.x < 200.0:
			push_error("[Check] tab[2] 切换后条目宽度塌陷: %s" % size2)
			ok = false

		# 10. 条目按钮信号（@pressed 声明，update 后 bind_events 重连）连通性
		var item = list2.get_at(0)
		var btn: Button = item.find_child("ItemBtn", true, false)
		if btn:
			var conn_count: int = btn.pressed.get_connections().size()
			print("[Check] item btn connections=%d" % conn_count)
			if conn_count == 0:
				push_error("[Check] 条目按钮信号未连接")
				ok = false
			btn.emit_signal("pressed")
			await process_frame
		else:
			push_error("[Check] 条目按钮 ItemBtn 不存在")
			ok = false

	# 11. 关闭抽屉（动画后整树隐藏）
	drawer.call("close")
	await create_timer(0.5).timeout
	print("[Check] drawer after close visible=%s is_open=%s" % [
		drawer.is_visible_in_tree(), drawer.call("is_drawer_open")])
	if drawer.is_visible_in_tree() or drawer.call("is_drawer_open"):
		push_error("[Check] 抽屉关闭失败（动画后应整树隐藏）")
		ok = false

	# 12. Bean 响应式刷新：等一个插入周期（5s），daily 页签应自动 +1 条
	var list0: GdUIVList = lists[0]
	if list0:
		var before: int = list0.get_child_count() - 1
		await create_timer(6.5).timeout
		var after: int = list0.get_child_count() - 1
		print("[Check] daily items before=%d after_6.5s=%d" % [before, after])
		if after <= before:
			push_error("[Check] Bean 动态插入未响应式刷新到列表")
			ok = false

	print("[Check] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
