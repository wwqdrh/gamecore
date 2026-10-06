# 校验脚本：修仙 Demo 组合根集成测试（统一 UI 管理层跨组件调用）
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_demo_ui.gd
# 职责：1) main.gml（组合根）构建校验并重生成 tscn（传递构建 mainhud + task 抽屉）
#       2) 实例化组合根：ui_id 自动注册（GdUIManager.has_ui/find_ui）
#       3) mainhud 右侧 MenuTask 按钮（Panel @pressed="show:TaskDrawer" 点击模拟）
#          → 跨文件夹触发 task 抽屉 open（丝滑动画展开）
#       4) 半屏宽校验 + 遮罩点击关闭 + 再次展开（toggle 全链路）
extends SceneTree

const MAIN_GML := "res://example/demo/xiuxian/main.gml"


func _initialize() -> void:
	_run()


func _run() -> void:
	var ok := true

	# 1. 组合根构建 + 重生成 tscn（<Gml> 传递构建全部子组件）
	var builder := GdUiBuilder.new()
	var scene: PackedScene = builder.build_scene_file(MAIN_GML)
	if scene == null:
		push_error("[Check] main.gml 构建失败: %s" % builder.last_error())
		quit(1)
		return
	var err := ResourceSaver.save(scene, MAIN_GML + ".tscn")
	print("[Check] build main.gml save_err=%d" % err)
	if err != OK:
		ok = false

	# 2. 实例化组合根（1920x1080 项目窗口尺寸）
	var holder: Control = Control.new()
	holder.size = Vector2(1920, 1080)
	root.size = Vector2(1920, 1080)
	root.add_child(holder)
	await process_frame
	var demo: Control = scene.instantiate()
	holder.add_child(demo)
	for i in 5:
		await process_frame

	# 3. 统一 UI 管理层：组合树挂载后 ui_id 自动注册
	print("[Check] registered ids=%s" % [GdUIManager.get_ui_ids()])
	if not GdUIManager.has_ui("TaskDrawer"):
		push_error("[Check] 组合树挂载后未自动注册 ui_id=TaskDrawer")
		ok = false

	# 4. 跨组件触发：mainhud MenuTask 按钮（点击模拟）→ task 抽屉展开
	var drawer: Control = demo.find_child("TaskDrawer", true, false)
	if drawer == null:
		push_error("[Check] 组合树中找不到 TaskDrawer")
		quit(1)
		return
	var menu_task: Control = demo.find_child("MenuTask", true, false)
	if menu_task == null:
		push_error("[Check] mainhud 中找不到 MenuTask 按钮")
		quit(1)
		return
	var ev: InputEventMouseButton = InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = true
	menu_task.emit_signal("gui_input", ev)
	await create_timer(0.5).timeout
	print("[Check] after MenuTask click: drawer visible=%s is_open=%s" % [
		drawer.is_visible_in_tree(), drawer.call("is_drawer_open")])
	if not drawer.is_visible_in_tree() or not drawer.call("is_drawer_open"):
		push_error("[Check] MenuTask 点击未跨组件展开任务抽屉（注册表或点击模拟断链？）")
		ok = false

	# 5. 半屏宽（width_ratio 锚点自适应，1920 组合根 → 960）
	var dpanel: Control = drawer.find_child("DrawerPanel", true, false)
	if dpanel:
		print("[Check] drawer panel width=%.1f expect=960.0" % dpanel.size.x)
		if absf(dpanel.size.x - 960.0) > 3.0:
			push_error("[Check] 抽屉面板宽度应为 960（1920 组合根的一半）")
			ok = false
	else:
		push_error("[Check] DrawerPanel 不存在")
		ok = false

	# 6. 遮罩点击关闭 → 再按按钮重新展开（全链路回归）
	var overlay: ColorRect = drawer.find_child("Overlay", true, false)
	if overlay:
		var ev2: InputEventMouseButton = InputEventMouseButton.new()
		ev2.button_index = MOUSE_BUTTON_LEFT
		ev2.pressed = true
		overlay.emit_signal("gui_input", ev2)
		await create_timer(0.4).timeout
		print("[Check] after overlay click is_open=%s" % drawer.call("is_drawer_open"))
		if drawer.call("is_drawer_open"):
			push_error("[Check] 点击遮罩未收起抽屉")
			ok = false
	else:
		push_error("[Check] Overlay 不存在")
		ok = false
	menu_task.emit_signal("gui_input", ev)
	await create_timer(0.5).timeout
	print("[Check] re-open via MenuTask is_open=%s" % drawer.call("is_drawer_open"))
	if not drawer.call("is_drawer_open"):
		push_error("[Check] MenuTask 二次点击未重新展开抽屉")
		ok = false

	# 7. GD 侧跨组件 API（find_ui 动态调用）
	var found: Control = GdUIManager.find_ui("TaskDrawer")
	if found:
		found.call("close")
		await create_timer(0.4).timeout
		print("[Check] find_ui close is_open=%s" % drawer.call("is_drawer_open"))
		if drawer.call("is_drawer_open"):
			push_error("[Check] find_ui 跨组件 close 失败")
			ok = false
	else:
		push_error("[Check] GdUIManager.find_ui(\"TaskDrawer\") 返回 null")
		ok = false

	print("[Check] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
