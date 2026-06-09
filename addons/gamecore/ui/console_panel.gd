# 控制台面板 - 后台控制台 UI
# 提供输入框和日志输出区域，用户可输入 Lua 命令获取响应
# 通过 GdConsole 单例执行命令，监听 console_output 信号显示输出
extends CanvasLayer

# 控制台可见性切换快捷键
@export var toggle_key: Key = KEY_QUOTELEFT
# 控制台高度占比
@export var console_height_ratio: float = 0.4
# 最大日志行数
@export var max_log_lines: int = 200
# 字体大小
@export var font_size: int = 14

var _console: Object  # GdConsole 单例
var _panel: PanelContainer
var _log_output: RichTextLabel
var _input_line: LineEdit
var _history: PackedStringArray = []
var _history_index: int = -1
var _visible: bool = false


func _ready() -> void:
	layer = 100
	_console = Engine.get_singleton("GdConsole")
	_build_ui()
	_hide_console()


func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == toggle_key:
			if _visible:
				_hide_console()
			else:
				_show_console()
			get_viewport().set_input_as_handled()
		elif _visible and event.keycode == KEY_UP:
			_navigate_history(-1)
			get_viewport().set_input_as_handled()
		elif _visible and event.keycode == KEY_DOWN:
			_navigate_history(1)
			get_viewport().set_input_as_handled()


func _build_ui() -> void:
	# 主面板容器
	_panel = PanelContainer.new()
	_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
	_panel.anchor_bottom = console_height_ratio
	# 半透明深色背景
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.05, 0.1, 0.92)
	style.border_color = Color(0.3, 0.3, 0.4)
	style.set_border_width_all(1)
	_panel.add_theme_stylebox_override("panel", style)
	add_child(_panel)

	# 垂直布局
	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 4)
	_panel.add_child(vbox)

	# 标题栏
	var title_bar := HBoxContainer.new()
	vbox.add_child(title_bar)

	var title := Label.new()
	title.text = "GdConsole (Lua 5.1)"
	title.add_theme_font_size_override("font_size", font_size)
	title.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
	title_bar.add_child(title)

	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_bar.add_child(spacer)

	var close_btn := Button.new()
	close_btn.text = "X"
	close_btn.add_theme_font_size_override("font_size", font_size - 2)
	close_btn.pressed.connect(_hide_console)
	title_bar.add_child(close_btn)

	# 分隔线
	var sep := HSeparator.new()
	vbox.add_child(sep)

	# 日志输出区域
	_log_output = RichTextLabel.new()
	_log_output.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_log_output.bbcode_enabled = true
	_log_output.scroll_following = true
	_log_output.add_theme_font_size_override("normal_font_size", font_size)
	_log_output.add_theme_color_override("default_color", Color(0.85, 0.85, 0.85))
	# 日志区域背景
	var log_style := StyleBoxFlat.new()
	log_style.bg_color = Color(0.02, 0.02, 0.05, 0.95)
	_log_output.add_theme_stylebox_override("normal", log_style)
	vbox.add_child(_log_output)

	# 输入框
	var input_container := HBoxContainer.new()
	vbox.add_child(input_container)

	var prompt := Label.new()
	prompt.text = ">"
	prompt.add_theme_font_size_override("font_size", font_size)
	prompt.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
	input_container.add_child(prompt)

	_input_line = LineEdit.new()
	_input_line.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_input_line.text = "Enter Lua command..."
	_input_line.add_theme_font_size_override("font_size", font_size)
	_input_line.text_submitted.connect(_on_submit)
	input_container.add_child(_input_line)

	# 连接 GdConsole 信号
	if _console and _console.has_signal("console_output"):
		_console.console_output.connect(_on_console_output)

	# 欢迎信息
	_append_log("[color=#4488cc]GdConsole ready. Type [b]help()[/b] for available commands.[/color]")


func _show_console() -> void:
	_visible = true
	_panel.visible = true
	_input_line.grab_focus()


func _hide_console() -> void:
	_visible = false
	_panel.visible = false


func _on_submit(text: String) -> void:
	if text.strip_edges().is_empty():
		return

	# 记录历史
	_history.append(text)
	_history_index = _history.size()

	# 显示输入
	_append_log("[color=#66cc66]> %s[/color]" % text)

	# 执行命令
	if _console:
		_console.execute(text)

	_input_line.clear()


func _on_console_output(text: String) -> void:
	if text.is_empty():
		return
	_append_log(text)


func _append_log(text: String) -> void:
	_log_output.append_text(text + "\n")

	# 限制日志行数
	var line_count := _log_output.get_line_count()
	if line_count > max_log_lines:
		# 清除旧内容，保留最近的
		var lines := _log_output.text.split("\n")
		var keep_from := max(0, lines.size() - max_log_lines)
		var trimmed := "\n".join(lines.slice(keep_from))
		_log_output.clear()
		_log_output.text = trimmed
		_log_output.scroll_to_line(_log_output.get_line_count())


func _navigate_history(direction: int) -> void:
	if _history.is_empty():
		return

	_history_index = clampi(_history_index + direction, 0, _history.size())
	if _history_index < _history.size():
		_input_line.text = _history[_history_index]
		_input_line.caret_column = _input_line.text.length()
