# suite: map - GdMapBasic 双网格地图节点测试
# 覆盖: 地形注册/查询、地形设置/擦除、多地形叠加、从字符串加载资源配置
extends "res://test/test_case.gd"


func _make_map() -> GdMapBasic:
	var map := GdMapBasic.new()
	return map


func test_register_terrain() -> void:
	var map := _make_map()
	var grass_id: int = map.register_terrain("grass")
	var mud_id: int = map.register_terrain("mud")

	assert_true(grass_id > 0, "注册地形 ID 应为正数")
	assert_true(mud_id > 0, "注册地形 ID 应为正数")
	assert_ne(grass_id, mud_id, "不同地形 ID 应不同")

	# 重复注册返回相同 ID
	assert_eq(map.register_terrain("grass"), grass_id, "重复注册应返回相同 ID")

	assert_eq(map.get_terrain_id("grass"), grass_id, "get_terrain_id 应能查回 grass")
	assert_eq(map.get_terrain_name(grass_id), "grass", "get_terrain_name 应能查回名称")
	assert_eq(map.get_terrain_name(mud_id), "mud", "get_terrain_name 应能查回 mud")

	var names := Array(map.get_all_terrain_names())
	assert_true(names.has("grass") and names.has("mud"), "地形名列表应包含已注册地形")
	map.free()


func test_set_and_query_terrain() -> void:
	var map := _make_map()
	var grass_id: int = map.register_terrain("grass")
	var mud_id: int = map.register_terrain("mud")
	var c := Vector2i(0, 0)

	map.set_terrain(c, grass_id)
	assert_true(map.has_terrain(c, grass_id), "设置后该坐标应有 grass")
	assert_false(map.has_terrain(c, mud_id), "未设置的地形查询应为 false")

	# 同一坐标叠加多地形
	map.set_terrain(c, mud_id)
	var terrains := Array(map.get_terrains_at(c))
	assert_eq(terrains.size(), 2, "同一坐标应支持叠加 2 种地形")
	assert_true(terrains.has(grass_id) and terrains.has(mud_id), "叠加后应包含两种地形 ID")

	# 已用地形格子应包含该坐标
	var used := Array(map.get_used_terrain_cells())
	assert_true(used.has(c), "used_terrain_cells 应包含已设置坐标")
	map.free()


func test_erase_tile() -> void:
	var map := _make_map()
	var grass_id: int = map.register_terrain("grass")
	var c := Vector2i(3, 5)

	map.set_terrain(c, grass_id)
	assert_true(map.has_terrain(c, grass_id), "写入后应有 grass")

	map.erase_tile(c, grass_id)
	assert_false(map.has_terrain(c, grass_id), "擦除后不应有 grass")
	assert_eq(Array(map.get_terrains_at(c)).size(), 0, "擦除后该坐标应无地形")
	map.free()


func test_load_resource_config_from_string() -> void:
	var map := _make_map()
	var config := """
	{
		"terrains": {
			"water": {},
			"sand": {}
		},
		"display_layers": {
			"water": { "source_id": 0 }
		}
	}
	"""
	var ok: bool = map.load_resource_config_from_string(config)
	assert_true(ok, "合法的资源配置 JSON 应加载成功")
	assert_true(map.get_terrain_id("water") > 0, "加载配置后应注册 water 地形")
	assert_true(map.get_terrain_id("sand") > 0, "加载配置后应注册 sand 地形")
	map.free()
