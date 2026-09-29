# suite: runtime-setting - GdViewSetting 与设置面板（setting.tscn）测试
# 覆盖: Rust 侧 GdViewSetting 音量/全屏/自定义键的持久化、
#        面板重建后从设置文件恢复、空文件默认值、独立设置文件（与存档分离）
extends "res://test/test_case.gd"

var _file_seq := 0


func _unique_file() -> String:
	_file_seq += 1
	return "user://test_setting_%d_%d.data" % [Time.get_ticks_msec(), _file_seq]


func _make_setting(file: String) -> GdViewSetting:
	return GdViewSetting.build(file, "test_setting")


func _make_panel(file: String) -> Control:
	var scene: PackedScene = load("res://example/runtime/setting.tscn")
	var panel = scene.instantiate()
	# 在 _ready 之前注入独立设置文件，避免写入真实 user://settings.data
	panel.save_file = file
	var tree := Engine.get_main_loop() as SceneTree
	tree.root.add_child(panel)
	return panel


func test_default_save_file_isolated() -> void:
	# 设置必须使用独立文件，不能与游戏存档 coredata.data 混在一起
	var scene: PackedScene = load("res://example/runtime/setting.tscn")
	var panel = scene.instantiate()
	assert_eq(panel.save_file, "user://settings.data",
		"默认应使用独立设置文件 settings.data")
	panel.free()


func test_setting_roundtrip() -> void:
	var file := _unique_file()

	# 音量：set_volume 应同步 AudioServer 总线并持久化
	var st := _make_setting(file)
	st.set_volume("Music", 0.35)
	var bus_idx := AudioServer.get_bus_index("Music")
	assert_true(bus_idx >= 0, "Music 总线应存在（管理器兜底创建）")
	assert_true(absf(AudioServer.get_bus_volume_db(bus_idx) - linear_to_db(0.35)) < 0.1,
		"Music 总线音量应同步为 linear_to_db(0.35)")
	assert_true(absf(st.get_value("volume_music", 0.0) - 0.35) < 0.001,
		"音量应持久化")

	# 全屏/自定义键：headless 下窗口调用跳过但字段照常持久化
	st.set_fullscreen(true)
	assert_true(st.get_value("fullscreen", false) == true, "全屏开关应持久化")
	st.set_value("player_name", "小明")
	assert_eq(String(st.get_value("player_name", "")), "小明", "昵称应持久化")
	st.set_value("difficulty", 2)
	assert_eq(int(st.get_value("difficulty", -1)), 2, "难度应持久化")

	# watch 订阅：强制写入应触发回调
	var hits := []
	st.watch("difficulty", func(path: String): hits.append(path))
	st.set_value("difficulty", 0)
	assert_eq(hits.size(), 1, "watch 回调应触发一次")

	# GdViewSetting 是 RefCounted，引用计数自动释放，不要 free()
	var st2 := _make_setting(file)
	assert_true(absf(st2.get_value("volume_music", 0.0) - 0.35) < 0.001,
		"重建实例应从设置文件恢复音量")
	assert_eq(int(st2.get_value("difficulty", -1)), 0, "重建实例应恢复难度")


func test_panel_restore_and_defaults() -> void:
	var file := _unique_file()

	# ---- 第一块面板：修改设置 ----
	var panel := _make_panel(file)
	panel.sliders["Music"].value = 0.35
	panel.fullscreen_check.button_pressed = true
	panel.name_edit.text_changed.emit("小明")
	panel.difficulty_option.select(2)
	panel.difficulty_option.item_selected.emit(2)
	assert_true(panel.watch_hits >= 1, "difficulty 的 watch 回调应已触发")
	panel.free()

	# ---- 第二块面板：从同一设置文件恢复 ----
	var panel2 := _make_panel(file)
	assert_true(absf(panel2.sliders["Music"].value - 0.35) < 0.001,
		"重建面板应恢复音量 0.35")
	assert_true(panel2.fullscreen_check.button_pressed == true,
		"重建面板应恢复全屏开关")
	assert_eq(panel2.name_edit.text, "小明", "重建面板应恢复玩家昵称")
	assert_eq(panel2.difficulty_option.selected, 2, "重建面板应恢复难度")
	panel2.free()

	# ---- 空设置文件：应回落到 UI 默认值 ----
	var panel3 := _make_panel(_unique_file())
	assert_true(absf(panel3.sliders["Music"].value - 0.8) < 0.001,
		"空文件音量应为默认 0.8")
	assert_eq(panel3.difficulty_option.selected, 1, "空文件难度应为默认普通")
	assert_eq(panel3.name_edit.text, "旅行者", "空文件昵称应为默认值")
	panel3.free()
