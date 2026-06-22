## 资源放置配置
## 每种可放置资源对应一个 PropConfig 实例
class_name MapPropConfig
extends Resource

## 资源名称
@export var prop_name: String = ""

## TileSet source_id
@export var source_id: int = 0

## alternative_tile 值
@export var alternative_tile: int = 0

## 放置概率 (0.0 ~ 1.0)
@export var probability: float = 0.05

## 可放置的地形名称列表
@export var allowed_terrains: PackedStringArray = []

## 噪声值范围 (x = min, y = max)
@export var noise_range: Vector2 = Vector2(-0.3, 0.0)
