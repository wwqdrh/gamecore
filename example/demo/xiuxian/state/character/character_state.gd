# 人物状态 Bean（分类：人物状态）—— 仙途 demo 游戏状态层
#
# 职责：主角养成状态（等级/经验/气血/灵力/灵石/属性/境界），
#       属性自动经 GDCORE 持久化到 GJson 存档
#       （user://coredata.data，路径 init;xiuxian_character;level 等）。
#
# 经验体系（定义在 state/level/level.gjson，经 XiuLevelTable 静态查询）：
#   - 11 阶（斗之气→斗帝）× 每阶 10 段，全局 1..110 级
#   - 阶内升级：经验足够即升（levels[i] = 阶内第 i+1 段所需）
#   - 阶间突破：10 段经验攒满 + 突破道具齐备（try_breakthrough 消耗背包），
#     突破后进入下一阶一段、经验清零；阶满时经验自动封顶
#
# 查询方式（三选一）：
#   1. 便捷方法：get_realm / get_attr / exp_to_next / can_breakthrough ...
#   2. Bean 路径查询：get_value_by_key("attrs;悟性")
#   3. GJson 直查（跨 Bean）：XiuGameState.query("xiuxian_character;level")
class_name XiuCharacterState
extends GdBean

## 五维属性默认值（键即 GJson 字段）
const DEFAULT_ATTRS: Dictionary = {"体魄": 5, "灵识": 7, "根骨": 6, "悟性": 8}

var char_name: String = "云无尘"
var level: int = 1
var exp: int = 0
var hp: int = 100
var max_hp: int = 100
var mp: int = 50
var max_mp: int = 50
var spirit_stones: int = 20
## 金币（市井货币：镇上消费/任务奖励；与修真界灵石区分）
var coins: int = 0
var attrs: Dictionary = DEFAULT_ATTRS.duplicate(true)


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuCharacterState:
	return GdBean.bean("xiuxian_character", func(): return new())


# ------------------------------------------------------------------ 查询

## 升到下一级所需经验（查 level.gjson；顶阶满级返回 0）
func exp_to_next(lv: int) -> int:
	return XiuLevelTable.get_level_exp(lv)


## 全局等级上限（110）
func get_max_level() -> int:
	return XiuLevelTable.get_max_level()


## 是否处于「阶满待突破」（阶内 10 段且非顶阶满级）
func is_rank_full() -> bool:
	return XiuLevelTable.is_rank_full(level)


## 当前阶 id（dou_zhiqi..dou_di）
func get_rank_id() -> String:
	return str(XiuLevelTable.get_rank_at(XiuLevelTable.rank_order_of(level)).get("id", ""))


## 当前境界（阶名，如 "斗者"）
func get_realm() -> String:
	return str(XiuLevelTable.get_rank_at(XiuLevelTable.rank_order_of(level)).get("name", ""))


## 指定等级的境界显示名（阶名 + 段位，如 "斗者七段"；顶阶满级 "斗帝圆满"）
func get_realm_title_at(lv: int) -> String:
	return XiuLevelTable.get_rank_title_at(lv)


## 当前境界显示名
func get_realm_title() -> String:
	return get_realm_title_at(level)


## 当前层经验进度 0..1（经验条/突破圆环共用）
func get_exp_progress() -> float:
	var need := exp_to_next(level)
	return 0.0 if need <= 0 else clampf(float(exp) / float(need), 0.0, 1.0)


func get_attr(key: String) -> int:
	return int(attrs.get(key, 0))


func get_attr_names() -> Array:
	return attrs.keys()


## 战斗属性派生（数据侧计算，UI 直接展示）：攻击/防御由 体魄/根骨 加权
func get_attack() -> int:
	return int(attrs.get("体魄", 0)) * 12 + int(attrs.get("根骨", 0)) * 4


func get_defense() -> int:
	return int(attrs.get("体魄", 0)) * 6 + int(attrs.get("根骨", 0)) * 12


# ------------------------------------------------------------------ 突破（阶间）

## 突破经验条件是否满足（阶满 10 段且经验攒满）
func is_breakthrough_exp_ready() -> bool:
	return is_rank_full() and exp >= exp_to_next(level)


## 当前突破材料清单（[{定义字段..., need: 需求数, have: 背包持有}, ...]；
## 数据来自 level.gjson breakthrough × item.gjson 定义）
func get_breakthrough_materials() -> Array:
	var items: Array = XiuLevelTable.get_breakthrough_items_at(level)
	var item_bean := XiuItemState.ins()
	var out: Array = []
	for req in items:
		var id := str(req.get("id", ""))
		var def := item_bean.get_item(id)
		var d: Dictionary = def.duplicate(true)
		d["need"] = int(req.get("count", 1))
		d["have"] = item_bean.get_count(id)
		out.append(d)
	return out


## 突破材料是否齐备（阶满前恒 false）
func can_breakthrough() -> bool:
	if not is_breakthrough_exp_ready():
		return false
	for m in get_breakthrough_materials():
		if int(m.get("have", 0)) < int(m.get("need", 0)):
			return false
	return true


## 尝试等阶突破：经验条件 + 材料齐备 → 扣除背包材料，进入下一阶一段
## （经验清零），返回是否成功。材料经 XiuItemState 扣减（跨 Bean 联动）。
func try_breakthrough() -> bool:
	if not can_breakthrough():
		return false
	var item_bean := XiuItemState.ins()
	for req in XiuLevelTable.get_breakthrough_items_at(level):
		if not item_bean.remove_item(str(req.get("id", "")), int(req.get("count", 1))):
			return false
	var nlv := level + 1
	var nhp := 100 + (nlv - 1) * 20
	var nmp := 50 + (nlv - 1) * 10
	update("level", nlv, {}, false)
	update("exp", 0, {}, false)
	update("max_hp", nhp, {}, false)
	update("hp", nhp, {}, false)
	update("max_mp", nmp, {}, false)
	update("mp", nmp, {}, false)
	return true


# ------------------------------------------------------------------ 变更

## 增加经验并处理阶内连续升级；返回升到的级数。
## 阶内经验足够即升级；阶满（10 段）时经验封顶，不再自动升级——
## 等待等阶突破（try_breakthrough，需道具）。
## 升级时自动刷新满血满灵（上限随等级成长）并通知监听者。
## 注意：GdBean.update(key, 同值, force=false) 会因值相等提前返回（不落盘、
## 不通知），所以必须先在局部变量算出新值再 update，严禁先改成员再 update 同值。
func add_exp(v: int) -> int:
	var gained := 0
	var lv := level
	var e := exp + v
	while lv < get_max_level() and not XiuLevelTable.is_rank_full(lv) \
			and e >= exp_to_next(lv):
		e -= exp_to_next(lv)
		lv += 1
		gained += 1
	# 阶满待突破 / 满级：经验封顶（不无限堆积）
	if lv >= get_max_level() or XiuLevelTable.is_rank_full(lv):
		var need := exp_to_next(lv)
		if need > 0:
			e = mini(e, need)
	if gained > 0:
		var nhp := 100 + (lv - 1) * 20
		var nmp := 50 + (lv - 1) * 10
		update("level", lv, {}, false)
		update("max_hp", nhp, {}, false)
		update("hp", nhp, {}, false)
		update("max_mp", nmp, {}, false)
		update("mp", nmp, {}, false)
	update("exp", e, {}, false)
	return gained


func add_spirit_stones(v: int) -> void:
	update("spirit_stones", spirit_stones + v, {}, false)


## 增加金币（任务奖励等）；返回持有量
func add_coins(v: int) -> int:
	update("coins", coins + v, {}, false)
	return coins


## 消耗灵石；不足时失败返回 false（不产生负数）
func spend_spirit_stones(v: int) -> bool:
	if spirit_stones < v:
		return false
	add_spirit_stones(-v)
	return true


func set_attr(key: String, v: int) -> void:
	var a: Dictionary = attrs.duplicate(true)
	a[key] = v
	update("attrs", a, {}, false)


## 还原演示基线（养成数值还原默认值；跨运行确定性测试用）
func reset_demo() -> void:
	update("level", 1, {}, true)
	update("exp", 0, {}, true)
	update("hp", 100, {}, true)
	update("max_hp", 100, {}, true)
	update("mp", 50, {}, true)
	update("max_mp", 50, {}, true)
	update("spirit_stones", 20, {}, true)
	update("coins", 0, {}, true)
	update("attrs", DEFAULT_ATTRS.duplicate(true), {}, true)
