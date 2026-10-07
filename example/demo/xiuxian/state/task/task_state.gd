# 任务状态 Bean（分类：任务）—— 仙途 demo 游戏状态层
#
# 职责：持有全量任务总表（不区分解锁状态，未解锁任务同样入库）+ 运行进度，
#       属性自动经 GDCORE 持久化到 GJson 存档（user://coredata.data，
#       路径 init;xiuxian_task;tasks / init;xiuxian_task;progress;任务id;...）。
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

## 全量任务总表（静态目录：含未解锁任务；id -> 定义）
const DEFAULT_TASKS: Dictionary = {
	"main_001": {
		"id": "main_001", "name": "初入仙途", "category": "主线",
		"desc": "离开青石镇，前往青云宗报到。", "unlock": true,
		"precondition": "", "steps": 3,
		"rewards": {"exp": 400, "spirit_stones": 30,
			"items": [{"id": "herb_lingzhi", "count": 2}, {"id": "pill_hp", "count": 3}]},
	},
	"main_002": {
		"id": "main_002", "name": "灵田风波", "category": "主线",
		"desc": "调查灵田减产之谜，驱逐作乱的地灵鼠。", "unlock": true,
		"precondition": "main_001", "steps": 3,
		"rewards": {"exp": 600, "spirit_stones": 50,
			"items": [{"id": "ore_coldiron", "count": 3}]},
	},
	"main_003": {
		"id": "main_003", "name": "秘境探幽", "category": "主线",
		"desc": "持秘境残图进入落霞秘境，寻机上探三层。", "unlock": false,
		"precondition": "main_002", "steps": 5,
		"rewards": {"exp": 1000, "spirit_stones": 100,
			"items": [{"id": "pill_break", "count": 1}]},
	},
	"side_001": {
		"id": "side_001", "name": "采药济世", "category": "支线",
		"desc": "为回春堂采集血参十株，救治镇上疫病。", "unlock": true,
		"precondition": "", "steps": 2,
		"rewards": {"exp": 150, "spirit_stones": 20,
			"items": [{"id": "herb_xueshen", "count": 2}]},
	},
	"side_002": {
		"id": "side_002", "name": "铸剑问道", "category": "支线",
		"desc": "替欧冶氏收集寒铁，铸成一柄本命飞剑。", "unlock": false,
		"precondition": "side_001", "steps": 4,
		"rewards": {"exp": 300, "spirit_stones": 40,
			"items": [{"id": "sword_qingfeng", "count": 1}]},
	},
	"daily_001": {
		"id": "daily_001", "name": "每日签到", "category": "日常",
		"desc": "每日到掌门处签到，领取基本供奉。", "unlock": true,
		"precondition": "", "steps": 1,
		"rewards": {"exp": 80, "spirit_stones": 10,
			"items": [{"id": "pill_hp", "count": 1}]},
	},
	"guild_001": {
		"id": "guild_001", "name": "宗门巡逻", "category": "宗门",
		"desc": "巡视山门四处阵眼，清理滋扰妖兽。", "unlock": true,
		"precondition": "", "steps": 2,
		"rewards": {"exp": 200, "spirit_stones": 25,
			"items": [{"id": "pill_mp", "count": 2}]},
	},
	"bounty_001": {
		"id": "bounty_001", "name": "悬赏·血衣楼", "category": "悬赏",
		"desc": "追查血衣楼细作，取回失窃的宗门名录。", "unlock": false,
		"precondition": "guild_001", "steps": 3,
		"rewards": {"exp": 500, "spirit_stones": 80,
			"items": [{"id": "map_secret", "count": 1}]},
	},
}

## 全量任务总表（不区分解锁状态；首次注册写入存档，此后随存档恢复）
var tasks: Dictionary = DEFAULT_TASKS.duplicate(true)

## 运行进度（task_id -> {step: int, status: String}；仅已接取/有进展的任务有记录）
var progress: Dictionary = {}

## 展示视图（UIVList 模板契约字段：category/icon/title/desc/progress/
## reward1/reward2/btn_text/btn_state）——由 tasks+progress 派生，
## 进度/总表变化后调 refresh_views() 重建，UI 列表 data="bean:xiuxian_task:views"
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
		out.append({
			"id": tid,
			"category": cid,
			"icon": "🔒" if locked else CATEGORY_ICONS.get(cid, "📜"),
			"title": "%s·%s" % [cat, str(t.get("name", tid))],
			"desc": str(t.get("desc", "")),
			"progress": "%d/%d" % [step, total],
			"reward1": "💎 %d" % int(r.get("spirit_stones", 0)),
			"reward2": "✨ %d" % int(r.get("exp", 0)),
			"btn_text": "封印" if locked else ("领取" if status == STATUS_COMPLETED else "前往"),
			"btn_state": "locked" if locked else ("reward" if status == STATUS_COMPLETED else "go"),
		})
	update("views", out, {}, false)


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


## 还原演示基线（目录 + 进度清空；跨运行确定性测试用）
func reset_demo() -> void:
	update("tasks", DEFAULT_TASKS.duplicate(true), {}, true)
	update("progress", {}, {}, true)
	refresh_views()
