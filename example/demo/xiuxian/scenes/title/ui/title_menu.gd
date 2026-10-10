# 标题页菜单控制器（挂载在 title_menu.gml 根 <Panel> 上）
# 开始游戏 → 打开存档选择弹窗（SaveModal，内容 save_modal.gml）：
#   空档「新游戏」/ 有档「继续游戏」，由 save_modal.gd 切档并 change_scene
# 退出游戏 → 结束进程
extends Panel


## 开始游戏：刷新槽位并打开存档选择弹窗
func _on_start_pressed() -> void:
	var modal: Node = find_child("SaveModal", true, false)
	if modal == null:
		push_warning("TitleMenu: 未找到存档选择弹窗 SaveModal")
		return
	var content: Node = modal.find_child("SaveRoot", true, false)
	if content != null and content.has_method("refresh_slots"):
		content.call("refresh_slots")
	modal.call("open")


## 退出游戏
func _on_quit_pressed() -> void:
	get_tree().quit()
