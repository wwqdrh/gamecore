# 传送网络回归：六张地图传送点必须
#  1) 双向成对（A→B 则 B 存在指回 A 的传送点）
#  2) 落点 cell 可行走、且紧邻目标图返程传送点（曼哈顿距离 ≤ 2，且不在传送点本体上）
#  3) 传送点本体可行走（snap 前提下不该依赖吸附）
#  4) 传送点位于地图边线（面向对图的连接处，消除割裂感）
#  5) 成对传送点在拼接方向上世界行/列对齐（陆桥对陆桥，偏差 ≤ 2 格）
# 用法：godot --headless --path . -s res://test/check_map_portal.gd
extends SceneTree

const MAPS := [
	"res://example/demo/xiuxian/map/home.tscn",
	"res://example/demo/xiuxian/map/xiaozhai.tscn",
	"res://example/demo/xiuxian/map/xiaozhen.tscn",
	"res://example/demo/xiuxian/map/town.tscn",
	"res://example/demo/xiuxian/map/map_a.tscn",
	"res://example/demo/xiuxian/map/map_b.tscn",
]

## 与 regen_map_thumbs.gd 的 LAYOUT 一致（世界拼接偏移，格子单位）
const LAYOUT := {
	"xiuxian_home": {"x": 0, "y": 8},
	"xiuxian_xiaozhai": {"x": 30, "y": 8},
	"xiuxian_xiaozhen": {"x": 60, "y": 7},
	"xiuxian_town": {"x": 98, "y": 6},
	"xiuxian_map_a": {"x": 140, "y": 6},
	"xiuxian_map_b": {"x": 140, "y": -16},
}

var _ok := 0
var _fail := 0
var _loaded := {}  # fname -> map node
var _portals := {}  # fname -> Array[Dictionary] (cell/target_alias/target_cell/name)


func _init() -> void:
	process_frame.connect(_on_frame)
	_frames = 0


var _frames := 0


func _on_frame() -> void:
	_frames += 1
	if _frames == 1:
		for path: String in MAPS:
			var map: Node = (load(path) as PackedScene).instantiate()
			root.add_child(map)
			_loaded[path.get_file()] = map
	elif _frames == 8:
		_collect()
		_verify()
		print("[MapPortal] 结果: %d 通过 / %d 失败" % [_ok, _fail])
		quit(1 if _fail > 0 else 0)


func _collect() -> void:
	for fname: String in _loaded:
		var map: Node = _loaded[fname]
		var list: Array = []
		for child: Node in map.get_children():
			if child.has_method("get_target_alias") and str(child.get("target_alias")) != "":
				list.append({
					"cell": child.get("cell") as Vector2i,
					"to": str(child.get("target_alias")),
					"to_cell": child.get("target_cell") as Vector2i,
					"name": child.name,
				})
		_portals[fname] = list


static func _alias_to_file(alias: String) -> String:
	return alias.replace("xiuxian_", "") + ".tscn"


func _check(cond: bool, msg: String) -> void:
	if cond:
		_ok += 1
	else:
		_fail += 1
		print("[MapPortal] FAIL: ", msg)


func _verify() -> void:
	for fname: String in _portals:
		var map: Node = _loaded[fname]
		for p: Dictionary in _portals[fname]:
			var cell: Vector2i = p["cell"]
			var to: String = p["to"]
			var to_file := _alias_to_file(to)
			_check(_loaded.has(to_file), "%s.%s 目标 %s 未在测试地图集" % [fname, p["name"], to])
			if not _loaded.has(to_file):
				continue
			# 1) 本体可行走
			_check(map.is_walkable(cell), "%s.%s 本体 (%d,%d) 不可行走" % [fname, p["name"], cell.x, cell.y])
			# 1b) 位于地图边线（面向对图的连接处）
			var w: int = map.get_map_width()
			var h: int = map.get_map_height()
			var on_edge: bool = cell.x == 0 or cell.y == 0 or cell.x == w - 1 or cell.y == h - 1
			_check(on_edge, "%s.%s (%d,%d) 不在地图边线（应为 x=0/%d 或 y=0/%d）" % [
				fname, p["name"], cell.x, cell.y, w - 1, h - 1])
			# 2) 双向：目标图存在指回本图的传送点
			var back: Array = _portals[to_file].filter(
				func(q: Dictionary) -> bool: return q["to"] == _alias_of(fname))
			_check(not back.is_empty(), "%s.%s → %s 缺返程传送点" % [fname, p["name"], to])
			if back.is_empty():
				continue
			# 3) 落点可行走
			var to_cell: Vector2i = p["to_cell"]
			var tmap: Node = _loaded[to_file]
			_check(tmap.is_walkable(to_cell),
				"%s.%s 落点 %s(%d,%d) 不可行走" % [fname, p["name"], to, to_cell.x, to_cell.y])
			# 4) 落点紧邻某返程传送点且不在其本体上（防落地即回传）
			var near := false
			for q: Dictionary in back:
				var b: Vector2i = q["cell"]
				var md: int = abs(to_cell.x - b.x) + abs(to_cell.y - b.y)
				if md >= 1 and md <= 2:
					near = true
					break
			_check(near, "%s.%s 落点 (%d,%d) 不在返程传送点旁 %s" % [
				fname, p["name"], to_cell.x, to_cell.y, str(back)])
			# 5) 成对传送点拼接向对齐（陆桥对陆桥）：
			#    同轴面对面（我东/西缘 ↔ 对方西/东缘，或南北缘相对）→
			#    世界行/列偏差 ≤ 2；对角连接（如 town 东北角 ↔ map_b 西南角
			#    的捷径环）天生不共线，只要求双边线（1b 已查），不查对齐。
			var la: Dictionary = LAYOUT[_alias_of(fname)]
			var lb: Dictionary = LAYOUT[to]
			var wa: Vector2i = Vector2i(la["x"] as int + cell.x, la["y"] as int + cell.y)
			var q0: Dictionary = back[0]
			var bc: Vector2i = q0["cell"]
			var wb: Vector2i = Vector2i(lb["x"] as int + bc.x, lb["y"] as int + bc.y)
			var me_ew: bool = cell.x == 0 or cell.x == w - 1
			var me_ns: bool = cell.y == 0 or cell.y == h - 1
			var bw: int = tmap.get_map_width()
			var bh: int = tmap.get_map_height()
			var back_ew: bool = bc.x == 0 or bc.x == bw - 1
			var back_ns: bool = bc.y == 0 or bc.y == bh - 1
			if me_ew and back_ew:
				_check(abs(wa.y - wb.y) <= 2, "%s.%s 世界(%d,%d) 与返程 %s 世界(%d,%d) 世界行偏差 > 2（陆桥未对齐）" % [
					fname, p["name"], wa.x, wa.y, str(q0["name"]), wb.x, wb.y])
			elif me_ns and back_ns:
				_check(abs(wa.x - wb.x) <= 2, "%s.%s 世界(%d,%d) 与返程 %s 世界(%d,%d) 世界列偏差 > 2（陆桥未对齐）" % [
					fname, p["name"], wa.x, wa.y, str(q0["name"]), wb.x, wb.y])
			# 6) 传送点与地图腹地连通：从传送点 BFS 找「距它最远的可行走格」
			#    并 find_path 验证可达（边线陆桥必须能走回图内，不许孤悬）
			if map.has_method("find_path"):
				var far := _farthest_walkable(map, w, h, cell)
				if far != cell:
					var path: PackedVector2Array = map.find_path(cell, far)
					_check(not path.is_empty(), "%s.%s (%d,%d) 无法 walk 到腹地 %s（陆桥与主陆区断开）" % [
						fname, p["name"], cell.x, cell.y, str(far)])


static func _alias_of(fname: String) -> String:
	return "xiuxian_" + fname.replace(".tscn", "")


## 全图扫描：取 Chebyshev 距离起点最远的可行走格（腹地锚点）
static func _farthest_walkable(map: Node, w: int, h: int, from: Vector2i) -> Vector2i:
	var best := from
	var best_d := -1
	for y: int in h:
		for x: int in w:
			var c := Vector2i(x, y)
			if not map.is_walkable(c):
				continue
			var d: int = maxi(abs(x - from.x), abs(y - from.y))
			if d > best_d:
				best_d = d
				best = c
	return best
