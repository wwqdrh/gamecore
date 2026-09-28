# DialogBox - 简易对话框 UI（配合 GdDialogue 使用）
#
# GdDialogue 会把每条对话行打包成 Dictionary 回调 handle_handle：
#   { name: 角色名, text: 文本, stage: 当前stage, response: [{text, fn, stage}] }
# 点击 / E / 空格推进对话；出现 response 选项时显示按钮，
# 点击按钮调用 dialogue.exec_response()（goto/continue/end 会自动续播）。
#
# 同时提供状态函数，可被 timeline 函数（@set_flag:xxx）或
# 触发器条件（condition_fn = "has_flag:xxx"）调用：
#   set_flag / has_flag / clear_flag
#   set_counter / add_counter / get_counter
extends CanvasLayer

## GdDialogue 节点路径
@export var dialogue_path: NodePath

var dialogue: GdDialogue
var flags := {}
var counters := {}
var current_role := ""
var responding := false

var panel: Panel
var name_label: Label
var text_label: Label
var choices_panel: PanelContainer
var choices_box: VBoxContainer
var hint_label: Label


func _ready() -> void:
	_build_ui()
	panel.hide()
	choices_panel.hide()
	if not dialogue_path.is_empty():
		var d := get_node_or_null(dialogue_path)
		if d != null:
			dialogue = d
			# 把自己注册为对话的 control 节点（setter 内会立即解析）
			d.set_dialogue_control_path(d.get_path_to(self))


# GdDialogue 每条对话行回调
func handle_line(line: Dictionary) -> void:
	panel.show()
	current_role = str(line.get("name", ""))
	text_label.text = str(line.get("text", ""))
	name_label.text = current_role

	responding = false
	choices_panel.hide()
	for c in choices_box.get_children():
		c.queue_free()
	var response: Array = line.get("response", [])
	if response.size() > 0:
		responding = true
		for item in response:
			var btn := Button.new()
			btn.text = str(item.get("text", ""))
			btn.pressed.connect(_choose.bind(item))
			choices_box.add_child(btn)
		choices_panel.show()
		hint_label.hide()
	else:
		hint_label.show()


func _choose(item: Dictionary) -> void:
	responding = false
	choices_panel.hide()
	for c in choices_box.get_children():
		c.queue_free()
	if dialogue != null:
		dialogue.exec_response(item, current_role)
		# exec_response 内部经 next() 续播；播完（无后续行）时 playing=false
		if not dialogue.is_playing():
			_close()


func _unhandled_input(event: InputEvent) -> void:
	if not panel.visible or responding:
		return
	var advance := false
	if event is InputEventMouseButton and event.pressed \
			and event.button_index == MOUSE_BUTTON_LEFT:
		advance = true
	elif event is InputEventKey and event.pressed and not event.echo \
			and (event.keycode == KEY_E or event.keycode == KEY_SPACE):
		advance = true
	if advance:
		_advance()


func _advance() -> void:
	if dialogue == null:
		return
	dialogue.next("")
	# 用 is_playing 判断结束：next() 显示最后一句后 playing 仍为 true，
	# 需再消费一次走到 timeline 末尾才会置 false（此时才允许关面板，
	# 否则触发器永远等不到对话结束、双方无法解除暂停）
	if not dialogue.is_playing():
		_close()


func _close() -> void:
	panel.hide()
	choices_panel.hide()


# ---------------------------------------------------------------------------
# 对话状态函数（timeline 函数 / 触发器条件都可调用）
# ---------------------------------------------------------------------------

## 设置标记：timeline 中写 @set_flag:标记名
func set_flag(flag: String) -> void:
	flags[flag] = true


## 查询标记：触发器条件写 condition_fn = "has_flag:标记名"
func has_flag(flag: String) -> bool:
	return flags.has(flag)


func clear_flag(flag: String) -> void:
	flags.erase(flag)


func set_counter(key: String, value: int) -> void:
	counters[key] = value


func add_counter(key: String, delta: int) -> void:
	counters[key] = counters.get(key, 0) + delta


func get_counter(key: String) -> int:
	return counters.get(key, 0)


# ---------------------------------------------------------------------------
# UI 构建
# ---------------------------------------------------------------------------

func _build_ui() -> void:
	panel = Panel.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.08, 0.09, 0.12, 0.92)
	style.set_corner_radius_all(10)
	style.set_border_width_all(2)
	style.border_color = Color(0.35, 0.5, 0.75)
	panel.add_theme_stylebox_override("panel", style)
	panel.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	panel.offset_left = -450
	panel.offset_right = 450
	panel.offset_top = -230
	panel.offset_bottom = -24
	# 面板本身不拦截点击，让点击落到 _unhandled_input 推进对话
	panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(panel)

	name_label = Label.new()
	name_label.position = Vector2(28, 14)
	name_label.add_theme_font_size_override("font_size", 22)
	name_label.add_theme_color_override("font_color", Color(1.0, 0.85, 0.45))
	panel.add_child(name_label)

	text_label = Label.new()
	text_label.position = Vector2(28, 52)
	text_label.size = Vector2(840, 90)
	text_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	text_label.add_theme_font_size_override("font_size", 19)
	text_label.add_theme_color_override("font_color", Color(0.92, 0.94, 0.98))
	panel.add_child(text_label)

	# 选项面板：屏幕正中独立悬浮，不与底部对话框重叠
	# （PRESET_CENTER + 四向 grow，尺寸随按钮数量围绕屏幕中心扩展）
	choices_panel = PanelContainer.new()
	var choice_style := StyleBoxFlat.new()
	choice_style.bg_color = Color(0.1, 0.11, 0.16, 0.95)
	choice_style.set_corner_radius_all(10)
	choice_style.set_border_width_all(2)
	choice_style.border_color = Color(0.85, 0.65, 0.3)
	choice_style.content_margin_left = 26
	choice_style.content_margin_right = 26
	choice_style.content_margin_top = 18
	choice_style.content_margin_bottom = 18
	choices_panel.add_theme_stylebox_override("panel", choice_style)
	choices_panel.set_anchors_preset(Control.PRESET_CENTER)
	choices_panel.grow_horizontal = Control.GROW_DIRECTION_BOTH
	choices_panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	add_child(choices_panel)

	choices_box = VBoxContainer.new()
	choices_box.add_theme_constant_override("separation", 10)
	choices_panel.add_child(choices_box)

	hint_label = Label.new()
	hint_label.text = "点击 / E 继续"
	hint_label.add_theme_font_size_override("font_size", 14)
	hint_label.add_theme_color_override("font_color", Color(0.6, 0.65, 0.75))
	hint_label.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	hint_label.offset_left = -130
	hint_label.offset_top = -32
	hint_label.offset_right = -16
	hint_label.offset_bottom = -10
	panel.add_child(hint_label)
