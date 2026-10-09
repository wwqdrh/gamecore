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

	# 2. 任务总表：全量查询（不区分解锁状态；定义源 task.json → task.gjson）
	check(task.get_all_tasks().size() == XiuTaskTable.count(), "任务总表应与定义表条数一致（9 条）")
	check(XiuTaskTable.count() == 9, "任务定义表应有 9 条（8 迁移 + side_003）")
	check(not task.get_task("side_003").is_empty(), "测试任务 side_003 应在总表中")
	check(task.get_task_ids().has("main_003"), "未解锁任务 main_003 也应在总表中")
	check(task.get_tasks_by_category("主线").size() == 3, "主线任务应 3 条")
	check(task.get_locked_tasks().size() == 3, "未解锁任务应 3 条（main_003/side_002/bounty_001）")
	check(task.get_unlocked_tasks().size() == 6, "已解锁任务应 6 条（含 side_003）")
	check(str(task.get_task("main_001").get("name")) == "初入仙途", "main_001 名称应正确")
	check(task.get_task("no_such").is_empty(), "未知 id 应返回空字典")
	# 初始状态推导：无进度记录时按 unlock 推导
	check(task.get_task_status("main_001") == "available", "已解锁无进度应为 available")
	check(task.get_task_status("main_003") == "locked", "未解锁无进度应为 locked")
	print("[State] task catalog: total=%d main=%d locked=%d" % [
		task.get_all_tasks().size(), task.get_tasks_by_category("主线").size(),
		task.get_locked_tasks().size()])

	# 3. 任务推进状态机：available → accepted → completed → submitted
	task.set_task_status("main_001", "accepted")
	for i in 3:
		var step: int = task.advance_task("main_001")
		check(step == i + 1, "main_001 第 %d 步推进应返回 step=%d" % [i + 1, i + 1])
	check(task.get_task_status("main_001") == "completed", "3 步走满应自动转 completed")
	check(task.get_tasks_by_status("completed").size() == 1, "completed 查询应命中 main_001")
	# Bean 路径查询（GJson 路径语义）
	check(str(task.get_value_by_key("progress;main_001;status")) == "completed",
		"路径查询 progress;main_001;status 应为 completed")
	check(str(task.get_value_by_key("tasks;main_002;precondition")) == "main_001",
		"路径查询 tasks;main_002;precondition 应为 main_001")
	print("[State] task flow: status=%s step=%s" % [
		task.get_task_status("main_001"), task.get_progress("main_001").get("step")])

	# 4. 道具总表：全量/分类/品阶/关键字（均含未解锁；定义源自 item.gjson）
	check(item.get_all_items().size() == 24, "道具总表应有 24 条（含未解锁）")
	check(item.get_items_by_type("消耗").size() == 13, "消耗类应 13 条")
	check(item.get_items_by_type("材料").size() == 7, "材料类应 7 条")
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

	# 6. 跨分类联动：提交 main_001 → 道具入库 + 经验/灵石 + 状态 submitted
	# （5c 已推进到斗者一段，先重置基线并复原第 4 节末状态：
	#   3 级余 50 / 灵石 40 / 背包回春丹 3）
	gs.reset_demo()
	character.add_exp(350)
	character.add_spirit_stones(20)
	item.add_item("pill_hp", 3)
	check(character.level == 3 and int(character.exp) == 50,
		"联动前置：应为 3 级余 50（got lv=%d exp=%d）" % [character.level, character.exp])
	var summary: Dictionary = gs.apply_task_rewards("main_001")
	check(not summary.is_empty(), "任务奖励发放应成功")
	check(int(summary["levels"]) == 1, "400 经验应再升 1 级（3→4）")
	check(character.level == 4 and character.exp == 150, "奖励后应 4 级余 150 经验（50+400-300）")
	check(character.spirit_stones == 70, "灵石 40+30 应为 70")
	check(item.get_count("herb_lingzhi") == 2, "奖励灵芝应入库 2")
	check(item.get_count("pill_hp") == 6, "奖励回春丹应 3+3=6")
	check(task.get_task_status("main_001") == "submitted", "提交后状态应为 submitted")
	print("[State] rewards main_001: items=%s exp=%s stones=%s levels=%s" % [
		str(summary["items"]), str(summary["exp"]),
		str(summary["spirit_stones"]), str(summary["levels"])])

	# 7. GJson 直查（跨 Bean，路径语义 "bean_id;字段;子键"）
	check(int(XiuGameState.query("xiuxian_character;level")) == 4,
		"query 人物等级应为 4")
	check(int(XiuGameState.query("xiuxian_item;bag;pill_hp")) == 6,
		"query 背包回春丹应为 6")
	check(str(XiuGameState.query("xiuxian_task;tasks;main_003;name")) == "秘境探幽",
		"query 未解锁任务名称应为 秘境探幽")
	check(str(XiuGameState.query("xiuxian_task;progress;main_001;status")) == "submitted",
		"query 任务进度状态应为 submitted")
	check(XiuGameState.query("no;such;path", "默认值") == "默认值",
		"未命中路径应返回默认值")
	var dump: String = XiuGameState.dump_json()
	check(dump.contains("xiuxian_character") and dump.contains("xiuxian_task"),
		"整库 JSON 应包含分类 Bean 数据")
	print("[State] gjson query: level=%s pill_hp=%s locked_name=%s json_len=%d" % [
		str(XiuGameState.query("xiuxian_character;level")),
		str(XiuGameState.query("xiuxian_item;bag;pill_hp")),
		str(XiuGameState.query("xiuxian_task;tasks;main_003;name")),
		dump.length()])

	# 8. 基线还原（演示语义：跑完检查回到干净状态）
	gs.reset_demo()
	check(character.level == 1 and character.exp == 0, "还原后应回到 1 级 0 经验")
	check(item.get_bag_list().is_empty(), "还原后背包应为空")
	check(task.get_task_status("main_001") == "available", "还原后 main_001 应回到 available")

	_finish()


func _finish() -> void:
	if ok:
		print("[State] RESULT=PASS")
	else:
		print("[State] RESULT=FAIL")
	quit(0 if ok else 1)
