# 动作光标抽象回归 —— GdActionCursor 组件 + GdQuickMap/GdDigLayer 统一协议
#   1. 协议统一：两类地图根都有 set_cursor_for / set_cursor_cell / clear_cursor
#      + ready 自建 ActionCursor 子节点（GdActionCursor 组件）
#   2. GdQuickMap（xiaozhai）："mine" 光标只在该格有可开采矿石时亮
#      （实时查子 GdDecoLayer.is_mineable_at）；农田类动作不亮（无语义）
#   3. GdDigLayer（home）：dig 过滤沿用农田谓词（可锄格才亮），
#      光标定位 = 世界语义格心（根恒单位变换）
# 用法：godot --headless --path . -s res://test/check_action_cursor.gd
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[ActionCursor] " + msg)
		ok = false


func _initialize() -> void:
	# 看门狗：意外卡死自动退出（quit(2)）；deferred 启动（_initialize 期
	# 树未就绪，Timer.start 会报 not inside the scene tree）
	var wd := Timer.new()
	wd.wait_time = 40.0
	wd.one_shot = true
	wd.timeout.connect(func() -> void:
		print("[ActionCursor] WATCHDOG 超时")
		quit(2))
	root.add_child(wd)
	wd.call_deferred("start")
	_run()


func _run() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	# ---- 1. GdQuickMap（萧宅）：mine 光标过滤 ----
	var map: Node = (load("res://example/demo/xiuxian/map/xiaozhai.tscn") as PackedScene).instantiate()
	root.add_child(map)
	for i in 3:
		await process_frame  # DecoLayer ready 的 call_deferred spawn_deco

	# 1a. 统一协议：三类光标 API + ActionCursor 组件自建
	for m in ["set_cursor_for", "set_cursor_cell", "clear_cursor"]:
		check(map.has_method(m), "GdQuickMap 应有光标协议 %s" % m)
	var cursor: Node2D = map.get_node_or_null("ActionCursor")
	check(cursor != null, "GdQuickMap ready 应自建 ActionCursor 组件")
	check(cursor != null and cursor.get_class() == "GdActionCursor",
		"ActionCursor 应为 GdActionCursor 组件")

	var layer: Node = map.get_node("DecoLayer")
	check(layer != null, "萧宅应有 DecoLayer")
	# 1b. 找一个真实矿石格（mineable 实例必挂 Health）
	var ore_cell := Vector2i(-1, -1)
	var ore_cells := {}
	if layer != null:
		for child in layer.get_children():
			if not String(child.name).begins_with("Deco_"):
				continue
			if child.get_node_or_null("Health") == null:
				continue
			var c: Vector2i = child.get_meta("deco_cell")
			ore_cells[c] = true
			if ore_cell.x < 0:
				ore_cell = c
	check(ore_cell.x >= 0, "萧宅应铺出可开采矿石")

	if cursor != null and ore_cell.x >= 0:
		# 1c. mine：矿石格亮 + 定位格心
		map.call("set_cursor_for", ore_cell, "mine")
		check(cursor.visible, "mine 光标：矿石格 %s 应显示" % ore_cell)
		check(cursor.global_position == map.call("cell_to_center", ore_cell),
			"mine 光标应定位在矿石格中心（%s vs %s）" % [
				cursor.global_position, map.call("cell_to_center", ore_cell)])
		# 1d. mine：非矿石格（可行走的普通格）不亮（负反馈）
		var plain := Vector2i(-1, -1)
		for y in range(0, 14):
			for x in range(0, 20):
				var c := Vector2i(x, y)
				if not ore_cells.has(c) and bool(map.call("is_walkable", c)):
					plain = c
					break
			if plain.x >= 0:
				break
		check(plain.x >= 0, "应存在非矿石可行走格")
		if plain.x >= 0:
			map.call("set_cursor_for", plain, "mine")
			check(not cursor.visible, "mine 光标：非矿石格 %s 应隐藏" % plain)
		# 1e. 农田类动作在随机地图无语义 → 不亮
		map.call("set_cursor_for", ore_cell, "dig")
		check(not cursor.visible, "dig 光标：GdQuickMap 无农田语义应隐藏")
		map.call("set_cursor_for", ore_cell, "harvest")
		check(not cursor.visible, "harvest 光标：GdQuickMap 无作物语义应隐藏")
		# 1f. clear + 越界防御
		map.call("set_cursor_for", ore_cell, "mine")
		map.call("clear_cursor")
		check(not cursor.visible, "clear_cursor 应隐藏光标")
		map.call("set_cursor_cell", Vector2i(-1, 5))
		check(not cursor.visible, "越界格 set_cursor_cell 应隐藏")
		map.call("set_cursor_cell", Vector2i(10, 7))
		check(cursor.visible, "界内格 set_cursor_cell 通用高亮应显示")
		map.call("clear_cursor")

	map.queue_free()
	for i in 2:
		await process_frame

	# ---- 2. GdDigLayer（家）：dig 谓词沿用 ----
	var home: Node = (load("res://example/demo/xiuxian/map/home.tscn") as PackedScene).instantiate()
	root.add_child(home)
	for i in 3:
		await process_frame
	for m in ["set_cursor_for", "set_cursor_cell", "clear_cursor"]:
		check(home.has_method(m), "GdDigLayer 应有光标协议 %s" % m)
	var hcursor: Node2D = home.get_node_or_null("ActionCursor")
	check(hcursor != null, "GdDigLayer ready 应自建 ActionCursor 组件")
	if hcursor != null:
		# (9,6) 在 dig_rects 内且为草地 → 可锄亮
		home.call("set_cursor_cell", Vector2i(9, 6))
		check(hcursor.visible, "dig 光标：可锄格 (9,6) 应显示")
		check(hcursor.global_position == home.call("cell_to_center", Vector2i(9, 6)),
			"dig 光标应定位格心（%s vs %s）" % [
				hcursor.global_position, home.call("cell_to_center", Vector2i(9, 6))])
		# (0,0) dig_rects 外装饰草 → 不亮
		home.call("set_cursor_cell", Vector2i(0, 0))
		check(not hcursor.visible, "dig 光标：dig_rects 外应隐藏")
		# mine：家无矿石 → 不亮
		home.call("set_cursor_for", Vector2i(9, 6), "mine")
		check(not hcursor.visible, "mine 光标：GdDigLayer 无矿石应隐藏")
		home.call("clear_cursor")
		check(not hcursor.visible, "clear_cursor 应隐藏光标")

	home.queue_free()
	for i in 2:
		await process_frame

	print("[ActionCursor] 结果: %s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
