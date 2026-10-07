# 校验脚本：GdSceneRoot 场景流端到端（入口场景 / 相机自动挂载 / 场景切换 / CanvasLayer UI）
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_scene_flow.gd
# 职责：1) 实例化 xiuxian/index.tscn（GdSceneRoot）
#       2) 入口场景自动进入 title（场景上 entry_scene="xiuxian_title"）
#       3) ViewCamera 自动挂载 + title UI（标题/开始/退出按钮）居中断言
#       4) 点开始游戏 → 转场 → main 场景（CanvasLayer 挂载 main.gml 组合根）
extends SceneTree


func _initialize() -> void:
	_run()


func _run() -> void:
	var ok := true
	root.size = Vector2(1920, 1080)

	# 1. 实例化 GdSceneRoot（xiuxian/index.tscn）
	var packed: PackedScene = load("res://example/demo/xiuxian/index.tscn")
	if packed == null:
		push_error("[Flow] xiuxian/index.tscn 加载失败")
		quit(1)
		return
	var scene_root: Node = packed.instantiate()
	root.add_child(scene_root)

	# 等待 ready 链路（config 加载 + entry_scene 无动画进入 + 布局）
	for i in 8:
		await process_frame

	# headless -s 模式下 Window 固定 64x64 且 size 不可改（生产窗口 1920x1080 不受影响），
	# 手动撑大场景容器使布局断言与生产一致
	var stage: Control = scene_root.get_node_or_null("SceneLayer")
	if stage != null:
		stage.size = Vector2(1920, 1080)
		await process_frame

	# 2. 入口场景 = title
	var current: Node = scene_root.call("get_current_scene")
	var cur_id: String = current.call("get_scene_id") if current != null else "<null>"
	print("[Flow] entry scene=%s camera=%s" % [
		cur_id, scene_root.find_child("ViewCamera", true, false) != null])
	if cur_id != "xiuxian_title":
		push_error("[Flow] 入口场景不是 xiuxian_title（got %s）" % cur_id)
		ok = false
	if scene_root.find_child("ViewCamera", true, false) == null:
		push_error("[Flow] GdSceneRoot 未自动挂载 ViewCamera")
		ok = false

	# 3. title UI：按钮存在 + 标题区水平居中
	var btn_start: Control = scene_root.find_child("BtnStart", true, false)
	var btn_quit: Control = scene_root.find_child("BtnQuit", true, false)
	print("[Flow] BtnStart=%s BtnQuit=%s" % [btn_start != null, btn_quit != null])
	if btn_start == null or btn_quit == null:
		push_error("[Flow] 标题页缺少 开始/退出 按钮")
		ok = false
	else:
		# 布局链路打印（Index → UI 实例根 → MenuColumn → BtnStart）
		var n: Node = btn_start
		var chain := []
		while n != null and n is Control or n is Node2D:
			chain.push_front("%s(%s size=%s)" % [n.get_name(), n.get_class(),
				str(n.get("size")) if n is Control else "-"])
			n = n.get_parent()
		print("[Flow] 布局链: %s" % [chain])
		var start_cx: float = btn_start.global_position.x + btn_start.size.x / 2.0
		var quit_cx: float = btn_quit.global_position.x + btn_quit.size.x / 2.0
		print("[Flow] 按钮 center_x start=%.1f quit=%.1f（视口中心 960）" % [start_cx, quit_cx])
		if absf(start_cx - 960.0) > 2.0 or absf(quit_cx - 960.0) > 2.0:
			push_error("[Flow] 标题按钮未水平居中")
			ok = false

	# 4. 点开始游戏 → 转场（0.5s 淡出 + 0.5s 淡入）→ main 场景
	if btn_start != null:
		btn_start.emit_signal("pressed")
		# 转场最长 2×0.5s + 余量
		await create_timer(2.4).timeout
		for i in 4:
			await process_frame

		var after: Node = scene_root.call("get_current_scene")
		var after_id: String = after.call("get_scene_id") if after != null else "<null>"
		var ui_layer: Node = scene_root.find_child("UILayer", true, false)
		var main_ui: Node = scene_root.find_child("MainUI", true, false)
		var title_gone: bool = scene_root.find_child("BtnStart", true, false) == null
		print("[Flow] after start: scene=%s UILayer=%s MainUI=%s title_gone=%s" % [
			after_id, ui_layer != null, main_ui != null, title_gone])
		if after_id != "xiuxian_main":
			push_error("[Flow] 开始游戏后未进入 xiuxian_main（got %s）" % after_id)
			ok = false
		if ui_layer == null or not ui_layer is CanvasLayer:
			push_error("[Flow] main 场景缺少 CanvasLayer（UILayer）")
			ok = false
		if main_ui == null:
			push_error("[Flow] CanvasLayer 上未挂载 main.gml 组合根（MainUI）")
			ok = false
		if not title_gone:
			push_error("[Flow] 旧 title 场景未被移除")
			ok = false
		# UI 管理层注册（组合根挂载即注册）
		print("[Flow] ui_ids=%s" % [GdUIManager.get_ui_ids()])
		if not GdUIManager.has_ui("TaskDrawer"):
			push_error("[Flow] main UI 挂载后未注册 TaskDrawer")
			ok = false

	print("[Flow] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
