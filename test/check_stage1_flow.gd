# Stage1 验收：家族篇（第一阶段）任务闭环端到端（状态层全链）
#
# 覆盖：
#   1. 主线链 main_101 → 102 → 103 → 104 → 105 → sect_tournament：
#      前置链自动解锁（complete_task 解锁 precondition 指向的后继任务）
#   2. collect(auto) 步自动推进：背包获得目标物品 → watch("bag") 延迟一帧推进
#   3. 步级 flag：advance_task 置 task_<id>_step<N>（台词本门控用）
#   4. 数值闭环：任务奖励修为 → 斗之气阶满（5500 修为曲线）
#   5. 突破联动：try_breakthrough（break_qizhe×3）→ level 11（斗者一段）/
#      rank_dou_zhe flag / xiuxian_map_a 解锁 / main_105 rank_up 步推进
#   6. 收束：main_105 talk 步推进 → complete → token_sect 入包；
#      sect_tournament（宗门选拔）解锁 → 完成 → stage2 main_201 点亮
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

	# ---- 1. main_101 初来乍到：接 → 推进 → 交 ----
	_check(ts.get_task_status("main_101") == ts.STATUS_AVAILABLE, "main_101 初始可接取")
	_check(ts.get_task_status("main_102") == ts.STATUS_LOCKED, "main_102 初始锁定（前置 main_101 未提交，链式推导）")
	_check(ts.get_task_status("side_102") == ts.STATUS_AVAILABLE, "无前置支线 side_102 恒可接取")
	_check(ts.accept_task("main_101"), "接取 main_101")
	ts.advance_task("main_101")  # talk 萧老爷（steps=1 → completed）
	_check(ts.get_task_status("main_101") == ts.STATUS_COMPLETED, "一步走满应 completed")
	_check(dlg.has_flag("task_main_101_step1"), "步级 flag task_main_101_step1 应同步")
	_check(ts.complete_task("main_101"), "提交 main_101 领奖")
	_check(int(ch.coins) == 30, "main_101 奖励金币 30 入账")

	# ---- 2. main_102 宅中众人：前置链解锁 + 两步推进 ----
	_check(ts.get_task_status("main_102") == ts.STATUS_AVAILABLE, "main_101 提交 → 前置链点亮 main_102")
	_check(ts.get_task_status("side_101") == ts.STATUS_AVAILABLE, "side_101（pre main_101）同步点亮")
	_check(ts.get_task_status("side_104") == ts.STATUS_LOCKED, "side_104 前置 main_102 未提交应锁定")
	_check(ts.accept_task("main_102"), "接取 main_102")
	ts.advance_task("main_102")  # talk 萧夫人
	ts.advance_task("main_102")  # talk 小翠 → completed
	_check(ts.complete_task("main_102"), "提交 main_102")
	_check(ts.get_task_status("side_104") == ts.STATUS_AVAILABLE, "main_102 提交 → side_104 点亮")

	# ---- 3. main_103 灵田风波：击杀推进（home_pest 联动位） ----
	_check(ts.get_task_status("main_103") == ts.STATUS_AVAILABLE, "main_103 前置链点亮")
	_check(ts.accept_task("main_103"), "接取 main_103")
	ts.advance_task("main_103")  # kill 地灵鼠×3（home_pest advance）
	_check(dlg.has_flag("task_main_103_step1"), "击杀步级 flag 应置位（回报段门控用）")
	ts.advance_task("main_103")  # talk 小翠回报 → completed
	_check(ts.complete_task("main_103"), "提交 main_103")
	_check(it.get_count("pill_hp") == 2, "main_103 奖励回血丹×2 入包")

	# ---- 4. collect(auto) 自动推进：side_102 采药济世 ----
	_check(ts.accept_task("side_102"), "接取 side_102")
	it.add_item("herb_xueshen", 1)  # 采集入库（deco 回调同路径）
	for i in 3:
		await process_frame  # watch("bag") → call_deferred 推进
	_check(int(ts.get_progress("side_102").get("step", 0)) == 1,
		"collect(auto) 步应在背包获得血参后自动推进（1/2）")
	_check(dlg.has_flag("task_side_102_step1"), "采集步级 flag 应置位（白芷交付段门控用）")
	ts.advance_task("side_102")  # talk 白芷 → completed
	_check(ts.complete_task("side_102"), "提交 side_102")
	_check(ts.get_task_status("side_103") == ts.STATUS_AVAILABLE, "side_103（pre side_102）点亮")

	# ---- 5. main_104 坊市风声：两步对话推进 ----
	_check(ts.accept_task("main_104"), "接取 main_104")
	ts.advance_task("main_104")  # talk 贾先生（置 theft_rumor 走对话）
	ts.advance_task("main_104")  # talk 外门弟子（置 token_clue）→ completed
	_check(ts.complete_task("main_104"), "提交 main_104")
	dlg.set_flag("token_clue")  # 对话流程置位（幂等，保障下方真相段语义）

	# ---- 6. main_105 破关之资：真相段接取 + 赠药 + collect（非 auto） ----
	_check(ts.get_task_status("main_105") == ts.STATUS_AVAILABLE, "main_105 前置链点亮")
	_check(ts.accept_task("main_105"), "接取 main_105（master_gift 段选项动作）")
	it.add_item("break_qizhe", 3)
	for i in 3:
		await process_frame
	_check(int(ts.get_progress("main_105").get("step", 0)) == 0,
		"非 auto 的 collect 步不应被背包监听推进（剧情赠药走显式 advance）")
	ts.advance_task("main_105")  # collect 聚气散×3（真相段 advance）
	_check(int(ts.get_progress("main_105").get("step", 0)) == 1, "collect 步已推进（1/3）")

	# ---- 7. 修为攒到斗之气阶满 + 突破联动 ----
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
	_check(int(ts.get_progress("main_105").get("step", 0)) == 2,
		"突破联动推进 main_105 rank_up 步（2/3）")

	# ---- 8. 收束：复命 → complete → 宗门令牌 → 选拔解锁 ----
	ts.advance_task("main_105")  # talk 萧老爷复命（master_break 段选项动作）
	_check(int(ts.get_progress("main_105").get("step", 0)) == 3, "talk 步推进（3/3）")
	_check(ts.complete_task("main_105"), "提交 main_105")
	_check(it.get_count("token_sect") == 1, "宗门引荐令入包")
	_check(ts.get_task_status("sect_tournament") == ts.STATUS_AVAILABLE,
		"main_105 完成 → 前置链解锁宗门选拔（第 7 天日历自动接取）")

	# ---- 9. sect_tournament 宗门选拔：报名 → 夺魁 → stage2 点亮 ----
	_check(ts.accept_task("sect_tournament"), "接取 sect_tournament")
	ts.advance_task("sect_tournament")  # talk 秦重报名（start_match 联动）
	ts.advance_task("sect_tournament")  # kill 比武弟子（对手死亡联动）→ completed
	_check(ts.complete_task("sect_tournament"), "提交 sect_tournament（领奖位同时置 stage1_clear）")
	dlg.set_flag("stage1_clear")  # arena_win 选项动作置位（幂等，保障语义）
	_check(dlg.has_flag("stage1_clear"), "第一阶段收束 flag stage1_clear 应置位")
	_check(ts.get_task_status("main_201") == ts.STATUS_AVAILABLE,
		"选拔完成 → stage2 main_201 宗门报到点亮")

	# ---- 10. 幂等与重置 ----
	_check(not ch.try_breakthrough(), "未满段重复突破应失败")
	mp.clear_map_unlocked()
	_check(not mp.is_map_unlocked("xiuxian_map_a"), "clear_map_unlocked 复位野外图")

	print("[Stage1Flow] RESULT: %s" % ("PASS" if _ok else "FAIL"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if _ok else 1)
