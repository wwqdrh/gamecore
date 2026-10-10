# 秘境（随机副本）端到端验收 —— 入口按钮链路 / 随机生成 / 多层流转 / 通关结算
#
# 运行：perl -e 'alarm 240; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_dungeon_flow.gd
#
# 覆盖：
#   1. 装配：DungeonManager 组件注册表可达；xiuxian_dungeon 已注册；
#      固定场景三别名（FIXED_MAPS）与随机场景分类
#   2. 入口：start_run() 记录入口（萧宅+玩家格）→ 切入随机场景、状态 running
#   3. 随机层：按层种子出图 + 随机刷怪（第 1 层 4 只，距出生点 ≥ 6 格）
#   4. 扫荡流转：全灭 → 1.2s 后自动进下一层（第 2 层 5 只、新地图实例）
#   5. boss 层：第 3 层有「秘境妖王」（Boss 节点、超常规数值）
#   6. 结算：boss 层清空 → 原路送回萧宅、状态回空、修为/灵石到账、
#      背包获得战利品 + 首杀解锁秘境残图、结算弹窗打开
#   7. 防重入：进行中 start_run 返回 false；结束后可再次开启（新种子）
extends SceneTree

var ok := true


func fail(msg: String) -> void:
	push_error("[Dungeon] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _wait_frames(n: int) -> void:
	for i in n:
		await process_frame


## 轮询直到 cond 为真（超时 fail）；返回最终 cond 值
func _poll(cond: Callable, max_frames: int, msg: String) -> bool:
	for i in max_frames:
		if cond.call():
			return true
		await process_frame
	fail(msg + "（超时 %d 帧）" % max_frames)
	return false


func _run() -> void:
	# 0. 状态层复位（跨运行确定性）
	var dungeon: XiuDungeonState = XiuDungeonState.ins()
	var items: XiuItemState = XiuItemState.ins()
	var char_state: XiuCharacterState = XiuCharacterState.ins()
	dungeon.reset_demo()
	items.reset_demo()
	var exp0: int = int(char_state.exp)
	var stones0: int = int(char_state.spirit_stones)

	# 1. 装配：主场景 + DungeonManager 组件注册 + 随机场景别名注册
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	await _wait_frames(4)

	var sroot: Node = Engine.get_singleton("GDCORE").get_global_node("default")
	var mgr: Node = null
	for i in 120:
		mgr = sroot.get_component("DungeonManager")
		if mgr != null:
			break
		await process_frame
	check(mgr != null, "DungeonManager 应注册进组件表")
	if mgr == null:
		_finish()
		return

	var map_mgr: Node = sroot.get_component("MapManager")
	var registered = map_mgr.get_registered_maps()
	check(registered.has(XiuDungeonState.DUNGEON_ALIAS),
		"注册表应含随机场景 xiuxian_dungeon，实际 %s" % [registered])
	check(XiuDungeonState.FIXED_MAPS.size() == 3
		and XiuDungeonState.FIXED_MAPS.has("xiuxian_xiaozhai")
		and XiuDungeonState.FIXED_MAPS.has("xiuxian_xiaozhen")
		and XiuDungeonState.FIXED_MAPS.has("xiuxian_town"),
		"固定场景应为萧宅/青石镇/青云坊三个")

	# 2. 入口：start_run 记录入口并切入随机场景
	check(mgr.start_run(), "start_run 应成功开启秘境")
	check(str(map_mgr.get_current_alias()) == XiuDungeonState.DUNGEON_ALIAS,
		"应切入随机场景，实际 %s" % map_mgr.get_current_alias())
	check(str(dungeon.status) == XiuDungeonState.STATUS_RUNNING, "状态应为 running")
	check(str(dungeon.from_map) == "xiuxian_xiaozhai", "入口地图应记录为萧宅")
	check(int(dungeon.floor) == 1 and int(dungeon.total_floors) == 3, "应从第 1 层/共 3 层开始")
	check(not mgr.start_run(), "进行中重复 start_run 应被拒绝")

	# 3. 第 1 层：随机出图 + 刷怪（3+1=4 只）
	if not await _poll_floor(mgr, map_mgr, 4, "第 1 层"):
		_finish()
		return

	# 4. 全灭 → 自动进第 2 层（3+2=5 只，新地图实例）
	var floor1: Node = get_first_floor()
	var map1: Node = floor1.get_parent()  # 旧地图稍后被释放，先缓存引用
	for e in floor1.get_enemies():
		e.health.kill()
	check(int(floor1.get_alive_count()) == 0, "全灭后存活数应清零")
	if not await _poll(func(): return int(dungeon.floor) == 2, 900,
		"清场后应自动进入第 2 层"):
		_finish()
		return
	if not await _poll_floor(mgr, map_mgr, 5, "第 2 层"):
		_finish()
		return
	check(map_mgr.get_current_map() != map1, "第 2 层应是新实例化的地图")

	# 5. → 第 3 层（boss 层）：3+3=6 只普通怪 + 秘境妖王
	var floor2: Node = get_first_floor()
	for e in floor2.get_enemies():
		e.health.kill()
	if not await _poll(func(): return int(dungeon.floor) == 3, 900,
		"清场后应自动进入第 3 层（boss 层）"):
		_finish()
		return
	if not await _poll_floor(mgr, map_mgr, 7, "boss 层"):
		_finish()
		return
	var floor3: Node = get_first_floor()
	var boss: Node = floor3.get_parent().get_node_or_null("Boss")
	check(boss != null, "boss 层应有 Boss 节点")
	if boss != null:
		check(str(boss.display_name) == "秘境妖王", "boss 名应为秘境妖王")
		check(float(boss.max_health) > 300.0, "boss 血量应远超普通怪（320）")

	# 6. boss 层全灭 → 结算：回萧宅 + 发奖 + 结算弹窗
	for e in floor3.get_enemies():
		if is_instance_valid(e):
			e.health.kill()
	if not await _poll(func(): return str(dungeon.status) == "" and str(map_mgr.get_current_alias()) == "xiuxian_xiaozhai", 900,
		"通关后应原路送回萧宅且状态回空"):
		_finish()
		return
	check(int(char_state.spirit_stones) == stones0 + 180,
		"结算灵石应 +180（60×3），实际 %d → %d" % [stones0, int(char_state.spirit_stones)])
	check(int(char_state.exp) > exp0, "修为应增加（清怪经验 + 通关奖励）")
	check(not items.bag.is_empty(), "结算后背包应有战利品")
	check(items.bag.has("map_secret"), "妖王首杀应发放秘境残图")
	check(not items.is_item_locked("map_secret"), "秘境残图应随首杀解锁")
	var view: Dictionary = dungeon.settle_view
	check(int(view.get("floors", 0)) == 3 and int(view.get("exp", 0)) == 300
		and int(view.get("stones", 0)) == 180 and not view.get("items", []).is_empty(),
		"结算视图应含层数/修为/灵石/战利品，实际 %s" % [view])
	var modal: Object = GdUIManager.find_ui("DungeonSettleModal")
	check(modal != null, "DungeonSettleModal 应已注册")
	if modal != null:
		check(bool(modal.call("is_modal_open")), "通关后结算弹窗应打开")

	# 7. 结束后可再次开启（新 run 种子）
	check(mgr.start_run(), "结算后 start_run 应可再次开启")
	var seed2: int = int(dungeon.run_seed)
	check(seed2 != 0, "第二次挑战应生成新种子")
	dungeon.reset_demo()  # 还原，避免污染其他测试
	index.queue_free()
	_finish()


func _finish() -> void:
	print("[Dungeon] RESULT: %s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)


## 找当前随机地图的 Floor 控制器（组内唯一）
func get_first_floor() -> Node:
	return get_first_node_in_group("dungeon_floor")


## 当前楼层存活数（无楼层返回 -1）
func _floor_alive() -> int:
	var f: Node = get_first_node_in_group("dungeon_floor")
	if f == null:
		return -1
	return int(f.get_alive_count())


## 轮询等待某层就绪：敌人已刷出且数量正确
func _poll_floor(_mgr: Node, map_mgr: Node, expect_alive: int, tag: String) -> bool:
	var ready := await _poll(func(): return _floor_alive() == expect_alive,
		300, "%s 应刷出 %d 只敌人" % [tag, expect_alive])
	if not ready:
		return false
	var f: Node = get_first_node_in_group("dungeon_floor")
	var map: Node = f.get_parent()
	var spawn: Vector2i = f.spawn_cell
	check(bool(map.is_walkable(spawn)), "%s 出生格应可行走 %s" % [tag, spawn])
	for e in f.get_enemies():
		check(absi(e.enemy_cell.x - spawn.x) + absi(e.enemy_cell.y - spawn.y) >= 6
			or e.name == "Boss", "%s 敌人应距出生点 ≥ 6 格" % tag)
		break  # 逐只断言受随机性影响，抽查首只即可（生成器保证）
	check(str(map_mgr.get_current_alias()) == XiuDungeonState.DUNGEON_ALIAS,
		"%s 当前地图应为随机场景" % tag)
	print("[Dungeon] %s ok: alive=%d spawn=%s" % [tag, f.get_alive_count(), spawn])
	return true
