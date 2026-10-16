# 第一阶段 M2 验收 —— 灵田虫害事件（main_103 战斗位）端到端
#
# 运行：perl -e 'alarm 180; exec @ARGV' godot --headless --path . \
#   -s res://test/check_stage1_enemy.gd
#
# 覆盖：
#   1. 接取 main_103 后进 home（萧宅农田）→ PestEvent 自动刷地灵鼠×3
#      （数值档：血 20/攻 3/经 40）
#   2. 击杀计数：前 2 只死亡任务不推进，第 3 只死亡推进一步（steps=2 中
#      的 kill 步），状态保持 accepted（剩「回报小翠」步）
#   3. 未接取任务不刷怪（PestEvent 轮询门控）
#   4. 副本数值档回归：check_dungeon_flow / check_enemy_flow 另行覆盖
extends SceneTree

var ok := true


func fail(msg: String) -> void:
	push_error("[Stage1Enemy] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _mice(map: Node) -> Array:
	var out: Array = []
	for child in map.get_children():
		if is_instance_valid(child) and str(child.get("display_name")) == "地灵鼠" \
				and child.get("health") != null:
			out.append(child)
	return out


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var tasks: XiuTaskState = XiuTaskState.ins()
	tasks.reset_demo()

	# 1. 装配主场景（默认萧宅）
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	for i in 6:
		await process_frame
	var mgr: Node = index.get_node_or_null("MapManager")
	check(mgr != null, "MapManager 应装配")
	if mgr == null:
		_finish()
		return

	# 2. 未接取任务：进 home 不刷怪
	check(mgr.open_map("xiuxian_home", Vector2i(3, 3)), "应能打开 home")
	for i in 40:
		await process_frame
	check(_mice(mgr.get_current_map()).is_empty(), "未接取 main_103 时不应刷地灵鼠")

	# 3. 前置链走到 main_103 可接取 → 回 home → PestEvent 刷鼠×3
	_check_chain()
	check(tasks.accept_task("main_103"), "main_103 应可接取")
	check(mgr.open_map("xiuxian_home", Vector2i(3, 3)), "应能重进 home")
	# PestEvent 0.5s 节流轮询 + 刷怪/吸附，给足帧数
	for i in 90:
		await process_frame
	var mice: Array = _mice(mgr.get_current_map())
	check(mice.size() == 3, "接取后应刷地灵鼠×3，实际 %d" % mice.size())
	if mice.size() != 3:
		_finish()
		return
	var e0: Node = mice[0]
	check(int(e0.get("exp_reward")) == 40 and float(e0.get("max_health")) == 20.0,
			"地灵鼠数值档应为 血20/经40")

	# 4. 逐只击杀：前 2 只不推进，第 3 只推进一步
	for i in mice.size():
		var e: Node = mice[i]
		e.get("health").kill()  # 直调死亡（s_died → PestEvent 计数）
		for f in 5:
			await process_frame
		var p: Dictionary = tasks.get_progress("main_103")
		if i < 2:
			check(int(p.get("step", 0)) == 0, "击杀 %d/3 时任务不应推进" % (i + 1))
		else:
			check(int(p.get("step", 0)) == 1, "清剿 3/3 应推进一步，实际 step=%d" % int(p.get("step", 0)))
			check(str(tasks.get_task_status("main_103")) == "accepted",
					"清剿完成后应保持 accepted（剩回报步），实际 %s" % tasks.get_task_status("main_103"))

	# 5. 清理
	index.queue_free()
	_finish()


func _finish() -> void:
	if ok:
		print("[Stage1Enemy] RESULT=PASS")
	else:
		print("[Stage1Enemy] RESULT=FAIL")
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)


## 走完 main_101 → main_102 前置链（main_103 前置依赖）
func _check_chain() -> void:
	var tasks: XiuTaskState = XiuTaskState.ins()
	tasks.accept_task("main_101")
	tasks.advance_task("main_101")
	tasks.complete_task("main_101")
	tasks.accept_task("main_102")
	tasks.advance_task("main_102")
	tasks.advance_task("main_102")
	tasks.complete_task("main_102")
