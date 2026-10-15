# Stage1 M4 验收：第一阶段任务闭环端到端（状态层全链）
#
# 覆盖：
#   1. 主线链 main_001 → main_002 → main_004：前置链自动解锁
#      （complete_task 解锁 precondition 指向的后继任务）
#   2. 数值闭环：任务奖励修为 + 副本经验 → 斗之气阶满（5500 修为曲线）
#   3. 突破联动：try_breakthrough（break_qizhe×3）→ level 11（斗者一段）/
#      rank_dou_zhe flag / xiuxian_map_a 解锁 / main_004 rank_up 步推进
#   4. 传送门控：XiuMapState.is_map_unlocked 状态转换
#   5. 收束：main_004 talk 步推进 → complete → token_sect 入包
#
# 运行：godot --headless -s res://test/check_stage1_flow.gd
extends SceneTree

var _ok := true


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[Stage1Flow] OK ", msg)
	else:
		_ok = false
		printerr("[Stage1Flow] FAIL ", msg)


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	# 存档隔离 + 状态基线
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var ts := XiuTaskState.ins()
	var ch := XiuCharacterState.ins()
	var dlg := XiuDialogState.ins()
	var mp := XiuMapState.ins()
	var it := XiuItemState.ins()
	ts.reset_demo()
	ch.reset_demo()
	dlg.reset_demo()
	mp.clear_map_unlocked()
	it.reset_demo()

	# ---- 1. main_001 初入仙途：接 → 3 步 → 交 ----
	_check(ts.get_task_status("main_001") == ts.STATUS_AVAILABLE, "main_001 初始可接取")
	_check(ts.get_task_status("main_002") == ts.STATUS_AVAILABLE, "main_002 初始可接取")
	_check(ts.get_task_status("main_004") == ts.STATUS_LOCKED, "main_004 初始锁定（前置未达）")
	_check(ts.accept_task("main_001"), "接取 main_001")
	ts.advance_task("main_001")  # goto 山门
	ts.advance_task("main_001")  # talk 长老
	_check(ts.get_task_status("main_001") == ts.STATUS_ACCEPTED, "两步后仍在进行中")
	ts.advance_task("main_001")  # goto 弟子舍（第 3 步 → completed）
	_check(ts.get_task_status("main_001") == ts.STATUS_COMPLETED, "三步后 completed")
	_check(ts.complete_task("main_001"), "提交 main_001 领奖")
	_check(ts.get_task_status("main_001") == ts.STATUS_SUBMITTED, "main_001 已提交")
	_check(int(ch.exp) > 0 or ch.level > 1, "奖励修为已入账（exp=%d lv=%d）" % [int(ch.exp), ch.level])
	_check(it.get_count("herb_lingzhi") == 2 and it.get_count("pill_hp") == 3,
		"main_001 奖励道具入包（灵芝×2 回血丹×3）")

	# ---- 2. main_002 灵田风波：前置链解锁检查 + 击杀推进 ----
	_check(ts.get_task_status("main_002") == ts.STATUS_AVAILABLE, "main_002 本就 unlock，不受链影响")
	_check(ts.accept_task("main_002"), "接取 main_002")
	ts.advance_task("main_002")  # kill×3 记 1 步（home_pest 联动）
	ts.advance_task("main_002")  # talk 外门弟子
	_check(ts.complete_task("main_002"), "提交 main_002")
	_check(it.get_count("ore_coldiron") == 3, "main_002 奖励寒铁×3 入包")
	_check(ts.get_task_status("main_004") == ts.STATUS_AVAILABLE,
		"main_002 完成 → 前置链自动解锁 main_004")

	# ---- 3. main_004 破关之资：真相段（接取+赠药+collect 步） ----
	_check(ts.accept_task("main_004"), "接取 main_004（elder 真相段选项动作）")
	it.add_item("break_qizhe", 3)
	ts.advance_task("main_004")  # collect 聚气散×3（真相段 advance）
	_check(int(ts.get_progress("main_004").get("step", 0)) == 1, "collect 步已推进（1/3）")

	# ---- 4. 修为攒到斗之气阶满 + 突破联动 ----
	# 斗之气 1→10 级共需 100+200+…+1000 = 5500；奖励+副本大头在任务外，
	# 测试直接补足（数值曲线校算见 04_stage1_design.md §5）
	ch.add_exp(20000)
	_check(ch.level == 10, "经验封顶在斗之气 10 段（lv=%d）" % ch.level)
	_check(ch.is_rank_full(), "斗之气阶满")
	_check(ch.can_breakthrough(), "聚气散×3 齐备 → 可突破")
	_check(ch.try_breakthrough(), "突破成功")
	_check(ch.level == 11, "突破进入斗者一段（lv=%d）" % ch.level)
	_check(str(ch.get_realm_title()).contains("斗者"), "等阶头衔更新：%s" % ch.get_realm_title())
	_check(it.get_count("break_qizhe") == 0, "突破消耗聚气散×3")
	_check(dlg.has_flag("rank_dou_zhe"), "突破联动置 rank_dou_zhe flag")
	_check(mp.is_map_unlocked("xiuxian_map_a"), "突破联动解锁青草岭（xiuxian_map_a）")
	_check(not mp.is_map_unlocked("xiuxian_map_b"), "幽竹林仍锁定（第二阶段）")
	_check(not mp.is_map_unlocked("某未入表地图"), "未入表地图默认锁定")
	_check(mp.is_map_unlocked("xiuxian_town"), "主线四图恒解锁")
	_check(int(ts.get_progress("main_004").get("step", 0)) == 2,
		"突破联动推进 main_004 rank_up 步（2/3）")

	# ---- 5. 收束：复命 → complete → 宗门令牌 ----
	ts.advance_task("main_004")  # talk 长老复命（elder final 段选项动作）
	_check(int(ts.get_progress("main_004").get("step", 0)) == 3, "talk 步推进（3/3）")
	_check(ts.complete_task("main_004"), "提交 main_004")
	_check(it.get_count("token_sect") == 1, "宗门令牌入包")
	_check(ts.get_task_status("main_003") == ts.STATUS_AVAILABLE,
		"main_004 完成 → 前置链解锁秘境探幽（第二阶段钩子）")

	# ---- 6. 幂等与重置 ----
	_check(not ch.try_breakthrough(), "未满段重复突破应失败")
	mp.clear_map_unlocked()
	_check(not mp.is_map_unlocked("xiuxian_map_a"), "clear_map_unlocked 复位野外图")

	print("[Stage1Flow] RESULT: %s" % ("PASS" if _ok else "FAIL"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if _ok else 1)
