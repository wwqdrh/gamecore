# 校验脚本：修仙角色面板 GML 结构验证
# 运行：godot --headless --path . -s res://example/demo/xiuxian/scenes/main/ui/profile/check_profile_ui.gd
# 职责：1) 全部 gml 构建校验并重生成同名 tscn（等效编辑器插件产物）
#       2) 实例化主面板验证 <Gml> 组合 / 数据驱动条目 / 模板渲染 / @export 契约注入
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/main/ui/profile/"
const GML_FILES := [
	"profile_panel.gml", "profile_topbar.gml", "profile_avatar.gml",
	"spirit_root_item.gml", "profile_realm.gml", "profile_skills.gml",
	"skill_item.gml", "profile_stats.gml",
]


func _initialize() -> void:
	_run()


func _run() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
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
	var panel_scene: PackedScene = load(DIR + "profile_panel.gml.tscn")
	if panel_scene == null:
		push_error("[Check] profile_panel.gml.tscn 加载失败")
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
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
	for view_name in ["TopBar", "AvatarPanel", "RealmPanel", "SkillsPanel", "StatsBar"]:
		if panel.find_child(view_name, true, false) == null:
			push_error("[Check] 子视图 %s 不存在" % view_name)
			ok = false
		else:
			print("[Check] view %s OK" % view_name)

	# 五行灵根列表：<script> 数据构建期直接驱动 UIVList
	var roots: GdUIVList = panel.find_child("SpiritRootList", true, false)
	var root_count: int = roots.get_child_count() - 1
	print("[Check] spirit_roots items=%d" % root_count)
	if root_count != 5:
		push_error("[Check] 五行灵根期望 5 条，实际 %d" % root_count)
		ok = false
	var root0: Control = roots.get_at(0)
	var name0: Label = root0.find_child("RootName", true, false)
	var bar0: ProgressBar = root0.find_child("RootBar", true, false)
	print("[Check] root0 element=%s bar_value=%s" % [name0.text, bar0.value])
	if name0.text != "金" or absf(bar0.value - 28.0) > 0.01:
		push_error("[Check] 五行灵根首条模板绑定失败")
		ok = false

	# 功法列表：条目数 + {{模板}} 渲染 + @export 契约注入 + locked 状态联动
	var skills: GdUIVList = panel.find_child("SkillList", true, false)
	var skill_count: int = skills.get_child_count() - 1
	print("[Check] skills items=%d" % skill_count)
	if skill_count != 3:
		push_error("[Check] 功法期望 3 条，实际 %d" % skill_count)
		ok = false
	var item0: Control = skills.get_at(0)
	var title0: Label = item0.find_child("SkillTitle", true, false)
	var contract_title: String = item0.get("title")
	print("[Check] skill0 tpl=%s contract=%s" % [title0.text, contract_title])
	if title0.text != "吐纳诀" or contract_title != "吐纳诀":
		push_error("[Check] 功法首条模板渲染/契约注入失败")
		ok = false
	var item2: Control = skills.get_at(2)
	var locked2: bool = item2.get("locked")
	var btn2: Button = item2.find_child("SkillBtn", true, false)
	print("[Check] skill2 locked=%s btn_disabled=%s" % [locked2, btn2.disabled])
	if not locked2 or not btn2.disabled:
		push_error("[Check] 御风术锁定态未生效")
		ok = false

	# 三列布局：BodyRow 内三列均分比例 3:4:3（stretch_ratio 静态生效，
	# 不依赖运行时百分比解析——此断言防"中列独占宽度"回归）
	var body_row: HBoxContainer = panel.find_child("BodyRow", true, false)
	if body_row == null:
		push_error("[Check] BodyRow 不存在")
		ok = false
	else:
		await process_frame
		var avatar: Control = panel.find_child("AvatarPanel", true, false)
		var realm: Control = panel.find_child("RealmPanel", true, false)
		var skills_panel: Control = panel.find_child("SkillsPanel", true, false)
		var total := body_row.size.x
		var w_avatar := avatar.size.x / total
		var w_realm := realm.size.x / total
		var w_skills := skills_panel.size.x / total
		print("[Check] columns avatar=%.2f realm=%.2f skills=%.2f (of %.0f)" % [
			w_avatar, w_realm, w_skills, total])
		# 比例 3:4:3，扣除两个 10px 间隔后各列占比约 0.3/0.4/0.3（±0.03 容差）
		if absf(w_avatar - 0.3) > 0.03 or absf(w_realm - 0.4) > 0.03 \
				or absf(w_skills - 0.3) > 0.03:
			push_error("[Check] 三列比例偏离 3:4:3（中列独占宽度的回归）")
			ok = false

	# 3. Modal 点击语义：框内（含非控件位置）点击不关闭，框外点击关闭
	#    （回归：内容窗口根曾因布局类默认 IGNORE，窗口内点击漏到遮罩被误关）
	# gml 根元素即 ProfilePanel：实例化后根节点自身就叫 ProfilePanel，
	# find_child 不搜自身，需先判根
	var win: Control = panel if str(panel.name) == "ProfilePanel" else panel.find_child("ProfilePanel", true, false)
	if win == null:
		push_error("[Check] ProfilePanel 不存在")
		ok = false
	else:
		print("[Check] profile window mouse_filter=%d" % win.mouse_filter)
		if win.mouse_filter != Control.MOUSE_FILTER_STOP:
			push_error("[Check] 窗口根应为 STOP（消费窗口内点击）")
			ok = false
	holder.queue_free()
	await process_frame

	var modal := GdUIModal.new()
	modal.content_margin = 20
	root.add_child(modal)
	var content: Control = panel_scene.instantiate()
	modal.add_content_child(content)
	for i in 4:
		await process_frame
	modal.open()
	await process_frame
	if not modal.is_modal_open():
		push_error("[Check] Modal open 失败")
		ok = false
	var overlay: ColorRect = modal.find_child("Overlay", true, false)
	var area: Control = modal.find_child("ContentArea", true, false)
	var win2: Control = content if str(content.name) == "ProfilePanel" else content.find_child("ProfilePanel", true, false)
	if overlay == null or area == null or win2 == null:
		push_error("[Check] Modal 内部节点缺失")
		ok = false
	else:
		await process_frame
		await process_frame
		# 框内点击：窗口中心（窗口根 STOP 本就该拦；即便漏到遮罩也不得关闭）
		var ev_in := InputEventMouseButton.new()
		ev_in.button_index = MOUSE_BUTTON_LEFT
		ev_in.pressed = true
		var in_global: Vector2 = (win2.get_global_rect() as Rect2).get_center()
		ev_in.position = overlay.get_global_transform().affine_inverse() * in_global
		overlay.emit_signal("gui_input", ev_in)
		await process_frame
		print("[Check] click inside -> open=%s" % modal.is_modal_open())
		if not modal.is_modal_open():
			push_error("[Check] 框内点击不应关闭 Modal")
			ok = false
		# 框外点击：遮罩角落（内容区外）
		var ev_out := InputEventMouseButton.new()
		ev_out.button_index = MOUSE_BUTTON_LEFT
		ev_out.pressed = true
		ev_out.position = Vector2(2, 2)
		overlay.emit_signal("gui_input", ev_out)
		await process_frame
		print("[Check] click outside -> open=%s" % modal.is_modal_open())
		if modal.is_modal_open():
			push_error("[Check] 框外点击应关闭 Modal")
			ok = false
	modal.queue_free()
	await process_frame

	print("[Check] %s" % ("ALL GREEN" if ok else "FAILED"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
