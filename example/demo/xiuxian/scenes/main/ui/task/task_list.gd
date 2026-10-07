# 任务列表组件控制器 —— 由 task_list.gml 的 <ui script="task_list.gd"> 自动挂载
# 到内容根节点（ScrollContainer）。
#
# 职责：注册 GdBean 可监听数据源（bean id: task_list），并每 5s 轮换分类插入
# 一条新任务，模拟数据变化 → 绑定该 Bean 的所有列表自动刷新
# （各实例列表经 filter_key/filter_value_var 只显示自己分类的任务）。
#
# 数据绑定链路：
#   task_list.gml  <UIVList data="bean:task_list:tasks"
#                          filter_key="category" filter_value_var="category">
#   声明式绑定 Bean 全量任务 + 分类过滤声明；分类值由引用方
#   <Gml src="task_list.gml" data-category="分类变量"> 具名映射注入。
#   GdGmlScene 场景加载后 auto_bind_data 自动完成：初始填充 + watch 注册，
#   Bean 属性 emit 变化时更新全部同名 TaskList 节点（各列表内部按分类过滤）。
#   独立打开 task_list.gml.tscn（无场景脚本）时本脚本兜底绑定。
extends ScrollContainer

const INSERT_INTERVAL := 5.0

# 动态插入任务的分类轮换序列（演示各页签独立增长）
const INSERT_CATEGORIES := ["daily", "main", "guild", "bounty"]
const CATEGORY_NAMES := {
	"daily": "日常",
	"main": "主线",
	"guild": "宗门",
	"bounty": "悬赏",
}

# Bean 数据类：tasks 数组即任务列表数据源（watch/emit 监听其变化）
class TaskBean:
	extends GdBean
	var tasks: Array = []

# 全部实例共享同一 Bean，只有存活的第一个实例启动定时器驱动数据
static var _driver: ScrollContainer

var _bean: GdBean
var _default_tasks: Array = []
var _insert_seq := 0


func _ready() -> void:
	_bean = GdBean.bean("task_list", _create_bean)
	# 演示语义：每次进入场景从 GML 默认数据重新开始（GdBean 属性会经
	# GDCORE 存档持久化，跨运行恢复旧数据；真实项目按需保留存档）
	if not _default_tasks.is_empty():
		_bean.set("tasks", _default_tasks.duplicate(true))
	# 延迟一帧再兜底绑定：GdGmlScene 场景的 auto_bind_data 在加载流程中
	# 同步执行（初始填充 + watch），届时列表已有条目则无需重复绑定
	_setup.call_deferred()


## Bean 工厂：初始数据取自 task_list.gml <script> 的 tasks 变量
## （挂在本文件内容根节点的 __script_vars meta 上，数据单一样本来源）
func _create_bean() -> TaskBean:
	var b := TaskBean.new()
	var vars: Variant = get_meta("__script_vars", {})
	if vars is Dictionary and vars.has("tasks"):
		b.tasks = vars["tasks"]
		_default_tasks = vars["tasks"]
	return b


func _setup() -> void:
	var list: Control = find_child("TaskList", true, false)
	if list == null:
		return
	# 列表无条目（index 0 是 slot 模板）→ 无场景脚本接管，本脚本自行绑定
	# 注意：首次填充必须用 update(data, false)——force=true 在列表为空时
	# 增删分支都不命中，一个条目都不会创建（框架已知坑）
	if list.get_child_count() <= 1:
		list.update(_bean.get_value_by_key("tasks"), false)
		_bean.watch("tasks", _on_tasks_changed)
	# 数据驱动定时器：仅存活的第一个实例持有
	if _driver == null or not is_instance_valid(_driver):
		_driver = self
		var timer := Timer.new()
		timer.wait_time = INSERT_INTERVAL
		timer.autostart = true
		timer.timeout.connect(_on_insert_tick)
		add_child(timer)


## Bean 兜底绑定的变化回调（GdGmlScene 场景下由场景的 watch 接管）
## 同样用 update(value, false)：count == 数据长度时 force=true 不会重建条目
func _on_tasks_changed(value: Variant, _metas: Variant) -> void:
	var list: Control = find_child("TaskList", true, false)
	if list:
		list.update(value, false)


## 每 5s 轮换分类插入一条新任务并通知所有监听者
## （各页签列表按 filter_key=category 过滤，只有分类匹配的页签条目增长）
func _on_insert_tick() -> void:
	var data: Array = _bean.get_value_by_key("tasks")
	var seq := data.size() + 1
	var category: String = INSERT_CATEGORIES[_insert_seq % INSERT_CATEGORIES.size()]
	_insert_seq += 1
	data.append({
		category = category,
		icon = "🆕",
		title = "%s·动态任务 %d" % [CATEGORY_NAMES.get(category, category), seq],
		desc = "每 %d 秒由 GdBean 自动插入" % int(INSERT_INTERVAL),
		progress = "0/1",
		reward1 = "💎 %d" % seq,
		reward2 = "🪙 3",
		btn_text = "前往",
		btn_state = "go",
	})
	_bean.emit(["tasks"])
