@tool
extends EditorPlugin

var _console_panel: CanvasLayer


func _enter_tree():
	# 自动加载控制台面板
	_console_panel = load("res://addons/gamecore/ui/console_panel.gd").new()
	get_editor_interface().get_base_control().add_child(_console_panel)


func _exit_tree():
	if _console_panel:
		_console_panel.queue_free()
		_console_panel = null
