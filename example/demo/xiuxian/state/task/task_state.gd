# 任务状态 Bean（分类：任务）—— 仙途 demo 游戏状态层
#
# 职责：持有全量任务总表（不区分解锁状态，未解锁任务同样入库）+ 运行进度
#       （接取/推进/完成/提交）+ 奖励发放，属性自动经 GDCORE 持久化到
#       GJson 存档（user://coredata.data，
#       路径 init;xiuxian_task;tasks / init;xiuxian_task;progress;任务id;...）。
#
# 数据源：任务**定义**在 state/task/stage1.json / stage2.json（明文源，
#   pipeline=gjson 管线加密为同名 .gjson，经 XiuTaskTable 多文件合并加载；
#   改文件即改全游戏目录）；
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
const STATUS_COMPLETED := "completed"    # 已完成待提交（auto_claim=false 时的手动领取停靠态；默认自动领奖，仅瞬时经过）
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
## "done"=已完成 submitted 留档、"avail"=可接取任务目录（📍 找 giver 接取）；
## locked=前置未满足，不进任何页签——前置没完成后续任务不显示）
## 任务列表默认页签即"可接取"——玩家不用盲找 NPC，看目录就知道下一步找谁对话
var views: Array = []

## 总表中文分类 -> 列表页签 category id
const CATEGORY_IDS := {"日常": "daily", "主线": "main", "支线": "side",
	"宗门": "guild", "悬赏": "bounty"}
## 分类 id -> 列表图标
const CATEGORY_ICONS := {"daily": "📦", "main": "⚔️", "side": "🌿",
	"guild": "🏯", "bounty": "📜"}

## 上次任务奖励展示视图（complete_task 自动发奖后写入，领奖弹窗确认关闭时
## 清空——防跨运行恢复旧值导致重弹）。条目契约（奖励格子渲染 + 悬停 tips）：
## {icon: emoji 字符, icon_texture: res:// 贴图路径(可空), count: "x2" 角标,
##  name: 展示名, tips_title: tips 标题, tips: 用途说明}
var last_reward: Array = []

## 上次完成的任务名（领奖弹窗副标题，随 last_reward 一同写入/清空）
var last_reward_task: String = ""


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）。
## 注册后构建展示视图（重复调用重复重建，幂等无害；不覆盖 GdBean.on_ready——
## GDScript 视其为原生方法覆盖告警，本项目按错误处理）。
## 同时挂背包监听：collect(auto) 步在物品入包时自动推进（见 _poll_collect_steps）
static func ins() -> XiuTaskState:
	var b: XiuTaskState = GdBean.bean("xiuxian_task", func(): return new())
	b.refresh_views()
	b._bind_bag_watcher()
	return b


## 背包监听（collect 步自动推进的触发源）：
## 采集入库 / 剧情赠药（item_give）/ 购买 / 任务发奖全部经
## XiuItemState.add_item → update("bag")，watch 一次即覆盖全部入包路径。
## 红线：watch 回调内禁同步调其他 Bean（单借用重入 panic）→ call_deferred
var _bag_bound := false

func _bind_bag_watcher() -> void:
	if _bag_bound:
		return
	_bag_bound = true
	XiuItemState.ins().watch("bag", func(_v = null, _m = null): _poll_collect_steps.call_deferred())


## collect(auto) 步自动推进：遍历进行中任务，当前步 objective 为
## type=collect 且 auto=true 时，背包持有目标物品即 advance 一步。
## 幂等：advance 后步进离开该 collect 步，不会重复触发。
## 说明：仅 auto=true 生效——剧情赠药型 collect（如 main_105 聚气散）
## 由 timeline 在赠药选项里显式 task_advance，避免双重推进跳步。
func _poll_collect_steps() -> void:
	var items := XiuItemState.ins()
	for t in tasks.values():
		var tid := str(t.get("id", ""))
		if str(get_task_status(tid)) != STATUS_ACCEPTED:
			continue
		var step := int(get_progress(tid).get("step", 0))
		var objs: Array = t.get("objectives", [])
		if step >= objs.size():
			continue
		var obj: Dictionary = objs[step]
		if str(obj.get("type", "")) != "collect" or not bool(obj.get("auto", false)):
			continue
		var target := str(obj.get("target_role", ""))
		if not target.is_empty() and items.get_count(target) > 0:
			advance_task(tid)


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
		# 接取引导：目录里直接告诉玩家去哪找谁对话（giver 取自定义表）。
		# 仅 available 进目录——前置未满足（locked）不显示（用户规约：
		# 前置任务没完成，后续任务不该出现在列表里制造噪音）
		var giver := str(t.get("giver", ""))
		var hint := ""
		var desc := str(t.get("desc", ""))
		if status == STATUS_AVAILABLE:
			hint = "找「%s」接取" % giver
			desc = "📍 %s · %s" % [hint, desc]
		out.append({
			"id": tid,
			"category": cid,
			"icon": "🔒" if locked else CATEGORY_ICONS.get(cid, "📜"),
			"title": "%s·%s" % [cat, str(t.get("name", tid))],
			"desc": desc,
			"hint": hint,
			"progress": "%d/%d" % [step, total],
			"reward1": "💎 %d" % int(r.get("spirit_stones", 0)),
			"reward2": r2,
			"status": status,
			"status_group": _status_group(status),
			"btn_text": "未解锁" if locked else ("领取" if status == STATUS_COMPLETED \
					else ("✅ 已完成" if status == STATUS_SUBMITTED else \
					("去接取" if status == STATUS_AVAILABLE else "前往"))),
			"btn_state": "locked" if locked else ("reward" if status == STATUS_COMPLETED \
					else ("done" if status == STATUS_SUBMITTED else \
					("find" if status == STATUS_AVAILABLE else "go"))),
		})
	update("views", out, {}, false)


## 视图状态组（任务列表页签过滤）：
## accepted/completed → "active"（进行中），submitted → "done"（已完成留档），
## available → "avail"（可接取目录，带找 giver 接取指引）；
## locked → ""（前置未满足，不进任何页签——前置任务没完成，后续任务不显示）
func _status_group(status: String) -> String:
	if status == STATUS_ACCEPTED or status == STATUS_COMPLETED:
		return "active"
	if status == STATUS_SUBMITTED:
		return "done"
	return "avail" if status == STATUS_AVAILABLE else ""


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
	for id in tasks.keys():
		if get_task_status(id) != STATUS_LOCKED:
			out.append(tasks[id])
	return out


func get_locked_tasks() -> Array:
	var out: Array = []
	for id in tasks.keys():
		if get_task_status(id) == STATUS_LOCKED:
			out.append(tasks[id])
	return out


## 当前状态：进度无记录时按**前置链推导**（定义表不携带 unlock 字段——
## 解锁状态属运行时数据，随 progress 持久化：无前置=可用；有前置=前置
## 已提交才可用。外部闸门（日历/剧情）走 unlock_task 写入记录）
func get_task_status(task_id: String) -> String:
	var e: Dictionary = progress.get(task_id, {})
	if not e.is_empty():
		return str(e.get("status", STATUS_AVAILABLE))
	var t: Dictionary = get_task(task_id)
	var pre := str(t.get("precondition", ""))
	if pre.is_empty() or pre == task_id:
		return STATUS_AVAILABLE
	# 前置任务完成后 _unlock_successors 会写入 available 记录；
	# 这里兜底直接查前置状态（兼容旧存档无记录的情况）
	return STATUS_AVAILABLE if get_task_status(pre) == STATUS_SUBMITTED else STATUS_LOCKED


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


## 解锁任务：locked → available（日历/剧情到点触发，如第 7 天宗门大比）。
## 非 locked 状态调用无副作用返回 false。解锁记录写入 progress 持久化；
## 前置链任务的解锁由 _unlock_successors 自动完成，无需手动调
func unlock_task(task_id: String) -> bool:
	if get_task_status(task_id) != STATUS_LOCKED:
		return false
	set_task_status(task_id, STATUS_AVAILABLE)
	return true


## 推进一步：step+1，达到 steps 总数自动转 completed，返回新 step。
## 同步步级 flag task_<id>_step<N> 到 XiuDialogState（持久化）——
## 台词本用 [stage@task_xxx_step2] 门控「上一步已达成」的回报/交付段
## （如杀完鼠才能回报、采到药才能交付），避免玩家跳步提前回报
##
## 自动领奖：终步达成即 complete_task（发奖 + 领奖弹窗），玩家不必再跑
## 任务列表手动点领取。数据驱动开关：定义表 auto_claim=false 保留
## 「待提交」手动交付设计（如未来做 NPC 交付仪式的任务）；任务列表的
## 「领取」按钮分支保留，兜底 auto_claim=false 的任务。
## call_deferred：advance_task 常处于 watch 回调 / 对话选项上下文，
## complete_task 跨多个 Bean（角色/物品/对话），同步调用有重入风险（红线）
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
	XiuDialogState.ins().set_flag("task_%s_step%d" % [task_id, step])
	refresh_views()
	if e["status"] == STATUS_COMPLETED and bool(t.get("auto_claim", true)):
		complete_task.call_deferred(task_id)
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
## 发奖后构建 last_reward 展示视图——领奖弹窗（TaskRewardModal）watch 该键
## 自动弹出，业务侧只管调 complete_task，不碰 UI
func complete_task(task_id: String) -> bool:
	var st := get_task_status(task_id)
	if st != STATUS_ACCEPTED and st != STATUS_COMPLETED:
		return false
	set_task_status(task_id, STATUS_COMPLETED)
	_grant_rewards(task_id)
	_build_last_reward_view(task_id)
	set_task_status(task_id, STATUS_SUBMITTED)
	XiuDialogState.ins().set_flag("task_%s_done" % task_id)
	_unlock_successors(task_id)
	print("[XiuTaskState] 任务完成: %s %s" % [task_id, str(get_task(task_id).get("name", ""))])
	return true


## 前置链解锁：任务完成后，解锁所有 precondition 指向它的任务
## （locked → available；主线链 main_101→…→sect_tournament 按完成进度
## 自动点亮，对话/任务的 task_accept 才能接取；幂等——非 locked 调 unlock 无副作用）
func _unlock_successors(task_id: String) -> void:
	for t in tasks.values():
		var tid := str(t.get("id", ""))
		if str(t.get("precondition", "")) == task_id:
			unlock_task(tid)


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


## 构建 last_reward 展示视图（领奖弹窗数据源）。
## 资源类（修为/灵石/金币）直接给 emoji + 固定说明；道具类解析 XiuIconMap
## （icon=逻辑图片 id → emoji 或 res:// 贴图，照抄 item_state.refresh_bag_views
## 范式），tips = 道具 desc（用途说明，弹窗格子悬停展示）
func _build_last_reward_view(task_id: String) -> void:
	var t := get_task(task_id)
	var r: Dictionary = t.get("rewards", {})
	var out: Array = []
	if int(r.get("exp", 0)) > 0:
		out.append({
			"icon": "✨", "icon_texture": "", "count": "x%d" % int(r.exp),
			"name": "修为", "tips_title": "修为",
			"tips": "修炼积累的经验，达到瓶颈后可冲击更高境界。",
		})
	if int(r.get("spirit_stones", 0)) != 0:
		out.append({
			"icon": "💎", "icon_texture": "", "count": "x%d" % int(r.spirit_stones),
			"name": "灵石", "tips_title": "灵石",
			"tips": "修真界通用的硬通货，可在坊市购买丹药、法器与炼器材料。",
		})
	if int(r.get("coins", 0)) != 0:
		out.append({
			"icon": "🪙", "icon_texture": "", "count": "x%d" % int(r.coins),
			"name": "金币", "tips_title": "金币",
			"tips": "凡俗市集流通的钱币，用于日常采买与打赏。",
		})
	var item_bean := XiuItemState.ins()
	for it in r.get("items", []):
		var iid := str(it.get("id", ""))
		var n := int(it.get("count", 1))
		var def: Dictionary = item_bean.get_item(iid)
		var resolved := XiuIconMap.resolve(str(def.get("icon", "")))
		var icon := resolved
		var icon_tex := ""
		if XiuIconMap.is_texture(resolved):
			icon_tex = resolved
			icon = "📦"
		var desc := str(def.get("desc", ""))
		if desc.is_empty():
			desc = str(def.get("source", ""))
		if desc.is_empty():
			desc = "任务奖励道具。"
		out.append({
			"icon": icon, "icon_texture": icon_tex, "count": "x%d" % n,
			"name": str(def.get("name", iid)),
			"tips_title": str(def.get("name", iid)), "tips": desc,
		})
	last_reward_task = "%s·%s" % [str(t.get("category", "")), str(t.get("name", task_id))]
	# 红线：先改成员再 update 同值会被去重短路（GDScript Array 引用比较，
	# 成员赋值后 update 的 current==value 同引用恒等 → 不通知不落盘）。
	# 只走 update（内部 set 写回成员）；先任务名后数组——弹窗控制器 watch
	# last_reward（deferred 回调），触发时两个键必须都已就位
	update("last_reward_task", last_reward_task, {}, false)
	update("last_reward", out, {}, false)


## 清空领奖展示视图（弹窗确认关闭后调用；防跨运行恢复旧值导致重弹）
func clear_last_reward() -> void:
	# 同红线：不先行成员赋值，update 内部 set 写回（重复清空同值去重无害）
	last_reward_task = ""
	update("last_reward_task", "", {}, false)
	update("last_reward", [], {}, false)


## 还原演示基线（目录 + 进度清空；跨运行确定性测试用）。
## 同时清掉对话联动 flag（任务门控段落复原）与领奖展示视图（落盘清，
## 防旧档恢复重弹）
func reset_demo() -> void:
	update("tasks", XiuTaskTable.get_tasks(), {}, true)
	update("progress", {}, {}, true)
	clear_last_reward()
	refresh_views()
	XiuDialogState.ins().reset_demo()
