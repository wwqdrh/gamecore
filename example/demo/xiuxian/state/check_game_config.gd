# 校验脚本：存档级游戏配置 Bean（XiuGameConfigState）+ 天气流速联动
# 运行：godot --headless --path . -s res://example/demo/xiuxian/state/check_game_config.gd
# 职责：1) 配置 Bean 默认值 / set_day_length 写入与 watch 通知
#       2) 持久化：写入后冷读存档文件能取回（跨存档隔离）
#       3) 流速换算：time_scale = 86400 / day_length 下发 WeatherManager 后
#          游戏时间按新流速推进
extends SceneTree

var ok := true


func _initialize() -> void:
	_run()


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	# 1. 默认值与写入/watch（watch 注册即回调当前值，随后通知新值）
	var cfg := XiuGameConfigState.ins()
	cfg.reset_demo()
	var watched: Array = []
	cfg.watch("day_length", func(v = null, _m = null): watched.append(float(v)))
	_check(absf(float(cfg.day_length) - 600.0) < 0.001,
		"默认 day_length=600（一昼夜现实 10 分钟；原 144s 过快）")
	_check(watched.size() >= 1 and absf(watched[0] - 600.0) < 0.001,
		"watch 注册即回调当前值")
	cfg.set_day_length(1200.0)
	_check(absf(float(cfg.day_length) - 1200.0) < 0.001, "set_day_length 写入成员")
	_check(watched.size() >= 2 and absf(watched[-1] - 1200.0) < 0.001,
		"变更通知携带新值")
	cfg.set_day_length(5.0)
	_check(absf(float(cfg.day_length) - float(XiuGameConfigState.MIN_DAY_LENGTH)) < 0.001,
		"低于下限按 MIN_DAY_LENGTH=30 截断")
	cfg.set_day_length(900.0)
	await process_frame

	# 2. 持久化（update 走 "~" 立即落盘）：新建 GdCoreData 冷读 _test 档文件
	var cold: GdCoreData = GdCoreData.build("user://coredata__test.data", "{}", false, "init")
	var saved: float = float(cold.value("xiuxian_gameconfig;day_length", 0.0, "init"))
	print("[GameConfig] cold read day_length=%f" % saved)
	_check(absf(saved - 900.0) < 0.001, "写入后存档文件可冷读（跨存档持久化）")

	# 3. 流速换算作用于 WeatherManager：time_scale = 86400 / day_length
	#    （CalendarManager._apply_time_speed 同公式；#[var] time_scale 自动 get/set）
	var weather: Node = ClassDB.instantiate("GdWeatherManager")
	root.add_child(weather)
	await process_frame
	await process_frame
	var expect: float = 86400.0 / 900.0
	weather.set("time_scale", expect)
	_check(absf(float(weather.get("time_scale")) - expect) < 0.01,
		"WeatherManager time_scale 可由配置驱动下发（%.1f 游戏秒/现实秒）" % expect)
	# 游戏时间按新流速推进：12 帧约 0.2 现实秒 ≈ 0.2 * 96 ≈ 19 游戏秒
	var t0: float = float(weather.get_game_time())
	var ms0: int = Time.get_ticks_msec()
	for i in 12:
		await process_frame
	var real_secs: float = float(Time.get_ticks_msec() - ms0) / 1000.0
	var dt: float = float(weather.get_game_time()) - t0
	print("[GameConfig] game_time +%f over %.3fs real (scale=%f)" % [
		dt, real_secs, float(weather.get("time_scale"))])
	# 流速生效：游戏秒增量 / 现实秒 ≈ time_scale（±30% 容差，帧率波动）
	_check(real_secs > 0.001 and absf(dt / real_secs - expect) < expect * 0.3,
		"游戏时间按新流速推进（实测 %.0f 游戏秒/现实秒，期望 %.0f）"
			% [dt / maxf(real_secs, 0.0001), expect])
	weather.free()
	await process_frame

	print("[GameConfig] RESULT=%s" % ("PASS" if ok else "FAIL"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)


func _check(cond: bool, msg: String) -> void:
	print("[GameConfig] %s %s" % ["[PASS]" if cond else "[FAIL]", msg])
	if not cond:
		ok = false
