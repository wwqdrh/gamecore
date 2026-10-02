# 宗门任务面板 —— 多 GML 组合示例（对照设计图用简单图形/emoji 占位）
#
# 拆分与引用（GML 间直接引用，无需脚本挂载）：
#   task_panel.gml    骨架：<Gml src="task_topbar/task_tabs/task_activity.gml">
#   task_tabs.gml     <script> 定义各页签任务数据，<Gml src="task_list.gml" data-tasks="..."> 映射传入
#   task_list.gml     <script> 定义默认数据，UIVList data="tasks" 绑定；
#                     <Gml src="task_item.gml"> 构建期注入条目模板
#   task_item.gml     单个任务条目（{{key}} 模板绑定 + @pressed 信号声明）
#
# 控制器只负责信号回调与表现（信号连接全部由 GML @pressed 声明自动完成）：
#   1. @pressed 声明          按钮 pressed 信号自动连到本脚本对应方法（含列表条目内按钮）
#   2. on_pressed 回调        顶部栏/条目按钮信号自动连到本脚本
extends GdGmlScene

const TAB_NAMES := ["日常", "主线", "宗门", "悬赏"]


func _ready() -> void:
	# gml_file 属性已配置时 GdGmlScene 自动加载（含全部 <Gml> 引用与 <script> 数据绑定）
	if not is_loaded():
		load_gml("res://example/ui/task/task_panel.gml")
	# 页签切换为原生行为，只连信号
	var tabs: TabContainer = find_node("TaskTabs")
	if tabs:
		tabs.tab_changed.connect(_on_tab_changed)
	# 列表条目已由 GML <script> 数据在构建期填充（data="tasks"），
	# 条目按钮的 @pressed 声明由列表 bind_events 自动重连到本脚本，
	# 控制器只剩状态着色这类表现逻辑
	for tab_name in TAB_NAMES:
		var page: Control = find_node(tab_name)
		if page == null:
			continue
		var list: GdUIVList = page.find_child("TaskList", true, false)
		if list == null:
			continue
		_apply_btn_state(list)
	# <script> 变量同时挂在内容根节点 meta 上，控制器可按需读取：
	# var vars: Dictionary = find_node("TaskTabs").get_meta("__script_vars")


## 条目状态着色（数据来自条目 meta __item_data）
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
	print("[TaskPanel] 切换页签: ", TAB_NAMES[tab])


# ---------- 顶部栏回调（task_topbar.gml 的 @pressed） ----------
func _on_close_pressed() -> void:
	print("[TaskPanel] 关闭任务面板")
	hide()
