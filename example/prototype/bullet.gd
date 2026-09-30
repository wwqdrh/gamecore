# bullet - 池化子弹示例
# 由 GdSpawnPool 管理生命周期：spawn 出场，超时自动 despawn 归还池中（不 free）
extends Node2D

## 飞行速度（像素/秒）
var velocity := Vector2(320, 0)
## 存活时长（秒），归零自动回池
var lifetime := 1.5


func _ready() -> void:
	# 随机暖色，便于肉眼确认"复用"还是"新实例"
	$Poly.modulate = Color(randf_range(0.8, 1.0), randf_range(0.4, 0.9), 0.2)


func _process(delta: float) -> void:
	position += velocity * delta
	lifetime -= delta
	if lifetime <= 0.0:
		Engine.get_singleton("GDSPAWNPOOL").despawn(self)
