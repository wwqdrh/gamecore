# 游戏标题界面 GML 控制器 - 继承 GdGmlScene，处理 GML 中的事件回调
# PopupPanel 的显示/隐藏由 GML 内部信号绑定自动处理
# 此脚本仅处理游戏逻辑回调
extends GdGmlScene


func _on_start_game() -> void:
	print("Start Game clicked")


func _on_continue_game() -> void:
	print("Continue Game clicked")


func _on_quit_game() -> void:
	get_tree().quit()


func _on_fullscreen_toggle() -> void:
	var check = find_node("FullscreenCheck")
	if check:
		get_window().mode = Window.MODE_FULLSCREEN if check.button_pressed else Window.MODE_WINDOWED
