# 状态层端到端验收 —— example/demo/xiuxian/state 分类状态 Bean + GJson 直查
#
# 运行：godot --headless --path . -s res://example/demo/xiuxian/state/check_state.gd
#
# 覆盖：分类 Bean 注册（幂等单例）、全量任务/道具总表查询（含未解锁）、
#       分类/状态/品阶/关键字过滤、任务推进与状态机、道具增减、
#       等阶经验体系（level.gjson 11 阶 × 10 级：阶内升级/阶满封顶/
#       突破材料查询/突破扣料进阶）、人物经验升级与境界、
#       跨分类联动（任务奖励发放）、GJson 路径直查与整库 dump、
#       reset_demo 基线还原。
extends SceneTree
var ok := true


func fail(msg: String) -> void:
	push_error("[State] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	var tsg: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	tsg.isolate()
	# 0. GDCORE 单例（GJson 存档后端）
	var core: Object = Engine.get_singleton("GDCORE")
	check(core != null, "GDCORE 单例应已注册")
	if core == null:
		_finish()
		return

	# 1. 分类 Bean 注册（幂等单例）
	var task = XiuTaskState.ins()
	var item = XiuItemState.ins()
	var character = XiuCharacterState.ins()
	var gs = XiuGameState.ins()
	check(gs.task.get_instance_id() == task.get_instance_id(), "gs.task 应与直取的 TaskBean 同实例")
	check(XiuTaskState.ins().get_instance_id() == task.get_instance_id(), "ins() 应幂等返回同一 Bean")
	print("[State] beans: task=%s item=%s character=%s" % [
		task != null, item != null, character != null])

	# 还原演示基线（GdBean 属性跨运行经存档恢复，必须先重置保证确定性）
	gs.reset_demo()

	# 2. 任务总表：全量查询（不区分解锁状态；定义源 stage1/stage2.json → *.gjson）
	check(task.get_all_tasks().size() == XiuTaskTable.count(), "任务总表应与定义表条数一致（15 条）")
	check(XiuTaskTable.count() == 15, "任务定义表应有 15 条（stage1 家族篇 10 + stage2 宗门篇 5）")
	check(not task.get_task("sect_tournament").is_empty(), "stage1 收官任务 sect_tournament 应在总表中")
	check(task.get_task_ids().has("main_202"), "未解锁任务 main_202 也应在总表中")
	check(task.get_tasks_by_category("主线").size() == 8, "主线任务应 8 条（stage1 六条 + stage2 报到/秘境）")
	# 解锁状态由前置链推导（2026-10-16 起定义表不携带 unlock 字段）：
	# locked = 有前置且前置未提交 11 条
	#（main_102/103/104/105、sect_tournament、side_101/103/104、main_201/202、bounty_201）
	check(task.get_locked_tasks().size() == 11, "未解锁任务应 11 条")
	check(task.get_unlocked_tasks().size() == 4, "已解锁任务应 4 条（无前置：main_101/side_102/daily_201/guild_201）")
	check(str(task.get_task("main_101").get("name")) == "初来乍到", "main_101 名称应正确")
	check(task.get_task("no_such").is_empty(), "未知 id 应返回空字典")
	# 初始状态推导：无进度记录时按前置链推导（定义表已无 unlock 字段）
	check(task.get_task_status("main_101") == "available", "无前置无进度应为 available")
	check(task.get_task_status("main_102") == "locked", "前置 main_101 未提交应推导 locked")
	check(task.get_task_status("sect_tournament") == "locked", "前置链未达应推导 locked")
	print("[State] task catalog: total=%d main=%d locked=%d" % [
		task.get_all_tasks().size(), task.get_tasks_by_category("主线").size(),
		task.get_locked_tasks().size()])

	# 3. 任务推进状态机：available → accepted → completed → submitted
	#    （advance 同步步级 flag task_<id>_step<N>，台词本门控用）
	task.set_task_status("main_101", "accepted")
	var step1: int = task.advance_task("main_101")
	check(step1 == 1, "main_101 推进应返回 step=1")
	check(task.get_task_status("main_101") == "completed", "1 步走满应自动转 completed")
	check(task.get_tasks_by_status("completed").size() == 1, "completed 查询应命中 main_101")
	check(XiuDialogState.ins().has_flag("task_main_101_step1"), "步级 flag task_main_101_step1 应同步到对话状态")
	# Bean 路径查询（GJson 路径语义）
	check(str(task.get_value_by_key("progress;main_101;status")) == "completed",
		"路径查询 progress;main_101;status 应为 completed")
	check(str(task.get_value_by_key("tasks;main_102;precondition")) == "main_101",
		"路径查询 tasks;main_102;precondition 应为 main_101")
	print("[State] task flow: status=%s step=%s" % [
		task.get_task_status("main_101"), task.get_progress("main_101").get("step")])

	# 4. 道具总表：全量/分类/品阶/关键字（均含未解锁；定义源自 item.gjson）
	check(item.get_all_items().size() == 34, "道具总表应有 34 条（含未解锁）")
	check(item.get_items_by_type("消耗").size() == 16, "消耗类应 16 条")
	check(item.get_items_by_type("材料").size() == 11, "材料类应 11 条")
	check(item.get_items_by_rarity("仙品").size() == 7, "仙品应 7 条（含未解锁）")
	check(item.get_items_by_rarity("帝品").size() == 1, "帝品应 1 条（帝品雏丹）")
	check(item.search_items("丹").size() == 10, "搜「丹」应命中 10 条")
	check(str(item.get_item("map_secret").get("name")) == "秘境残图", "未解锁道具定义应可查")
	# 静态定义表直查（XiuItemTable，非 Bean）
	check(str(XiuItemTable.get_item("break_qizhe").get("name")) == "聚气散",
		"定义表应可查突破材料 聚气散")
	check(XiuItemTable.get_items_by_use("breakthrough").size() == 12,
		"突破材料定义应 12 条")
	check(str(XiuItemTable.get_item("pill_exp_m").get("use")) == "exp",
		"修为丹应标记 use=exp")
	# 背包增减
	item.add_item("pill_hp", 5)
	check(item.get_count("pill_hp") == 5, "pill_hp 应持有 5")
	item.remove_item("pill_hp", 2)
	check(item.get_count("pill_hp") == 3, "pill_hp 应剩余 3")
	check(not item.remove_item("pill_hp", 99), "超出持有量扣减应失败")
	check(item.get_count("pill_hp") == 3, "失败扣减不应改变持有量")
	check(not item.remove_item("no_such", 1), "未持有道具扣减应失败")
	check(item.get_bag_list().size() == 1, "背包应只有 1 种道具")
	# Bean 路径查询
	check(int(item.get_value_by_key("bag;pill_hp")) == 3, "路径查询 bag;pill_hp 应为 3")
	print("[State] item: total=%d consume=%d bag=%s" % [
		item.get_all_items().size(), item.get_items_by_type("消耗").size(),
		str(item.get_bag_list())])

	# 5. 人物状态：经验升级 / 境界 / 灵石 / 属性
	check(int(character.get_attr("悟性")) == 8, "悟性初值应为 8")
	var gained: int = character.add_exp(350)
	check(gained == 2 and character.level == 3 and character.exp == 50,
		"350 经验应从 1 级升到 3 级余 50（got level=%d exp=%d gained=%d）" % [
			character.level, character.exp, gained])
	check(character.get_realm() == "斗之气", "3 级境界应为 斗之气")
	check(character.get_realm_title() == "斗之气三段", "3 级显示名应为 斗之气三段")
	check(character.hp == 140 and character.max_hp == 140, "升级应刷新满血 140")
	character.add_spirit_stones(30)
	check(character.spirit_stones == 50, "灵石 20+30 应为 50")
	check(character.spend_spirit_stones(10), "灵石充足扣减应成功")
	check(not character.spend_spirit_stones(100), "灵石不足扣减应失败")
	check(character.spirit_stones == 40, "灵石最终应为 40")
	character.set_attr("悟性", 9)
	check(character.get_attr("悟性") == 9, "悟性改后应为 9")
	check(int(character.get_value_by_key("attrs;根骨")) == 6, "路径查询 attrs;根骨 应为 6")
	print("[State] character: lv=%d exp=%d realm=%s stones=%d" % [
		character.level, character.exp, character.get_realm(), character.spirit_stones])

	# 5b. 等阶经验体系定义表（level.json→加密 level.gjson 产物，XiuLevelTable 静态查询）
	# 管线回归：产物存在、确为密文（非 '{' 开头）、解密回环与明文源一致
	var lv_gjson := "res://example/demo/xiuxian/state/level/level.gjson"
	var lv_src := "res://example/demo/xiuxian/state/level/level.json"
	check(FileAccess.file_exists(lv_gjson), "level.gjson 加密产物应存在（重跑 test/regen_gjson.gd）")
	var enc := FileAccess.get_file_as_bytes(lv_gjson)
	check(enc.size() > 0 and enc[0] != 0x7b, "level.gjson 应为密文（不应以 '{' 开头）")
	check(GdJsonCodec.decrypt_to_text(enc) == FileAccess.get_file_as_string(lv_src),
		"level.gjson 解密应与明文 level.json 逐字节一致")
	check(XiuLevelTable.get_rank_count() == 11, "等阶应 11 阶（斗之气→斗帝）")
	check(XiuLevelTable.get_max_level() == 110, "全局等级上限应为 110")
	check(XiuLevelTable.get_level_exp(1) == 100, "斗之气一段升二段应需 100 经验")
	check(XiuLevelTable.get_level_exp(10) == 1000, "斗之气十段升满应需 1000 经验")
	check(XiuLevelTable.get_level_exp(11) == 400, "斗者一段升二段应需 400 经验")
	check(XiuLevelTable.get_level_exp(110) == 0, "顶阶满级应无升级需求（返回 0）")
	check(XiuLevelTable.is_rank_full(10), "10 级应为斗之气阶满")
	check(not XiuLevelTable.is_rank_full(11), "11 级不应为阶满")
	check(XiuLevelTable.get_rank_title_at(3) == "斗之气三段", "3 级应显示 斗之气三段")
	check(XiuLevelTable.get_rank_title_at(110) == "斗帝圆满", "满级应显示 斗帝圆满")
	var bt_reqs: Array = XiuLevelTable.get_breakthrough_items("da_dou_shi")
	check(bt_reqs.size() == 2, "大斗师突破应需 2 种材料（玄玉膏+凝魂草）")
	print("[State] level table: ranks=%d max_lv=%d bt(dadoushi)=%s" % [
		XiuLevelTable.get_rank_count(), XiuLevelTable.get_max_level(),
		str(bt_reqs)])

	# 5c. 等阶突破流程：阶满封顶 → 材料不足失败 → 备料成功进阶
	check(not character.is_rank_full(), "3 级不应处于阶满")
	var big: int = character.add_exp(6000)
	check(character.level == 10 and character.is_rank_full(),
		"6000 经验应从 3 级升到 10 级阶满（got lv=%d gained=%d）" % [character.level, big])
	check(int(character.exp) == 1000, "阶满经验应封顶 1000（got %d）" % int(character.exp))
	check(not character.can_breakthrough(), "无突破材料时 can_breakthrough 应为 false")
	check(not character.try_breakthrough(), "无突破材料时突破应失败")
	var mats: Array = character.get_breakthrough_materials()
	check(mats.size() == 1 and str(mats[0].get("id")) == "break_qizhe",
		"斗之气突破应需聚气散")
	item.add_item("break_qizhe", 3)
	check(character.can_breakthrough(), "备齐聚气散 x3 后应可突破")
	check(character.try_breakthrough(), "突破应成功")
	check(character.level == 11 and int(character.exp) == 0,
		"突破后应进入斗者一段、经验清零")
	check(character.get_realm() == "斗者", "突破后境界应为 斗者")
	check(item.get_count("break_qizhe") == 0, "突破材料应被扣除")
	print("[State] breakthrough: lv=%d realm=%s" % [
		character.level, character.get_realm_title()])

	# 6. 跨分类联动：提交 main_101 → 金币/经验入账 + 状态 submitted
	# （5c 已推进到斗者一段，先重置基线并复原第 4 节末状态：
	#   3 级余 50 / 灵石 40 / 背包回春丹 3）
	gs.reset_demo()
	character.add_exp(350)
	character.add_spirit_stones(20)
	item.add_item("pill_hp", 3)
	check(character.level == 3 and int(character.exp) == 50,
		"联动前置：应为 3 级余 50（got lv=%d exp=%d）" % [character.level, character.exp])
	var summary: Dictionary = gs.apply_task_rewards("main_101")
	check(not summary.is_empty(), "任务奖励发放应成功")
	check(int(summary["levels"]) == 0, "80 经验不足升 1 级（3 级需 300）")
	check(character.level == 3 and character.exp == 130, "奖励后应 3 级余 130 经验（50+80）")
	check(int(character.coins) == 30, "奖励金币 30 应入账")
	check(item.get_bag_list().size() == 1, "main_101 无道具奖励，背包应仍只有回春丹")
	check(task.get_task_status("main_101") == "submitted", "提交后状态应为 submitted")
	print("[State] rewards main_101: items=%s exp=%s levels=%s" % [
		str(summary["items"]), str(summary["exp"]), str(summary["levels"])])

	# 7. GJson 直查（跨 Bean，路径语义 "bean_id;字段;子键"）
	check(int(XiuGameState.query("xiuxian_character;level")) == 3,
		"query 人物等级应为 3")
	check(int(XiuGameState.query("xiuxian_item;bag;pill_hp")) == 3,
		"query 背包回春丹应为 3")
	check(str(XiuGameState.query("xiuxian_task;tasks;main_202;name")) == "秘境探幽",
		"query 未解锁任务名称应为 秘境探幽")
	check(str(XiuGameState.query("xiuxian_task;progress;main_101;status")) == "submitted",
		"query 任务进度状态应为 submitted")
	check(XiuGameState.query("no;such;path", "默认值") == "默认值",
		"未命中路径应返回默认值")
	var dump: String = XiuGameState.dump_json()
	check(dump.contains("xiuxian_character") and dump.contains("xiuxian_task"),
		"整库 JSON 应包含分类 Bean 数据")
	print("[State] gjson query: level=%s pill_hp=%s locked_name=%s json_len=%d" % [
		str(XiuGameState.query("xiuxian_character;level")),
		str(XiuGameState.query("xiuxian_item;bag;pill_hp")),
		str(XiuGameState.query("xiuxian_task;tasks;main_202;name")),
		dump.length()])

	# 8. 基线还原（演示语义：跑完检查回到干净状态）
	gs.reset_demo()
	check(character.level == 1 and character.exp == 0, "还原后应回到 1 级 0 经验")
	check(item.get_bag_list().is_empty(), "还原后背包应为空")
	check(task.get_task_status("main_101") == "available", "还原后 main_101 应回到 available")

	_finish()


func _finish() -> void:
	if ok:
		print("[State] RESULT=PASS")
	else:
		print("[State] RESULT=FAIL")
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
