# 任务列表组件控制器 —— 由 task_list.gml 的 <ui script="task_list.gd"> 自动挂载
# 到内容根节点（ScrollContainer）。
#
# 职责：把列表接到状态层 XiuTaskState（bean id: xiuxian_task，
# example/demo/xiuxian/state/task/task_state.gd）的展示视图 `views` 上——
# 数据完全由状态 Bean 驱动（全量任务总表 + 运行进度派生，含未解锁封印态），
# 任务推进/奖励发放后 refresh_views → watch 回调 → 全部同名列表自动刷新。
# （旧版内置演示数据 + 5s 定时插桩已移除：数据归状态层，UI 只做视图。）
#
# 数据绑定链路（组合根直开 tscn 场景不走 GdGmlScene auto_bind_data，
# 由本控制器手动接线）：
#   task_list.gml  <UIVList data="bean:xiuxian_task:views"
#                          filter_key="category" filter_value_var="category">
#   分类值由引用方 <Gml src="task_list.gml" data-category="分类变量"> 具名映射注入。
extends ScrollContainer
var _bean: GdBean


func _ready() -> void:
	_bean = XiuTaskState.ins()
	# 延迟一帧再绑定：子区块 _ready 自底向上先于父级，等页签/面板就绪
	_setup.call_deferred()


func _setup() -> void:
	var list: Control = find_child("TaskList", true, false)
	if list == null:
		return
	# 初始填充（update(data, false)：count==0 走动态分支才会建条目——
	# force=true 在列表为空时增删分支都不命中，一个条目都不会创建）
	list.update(_bean.get_value_by_key("views"), false)
	# 响应式：状态层 refresh_views → watch 回调 → 列表自动刷新
	_bean.watch("views", _on_views_changed)


## Bean watch 回调（参数个数自适应：value 或 value+metas）
func _on_views_changed(value: Variant, _metas: Variant = null) -> void:
	var list: Control = find_child("TaskList", true, false)
	if list:
		list.update(value, false)
