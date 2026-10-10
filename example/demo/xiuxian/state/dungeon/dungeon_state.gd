# 秘境（随机副本）状态 Bean —— 仙途 demo 游戏状态层
#
# 职责：持有一次秘境挑战的进度数据（随存档持久化，中途退出可循 seed 复现地图）：
#   · 场景分类：FIXED_MAPS 固定场景（萧宅/青石镇/青云坊） vs DUNGEON_ALIAS 随机场景
#   · 一次 run：入口记录（from_map/from_cell，结算后送回原处）、run_seed（每层
#     seed = run_seed + floor * 7919，同一次挑战内层数越多地形越不同）、层数进度
#   · 结算数据：settle_view（通关层数/修为/灵石/战利品列表，供结算弹窗展示）
#   · boss_cleared：妖王是否击杀过（首杀解锁「秘境残图」，持久化一次性剧情）
#
# 流程控制（换图/计时/发奖时机）在 DungeonManager（scenes/main/dungeon_manager.gd）；
# 本 Bean 只存数据 + 派生（floor_seed/is_boss_floor），与地图/UI 零耦合。
class_name XiuDungeonState
extends GdBean

## 副本进行中
const STATUS_RUNNING := "running"
## 固定场景别名（xiaozhai.tscn / xiaozhen.tscn / town.tscn 三个固定地图）
const FIXED_MAPS := ["xiuxian_xiaozhai", "xiuxian_xiaozhen", "xiuxian_town"]
## 随机场景别名（map/dungeon/dungeon.tscn，每次进入按 seed 重新生成）
const DUNGEON_ALIAS := "xiuxian_dungeon"
## 一次秘境挑战的总层数（最后一层为 boss 层）
const TOTAL_FLOORS := 3
## 战利品池（结算随机取 2~4 条；id 需在 item.gjson 定义表内）
const LOOT_POOL := [
	"pill_hp", "pill_mp", "herb_xueshen", "herb_lingzhi",
	"ore_coldiron", "pill_exp_s",
]

## 副本状态："" 未在副本中 / STATUS_RUNNING 挑战进行中
var status: String = ""
## 当前层数（1 起，最后一层 boss）
var floor: int = 1
## 本次挑战总层数（预留后续按难度扩展，当前恒 TOTAL_FLOORS）
var total_floors: int = TOTAL_FLOORS
## 本次挑战随机种子（start_run 生成；每层地图 seed = floor_seed()）
var run_seed: int = 0
## 入口记录：进入前所在固定地图与所在格（结算后送回原处）
var from_map: String = ""
var from_cell: Vector2i = Vector2i(-1, -1)
## 妖王击杀进度（持久化：首杀解锁秘境残图）
var boss_cleared: bool = false
## 结算视图（结算弹窗数据契约：title/floors/exp/stones/items[{icon,name,count,note}]）
var settle_view: Dictionary = {}


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuDungeonState:
	return GdBean.bean("xiuxian_dungeon", func(): return new())


# ------------------------------------------------------------------ 派生

## 当前层地图生成种子（同一次挑战内每层不同，同一层恒定 → 可复现）
func floor_seed() -> int:
	return int(run_seed) + int(floor) * 7919


## 当前层是否为 boss 层（最后一层）
func is_boss_floor() -> bool:
	return int(floor) >= int(total_floors)


# ------------------------------------------------------------------ 变更

## 开始一次秘境挑战（记录入口，层数回 1，生成新 run 种子）
func start_run(from: String, cell: Vector2i) -> void:
	run_seed = randi() % 1000000000
	from_map = from
	from_cell = cell
	floor = 1
	total_floors = TOTAL_FLOORS
	settle_view = {}
	status = STATUS_RUNNING
	update("run_seed", run_seed, {}, false)
	update("from_map", from_map, {}, false)
	update("from_cell", from_cell, {}, false)
	update("floor", floor, {}, false)
	update("total_floors", total_floors, {}, false)
	update("settle_view", settle_view, {}, false)
	update("status", status, {}, false)


## 进入下一层（floor+1）
func advance_floor() -> void:
	floor = int(floor) + 1
	update("floor", floor, {}, false)


## 结算视图写入（发放在 DungeonManager.settle 编排，此处仅记录展示数据）
func write_settle(view: Dictionary) -> void:
	settle_view = view
	update("settle_view", view, {}, false)


## 结束本次挑战（status 回空串；settle_view 保留供弹窗展示）
func finish_run() -> void:
	status = ""
	update("status", status, {}, false)


## 标记妖王已被击杀，返回是否为本次首杀
func mark_boss_cleared() -> bool:
	if boss_cleared:
		return false
	boss_cleared = true
	update("boss_cleared", boss_cleared, {}, false)
	return true


## 还原演示基线（跨运行确定性测试用）
func reset_demo() -> void:
	status = ""
	floor = 1
	total_floors = TOTAL_FLOORS
	run_seed = 0
	from_map = ""
	from_cell = Vector2i(-1, -1)
	boss_cleared = false
	settle_view = {}
	update("status", status, {}, true)
	update("floor", floor, {}, true)
	update("total_floors", total_floors, {}, true)
	update("run_seed", run_seed, {}, true)
	update("from_map", from_map, {}, true)
	update("from_cell", from_cell, {}, true)
	update("boss_cleared", boss_cleared, {}, true)
	update("settle_view", settle_view, {}, true)
