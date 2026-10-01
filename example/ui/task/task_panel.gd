# 宗门任务面板 —— 多 GML 组合示例（对照设计图用简单图形/emoji 占位）
#
# 拆分与引用（GML 间直接引用，无需脚本挂载）：
#   task_panel.gml    骨架：<Gml src="task_topbar/task_tabs/task_activity.gml">
#   task_tabs.gml     每个 <Tab> 内 <Gml src="task_list.gml">（各自独立实例）
#   task_list.gml     UIVList 内 <Gml src="task_item.gml">（构建期注入条目模板）
#   task_item.gml     单个任务条目（{{key}} 模板绑定）
#
# 控制器只负责数据与信号：
#   1. list.update(data)     数据驱动刷新（模式三）
#   2. allbind_signal        批量连接条目按钮信号
#   3. on_pressed 回调       顶部栏/条目按钮信号自动连到本脚本
extends GdGmlScene

const TAB_NAMES := ["日常", "主线", "宗门", "悬赏"]

var _lists: Array = []   # 每个页签一个 GdUIVList

# 任务数据（真实项目中来自 GdBean / 服务器，见 references/state-data.md）
# btn_state: go=前往 claim=可领取(金色) done=已完成(置灰)
var _tab_data := {
	"日常": [
		{"icon": "🌿", "title": "采集灵草", "desc": "在宗门后山采集灵草", "progress": "3/5", "reward1": "💎 100", "reward2": "🪙 50", "btn_text": "前往", "btn_state": "go"},
		{"icon": "🐺", "title": "击败妖狼", "desc": "击败宗门山林中的妖狼", "progress": "8/8", "reward1": "💎 150", "reward2": "🪙 75", "btn_text": "领取", "btn_state": "claim"},
		{"icon": "🏺", "title": "炼制聚气丹", "desc": "炼制聚气丹", "progress": "1/1", "reward1": "💎 120", "reward2": "🪙 60", "btn_text": "已完成", "btn_state": "done"},
		{"icon": "📜", "title": "传功授业", "desc": "为宗门弟子传功1次", "progress": "0/1", "reward1": "💎 80", "reward2": "🪙 40", "btn_text": "前往", "btn_state": "go"},
		{"icon": "🧰", "title": "捐献物资", "desc": "向宗门仓库捐献物资5次", "progress": "2/5", "reward1": "💎 100", "reward2": "🪙 50", "btn_text": "前往", "btn_state": "go"},
	],
	"主线": [
		{"icon": "🗡", "title": "初入宗门", "desc": "与执事长老对话", "progress": "1/1", "reward1": "💎 200", "reward2": "🪙 100", "btn_text": "领取", "btn_state": "claim"},
		{"icon": "🏔", "title": "后山历练", "desc": "在后山存活一炷香时间", "progress": "0/1", "reward1": "💎 300", "reward2": "🪙 150", "btn_text": "前往", "btn_state": "go"},
	],
	"宗门": [
		{"icon": "🥋", "title": "宗门大比", "desc": "报名参加季度大比", "progress": "0/1", "reward1": "💎 500", "reward2": "🪙 200", "btn_text": "前往", "btn_state": "go"},
		{"icon": "🏮", "title": "坊市采购", "desc": "为坊市补充3件货物", "progress": "1/3", "reward1": "💎 120", "reward2": "🪙 60", "btn_text": "前往", "btn_state": "go"},
	],
	"悬赏": [
		{"icon": "🏴", "title": "清剿山贼", "desc": "击退黑风寨山贼10名", "progress": "4/10", "reward1": "💎 400", "reward2": "🪙 180", "btn_text": "前往", "btn_state": "go"},
		{"icon": "🐎", "title": "护送商队", "desc": "护送商队至青州城", "progress": "0/1", "reward1": "💎 350", "reward2": "🪙 160", "btn_text": "前往", "btn_state": "go"},
	],
}


func _ready() -> void:
	# gml_file 属性已配置时 GdGmlScene 自动加载（含全部 <Gml> 引用）
	if not is_loaded():
		load_gml("res://example/ui/task/task_panel.gml")
	# 页签切换为原生行为，只连信号（当前引擎 TabContainer 仅支持顶部/底部页签，
	# 设计图中的左侧竖排页签如需还原，可换回自定义按钮列表 + 数据驱动高亮）
	var tabs: TabContainer = find_node("TaskTabs")
	if tabs:
		tabs.tab_changed.connect(_on_tab_changed)
	# 收集每个页签的列表实例（由 <Gml src="task_list.gml"> 构建期创建）
	for tab_name in TAB_NAMES:
		var page: Control = find_node(tab_name)
		if page == null:
			continue
		var list: GdUIVList = page.find_child("TaskList", true, false)
		if list:
			_lists.append(list)
	# 填充各页签的任务数据
	for i in range(_lists.size()):
		_refresh_list(i)


## 数据驱动刷新 —— {{key}} 模板绑定 + allbind 批量连接条目信号
func _refresh_list(idx: int) -> void:
	var list: GdUIVList = _lists[idx]
	var tasks: Array = _tab_data[TAB_NAMES[idx]]
	# force=false：保持 count<=0，走"按 data 长度动态增删条目"分支
	# （force=true 会把 count 固定为数据长度，首次更新时反而不会创建条目）
	list.update(tasks, false)
	# duplicate 出来的条目实例需要重新绑定内部按钮信号
	# path 为相对条目根节点的 NodePath，结构节点必须在 GML 中显式命名
	list.allbind_signal("ItemMargin/ItemRow/ItemBtn", "pressed", _on_task_action)
	# 条目状态着色（节点复用，先复位再按状态设置）
	for i in range(tasks.size()):
		var item: Control = list.get_at(i)
		if item == null:
			continue
		item.modulate = Color.WHITE
		var btn: Button = item.find_child("ItemBtn", true, false)
		if btn:
			btn.remove_theme_color_override("font_color")
		match tasks[i]["btn_state"]:
			"claim":
				if btn:
					btn.add_theme_color_override("font_color", Color("#ffd968"))
			"done":
				item.modulate = Color(1.0, 1.0, 1.0, 0.55)


# ---------- TabContainer 原生页签切换回调 ----------
func _on_tab_changed(tab: int) -> void:
	print("[TaskPanel] 切换页签: ", TAB_NAMES[tab])


# ---------- 任务条目按钮回调（task_item.gml，经 allbind 批量连接） ----------
func _on_task_action(btn: Control) -> void:
	# 沿父链找到携带 __item_data 的条目根节点
	var item: Node = btn
	while item and not item.has_meta("__item_data"):
		item = item.get_parent()
	if item and item.has_meta("__item_data"):
		var data: Dictionary = item.get_meta("__item_data")
		print("[TaskPanel] 点击任务按钮: ", data.get("title", "?"), " state=", data.get("btn_state", "?"))


# ---------- 顶部栏回调（task_topbar.gml 的 on_pressed） ----------
func _on_close_pressed() -> void:
	print("[TaskPanel] 关闭任务面板")
	hide()
