# GdWeather 双通道过渡验收 —— tint 调色板缓动 / fx density 门控 / 交叉渐变
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://test/check_weather_fade.gd
#
# 覆盖：
#   1. 初始无覆盖层（tint 强度 0、无 fx）
#   2. tint 通道：set_tint 后强度缓动 0→1（连续取值递增，无跳变）；同名去重
#   3. fx 通道：set_effect 后 density 渐入（雨滴由少到多）；同名去重
#   4. 交叉渐变：效果切换瞬间双层同显，稳定后单层
#   5. 效果清除：density 渐出至 0（雨滴由多到少）
#   6. tint 清除：强度缓出至 0 后覆盖层隐藏
#   7. 全集切换不报错（dawn/dusk/fog/storm/snow/wind）
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[WeatherFade] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	var w: Node = GdWeather.new()
	w.set("fade_in_time", 0.3)
	w.set("fade_out_time", 0.4)
	w.set("tint_fade_time", 0.4)
	root.add_child(w)
	await physics_frame
	await physics_frame

	# ---- 1. 初始 ----
	check(int(w.call("get_visible_overlay_count")) == 0, "初始应无可见覆盖层")
	check(str(w.call("get_weather")) == "clear", "初始天气应为 clear")
	check(float(w.call("get_tint_strength")) == 0.0, "初始 tint 强度应为 0")

	# ---- 2. tint 缓动：强度连续递增（亮度渐变无跳变的核心断言）----
	w.call("set_tint", "night")
	await process_frame
	await process_frame
	var s0: float = float(w.call("get_tint_strength"))
	check(s0 > 0.0 and s0 < 1.0, "tint 缓动中强度应在 0..1，实际 %.3f" % s0)
	await create_timer(0.12).timeout
	var s1: float = float(w.call("get_tint_strength"))
	check(s1 > s0, "tint 强度应持续递增（%.3f → %.3f）" % [s0, s1])
	await create_timer(0.6).timeout
	check(absf(float(w.call("get_tint_strength")) - 1.0) < 0.02, "缓动完成后 tint 强度应为 1")
	# 同名去重：重复 set_tint 不重启缓动
	w.call("set_tint", "night")
	await process_frame
	check(absf(float(w.call("get_tint_strength")) - 1.0) < 0.02, "同名 set_tint 应去重")

	# ---- 3. fx density 渐入（雨滴由少到多）----
	w.call("set_effect", "rain")
	await process_frame
	await process_frame
	var d0: float = float(w.call("get_fx_density"))
	check(d0 > 0.0 and d0 < 1.0, "雨势渐入中 density 应在 0..1，实际 %.3f" % d0)
	await create_timer(0.15).timeout
	var d1: float = float(w.call("get_fx_density"))
	check(d1 > d0, "density 应持续递增（雨滴由少到多，%.3f → %.3f）" % [d0, d1])
	await create_timer(0.5).timeout
	check(absf(float(w.call("get_fx_density")) - 1.0) < 0.02, "渐入完成后 density 应为 1")

	# ---- 4. 交叉渐变：效果切换瞬间双层同显 ----
	w.call("set_effect", "storm")
	await process_frame
	await process_frame
	check(str(w.call("get_weather")) == "storm", "切换后目标效果应为 storm")
	check(int(w.call("get_visible_overlay_count")) == 3,
		"交叉渐变中间态应 tint 层 + 新旧两层 fx 同显，实际 %d" % int(w.call("get_visible_overlay_count")))
	await create_timer(0.8).timeout
	check(absf(float(w.call("get_fx_density")) - 1.0) < 0.02, "新效果应渐入完成")

	# ---- 5. 效果清除：density 渐出（雨滴由多到少）----
	w.call("set_effect", "clear")
	await process_frame
	await process_frame
	var d2: float = float(w.call("get_fx_density"))
	check(d2 > 0.0 and d2 < 1.0, "雨势渐出中 density 应在 0..1，实际 %.3f" % d2)
	await create_timer(0.8).timeout
	check(absf(float(w.call("get_fx_density"))) < 0.02, "渐出完成后 density 应为 0")

	# ---- 6. tint 清除：强度缓出至 0 后覆盖层隐藏 ----
	w.call("set_tint", "clear")
	await create_timer(0.8).timeout
	check(absf(float(w.call("get_tint_strength"))) < 0.02, "清除后 tint 强度应为 0")
	check(int(w.call("get_visible_overlay_count")) == 0, "清除后应无可见覆盖层")

	# ---- 7. 全集切换不报错 ----
	for name in ["dawn", "dusk", "fog", "storm", "snow", "wind", "day", "night"]:
		w.call("set_weather", name)
		await process_frame
		check(str(w.call("get_weather")) == name,
			"切换到 %s 应生效，实际 %s" % [name, str(w.call("get_weather"))])
		await create_timer(0.12).timeout

	# ---- 8. 兼容入口 set_weather 的 fx 路由（雨停回落即 clear fx）----
	w.call("set_weather", "rain")
	await create_timer(0.5).timeout
	w.call("set_weather", "day")   # 相位名 → tint + 清效果
	await create_timer(0.8).timeout
	check(str(w.call("get_weather")) == "day", "set_weather 相位名应路由到 tint")
	check(absf(float(w.call("get_fx_density"))) < 0.02, "相位切换应清掉 fx 效果（雨停渐出）")
	check(float(w.call("get_tint_strength")) > 0.9, "day 色调应缓动到位")

	if ok:
		print("[WeatherFade] RESULT=PASS")
	else:
		print("[WeatherFade] RESULT=FAIL")
	quit(0 if ok else 1)
