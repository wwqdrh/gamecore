# 人物状态 Bean（分类：人物状态）—— 仙途 demo 游戏状态层
#
# 职责：主角养成状态（等级/经验/气血/灵力/灵石/属性/境界），
#       属性自动经 GDCORE 持久化到 GJson 存档
#       （user://coredata.data，路径 init;xiuxian_character;level 等）。
#
# 查询方式（三选一）：
#   1. 便捷方法：get_realm / get_attr / exp_to_next ...
#   2. Bean 路径查询：get_value_by_key("attrs;悟性")
#   3. GJson 直查（跨 Bean）：XiuGameState.query("xiuxian_character;level")
class_name XiuCharacterState
extends GdBean

## 境界表：每 5 级进阶一境（练气 → 炼虚）
const REALMS: Array = ["练气", "筑基", "金丹", "元婴", "化神", "炼虚"]
const REALM_STEP := 5
## 境界层数中文数字
const CHINESE_NUM: Array = ["一", "二", "三", "四", "五"]

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
var attrs: Dictionary = DEFAULT_ATTRS.duplicate(true)


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuCharacterState:
	return GdBean.bean("xiuxian_character", func(): return new())


# ------------------------------------------------------------------ 查询

## 升到下一级所需经验（线性成长：level * 100）
func exp_to_next(lv: int) -> int:
	return lv * 100


## 当前境界（每 REALM_STEP 级进阶，超出表尾取最后一级）
func get_realm() -> String:
	var idx: int = (level - 1) / REALM_STEP
	idx = clampi(idx, 0, REALMS.size() - 1)
	return REALMS[idx]


func get_attr(key: String) -> int:
	return int(attrs.get(key, 0))


func get_attr_names() -> Array:
	return attrs.keys()


## 指定等级的境界显示名（境界 + 层数，如 "练气三层"）
func get_realm_title_at(lv: int) -> String:
	var idx: int = (lv - 1) / REALM_STEP
	idx = clampi(idx, 0, REALMS.size() - 1)
	var layer: int = (lv - 1) % REALM_STEP
	return "%s%s层" % [REALMS[idx], CHINESE_NUM[layer]]


## 当前境界显示名
func get_realm_title() -> String:
	return get_realm_title_at(level)


## 当前层经验进度 0..1（经验条/突破圆环共用）
func get_exp_progress() -> float:
	var need := exp_to_next(level)
	return 0.0 if need <= 0 else clampf(float(exp) / float(need), 0.0, 1.0)


## 战斗属性派生（数据侧计算，UI 直接展示）：攻击/防御由 体魄/根骨 加权
func get_attack() -> int:
	return int(attrs.get("体魄", 0)) * 12 + int(attrs.get("根骨", 0)) * 4


func get_defense() -> int:
	return int(attrs.get("体魄", 0)) * 6 + int(attrs.get("根骨", 0)) * 12


# ------------------------------------------------------------------ 变更

## 增加经验并处理连续升级；返回升到的级数。
## 升级时自动刷新满血满灵（上限随等级成长）并通知监听者。
## 注意：GdBean.update(key, 同值, force=false) 会因值相等提前返回（不落盘、
## 不通知），所以必须先在局部变量算出新值再 update，严禁先改成员再 update 同值。
func add_exp(v: int) -> int:
	var gained := 0
	var lv := level
	var e := exp + v
	while e >= exp_to_next(lv):
		e -= exp_to_next(lv)
		lv += 1
		gained += 1
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
	update("attrs", DEFAULT_ATTRS.duplicate(true), {}, true)
