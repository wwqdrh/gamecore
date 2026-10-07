# 标题页菜单控制器（挂载在 title_menu.gml 根 <Panel> 上）
# 开始游戏 → 沿父链找 GdScene 场景根 → change_scene("xiuxian_main")
#   （GdSceneRoot 播放遮罩转场，进入主场景 main/index.tscn）
# 退出游戏 → 结束进程
extends Panel


## 沿父链向上找场景根（GdScene，具备 change_scene 方法）
func _find_scene() -> Node:
	var node: Node = get_parent()
	while node != null:
		if node.has_method("change_scene"):
			return node
		node = node.get_parent()
	return null


## 开始游戏：切换到主场景
func _on_start_pressed() -> void:
	var scene := _find_scene()
	if scene != null:
		var entered: bool = scene.call("change_scene", "xiuxian_main", {})
		if not entered:
			push_warning("TitleMenu: change_scene(xiuxian_main) 失败（场景未注册？）")
	else:
		push_warning("TitleMenu: 未找到 GdScene 场景根，无法切换场景")


## 退出游戏
func _on_quit_pressed() -> void:
	get_tree().quit()
