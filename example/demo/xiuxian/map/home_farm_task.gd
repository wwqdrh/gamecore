# 萧宅灵田农事事件（side_104「灵田初耕」教学位）—— 挂在 home 场景根下
#
# 职责：side_104 进行中（accepted）时监听地图根（GdDigLayer）的农事信号，
#   按任务步骤逐次推进：锄地（s_dug）→ 播种（s_planted）→ 浇灌（s_watered）。
#
# 语义与边界：
#   · 信号直连（非 Bean watch 借用栈），advance 走 call_deferred——
#     红线：GdDigLayer 信号在 rust 内发射，回调里同步调 Bean 有单借用风险
#   · 幂等：每步只推进一次（_done 步骤标记），重复锄地/浇水不重复计步；
#     未接取/已提交均无副作用
#   · 切图重进：本节点随地图重建，未完成的步骤重新可触发（demo 级容忍；
#     步进语义由 XiuTaskState 持久化承担）
extends Node

const TASK_ID := "side_104"

var _done := {}


func _ready() -> void:
	var layer := get_parent()
	if layer.has_signal("s_dug"):
		layer.connect("s_dug", _on_dug)
	if layer.has_signal("s_planted"):
		layer.connect("s_planted", _on_planted)
	if layer.has_signal("s_watered"):
		layer.connect("s_watered", _on_watered)


func _on_dug(_cell: Vector2i, _from: String, _to: String) -> void:
	_mark("s_dug")


func _on_planted(_cell: Vector2i) -> void:
	_mark("s_planted")


func _on_watered(_cell: Vector2i) -> void:
	_mark("s_watered")


func _mark(sig: String) -> void:
	if _done.get(sig, false):
		return
	_done[sig] = true
	_advance.call_deferred()


func _advance() -> void:
	var tasks := XiuTaskState.ins()
	if str(tasks.get_task_status(TASK_ID)) == tasks.STATUS_ACCEPTED:
		tasks.advance_task(TASK_ID)
		print("[FarmEvent] 灵田初耕推进：%s" % str(_done.keys()))
