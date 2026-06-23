class_name GdMapIsometricDefault
extends GdMapIsometric

## 地图大小
@export var map_width: int = 64
@export var map_height: int = 32
## 随机种子（0 为随机）
@export var map_seed: int = 42
## 外部 TileSet 资源路径（tile_set 为空时使用此路径加载）
@export var external_tile_set_path: String = ""

## 地形配置列表（在 Inspector 中可新增/编辑地形）
@export var terrains: Array[TerrainConfig] = []

## 资源放置配置列表
@export var props: Array[MapPropConfig] = []

func _ready() -> void:
	# 注册地形并配置资源
	_setup_terrains()
	_setup_props()
	_setup_thresholds()

	self_modulate = Color(1, 1, 1, 0)
	generate_map_from_tiles()
	## 生成带资源的随机地图
	#_generate_map()


func generate_map_with_seed(new_seed: int) -> void:
	map_seed = new_seed
	_generate_map()


func _generate_map() -> void:
	generate_map_with_resources(map_width, map_height, map_seed)
	print("等距地图生成完成: %dx%d, seed=%d" % [map_width, map_height, map_seed])


## 解析 TileSet：优先直接引用，其次外部路径
func _resolve_tile_set() -> TileSet:
	if tile_set != null:
		return tile_set
	if external_tile_set_path != "":
		var loaded = load(external_tile_set_path)
		if loaded is TileSet:
			return loaded
		push_warning("GdMapIsometricDefault: 无法从路径加载 TileSet: %s" % external_tile_set_path)
	return null

## 遍历 terrains 数组，注册地形并添加配置
func _setup_terrains() -> void:
	for terrain: TerrainConfig in terrains:
		if terrain.terrain_name == "":
			continue
		register_terrain(terrain.terrain_name)
		add_terrain_config(
			terrain.terrain_name,
			terrain.atlas_coord.x,
			terrain.atlas_coord.y,
			terrain.source_id,
			terrain.display_source_id,
			terrain.priority
		)


## 遍历 props 数组，添加资源配置
func _setup_props() -> void:
	for prop: MapPropConfig in props:
		if prop.prop_name == "":
			continue
		add_prop_config(
			prop.prop_name,
			prop.source_id,
			prop.alternative_tile,
			prop.probability,
			prop.allowed_terrains,
			prop.noise_range.x,
			prop.noise_range.y,
		)


## 根据 terrains 数组的 threshold_max 设置阈值
func _setup_thresholds() -> void:
	var terrain_names := PackedStringArray()
	var threshold_maxs := PackedFloat64Array()
	for terrain: TerrainConfig in terrains:
		if terrain.terrain_name == "":
			continue
		terrain_names.append(terrain.terrain_name)
		threshold_maxs.append(terrain.threshold_max)
	set_thresholds(terrain_names, threshold_maxs)
