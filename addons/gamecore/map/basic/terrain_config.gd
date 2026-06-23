## 地形配置资源
## 每种地形对应一个 TerrainConfig 实例，可在编辑器 Inspector 中配置
class_name TerrainConfig
extends Resource

## 地形名称（需唯一，用于注册和查找，需与 GdMapBasic 下子 TileMapLayer 节点名一致）
@export var terrain_name: String = ""

## 显示层 source_id（过渡贴图所在的 TileSet source）
@export var display_source_id: int = 0

## 噪声阈值上限（噪声值 < 此值时为该地形，按 terrains 数组顺序匹配）
@export var threshold_max: float = 1.0
