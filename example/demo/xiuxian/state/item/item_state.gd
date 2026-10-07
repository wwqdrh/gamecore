# 道具状态 Bean（分类：道具）—— 仙途 demo 游戏状态层
#
# 职责：持有全量道具总表（不区分解锁状态，未解锁/未产出道具同样入库）
#       + 背包持有量，属性自动经 GDCORE 持久化到 GJson 存档
#       （user://coredata.data，路径 init;xiuxian_item;items / init;xiuxian_item;bag）。
#
# 查询方式（三选一）：
#   1. 便捷方法：get_item / get_all_items / get_items_by_type / get_items_by_rarity / search_items
#   2. Bean 路径查询：get_value_by_key("bag;pill_hp") / get_value_by_key("items;pill_hp;name")
#   3. GJson 直查（跨 Bean）：XiuGameState.query("xiuxian_item;bag;pill_hp")
class_name XiuItemState
extends GdBean

const TYPE_CONSUME := "消耗"
const TYPE_MATERIAL := "材料"
const TYPE_EQUIP := "装备"
const TYPE_SPECIAL := "特殊"

## 全量道具总表（静态目录：含未解锁；id -> 定义）
const DEFAULT_ITEMS: Dictionary = {
	"pill_hp": {
		"id": "pill_hp", "name": "回春丹", "type": TYPE_CONSUME, "rarity": "凡品",
		"desc": "服用后恢复少量气血。", "price": 10, "stack": 99, "unlock": true,
	},
	"pill_mp": {
		"id": "pill_mp", "name": "凝神丹", "type": TYPE_CONSUME, "rarity": "凡品",
		"desc": "服用后恢复少量灵力。", "price": 12, "stack": 99, "unlock": true,
	},
	"pill_break": {
		"id": "pill_break", "name": "破境丹", "type": TYPE_CONSUME, "rarity": "仙品",
		"desc": "冲击瓶颈时服用，可提升破境成功率。", "price": 2000, "stack": 9, "unlock": false,
	},
	"herb_lingzhi": {
		"id": "herb_lingzhi", "name": "灵芝", "type": TYPE_MATERIAL, "rarity": "灵品",
		"desc": "百年灵芝，炼丹常用辅材。", "price": 40, "stack": 999, "unlock": true,
	},
	"herb_xueshen": {
		"id": "herb_xueshen", "name": "血参", "type": TYPE_MATERIAL, "rarity": "凡品",
		"desc": "药性温和的补血药材。", "price": 8, "stack": 999, "unlock": true,
	},
	"ore_coldiron": {
		"id": "ore_coldiron", "name": "寒铁矿", "type": TYPE_MATERIAL, "rarity": "灵品",
		"desc": "蕴含寒气的矿石，铸剑上品。", "price": 60, "stack": 999, "unlock": true,
	},
	"sword_qingfeng": {
		"id": "sword_qingfeng", "name": "青锋剑", "type": TYPE_EQUIP, "rarity": "灵品",
		"desc": "青云宗制式飞剑，锋锐轻灵。", "price": 500, "stack": 1, "unlock": true,
	},
	"robe_yunwen": {
		"id": "robe_yunwen", "name": "云纹袍", "type": TYPE_EQUIP, "rarity": "灵品",
		"desc": "绣有云纹法阵的护身法袍。", "price": 450, "stack": 1, "unlock": false,
	},
	"token_sect": {
		"id": "token_sect", "name": "宗门令牌", "type": TYPE_SPECIAL, "rarity": "灵品",
		"desc": "出入青云宗各处的身份凭证。", "price": 0, "stack": 1, "unlock": true,
	},
	"map_secret": {
		"id": "map_secret", "name": "秘境残图", "type": TYPE_SPECIAL, "rarity": "仙品",
		"desc": "落霞秘境的残缺地图，拼齐可指引传送。", "price": 0, "stack": 1, "unlock": false,
	},
}

## 全量道具总表（不区分解锁状态；首次注册写入存档，此后随存档恢复）
var items: Dictionary = DEFAULT_ITEMS.duplicate(true)

## 背包持有量（item_id -> 数量；运行数据）
var bag: Dictionary = {}

## 背包展示视图（UIGrid 模板契约字段：category/icon/count/name/desc/
## quality/locked）——由 items+bag 派生（只含有持有量 >0 的道具），
## 背包变化后自动重建，UI 网格 data="bean:xiuxian_item:bag_views"
var bag_views: Array = []

## 道具类型 -> 背包分类页签 id（与 bag_categories.gd NAME_TO_CAT 一致）
const TYPE_TO_CAT := {"消耗": "pill", "材料": "material", "装备": "tool", "特殊": "talisman"}
## 道具类型 -> 网格图标
const TYPE_ICONS := {"消耗": "🧪", "材料": "🌿", "装备": "⚔️", "特殊": "📜"}


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）。
## 注册后构建背包视图（重复调用重复重建，幂等无害；不覆盖 GdBean.on_ready——
## GDScript 视其为原生方法覆盖告警，本项目按错误处理）
static func ins() -> XiuItemState:
	var b: XiuItemState = GdBean.bean("xiuxian_item", func(): return new())
	b.refresh_bag_views()
	return b


## 重建背包展示视图并通知监听网格
func refresh_bag_views() -> void:
	var out: Array = []
	var ids: Array = bag.keys()
	ids.sort()
	for id in ids:
		var n := int(bag[id])
		if n <= 0:
			continue
		var it: Dictionary = get_item(id)
		var type := str(it.get("type", ""))
		out.append({
			"id": id,
			"category": TYPE_TO_CAT.get(type, "tool"),
			"icon": TYPE_ICONS.get(type, "📦"),
			"count": "x%d" % n,
			"name": str(it.get("name", id)),
			"desc": str(it.get("desc", "")),
			"quality": "稀有" if str(it.get("rarity", "")) == "灵品" else "",
			"locked": not bool(it.get("unlock", true)),
		})
	update("bag_views", out, {}, false)


# ------------------------------------------------------------------ 查询

## 按 id 取道具定义（无则返回空 Dictionary）
func get_item(item_id: String) -> Dictionary:
	return items.get(item_id, {})


## 全量道具列表（含未解锁——总表不区分解锁状态）
func get_all_items() -> Array:
	return items.values()


func get_item_ids() -> Array:
	return items.keys()


## 按类型查询（消耗/材料/装备/特殊），含未解锁
func get_items_by_type(type_name: String) -> Array:
	var out: Array = []
	for it in items.values():
		if str(it.get("type", "")) == type_name:
			out.append(it)
	return out


## 按品阶查询（凡品/灵品/仙品），含未解锁
func get_items_by_rarity(rarity: String) -> Array:
	var out: Array = []
	for it in items.values():
		if str(it.get("rarity", "")) == rarity:
			out.append(it)
	return out


## 关键字搜索（匹配名称或描述，子串包含），含未解锁
func search_items(keyword: String) -> Array:
	var out: Array = []
	for it in items.values():
		if str(it.get("name", "")).contains(keyword) \
				or str(it.get("desc", "")).contains(keyword):
			out.append(it)
	return out


func get_count(item_id: String) -> int:
	return int(bag.get(item_id, 0))


## 背包列表（[{id, name, count}, ...]，按 id 排序保证稳定顺序）
func get_bag_list() -> Array:
	var out: Array = []
	var ids: Array = bag.keys()
	ids.sort()
	for id in ids:
		out.append({"id": id, "name": str(get_item(id).get("name", id)),
			"count": int(bag[id])})
	return out


# ------------------------------------------------------------------ 变更

func add_item(item_id: String, count: int = 1) -> void:
	var b: Dictionary = bag.duplicate(true)
	b[item_id] = int(b.get(item_id, 0)) + count
	update("bag", b, {}, false)
	refresh_bag_views()


## 扣减持有量；不足时失败返回 false（不产生负数）
func remove_item(item_id: String, count: int = 1) -> bool:
	var cur: int = int(bag.get(item_id, 0))
	if cur < count:
		return false
	var b: Dictionary = bag.duplicate(true)
	var left: int = cur - count
	if left <= 0:
		b.erase(item_id)
	else:
		b[item_id] = left
	update("bag", b, {}, false)
	refresh_bag_views()
	return true


## 还原演示基线（目录 + 背包清空；跨运行确定性测试用）
func reset_demo() -> void:
	update("items", DEFAULT_ITEMS.duplicate(true), {}, true)
	update("bag", {}, {}, true)
	refresh_bag_views()
