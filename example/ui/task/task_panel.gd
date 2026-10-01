# 宗门任务面板 —— 多 GML 文件组合示例（对照设计图用简单图形/emoji 占位）
#
# 拆分策略（复用框架组件）：
#   task_panel.gml    骨架：整体布局 + Slot 占位 + 标题横幅
#   task_topbar.gml   顶部资源栏 + 关闭按钮
#   task_tabs.gml     TabContainer/Tab 原生页签切换（4 个页签）
#   task_list.gml     任务列表容器（ScrollContainer + UIVList，每个页签挂一个实例）
#   task_item.gml     单个任务条目（作为 UIVList 的 slot 模板注入）
#   task_activity.gml 底部活跃度进度 + 宝箱里程碑
#
# 组合方式（三种典型模式各一）：
#   1. Slot 挂载    ：GdUiBuilder.parse_file + find_node(slot).add_child
#   2. 列表模板注入 ：条目 gml 运行时塞进 UIVList 作为 slot 模板
#   3. 数据驱动刷新 ：list.update(data) + {{key}} 模板绑定 + allbind 批量连信号
extends GdGmlScene

const DIR := "res://example/ui/task/"
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
	# 1. 加载骨架（Slot 占位布局）
	load_gml(DIR + "task_panel.gml")
	# 2. 挂载各功能区块
	_mount("TopBarSlot", "task_topbar.gml")
	_mount("TabSlot", "task_tabs.gml")
	_mount("ActivitySlot", "task_activity.gml")
	# 3. 每个页签挂载一个 task_list 实例并注入条目模板
	_setup_tab_lists()
	# 4. 填充各页签的任务数据
	for i in range(TAB_NAMES.size()):
		_refresh_list(i)


## 解析子 gml 并剥掉 UiRoot 包装层，返回真正的根控件。
## 包装层是普通 Control（无尺寸语义）：
##   - 作为列表 slot 模板时，条目最小高度无法向上传递（高度塌陷为 0）
##   - 其内部未命名节点是 @Class@id 形式，NodePath 无法稳定命中
## 因此组合场景统一剥壳后使用。
func _load_root(gml_name: String) -> Control:
	var builder := GdUiBuilder.new()
	builder.set_theme("cartoon")
	var wrapper := builder.parse_file(DIR + gml_name)
	var root: Control = wrapper.get_child(0)
	wrapper.remove_child(root)
	wrapper.free()
	builder.connect_signals(root, self)   # 子视图信号连到本脚本
	return root


## 模式一：Slot 挂载 —— 把子视图放进骨架的占位节点
func _mount(slot_name: String, gml_name: String) -> void:
	var view := _load_root(gml_name)
	var slot := find_node(slot_name)
	if slot:
		slot.add_child(view)


## 页签 + 列表装配：每个 Tab 页挂一个 task_list 实例（模式二：列表模板注入）
func _setup_tab_lists() -> void:
	var tabs: TabContainer = find_node("TaskTabs")
	if tabs == null:
		return
	# 页签切换为原生行为，只连信号（当前引擎 TabContainer 仅支持顶部/底部页签，
	# 设计图中的左侧竖排页签如需还原，可换回自定义按钮列表 + 数据驱动高亮）
	tabs.tab_changed.connect(_on_tab_changed)
	for i in range(TAB_NAMES.size()):
		var page := find_node(TAB_NAMES[i])
		if page == null:
			continue
		var list_view := _load_root("task_list.gml")
		page.add_child(list_view)
		var list: GdUIVList = list_view.find_child("TaskList", true, false)
		var tpl := _load_root("task_item.gml")
		list.add_child(tpl)       # 第一个子节点 = slot 模板
		list.initial()
		_lists.append(list)


## 模式三：数据驱动刷新 —— {{key}} 模板绑定 + allbind 批量连接条目信号
func _refresh_list(idx: int) -> void:
	var list: GdUIVList = _lists[idx]
	var tasks: Array = _tab_data[TAB_NAMES[idx]]
	# force=false：保持 count<=0，走“按 data 长度动态增删条目”分支
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
