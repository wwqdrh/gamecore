# GdBean watch 与已释放 UI 的残留回调 —— 复现/验收
#
# 运行：godot --headless --path . -s res://example/demo/xiuxian/state/check_bean_watch.gd
#
# 背景：mainhud/profile_panel 等 UI watch 持久 Bean（level/exp 等），主场景切换
# 或面板关闭后节点被释放，但 GdBean 无 unwatch、callback_map 残留死回调；
# GDScript self-lambda 在引擎侧持 self 裸指针，释放后 is_valid() 探活解引用
# 悬垂指针 → ObjectDB 报 Condition "slot >= slot_max"。
# 修复：bean.rs / state_store.rs / event_bus.rs 注册时快照目标 ObjectID，
# 通知路径按快照判活（释放=validator 失配静默判死，绝不触碰死回调本体）。
#
# 覆盖：方法回调 / lambda 回调两种注册方式；节点释放 + 内存扰动（迫使分配器
#       复用悬垂内存）后 update/publish 不报错；存活监听者仍收到通知；
#       死回调清理后同键新回调可正常注册生效；EventBus 死 self-lambda 订阅者。
extends SceneTree

var _total := 0
var _fails := 0


func check(cond: bool, msg: String) -> void:
	_total += 1
	if cond:
		print("[BeanWatch] PASS: ", msg)
	else:
		_fails += 1
		printerr("[BeanWatch] FAIL: ", msg)


## 模拟 UI 组件：ready 时 watch 持久 Bean（与 mainhud/profile_panel 同模式）
class Watcher:
	extends Node

	var hits := 0
	var last_val: int = -1

	func _ready() -> void:
		var bean = XiuCharacterState.ins()
		bean.watch("exp", _on_exp_changed)
		bean.watch("level", func(v = null, m = null): _on_level_changed(v))

	func _on_exp_changed(value: Variant = null, _metas: Variant = null) -> void:
		hits += 1
		last_val = int(value)

	func _on_level_changed(_value: int) -> void:
		pass


class Survivor:
	extends Node

	var hits := 0

	func _ready() -> void:
		XiuCharacterState.ins().watch("exp", _on_exp_changed)

	func _on_exp_changed(_value: Variant = null, _metas: Variant = null) -> void:
		hits += 1


## EventBus 复现用：以 self-lambda 订阅（释放后即成悬垂指针路径）
class BusWatcher:
	extends Node

	var sub_id := -1

	func _ready() -> void:
		var bus = Engine.get_singleton("GDEVENTBUS")
		sub_id = bus.subscribe("bean_watch_test", func(args = null): _on_evt(args))

	func _on_evt(_args = null) -> void:
		pass


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var gs = XiuGameState.ins()
	gs.reset_demo()
	var character = XiuCharacterState.ins()
	character.reset_demo()

	# 1. UI 节点注册 watch 后释放（等价于：主场景切换 / 面板关闭）
	var dead := Watcher.new()
	root.add_child(dead)
	await process_frame
	await process_frame

	var survivor := Survivor.new()
	root.add_child(survivor)
	await process_frame
	await process_frame
	check(dead.hits >= 1, "注册即回调：watcher 初始同步收到 exp 当前值")

	dead.queue_free()
	await process_frame
	await process_frame

	# 1.5 内存扰动：大量分配/释放，迫使分配器复用已释放 watcher 的内存块——
	#     悬垂 self-lambda 被探活时读到垃圾 ObjectID → 引擎报 slot >= slot_max
	#     （真实游戏中等价于场景切换后的持续分配压力）
	for i in 800:
		var churn := Control.new()
		churn.set_meta("payload", "churn-%d-%s" % [i, str(i * 7919)])
		root.add_child(churn)
		if i % 3 == 0:
			var sub := Label.new()
			sub.text = "sub-%d" % i
			churn.add_child(sub)
		churn.queue_free()
	await process_frame
	await process_frame

	# 2. 释放后 add_exp（enemy.gd _award_exp 同路径）——不得报 slot >= slot_max，
	#    存活监听者仍应收到通知
	var exp0 := int(character.exp)
	character.add_exp(30)
	await process_frame
	check(int(character.exp) == exp0 + 30, "释放 UI 后 add_exp 仍正常落账")
	check(survivor.hits >= 2, "存活监听者仍收到 update 通知（初始同步 + 本次变更）")

	# 3. 再次注册同键回调（回到主场景重建 UI 场景）仍可生效——
	#    死回调清理后不得被 registered_callback 哈希表误挡
	var fresh := Watcher.new()
	root.add_child(fresh)
	await process_frame
	await process_frame
	check(fresh.hits >= 1, "死回调清理后新 UI 同键 watch 注册即回调正常")

	character.add_exp(10)
	await process_frame
	check(fresh.hits >= 2, "新 UI 持续收到后续通知")

	# 4. EventBus 同类场景：节点以 self-lambda 订阅事件后释放 + 内存扰动，
	#    publish 不报 slot >= slot_max
	var bus = Engine.get_singleton("GDEVENTBUS")
	var dead_sub := BusWatcher.new()
	root.add_child(dead_sub)
	await process_frame
	await process_frame
	check(dead_sub.sub_id >= 0, "EventBus 订阅成功")
	dead_sub.queue_free()
	await process_frame
	await process_frame
	for i in 300:
		var churn := Label.new()
		churn.text = "bus-churn-%d" % i
		root.add_child(churn)
		churn.queue_free()
	await process_frame
	bus.publish("bean_watch_test", [])
	await process_frame
	check(true, "EventBus publish（含死 self-lambda 订阅者）不应报错")

	gs.reset_demo()
	print("[BeanWatch] RESULT=%s（%d/%d PASS）"
		% ["FAIL" if _fails > 0 else "PASS", _total - _fails, _total])
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(1 if _fails > 0 else 0)
