# 道具状态 Bean（分类：道具）—— 仙途 demo 游戏状态层
#
# 职责：持有全量道具总表（不区分锁定状态，未解锁/未产出道具同样入库）
#       + 背包持有量 + 道具锁定进度（游戏进度状态，随存档持久化），
#       属性自动经 GDCORE 持久化到 GJson 存档
#       （user://coredata.data，路径 init;xiuxian_item;items / init;xiuxian_item;bag）。
#
# 数据源：道具**定义**已迁移到 state/item/item.gjson（GJson 定义表，
# 经 XiuItemTable 静态加载；改文件即改全游戏目录，无需动代码）；
# 本 Bean 只持有**运行数据**（首次注册从定义表派生总表写入存档 + 背包）。
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

## 全量道具总表（运行数据：首次注册由 item.gjson 定义表派生，此后随存档恢复；
## 定义本身的权威来源是 item.gjson，重置/补档经 XiuItemTable 读取）
var items: Dictionary = XiuItemTable.get_items()

## 背包持有量（item_id -> 数量；运行数据）
var bag: Dictionary = {}

## 道具锁定进度（游戏进度状态：item_id -> true 锁定；随存档持久化）。
## 静态定义表 item.json 不携带 unlock 字段——锁定/解锁随玩家游玩进度
## 变化，属于运行时状态，由本 Bean 持有；未记录的道具视为已解锁。
## 初始锁定集 DEFAULT_LOCKED 为演示剧情设定（高阶道具开局封印）
var locked_items: Dictionary = DEFAULT_LOCKED.duplicate()

## 背包展示视图（UIGrid 模板契约字段：category/icon/count/name/desc/
## quality/locked）——由 items+bag 派生（只含有持有量 >0 的道具），
## 背包变化后自动重建，UI 网格 data="bean:xiuxian_item:bag_views"
var bag_views: Array = []

## 道具类型 -> 背包分类页签 id（与 bag_categories.gd NAME_TO_CAT 一致）
const TYPE_TO_CAT := {"消耗": "pill", "材料": "material", "装备": "tool", "特殊": "talisman"}
## 道具类型 -> 网格图标
const TYPE_ICONS := {"消耗": "🧪", "材料": "🌿", "装备": "⚔️", "特殊": "📜"}

## 演示初始锁定集（高阶道具开局封印；定义表不再携带 unlock，此集属
## 游戏进度初始化数据，解锁后由运行时移除）
const DEFAULT_LOCKED := {
	"pill_break": true, "robe_yunwen": true, "map_secret": true,
	"break_douhuang": true, "break_douzong": true, "break_douzun": true,
	"break_dousheng": true, "mat_longxue": true, "break_doudi": true,
}


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
			"locked": is_item_locked(id),
		})
	update("bag_views", out, {}, false)


# ------------------------------------------------------------------ 查询

## 按 id 取道具定义（无则返回空 Dictionary）
func get_item(item_id: String) -> Dictionary:
	return items.get(item_id, {})


## 全量道具列表（含未解锁入包的道具——总表不区分锁定状态）
func get_all_items() -> Array:
	return items.values()


func get_item_ids() -> Array:
	return items.keys()


## 按类型查询（消耗/材料/装备/特殊），返回全量定义
func get_items_by_type(type_name: String) -> Array:
	var out: Array = []
	for it in items.values():
		if str(it.get("type", "")) == type_name:
			out.append(it)
	return out


## 按品阶查询（凡品/灵品/仙品），返回全量定义
func get_items_by_rarity(rarity: String) -> Array:
	var out: Array = []
	for it in items.values():
		if str(it.get("rarity", "")) == rarity:
			out.append(it)
	return out


## 关键字搜索（匹配名称或描述，子串包含），返回全量定义
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

## 道具是否锁定（游戏进度状态，见 locked_items 注释）
func is_item_locked(item_id: String) -> bool:
	return bool(locked_items.get(item_id, false))


## 解锁道具（游戏进度推进后调用；先改成员再 update 落盘）并刷新视图
func unlock_item(item_id: String) -> void:
	if not bool(locked_items.get(item_id, false)):
		return
	var d: Dictionary = locked_items.duplicate(true)
	d.erase(item_id)
	locked_items = d
	update("locked_items", d, {}, false)
	refresh_bag_views()


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


## 还原演示基线（目录重置为 item.gjson 定义 + 背包清空 + 锁定集回到
## 初始锁定；跨运行确定性测试用）
func reset_demo() -> void:
	update("items", XiuItemTable.get_items(), {}, true)
	update("bag", {}, {}, true)
	locked_items = DEFAULT_LOCKED.duplicate()
	update("locked_items", locked_items, {}, true)
	refresh_bag_views()
