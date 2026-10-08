# 背包网格控制器 —— 由 bag_grid.gml 的 <ui script="bag_grid.gd"> 声明，
# 挂载到网格面板根节点（GoodsPanel）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   0. 状态层数据源：把网格接到 XiuItemState（bean id: xiuxian_item，
#      example/demo/xiuxian/state/item/item_state.gd）的展示视图 `bag_views` 上
#      ——初始填充 + watch 响应式刷新（道具增减自动重建视图），并把全量视图
#      写入 __script_data meta 供分类重刷路径复用
#   1. 命令上行：点选条目 -> 选中互斥（selected 契约 setter）-> 写 KEY_SELECTED_ITEM
#   2. 状态下行：watch KEY_CATEGORY -> 更新过滤 meta 并重刷条目
#   3. 初始上报：就绪后把首个条目写入总线（视图数据不再内置 selected 字段）
# 物品名称/描述由状态层道具总表自带（数据完整，消费方不做映射补全）
#
# 注意：watch 回调内不得同步再调用 GDSTATE（重入 panic）
extends Panel
## 与 bag_panel.gd 的 KEY_CATEGORY 一致（demo 各自声明，避免 class_name 缓存依赖）
const KEY_CATEGORY := "bag.category"
const KEY_SELECTED_ITEM := "bag.selected_item"

var _bean: GdBean


func _ready() -> void:
	_bean = XiuItemState.ins()
	# 状态层数据源：初始填充 + 响应式刷新（等一帧确保面板初始分类已写入）
	_bind_state_data.call_deferred()
	# 状态下行：监听分类变化（注册即回调当前值；值为 nil 表示尚未初始化，跳过）
	_state().watch(KEY_CATEGORY, _on_category_changed)
	# 延迟到本帧末上报初始选中：此时面板已写入初始分类、详情卡已挂好监听
	_report_initial_selection.call_deferred()


## 状态层数据接线：初始填充 + watch（组合根直开 tscn 不走 auto_bind_data）
func _bind_state_data() -> void:
	var grid := _find_grid()
	if grid == null:
		return
	var views: Array = _bean.get_value_by_key("bag_views")
	grid.set_meta("__script_data", views)
	grid.call("update", views, false)
	_bean.watch("bag_views", _on_bag_views_changed)


## 背包视图变化：更新 __script_data（分类重刷取全量）+ 重刷网格
## （__filter_value meta 由分类 watch 维护，此处不触碰——保持当前过滤值）
func _on_bag_views_changed(value: Variant, _metas: Variant = null) -> void:
	var grid := _find_grid()
	if grid == null:
		return
	var views: Array = value if value is Array else []
	grid.set_meta("__script_data", views)
	grid.call("update", views, false)


func _state():
	return Engine.get_singleton("GDSTATE")


## UIGrid s_click_item 回调（@s_click_item 声明，信号自带条目参数）
func _on_item_clicked(item: Control) -> void:
	_set_selected(item)
	_state().set_state(KEY_SELECTED_ITEM, item.get_meta("__item_data", {}))


## 状态下行：分类变化 -> 写过滤 meta（空串 = 不过滤）+ 全量数据重刷
## （filter 在 update_container 内生效；数据源 = __script_data meta，
##   meta 未就绪时（面板初始分类写入早于本控制器延迟绑定）兜底取 Bean 当前视图）
func _on_category_changed(value: Variant) -> void:
	if value == null:
		return  # 尚无初始值，等面板写入
	var grid := _find_grid()
	if grid == null:
		return
	grid.set_meta("__filter_value", str(value))
	var data: Array = (grid.get_meta("__script_data") if grid.has_meta("__script_data")
			else _bean.get_value_by_key("bag_views"))
	grid.call("update", data, false)


## 初始上报：预选中条目（无则首件）写入总线
func _report_initial_selection() -> void:
	var grid := _find_grid()
	if grid == null:
		return
	var children := grid.get_children()
	for i in range(1, children.size()):  # index 0 为 slot 模板
		var cell := children.get(i) as Control
		if cell != null and cell.get("selected"):
			_report(cell)
			return
	if children.size() > 1:
		_report(children.get(1) as Control)


func _report(cell: Control) -> void:
	if cell == null:
		return
	_state().set_state(KEY_SELECTED_ITEM, cell.get_meta("__item_data", {}))


## 选中态互斥：清空其余格子金边，点亮当前格子（selected 契约 setter 联动）
func _set_selected(item: Control) -> void:
	var grid := _find_grid()
	if grid == null:
		return
	var children := grid.get_children()
	for i in range(1, children.size()):  # index 0 为 slot 模板
		var cell := children.get(i) as Control
		if cell != null:
			cell.set("selected", cell == item)


func _find_grid() -> GdUIGrid:
	var n: Node = self
	while n != null:
		var found := n.find_child("GoodsGrid", true, false)
		if found != null:
			return found as GdUIGrid
		n = n.get_parent()
	return null
