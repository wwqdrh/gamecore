# 任务领奖弹窗控制器 —— 挂 reward_panel.gml 根（Panel）
#
# 职责：watch XiuTaskState.last_reward（complete_task 自动发奖后写入）
# → 填充标题/任务名/奖励格子 → 自动弹 TaskRewardModal；「确认」按钮
# clear_last_reward + emit s_close_requested（Modal 关闭联动）。
# 业务侧只调 complete_task，不碰 UI（单向数据流：数据变更驱动弹窗）。
#
# 红线对齐：
#   · watch 回调内禁同步调其他 Bean → _on_last_reward.call_deferred
#     （回调里读 XiuTaskState 属性 + 开 Modal）
#   · 列表刷新统一 update(data, false)（force=true 在 count==len 时不建条目）
#   · 悬停 tips：gml_tips_host() 懒创建共享 GdUITooltip 宿主
#     （__GmlTipsHost，最后添加=绘制最上层），条目沿父链取用
extends Panel

## 展示延迟（complete_task 到弹窗的缓冲，让世界观感上先结算再领奖）
const OPEN_DELAY_FRAMES := 2

## 内容关闭联动信号：Modal 自动连接，「确认」时 emit → 整体关闭
signal s_close_requested


func _ready() -> void:
	var task := XiuTaskState.ins()
	# watch 去重红线：独立 lambda（共用 Callable 只首个 key 生效）
	task.watch("last_reward", func(_v = null, _m = null): _on_last_reward.call_deferred())


## 奖励数据到达：填充视图并自动弹窗（空数据跳过——关闭时清空也走这里）
func _on_last_reward() -> void:
	var task := XiuTaskState.ins()
	var items: Array = task.last_reward
	if items.is_empty():
		return
	var title := find_child("TitleLabel", true, false) as Label
	var sub := find_child("SubtitleLabel", true, false) as Label
	if title:
		title.text = "任务完成"
	if sub:
		sub.text = str(task.last_reward_task)
	var list := find_child("RewardList", true, false)
	if list == null:
		return
	list.call("update", items, false)
	# 缓冲两帧再开：等列表条目重建完成（格子 _ready 已接线 tips）
	await get_tree().process_frame
	await get_tree().process_frame
	var modal: Node = GdUIManager.find_ui("TaskRewardModal")
	if modal != null and not modal.is_modal_open():
		modal.open()


## 确认按钮：清空展示数据（防跨运行恢复旧值重弹）+ 请求 Modal 关闭
func _on_confirm_pressed() -> void:
	XiuTaskState.ins().clear_last_reward()
	emit_signal("s_close_requested")


## 共享悬停 tips 宿主（奖励格子经 reward_item.gd 沿父链调用取用）。
## 懒创建：首次有格子申请时挂到本面板（最后添加 = 绘制最上层）
func gml_tips_host() -> Control:
	var host := get_node_or_null("__GmlTipsHost") as Control
	if host == null:
		var tip: Control = GdUITooltip.new()
		tip.name = "__GmlTipsHost"
		tip.set("follow_mouse", false)
		add_child(tip)
		host = tip
	return host
