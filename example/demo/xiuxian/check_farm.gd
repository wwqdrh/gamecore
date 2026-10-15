# 农田状态流转端到端验收 —— 5 号槽水壶 / 6 号槽种子 → GdDigLayer 农田层
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_farm.gd
#
# 状态机：草地 --锄→ 耕地(sand) --浇→ 湿润 --种→ 作物 --生长(仅湿润)→ 成熟 --收→ 耕地
#
# 覆盖：
#   A. 静态配置：5/6 号槽物品/图标/定义表（item.gjson 产物链）
#   B. 装备联动：equip=tool_watering_can → water 模式 / seeds_veg → plant 模式
#      （不走判定盒 mask=0、挥击路由开启、WeaponKettle/WeaponSeedBag 图形）
#   C. 框架农田 API：water/plant/grow/harvest 状态机全流程 + 信号 +
#      边界（草格不可浇/未湿润不可播/重复浇/未熟不可收）
#   D. 动作光标：set_cursor_for 按动作过滤（可浇/可播格才亮）
#   E. 持久化：dig_save 1s 轮询 diff 写回 XiuMapState.farm_states，
#      新实例进场 set_farm_state 恢复湿润 + 作物阶段
extends SceneTree

const MAP_SCENE := "res://example/demo/xiuxian/map/home.tscn"
const PLAYER_SCRIPT := "res://example/demo/xiuxian/role/player/player.gd"
const EQUIP_SCRIPT := "res://example/demo/xiuxian/scenes/main/ui/mainhud/mainhud_equipbar.gd"

## 桩 MapManager：player._try_water/_try_plant 仅依赖 get_current_map（鸭子类型）
class FakeMgr extends Node:
	var map: Node = null
	func get_current_map() -> Node:
		return map
	func get_current_alias() -> String:
		return "xiuxian_home"
	func try_teleport(_pos: Vector2) -> void:
		pass

var ok := true


func _initialize() -> void:
	# 看门狗：脚本错误中止协程时强制退出（防零输出假死）
	create_timer(90.0).timeout.connect(func() -> void:
		print("[Farm] WATCHDOG 超时——测试未完成，强制退出")
		quit(2))
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[Farm] OK  ", msg)
	else:
		push_error("[Farm] FAIL  " + msg)
		ok = false


func _wait_frames(n: int) -> void:
	for i in n:
		await process_frame


func _state():
	return Engine.get_singleton("GDSTATE")


func _finish() -> void:
	print("[Farm] RESULT: ", "ALL GREEN" if ok else "HAS FAILURES")
	quit(0 if ok else 1)


func _run() -> void:
	# 存档隔离：测试全程跑专用档，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var it := XiuItemState.ins()
	it.reset_demo()
	# 清空存档地形网格 + 农田状态（前次运行残留不带入，从初始配置起步）
	XiuMapState.ins().clear_terrain_grids()
	XiuMapState.ins().clear_farm_states()

	# ---- A. 静态配置 ----
	var slots: Array = load(EQUIP_SCRIPT).SLOT_ITEMS
	_check(slots.size() == 7 and str(slots[0]) == "" and str(slots[5]) == "tool_watering_can"
		and str(slots[6]) == "seeds_veg",
		"1 号空手 + 6/7 号槽应配置 tool_watering_can / seeds_veg（%s）" % str(slots))
	var kettle: Dictionary = it.get_item("tool_watering_can")
	_check(str(kettle.get("name", "")) == "青铜水壶" and str(kettle.get("type", "")) == "装备",
		"定义表应有青铜水壶（装备类），实际 %s" % str(kettle))
	var seeds: Dictionary = it.get_item("seeds_veg")
	_check(str(seeds.get("name", "")) == "青菜种子" and str(seeds.get("type", "")) == "消耗",
		"定义表应有青菜种子（消耗类），实际 %s" % str(seeds))
	_check(XiuIconMap.resolve("img_tool_watering_can") == "🪣"
		and XiuIconMap.resolve("img_seeds") == "🌱", "图标注册表应解析 🪣/🌱")

	# ---- B. 装备联动 ----
	var map: Node = load(MAP_SCENE).instantiate()
	# 测试用生长参数：改 def 属性（实时生效）青菜 0.1s/阶段（4 阶段 → 播种后
	# ~0.4s 成熟）；湿润 90s 不干燥。def 为 tscn 内嵌共享资源，仅本进程生效
	root.add_child(map)
	await _wait_frames(4)
	var defs0: Array = map.get("crops")
	defs0[0].set("grow_secs", 0.1)

	var player: Node = load(PLAYER_SCRIPT).new()
	root.add_child(player)
	var fake := FakeMgr.new()
	fake.map = map
	root.add_child(fake)
	player.set("map_mgr", fake)
	# 站 (8,6)，面向右挥击目标恒为 (9,6)（8 邻格 45° 扇区）
	player.set("global_position", map.call("cell_to_center", Vector2i(8, 6)))
	await _wait_frames(3)

	_state().set_state("mainhud.equip", "tool_watering_can")
	await _wait_frames(1)
	_check(str(player.get("attack_mode")) == "water",
		"装备水壶应进入 water 模式，实际 %s" % str(player.get("attack_mode")))
	var melee: Node = player.get_node("Melee")
	_check(int(melee.get("target_mask")) == 0 and bool(melee.get("auto_attack_mouse")),
		"浇水模式不走判定盒（mask=0）且挥击路由开启")
	_check(player.get_node("WeaponMount").get_node_or_null("WeaponKettle") != null,
		"WeaponMount 应装配水壶图形（WeaponKettle）")

	_state().set_state("mainhud.equip", "seeds_veg")
	await _wait_frames(1)
	_check(str(player.get("attack_mode")) == "plant",
		"装备种子应进入 plant 模式，实际 %s" % str(player.get("attack_mode")))
	_check(player.get_node("WeaponMount").get_node_or_null("WeaponSeedBag") != null,
		"WeaponMount 应装配种子袋图形（WeaponSeedBag）")

	# ---- C. 框架农田 API（先手动锄出耕地） ----
	var C := Vector2i(9, 6)
	_check(map.call("get_terrain_name_at", C) == "grass", "初始 (9,6) 应为 grass")
	_check(map.call("dig", C), "锄 (9,6) 应成功（grass→sand 耕地）")
	_check(not map.call("is_waterable", Vector2i(8, 6)), "草地不可浇（只认耕地 sand）")
	_check(map.call("is_waterable", C), "耕地 (9,6) 应可浇")
	_check(not map.call("water", Vector2i(8, 6)), "浇草地应失败")

	var sigs := {"watered": 0, "planted": 0, "harvested": 0}
	map.connect("s_watered", func(_c) -> void: sigs["watered"] += 1)
	map.connect("s_planted", func(_c) -> void: sigs["planted"] += 1)
	map.connect("s_harvested", func(_c) -> void: sigs["harvested"] += 1)

	_check(map.call("water", C), "首次浇水应成功")
	_check(map.call("is_wet", C), "浇水后 (9,6) 应湿润")
	_check(not map.call("water", C), "已湿润格重复浇应失败（换水不刷计时由框架冷却）")
	_check(not map.call("is_plantable", Vector2i(8, 6)), "未湿润格不可播种")
	_check(map.call("is_plantable", C), "湿润耕地应可播种")

	_check(map.call("plant", C), "播种 (9,6) 应成功")
	_check(int(map.call("get_crop_stage", C)) == 0, "播种后作物阶段应为 0")
	_check(not map.call("is_plantable", C), "已有作物的格不可再播")
	_check(not map.call("is_mature", C), "阶段 0 未成熟")
	_check(not map.call("harvest", C), "未成熟收获应失败")

	# ---- 生长：仅湿润累计，0.1s/阶段 → 0.4s 后成熟（阶段 3） ----
	await create_timer(0.8).timeout
	_check(int(map.call("get_crop_stage", C)) == 3,
		"0.8s 后作物应到成熟阶段 3（0.1s/阶段×4），实际 %s"
			% str(map.call("get_crop_stage", C)))
	_check(map.call("is_mature", C), "成熟判定应通过")
	_check(map.call("harvest", C), "成熟收获应成功")
	_check(int(map.call("get_crop_stage", C)) == -1, "收获后作物应移除（阶段 -1）")
	_check(map.call("is_wet", C), "收获后耕地保留湿润（可立即补种）")
	_check(sigs["watered"] >= 1 and sigs["planted"] == 1 and sigs["harvested"] == 1,
		"信号计数 watered≥1 / planted=1 / harvested=1，实际 %s" % str(sigs))

	# ---- D. 动作光标：set_cursor_for 按动作过滤 ----
	var cursor: Node2D = map.get_node_or_null("ActionCursor")
	_check(cursor != null, "应创建 ActionCursor 光标组件")
	if cursor != null:
		map.call("set_cursor_for", Vector2i(8, 6), "dig")
		_check(cursor.visible, "dig 光标：可锄草格 → 显示")
		map.call("set_cursor_for", C, "dig")
		_check(not cursor.visible, "dig 光标：耕地不可再锄 → 隐藏")
		map.call("set_cursor_for", Vector2i(8, 6), "water")
		_check(not cursor.visible, "water 光标：草地不可浇 → 隐藏")
		map.call("set_cursor_for", C, "water")
		_check(not cursor.visible, "water 光标：已湿润格 → 隐藏（负反馈）")
		# 补种一次（有作物）→ plant 光标应隐藏；湿润空耕地 (16,4) → 显示
		_check(map.call("plant", C), "湿润耕地补种应成功")
		map.call("set_cursor_for", C, "plant")
		_check(not cursor.visible, "plant 光标：已有作物格 → 隐藏")
		map.call("dig", Vector2i(16, 4))
		map.call("water", Vector2i(16, 4))
		map.call("set_cursor_for", Vector2i(16, 4), "plant")
		_check(cursor.visible, "plant 光标：湿润空耕地 → 显示")
		map.call("clear_cursor")
	# 等补种作物成熟后收获，(9,6) 回归「湿润空耕地」（E 节存档断言依赖）；
	# 顺带二次验证成熟收获
	await create_timer(0.5).timeout
	_check(map.call("harvest", C), "补种作物成熟后收获应成功")
	_check(int(map.call("get_crop_stage", C)) == -1
		and map.call("is_wet", C), "收获后 (9,6) 回归湿润空耕地")

	# ---- D2. 玩家挥击端到端：水壶挥击浇水 / 种子挥击播种与收获 ----
	var C3 := Vector2i(9, 8)
	map.call("dig", C3)  # 预先锄出耕地
	_state().set_state("mainhud.equip", "tool_watering_can")
	await _wait_frames(1)
	player.set("global_position", map.call("cell_to_center", Vector2i(8, 8)))
	await create_timer(0.6).timeout  # 挥击冷却走完
	_check(not map.call("is_wet", C3), "挥击前 (9,8) 应未湿润")
	melee.call("swing", Vector2.RIGHT)
	await _wait_frames(2)
	_check(map.call("is_wet", C3), "水壶挥击前方一格应浇水成功（(9,8) 湿润）")

	_state().set_state("mainhud.equip", "seeds_veg")
	await _wait_frames(1)
	await create_timer(0.6).timeout
	melee.call("swing", Vector2.RIGHT)
	await _wait_frames(2)
	_check(int(map.call("get_crop_stage", C3)) == 0, "种子挥击前方一格应播种成功（阶段 0）")
	await create_timer(0.8).timeout  # 生长至成熟（0.1s/阶段）
	melee.call("swing", Vector2.RIGHT)  # plant 模式对成熟格 = 收获
	await _wait_frames(2)
	_check(int(map.call("get_crop_stage", C3)) == -1,
		"挥击成熟作物应收获移除（阶段 -1），实际 %s" % str(map.call("get_crop_stage", C3)))

	# ---- D3. 空手采摘入包：✋ 槽 equip="" → 点击相邻成熟作物 harvest + add_item ----
	_state().set_state("mainhud.equip", "")
	await _wait_frames(1)
	_check(str(player.get("attack_mode")) == "", "空手槽应使玩家为徒手模式（attack_mode=\"\"）")
	_check(map.call("plant", C), "种一株作物供空手采摘（(9,6)）")
	await create_timer(0.5).timeout  # 长到成熟
	player.set("global_position", map.call("cell_to_center", Vector2i(8, 6)))  # 走到 (9,6) 西邻
	var bag0: int = int(it.bag.get("veg_green", 0))
	_check(bool(player.call("_harvest_at", map, C)),
		"空手点击相邻成熟作物应采摘成功")
	_check(int(it.bag.get("veg_green", 0)) == bag0 + 1,
		"采摘后青菜应入包（%d → %d）" % [bag0, int(it.bag.get("veg_green", 0))])
	_check(int(map.call("get_crop_stage", C)) == -1, "采摘后作物应移除")
	# 远格不消费点击（放行寻路）、不入包：种 (16,4) 成熟后远距离点击
	_check(map.call("plant", Vector2i(16, 4)), "种远格作物 (16,4)")
	await create_timer(0.5).timeout
	var bag1: int = int(it.bag.get("veg_green", 0))
	_check(not bool(player.call("_harvest_at", map, Vector2i(16, 4))),
		"远格成熟作物不应被直接采摘（提示走近，点击放行寻路）")
	_check(int(it.bag.get("veg_green", 0)) == bag1, "远格未摘不应入包")
	# 脚下格：玩家站进作物格直接摘
	_check(map.call("plant", Vector2i(9, 8)), "种脚下格作物 (9,8)")
	await create_timer(0.5).timeout
	player.set("global_position", map.call("cell_to_center", Vector2i(9, 8)))
	_check(bool(player.call("_harvest_at", map, Vector2i(9, 8))),
		"站在作物格上点击脚下应可采摘")
	_check(int(it.bag.get("veg_green", 0)) == bag1 + 1, "脚下采摘应入包")
	# 未成熟格不放行（false 但无副作用）；定义表/图标链路
	_check(map.call("plant", C) and int(map.call("get_crop_stage", C)) == 0,
		"再种一株未成熟作物")
	_check(not bool(player.call("_harvest_at", map, C)), "未成熟作物不可采摘")
	var veg: Dictionary = it.get_item("veg_green")
	_check(str(veg.get("name", "")) == "青菜" and str(veg.get("type", "")) == "材料",
		"定义表应有青菜（材料类），实际 %s" % str(veg))
	_check(XiuIconMap.resolve("img_veg_green") == "🥬", "青菜图标应解析 🥬")
	player.set("global_position", map.call("cell_to_center", Vector2i(8, 6)))
	# 收掉测试作物，(9,6) 回归湿润空耕地（E 节存档断言依赖「最终状态」写回）
	await create_timer(0.4).timeout
	map.call("harvest", C)

	# ---- D4. 空手点击全链路：_on_map_clicked（find_path 守卫次序回归）----
	# 回归前提：GdDigLayer 小地图没有 find_path（BFS 是 GdQuickMap 的能力）——
	# 曾因守卫顺序错误（find_path 前置）把 home 的徒手采摘整个吞掉（实机
	# 点击无反应，headless 直调 _harvest_at 绕过守卫测不出来）
	_check(not map.has_method("find_path"),
		"GdDigLayer 应无 find_path（回归前提）")
	_check(map.call("plant", C), "D4：种一株成熟作物供点击采摘")
	await create_timer(0.5).timeout
	player.set("_wired", true)  # 测试桩：_on_map_clicked 的接线守卫
	# headless 无真实鼠标（warp_mouse/push_input 均无法驱动视口鼠标位置）：
	# 点击分流逻辑在 _on_map_clicked_at(map, target)，直调任意格覆盖全守卫链
	var cursor2: Node2D = map.get_node_or_null("ActionCursor")
	if cursor2 != null:
		map.call("set_cursor_for", C, "harvest")
		_check(cursor2.visible, "harvest 光标：成熟作物格 → 显示")
		map.call("set_cursor_for", Vector2i(8, 6), "harvest")
		_check(not cursor2.visible, "harvest 光标：无作物格 → 隐藏（负反馈）")
		map.call("clear_cursor")
	var bagc: int = int(it.bag.get("veg_green", 0))
	player.call("_on_map_clicked_at", map, C)
	await _wait_frames(2)
	_check(int(it.bag.get("veg_green", 0)) == bagc + 1,
		"空手点击全链路（_on_map_clicked_at → _harvest_at）应采摘入包")
	_check(int(map.call("get_crop_stage", C)) == -1, "点击采摘后作物应移除")
	player.set("global_position", map.call("cell_to_center", Vector2i(8, 6)))

	# ---- F. 多作物：crops 数组 def + 按种子播种 + 各自节奏 + 收获物分流 ----
	var defs: Array = map.get("crops")
	_check(defs.size() == 3, "home.tscn 应配置 3 种作物 def，实际 %d" % defs.size())
	if defs.size() == 3:
		_check(str(defs[0].get("seed_id")) == "seeds_veg"
			and str(defs[1].get("seed_id")) == "seeds_radish"
			and str(defs[2].get("seed_id")) == "seeds_melon",
			"三种 def 的 seed_id 应分别为 veg/radish/melon")
		_check(int(defs[1].get("stages")) == 3 and int(defs[2].get("stages")) == 5,
			"萝卜 3 阶段 / 灵瓜 5 阶段（各自节奏）")
	var C4 := Vector2i(15, 4)
	map.call("dig", C4)
	map.call("water", C4)
	_check(not bool(map.call("plant_seed", C4, "seeds_不存在的")),
		"未知种子 id 播种应失败")
	_check(bool(map.call("plant_seed", C4, "seeds_radish")),
		"萝卜种子播种应成功（按 seed_id 匹配 def）")
	_check(str(map.call("get_crop_id", C4)) == "veg_radish",
		"格上作物种类应为 veg_radish，实际 %s" % str(map.call("get_crop_id", C4)))
	_check(not bool(map.call("is_mature", C4)), "萝卜未成熟")
	# def 属性实时生效：改萝卜节奏（stages=2 / 0.05s）后立即快熟（测试提速）
	defs[1].set("stages", 2)
	defs[1].set("grow_secs", 0.05)
	await create_timer(0.3).timeout
	_check(bool(map.call("is_mature", C4)), "改 def 参数后萝卜应快速成熟")
	# 持久化：轮询写回的作物条目带 id（1.0 = radish 下标）
	await create_timer(1.3).timeout
	var saved_f: Dictionary = XiuMapState.ins().load_farm_state("xiuxian_home")
	var saved_cf: Dictionary = saved_f.get("crops", {}).get("15,4", {})
	_check(int(saved_cf.get("id", -1.0)) == 1, "作物条目应带种类 id（radish=1），实际 %s" % str(saved_cf))
	# 收获分流：萝卜 → veg_radish 入包（不是青菜）
	player.set("_wired", true)
	player.set("global_position", map.call("cell_to_center", Vector2i(14, 4)))
	var rad0: int = int(it.bag.get("veg_radish", 0))
	_check(bool(player.call("_harvest_at", map, C4)), "空手采摘成熟萝卜应成功")
	_check(int(it.bag.get("veg_radish", 0)) == rad0 + 1, "萝卜应入包（veg_radish +1）")
	defs[1].set("stages", 3)  # 还原 def（共享资源，别污染后续加载）
	defs[1].set("grow_secs", 20.0)

	# ---- E. 持久化：dig_save 1s 轮询 diff 写回 + 新实例进场恢复 ----
	# 此刻 (9,6)：湿润 + 成熟作物（上面 D 节末补种后收掉 → 湿润空耕地）
	# 再种一格 (9,7) 留成熟作物在存档里，验证双状态恢复
	var C2 := Vector2i(9, 7)
	map.call("dig", C2)
	map.call("water", C2)
	map.call("plant", C2)
	await create_timer(1.4).timeout  # > FARM_POLL_SECS，轮询应已 diff 写回
	var saved: Dictionary = XiuMapState.ins().load_farm_state("xiuxian_home")
	var saved_wet: Dictionary = saved.get("wet", {})
	var saved_crops: Dictionary = saved.get("crops", {})
	_check(not saved.is_empty() and saved_wet.has("9,6") and saved_wet.has("9,7"),
		"轮询应把湿润格写回 Bean（%s）" % str(saved_wet.keys()))
	_check(saved_crops.has("9,7"), "作物状态应写回 Bean（%s）" % str(saved_crops.keys()))
	map.queue_free()
	fake.map = null  # 断开桩引用：防 player._process 拿已释放地图（_apply_camera 报错刷屏）
	await _wait_frames(2)

	var map2: Node = load(MAP_SCENE).instantiate()
	root.add_child(map2)
	await _wait_frames(4)  # DigSave deferred restore 需等一帧
	_check(map2.call("is_wet", C), "新实例进场应恢复湿润状态（(9,6)）")
	_check(map2.call("is_wet", C2), "新实例进场应恢复湿润状态（(9,7)）")
	_check(int(map2.call("get_crop_stage", C2)) >= 0,
		"新实例进场应恢复作物（(9,7) 阶段 %s）" % str(map2.call("get_crop_stage", C2)))
	_check(int(map2.call("get_crop_stage", C)) == -1, "无作物格恢复为空（(9,6)）")
	# 恢复后生长引擎应继续运转（湿润 + 作物 → set_process(true)）；
	# 改 def 参数（共享资源实时生效）提速
	var defs2: Array = map2.get("crops")
	defs2[0].set("grow_secs", 0.05)
	await create_timer(0.5).timeout
	_check(int(map2.call("get_crop_stage", C2)) == 3,
		"恢复后作物应继续生长至成熟（实际 %s）" % str(map2.call("get_crop_stage", C2)))
	map2.queue_free()
	await _wait_frames(2)

	player.queue_free()
	fake.queue_free()
	await _wait_frames(2)
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	_finish()
