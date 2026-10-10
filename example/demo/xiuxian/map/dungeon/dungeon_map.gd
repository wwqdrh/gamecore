# 秘境楼层生成器 —— 挂在随机地图 dungeon.tscn 根（GdQuickMap）下的控制器
#
# 职责（一次挑战每层实例化一份本场景，随地图一起释放）：
#   1. 副本进行中（XiuDungeonState.status=running）时按层种子重新生成地形
#      （GdQuickMap.generate(seed)：噪声+连通性挖桥，同 seed 必然同图 → 可复现）
#   2. 随机刷怪：普通小怪 3+层号 只，按权重从史莱姆/妖狐/傀儡池抽取，
#      落在距出生点 ≥ 6 格的随机可行走格；boss 层额外生成「秘境妖王」
#   3. 扫荡判定：每只怪死亡 alive-1，清零时发 s_floor_cleared
#      （换层/结算流转由 DungeonManager 编排，本节点只管本层）
#
# 场景分类说明：固定场景（xiaozhai/xiaozhen/town）为手工 tscn；
# 随机场景仅 dungeon.tscn 一张壳，运行时按种子出图——「随机」体现在
# 种子与刷怪，注册表仍是普通别名 xiuxian_dungeon。
extends Node

## 楼层清空信号（全部敌人死亡时发出，DungeonManager 接收进入下一层/结算）
signal s_floor_cleared

## 敌人变体场景池（key -> tscn；变体 = 同基类脚本不同导出值）
const ENEMY_SCENES := {
	"slime": preload("res://example/demo/xiuxian/role/enemy/slime.tscn"),
	"fox": preload("res://example/demo/xiuxian/role/enemy/fox.tscn"),
	"golem": preload("res://example/demo/xiuxian/role/enemy/golem.tscn"),
}
## 抽怪权重池（前期小怪多、越深层傀儡比例越高）
const ENEMY_POOL := [
	"slime", "slime", "slime", "fox", "fox", "golem",
]

## 出生格（generate 后解析，供玩家/测试查询）
var spawn_cell := Vector2i(-1, -1)

var _alive := 0
var _enemies: Array = []
var _done := false


func _ready() -> void:
	add_to_group("dungeon_floor")


func _process(_delta: float) -> void:
	set_process(false)
	if _done:
		return
	var state := XiuDungeonState.ins()
	if str(state.status) != XiuDungeonState.STATUS_RUNNING:
		return  # 非副本状态（编辑器/固定场景误挂）：保持场景静态配置
	_done = true
	var map := get_parent()
	if map == null or not map.has_method("generate"):
		return
	map.generate(int(state.floor_seed()))
	_spawn(map, state)


## 存活敌人数（测试观测）
func get_alive_count() -> int:
	return _alive


## 本层敌人引用列表（测试观测；死亡节点随引擎回收自动失效）
func get_enemies() -> Array:
	return _enemies


# ------------------------------------------------------------------ 生成

## 按层号刷怪（普通怪 3+floor 只；boss 层加「秘境妖王」）
func _spawn(map: Node, state: XiuDungeonState) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = int(state.floor_seed())
	var w := int(map.get_map_width())
	var h := int(map.get_map_height())
	var center := Vector2i(w / 2, h / 2)
	spawn_cell = _nearest_walkable(map, center)

	# 玩家出生格留白：怪距出生点至少 6 格
	for i in 3 + int(state.floor):
		var key: String = ENEMY_POOL[rng.randi_range(0, ENEMY_POOL.size() - 1)]
		_spawn_enemy(map, rng, ENEMY_SCENES[key], _random_cell(map, rng, 6), {})

	if state.is_boss_floor():
		var boss_cfg := {
			"display_name": "秘境妖王",
			"max_health": 320.0,
			"attack": 22.0,
			"defense": 10.0,
			"exp_reward": 500,
			"sight_range": 288.0,
			"body_color": Color(0.75, 0.3, 0.85),
		}
		var boss := _spawn_enemy(map, rng, ENEMY_SCENES["golem"], _random_cell(map, rng, 10), boss_cfg)
		if boss != null:
			boss.scale = Vector2(1.5, 1.5)
			boss.name = "Boss"


func _spawn_enemy(map: Node, rng: RandomNumberGenerator, scene: PackedScene, cell: Vector2i, cfg: Dictionary) -> Node:
	var e := scene.instantiate()
	e.enemy_cell = cell
	for k in cfg:
		e.set(k, cfg[k])
	map.add_child(e)  # 实例化后挂到地图根：enemy 自吸附可行走格 + 绑定玩家
	_alive += 1
	_enemies.append(e)
	e.health.s_died.connect(_on_enemy_died)  # health 在 enemy _ready 创建
	return e


func _on_enemy_died() -> void:
	_alive -= 1
	if _alive <= 0:
		s_floor_cleared.emit()


# ------------------------------------------------------------------ 格子工具

## 距出生点至少 min_dist（曼哈顿）的随机可行走格（32 次重试后退化为 BFS 吸附）
func _random_cell(map: Node, rng: RandomNumberGenerator, min_dist: int) -> Vector2i:
	var w := int(map.get_map_width())
	var h := int(map.get_map_height())
	for i in 32:
		var c := Vector2i(rng.randi_range(1, w - 2), rng.randi_range(1, h - 2))
		if absi(c.x - spawn_cell.x) + absi(c.y - spawn_cell.y) < min_dist:
			continue
		if bool(map.is_walkable(c)):
			return c
	return _nearest_walkable(map, Vector2i(rng.randi_range(1, w - 2), rng.randi_range(1, h - 2)))


## BFS 距 from 最近的可行走格（含 from 本身）
func _nearest_walkable(map: Node, from: Vector2i) -> Vector2i:
	if bool(map.is_walkable(from)):
		return from
	var w := int(map.get_map_width())
	var h := int(map.get_map_height())
	var queue: Array = [from]
	var seen := {from: true}
	while not queue.is_empty():
		var cur: Vector2i = queue.pop_front()
		if bool(map.is_walkable(cur)):
			return cur
		for off in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
			var nxt: Vector2i = cur + off
			if nxt.x < 0 or nxt.y < 0 or nxt.x >= w or nxt.y >= h or seen.has(nxt):
				continue
			seen[nxt] = true
			queue.append(nxt)
	return from
