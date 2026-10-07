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

	# 8. Modal 组件：ui_id 注册 + 按钮跨组件弹出 + 内容关闭联动 + 遮罩关闭
	for modal_id in ["ProfileModal", "BagModal", "StoreModal"]:
		if not GdUIManager.has_ui(modal_id):
			push_error("[Check] 组合树挂载后未自动注册 ui_id=%s" % modal_id)
			ok = false
	print("[Check] modal ids registered=%s" % [GdUIManager.get_ui_ids()])

	var bag_modal: Control = demo.find_child("BagModal", true, false)
	var store_modal: Control = demo.find_child("StoreModal", true, false)
	var profile_modal: Control = demo.find_child("ProfileModal", true, false)
	for m in [bag_modal, store_modal, profile_modal]:
		if m == null or m.is_visible_in_tree():
			push_error("[Check] Modal 初始应为存在且隐藏")
			ok = false

	# 8a. MenuBag 点击 → BagModal 弹出（内容 = 现成 bag_panel.gml）
	var menu_bag: Control = demo.find_child("MenuBag", true, false)
	if menu_bag == null:
		push_error("[Check] mainhud 中找不到 MenuBag 按钮")
		quit(1)
		return
	menu_bag.emit_signal("gui_input", ev)
	await create_timer(0.4).timeout
	var bag_content: Control = bag_modal.find_child("BagPanel", true, false)
	print("[Check] MenuBag click -> BagModal open=%s visible=%s content=%s" % [
		bag_modal.call("is_modal_open"), bag_modal.is_visible_in_tree(), bag_content != null])
	if not bag_modal.call("is_modal_open") or bag_content == null:
		push_error("[Check] MenuBag 点击未弹出 BagModal（内容缺失或联动断链）")
		ok = false

	# 8b. 内容关闭联动：面板关闭按钮 emit s_close_requested → Modal 整体关闭
	bag_content.emit_signal("s_close_requested")
	await create_timer(0.4).timeout
	print("[Check] BagPanel s_close_requested -> BagModal open=%s visible=%s" % [
		bag_modal.call("is_modal_open"), bag_modal.is_visible_in_tree()])
	if bag_modal.call("is_modal_open") or bag_modal.is_visible_in_tree():
		push_error("[Check] 面板 s_close_requested 未联动关闭 Modal")
		ok = false

	# 8b+. 二次打开回归：内容根上次被面板 hide()，re-open 必须恢复可见
	menu_bag.emit_signal("gui_input", ev)
	await create_timer(0.4).timeout
	print("[Check] BagModal re-open -> open=%s visible=%s content_visible=%s" % [
		bag_modal.call("is_modal_open"), bag_modal.is_visible_in_tree(),
		bag_content.is_visible_in_tree()])
	if not bag_modal.call("is_modal_open") or not bag_content.is_visible_in_tree():
		push_error("[Check] BagModal 二次打开内容不可见（open 未恢复内容根可见？）")
		ok = false
	GdUIManager.find_ui("BagModal").call("close")
	await create_timer(0.4).timeout

	# 8c. MenuMarket 点击 → StoreModal 弹出；遮罩点击关闭
	var menu_market: Control = demo.find_child("MenuMarket", true, false)
	menu_market.emit_signal("gui_input", ev)
	await create_timer(0.4).timeout
	var store_content: Control = store_modal.find_child("StorePanel", true, false)
	print("[Check] MenuMarket click -> StoreModal open=%s content=%s" % [
		store_modal.call("is_modal_open"), store_content != null])
	if not store_modal.call("is_modal_open") or store_content == null:
		push_error("[Check] MenuMarket 点击未弹出 StoreModal")
		ok = false
	var store_overlay: ColorRect = store_modal.find_child("Overlay", true, false)
	if store_overlay:
		store_overlay.emit_signal("gui_input", ev)
		await create_timer(0.4).timeout
		print("[Check] StoreModal overlay click -> open=%s" % store_modal.call("is_modal_open"))
		if store_modal.call("is_modal_open"):
			push_error("[Check] StoreModal 点击遮罩未关闭")
			ok = false
	else:
		push_error("[Check] StoreModal Overlay 不存在")
		ok = false

	# 8d. 头像点击 → ProfileModal 弹出；GD 侧 find_ui 关闭
	var avatar: Control = demo.find_child("AvatarRing", true, false)
	if avatar == null:
		push_error("[Check] mainhud 中找不到 AvatarRing")
		quit(1)
		return
	avatar.emit_signal("gui_input", ev)
	await create_timer(0.4).timeout
	var profile_content: Control = profile_modal.find_child("ProfilePanel", true, false)
	print("[Check] Avatar click -> ProfileModal open=%s content=%s" % [
		profile_modal.call("is_modal_open"), profile_content != null])
	if not profile_modal.call("is_modal_open") or profile_content == null:
		push_error("[Check] 头像点击未弹出 ProfileModal")
		ok = false
	var profile_ui: Control = GdUIManager.find_ui("ProfileModal")
	if profile_ui:
		profile_ui.call("close")
		await create_timer(0.4).timeout
		if profile_modal.call("is_modal_open"):
			push_error("[Check] find_ui 跨组件 close ProfileModal 失败")
			ok = false
	else:
		push_error("[Check] GdUIManager.find_ui(\"ProfileModal\") 返回 null")
		ok = false

	print("[Check] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
