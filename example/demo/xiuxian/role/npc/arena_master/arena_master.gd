# 演武教头·秦重 —— 青云坊演武堂主持（站桩）：宗门大比剧情节点
#
# 剧本职责（对话命令经 timeline @方法名 调达，NPC 专属动作写在本脚本）：
#   start_match()：@start_match 开赛 —— 置 flag match_started、任务推进一步
#   （报名），在演武场刷出比武弟子（XiuEnemyBase 变体）；
#   对手死亡 → 置 flag tournament_won + 任务推进完成（completed 待领奖），
#   玩家与教头再对话走 [arena_win] 段 @task_complete:sect_tournament 领奖。
extends XiuNpcBase

## 比武对手变体（golem 同基类不同导出值）
const OPPONENT_SCENE := preload("res://example/demo/xiuxian/role/enemy/golem.tscn")
## 对手站位格（演武场东侧；落在不可行走地形会被敌人自吸附修正）
const OPPONENT_CELL := Vector2i(23, 10)


func _init() -> void:
	display_name = "演武教头·秦重"
	role_name = "秦重"
	timeline_path = "res://example/demo/xiuxian/role/npc/arena_master/arena_timeline.txt"
	entry_stage = ""  # 置空：入口选段交给 timeline gate 优先级扫描（entry 直达无条件兜底段会绕过门控）
	npc_cell = Vector2i(20, 10)
	ai_behavior = 0  # AI_IDLE 站桩（主持台）
	body_color = Color(0.85, 0.62, 0.3)

var _opponent: Node = null


## 开赛：比武弟子登台（timeline @start_match 调达；重复调用幂等）
func start_match() -> void:
	if is_instance_valid(_opponent):
		return
	XiuDialogState.ins().set_flag("match_started")
	var step: int = XiuTaskState.ins().advance_task("sect_tournament")  # 报名 1/2
	print("[Arena] 比试开始（step %d）：比武弟子登台" % step)
	var map := get_parent()
	if map == null:
		return
	_opponent = OPPONENT_SCENE.instantiate()
	_opponent.display_name = "比武弟子·武痴"
	_opponent.max_health = 160.0
	_opponent.attack = 12.0
	_opponent.defense = 6.0
	_opponent.exp_reward = 200
	_opponent.sight_range = 320.0
	_opponent.body_color = Color(0.95, 0.8, 0.3)
	_opponent.enemy_cell = OPPONENT_CELL
	map.add_child(_opponent)  # 自吸附可行走格 + 绑定玩家追击
	_opponent.health.s_died.connect(_on_opponent_died)


## 对手倒下：夺魁（flag 供台词门控，任务推进一步 → completed 待领奖）
func _on_opponent_died() -> void:
	XiuDialogState.ins().set_flag("tournament_won")
	var step: int = XiuTaskState.ins().advance_task("sect_tournament")  # 夺魁 2/2
	print("[Arena] 比武弟子倒下，夺魁！（step %d，找教头领奖）" % step)
	_opponent = null
