# map_demo - 快速地图 + 网格移动玩家演示
#
# 场景结构（运行时构建）：
#   Index     GdSceneRoot  场景管理器根节点
#   └ Demo    Node2D       本脚本，演示内容
#     - QuickMap  GdQuickMap  30x18 格、cell=32，种子 20260928 生成
#     - Player    GdRoleMover  键盘控制 + MODE_GRID 网格移动（一次一格、禁止斜向）
#     - Camera    GdViewCamera 平滑放大视野、跟随玩家、限制在地图边界内
#
# 网格移动约定：格 (cx, cy) 中心在 ((cx+0.5)*32, (cy+0.5)*32)，
# 水体/山地不可通行（由 QuickMap.is_walkable 判定，Mover 每步自动查询）。
# WASD / 方向键移动（按住可连续走格）；
# 鼠标左键点击地图 → BFS 计算路径 → 角色沿路径逐格走过去（键盘输入随时打断）。
extends Node2D

# 移动模式常量（与 GdRoleMover 一致）
const MODE_GRID := 2

const MAP_W := 30
const MAP_H := 18
const CELL := 32
const MAP_SEED := 20260928

var map: GdQuickMap
var player: GdRoleMover
var path_line: Line2D
var path_label: Label


func _ready() -> void:
	_make_map()
	_make_player()
	_make_camera()
	_make_path_preview()


func _make_map() -> void:
	map = GdQuickMap.new()
	map.name = "QuickMap"
	map.width = MAP_W
	map.height = MAP_H
	map.cell_size = CELL
	map.seed_value = MAP_SEED
	map.draw_grid_lines = true
	# 每种地形一个 TileMapLayer，图层栈顺序 = 地形下标顺序（0 在最底、越靠后越靠上）。
	# 水放最上层：岸线过渡片的圆角缺口压在沙滩/草地上，露出下层而不是灰底。
	# 阈值保持各地形占比：sand[0,0.2) grass[0.2,0.4) forest[0.4,0.6)
	# mountain[0.6,0.8) water[0.8,1.0)（water 是最后一个地形，兜底接收最高噪声段）
	map.terrain_names = PackedStringArray(["sand", "grass", "forest", "mountain", "water"])
	map.terrain_thresholds = PackedFloat64Array([0.2, 0.4, 0.6, 0.8])
	map.terrain_dualgrid_textures = PackedStringArray([
		"",
		"res://example/map/assets/tileset_grass.png",
		"",
		"",
		"res://example/map/assets/tileset_water.png",
	])
	map.terrain_shaders = PackedStringArray([
		"", "", "", "",
		"water_flow",  # 内置水体流动 shader（源码在 Rust 侧动态创建）
	])
	add_child(map)
	map.generate(MAP_SEED)
	map.set_blocked_terrain_names(PackedStringArray(["water", "mountain"]))


func _make_player() -> void:
	player = GdRoleMover.new()
	player.name = "Player"
	player.control_mode = 1      # 键盘
	player.move_mode = MODE_GRID # 网格移动
	player.speed = CELL * 5.0    # 5 格/秒
	player.grid_cell_size = CELL
	player.grid_map_path = NodePath("../QuickMap")
	player.s_grid_path_finished.connect(_on_path_finished)

	# 出生点：从地图中心螺旋向外找第一个可通行格
	var spawn := _find_walkable_near(Vector2i(MAP_W / 2, MAP_H / 2))
	player.position = map.cell_to_center(spawn)

	var rect := ColorRect.new()
	rect.color = Color(0.3, 0.6, 1.0)
	rect.size = Vector2(22, 22)
	rect.position = -rect.size / 2.0
	rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	player.add_child(rect)

	add_child(player)
	_make_label("WASD/方向键移动 | 鼠标左键点击寻路", Vector2(16, 16))


func _make_path_preview() -> void:
	# 路径预览折线（黄色），角色出发后保留、走完清除
	path_line = Line2D.new()
	path_line.name = "PathPreview"
	path_line.width = 3.0
	path_line.default_color = Color(1.0, 0.85, 0.25, 0.9)
	add_child(path_line)


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed \
			and event.button_index == MOUSE_BUTTON_LEFT:
		_on_map_clicked()


func _on_map_clicked() -> void:
	var target := map.world_to_cell(map.get_global_mouse_position())
	if not map.is_walkable(target):
		return
	var from := map.world_to_cell(player.position)
	if from == target:
		return
	var path: PackedVector2Array = map.find_path(from, target)
	if path.is_empty():
		# 不可达（被水域/山地隔开）：短暂提示
		if path_label != null:
			path_label.text = "目标不可达 (%d,%d)" % [target.x, target.y]
		return
	player.set_grid_path(path)
	# 预览线：当前位置 → 依次各格中心
	var pts := PackedVector2Array([player.position])
	pts.append_array(path)
	path_line.points = pts
	if path_label != null:
		path_label.text = "寻路 %d 格 → (%d,%d)" % [path.size(), target.x, target.y]


func _on_path_finished() -> void:
	path_line.clear_points()
	if path_label != null:
		path_label.text = ""


## 从指定格开始向外找可通行出生格
func _find_walkable_near(center: Vector2i) -> Vector2i:
	for r in range(0, 20):
		for dy in range(-r, r + 1):
			for dx in range(-r, r + 1):
				if max(abs(dx), abs(dy)) != r:
					continue
				var cell := Vector2i(center.x + dx, center.y + dy)
				if map.is_walkable(cell):
					return cell
	return center


func _make_camera() -> void:
	# 相机管理器：跟随玩家、限制在地图边界内、平滑放大视野
	var cam := GdViewCamera.new()
	cam.name = "Camera"
	cam.zoom_min = 1.5
	cam.zoom_max = 2.5
	add_child(cam)
	cam.make_current()
	# 放大视野：从当前缩放平滑过渡到 zoom_max（zoom_steps=2，步 2 即最大档）
	cam.start_zoom(2, -1.0, -1.0)
	# 立即贴合玩家并开启平滑跟随
	cam.follow(player, true, true)
	# 视野边界 = 地图矩形（left, right, top, bottom），不超出地图
	cam.update_limit(Vector4(0.0, MAP_W * CELL, 0.0, MAP_H * CELL))


func _make_label(text: String, pos: Vector2) -> void:
	var label := Label.new()
	label.text = text
	label.position = pos
	label.add_theme_font_size_override("font_size", 14)
	label.add_theme_color_override("font_color", Color(0.8, 0.85, 0.95))
	add_child(label)
	# 第二行：寻路状态提示
	path_label = Label.new()
	path_label.text = ""
	path_label.position = pos + Vector2(0, 22)
	path_label.add_theme_font_size_override("font_size", 13)
	path_label.add_theme_color_override("font_color", Color(1.0, 0.85, 0.3))
	add_child(path_label)
