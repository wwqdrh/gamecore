# GdWeather 换图回归 —— 换地图后天气状态不重置、覆盖层始终盖住屏幕
#
# 运行：perl -e 'alarm 180; exec @ARGV' godot --headless --path . \
#   -s res://test/check_weather_map_switch.gd
#
# 背景（2026-10-10 bug）：天气覆盖层曾锚定世界原点、按窗口像素尺寸摆放，
#   换图后相机 zoom/边界变化 + limit smoothing 滑行期间可视区越过地图边界
#   进入负坐标 → 屏幕出现无天气亮带，表现为"换图后天气被重置"。
#   修复：覆盖层每帧对齐可视世界矩形（画布变换逆推），与相机彻底解耦。
#
# 覆盖：
#   1. 管理器驱动：夜晚 tint + 降雨 fx 双通道同时生效
#   2. 换图（open_map 正式入口）：逻辑天气/视觉天气/雨密度/tint 全部保持
#   3. 逐帧监控切图后 90 帧：所有可见覆盖层矩形必须始终盖住可视世界矩形
#      （含相机 limit smoothing 滑行期）
#   4. 换回原图再验证一轮
extends SceneTree

var ok := true
var events: Array = []


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[WeatherMapSwitch] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _vis_world_rect() -> Rect2:
	var inv: Transform2D = root.get_canvas_transform().affine_inverse()
	var screen := root.get_visible_rect()
	var c0: Vector2 = inv * screen.position
	var c1: Vector2 = inv * (screen.position + screen.size)
	return Rect2(c0.min(c1), (c0.max(c1) - c0.min(c1)).abs())


func _coverage_failures(weather: Node) -> Array:
	# 所有可见 ColorRect 覆盖层必须盖住可视世界矩形（2px 容差吸收同帧时序差）
	var fails: Array = []
	var vis := _vis_world_rect()
	for child in weather.get_children():
		if child is ColorRect and (child as ColorRect).is_visible():
			var r := Rect2((child as ColorRect).get_global_position(), (child as ColorRect).get_size())
			if not r.grow(2.0).encloses(vis):
				fails.append("%s rect=%s vis=%s" % [child.name, r, vis])
	return fails


func _run() -> void:
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "主场景应可加载")
	if main_scene == null:
		quit(1)
		return
	var index: Node = main_scene.instantiate()

	# 管理器属性在挂树前配置（ready 读到的才是配置值）
	var mgr: Node = index.get_node_or_null("WeatherManager")
	var weather: Node = index.get_node_or_null("Weather")
	var map_mgr: Node = index.get_node_or_null("MapManager")
	check(mgr != null and weather != null and map_mgr != null, "三大节点应存在")
	if mgr == null or weather == null or map_mgr == null:
		quit(1)
		return
	mgr.set("start_time", 0.0)          # 不加偏移，用 set_total_time 精确控制
	mgr.set("rain_chance", 1.0)         # 必然下雨
	mgr.set("storm_chance", 0.0)
	mgr.set("fog_chance", 0.0)
	mgr.set("check_interval", 1.0)      # 挂树首帧即 roll 雨
	mgr.set("rain_max_duration", 7200.0)
	mgr.connect("s_weather_changed", func(n) -> void: events.append(str(n)))

	root.add_child(index)
	for i in 12:
		await physics_frame

	# ---- 1. 夜晚 + 降雨双通道 ----
	mgr.set_total_time(22.0 * 3600.0)
	check(str(mgr.get_current_weather()) == "night", "22 点应为 night")
	var got_rain := false
	for i in 300:
		await process_frame
		if float(weather.call("get_fx_density")) > 0.3:
			got_rain = true
			break
	check(got_rain, "rain_chance=1 应进入降雨且密度渐入")
	check(str(weather.call("get_weather")) == "rain", "视觉节点应为 rain（tint 保持 night 双通道）")
	check(int(weather.call("get_visible_overlay_count")) >= 2, "夜晚+雨应有 tint + fx 覆盖层")

	var logic_before: String = str(mgr.get_current_weather())
	var visual_before: String = str(weather.call("get_weather"))
	var density_before: float = float(weather.call("get_fx_density"))
	var fails := _coverage_failures(weather)
	check(fails.is_empty(), "切图前覆盖层应盖住可视区：%s" % [fails])

	# ---- 2/3. 换图 + 逐帧监控（含相机 limit smoothing 滑行期）----
	events.clear()
	var switched: bool = map_mgr.open_map("xiuxian_town", Vector2i(5, 5))
	check(switched, "open_map(xiuxian_town) 应成功")
	for i in 90:
		await process_frame
		var logic_now: String = str(mgr.get_current_weather())
		if logic_now != logic_before:
			check(false, "第 %d 帧管理器天气被重置：%s -> %s" % [i, logic_before, logic_now])
		var visual_now: String = str(weather.call("get_weather"))
		if visual_now != visual_before:
			check(false, "第 %d 帧视觉天气被重置：%s -> %s" % [i, visual_before, visual_now])
		var density_now: float = float(weather.call("get_fx_density"))
		if density_now < density_before - 0.05:
			check(false, "第 %d 帧雨密度回退：%.3f -> %.3f" % [i, density_before, density_now])
		fails = _coverage_failures(weather)
		if not fails.is_empty():
			check(false, "第 %d 帧覆盖层未盖住可视区：%s" % [i, fails])
	# 去重后不应有新的天气下发（换图不触发任何 push_weather）
	check(events.is_empty(), "换图不应触发 s_weather_changed，实际 %s" % [events])

	# ---- 4. 换回原图再验证一轮 ----
	events.clear()
	map_mgr.open_map("xiuxian_xiaozhai", Vector2i(10, 8))
	for i in 90:
		await process_frame
		fails = _coverage_failures(weather)
		if not fails.is_empty():
			check(false, "换回后第 %d 帧覆盖层未盖住可视区：%s" % [i, fails])
	check(str(mgr.get_current_weather()) == logic_before, "换回后天气逻辑仍应保持")
	check(str(weather.call("get_weather")) == visual_before, "换回后视觉天气仍应保持")
	check(events.is_empty(), "换回原图也不应触发 s_weather_changed，实际 %s" % [events])

	if ok:
		print("[WeatherMapSwitch] RESULT=PASS")
	else:
		print("[WeatherMapSwitch] RESULT=FAIL")
	quit(0 if ok else 1)
