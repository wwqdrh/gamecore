# 宗门任务面板控制器 —— 由 task_panel.gml 的 <ui script="task_panel.gd"> 声明，
# 构建期自动挂载到 gml 根元素（WindowPanel Panel）。无包装层：
# 编辑器生成的 task_panel.gml.tscn 直开运行即完整可用（信号绑定挂树自动连接）。
#
# 拆分与引用（GML 间直接 <Gml> 引用，无需脚本手动挂载）：
#   task_panel.gml    骨架：<Gml src="task_topbar/task_tabs/task_activity.gml">
#   task_tabs.gml     4 个页签各自 <Gml src="task_list.gml">（独立实例，共享数据源）
#   task_list.gml     <ui script="task_list.gd"> 自动挂载组件脚本；
#                     UIVList data="bean:task_list:tasks" 绑定 GdBean 数据源
#                     （task_list.gd 每 5s 插入任务，全部页签响应式刷新）；
#                     <Gml src="task_item.gml"> 构建期注入条目模板
#   task_item.gml     单个任务条目（{{key}} 模板绑定 + @pressed 信号声明）
#
# 控制器只负责信号回调与表现（信号连接由 GML @pressed 声明自动完成）：
#   1. @pressed 声明   按钮 pressed 信号自动连到本脚本对应方法（含列表条目内按钮）
#   2. 页签切换        TabContainer 原生行为，_ready 里只连 tab_changed 信号
#   3. 条目状态着色    数据到位后按 btn_state 给条目/按钮着色（纯表现，不碰数据）
extends Panel

const TAB_NAMES := ["日常", "主线", "宗门", "悬赏"]


func _ready() -> void:
	# 页签切换为原生行为，只连信号（TabContainer 在 task_tabs.gml 中显式命名）
	var tabs: TabContainer = find_child("TaskTabs", true, false)
	if tabs:
		tabs.tab_changed.connect(_on_tab_changed)
	# 任务条目由 task_list.gd 的 Bean 兜底绑定在延迟帧创建（_setup call_deferred），
	# 这里等一帧再做状态着色，保证各页签条目已存在
	await get_tree().process_frame
	_apply_all_btn_states()


## 遍历 4 个页签，对每个列表做条目状态着色
func _apply_all_btn_states() -> void:
	for tab_name in TAB_NAMES:
		var page: Control = find_child(tab_name, true, false)
		if page == null:
			continue
		var list: GdUIVList = page.find_child("TaskList", true, false)
		if list:
			_apply_btn_state(list)


# 条目状态着色（数据来自条目 meta __item_data）
func _apply_btn_state(list: GdUIVList) -> void:
	var i := 0
	while true:
		var item: Control = list.get_at(i)
		if item == null:
			break
		item.modulate = Color.WHITE
		var btn: Button = item.find_child("ItemBtn", true, false)
		if btn:
			btn.remove_theme_color_override("font_color")
		if item.has_meta("__item_data"):
			var data: Dictionary = item.get_meta("__item_data")
			match data.get("btn_state", "go"):
				"claim":
					if btn:
						btn.add_theme_color_override("font_color", Color("#ffd968"))
				"done":
					item.modulate = Color(1.0, 1.0, 1.0, 0.55)
		i += 1


# ---------- TabContainer 原生页签切换回调 ----------
func _on_tab_changed(tab: int) -> void:
	print("[TaskPanel] 切换页签: ", TAB_NAMES[tab] if tab < TAB_NAMES.size() else tab)


# ---------- 顶部栏回调（task_topbar.gml 的 @pressed） ----------
func _on_close_pressed() -> void:
	print("[TaskPanel] 关闭任务面板")
	hide()
