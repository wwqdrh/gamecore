# 校验脚本：任务抽屉独立组件（task 目录只保留 Drawer）+ 统一 UI 管理层
# 运行：godot --headless --path . -s res://example/demo/xiuxian/scenes/main/ui/task/check_task_drawer.gd
# 职责：1) task gml 构建校验并重生成 tscn（抽屉组件 + tabs/list/item）
#       2) 独立实例化 task_drawer.gml.tscn：
#          挂树后自动注册 ui_id → GdUIManager.has_ui/find_ui（GD API 跨组件查找）
#          抽屉属性（方向/width_ratio 锚点）/初始隐藏/内容嫁接/半屏宽
#       3) find_ui 拿到组件后 open/close（GD 侧跨组件调用）+ 遮罩点击关闭
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/main/ui/task/"
const GML_FILES := [
	"task_drawer.gml", "task_tabs.gml",
	"task_list.gml", "task_item.gml",
]


func _initialize() -> void:
	_run()


func _run() -> void:
	# 1. 全部 gml 可构建 + 重生成 tscn
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

	# 2. 独立实例化抽屉组件（等价于组合根挂载）
	var scene: PackedScene = load(DIR + "task_drawer.gml.tscn")
	if scene == null:
		push_error("[Check] task_drawer.gml.tscn 加载失败")
		quit(1)
		return
	var holder: Control = Control.new()
	holder.size = Vector2(1152, 648)
	root.size = Vector2(1152, 648)
	root.add_child(holder)
	await process_frame
	var drawer: Control = scene.instantiate()
	holder.add_child(drawer)
	for i in 5:
		await process_frame

	# 3. 统一 UI 管理层：挂树自动注册 + GD API 查找
	print("[Check] registered ids=%s" % [GdUIManager.get_ui_ids()])
	if not GdUIManager.has_ui("TaskDrawer"):
		push_error("[Check] 抽屉挂树后未自动注册 ui_id=TaskDrawer")
		ok = false
	var found: Control = GdUIManager.find_ui("TaskDrawer")
	if found == null:
		push_error("[Check] GdUIManager.find_ui(\"TaskDrawer\") 返回 null（注册链路断）")
		print("[Check] RESULT=FAIL")
		quit(1)
		return
	if found != drawer:
		push_error("[Check] find_ui 返回的不是抽屉实例")
		ok = false

	# 4. 抽屉属性与内容
	print("[Check] drawer direction=%s width_ratio=%s" % [
		drawer.get("direction"), drawer.get("width_ratio")])
	if int(drawer.get("direction")) != 1:
		push_error("[Check] 抽屉方向应为 left(1)")
		ok = false
	if absf(float(drawer.get("width_ratio")) - 0.5) > 0.001:
		push_error("[Check] slide_width=\"50%%\" 应换算为 width_ratio=0.5")
		ok = false
	if drawer.is_visible_in_tree():
		push_error("[Check] 抽屉初始应为隐藏")
		ok = false
	var tabs: Control = drawer.find_child("TaskTabs", true, false)
	if tabs == null:
		push_error("[Check] 抽屉内容区未嫁接 TaskTabs")
		ok = false

	# 5. GD 侧跨组件调用：find_ui 拿到组件后直接 open（丝滑动画展开）
	found.call("open")
	await create_timer(0.5).timeout
	print("[Check] drawer open visible=%s is_open=%s" % [
		found.is_visible_in_tree(), found.call("is_drawer_open")])
	if not found.is_visible_in_tree() or not found.call("is_drawer_open"):
		push_error("[Check] find_ui 跨组件 open 失败")
		ok = false
	var dpanel: Control = found.find_child("DrawerPanel", true, false)
	if dpanel:
		print("[Check] drawer panel width=%.1f expect=576.0" % dpanel.size.x)
		if absf(dpanel.size.x - 576.0) > 2.0:
			push_error("[Check] 抽屉面板宽度应为 576（半屏）")
			ok = false
	else:
		push_error("[Check] DrawerPanel 不存在")
		ok = false
	# 5. 任务列表按接取状态动态显示（进行中/已完成两页签，数据由 XiuTaskState 驱动）：
	#    基线为空 → 接取进"进行中" → 推满待领取仍留"进行中" → 领奖后移入"已完成"
	if tabs:
		var task := XiuTaskState.ins()
		task.reset_demo()
		var lists: Array[Node] = tabs.find_children("TaskList", "", true, false)
		if lists.size() != 2:
			push_error("[Check] 应有进行中/已完成两个列表实例，实际 %d" % lists.size())
			ok = false
		else:
			var active_list: GdUIVList = lists[0]
			var done_list: GdUIVList = lists[1]
			# 条目数 = 子节点数 - 1（slot 模板）
			var cnt := func(l: GdUIVList) -> int: return l.get_child_count() - 1
			print("[Check] baseline active=%d done=%d" % [cnt.call(active_list), cnt.call(done_list)])
			if int(cnt.call(active_list)) != 0 or int(cnt.call(done_list)) != 0:
				push_error("[Check] 基线（未接取任何任务）两页签都应为空")
				ok = false
			task.accept_task("main_001")
			print("[Check] accepted active=%d done=%d" % [cnt.call(active_list), cnt.call(done_list)])
			if int(cnt.call(active_list)) != 1 or int(cnt.call(done_list)) != 0:
				push_error("[Check] 接取后进行中应 1 条、已完成 0 条")
				ok = false
			for i in 3:
				task.advance_task("main_001")
			if int(cnt.call(active_list)) != 1 or int(cnt.call(done_list)) != 0:
				push_error("[Check] 已完成待领取应留在进行中页签")
				ok = false
			task.complete_task("main_001")
			print("[Check] submitted active=%d done=%d" % [cnt.call(active_list), cnt.call(done_list)])
			if int(cnt.call(active_list)) != 0 or int(cnt.call(done_list)) != 1:
				push_error("[Check] 领奖后应移入已完成页签")
				ok = false
			# 还原基线，避免污染后续断言与其他测试
			task.reset_demo()

	# 6. 遮罩点击关闭
	var overlay: ColorRect = found.find_child("Overlay", true, false)
	if overlay:
		var ev: InputEventMouseButton = InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = true
		overlay.emit_signal("gui_input", ev)
		await create_timer(0.4).timeout
		print("[Check] drawer after overlay-click is_open=%s" % found.call("is_drawer_open"))
		if found.call("is_drawer_open"):
			push_error("[Check] 点击遮罩未收起抽屉")
			ok = false
	else:
		push_error("[Check] Overlay 不存在")
		ok = false

	# 7. 释放后注册表懒清理
	found.queue_free()
	await process_frame
	await process_frame
	print("[Check] after free has_ui=%s" % GdUIManager.has_ui("TaskDrawer"))
	if GdUIManager.has_ui("TaskDrawer"):
		push_error("[Check] 组件释放后注册表未清理")
		ok = false

	print("[Check] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
