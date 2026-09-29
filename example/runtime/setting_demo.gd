# setting_demo - 游戏设置面板示例
#
# 所有设置逻辑（音频音量、全屏/垂直同步、自定义键值、持久化）均封装在
# Rust 侧 GdViewSetting（rust/src/manager/setting.rs），本脚本只负责 UI 与接线：
#   - build 时自动从 user://settings.data 恢复并应用（独立于游戏存档）
#   - set_volume/set_fullscreen/set_vsync = 应用 + 立即持久化
#   - get_value/set_value = 用户自定义设置的读写
#   - watch = 变更订阅演示
extends Control

const VOLUME_BUSES := ["Master", "Music", "Audio", "Voice"]
const DIFFICULTY_NAMES := ["轻松", "普通", "困难"]
const DEFAULTS := {
	"volume_master": 1.0,
	"volume_music": 0.8,
	"volume_audio": 1.0,
	"volume_voice": 1.0,
	"fullscreen": false,
	"vsync": true,
	"player_name": "旅行者",
	"difficulty": 1,
}

var setting: GdViewSetting
var sliders := {}            # bus -> HSlider
var volume_labels := {}      # bus -> Label（百分比显示）
var fullscreen_check: CheckBox
var vsync_check: CheckBox
var name_edit: LineEdit
var difficulty_option: OptionButton
var watch_log: Label
var status_label: Label
var watch_hits := 0

## 设置文件路径（默认独立于游戏存档；测试可注入专用路径做隔离）
@export var save_file: String = "user://settings.data"


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	setting = GdViewSetting.build(save_file, "setting")
	_build_ui()
	_load_to_ui()
	_register_watch()
	_set_status("设置已从 %s 恢复" % setting.save_file)


# ---------- UI 交互（一行接线 = 应用 + 持久化） ----------

func _on_volume_changed(bus_name: String, v: float) -> void:
	setting.set_volume(bus_name, v)
	volume_labels[bus_name].text = "%d%%" % roundi(v * 100.0)


func _on_fullscreen_toggled(pressed: bool) -> void:
	setting.set_fullscreen(pressed)
	_set_status("全屏: %s" % ("开" if pressed else "关"))


func _on_vsync_toggled(pressed: bool) -> void:
	setting.set_vsync(pressed)
	_set_status("垂直同步: %s" % ("开" if pressed else "关"))


func _on_name_changed(text: String) -> void:
	setting.set_value("player_name", text)


func _on_difficulty_selected(idx: int) -> void:
	setting.set_value("difficulty", idx)
	_set_status("难度已保存: %s" % DIFFICULTY_NAMES[idx])


func _register_watch() -> void:
	setting.watch("difficulty", func(_path: String):
		watch_hits += 1
		watch_log.text = "difficulty 变更订阅触发 %d 次（watch 回调）" % watch_hits
	)


func _reset_defaults() -> void:
	for bus_name: String in VOLUME_BUSES:
		sliders[bus_name].value = DEFAULTS["volume_" + bus_name.to_lower()]
	fullscreen_check.button_pressed = DEFAULTS.fullscreen
	vsync_check.button_pressed = DEFAULTS.vsync
	name_edit.text = DEFAULTS.player_name
	_on_name_changed(DEFAULTS.player_name)
	difficulty_option.select(DEFAULTS.difficulty)
	_on_difficulty_selected(DEFAULTS.difficulty)
	_set_status("已恢复默认设置")


func _dump_data() -> void:
	var dump := "音量: "
	for bus_name: String in VOLUME_BUSES:
		dump += "%s=%.0f%% " % [bus_name, setting.get_volume(bus_name) * 100.0]
	dump += "| 全屏: %s | 昵称: %s | 难度: %d" % [
		setting.is_fullscreen(),
		setting.get_value("player_name", DEFAULTS.player_name),
		setting.get_value("difficulty", DEFAULTS.difficulty),
	]
	_set_status(dump)


# ---------- UI 构建 ----------

func _load_to_ui() -> void:
	for bus_name: String in VOLUME_BUSES:
		var v: float = setting.get_value("volume_" + bus_name.to_lower(),
			DEFAULTS["volume_" + bus_name.to_lower()])
		sliders[bus_name].set_value_no_signal(v)
		volume_labels[bus_name].text = "%d%%" % roundi(v * 100.0)

	fullscreen_check.set_pressed_no_signal(
		setting.get_value("fullscreen", DEFAULTS.fullscreen))
	vsync_check.set_pressed_no_signal(setting.get_value("vsync", DEFAULTS.vsync))
	name_edit.text = setting.get_value("player_name", DEFAULTS.player_name)
	difficulty_option.select(setting.get_value("difficulty", DEFAULTS.difficulty))


func _build_ui() -> void:
	var bg := ColorRect.new()
	bg.color = Color(0.09, 0.10, 0.13)
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	for side in ["margin_left", "margin_right", "margin_top", "margin_bottom"]:
		margin.add_theme_constant_override(side, 24)
	add_child(margin)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 10)
	margin.add_child(root)

	root.add_child(_make_title("游戏设置"))

	# 音频区
	root.add_child(_make_section("音频音量"))
	for bus_name in VOLUME_BUSES:
		var row := HBoxContainer.new()
		var name_label := _make_label(bus_name, 90)
		row.add_child(name_label)
		var slider := HSlider.new()
		slider.min_value = 0.0
		slider.max_value = 1.0
		slider.step = 0.05
		slider.custom_minimum_size = Vector2(320, 20)
		slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(slider)
		var pct := _make_label("100%", 56)
		row.add_child(pct)
		slider.value_changed.connect(func(v: float): _on_volume_changed(bus_name, v))
		sliders[bus_name] = slider
		volume_labels[bus_name] = pct
		root.add_child(row)

	# 窗口区
	root.add_child(_make_section("窗口"))
	fullscreen_check = CheckBox.new()
	fullscreen_check.text = "全屏模式"
	fullscreen_check.toggled.connect(_on_fullscreen_toggled)
	root.add_child(fullscreen_check)
	vsync_check = CheckBox.new()
	vsync_check.text = "垂直同步 (VSync)"
	vsync_check.toggled.connect(_on_vsync_toggled)
	root.add_child(vsync_check)

	# 自定义区
	root.add_child(_make_section("自定义设置（GdViewSetting 持久化）"))
	var name_row := HBoxContainer.new()
	name_row.add_child(_make_label("玩家昵称", 90))
	name_edit = LineEdit.new()
	name_edit.custom_minimum_size = Vector2(320, 0)
	name_edit.placeholder_text = "输入昵称（每次输入即保存）"
	name_edit.text_changed.connect(_on_name_changed)
	name_row.add_child(name_edit)
	root.add_child(name_row)

	var diff_row := HBoxContainer.new()
	diff_row.add_child(_make_label("游戏难度", 90))
	difficulty_option = OptionButton.new()
	for d in DIFFICULTY_NAMES:
		difficulty_option.add_item(d)
	difficulty_option.item_selected.connect(_on_difficulty_selected)
	diff_row.add_child(difficulty_option)
	root.add_child(diff_row)

	watch_log = _make_label("", 400)
	watch_log.modulate = Color(0.6, 0.85, 1.0)
	root.add_child(watch_log)

	# 操作区
	var btn_row := HBoxContainer.new()
	var reset_btn := Button.new()
	reset_btn.text = "恢复默认"
	reset_btn.pressed.connect(_reset_defaults)
	btn_row.add_child(reset_btn)
	var dump_btn := Button.new()
	dump_btn.text = "查看当前设置"
	dump_btn.pressed.connect(_dump_data)
	btn_row.add_child(dump_btn)
	root.add_child(btn_row)

	status_label = _make_label("", 400)
	status_label.modulate = Color(0.5, 0.55, 0.6)
	root.add_child(status_label)


func _set_status(text: String) -> void:
	if status_label != null:
		status_label.text = text


func _make_title(text: String) -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", 26)
	return l


func _make_section(text: String) -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", 17)
	l.modulate = Color(1.0, 0.85, 0.4)
	return l


func _make_label(text: String, min_width: int) -> Label:
	var l := Label.new()
	l.text = text
	l.custom_minimum_size = Vector2(min_width, 0)
	l.add_theme_font_size_override("font_size", 14)
	return l
