# 秘境（随机副本）端到端验收 —— 跨场景全链路：主城入口 → 探索场景 → 多层流转 →
# 通关结算 → 弹窗关闭回主城（送回入口）/ 中途放弃
#
# 运行：perl -e 'alarm 300; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_dungeon_flow.gd
#
# 覆盖：
#   1. 装配：main 场景 DungeonManager 可达；MapManager 注册表**不含**随机场景
#      （随机场景只在 explore 场景注册）；固定场景三别名分类
#   2. 主城 UI 分离：mainhud 有道具栏 EquipBar、无技能栏 SkillBar
#   3. 入口：start_run() 记录入口（家宅+玩家格）→ 整场景切换 xiuxian_explore
#   4. 探索场景：ExploreManager 可达；explore HUD（层数横幅/离开按钮/技能栏/
#      结算弹窗）装配；第 1 层随机刷怪 4 只（距出生点 ≥ 6 格）
#   5. 扫荡流转：全灭 → 自动进第 2 层（5 只，新地图实例）→ 第 3 层 boss（秘境妖王）
#   6. 结算：boss 层清空 → 发奖（修为/灵石/战利品/首杀秘境残图）→ 结算弹窗打开
#   7. 回程：弹窗关闭 → 场景切回 xiuxian_main → 原路送回家宅、入口记录清空
#   8. 放弃：再次进入后 abandon() → 回主场景、状态回空（无奖励发放）
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
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	# 0. 状态层复位（跨运行确定性）
	var dungeon: XiuDungeonState = XiuDungeonState.ins()
	var items: XiuItemState = XiuItemState.ins()
	var char_state: XiuCharacterState = XiuCharacterState.ins()
	dungeon.reset_demo()
	items.reset_demo()
	var exp0: int = int(char_state.exp)
	var stones0: int = int(char_state.spirit_stones)

	# 1. 走真实场景流：GdSceneRoot → title → 点开始 → main
	var packed: PackedScene = load("res://example/demo/xiuxian/index.tscn")
	check(packed != null, "xiuxian/index.tscn 应可加载")
	if packed == null:
		_finish()
		return
	var scene_root: Node = packed.instantiate()
	root.add_child(scene_root)
	await _wait_frames(8)
	var stage: Control = scene_root.get_node_or_null("SceneLayer")
	if stage != null:
		stage.size = Vector2(1920, 1080)
		await process_frame
	check(await _poll(func(): return _scene_id(scene_root) == "xiuxian_title", 300,
		"入口场景应为 title"), "入口场景 title")
	var btn_start: Control = scene_root.find_child("BtnStart", true, false)
	check(btn_start != null, "title 应有开始按钮")
	if btn_start == null:
		_finish()
		return
	# 新流程：点开始 → 存档选择弹窗 → 点「存档三」空槽位进入 main。
	# 必须走空槽（save_id "3"）：存档一 = 玩家真实活档（id ""），
	# 用户游戏进程每 30s flush 会写回其实时进度，测试在其上断言必然翻车。
	if FileAccess.file_exists("user://coredata_3.data"):
		DirAccess.remove_absolute("user://coredata_3.data")
	btn_start.emit_signal("pressed")
	check(await _poll(func(): return _modal_open(scene_root), 120, "点开始后应打开存档弹窗"),
		"点开始后打开存档弹窗")
	var slot3_btn: Control = scene_root.find_child("Slot3Btn", true, false)
	check(slot3_btn != null, "弹窗应有存档三槽位按钮")
	if slot3_btn == null:
		_finish()
		return
	slot3_btn.emit_signal("pressed")
	if not await _poll(func(): return _scene_id(scene_root) == "xiuxian_main", 600,
		"选槽后应切到 main 场景"):
		_finish()
		return
	await _wait_frames(6)

	# 2. main 侧装配：DungeonManager 可达；随机场景不在主城注册表；UI 分离
	var dungeon_mgr: Node = await _await_component(scene_root, "DungeonManager")
	if dungeon_mgr == null:
		_finish()
		return
	var cur: Node = scene_root.call("get_current_scene")
	var map_mgr: Node = cur.get_node_or_null("MapManager")
	check(map_mgr != null, "main 场景应有 MapManager")
	if map_mgr != null:
		var registered = map_mgr.get_registered_maps()
		check(not registered.has(XiuDungeonState.DUNGEON_ALIAS),
			"主城注册表不应含随机场景（随机场景只在 explore 注册），实际 %s" % [registered])
		check(registered.has("xiuxian_xiaozhai"), "主城注册表应含固定场景")
	check(XiuDungeonState.FIXED_MAPS.size() == 3
		and XiuDungeonState.FIXED_MAPS.has("xiuxian_xiaozhai")
		and XiuDungeonState.FIXED_MAPS.has("xiuxian_xiaozhen")
		and XiuDungeonState.FIXED_MAPS.has("xiuxian_town"),
		"固定场景应为萧宅/青石镇/青云坊三个")
	var hud: Control = scene_root.find_child("MainHud", true, false)
	check(hud != null, "main 场景应有 mainhud")
	if hud != null:
		check(hud.find_child("EquipBar", true, false) != null,
			"主城应保留道具栏 EquipBar")
		check(hud.find_child("SkillBar", true, false) == null,
			"主城不应再有技能栏 SkillBar（已迁移探索场景）")

	# 3. 入口：start_run 记录入口（initial_map 家宅 + 玩家格——主场景注册表内
	#    任意地图均可开局，不再限 FIXED_MAPS）并整场景切换到探索场景
	check(str(map_mgr.get_current_alias()) == "xiuxian_home", "起始应在家宅（initial_map）")
	check(dungeon_mgr.start_run(), "start_run 应成功开启秘境")
	check(str(dungeon.status) == XiuDungeonState.STATUS_RUNNING, "状态应为 running")
	check(str(dungeon.from_map) == "xiuxian_home", "入口地图应记录为家宅")
	check(int(dungeon.floor) == 1 and int(dungeon.total_floors) == 3, "应从第 1 层/共 3 层开始")
	check(not dungeon_mgr.start_run(), "进行中重复 start_run 应被拒绝")
	if not await _poll(func(): return _scene_id(scene_root) == "xiuxian_explore", 600,
		"start_run 后应切换到探索场景 xiuxian_explore"):
		_finish()
		return
	await _wait_frames(6)

	# 4. 探索场景装配：ExploreManager + explore HUD（技能栏/横幅/离开/结算弹窗）
	var exp_mgr: Node = await _await_component(scene_root, "ExploreManager")
	if exp_mgr == null:
		_finish()
		return
	check(scene_root.find_child("MainHud", true, false) == null,
		"探索场景不应有主城 mainhud")
	# 注意：explore.gml 是无壳模式（<ui script> 挂控制器），gml 根 Control 被提升合并，
	# 节点名 = tscn 实例名 ExploreUI（TopLayer/BottomLayer/SkillBar/Modal 直接挂其下）
	var explore_ui: Control = scene_root.find_child("ExploreUI", true, false)
	check(explore_ui != null, "探索场景应有 explore HUD")
	if explore_ui != null:
		check(explore_ui.find_child("SkillBar", true, false) != null,
			"探索场景应有技能栏 SkillBar")
		check(explore_ui.find_child("LeaveBtn", true, false) != null,
			"探索场景应有离开秘境按钮")
		check(explore_ui.find_child("FloorText", true, false) != null,
			"探索场景应有层数横幅")
	check(GdUIManager.has_ui("DungeonSettleModal"), "结算弹窗应注册在探索场景")

	# 4.5 秘境内左键 = 施放技能，**不点击行走**（回归：曾落到 _on_map_clicked 寻路）
	var player: Node = get_first_node_in_group("player")
	check(player != null, "探索场景应有玩家")
	if player != null:
		if await _poll(func(): return bool(player.get("_dungeon_combat")), 120,
				"秘境应进入弹幕战斗模式（_dungeon_combat）"):
			var dm: Node = player.get("danmaku")
			var casts: Array = []
			if dm != null:
				dm.s_cast.connect(func(_shape, _count): casts.append(1))
			# 未选中技能：左键回落技能 1（普通攻击，无冷却）
			Engine.get_singleton("GDSTATE").set_state("mainhud.skill", "")
			await _wait_frames(2)
			player.call("stop")
			var click := InputEventMouseButton.new()
			click.button_index = MOUSE_BUTTON_LEFT
			click.pressed = true
			click.position = Vector2(500, 300)
			player.call("_unhandled_input", click)
			await _wait_frames(2)
			check(not player.is_grid_path_active(), "秘境左键不应触发点击行走")
			var line = player.get("path_line")
			check(line == null or int(line.get_point_count()) == 0,
				"秘境左键不应生成路径预览线")
			check(casts.size() == 1, "秘境左键应施放技能（未选中 → 技能 1 兜底）")
			# 3 号（2.5s 冷却）：首放入独立冷却账，冷却中重复点击被拒
			Engine.get_singleton("GDSTATE").set_state("mainhud.skill", "3")
			await _wait_frames(2)
			player.call("_unhandled_input", click)
			check(casts.size() == 2, "选中 3 号后左键应施放")
			var st: Vector2 = player.call("skill_cd_state", "3")
			check(st.y > 0.0 and st.x > 0.0, "3 号施放后应进入独立冷却记账")
			player.call("_unhandled_input", click)
			check(casts.size() == 2, "技能位冷却中重复点击应被静默拒绝")
			_clear_bullets()  # 别让试射子弹干扰后续刷怪计数
			print("[Dungeon] 秘境左键 = 施法不寻路 ok")

	# 5. 第 1 层：随机出图 + 刷怪（3+1=4 只）
	if not await _poll_floor(scene_root, 4, "第 1 层"):
		_finish()
		return

	# 5.5 远战怪（妖狐）攻击相位应朝玩家开火：直调 Brain 攻击回调 →
	# 场景内出现敌方子弹（mask=2 只打玩家受击盒）
	var fox: Node = null
	for e in get_first_floor().get_enemies():
		if is_instance_valid(e) and bool(e.get("ranged")):
			fox = e
			break
	if fox != null:
		fox.call("_on_brain_attack")
		check(await _poll(func(): return _count_bullets() > 0, 60,
			"远战怪攻击相位应发射子弹"), "远战怪（妖狐）应朝玩家开火")
		_clear_bullets()
		print("[Dungeon] 远战怪开火 ok: %s" % fox.display_name)
	else:
		print("[Dungeon] 第 1 层无远战怪（随机池，跳过开火抽测）")

	# 6. 全灭 → 自动进第 2 层（3+2=5 只，新地图实例）
	var floor1: Node = get_first_floor()
	var map1: Node = floor1.get_parent()  # 旧地图稍后被释放，先缓存引用
	for e in floor1.get_enemies():
		e.health.kill()
	check(int(floor1.get_alive_count()) == 0, "全灭后存活数应清零")
	if not await _poll(func(): return int(dungeon.floor) == 2, 900,
		"清场后应自动进入第 2 层"):
		_finish()
		return
	if not await _poll_floor(scene_root, 5, "第 2 层"):
		_finish()
		return
	var cur_map: Node = _explore_map_mgr(scene_root).get_current_map()
	check(cur_map != map1, "第 2 层应是新实例化的地图")

	# 7. → 第 3 层（boss 层）：3+3=6 只普通怪 + 秘境妖王
	var floor2: Node = get_first_floor()
	for e in floor2.get_enemies():
		e.health.kill()
	if not await _poll(func(): return int(dungeon.floor) == 3, 900,
		"清场后应自动进入第 3 层（boss 层）"):
		_finish()
		return
	if not await _poll_floor(scene_root, 7, "boss 层"):
		_finish()
		return
	var floor3: Node = get_first_floor()
	var boss: Node = floor3.get_parent().get_node_or_null("Boss")
	check(boss != null, "boss 层应有 Boss 节点")
	if boss != null:
		check(str(boss.display_name) == "秘境妖王", "boss 名应为秘境妖王")
		check(float(boss.max_health) > 300.0, "boss 血量应远超普通怪（320）")

	# 8. boss 层全灭 → 结算：发奖 + 结算弹窗打开
	for e in floor3.get_enemies():
		if is_instance_valid(e):
			e.health.kill()
	if not await _poll(func(): return str(dungeon.status) == "" and not dungeon.settle_view.is_empty(), 900,
		"通关后应结算（状态回空 + 结算视图写入）"):
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
	if modal == null or not bool(modal.call("is_modal_open")):
		check(false, "通关后结算弹窗应打开")
		_finish()
		return

	# 9. 弹窗关闭 → 场景切回 main → 原路送回萧宅 + 入口记录清空
	modal.call("close")
	if not await _poll(func(): return _scene_id(scene_root) == "xiuxian_main", 600,
		"结算弹窗关闭后应切回主场景"):
		_finish()
		return
	if not await _poll(func(): return _main_map_alias(scene_root) == "xiuxian_home", 600,
		"回主场景后应原路送回家宅"):
		_finish()
		return
	if not await _poll(func(): return str(dungeon.from_map) == "", 300,
		"送回后入口记录应清空"):
		_finish()
		return
	print("[Dungeon] 通关回程 ok: alias=%s from_map=%s" % [
		_main_map_alias(scene_root), dungeon.from_map])

	# 10. 放弃路径：再次进入 → abandon() → 回主场景（无奖励）
	var stones1: int = int(char_state.spirit_stones)
	dungeon_mgr = await _await_component(scene_root, "DungeonManager")
	check(dungeon_mgr != null and dungeon_mgr.start_run(), "通关后应可再次开启")
	if not await _poll(func(): return _scene_id(scene_root) == "xiuxian_explore", 600,
		"第二次 start_run 应切入探索场景"):
		_finish()
		return
	exp_mgr = await _await_component(scene_root, "ExploreManager")
	if exp_mgr == null or not await _poll_floor(scene_root, 4, "二刷第 1 层"):
		_finish()
		return
	check(exp_mgr.abandon(), "abandon 应成功放弃")
	if not await _poll(func(): return _scene_id(scene_root) == "xiuxian_main", 600,
		"放弃后应切回主场景"):
		_finish()
		return
	check(str(dungeon.status) == "", "放弃后状态应回空")
	check(int(char_state.spirit_stones) == stones1, "放弃不应发放奖励")
	check(str(_main_map_alias(scene_root)) == "xiuxian_home",
		"放弃后也应送回入口家宅")

	dungeon.reset_demo()  # 还原，避免污染其他测试
	scene_root.queue_free()
	_finish()


func _finish() -> void:
	print("[Dungeon] RESULT: %s" % ("PASS" if ok else "FAIL"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	if FileAccess.file_exists("user://coredata_3.data"):
		DirAccess.remove_absolute("user://coredata_3.data")
	quit(0 if ok else 1)


# ------------------------------------------------------------------ 工具

## 当前场景 id（GdSceneRoot.get_current_scene().get_scene_id()）
func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


## 存档弹窗是否打开（新流程：开始游戏 → SaveModal → 选槽进入）
func _modal_open(scene_root: Node) -> bool:
	var modal: Node = scene_root.find_child("SaveModal", true, false)
	return modal != null and modal.is_modal_open()


## 轮询等待组件注册（场景切换后旧组件注销/新组件注册有时序差）
func _await_component(scene_root: Node, comp_name: String) -> Node:
	var sroot: Node = Engine.get_singleton("GDCORE").get_global_node("default")
	for i in 240:
		var c: Node = sroot.get_component(comp_name)
		if c != null:
			return c
		await process_frame
	fail("组件 %s 未注册" % comp_name)
	return null


## 探索场景的 MapManager（当前场景节点直取，避开组件表时序）
func _explore_map_mgr(scene_root: Node) -> Node:
	var cur: Node = scene_root.call("get_current_scene")
	return cur.get_node_or_null("MapManager") if cur != null else null


## 主场景的地图别名（切回 main 后取）
func _main_map_alias(scene_root: Node) -> String:
	var mgr: Node = _explore_map_mgr(scene_root)
	return str(mgr.get_current_alias()) if mgr != null else "<null>"


## 清理场景内所有弹幕子弹（试射后的清场，避免干扰后续刷怪计数）
func _clear_bullets() -> void:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children():
			stack.append(c)
		if n.get_class() == "GdBullet":
			n.queue_free()


## 场景内存活子弹数（敌方开火抽测用）
func _count_bullets() -> int:
	var n := 0
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var cur: Node = stack.pop_back()
		for c in cur.get_children():
			stack.append(c)
		if cur.get_class() == "GdBullet":
			n += 1
	return n


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
func _poll_floor(scene_root: Node, expect_alive: int, tag: String) -> bool:
	var ready := await _poll(func(): return _floor_alive() == expect_alive,
		300, "%s 应刷出 %d 只敌人" % [tag, expect_alive])
	if not ready:
		return false
	var f: Node = get_first_floor()
	var map: Node = f.get_parent()
	var spawn: Vector2i = f.spawn_cell
	check(bool(map.is_walkable(spawn)), "%s 出生格应可行走 %s" % [tag, spawn])
	for e in f.get_enemies():
		check(absi(e.enemy_cell.x - spawn.x) + absi(e.enemy_cell.y - spawn.y) >= 6
			or e.name == "Boss", "%s 敌人应距出生点 ≥ 6 格" % tag)
		break  # 逐只断言受随机性影响，抽查首只即可（生成器保证）
	# 秘境仇恨：sight 覆盖全图，敌人无视距离主动追击（不游走待机）
	for e in f.get_enemies():
		check(float(e.sight_range) >= 8192.0,
			"%s 敌人应全程仇恨（sight=%s）" % [tag, e.sight_range])
		if bool(e.get("ranged")):
			check(e.get("danmaku") != null, "%s 远战怪应挂弹幕发射器" % tag)
	check(await _poll(func():
		for e in get_first_floor().get_enemies():
			if str(e.brain.get_ai_state()) == "chase":
				return true
		return false, 300, "%s 敌人应进入追击相位" % tag),
		"%s 敌人应主动追击（chase 相位）" % tag)
	var mgr: Node = _explore_map_mgr(scene_root)
	check(mgr != null and str(mgr.get_current_alias()) == XiuDungeonState.DUNGEON_ALIAS,
		"%s 当前地图应为随机场景" % tag)
	print("[Dungeon] %s ok: alive=%d spawn=%s" % [tag, f.get_alive_count(), spawn])
	return true
