# 任务状态 Bean（分类：任务）—— 仙途 demo 游戏状态层
#
# 职责：持有全量任务总表（不区分解锁状态，未解锁任务同样入库）+ 运行进度
#       （接取/推进/完成/提交）+ 奖励发放，属性自动经 GDCORE 持久化到
#       GJson 存档（user://coredata.data，
#       路径 init;xiuxian_task;tasks / init;xiuxian_task;progress;任务id;...）。
#
# 数据源：任务**定义**在 state/task/task.json（明文源，pipeline=gjson 管线
#   加密为 task.gjson，经 XiuTaskTable 静态加载；改文件即改全游戏目录）；
#   本 Bean 只持有**运行数据**（首次注册从定义表派生总表写入存档 + 进度）。
#
# 对话联动：接取/完成时同步 flag（task_<id>_accepted / task_<id>_done）到
#   XiuDialogState（持久化）——台词本用 [stage@task_xxx_accepted] 门控
#   任务相关段落，任务结束后对应选项/对话自动消失。
#
# 查询方式（三选一）：
#   1. 便捷方法：get_task / get_all_tasks / get_tasks_by_category / get_tasks_by_status ...
#   2. Bean 路径查询：get_value_by_key("tasks;main_001;name")
#   3. GJson 直查（跨 Bean）：XiuGameState.query("xiuxian_task;tasks;main_003;name")
#
# 存档语义：tasks 总表首次写入后随存档恢复；进度 progress 跨运行保留。
# 测试/演示用 reset_demo() 还原基线（GdBean 属性会跨运行恢复旧数据）。
class_name XiuTaskState
extends GdBean

const STATUS_LOCKED := "locked"          # 未解锁（总表里有，进度链未达）
const STATUS_AVAILABLE := "available"    # 可接取
const STATUS_ACCEPTED := "accepted"      # 进行中
const STATUS_COMPLETED := "completed"    # 已完成待提交
const STATUS_SUBMITTED := "submitted"    # 已提交领奖

## 全量任务总表（静态目录：含未解锁任务；首次注册由 task.gjson 定义表派生，
## 此后随存档恢复；定义本身的权威来源是 task.json，重置经 XiuTaskTable 读取）
var tasks: Dictionary = XiuTaskTable.get_tasks()

## 运行进度（task_id -> {step: int, status: String}；仅已接取/有进展的任务有记录）
var progress: Dictionary = {}

## 展示视图（UIVList 模板契约字段：category/icon/title/desc/progress/
## reward1/reward2/btn_text/btn_state/status/status_group）——由 tasks+progress
## 派生，进度/总表变化后调 refresh_views() 重建，
## UI 列表 data="bean:xiuxian_task:views"。
## status_group：任务列表页签过滤字段（"active"=进行中 accepted/completed、
## "done"=已完成 submitted 留档、""=未接取 available/locked 不进列表）
var views: Array = []

## 总表中文分类 -> 列表页签 category id
const CATEGORY_IDS := {"日常": "daily", "主线": "main", "支线": "side",
	"宗门": "guild", "悬赏": "bounty"}
## 分类 id -> 列表图标
const CATEGORY_ICONS := {"daily": "📦", "main": "⚔️", "side": "🌿",
	"guild": "🏯", "bounty": "📜"}


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）。
## 注册后构建展示视图（重复调用重复重建，幂等无害；不覆盖 GdBean.on_ready——
## GDScript 视其为原生方法覆盖告警，本项目按错误处理）
static func ins() -> XiuTaskState:
	var b: XiuTaskState = GdBean.bean("xiuxian_task", func(): return new())
	b.refresh_views()
	return b


## 重建展示视图并通知所有监听列表（进度/总表变化后调用）
func refresh_views() -> void:
	var out: Array = []
	for t in tasks.values():
		var tid := str(t.get("id", ""))
		var cat := str(t.get("category", ""))
		var cid: String = CATEGORY_IDS.get(cat, "main")
		var status := get_task_status(tid)
		var p: Dictionary = get_progress(tid)
		var step := int(p.get("step", 0))
		var total := int(t.get("steps", 1))
		var r: Dictionary = t.get("rewards", {})
		var locked := status == STATUS_LOCKED
		var r2 := "✨ %d" % int(r.get("exp", 0)) if int(r.get("exp", 0)) > 0 \
				else ("🪙 %d" % int(r.get("coins", 0)) if int(r.get("coins", 0)) > 0 else "")
		out.append({
			"id": tid,
			"category": cid,
			"icon": "🔒" if locked else CATEGORY_ICONS.get(cid, "📜"),
			"title": "%s·%s" % [cat, str(t.get("name", tid))],
			"desc": str(t.get("desc", "")),
			"progress": "%d/%d" % [step, total],
			"reward1": "💎 %d" % int(r.get("spirit_stones", 0)),
			"reward2": r2,
			"status": status,
			"status_group": _status_group(status),
			"btn_text": "封印" if locked else ("领取" if status == STATUS_COMPLETED \
					else ("✅ 已完成" if status == STATUS_SUBMITTED else "前往")),
			"btn_state": "locked" if locked else ("reward" if status == STATUS_COMPLETED \
					else ("done" if status == STATUS_SUBMITTED else "go")),
		})
	update("views", out, {}, false)


## 视图状态组（任务列表页签过滤）：
## accepted/completed → "active"（进行中），submitted → "done"（已完成留档），
## available/locked → ""（未接取，不进任务列表）
func _status_group(status: String) -> String:
	if status == STATUS_ACCEPTED or status == STATUS_COMPLETED:
		return "active"
	return "done" if status == STATUS_SUBMITTED else ""


# ------------------------------------------------------------------ 查询

## 按 id 取任务定义（无则返回空 Dictionary）
func get_task(task_id: String) -> Dictionary:
	return tasks.get(task_id, {})


## 全量任务列表（含未解锁——总表不区分解锁状态）
func get_all_tasks() -> Array:
	return tasks.values()


func get_task_ids() -> Array:
	return tasks.keys()


## 按分类查询（主线/支线/日常/宗门/悬赏），含未解锁
func get_tasks_by_category(category: String) -> Array:
	var out: Array = []
	for t in tasks.values():
		if str(t.get("category", "")) == category:
			out.append(t)
	return out


func get_unlocked_tasks() -> Array:
	var out: Array = []
	for t in tasks.values():
		if bool(t.get("unlock", false)):
			out.append(t)
	return out


func get_locked_tasks() -> Array:
	var out: Array = []
	for t in tasks.values():
		if not bool(t.get("unlock", false)):
			out.append(t)
	return out


## 当前状态：进度无记录时按总表 unlock 推导（locked / available）
func get_task_status(task_id: String) -> String:
	var e: Dictionary = progress.get(task_id, {})
	if not e.is_empty():
		return str(e.get("status", STATUS_AVAILABLE))
	var t: Dictionary = get_task(task_id)
	return STATUS_AVAILABLE if bool(t.get("unlock", false)) else STATUS_LOCKED


## 按状态查询（accepted/completed/...，不含 locked——locked 是"无记录"态）
func get_tasks_by_status(status: String) -> Array:
	var out: Array = []
	for id in tasks.keys():
		if get_task_status(id) == status:
			out.append(tasks[id])
	return out


func get_progress(task_id: String) -> Dictionary:
	return progress.get(task_id, {})


# ------------------------------------------------------------------ 变更

func set_task_status(task_id: String, status: String) -> void:
	var p: Dictionary = progress.duplicate(true)
	var e: Dictionary = p.get(task_id, {"step": 0})
	e["status"] = status
	p[task_id] = e
	update("progress", p, {}, false)
	refresh_views()


## 推进一步：step+1，达到 steps 总数自动转 completed，返回新 step
func advance_task(task_id: String) -> int:
	var t: Dictionary = get_task(task_id)
	var p: Dictionary = progress.duplicate(true)
	var e: Dictionary = p.get(task_id, {"step": 0, "status": STATUS_ACCEPTED})
	var step: int = int(e.get("step", 0)) + 1
	e["step"] = step
	var total: int = int(t.get("steps", 1))
	e["status"] = STATUS_COMPLETED if step >= total else STATUS_ACCEPTED
	p[task_id] = e
	update("progress", p, {}, false)
	refresh_views()
	return step


## 接取任务：available → accepted；成功返回 true
## （同步 flag task_<id>_accepted 到 XiuDialogState，供台词本门控）
func accept_task(task_id: String) -> bool:
	if get_task_status(task_id) != STATUS_AVAILABLE:
		return false
	set_task_status(task_id, STATUS_ACCEPTED)
	XiuDialogState.ins().set_flag("task_%s_accepted" % task_id)
	return true


## 完成任务并立即发放奖励（coins→金币 / exp→修为 / spirit_stones→灵石 /
## items→道具入包），状态直达 submitted；可从 accepted/completed 调用。
## （同步 flag task_<id>_done 到 XiuDialogState，台词本的任务段落随之关闭）
func complete_task(task_id: String) -> bool:
	var st := get_task_status(task_id)
	if st != STATUS_ACCEPTED and st != STATUS_COMPLETED:
		return false
	set_task_status(task_id, STATUS_COMPLETED)
	_grant_rewards(task_id)
	set_task_status(task_id, STATUS_SUBMITTED)
	XiuDialogState.ins().set_flag("task_%s_done" % task_id)
	print("[XiuTaskState] 任务完成: %s %s" % [task_id, str(get_task(task_id).get("name", ""))])
	return true


## 发放任务奖励（各资源路由到对应 Bean）
func _grant_rewards(task_id: String) -> void:
	var r: Dictionary = get_task(task_id).get("rewards", {})
	var ch := XiuCharacterState.ins()
	if int(r.get("exp", 0)) > 0:
		ch.add_exp(int(r.exp))
	if int(r.get("spirit_stones", 0)) != 0:
		ch.add_spirit_stones(int(r.spirit_stones))
	if int(r.get("coins", 0)) != 0:
		ch.add_coins(int(r.coins))
	var item_bean := XiuItemState.ins()
	for it in r.get("items", []):
		item_bean.add_item(str(it.get("id", "")), int(it.get("count", 1)))


## 还原演示基线（目录 + 进度清空；跨运行确定性测试用）。
## 同时清掉对话联动 flag（任务门控段落复原）
func reset_demo() -> void:
	update("tasks", XiuTaskTable.get_tasks(), {}, true)
	update("progress", {}, {}, true)
	refresh_views()
	XiuDialogState.ins().reset_demo()
