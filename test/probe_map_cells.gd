# 探针：批量检查六张地图候选传送/落点格的地形与可行走性
# 用法：godot --headless --path . -s res://test/probe_map_cells.gd
# 注意：GdQuickMap 地形生成在 ready 后异步完成，需等帧后查询
extends SceneTree

const MAPS := [
	"res://example/demo/xiuxian/map/home.tscn",
	"res://example/demo/xiuxian/map/xiaozhai.tscn",
	"res://example/demo/xiuxian/map/xiaozhen.tscn",
	"res://example/demo/xiuxian/map/town.tscn",
	"res://example/demo/xiuxian/map/map_a.tscn",
	"res://example/demo/xiuxian/map/map_b.tscn",
]

# 每张地图：候选格列表（现有传送点 + 拟调整的落点/新传送点）
const CANDIDATES := {
	"home.tscn": [
		Vector2i(2, 12), Vector2i(3, 12), Vector2i(10, 8), Vector2i(1, 12), Vector2i(4, 12),
	],
	"xiaozhai.tscn": [
		Vector2i(9, 11), Vector2i(9, 10), Vector2i(14, 12), Vector2i(13, 12), Vector2i(8, 11),
	],
	"xiaozhen.tscn": [
		Vector2i(1, 9), Vector2i(2, 9), Vector2i(25, 9), Vector2i(26, 9), Vector2i(13, 9),
	],
	"town.tscn": [
		Vector2i(2, 10), Vector2i(3, 10), Vector2i(27, 10), Vector2i(28, 10),
		Vector2i(16, 2), Vector2i(16, 3), Vector2i(16, 4),
	],
	"map_a.tscn": [
		Vector2i(2, 13), Vector2i(3, 13), Vector2i(4, 13),
		Vector2i(24, 5), Vector2i(25, 5), Vector2i(26, 5), Vector2i(27, 5), Vector2i(28, 5), Vector2i(29, 5),
		Vector2i(24, 6), Vector2i(25, 6), Vector2i(26, 6), Vector2i(27, 6), Vector2i(28, 6), Vector2i(29, 6),
		Vector2i(24, 7), Vector2i(25, 7), Vector2i(26, 7), Vector2i(27, 7), Vector2i(28, 7), Vector2i(29, 7),
		Vector2i(24, 8), Vector2i(25, 8), Vector2i(26, 8), Vector2i(27, 8), Vector2i(28, 8), Vector2i(29, 8),
		Vector2i(24, 9), Vector2i(27, 9), Vector2i(28, 9), Vector2i(29, 9),
		Vector2i(24, 10), Vector2i(25, 10), Vector2i(26, 10), Vector2i(27, 10), Vector2i(28, 10), Vector2i(29, 10),
		Vector2i(24, 11), Vector2i(25, 11), Vector2i(26, 11), Vector2i(27, 11), Vector2i(28, 11), Vector2i(29, 11),
		Vector2i(24, 12), Vector2i(25, 12), Vector2i(26, 12), Vector2i(27, 12), Vector2i(28, 12), Vector2i(29, 12),
	],
	"map_b.tscn": [
		Vector2i(3, 8), Vector2i(4, 8), Vector2i(3, 9), Vector2i(13, 13),
		Vector2i(13, 14), Vector2i(13, 12), Vector2i(22, 8),
	],
}

var _loaded: Array[Node] = []


func _init() -> void:
	process_frame.connect(_on_frame)


var _frames := 0


func _on_frame() -> void:
	_frames += 1
	if _frames == 1:
		print("[MapCells] begin")
		for path: String in MAPS:
			var ps: PackedScene = load(path)
			if ps == null:
				print("[MapCells] LOAD FAIL ", path)
				continue
			var map: Node = ps.instantiate()
			root.add_child(map)
			_loaded.append(map)
	elif _frames == 8:
		for map: Node in _loaded:
			var w: int = map.get_map_width()
			var h: int = map.get_map_height()
			var fname: String = map.get_scene_file_path().get_file()
			print("[MapCells] %s  %dx%d" % [fname, w, h])
			for cell: Vector2i in CANDIDATES.get(fname, []):
				var walk: bool = map.is_walkable(cell)
				var terr: String = map.get_terrain_name_at(cell)
				print("  (%d,%d) walk=%s terr=%s" % [cell.x, cell.y, walk, terr])
		print("[MapCells] end")
		quit(0)
