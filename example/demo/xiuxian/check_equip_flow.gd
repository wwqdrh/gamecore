# 装备栏端到端验收 —— GdUIHotbar（数字键/点击选中 + 高亮框）+ GdState 装备联动
#
# 运行：perl -e 'alarm 180; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_equip_flow.gd
#
# 覆盖：
#   1. 装配：底部装备栏 7 格 + 数字键绑定 + 初始选中第 1 格（空手 ✋，equip=""），
#      选中格 3px 高亮描边
#   2. 数字键选择：按 2 → 枪支（射击模式）；按 3 → 剑（近战）；按 4 → 镐（挖矿
#      mask=16）；按 5 → 锄（锄地 mask=0）；状态直写空串 → 空手；
#   2b. 空手槽回归：按 1 → equip="" 徒手（相邻成熟作物可采摘，见 check_farm.gd D3）
#   3. 射击能力：持枪时 shooter.fire_toward_mouse 发出子弹（s_fired 计数）
#   3b. 近战能力：近战模式下 melee.swing_toward_mouse 挥击（s_swing 计数）
#   5. 点击选择：向槽位 3 发左键 → 选中格 3
#   6. 边界：越界数字键（7 格时按 8）不改变选中
extends SceneTree

var ok := true


func fail(msg: String) -> void:
	push_error("[Equip] " + msg)
	ok = false


func check(cond: bool, msg: String) -> void:
	if not cond:
		fail(msg)


func _initialize() -> void:
	_run()


func _state():
	return Engine.get_singleton("GDSTATE")


func _key_event(keycode: Key, pressed: bool) -> void:
	var ev := InputEventKey.new()
	ev.physical_keycode = keycode
	ev.keycode = keycode
	ev.pressed = pressed
	Input.parse_input_event(ev)
	Input.flush_buffered_events()  # 同步派发（headless 帧节奏不定，等待不可靠）


func _press_digit(keycode: Key) -> void:
	_key_event(keycode, true)
	_key_event(keycode, false)
	await physics_frame


func _release_click(pos: Vector2) -> InputEventMouseButton:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = false
	ev.position = pos
	ev.global_position = pos
	return ev


func _run() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	# 1. 装配
	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	check(main_scene != null, "scenes/main/index.tscn 应可加载")
	if main_scene == null:
		_finish()
		return
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	for i in 10:
		await physics_frame

	var player: Node = index.get_node_or_null("Player")
	check(player != null, "主场景应挂载 Player")
	var hud: Node = index.get_node_or_null("UILayer/MainUI")
	check(hud != null, "UILayer 下应有 MainUI 组合根")
	if player == null or hud == null:
		_finish()
		return
	var hotbar: Node = hud.find_child("EquipSlots", true, false)
	check(hotbar != null, "装备栏应包含 Hotbar（EquipSlots）")
	if hotbar == null:
		_finish()
		return
	check(int(hotbar.get("slot_count")) == 7, "装备栏应为 7 格")
	check(bool(hotbar.get("key_bind")), "Hotbar 应启用数字键绑定")
	check(int(hotbar.call("get_selected")) == 0, "初始应选中第 1 格（枪支）")

	# 高亮框：选中格描边 3px，其余 1px
	var slots := hotbar.get_children()
	check(slots.size() == 7, "Hotbar 应有 7 个槽位 Panel")
	if slots.size() == 7:
		var sb0: StyleBox = slots[0].get_theme_stylebox("panel")
		var sb1: StyleBox = slots[1].get_theme_stylebox("panel")
		check(sb0 is StyleBoxFlat and int(sb0.border_width_left) == 3,
			"选中格应为 3px 高亮描边")
		check(sb1 is StyleBoxFlat and int(sb1.border_width_left) == 1,
			"未选中格应为 1px 描边")
		# 图标占位与键位角标
		var glyph0: Label = slots[0].find_child("Glyph", true, false)
		check(glyph0 != null and glyph0.text == "✋", "第 1 格应显示空手图标占位")
		var glyph1: Label = slots[1].find_child("Glyph", true, false)
		check(glyph1 != null and glyph1.text == "🔫", "第 2 格应显示枪支图标占位")
		var glyph2: Label = slots[2].find_child("Glyph", true, false)
		check(glyph2 != null and glyph2.text == "🗡️", "第 3 格应显示近战武器图标占位")
		var glyph4: Label = slots[4].find_child("Glyph", true, false)
		check(glyph4 != null and glyph4.text == "🔨", "第 5 格应显示锄头图标占位")
		var glyph5: Label = slots[5].find_child("Glyph", true, false)
		check(glyph5 != null and glyph5.text == "🪣", "第 6 格应显示水壶图标占位")
		var glyph6: Label = slots[6].find_child("Glyph", true, false)
		check(glyph6 != null and glyph6.text == "🌱", "第 7 格应显示种子图标占位")
		var key2: Label = slots[1].find_child("KeyNum", true, false)
		check(key2 != null and key2.text == "2", "第 2 格应显示键位角标 2")

	# 2. 初始装备联动（equipbar ready 补报 → 玩家 watch 同步）：1 号空手默认选中
	check(str(_state().get_state("mainhud.equip")) == "",
		"初始状态 mainhud.equip 应为空串（空手），实际 %s" % str(_state().get_state("mainhud.equip")))
	check(not bool(player.gun_equipped), "初始应空手（1 号槽 ✋ 默认选中，手上什么都不拿）")
	check(str(player.attack_mode) == "", "空手玩家 attack_mode 应为空串，实际 %s" % str(player.attack_mode))
	# 武器装配：空手 → WeaponMount 不挂任何图形
	var mount: Node2D = player.get_node_or_null("WeaponMount")
	check(mount != null, "玩家应有 WeaponMount 武器挂点")
	check(mount != null and mount.get_node_or_null("WeaponGun") == null,
		"空手时 WeaponMount 不应装配枪图形")

	# 2b. 鼠标穿透回归：组合根/场景根不得拦截或悬停——否则 gui_get_hovered_control
	#     恒非 null，玩家"无 UI 悬停才开火"分支永不成立（真实窗口点枪无子弹）
	#     注：DemoRoot 是 main.gml.tscn 的根，实例化到 index.tscn 后节点名 = MainUI
	var demo_root := hud as Control
	check(demo_root != null, "组合根 DemoRoot（实例节点名 MainUI）应存在")
	if demo_root != null:
		check(int(demo_root.mouse_filter) == Control.MOUSE_FILTER_IGNORE,
			"组合根 DemoRoot 应为 mouse_filter IGNORE（穿透），实际 %d" % int(demo_root.mouse_filter))
	check(int((index as Control).mouse_filter) == Control.MOUSE_FILTER_IGNORE,
		"GdScene 场景根应为 mouse_filter IGNORE（穿透），实际 %d" % int((index as Control).mouse_filter))
	# 事件路由回归：持枪时框架经 unhandled 路由自动开火——点击 UI（STOP 消费）
	# 不开火；下面第 3 步直接调 fire_toward_mouse 验证发射能力本身。
	# 模拟鼠标移动到 HUD 覆盖不到的开阔点 → hover 应为 null（鼠标穿透失效会让
	# PASS/STOP 全屏壳层出现在 hover 链上）
	var motion := InputEventMouseMotion.new()
	motion.position = Vector2(60, 8)
	motion.global_position = Vector2(60, 8)
	Input.parse_input_event(motion)
	Input.flush_buffered_events()
	await physics_frame
	check(player.get_viewport().gui_get_hovered_control() == null,
		"开阔点悬停应无 UI 控件（全屏壳层未穿透会被报为 hovered）")

	# 2b-2. 按 2 切枪支槽（初始空手，持枪后再验开火路由）
	await _press_digit(KEY_2)
	check(int(hotbar.call("get_selected")) == 1, "按 2 应选中第 2 格")
	check(str(_state().get_state("mainhud.equip")) == "gun", "按 2 后状态应为 gun")
	check(bool(player.gun_equipped), "按 2 后玩家应持枪")
	check(mount != null and mount.get_node_or_null("WeaponGun") != null,
		"持枪后 WeaponMount 应装配枪图形（WeaponGun）")

	# 2c. 开火路由回归：持枪（auto_fire_mouse=true）下点击开阔点 → unhandled
	#     路由开火；点击 UI 按钮槽位 → GUI 消费不开火
	var shooter_rt: Node = player.get_node_or_null("Shooter")
	check(shooter_rt != null, "玩家应装配 GdShooter")
	check(bool(shooter_rt.get("auto_fire_mouse")), "持枪时 GdShooter 应开启 unhandled 开火路由")
	var routed := [0]
	if shooter_rt != null:
		shooter_rt.connect("s_fired", func(_m, _d) -> void: routed[0] += 1)
		var world_click := InputEventMouseButton.new()
		world_click.button_index = MOUSE_BUTTON_LEFT
		world_click.pressed = true
		world_click.position = Vector2(60, 8)
		world_click.global_position = Vector2(60, 8)
		Input.parse_input_event(world_click)
		Input.flush_buffered_events()
		await physics_frame
		check(routed[0] >= 1, "持枪点击世界（unhandled 路由）应开火")
		Input.parse_input_event(_release_click(Vector2(60, 8)))
		Input.flush_buffered_events()
		await physics_frame
	# 3. 持枪射击：s_fired 计数（2c 路由测试刚开过一枪，先等冷却走完）
	await create_timer(0.4).timeout
	await physics_frame
	var shooter: Node = player.get_node_or_null("Shooter")
	check(shooter != null, "玩家应装配 GdShooter")
	var fired := [0]
	if shooter != null:
		shooter.connect("s_fired", func(_m, _d) -> void: fired[0] += 1)
		var shot_ok: bool = shooter.fire_toward_mouse()
		check(shot_ok, "持枪时 fire_toward_mouse 应发射成功")
		await physics_frame
		check(fired[0] >= 1, "射击后应发出 s_fired 信号")

	# 4. 数字键 3 → 近战武器槽 → 近战模式（卸枪 + 开挥击 + 剑图形装配）
	await _press_digit(KEY_3)
	check(int(hotbar.call("get_selected")) == 2, "按 3 应选中第 3 格")
	check(str(_state().get_state("mainhud.equip")) == "sword_qingfeng", "按 3 后状态应为 sword_qingfeng")
	check(not bool(player.gun_equipped), "选近战武器后玩家不应持枪")
	check(str(player.attack_mode) == "melee", "玩家应进入近战攻击模式，实际 %s" % str(player.attack_mode))
	var shooter_after: Node = player.get_node_or_null("Shooter")
	var melee_after: Node = player.get_node_or_null("Melee")
	check(shooter_after != null and not bool(shooter_after.get("auto_fire_mouse")),
		"近战模式下射击路由应关闭")
	check(melee_after != null and bool(melee_after.get("auto_attack_mouse")),
		"近战模式下挥击路由应开启")
	check(mount != null and mount.get_node_or_null("WeaponSword") != null,
		"近战模式下 WeaponMount 应装配剑图形（WeaponSword）")
	check(mount != null and mount.get_node_or_null("WeaponGun") == null,
		"近战模式下枪图形应卸下")

	# 3b. 近战挥击：s_swing 计数（判定窗口 0.12s，直调路由入口）
	if melee_after != null:
		var swung := [0]
		melee_after.connect("s_swing", func(_d) -> void: swung[0] += 1)
		var swing_ok: bool = melee_after.swing_toward_mouse()
		check(swing_ok, "近战模式 swing_toward_mouse 应挥击成功")
		await physics_frame
		check(swung[0] >= 1, "挥击后应发出 s_swing 信号")

	# 3. 按 4 → 挖矿镐槽 → 挖矿模式（判定盒指向 deco 受击盒 layer5/16）
	await _press_digit(KEY_4)
	check(int(hotbar.call("get_selected")) == 3, "按 4 应选中第 4 格")
	check(str(_state().get_state("mainhud.equip")) == "tool_pickaxe", "按 4 后状态应为 tool_pickaxe")
	check(str(player.attack_mode) == "mine", "挖矿镐玩家应进入挖矿模式，实际 %s" % str(player.attack_mode))
	var melee_mine: Node = player.get_node_or_null("Melee")
	check(melee_mine != null and int(melee_mine.get("target_mask")) == 16,
		"挖矿模式判定盒应指向 deco 受击盒（mask=16），实际 %s" % str(melee_mine.get("target_mask")))
	check(mount != null and mount.get_node_or_null("WeaponPickaxe") != null,
		"挖矿模式 WeaponMount 应装配镐图形（WeaponPickaxe）")

	# 3a. 状态直写空串 → 徒手（卸装备路径；hotbar 选中格不变）
	_state().set_state("mainhud.equip", "")
	check(str(player.attack_mode) == "", "空装备玩家应徒手，实际 %s" % str(player.attack_mode))
	await physics_frame  # 武器图形卸载走 queue_free，需等一帧
	check(mount != null and mount.get_node_or_null("WeaponPickaxe") == null,
		"徒手时镐图形应卸下")

	# 4. 按 5 → 锄头槽 → 锄地模式（不走判定盒 mask=0，锄挖逻辑挂 s_swing 信号）
	await _press_digit(KEY_5)
	check(int(hotbar.call("get_selected")) == 4, "按 5 应选中第 5 格")
	check(str(_state().get_state("mainhud.equip")) == "tool_hoe", "按 5 后状态应为 tool_hoe")
	check(str(player.attack_mode) == "till", "锄头玩家应进入锄地模式，实际 %s" % str(player.attack_mode))
	var melee_till: Node = player.get_node_or_null("Melee")
	check(melee_till != null and int(melee_till.get("target_mask")) == 0,
		"锄地模式不走判定盒（mask=0），实际 %s" % str(melee_till.get("target_mask")))
	check(melee_till != null and bool(melee_till.get("auto_attack_mouse")),
		"锄地模式挥击路由应开启")
	check(mount != null and mount.get_node_or_null("WeaponHoe") != null,
		"锄地模式 WeaponMount 应装配锄图形（WeaponHoe）")

	# 4b. 按 2 → 回到枪支（重装备路径），再按 5 回到锄头（后续越界检查基准）
	await _press_digit(KEY_2)
	check(str(_state().get_state("mainhud.equip")) == "gun", "按 2 应回到枪支槽")
	check(bool(player.gun_equipped), "回到枪支槽后玩家应持枪")
	check(mount != null and mount.get_node_or_null("WeaponGun") != null,
		"回到枪支槽后应重新装配枪图形")
	await _press_digit(KEY_5)
	check(str(_state().get_state("mainhud.equip")) == "tool_hoe", "再按 5 应回到锄头槽")

	# 4c. 空手槽回归：按 1 → equip="" 徒手（采摘交互见 check_farm.gd D3）
	await _press_digit(KEY_1)
	check(int(hotbar.call("get_selected")) == 0, "按 1 应回到第 1 格（空手）")
	check(str(_state().get_state("mainhud.equip")) == "", "按 1 后状态应为空串（空手）")
	check(str(player.attack_mode) == "", "空手玩家 attack_mode 应为空串")
	check(mount != null and mount.get_node_or_null("WeaponGun") == null,
		"回到空手槽后武器图形应卸下")
	await _press_digit(KEY_5)
	check(str(_state().get_state("mainhud.equip")) == "tool_hoe", "再按 5 回到锄头槽")

	# 5. 按 6 → 水壶槽 → 浇水模式；按 7 → 种子槽 → 播种模式（农田链路装备联动）
	await _press_digit(KEY_6)
	check(int(hotbar.call("get_selected")) == 5, "按 6 应选中第 6 格")
	check(str(_state().get_state("mainhud.equip")) == "tool_watering_can",
		"按 6 后状态应为 tool_watering_can")
	check(str(player.attack_mode) == "water", "水壶玩家应进入浇水模式，实际 %s" % str(player.attack_mode))
	check(mount != null and mount.get_node_or_null("WeaponKettle") != null,
		"浇水模式 WeaponMount 应装配水壶图形（WeaponKettle）")
	await _press_digit(KEY_7)
	check(int(hotbar.call("get_selected")) == 6, "按 7 应选中第 7 格")
	check(str(_state().get_state("mainhud.equip")) == "seeds_veg", "按 7 后状态应为 seeds_veg")
	check(str(player.attack_mode) == "plant", "种子玩家应进入播种模式，实际 %s" % str(player.attack_mode))
	check(mount != null and mount.get_node_or_null("WeaponSeedBag") != null,
		"播种模式 WeaponMount 应装配种子袋图形（WeaponSeedBag）")

	# 5b. 种子轮换：选中种子槽再按 7 → 轮换下一种种子（图标/状态同步）
	await _press_digit(KEY_7)
	check(str(_state().get_state("mainhud.equip")) == "seeds_radish",
		"再按 7 应轮换到 seeds_radish，实际 %s" % str(_state().get_state("mainhud.equip")))
	check(str(player.attack_mode) == "plant", "轮换后仍应处于播种模式")
	check(str(slots[6].find_child("Glyph", true, false).text) == "🥕",
		"轮换后种子槽图标应更新为 🥕")
	await _press_digit(KEY_7)
	check(str(_state().get_state("mainhud.equip")) == "seeds_melon",
		"再按 7 应轮换到 seeds_melon")
	await _press_digit(KEY_7)
	check(str(_state().get_state("mainhud.equip")) == "seeds_veg",
		"轮换到末尾应回到 seeds_veg（循环）")

	# 6. 越界数字键：7 格时按 8 不改变选中
	await _press_digit(KEY_8)
	check(int(hotbar.call("get_selected")) == 6, "按 8（越界）不应改变选中（仍为第 7 格）")

	# 7. 点击选择：向槽位 3（剑）发左键 → 选中格 3（0 起）
	var click := InputEventMouseButton.new()
	click.button_index = MOUSE_BUTTON_LEFT
	click.pressed = true
	click.position = slots[2].get_global_rect().get_center()
	slots[2].get_viewport().push_input(click)
	await physics_frame
	await physics_frame
	check(int(hotbar.call("get_selected")) == 2, "点击槽位 3 应选中第 3 格")
	check(str(_state().get_state("mainhud.equip")) == "sword_qingfeng", "点击后状态应为 sword_qingfeng")

	if ok:
		print("[Equip] RESULT=PASS")
	else:
		print("[Equip] RESULT=FAIL")
	_finish()


func _finish() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
