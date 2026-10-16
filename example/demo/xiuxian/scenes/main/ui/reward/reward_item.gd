# 领奖弹窗奖励格子条目控制器 —— 挂 reward_item.gml 根（Panel）
#
# 数据契约（列表 update 注入，@export 即契约）：icon / icon_texture /
# count / tips_title / tips（+ name 走 {{name}} 模板渲染）。
#
# tips 运行期接线：GML `tips` 属性是构建期 meta（__tips/__tips_title，
# 随 tscn 序列化），列表 update() 重建条目后 meta 是模板静态值——本脚本
# 在数据注入后沿父链找面板控制器（实现 gml_tips_host()）取共享 Tooltip
# 宿主（GdUITooltip），setup_hover(self, title, content) 重接悬停。
# 时序坑：条目注入发生在挂树前，_ready 必须补偿调用全部 _apply_xxx。
extends Panel

@export var icon: String = "":
	set(value):
		icon = value
		if is_node_ready():
			_apply_icon()
@export var icon_texture: String = "":
	set(value):
		icon_texture = value
		if is_node_ready():
			_apply_icon()
@export var count: String = "":
	set(value):
		count = value
		if is_node_ready():
			_apply_count()
@export var tips_title: String = "":
	set(value):
		tips_title = value
		if is_node_ready():
			_apply_tips()
@export var tips: String = "":
	set(value):
		tips = value
		if is_node_ready():
			_apply_tips()


func _ready() -> void:
	_apply_icon()
	_apply_count()
	_apply_tips()


func _apply_icon() -> void:
	var label := find_child("CellIcon", true, false) as Label
	var tex := find_child("CellTex", true, false) as TextureRect
	if label == null or tex == null:
		return
	if icon_texture != "":
		tex.texture = load(icon_texture)
		tex.visible = true
		label.visible = false
	else:
		tex.visible = false
		label.visible = true


func _apply_count() -> void:
	var label := find_child("CountLabel", true, false) as Label
	if label:
		label.visible = count != ""


## 悬停 tips 接线：沿父链向上找实现 gml_tips_host() 的面板控制器，
## 取共享 Tooltip 宿主把本格子（当前 tips 数据）挂上悬停
func _apply_tips() -> void:
	if tips.is_empty():
		return
	var n: Node = self.get_parent()
	while n != null:
		if n.has_method("gml_tips_host"):
			var host: Control = n.call("gml_tips_host")
			if host != null and host.has_method("setup_hover"):
				host.call("setup_hover", self, tips_title, tips)
			return
		n = n.get_parent()
