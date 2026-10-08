# 游戏状态总控门面 —— 仙途 demo 状态层入口（state/ 目录）
#
# 职责：
#   1. 统一注册/持有各分类状态 Bean（任务/道具/人物状态，见子目录）；
#   2. 跨分类联动（如任务奖励发放：道具入库 + 经验/灵石入人物）；
#   3. GJson 直查：绕过 Bean 便捷方法，直接按路径查询 GDCORE 存档 JSON
#      （路径格式 "bean_id;字段;子键"，scope 为 init）。
#
# 用法：
#   var gs := XiuGameState.ins()
#   gs.task.get_task("main_001")
#   gs.item.add_item("pill_hp", 3)
#   gs.character.add_exp(100)
#   XiuGameState.query("xiuxian_task;tasks;main_003;name")   # 未解锁任务同样可查
#   XiuGameState.dump_json()                                  # 整库存档 JSON
class_name XiuGameState
extends RefCounted
static var _inst: RefCounted

## 分类 Bean（task/item/character，类型见各 preload 脚本）
var task
var item
var character


## 单例门面（分类 Bean 经 GdBean.bean 幂等注册，跨调用稳定）
static func ins() -> RefCounted:
	if _inst == null:
		_inst = new()
	return _inst


func _init() -> void:
	task = XiuTaskState.ins()
	item = XiuItemState.ins()
	character = XiuCharacterState.ins()


# ------------------------------------------------------------------ GJson 直查

## 按 GJson 路径直查 GDCORE 存档（"bean_id;字段;子键"，未命中返回 default_v）。
## 例：query("xiuxian_task;tasks;main_003;name") → "秘境探幽"（未解锁也可查）
static func query(path: String, default_v: Variant = null) -> Variant:
	var core: Object = Engine.get_singleton("GDCORE")
	if core == null:
		return default_v
	var root: Resource = core.get_root_data()
	if root == null:
		return default_v
	return root.value(path, default_v, "init")


## 整库存档 JSON 字符串（GJson duplicate_all_string，调试/存档预览用）
static func dump_json() -> String:
	var core: Object = Engine.get_singleton("GDCORE")
	if core == null:
		return ""
	var root: Resource = core.get_root_data()
	if root == null:
		return ""
	return root.duplicate_all_string()


# ------------------------------------------------------------------ 跨分类联动

## 提交任务并发放奖励：道具入库 + 经验（含升级）/灵石入人物，
## 任务状态置 submitted。返回发放摘要（task/items/exp/spirit_stones/levels）。
func apply_task_rewards(task_id: String) -> Dictionary:
	var t: Dictionary = task.get_task(task_id)
	if t.is_empty():
		return {}
	var rewards: Dictionary = t.get("rewards", {})
	var summary: Dictionary = {"task": task_id, "items": [], "exp": 0,
		"spirit_stones": 0, "levels": 0}
	for it in rewards.get("items", []):
		var count := int(it.get("count", 1))
		item.add_item(str(it["id"]), count)
		summary["items"].append("%s x%d" % [it["id"], count])
	var exp_gain := int(rewards.get("exp", 0))
	if exp_gain > 0:
		summary["exp"] = exp_gain
		summary["levels"] = character.add_exp(exp_gain)
	var stones := int(rewards.get("spirit_stones", 0))
	if stones != 0:
		character.add_spirit_stones(stones)
		summary["spirit_stones"] = stones
	task.set_task_status(task_id, XiuTaskState.STATUS_SUBMITTED)
	return summary


## 还原全部演示基线（跨运行确定性测试用）
func reset_demo() -> void:
	task.reset_demo()
	item.reset_demo()
	character.reset_demo()
