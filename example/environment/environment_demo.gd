# environment_demo - 环境天气演示
#
# 场景结构（运行时构建）：
#   - Background  GdCanvasDrawer  天空/远山/地面/树/灌木/落叶（基础图形 + 树叶绘制演示）
#   - Weather     GdWeather       天气系统（白天/夜晚/雨/雪/风，shader 动态绑定）
#   - UI          CanvasLayer     天气切换按钮 + 强度滑条
#
# 底部按钮切换天气，覆盖层从 0 渐入到 intensity；强度滑条实时调节效果浓度。
# 所有坐标按实际视口尺寸计算（设计基准 1152x648，等比缩放到窗口大小）。
extends Node2D

## 设计基准尺寸（坐标按此设计，运行时等比缩放到实际视口）
const DESIGN := Vector2(1152.0, 648.0)

const WEATHERS: Array[String] = ["day", "night", "rain", "snow", "wind"]
const WEATHER_NAMES: Array[String] = ["白天", "夜晚", "下雨", "下雪", "刮风"]

var weather: GdWeather
var status_label: Label
var W: float
var H: float
var GROUND_Y: float


func _ready() -> void:
	var vp := get_viewport_rect().size
	W = vp.x
	H = vp.y
	GROUND_Y = H * 0.784
	_build_background()
	weather = GdWeather.new()
	weather.leaf_count = 50
	add_child(weather)
	weather.set_weather("day")
	_build_ui()


func _set_weather(weather_name: String) -> void:
	weather.set_weather(weather_name)
	_set_status("当前天气: %s" % weather_name)


func _set_status(text: String) -> void:
	if status_label != null:
		status_label.text = text


# 设计坐标 → 实际视口坐标（等比缩放）
func _p(x: float, y: float) -> Vector2:
	return Vector2(x / DESIGN.x * W, y / DESIGN.y * H)


# 设计尺寸 → 实际尺寸（取两轴缩放的较小值，保持形状比例）
func _s(v: float) -> float:
	return v * minf(W / DESIGN.x, H / DESIGN.y)


# 设计坐标偏移 → 实际偏移（按 _p 同样缩放）
func _v(x: float, y: float) -> Vector2:
	return _p(x, y) - _p(0.0, 0.0)


# ---------- 场景绘制（GdCanvasDrawer 演示） ----------

func _build_background() -> void:
	var d := GdCanvasDrawer.new()
	d.name = "Background"
	add_child(d)

	# 天空
	d.add_rect(Vector2.ZERO, Vector2(W, H), Color(0.62, 0.78, 0.92), true)
	# 云（几个交叠的圆）
	for c: Array in [[180.0, 90.0, 46.0], [232.0, 100.0, 38.0], [282.0, 92.0, 42.0],
			[830.0, 70.0, 40.0], [878.0, 80.0, 34.0], [922.0, 72.0, 38.0]]:
		d.add_circle(_p(c[0], c[1]), _s(c[2]), Color(1, 1, 1, 0.85), true)
	# 远山
	d.add_circle(_p(240.0, 628.0), _s(260.0), Color(0.52, 0.66, 0.58), true)
	d.add_circle(_p(700.0, 708.0), _s(320.0), Color(0.46, 0.60, 0.54), true)
	d.add_circle(_p(1060.0, 608.0), _s(220.0), Color(0.55, 0.68, 0.60), true)
	# 地面
	d.add_rect(Vector2(0, GROUND_Y), Vector2(W, H - GROUND_Y),
		Color(0.36, 0.55, 0.28), true)

	# 主树：树干 + 两根树枝 + 叶簇（树叶 + 圆组合）
	var trunk := Color(0.40, 0.28, 0.18)
	var base := _p(576.0, 514.0)
	d.add_line(base, base + _v(-16.0, -200.0), trunk, _s(20.0))
	d.add_line(base + _v(-8.0, -150.0), base + _v(-90.0, -230.0), trunk, _s(10.0))
	d.add_line(base + _v(-10.0, -170.0), base + _v(84.0, -250.0), trunk, _s(10.0))
	_draw_foliage(d, base + _v(-8.0, -250.0), _s(120.0))
	_draw_foliage(d, base + _v(-100.0, -250.0), _s(70.0))
	_draw_foliage(d, base + _v(94.0, -270.0), _s(76.0))

	# 左右两棵小树
	_draw_small_tree(d, _p(160.0, 512.0), _s(110.0))
	_draw_small_tree(d, _p(960.0, 512.0), _s(130.0))

	# 灌木（圆）+ 地面落叶（树叶随机散布）
	for i: int in range(7):
		d.add_circle(_p(80.0 + i * 170.0, 502.0), _s(26.0 + (i % 3) * 10.0),
			Color(0.30, 0.48, 0.24), true)
	for i: int in range(14):
		var pos := _p(60.0 + i * 82.0, 532.0 + (i % 4) * 22.0)
		d.add_leaf(pos, _s(14.0), 60.0 + i * 37.0, GdColorUtil.autumn_leaf_color(i))


func _draw_foliage(d: GdCanvasDrawer, center: Vector2, radius: float) -> void:
	# 树冠底色（圆）+ 表层树叶
	d.add_circle(center, radius, Color(0.29, 0.49, 0.18), true)
	for i: int in range(16):
		var ang := TAU * i / 16.0
		var r := radius * (0.45 + 0.4 * ((i * 7) % 5) / 5.0)
		var pos := center + Vector2(cos(ang) * r, sin(ang) * r * 0.8)
		d.add_leaf(pos, radius * 0.18 + (i % 4) * 2.0,
			rad_to_deg(ang) + 90.0 + (i % 5) * 14.0,
			GdColorUtil.autumn_leaf_color(i % 3))


func _draw_small_tree(d: GdCanvasDrawer, base: Vector2, height: float) -> void:
	var trunk := Color(0.42, 0.30, 0.20)
	d.add_line(base, base + Vector2(-height * 0.07, -height), trunk, height * 0.11)
	_draw_foliage(d, base + Vector2(-height * 0.05, -height - height * 0.27),
		height * 0.42)


# ---------- UI ----------

func _build_ui() -> void:
	var ui := CanvasLayer.new()
	ui.name = "UI"
	add_child(ui)

	# 状态栏
	status_label = Label.new()
	status_label.text = "当前天气: day"
	status_label.position = Vector2(24.0, 16.0)
	status_label.add_theme_font_size_override("font_size", 20)
	status_label.add_theme_color_override("font_color", Color(1, 1, 1))
	ui.add_child(status_label)

	# 底部控制条：天气按钮 + 强度滑条
	var panel := PanelContainer.new()
	panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER_BOTTOM)
	panel.grow_horizontal = Control.GROW_DIRECTION_BOTH
	panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	ui.add_child(panel)

	var margin := MarginContainer.new()
	for side in ["margin_left", "margin_right", "margin_top", "margin_bottom"]:
		margin.add_theme_constant_override(side, 12)
	panel.add_child(margin)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	margin.add_child(row)

	for i: int in WEATHERS.size():
		var weather_name: String = WEATHERS[i]
		var b := Button.new()
		b.text = WEATHER_NAMES[i]
		b.pressed.connect(func(): _set_weather(weather_name))
		row.add_child(b)

	row.add_child(VSeparator.new())
	row.add_child(_make_label("强度"))
	var slider := HSlider.new()
	slider.min_value = 0.0
	slider.max_value = 1.0
	slider.step = 0.05
	slider.value = 1.0
	slider.custom_minimum_size = Vector2(140, 20)
	slider.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	slider.value_changed.connect(func(v: float): weather.intensity = v)
	row.add_child(slider)
	row.add_child(_make_label("清空"))
	var clear_btn := Button.new()
	clear_btn.text = "无"
	clear_btn.pressed.connect(func(): _set_weather("clear"))
	row.add_child(clear_btn)


func _make_label(text: String) -> Label:
	var l := Label.new()
	l.text = text
	l.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	return l
