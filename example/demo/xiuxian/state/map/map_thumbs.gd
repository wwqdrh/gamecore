# 世界地图缩略数据加载器（派生数据的只读入口）
#
# 数据源：state/map/map_thumbs.json（test/regen_map_thumbs.gd 从六张地图
# 场景离线 dump：地形网格 + 传送点 + 拼接布局；地图改动后重跑该工具）
# 消费方：main/ui/map/map_modal.gd → GdMiniMap 渲染世界拼接缩略图
class_name XiuMapThumbs
extends RefCounted

## 缩略数据路径（派生产物，勿手改）
const DATA_PATH := "res://example/demo/xiuxian/state/map/map_thumbs.json"

static var _cache: Dictionary = {}


## 加载缩略数据（首次读盘缓存；force=true 强制重读）
static func load_data(force: bool = false) -> Dictionary:
	if not _cache.is_empty() and not force:
		return _cache
	if not FileAccess.file_exists(DATA_PATH):
		push_error("[XiuMapThumbs] 缩略数据缺失: %s（重跑 test/regen_map_thumbs.gd）" % DATA_PATH)
		_cache = {}
		return _cache
	var text := FileAccess.get_file_as_string(DATA_PATH)
	var parsed: Variant = JSON.parse_string(text)
	if parsed is Dictionary:
		_cache = parsed
	else:
		push_error("[XiuMapThumbs] 缩略数据解析失败: %s" % DATA_PATH)
		_cache = {}
	return _cache


## 拼接布局：alias -> {x, y, name}（格子单位偏移 + 显示名）
static func get_layout() -> Dictionary:
	return load_data().get("layout", {})


## 地形网格：alias -> {w, h, terrains: [名...], cells: [索引...]（行主序）}
static func get_maps() -> Dictionary:
	return load_data().get("maps", {})


## 传送点：alias -> [{cell:[x,y], to, to_cell:[x,y], label}]
static func get_portals() -> Dictionary:
	return load_data().get("portals", {})


## 按布局顺序返回 alias 列表（西→东稳定排序，缩略图绘制顺序）
static func get_ordered_aliases() -> Array:
	var layout := get_layout()
	var aliases: Array = layout.keys()
	aliases.sort_custom(func(a: String, b: String) -> bool:
		var la: Dictionary = layout[a]
		var lb: Dictionary = layout[b]
		if int(la["x"]) != int(lb["x"]):
			return int(la["x"]) < int(lb["x"])
		return int(la["y"]) < int(lb["y"]))
	return aliases


## 地图显示名（无记录回退 alias）
static func get_map_name(alias: String) -> String:
	var info: Dictionary = get_layout().get(alias, {})
	return str(info.get("name", alias))
