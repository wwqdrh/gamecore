# 状态层端到端验收 —— example/demo/xiuxian/state 分类状态 Bean + GJson 直查
#
# 运行：godot --headless --path . -s res://example/demo/xiuxian/state/check_state.gd
#
# 覆盖：分类 Bean 注册（幂等单例）、全量任务/道具总表查询（含未解锁）、
#       分类/状态/品阶/关键字过滤、任务推进与状态机、道具增减、
#       人物经验升级与境界、跨分类联动（任务奖励发放）、
#       GJson 路径直查与整库 dump、reset_demo 基线还原。
extends SceneTree

const TaskStateScript := preload("res://example/demo/xiuxian/state/task/task_state.gd")
const ItemStateScript := preload("res://example/demo/xiuxian/state/item/item_state.gd")
const CharacterStateScript := preload("res://example/demo/xiuxian/state/character/character_state.gd")
const GameStateScript := preload("res://example/demo/xiuxian/state/game_state.gd")

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
	var task = TaskStateScript.ins()
	var item = ItemStateScript.ins()
	var character = CharacterStateScript.ins()
	var gs = GameStateScript.ins()
	check(gs.task.get_instance_id() == task.get_instance_id(), "gs.task 应与直取的 TaskBean 同实例")
	check(TaskStateScript.ins().get_instance_id() == task.get_instance_id(), "ins() 应幂等返回同一 Bean")
	print("[State] beans: task=%s item=%s character=%s" % [
		task != null, item != null, character != null])

	# 还原演示基线（GdBean 属性跨运行经存档恢复，必须先重置保证确定性）
	gs.reset_demo()

	# 2. 任务总表：全量查询（不区分解锁状态）
	check(task.get_all_tasks().size() == 8, "任务总表应有 8 条（含未解锁）")
	check(task.get_task_ids().has("main_003"), "未解锁任务 main_003 也应在总表中")
	check(task.get_tasks_by_category("主线").size() == 3, "主线任务应 3 条")
	check(task.get_locked_tasks().size() == 3, "未解锁任务应 3 条（main_003/side_002/bounty_001）")
	check(task.get_unlocked_tasks().size() == 5, "已解锁任务应 5 条")
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

	# 4. 道具总表：全量/分类/品阶/关键字（均含未解锁）
	check(item.get_all_items().size() == 10, "道具总表应有 10 条（含未解锁）")
	check(item.get_items_by_type("消耗").size() == 3, "消耗类应 3 条")
	check(item.get_items_by_rarity("仙品").size() == 2, "仙品应 2 条（破境丹/秘境残图，含未解锁）")
	check(item.search_items("丹").size() == 4, "搜「丹」应命中 4 条（3 丹药名 + 灵芝描述炼丹，含未解锁）")
	check(str(item.get_item("map_secret").get("name")) == "秘境残图", "未解锁道具定义应可查")
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
	check(character.get_realm() == "练气", "3 级境界应为 练气")
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

	# 6. 跨分类联动：提交 main_001 → 道具入库 + 经验/灵石 + 状态 submitted
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
	check(int(GameStateScript.query("xiuxian_character;level")) == 4,
		"query 人物等级应为 4")
	check(int(GameStateScript.query("xiuxian_item;bag;pill_hp")) == 6,
		"query 背包回春丹应为 6")
	check(str(GameStateScript.query("xiuxian_task;tasks;main_003;name")) == "秘境探幽",
		"query 未解锁任务名称应为 秘境探幽")
	check(str(GameStateScript.query("xiuxian_task;progress;main_001;status")) == "submitted",
		"query 任务进度状态应为 submitted")
	check(GameStateScript.query("no;such;path", "默认值") == "默认值",
		"未命中路径应返回默认值")
	var dump: String = GameStateScript.dump_json()
	check(dump.contains("xiuxian_character") and dump.contains("xiuxian_task"),
		"整库 JSON 应包含分类 Bean 数据")
	print("[State] gjson query: level=%s pill_hp=%s locked_name=%s json_len=%d" % [
		str(GameStateScript.query("xiuxian_character;level")),
		str(GameStateScript.query("xiuxian_item;bag;pill_hp")),
		str(GameStateScript.query("xiuxian_task;tasks;main_003;name")),
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
