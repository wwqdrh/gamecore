# 背包详情卡控制器 —— 由 bag_detail.gml 的 <ui script="bag_detail.gd"> 声明，
# 挂载到详情面板根节点（DetailPanel）。操作按钮回调就近绑定本脚本。
#
# 职责（GdState 状态总线模式中的「视图」）：只 watch KEY_SELECTED_ITEM
# 更新自己，与网格/分类零耦合。展示规则归本视图所有：
#   locked -> 封印态（品质徽章"封印"+ 描述替换）；quality 缺省 -> 显示"普通"
#
# 注意：watch 回调内不得同步再调用 GDSTATE（重入 panic）
extends Panel

## 与 bag_panel.gd 的 KEY_SELECTED_ITEM 一致（demo 各自声明）
const KEY_SELECTED_ITEM := "bag.selected_item"

var _current_item: Dictionary = {}


func _ready() -> void:
	_state().watch(KEY_SELECTED_ITEM, _on_selection_changed)


func _state():
	return Engine.get_singleton("GDSTATE")


## 状态下行：填充图标/名称/品质徽章/描述
func _on_selection_changed(value: Variant) -> void:
	if value == null:
		return  # 尚无初始值，等网格初始上报
	var item: Dictionary = value
	_current_item = item
	var quality := str(item.get("quality", ""))
	var desc := str(item.get("desc", ""))
	if item.get("locked", false):
		quality = "封印"
		desc = "此格已被封印锁定，无法查看物品详情。"
	elif quality.is_empty():
		quality = "普通"
	var icon_label := find_child("DetailIcon", true, false) as Label
	if icon_label:
		icon_label.text = str(item.get("icon", "❓"))
	var name_label := find_child("DetailName", true, false) as Label
	if name_label:
		name_label.text = str(item.get("name", "未知物品"))
	var quality_text := find_child("QualityText", true, false) as Label
	if quality_text:
		quality_text.text = quality
	var desc_label := find_child("DetailDesc", true, false) as Label
	if desc_label:
		desc_label.text = desc


func _on_use_pressed() -> void:
	print("[BagDetail] 使用物品: ", _item_name())


func _on_sell_pressed() -> void:
	print("[BagDetail] 出售物品: ", _item_name())


func _on_compose_pressed() -> void:
	print("[BagDetail] 合成物品: ", _item_name())


func _item_name() -> String:
	return str(_current_item.get("name", "")) if not _current_item.is_empty() else "未知物品"
