# 单个任务条目控制器
# 挂载方式：task_item.gml 根标签 <ui script="task_item.gd"> 声明，构建期自动挂到
# 条目根节点（ItemRoot Panel）上——无需在场景或控制器中手动 add script。
# 条目被列表 duplicate 时脚本随节点复制，条目内 @pressed 声明就近绑定到本脚本；
# 列表 update() 重建条目后由 bind_events 自动重连，绑定声明完全收敛在 GML 与本文件内。
extends Panel


## 条目按钮回调（task_item.gml ItemBtn 的 @pressed 声明）
## btn: 发出信号的按钮；参数由信号绑定的智能补绑机制自动传入
func _on_task_action(btn: Control) -> void:
	# 沿父链找到携带 __item_data 的条目根节点（本节点的直接父链即条目内部）
	var item: Node = btn
	while item and not item.has_meta("__item_data"):
		item = item.get_parent()
	if item and item.has_meta("__item_data"):
		var data: Dictionary = item.get_meta("__item_data")
		print("[TaskItem] 点击任务按钮: ", data.get("title", "?"), " state=", data.get("btn_state", "?"))
