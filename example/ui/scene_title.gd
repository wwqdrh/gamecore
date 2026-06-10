# 游戏标题界面 - 使用 GmlScene 加载 GML 布局
# PopupPanel 的显示/隐藏由 GML 内部信号绑定自动处理
# 此脚本仅处理游戏逻辑回调
extends Control


func _on_start_game() -> void:
	print("Start Game clicked")


func _on_continue_game() -> void:
	print("Continue Game clicked")


func _on_quit_game() -> void:
	get_tree().quit()
