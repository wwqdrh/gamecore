# 生成世界地图缩略数据：六张地图的地形网格 + 传送点 + 拼接布局
# 产物：example/demo/xiuxian/state/map/map_thumbs.json（明文派生数据，GdMiniMap 渲染源）
# 地图改动（尺寸/种子/地形/传送点）后重跑：godot --headless --path . -s res://test/regen_map_thumbs.gd
extends SceneTree

const OUT_PATH := "res://example/demo/xiuxian/state/map/map_thumbs.json"

const MAPS := [
	"res://example/demo/xiuxian/map/home.tscn",
	"res://example/demo/xiuxian/map/xiaozhai.tscn",
	"res://example/demo/xiuxian/map/xiaozhen.tscn",
	"res://example/demo/xiuxian/map/town.tscn",
	"res://example/demo/xiuxian/map/map_a.tscn",
	"res://example/demo/xiuxian/map/map_b.tscn",
	"res://example/demo/xiuxian/map/sect.tscn",
]

## 世界拼接布局（格子单位，含 6~8 格间隙）——与传送拓扑对应的地理排布：
##   home — xiaozhai — xiaozhen — town — map_a 自西向东；
##   map_b 在 map_a 正北（主线 town→map_a→map_b 纵向直连），
##   town 东北角与 map_b 西南角对角互通（捷径环）。
## 成对地图的传送点位于相互面对的边线、世界行/列对齐（gate 陆桥对陆桥）
const LAYOUT := {
	"xiuxian_home": {"x": 0, "y": 8, "name": "萧宅农田"},
	"xiuxian_xiaozhai": {"x": 30, "y": 8, "name": "萧宅"},
	"xiuxian_xiaozhen": {"x": 60, "y": 7, "name": "青石镇"},
	"xiuxian_town": {"x": 98, "y": 6, "name": "青云坊"},
	"xiuxian_map_a": {"x": 140, "y": 6, "name": "青草岭"},
	"xiuxian_map_b": {"x": 140, "y": -16, "name": "幽竹林"},
	"xiuxian_sect": {"x": 98, "y": -22, "name": "青云宗山门"},
}

var _maps: Array[Node] = []


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
			_maps.append(map)
	elif _frames == 8:
		_dump()
		quit(0)


func _dump() -> void:
	var maps := {}
	var portals := {}
	for map: Node in _maps:
		var alias := _alias_of(map.get_scene_file_path().get_file())
		var w: int = map.get_map_width()
		var h: int = map.get_map_height()
		# 地形名表（按行扫描顺序首次出现登记）+ 每格索引
		var terrains: Array = []
		var cells: Array = []
		for y: int in h:
			for x: int in w:
				var tname := str(map.get_terrain_name_at(Vector2i(x, y)))
				var idx := terrains.find(tname)
				if idx < 0:
					terrains.append(tname)
					idx = terrains.size() - 1
				cells.append(idx)
		maps[alias] = {"w": w, "h": h, "terrains": terrains, "cells": cells}
		# 传送点
		var plist: Array = []
		for child: Node in map.get_children():
			if child.has_method("get_target_alias") and str(child.get("target_alias")) != "":
				var c: Vector2i = child.get("cell")
				var tc: Vector2i = child.get("target_cell")
				plist.append({
					"cell": [c.x, c.y],
					"to": str(child.get("target_alias")),
					"to_cell": [tc.x, tc.y],
					"label": str(child.get("label")),
				})
		portals[alias] = plist

	var doc := {
		"version": 1,
		"desc": "世界地图缩略数据（test/regen_map_thumbs.gd 从六张地图场景 dump 的派生数据，勿手改）："
			+ "layout=拼接布局（格子单位偏移 + 显示名）；maps=地形网格（cells 行主序，值为 terrains 下标）；"
			+ "portals=传送点（cell/to/to_cell/label）。地图尺寸/种子/地形/传送点改动后重跑该工具。",
		"layout": LAYOUT,
		"maps": maps,
		"portals": portals,
	}
	var f := FileAccess.open(OUT_PATH, FileAccess.WRITE)
	if f == null:
		push_error("无法写入 %s" % OUT_PATH)
		quit(1)
		return
	f.store_string(JSON.stringify(doc, "  "))
	f.close()
	var total := 0
	for a: String in maps:
		total += maps[a]["cells"].size() as int
	print("[MapThumbs] 写出 %s（%d 张地图 / %d 格 / %d 传送点）" % [
		OUT_PATH, maps.size(), total, _count_portals(portals)])


static func _count_portals(portals: Dictionary) -> int:
	var n := 0
	for k: String in portals:
		n += (portals[k] as Array).size()
	return n


static func _alias_of(fname: String) -> String:
	return "xiuxian_" + fname.replace(".tscn", "")
