# 萧宅农田虫害事件（main_002「灵田风波」战斗位）—— 挂在 home 场景根下
#
# 职责：
#   1. main_002 进行中（accepted）时在农田刷 3 只地灵鼠（换皮变体
#      mouse.tscn，站位格不可行走会由 enemy 自吸附兜底）
#   2. 清剿计数满 3 → advance_task 推进一步（剩下「回报外门弟子」1 步）
#
# 语义与边界：
#   · _process 0.5s 节流轮询任务状态，不走 Bean watch（红线：watch 回调内
#     禁同步调其他 Bean）；刷完即 set_process(false) 停轮询
#   · 切图重进：本节点随地图重建，step<1 时重刷 3 只（demo 级容忍；
#     清剿计步后不再刷，存档语义由 XiuTaskState 承担）
#   · 击杀回调内调 Bean：s_died 来自战斗受击（非 Bean 借用栈内），可直调；
#     仅在 status==accepted 时推进，未接取/已提交均无副作用
extends Node

const TASK_ID := "main_002"
const TOTAL := 3
const MOUSE_SCENE := preload("res://example/demo/xiuxian/role/enemy/mouse.tscn")
## 刷怪格（农田中部散布；home 全图草地方可走）
const SPAWN_CELLS := [Vector2i(7, 5), Vector2i(12, 8), Vector2i(9, 10)]

var _spawned := false
var _killed := 0
var _acc := 0.0


func _process(delta: float) -> void:
	if _spawned:
		set_process(false)
		return
	_acc += delta
	if _acc < 0.5:
		return
	_acc = 0.0
	var tasks := XiuTaskState.ins()
	if str(tasks.get_task_status(TASK_ID)) != tasks.STATUS_ACCEPTED:
		return
	_spawn_mice()


func _spawn_mice() -> void:
	_spawned = true
	set_process(false)
	var map := get_parent()
	for cell in SPAWN_CELLS:
		var e := MOUSE_SCENE.instantiate()
		e.enemy_cell = cell
		map.add_child(e)  # enemy 自吸附可行走格 + 绑定玩家
		e.health.s_died.connect(_on_mouse_died)
	print("[PestEvent] 灵田虫害：地灵鼠×3 出没")


func _on_mouse_died() -> void:
	_killed += 1
	if _killed < TOTAL:
		return
	var tasks := XiuTaskState.ins()
	if str(tasks.get_task_status(TASK_ID)) == tasks.STATUS_ACCEPTED:
		tasks.advance_task(TASK_ID)
		print("[PestEvent] 地灵鼠清剿完成，任务推进 1 步")
