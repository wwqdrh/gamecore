# 对话框面板 - 对话系统 UI
# 提供说话人名称、对话文本显示和选项按钮交互
# 作为 GdDialogue 的 dialogue_control 节点，接收 handle_line 回调
# 支持点击推进对话、选项选择、面板显示/隐藏
# 有选项时：先显示文本，点击后再显示屏幕居中的选项框
extends CanvasLayer

# 面板高度占屏幕比例
@export var panel_height_ratio: float = 0.3
# 字体大小
@export var font_size: int = 18
# 名称字体大小
@export var name_font_size: int = 20
# 选项字体大小
@export var option_font_size: int = 16
# 面板背景色
@export var panel_bg_color: Color = Color(0.05, 0.05, 0.1, 0.92)
# 名称颜色
@export var name_color: Color = Color(0.4, 0.8, 1.0)
# 文本颜色
@export var text_color: Color = Color(0.9, 0.9, 0.9)
# 选项按钮正常色
@export var option_normal_color: Color = Color(0.15, 0.25, 0.4, 0.9)
# 选项按钮悬停色
@export var option_hover_color: Color = Color(0.2, 0.35, 0.55, 0.95)
# 选项文字颜色
@export var option_text_color: Color = Color(0.85, 0.9, 1.0)
# 点击提示文字
@export var click_hint_text: String = "点击继续..."
# 选项框宽度
@export var option_box_width: int = 300

var _panel: PanelContainer
var _name_label: Label
var _text_label: RichTextLabel
var _hint_label: Label
var _dialogue: Node  # GdDialogue 节点引用
var _current_responses: Array = []
var _is_visible: bool = false
var _waiting_for_option: bool = false  # 是否在等待点击显示选项

# 选项框（屏幕居中）
var _option_panel: PanelContainer
var _option_vbox: VBoxContainer


func _ready() -> void:
	layer = 100
	_build_ui()
	hide_dialogue()


func _input(event: InputEvent) -> void:
	if not _is_visible:
		return

	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		if _waiting_for_option:
			# 点击后显示选项框
			_waiting_for_option = false
			_hint_label.text = ""
			_show_options()
			get_viewport().set_input_as_handled()
		elif _current_responses.is_empty() and _dialogue:
			# 无选项时点击推进下一条
			_dialogue.call("next", "")
			get_viewport().set_input_as_handled()


func _build_ui() -> void:
	# === 底部对话面板 ===
	_panel = PanelContainer.new()
	_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
	_panel.anchor_top = 1.0 - panel_height_ratio
	var style := StyleBoxFlat.new()
	style.bg_color = panel_bg_color
	style.border_color = Color(0.3, 0.3, 0.4)
	style.set_border_width_all(1)
	style.set_border_width(SIDE_TOP, 2)
	_panel.add_theme_stylebox_override("panel", style)
	add_child(_panel)

	var margin := MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 20)
	margin.add_theme_constant_override("margin_right", 20)
	margin.add_theme_constant_override("margin_top", 12)
	margin.add_theme_constant_override("margin_bottom", 12)
	_panel.add_child(margin)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)
	margin.add_child(vbox)

	_name_label = Label.new()
	_name_label.add_theme_font_size_override("font_size", name_font_size)
	_name_label.add_theme_color_override("font_color", name_color)
	vbox.add_child(_name_label)

	_text_label = RichTextLabel.new()
	_text_label.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_text_label.bbcode_enabled = true
	_text_label.add_theme_font_size_override("normal_font_size", font_size)
	_text_label.add_theme_color_override("default_color", text_color)
	var text_style := StyleBoxFlat.new()
	text_style.bg_color = Color(0.02, 0.02, 0.05, 0.5)
	_text_label.add_theme_stylebox_override("normal", text_style)
	vbox.add_child(_text_label)

	_hint_label = Label.new()
	_hint_label.add_theme_font_size_override("font_size", option_font_size - 2)
	_hint_label.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
	_hint_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	vbox.add_child(_hint_label)

	# === 屏幕居中的选项框 ===
	_option_panel = PanelContainer.new()
	_option_panel.set_anchors_preset(Control.PRESET_CENTER)
	_option_panel.offset_left = -option_box_width / 2.0
	_option_panel.offset_top = -80
	_option_panel.offset_right = option_box_width / 2.0
	_option_panel.offset_bottom = 80
	_option_panel.visible = false
	var opt_style := StyleBoxFlat.new()
	opt_style.bg_color = Color(0.08, 0.08, 0.14, 0.95)
	opt_style.border_color = Color(0.35, 0.4, 0.55)
	opt_style.set_border_width_all(2)
	opt_style.set_corner_radius_all(8)
	_option_panel.add_theme_stylebox_override("panel", opt_style)
	add_child(_option_panel)

	var opt_margin := MarginContainer.new()
	opt_margin.add_theme_constant_override("margin_left", 16)
	opt_margin.add_theme_constant_override("margin_right", 16)
	opt_margin.add_theme_constant_override("margin_top", 12)
	opt_margin.add_theme_constant_override("margin_bottom", 12)
	_option_panel.add_child(opt_margin)

	_option_vbox = VBoxContainer.new()
	_option_vbox.add_theme_constant_override("separation", 6)
	opt_margin.add_child(_option_vbox)


# GdDialogue 回调函数 - 处理每一行对话
func handle_line(dia_line: Dictionary) -> void:
	var speaker_name: String = str(dia_line.get("name", ""))
	var text: String = str(dia_line.get("text", ""))

	# 显示说话人名称
	if speaker_name.is_empty():
		_name_label.text = ""
	else:
		_name_label.text = speaker_name

	# 显示对话文本
	_text_label.clear()
	_text_label.append_text(text)

	# 清除旧选项
	_clear_options()
	_current_responses = []
	_waiting_for_option = false
	_option_panel.visible = false

	# 处理选项
	var responses = dia_line.get("response")
	var has_options: bool = false
	if responses != null:
		if typeof(responses) == TYPE_ARRAY:
			has_options = responses.size() > 0
		elif responses is Array:
			has_options = responses.size() > 0

	if has_options:
		# 先缓存选项数据，显示"点击继续"提示
		_current_responses = responses
		_waiting_for_option = true
		_hint_label.text = click_hint_text
	else:
		_hint_label.text = click_hint_text

	# 确保面板可见
	show_dialogue()


# 显示选项框（屏幕居中）
func _show_options() -> void:
	_clear_options()
	for i in range(_current_responses.size()):
		var resp = _current_responses[i]
		var resp_text: String = ""
		if resp is Dictionary:
			resp_text = str(resp.get("text", ""))
		elif resp != null:
			resp_text = str(resp)
		if not resp_text.is_empty():
			_add_option_button(resp_text, i)

	# 根据选项数量调整选项框高度
	var btn_height := 36
	var spacing := 6
	var padding := 24
	var total_height := _current_responses.size() * btn_height + (_current_responses.size() - 1) * spacing + padding
	_option_panel.offset_top = -total_height / 2.0
	_option_panel.offset_bottom = total_height / 2.0
	_option_panel.visible = true


# 添加选项按钮
func _add_option_button(text: String, index: int) -> void:
	var btn := Button.new()
	btn.text = text
	btn.custom_minimum_size.y = 36
	btn.add_theme_font_size_override("font_size", option_font_size)
	btn.add_theme_color_override("font_color", option_text_color)
	btn.add_theme_color_override("font_hover_color", Color.WHITE)

	var normal_style := StyleBoxFlat.new()
	normal_style.bg_color = option_normal_color
	normal_style.set_border_width_all(1)
	normal_style.border_color = Color(0.3, 0.4, 0.6)
	normal_style.set_corner_radius_all(4)
	normal_style.set_content_margin_all(8)
	btn.add_theme_stylebox_override("normal", normal_style)

	var hover_style := StyleBoxFlat.new()
	hover_style.bg_color = option_hover_color
	hover_style.set_border_width_all(1)
	hover_style.border_color = Color(0.4, 0.5, 0.7)
	hover_style.set_corner_radius_all(4)
	hover_style.set_content_margin_all(8)
	btn.add_theme_stylebox_override("hover", hover_style)

	var pressed_style := StyleBoxFlat.new()
	pressed_style.bg_color = Color(0.3, 0.45, 0.65, 0.95)
	pressed_style.set_border_width_all(1)
	pressed_style.border_color = Color(0.5, 0.6, 0.8)
	pressed_style.set_corner_radius_all(4)
	pressed_style.set_content_margin_all(8)
	btn.add_theme_stylebox_override("pressed", pressed_style)

	btn.pressed.connect(_on_option_selected.bind(index))
	_option_vbox.add_child(btn)


# 清除所有选项按钮
func _clear_options() -> void:
	for child in _option_vbox.get_children():
		child.queue_free()


# 选项被选中
func _on_option_selected(index: int) -> void:
	if index < 0 or index >= _current_responses.size():
		return
	if not _dialogue:
		return

	_option_panel.visible = false
	var resp_data: Dictionary = _current_responses[index]
	_dialogue.call("exec_response", resp_data, "")


# 设置 GdDialogue 节点引用
func set_dialogue(dialogue: Node) -> void:
	_dialogue = dialogue


# 显示对话框
func show_dialogue() -> void:
	_is_visible = true
	_panel.visible = true


# 隐藏对话框
func hide_dialogue() -> void:
	_is_visible = false
	_panel.visible = false
	_option_panel.visible = false
	_name_label.text = ""
	_text_label.clear()
	_clear_options()
	_current_responses = []
	_waiting_for_option = false
	_hint_label.text = ""
