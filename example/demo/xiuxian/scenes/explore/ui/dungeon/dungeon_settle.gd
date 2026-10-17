# 秘境结算面板控制器 —— 由 dungeon_settle.gml 的 <ui script> 声明挂载。
#
# 数据源：XiuDungeonState（xiuxian_dungeon）.settle_view
#   {title, floors, exp, stones, items:[{icon,name,count,note}]}
# 由 ExploreManager.settle 在发奖后写入；watch 注册即回调当前值
# （弹窗打开时若已有上次结算数据则直接展示）。
#
# 布局：外层 CenterContainer 全屏居中（面板收缩到内容尺寸）；战利品横向
# 排列（每项 = 图标方框 + 方框下方数量 [+ 备注]）；按钮固定宽度居中。
extends Control

## 面板关闭请求：<Modal> 监听本信号整体关闭（同 profile/settings 面板约定）
signal s_close_requested

var _bean: XiuDungeonState


func _ready() -> void:
	_bean = XiuDungeonState.ins()
	_bean.watch("settle_view", _on_settle_changed)
	# gml 容器 separation/alignment 构建期未接 → 代码补齐
	var content: VBoxContainer = find_child("Content", true, false)
	if content != null:
		content.add_theme_constant_override("separation", 12)
	var list: HBoxContainer = find_child("ItemList", true, false)
	if list != null:
		list.add_theme_constant_override("separation", 18)
		list.alignment = BoxContainer.ALIGNMENT_CENTER


func _on_settle_changed(value: Variant = null, _metas: Variant = null) -> void:
	if value == null or not (value is Dictionary) or (value as Dictionary).is_empty():
		return
	var view: Dictionary = value
	var title: Label = find_child("Title", true, false)
	if title:
		title.text = str(view.get("title", "秘境通关"))
	var floors: Label = find_child("LineFloors", true, false)
	if floors:
		floors.text = "扫荡层数：%d 层" % int(view.get("floors", 0))
	var exp_l: Label = find_child("LineExp", true, false)
	if exp_l:
		exp_l.text = "获得修为：+%d" % int(view.get("exp", 0))
	var stones: Label = find_child("LineStones", true, false)
	if stones:
		stones.text = "获得灵石：+%d" % int(view.get("stones", 0))
	# 战利品横向重建（每次结算条数不固定，整体重建最简单）
	var list: HBoxContainer = find_child("ItemList", true, false)
	if list == null:
		return
	for c in list.get_children():
		c.queue_free()
	for it in view.get("items", []):
		list.add_child(_make_item_cell(it))


# ---------- 战利品格子（横向排列：图标方框 + 下方数量） ----------

func _make_item_cell(it: Dictionary) -> VBoxContainer:
	var cell := VBoxContainer.new()
	cell.add_theme_constant_override("separation", 4)
	cell.mouse_filter = Control.MOUSE_FILTER_IGNORE

	# 图标方框（道具名作悬停提示）
	var box := Panel.new()
	box.custom_minimum_size = Vector2(56, 56)
	box.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	box.add_theme_stylebox_override("panel", _item_box_style())
	box.tooltip_text = str(it.get("name", ""))
	box.mouse_filter = Control.MOUSE_FILTER_PASS
	var icon := Label.new()
	icon.text = str(it.get("icon", "📦"))
	icon.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	icon.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	icon.add_theme_font_size_override("font_size", 26)
	icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	icon.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	box.add_child(icon)
	cell.add_child(box)

	# 数量在方框下方
	var count := Label.new()
	count.text = "×%d" % int(it.get("count", 1))
	count.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	count.add_theme_font_size_override("font_size", 14)
	count.add_theme_color_override("font_color", Color(0.95, 0.92, 0.82))
	count.mouse_filter = Control.MOUSE_FILTER_IGNORE
	cell.add_child(count)

	# 备注（如「首杀奖励」）：数量下方小字
	var note := str(it.get("note", ""))
	if note != "":
		var note_l := Label.new()
		note_l.text = note
		note_l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		note_l.add_theme_font_size_override("font_size", 11)
		note_l.add_theme_color_override("font_color", Color(0.73, 0.66, 0.82))
		note_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
		cell.add_child(note_l)
	return cell


## 图标方框样式（与 gml .item-box 同款，运行期节点需代码注入 stylebox）
func _item_box_style() -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0.227, 0.192, 0.275)      # #3a3146
	sb.border_color = Color(0.788, 0.725, 0.549)  # #c9b98c
	sb.set_border_width_all(1)
	sb.set_corner_radius_all(8)
	return sb


# ---------- 底部按钮（dungeon_settle.gml 的 @pressed） ----------
func _on_close_pressed() -> void:
	s_close_requested.emit()
