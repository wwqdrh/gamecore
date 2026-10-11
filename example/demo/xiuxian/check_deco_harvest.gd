# 矿石 deco + 挖矿入库 + 背包展示 端到端验收 —— GdDecoLayer 矿石铺设/采集回调/XiuItemState 背包联动
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_deco_harvest.gd
#
# 覆盖：
#   1. 场景装配：xiaozhai/xiaozhen 含 DecoLayer(GdDecoLayer) + DecoHarvest 回调宿主
#   2. 矿石铺设：频率/地形配置下有矿石生成，落点均在可行走的配置地形上，
#      受击盒 collision_layer=16（layer5 可采集物）
#   3. 采集协议：矿石被敲 3 次（GdHurtbox.take_damage 逐次）→ 血量归零 →
#      按名回调 deco_harvest.gd on_ore_harvested → XiuItemState.add_item 入包
#   4. 背包展示数据：bag_views 派生包含矿石条目（name/icon/category 正确，
#      bag_grid.gd watch 该视图驱动背包 UI 显示）
#   5. 装备链路：mainhud_equipbar 3 号槽 = tool_pickaxe；player.gd 挖矿模式
#      TARGET_DECO=16 与矿石受击盒层匹配
extends SceneTree

const PLAYER_SCRIPT := "res://example/demo/xiuxian/role/player/player.gd"
const EQUIPBAR_SCRIPT := "res://example/demo/xiuxian/scenes/main/ui/mainhud/mainhud_equipbar.gd"
const HARVEST_SCRIPT := "res://example/demo/xiuxian/map/deco_harvest.gd"

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[DecoHarvest] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	# 1. 装配检查
	for path in ["res://example/demo/xiuxian/map/xiaozhai.tscn",
			"res://example/demo/xiuxian/map/xiaozhen.tscn"]:
		var ps: PackedScene = load(path)
		check(ps != null, "地图场景应可加载: %s" % path)
		if ps == null:
			continue
		var packed := ps.instantiate()
		var has_deco := packed.find_child("DecoLayer", true, false) != null \
				or packed.get_node_or_null("DecoLayer") != null
		check(has_deco, "%s 应含 DecoLayer 节点" % path)
		var harvest: Node = packed.get_node_or_null("DecoHarvest")
		check(harvest != null and harvest.get_script() == load(HARVEST_SCRIPT),
			"%s 应挂 deco_harvest.gd 回调宿主" % path)
		packed.free()

	# 2/3/4. 实例化萧宅：铺设 + 敲矿 + 入包 + 视图派生
	var item_bean: GdBean = load("res://example/demo/xiuxian/state/item/item_state.gd").ins()
	var map: Node = (load("res://example/demo/xiuxian/map/xiaozhai.tscn") as PackedScene).instantiate()
	root.add_child(map)
	for i in 3:
		await process_frame  # DecoLayer ready 的 call_deferred spawn_deco

	var layer: Node = map.get_node("DecoLayer")
	check(layer != null and layer.get_class() == "GdDecoLayer", "DecoLayer 应为 GdDecoLayer")
	var ores: Array = []
	for child in layer.get_children():
		if String(child.name).begins_with("Deco_"):
			ores.append(child)
	check(ores.size() > 0, "萧宅应铺出矿石（频率配置生效），实际 %d" % ores.size())

	var ids_ok := true
	var layers_ok := true
	var pos_ok := true
	for ore in ores:
		var item_id := str(ore.get_meta("deco_item_id"))
		if item_id != "ore_coldiron" and item_id != "ore_amethyst":
			ids_ok = false
		var cell: Vector2i = map.world_to_cell(ore.position)
		var terrain := String(map.get_terrain_name_at(cell))
		if terrain != "grass" and terrain != "forest":
			pos_ok = false
		var hurt: Node = ore.get_node_or_null("Hurtbox")
		if hurt == null or int(hurt.get("collision_layer")) != 16:
			layers_ok = false
	check(ids_ok, "矿石物品 id 应为 ore_coldiron/ore_amethyst")
	check(pos_ok, "矿石应只落在可行走的 grass/forest 地形上")
	check(layers_ok, "矿石受击盒应挂 layer5(16) 可采集层")

	# 3. 敲三下入包：首个矿石 take_damage x3（GdHitbox 同款入口）
	if ores.size() > 0:
		var ore: Node = ores[0]
		var item_id := str(ore.get_meta("deco_item_id"))
		var hurt: Node = ore.get_node("Hurtbox")
		hurt.call("take_damage", 1.0)
		hurt.call("take_damage", 1.0)
		check(item_bean.get_count(item_id) == 0, "敲击未满 3 次不应入包")
		hurt.call("take_damage", 1.0)
		check(ore.is_queued_for_deletion(), "采集后矿石应被移除")
		await process_frame
		check(item_bean.get_count(item_id) == 1,
			"敲满 3 次应采集入包 %s（实际 %d）" % [item_id, item_bean.get_count(item_id)])
		check(not is_instance_valid(ore), "帧末矿石应已释放")

		# 4. 背包展示数据：bag_views 派生条目（bag_grid watch 显示链路）
		var views: Array = item_bean.get_value_by_key("bag_views")
		var found := {}
		for v in views:
			if str(v.get("id")) == item_id:
				found = v
		check(not found.is_empty(), "bag_views 应含采集到的矿石条目（背包 UI 数据源）")
		if not found.is_empty():
			check(str(found.get("name")) != item_id and str(found.get("name")) != "",
				"条目应显示定义表名称（实际 %s）" % found.get("name"))
			check(str(found.get("icon")) == "🪨", "矿石图标应为定义表 icon 🪨")
			check(str(found.get("category")) == "material", "材料类矿石应归 material 页签")
			check(str(found.get("count")) == "x1", "数量应显示 x1")

	# 5. 装备链路：3 号槽 = 挖矿镐；玩家挖矿目标层与矿石受击盒匹配
	var equipbar_constants: Dictionary = load(EQUIPBAR_SCRIPT).get_script_constant_map()
	var slots: Array = equipbar_constants.get("SLOT_ITEMS", [])
	check(slots.size() >= 3 and str(slots[2]) == "tool_pickaxe",
		"装备栏 3 号槽应为 tool_pickaxe（实际 %s）" % [slots])
	var player_constants: Dictionary = load(PLAYER_SCRIPT).get_script_constant_map()
	check(int(player_constants.get("TARGET_DECO", 0)) == 16,
		"player.gd TARGET_DECO 应为 16（layer5，与矿石受击盒匹配）")
	check(int(player_constants.get("TARGET_ENEMY", 0)) == 4,
		"player.gd TARGET_ENEMY 应保持 4（layer3 战斗）")

	# 6. UI 级联动：实例化背包面板（默认「全部」页签），矿石条目应直接可见，
	#    再次采集后数量角标刷新（用户报告场景：挖到矿打开背包看不到——
	#    根因曾是初始页签 pill 过滤掉材料类，现默认全部）
	var bag_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/ui/bag/bag_panel.gml.tscn")
	check(bag_scene != null, "bag_panel.gml.tscn 应存在（gml 构建产物）")
	if bag_scene != null:
		var holder := Control.new()
		holder.size = Vector2(1152, 648)
		root.add_child(holder)
		var panel: Control = bag_scene.instantiate()
		holder.add_child(panel)
		for i in 5:
			await process_frame
		var grid: Node = panel.find_child("GoodsGrid", true, false)
		check(grid != null, "背包网格应存在")
		if grid != null:
			# 默认「全部」页签：矿石条目打开即见（无需切材料页签）
			var ore_name := ""
			var ore_count := ""
			for i in range(grid.get_child_count() - 1):
				var cell: Control = grid.get_at(i)
				if cell == null:
					continue
				var data: Dictionary = cell.get_meta("__item_data", {})
				if str(data.get("id")) == "ore_coldiron":
					ore_name = str(data.get("name"))
					var cnt: Label = cell.find_child("CountLabel", true, false)
					ore_count = cnt.text if cnt != null else ""
			check(ore_name == "寒铁矿",
				"默认页签下背包网格应直接可见寒铁矿（实际 %s）" % ore_name)
			check(ore_count == "x1", "矿石数量角标应 x1（实际 %s）" % ore_count)
			# 再次采集：watch bag_views → 网格刷新，数量 x2
			item_bean.add_item("ore_coldiron", 1)
			await process_frame
			for i in range(grid.get_child_count() - 1):
				var cell2: Control = grid.get_at(i)
				if cell2 == null:
					continue
				var data2: Dictionary = cell2.get_meta("__item_data", {})
				if str(data2.get("id")) == "ore_coldiron":
					var cnt2: Label = cell2.find_child("CountLabel", true, false)
					ore_count = cnt2.text if cnt2 != null else ""
			check(ore_count == "x2", "再次采集后数量角标应刷新为 x2（实际 %s）" % ore_count)
			check(item_bean.get_count("ore_coldiron") == 2, "bean 持有量应为 2")
		holder.queue_free()
		await process_frame

	map.free()
	_finish()


func _finish() -> void:
	if ok:
		print("[DecoHarvest] RESULT=PASS")
	else:
		print("[DecoHarvest] RESULT=FAIL")
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
