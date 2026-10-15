# 右上角小地图回归：mainhud_minimap 聚焦模式（GdMiniMap.fit_current）
# 运行：godot --headless --path . -s res://test/check_hud_minimap.gd
# 职责：1) mainhud 装配后 MiniMapView 为 GdMiniMap 且 fit_current=true
#       2) XiuMapThumbs 数据灌入（set_current 前后可安全切换）
#       3) 脚本刷新链：无 MapManager 时兜底 xiaozhai + 地图名 Label 更新
#       4) 聚焦绘制不炸（set_player_cell/越界格/隐藏光点）
extends SceneTree


func _initialize() -> void:
	_run()


func _run() -> void:
	# 看门狗：意外卡死自动退出（quit(2)）
	var wd := Timer.new()
	wd.wait_time = 40.0
	wd.one_shot = true
	wd.timeout.connect(func() -> void:
		print("[HudMiniMap] WATCHDOG 超时")
		quit(2))
	root.add_child(wd)
	wd.start()

	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	var hud_scene: PackedScene = load(
		"res://example/demo/xiuxian/scenes/main/ui/mainhud/mainhud.gml.tscn")
	if hud_scene == null:
		push_error("[HudMiniMap] mainhud.gml.tscn 加载失败")
		quit(1)
		return
	var holder := Control.new()
	holder.size = Vector2(1152, 648)
	root.add_child(holder)
	var hud: Control = hud_scene.instantiate()
	holder.add_child(hud)
	for i in 8:
		await process_frame

	_ok = true
	var view: Control = hud.find_child("MiniMapView", true, false)
	if view == null:
		push_error("[HudMiniMap] MiniMapView 不存在")
		quit(1)
		return
	_check(view.get_class() == "GdMiniMap", "MiniMapView 应为 GdMiniMap（实际 %s）" % view.get_class())
	_check(bool(view.get("fit_current")), "聚焦模式 fit_current 应为 true")
	_check(not bool(view.get("show_names")), "聚焦模式 show_names 应为 false（地图名走底部 Label）")

	# 地图名 Label：无 MapManager（独立 HUD 无主场景）→ 兜底 xiaozhai
	var name_label: Label = hud.find_child("MapName", true, false)
	_check(name_label != null and name_label.text == "萧宅",
		"无 MapManager 时地图名应兜底「萧宅」（实际 %s）" % (name_label.text if name_label else "null"))

	# 聚焦切换：灌数据后切任意 alias 绘制不炸（含玩家光点/越界/隐藏）
	view.call("clear_all")
	var layout: Dictionary = XiuMapThumbs.get_layout()
	var maps: Dictionary = XiuMapThumbs.get_maps()
	var portals: Dictionary = XiuMapThumbs.get_portals()
	var count := 0
	for alias in XiuMapThumbs.get_ordered_aliases():
		var info: Dictionary = layout.get(alias, {})
		var m: Dictionary = maps.get(alias, {})
		if m.is_empty():
			continue
		count += 1
		view.call("add_map", alias, str(info.get("name", alias)),
			int(info.get("x", 0)), int(info.get("y", 0)),
			int(m["w"]), int(m["h"]),
			PackedInt32Array(m["cells"]), PackedStringArray(m["terrains"]))
		for p in portals.get(alias, []):
			var c: Array = p["cell"]
			var tc: Array = p["to_cell"]
			view.call("add_portal", alias,
				Vector2i(int(c[0]), int(c[1])), str(p["to"]),
				Vector2i(int(tc[0]), int(tc[1])), str(p.get("label", "")))
	_check(count == 6, "XiuMapThumbs 应灌入 6 张地图（实际 %d）" % count,)
	for alias in ["xiuxian_home", "xiuxian_town", "xiuxian_map_b", "xiuxian_xiaozhai"]:
		view.call("set_current", alias)
		view.call("set_player_cell", Vector2i(10, 8))
		await process_frame
		view.call("set_player_cell", Vector2i(-9999, -9999))
	_check(true, "聚焦切换 + 玩家光点 + 隐藏全链路绘制无异常",)

	print("[HudMiniMap] 结果：", "PASS" if _ok else "FAIL")
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if _ok else 1)


var _ok := true


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[HudMiniMap] OK  ", msg)
	else:
		push_error("[HudMiniMap] FAIL  " + msg)
		_ok = false
