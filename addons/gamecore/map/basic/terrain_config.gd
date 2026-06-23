## 地形配置资源
## 每种地形对应一个 TerrainConfig 实例，可在编辑器 Inspector 中配置
class_name TerrainConfig
extends Resource

## 地形名称（需唯一，用于注册和查找，需与 GdMapBasic 下子 TileMapLayer 节点名一致）
@export var terrain_name: String = ""

## 显示层 source_id（过渡贴图所在的 TileSet source）
@export var display_source_id: int = 0

## 噪声阈值下限（噪声值 >= 此值时该地形存在）
@export var threshold_min: float = 0.0

## 噪声阈值上限（噪声值 < 此值时该地形存在）
## 噪声值在 [threshold_min, threshold_max) 范围内时该坐标属于此地形
## 一个坐标可属于多个地形（不同地形的范围可重叠）
@export var threshold_max: float = 1.0
