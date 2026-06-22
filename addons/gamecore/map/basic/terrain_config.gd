## 地形配置资源
## 每种地形对应一个 TerrainConfig 实例，可在编辑器 Inspector 中配置
class_name TerrainConfig
extends Resource

## 地形名称（需唯一，用于注册和查找）
@export var terrain_name: String = ""

## 世界层图集坐标
@export var atlas_coord: Vector2i = Vector2i.ZERO

## 世界层 source_id
@export var source_id: int = 0

## 显示层 source_id（过渡贴图）
@export var display_source_id: int = 0

## 噪声阈值上限（噪声值 < 此值时为该地形，按 terrains 数组顺序匹配）
@export var threshold_max: float = 1.0

@export var priority: int = 1

static func build(data: Dictionary) -> TerrainConfig:
	var res = TerrainConfig.new()
	
	return res
