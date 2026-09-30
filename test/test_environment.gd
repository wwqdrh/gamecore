# suite: environment - drawer 绘图与天气系统测试
# 覆盖: GdCanvasDrawer 图形队列 / 树叶轮廓几何 / GdColorUtil 颜色工具 /
#       GdWeather 天气切换与 shader 动态绑定
extends "res://test/test_case.gd"


func test_drawer_shapes() -> void:
	var tree := Engine.get_main_loop() as SceneTree
	var d := GdCanvasDrawer.new()
	tree.root.add_child(d)

	d.add_rect(Vector2.ZERO, Vector2(10, 10), Color.RED, true)
	d.add_circle(Vector2(5, 5), 3.0, Color.BLUE, false)
	d.add_line(Vector2.ZERO, Vector2(1, 1), Color.WHITE, 2.0)
	d.add_polygon(PackedVector2Array([Vector2.ZERO, Vector2(4, 0), Vector2(2, 3)]),
		Color.GREEN, true)
	d.add_leaf(Vector2.ZERO, 12.0, 45.0, Color(0.3, 0.5, 0.2))
	assert_eq(d.shape_count(), 5, "5 个图形应全部入队")

	d.clear_shapes()
	assert_eq(d.shape_count(), 0, "清空后应无图形")

	d.free()


func test_leaf_outline() -> void:
	var pts: PackedVector2Array = GdCanvasDrawer.leaf_outline_points(20.0, 1.0)
	assert_true(pts.size() >= 16, "树叶轮廓应有足够采样点")
	assert_true(absf(pts[0].x) < 0.01 and absf(pts[0].y) < 0.01,
		"轮廓应起于叶尖(0,0)")
	var max_x := 0.0
	var max_y := 0.0
	for p: Vector2 in pts:
		max_x = maxf(max_x, p.x)
		max_y = maxf(max_y, p.y)
	assert_true(max_x > 3.0, "轮廓应有宽度")
	assert_true(absf(max_y - 20.0) < 0.5, "轮廓应沿 +Y 延伸到叶长")


func test_color_util() -> void:
	var c := GdColorUtil.hex_to_color("#FF8000")
	assert_eq(int(roundi(c.r * 255.0)), 255, "hex 红通道应正确")
	assert_eq(int(roundi(c.g * 255.0)), 128, "hex 绿通道应正确")
	assert_eq(int(roundi(c.b * 255.0)), 0, "hex 蓝通道应正确")

	var c8 := GdColorUtil.hex_to_color("#8040C0FF")
	assert_eq(int(roundi(c8.a * 255.0)), 255, "8 位 hex 的 alpha 应正确")

	assert_eq(GdColorUtil.color_to_hex(Color(1.0, 0.5, 0.0, 1.0)), "#FF8000FF",
		"color_to_hex 应输出 RRGGBBAA")

	var m := GdColorUtil.mix_colors(Color.BLACK, Color.WHITE, 0.5)
	assert_true(absf(m.r - 0.5) < 0.01, "黑白混合中点应为 0.5")

	var d := GdColorUtil.darken(Color(1.0, 1.0, 1.0, 1.0), 0.4)
	assert_true(absf(d.r - 0.6) < 0.01, "加深 0.4 应得 0.6")


func test_weather_switch() -> void:
	var tree := Engine.get_main_loop() as SceneTree
	var w := GdWeather.new()
	w.leaf_count = 8
	tree.root.add_child(w)

	for weather_name: String in ["day", "night", "rain", "snow", "wind"]:
		w.set_weather(weather_name)
		assert_eq(w.get_weather(), weather_name, "天气应切换为 " + weather_name)
		var ov := w.get_overlay()
		assert_true(ov != null, weather_name + " 应有覆盖层")
		if ov != null:
			assert_true(ov.visible, weather_name + " 覆盖层应可见")
			var mat := ov.get_material() as ShaderMaterial
			assert_true(mat != null, weather_name + " 应挂载 ShaderMaterial")
			if mat != null:
				assert_true(mat.get_shader() != null,
					weather_name + " 应有动态创建的 shader")

	# clear / 未知天气：覆盖层保留但隐藏，不崩溃
	w.set_weather("clear")
	assert_eq(w.get_weather(), "clear", "天气应切换为 clear")
	assert_true(w.get_overlay() != null, "clear 后覆盖层节点应保留")
	assert_true(w.get_overlay().visible == false, "clear 后覆盖层应隐藏")

	w.set_weather("unknown_xyz")
	assert_true(w.get_overlay().visible == false, "未知天气应无效果")

	w.free()
