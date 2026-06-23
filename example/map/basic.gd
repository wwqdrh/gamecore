# GdMapBasic 双网格地图示例
# 演示如何使用 GdMapBasic 生成随机地图并配置资源

extends Node2D


## UI 节点引用
@onready var btn_regenerate: Button = %BtnRegenerate
@onready var btn_clear: Button = %BtnClear
@onready var map: GdMapBasicDefault = $Map

func _ready() -> void:
	# 连接按钮信号
	btn_regenerate.pressed.connect(_on_regenerate_pressed)
	btn_clear.pressed.connect(_on_clear_pressed)

func _on_regenerate_pressed() -> void:
	map.generate_map_with_seed(randi())


func _on_clear_pressed() -> void:
	map.clear_map()
	print("地图已清除")
