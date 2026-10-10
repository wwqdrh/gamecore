# GdWeatherManager 天气/昼夜管理器验收 —— 四相位 / 时间流 / 降雨雷暴晨雾 / 视觉驱动
#
# 运行：perl -e 'alarm 180; exec @ARGV' godot --headless --path . \
#   -s res://test/check_weather_manager.gd
#
# 覆盖：
#   1. 配置驱动：time_scale/day_length/start_time/rain_chance/storm_chance/
#      fog_chance/check_interval 等编辑器属性可配置且生效
#   2. 时间流：set_total_time 一次传初值 → 内部按 time_scale 自行累计
#   3. 四相位：午夜 night / 早 7 点 day / 傍晚 17 点 dusk / 清晨 5 点 dawn，
#      s_phase_changed / s_phase_name_changed 信号正确
#   4. 降雨：rain_chance=1 + storm_chance=0 → rain；storm_chance=1 → storm；
#      rain_chance=0 后降雨到期回到相位基准
#   5. 晨雾：dawn 相位 + fog_chance=1 → fog；set_total_time 换时刻后雾散
#   6. 视觉驱动：weather_path 指向 GdWeather 子节点 → 天气变化自动下发
#   7. demo 集成：主场景挂载 WeatherManager + Weather，初始时长取
#      GDCORE 存档累计时长，视觉与逻辑一致
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[Weather] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _build_mgr() -> Node:
	var root_node := Node2D.new()
	root_node.name = "WeatherRoot"
	root.add_child(root_node)

	var weather: Node = GdWeather.new()
	weather.name = "Weather"
	root_node.add_child(weather)

	var mgr: Node = GdWeatherManager.new()
	mgr.name = "Manager"
	root_node.add_child(mgr)
	# 编辑器可配置属性（Inspector 同名）
	mgr.set("time_scale", 200.0)        # 1 现实秒 = 200 游戏秒
	mgr.set("day_length", 86400.0)      # 一昼夜 86400 游戏秒
	mgr.set("start_time", 0.0)          # 不加偏移，用 set_total_time 精确控制
	mgr.set("twilight_hours", 1.5)      # 黎明/黄昏各 1.5 小时
	mgr.set("rain_chance", 0.0)         # 默认不下雨（各节单独开）
	mgr.set("storm_chance", 0.0)
	mgr.set("fog_chance", 0.0)
	mgr.set("check_interval", 5.0)      # 每 5 游戏秒判一次（0.025 现实秒）
	mgr.set("rain_max_duration", 40.0)  # 单次雨 ≤40 游戏秒（≤0.2 现实秒）
	mgr.set("auto_start_from_ticks", false)
	return mgr


func _run() -> void:
	# ---- 1/3. 构造 + 四相位判定 ----
	var mgr: Node = _build_mgr()
	await physics_frame
	await physics_frame

	# 午夜（hour 0）→ 黑夜相位
	check(not bool(mgr.is_daytime()), "total_time=0（午夜）应为黑夜语义")
	check(absf(float(mgr.get_hour()) - 0.0) < 0.1, "get_hour 应为 0 附近")
	check(str(mgr.get_phase_name()) == "night", "午夜相位应为 night，实际 %s" % str(mgr.get_phase_name()))
	check(str(mgr.get_current_weather()) == "night", "黑夜基准天气应为 night")

	# 早 7 点 → 白天（信号：夜→昼一次）
	var phases := [0]
	var phase_names := [""]
	mgr.connect("s_phase_changed", func(_d) -> void: phases[0] += 1)
	mgr.connect("s_phase_name_changed", func(n) -> void: phase_names[0] = str(n))
	mgr.set_total_time(7.0 * 3600.0)
	check(bool(mgr.is_daytime()), "早 7 点应为白天语义")
	check(str(mgr.get_phase_name()) == "day", "早 7 点相位应为 day，实际 %s" % str(mgr.get_phase_name()))
	check(str(mgr.get_current_weather()) == "day", "白天基准天气应为 day")
	check(phases[0] == 1, "夜→昼切换应发一次 s_phase_changed，实际 %d" % phases[0])
	check(phase_names[0] == "day", "s_phase_name_changed 应携带 day")

	# 傍晚 17 点 → 黄昏（day_start 6 / night_start 18 / twilight 1.5）
	mgr.set_total_time(17.0 * 3600.0)
	check(str(mgr.get_phase_name()) == "dusk", "17 点相位应为 dusk，实际 %s" % str(mgr.get_phase_name()))
	check(bool(mgr.is_daytime()), "17 点仍属白天语义（< 18 点）")
	check(str(mgr.get_current_weather()) == "dusk", "黄昏基准天气应为 dusk")

	# 清晨 5 点 → 黎明
	mgr.set_total_time(5.0 * 3600.0)
	check(str(mgr.get_phase_name()) == "dawn", "5 点相位应为 dawn，实际 %s" % str(mgr.get_phase_name()))
	check(not bool(mgr.is_daytime()), "5 点属黑夜语义（< 6 点）")

	# ---- 2. 时间流：一次传初值，内部自算 ----
	mgr.set_total_time(12.0 * 3600.0)  # 正午
	var t0: float = mgr.get_game_time()
	await create_timer(0.3).timeout
	var t1: float = mgr.get_game_time()
	check(t1 > t0 + 30.0, "时间应内部自算增长（%.1f → %.1f）" % [t0, t1])

	# ---- 4. 降雨与雷暴 ----
	var rains := [0]
	mgr.connect("s_weather_changed", func(_n) -> void: rains[0] += 1)
	mgr.set("rain_chance", 1.0)   # 必然下雨
	mgr.set("storm_chance", 0.0)  # 先纯雨
	# 轮询等待降雨出现（判定周期间有回到基准的短暂空隙，采样点可能撞上）
	var got_rain := false
	for i in 240:
		await process_frame
		if str(mgr.get_current_weather()) == "rain":
			got_rain = true
			break
	check(got_rain, "rain_chance=1 应进入降雨")

	# storm_chance=1 → 后续判定转雷暴（轮询等待：雨停→下一判定窗口有短暂基准空隙）
	mgr.set("storm_chance", 1.0)
	var got_storm := false
	for i in 240:
		await process_frame
		if str(mgr.get_current_weather()) == "storm":
			got_storm = true
			break
	check(got_storm, "storm_chance=1 应在连续判定中转入雷暴")

	# rain_chance=0 → 降雨到期回到相位基准（正午 day）
	mgr.set("rain_chance", 0.0)
	await create_timer(0.6).timeout
	check(str(mgr.get_current_weather()) == "day",
		"降雨结束后应回到相位基准 day，实际 %s" % str(mgr.get_current_weather()))
	check(rains[0] >= 3, "天气切换应发 s_weather_changed，实际 %d 次" % rains[0])

	# ---- 5. 晨雾：黎明 + fog_chance=1 ----
	mgr.set("fog_chance", 1.0)
	mgr.set_total_time(5.0 * 3600.0)  # 黎明（set_total_time 清空降雨/雾瞬态）
	await create_timer(0.4).timeout
	check(str(mgr.get_current_weather()) == "fog",
		"黎明 fog_chance=1 应起雾，实际 %s" % str(mgr.get_current_weather()))
	# set_total_time 换时刻 → 瞬态雾强制清空回基准
	mgr.set_total_time(12.0 * 3600.0)
	check(str(mgr.get_current_weather()) == "day",
		"换时刻后雾应消散回基准 day，实际 %s" % str(mgr.get_current_weather()))

	# ---- 6. 视觉驱动 ----
	var weather: Node = mgr.get_node_or_null("../Weather")
	check(weather != null, "兄弟位置应找到 Weather 视觉节点（子节点/兄弟回退）")
	if weather != null:
		check(str(weather.call("get_weather")) == "day", "视觉节点应与逻辑天气同步")

	# ---- 7. demo 集成：主场景挂载 + GDCORE 时长初值 ----
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "主场景应可加载")
	if main_scene != null:
		var index: Node = main_scene.instantiate()
		root.add_child(index)
		for i in 10:
			await physics_frame
		var demo_mgr: Node = index.get_node_or_null("WeatherManager")
		check(demo_mgr != null, "主场景应挂载 WeatherManager")
		var demo_weather: Node = index.get_node_or_null("Weather")
		check(demo_weather != null, "主场景应挂载 Weather 视觉节点")
		if demo_mgr != null:
			# auto_start_from_ticks=true：初值取 GDCORE 当前存档累计时长
			# （测试环境时长较短，默认 start_time 早 6 点 → 白天）
			var demo_w: String = str(demo_mgr.get_current_weather())
			check(demo_w == "day" or demo_w == "dawn",
				"demo 启动（时长较短+早6点基准）应为白天侧天气，实际 %s" % demo_w)
			check(str(demo_weather.call("get_weather")) == demo_w,
				"demo 视觉应与逻辑一致（%s）" % demo_w)
		index.queue_free()

	if ok:
		print("[Weather] RESULT=PASS")
	else:
		print("[Weather] RESULT=FAIL")
	quit(0 if ok else 1)
