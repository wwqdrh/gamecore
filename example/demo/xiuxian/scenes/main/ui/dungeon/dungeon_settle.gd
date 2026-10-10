# 秘境结算面板控制器 —— 由 dungeon_settle.gml 的 <ui script> 声明挂载。
#
# 数据源：XiuDungeonState（xiuxian_dungeon）.settle_view
#   {title, floors, exp, stones, items:[{icon,name,count,note}]}
# 由 DungeonManager.settle 在发奖后写入；watch 注册即回调当前值
# （弹窗打开时若已有上次结算数据则直接展示）。
extends Control

## 面板关闭请求：<Modal> 监听本信号整体关闭（同 profile/settings 面板约定）
signal s_close_requested

var _bean: XiuDungeonState


func _ready() -> void:
	_bean = XiuDungeonState.ins()
	_bean.watch("settle_view", _on_settle_changed)


func _on_settle_changed(value: Variant = null, _metas: Variant = null) -> void:
	if value == null or not (value is Dictionary) or (value as Dictionary).is_empty():
		return
	var view: Dictionary = value
	var title: Label = find_child("Title", true, false)
	if title:
		title.text = str(view.get("title", "秘境通关结算"))
	var floors: Label = find_child("LineFloors", true, false)
	if floors:
		floors.text = "扫荡层数：%d 层" % int(view.get("floors", 0))
	var exp_l: Label = find_child("LineExp", true, false)
	if exp_l:
		exp_l.text = "获得修为：+%d" % int(view.get("exp", 0))
	var stones: Label = find_child("LineStones", true, false)
	if stones:
		stones.text = "获得灵石：+%d" % int(view.get("stones", 0))
	# 战利品列表重建（每次结算条数不固定，整体重建最简单）
	var list: VBoxContainer = find_child("ItemList", true, false)
	if list == null:
		return
	for c in list.get_children():
		c.queue_free()
	for it in view.get("items", []):
		var row := Label.new()
		row.text = "%s %s %s%s" % [
			str(it.get("icon", "📦")), str(it.get("name", "")),
			str(it.get("count", "")),
			"（%s）" % str(it.get("note")) if str(it.get("note", "")) != "" else ""]
		row.add_theme_font_size_override("font_size", 15)
		row.add_theme_color_override("font_color", Color(0.81, 0.91, 0.79))
		row.mouse_filter = Control.MOUSE_FILTER_IGNORE
		list.add_child(row)


# ---------- 底部按钮（dungeon_settle.gml 的 @pressed） ----------
func _on_close_pressed() -> void:
	s_close_requested.emit()
