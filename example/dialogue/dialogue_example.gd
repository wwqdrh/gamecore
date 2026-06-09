# 对话系统示例 - 演示 GdDialogue + dialogue_panel 的使用
# 加载 chat1.txt 对话脚本，通过对话框 UI 面板展示对话内容
# 点击推进对话，选择选项触发分支跳转
# 本脚本作为 GdDialogue 的 dialogue_control 节点，转发 handle_line 到面板
extends Control

var _dialogue: Node  # GdDialogue 节点
var _panel: CanvasLayer  # dialogue_panel 面板


func _ready() -> void:
	# 创建对话框面板
	_panel = load("res://addons/gamecore/ui/dialogue_panel.gd").new()
	add_child(_panel)

	# 创建 GdDialogue 实例（GDExtension 类通过 ClassDB 创建）
	_dialogue = ClassDB.instantiate("GdDialogue")
	if _dialogue == null:
		push_error("无法创建 GdDialogue 实例，请检查 GDExtension 是否正确加载")
		return
	add_child(_dialogue)

	# 设置面板引用 GdDialogue
	_panel.set_dialogue(_dialogue)

	# 设置本脚本为 GdDialogue 的 dialogue_control 节点
	# Control 节点路径比 CanvasLayer 更可靠
	_dialogue.call("set_dialogue_control_path", get_path())

	# 加载对话脚本
	var file := FileAccess.open("res://example/dialogue/chat1.txt", FileAccess.READ)
	if file:
		var data := file.get_as_text()
		_dialogue.call("initial", data)
		file.close()

		# 启动对话
		_dialogue.call("next", "")
	else:
		push_error("无法加载对话文件: res://example/dialogue/chat1.txt")

	# 监听对话结束信号
	if _dialogue.has_signal("s_finished"):
		_dialogue.connect("s_finished", _on_dialogue_finished)


# GdDialogue 回调 - 转发到面板
func handle_line(dia_line: Dictionary) -> void:
	_panel.handle_line(dia_line)


func _on_dialogue_finished() -> void:
	_panel.hide_dialogue()
	print("对话结束")
