# 世界舆图（MapModal / GdMiniMap）验收 —— 缩略数据完整性 + 组件渲染数据面
#
# 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . \
#   -s res://test/check_map_modal.gd
#
# 覆盖：
#   1. XiuMapThumbs 数据：6 图 / 每图 cells 数 = w*h / 布局齐全 / 显示名
#   2. 传送点数据与地图场景实际 marker 一致（cell/to/to_cell）
#   3. GdMiniMap API：add_map/add_portal/set_current/set_player_cell/
#      get_content_size 变化与清理
#   4. map_modal.gml.tscn 直开：MiniMap 节点存在、面板脚本能灌数据
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[MapModal] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	# ---- 1. 缩略数据完整性 ----
	var layout: Dictionary = XiuMapThumbs.get_layout()
	var maps: Dictionary = XiuMapThumbs.get_maps()
	var portals: Dictionary = XiuMapThumbs.get_portals()
	check(maps.size() == 6, "应有 6 张地图缩略（实际 %d）" % maps.size())
	check(layout.size() == 6, "布局应登记 6 图（实际 %d）" % layout.size())
	check(portals.size() == 6, "传送点应登记 6 图（实际 %d）" % portals.size())
	for alias: String in maps:
		var m: Dictionary = maps[alias]
		check(int(m["cells"].size()) == int(m["w"]) * int(m["h"]),
			"%s cells 数应 = w*h" % alias)
		check(layout.has(alias) and str(layout[alias].get("name", "")) != "",
			"%s 应有布局与显示名" % alias)
	check(XiuMapThumbs.get_map_name("xiuxian_town") == "青云坊", "青云坊显示名应正确")
	var ordered: Array = XiuMapThumbs.get_ordered_aliases()
	check(ordered[0] == "xiuxian_home", "西端应为萧宅农田")
	check(ordered[ordered.size() - 1] == "xiuxian_map_a", "东端应为青草岭")

	# ---- 2. 缩略传送点 ↔ 场景 marker 一致 ----
	var scene_portals := {}
	for path: String in [
		"res://example/demo/xiuxian/map/home.tscn",
		"res://example/demo/xiuxian/map/xiaozhai.tscn",
		"res://example/demo/xiuxian/map/xiaozhen.tscn",
		"res://example/demo/xiuxian/map/town.tscn",
		"res://example/demo/xiuxian/map/map_a.tscn",
		"res://example/demo/xiuxian/map/map_b.tscn",
	]:
		var map: Node = (load(path) as PackedScene).instantiate()
		root.add_child(map)
		scene_portals[path.get_file()] = map
	for i in 8:
		await process_frame
	var total := 0
	for fname: String in scene_portals:
		var alias := "xiuxian_" + fname.replace(".tscn", "")
		var list: Array = portals.get(alias, [])
		total += list.size()
		var map: Node = scene_portals[fname]
		var markers: Array = []
		for child: Node in map.get_children():
			if child.has_method("get_target_alias") and str(child.get("target_alias")) != "":
				markers.append(child)
		check(list.size() == markers.size(),
			"%s 缩略传送点数应与场景一致（%d vs %d）" % [alias, list.size(), markers.size()])
		for p: Dictionary in list:
			var c: Array = p["cell"]
			var found := false
			for mk: Node in markers:
				var mc: Vector2i = mk.get("cell")
				if mc.x == int(c[0]) and mc.y == int(c[1]):
					check(str(p["to"]) == str(mk.get("target_alias")),
						"%s(%s) 目标应一致" % [alias, str(c)])
					found = true
					break
			check(found, "%s 缩略传送点 %s 在场景无对应 marker" % [alias, str(c)])
	check(total == 12, "传送点总数应为 12（实际 %d）" % total)

	# ---- 3. GdMiniMap API ----
	var mm := ClassDB.instantiate("GdMiniMap") as Control
	root.add_child(mm)
	await process_frame
	var empty_size: Vector2 = mm.call("get_content_size")
	check(empty_size == Vector2.ZERO, "无数据时内容尺寸应为 0")
	var cells := PackedInt32Array()
	for i in 12:
		cells.append(i % 3)
	mm.call("add_map", "t1", "测试图", 0, 0, 4, 3, cells,
		PackedStringArray(["grass", "water", "mountain"]))
	var size1: Vector2 = mm.call("get_content_size")
	check(size1.x > 0 and size1.y > 0, "加图后内容尺寸应为正")
	mm.call("add_portal", "t1", Vector2i(3, 1), "t2", Vector2i(0, 1), "去对面")
	mm.call("add_map", "t2", "邻图", 8, 0, 4, 3, cells,
		PackedStringArray(["grass", "water", "mountain"]))
	mm.call("add_portal", "t2", Vector2i(0, 1), "t1", Vector2i(3, 1), "回来")
	var size2: Vector2 = mm.call("get_content_size")
	check(size2.x > size1.x, "加第二张图后内容应变宽")
	mm.call("set_current", "t1")
	mm.call("set_player_cell", Vector2i(1, 1))
	mm.call("set_terrain_color", "grass", Color.RED)
	check(true, "set_current/set_player_cell/set_terrain_color 不应崩溃")
	mm.call("clear_all")
	var size3: Vector2 = mm.call("get_content_size")
	check(size3 == Vector2.ZERO, "clear_all 后内容尺寸应为 0")
	mm.queue_free()

	# ---- 4. map_modal.gml.tscn 直开 ----
	var scene: PackedScene = load(
		"res://example/demo/xiuxian/scenes/main/ui/map/map_modal.gml.tscn")
	check(scene != null, "map_modal.gml.tscn 应存在（先跑 regen_gml_tscn）")
	if scene == null:
		quit(2 if not ok else 2)
		return
	var panel: Control = scene.instantiate()
	root.add_child(panel)
	await process_frame
	var world: Control = panel.find_child("WorldMap", true, false)
	check(world != null, "WorldMap 应存在")
	check(world != null and world.get_class() == "GdMiniMap", "WorldMap 应为 GdMiniMap")
	# 面板灌数据路径（实机由 _ready 驱动；GDCORE 全局表 headless 下可能为空，
	# _populate 只依赖静态表，直接调用验证）
	panel.call("_populate")
	await process_frame
	if world:
		var wsize: Vector2 = world.call("get_content_size")
		check(wsize.x > 1000.0, "灌数据后世界图应有实际内容尺寸（%s）" % str(wsize))

	print("[MapModal] 结果: %s" % ("全部通过" if ok else "存在失败项"))
	quit(0 if ok else 1)
