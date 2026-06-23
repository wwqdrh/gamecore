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

## 允许放置的地形名称列表
@export var allowed_terrains: Array[String] = []

## 噪声值范围 (x = min, y = max)，噪声已归一化到 [0, 1]
## 资源只放置在噪声值属于 [min, max) 的坐标上
@export var noise_range: Vector2 = Vector2(0.0, 0.1)
