# 临时校验脚本：验证宗门任务面板多 GML 组合结构（验证后删除）
extends SceneTree


func _initialize() -> void:
	_run()


func _run() -> void:
	var scene: PackedScene = load("res://example/ui/task/task_panel.tscn")
	if scene == null:
		push_error("[Check] task_panel.tscn 加载失败")
		quit(1)
		return
	# -s 模式下根窗口尺寸不可控，用定尺寸 holder 模拟真实窗口
	var holder: Control = Control.new()
	holder.size = Vector2(1152, 648)
	root.add_child(holder)
	var panel: Control = scene.instantiate()
	holder.add_child(panel)
	for i in 5:
		await process_frame

	var ok := true

	# 1. 骨架 slot 均已挂载
	for slot_name in ["TopBarSlot", "TabSlot", "ActivitySlot"]:
		var slot = panel.find_node(slot_name)
		if slot == null or slot.get_child_count() == 0:
			push_error("[Check] %s 未挂载子视图" % slot_name)
			ok = false
		else:
			print("[Check] slot=%s children=%d size=%s" % [slot_name, slot.get_child_count(), slot.get_size()])

	# 2. TabContainer：4 个页签 + 原生切换
	var tabs: TabContainer = panel.find_node("TaskTabs")
	if tabs == null:
		push_error("[Check] TaskTabs 不存在")
		ok = false
		quit(1)
		return
	print("[Check] tab_count=%d tab_position=%s" % [tabs.get_tab_count(), tabs.get("tab_position")])
	if tabs.get_tab_count() != 4:
		push_error("[Check] 期望 4 个页签")
		ok = false

	# 3. 每页列表条目数（隐藏页不参与布局，只验条目数；当前页额外验宽度）
	var expect := [5, 2, 2, 2]
	for i in range(4):
		var page: Control = tabs.get_tab_control(i)
		var list: GdUIVList = page.find_child("TaskList", true, false)
		var count: int = list.get_child_count() - 1
		var item0: Control = list.get_at(0)
		print("[Check] tab[%d] items=%d item0.size=%s" % [i, count, item0.get_size()])
		if count != expect[i]:
			push_error("[Check] tab[%d] 期望 %d 条，实际 %d" % [i, expect[i], count])
			ok = false
		if i == 0 and item0.get_size().x < 300.0:
			push_error("[Check] tab[0] 条目宽度塌陷: %s" % item0.get_size())
			ok = false

	# 3.5 探测 TabContainer 侧边页签属性名
	for prop in ClassDB.class_get_property_list("TabContainer"):
		var pname := str(prop["name"])
		if pname.containsn("position"):
			print("[Check] TabContainer prop: ", pname)

	# 4. 原生页签切换（切换后该页参与布局，验宽度）
	tabs.current_tab = 2
	await process_frame
	print("[Check] current_tab=%d visible_page=%s" % [tabs.current_tab, tabs.get_tab_control(2).is_visible_in_tree()])
	if tabs.current_tab != 2:
		push_error("[Check] 页签切换失败")
		ok = false
	var list2: GdUIVList = tabs.get_tab_control(2).find_child("TaskList", true, false)
	var item2: Control = list2.get_at(0)
	print("[Check] tab[2] after switch item0.size=%s" % item2.get_size())
	if item2.get_size().x < 300.0:
		push_error("[Check] tab[2] 切换后条目宽度塌陷: %s" % item2.get_size())
		ok = false

	# 5. 条目按钮信号（allbind）连通性
	var item = list2.get_at(0)
	var btn: Button = item.find_child("ItemBtn", true, false)
	var conn_count: int = btn.pressed.get_connections().size()
	print("[Check] item btn connections=%d" % conn_count)
	if conn_count == 0:
		push_error("[Check] 条目按钮信号未连接")
		ok = false
	btn.emit_signal("pressed")
	await process_frame

	print("[Check] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
