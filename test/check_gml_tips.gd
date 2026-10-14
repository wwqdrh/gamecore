# GML 悬停 tips 验收 —— tips/tips_title 属性 + Tooltip 宿主 hover 模式
#
# 运行：perl -e 'alarm 60; exec @ARGV' godot --headless --path . \
#   -s res://test/check_gml_tips.gd
#
# 覆盖：
#   1. 构建期：tips/tips_title 属性 → __tips/__tips_title meta 存节点；
#      带 tips 的布局类元素 mouse_filter 强制 STOP（默认 IGNORE 会吞悬停）
#   2. 运行期：GdGmlScene 加载 → connect_signals → 自动创建 __GmlTipsHost
#      宿主（follow_mouse=false），目标 mouse_entered/exited 已连接；幂等：
#      重复 connect_signals 不产生第二个宿主、不重复连接
#   3. 显示语义：_on_tips_entered → 延迟后显示、定位一次后固定不跟随、
#      标题/内容正确；切目标内容刷新
#   4. 关闭语义：_on_tips_exited 且鼠标不在目标/面板上 → 关闭
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[Tips] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	var gml := """
<ui>
  <Panel name="TopBar" custom_minimum_size="0,56">
	<HBoxContainer anchor="full" margin="8 8 8 8">
	  <Panel name="BadgeA" custom_minimum_size="100,36"
	         tips_title="灵石" tips="修真界通用货币。">
		<Label text="A" />
	  </Panel>
	  <Panel name="BadgeB" custom_minimum_size="100,36"
	         tips="只有内容的提示。">
		<Label text="B" />
	  </Panel>
	</HBoxContainer>
  </Panel>
</ui>
"""

	# ---- 1. 构建期 meta 与 mouse_filter ----
	var builder := GdUiBuilder.new()
	var ui_root: Control = builder.parse_string(gml)
	check(ui_root != null, "gml 应解析成功")
	if ui_root:
		var badge_a: Panel = ui_root.find_child("BadgeA", true, false)
		var badge_b: Panel = ui_root.find_child("BadgeB", true, false)
		check(badge_a != null and badge_b != null, "两个徽章应存在")
		if badge_a:
			check(badge_a.has_meta("__tips"), "BadgeA 应有 __tips meta")
			check(String(badge_a.get_meta("__tips")) == "修真界通用货币。", "tips 内容应正确")
			check(String(badge_a.get_meta("__tips_title")) == "灵石", "tips_title 应正确")
			check(badge_a.mouse_filter == Control.MOUSE_FILTER_STOP,
				"带 tips 的布局元素应强制 STOP（实际 %d）" % badge_a.mouse_filter)
		if badge_b:
			check(badge_b.has_meta("__tips"), "BadgeB 应有 __tips meta")
			check(not badge_b.has_meta("__tips_title"), "省略 tips_title 不应有 meta")
		ui_root.free()

	# ---- 2. 运行期接线（GdGmlScene 自动 connect → wire） ----
	var scene := GdGmlScene.new()
	root.add_child(scene)
	scene.position = Vector2(40, 40)
	scene.size = Vector2(400, 300)
	scene.call("load_from_string", gml)
	var content: Control = scene.call("get_content")
	check(content != null, "内容根应存在")
	if content == null:
		quit(1 if not ok else 0)
		return

	var host: Control = content.find_child("__GmlTipsHost", true, false)
	check(host != null, "应自动创建 __GmlTipsHost 宿主")
	if host == null:
		quit(1 if not ok else 0)
		return
	check(bool(host.get("follow_mouse")) == false, "宿主应为悬停模式（follow_mouse=false）")
	# 复现实机时序：宿主进树即 ready（空文本构建）——强制此刻建好 UI，
	# 之后 _on_tips_entered 才填文本（若此刻不建 Label，面板永远空白）
	host.call("ensure_ui_built")

	var badge_a: Panel = content.find_child("BadgeA", true, false)
	var badge_b: Panel = content.find_child("BadgeB", true, false)
	check(badge_a != null and badge_b != null, "内容树应含两个徽章")
	if badge_a:
		check(badge_a.get_signal_connection_list("mouse_entered").size() >= 1,
			"BadgeA 的 mouse_entered 应已连接")
		check(badge_a.get_signal_connection_list("mouse_exited").size() >= 1,
			"BadgeA 的 mouse_exited 应已连接")

	# ---- 幂等：重复 connect 不重复连接/不重建宿主 ----
	scene.call("connect_signals", scene)
	var host2: Control = content.find_child("__GmlTipsHost", true, false)
	check(host2 == host, "重复 connect 不应新建宿主")
	if badge_a:
		check(badge_a.get_signal_connection_list("mouse_entered").size() == 1,
			"重复 connect 不应产生重复连接（实际 %d）"
			% badge_a.get_signal_connection_list("mouse_entered").size())

	# ---- 3. 显示语义 ----
	host.call("_on_tips_entered", badge_a, "灵石", "修真界通用货币。")
	# 默认 delay=0.3s，等显示
	await create_timer(0.6).timeout
	check(host.visible, "悬停进入后宿主应显示")
	if host.visible:
		var panel: Control = host.get_child(0)
		var pos0: Vector2 = panel.position
		check(panel.get_size().x > 0.0, "面板应有尺寸（实际 %s）" % str(panel.get_size()))
		# follow_mouse=false：多帧后位置不动（headless 鼠标恒 (0,0)，
		# 视口 64x64 触发翻转定位属正常，只验证固定不跟随）
		for i in 5:
			await process_frame
		check(panel.position == pos0, "显示后面板应固定不移动")
		var title: Label = host.find_child("TooltipTitle", true, false)
		check(title != null and String(title.text) == "灵石", "标题应正确（实际 %s）"
			% (String(title.text) if title else "<无>"))
		check(title != null and title.visible, "有标题时应可见")
		var body: Label = host.find_child("TooltipContent", true, false)
		check(body != null and String(body.text) == "修真界通用货币。", "内容应正确")
		check(body != null and body.visible, "有内容时应可见")

	# 切目标：内容刷新
	host.call("_on_tips_entered", badge_b, "", "只有内容的提示。")
	await create_timer(0.5).timeout
	if host.visible:
		var title2: Label = host.find_child("TooltipTitle", true, false)
		check(title2 != null and String(title2.text) == "",
			"无标题目标应显示空标题（实际 %s）" % (String(title2.text) if title2 else "<无>"))
		check(title2 != null and not title2.visible, "空标题应隐藏（含分隔线）")

	# ---- 4. 关闭语义：headless 鼠标在 (0,0)，不在目标/面板上 ----
	host.call("_on_tips_exited", badge_b)
	await create_timer(0.6).timeout
	check(not host.visible, "离开目标且不在面板上应关闭")
	# 重入新目标后再退出，应同样关闭（pending 状态复位）
	host.call("_on_tips_entered", badge_a, "灵石", "内容")
	await create_timer(0.5).timeout
	check(host.visible, "重新进入应再次显示")
	host.call("_on_tips_exited", badge_a)
	await create_timer(0.6).timeout
	check(not host.visible, "再次退出应关闭")

	scene.free()
	print("[Tips] %s" % ("ALL PASS" if ok else "FAILED"))
	quit(0 if ok else 1)
