# map_demo - 快速地图 + 网格移动玩家演示
#
# 场景结构（运行时构建）：
#   - QuickMap  GdQuickMap  30x18 格、cell=32，种子 20260928 生成
#   - Player    GdRoleMover  键盘控制 + MODE_GRID 网格移动（一次一格、禁止斜向）
#   - Camera    跟随地图中心
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
	var cam := Camera2D.new()
	cam.name = "Camera"
	cam.position = Vector2(MAP_W, MAP_H) * CELL / 2.0
	add_child(cam)
	cam.make_current()


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
