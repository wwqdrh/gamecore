extends Panel
# 世界舆图弹窗（MapModal 内容面板）
#
# 职责：把 XiuMapThumbs 缩略数据（六图地形/传送点/拼接布局）灌入 GdMiniMap，
# 并随「Modal 打开 / 地图切换」刷新当前地图高亮与玩家位置光点。
# MapManager 经场景根组件表获取（同 player.gd 范式）；取子节点一律
# GdUIManager.get_gml_node（免写长路径，见 ui-gml.md）。

const PLAYER_HIDDEN := Vector2i(-9999, -9999)

@onready var _minimap: Control = GdUIManager.get_gml_node(self, "WorldMap")
@onready var _cur_label: Label = GdUIManager.get_gml_node(self, "CurrentName")

var _map_mgr: Node = null


func _ready() -> void:
	_populate()
	# Modal 打开即刷新（玩家位置/当前地图随游戏进行变化）。
	# 用父链找宿主 Modal：面板 _ready 处于 Modal 装配期内，ui_id 注册表
	# 可能尚未登记（find_ui 落空），父链是确定性锚点
	var modal := _find_modal_ancestor()
	if modal != null and not modal.is_connected("s_modal_opened", _refresh):
		modal.connect("s_modal_opened", _refresh)
	_refresh.call_deferred()


## 沿父链向上找宿主 Modal（有 is_modal_open 方法的组件即 GdUIModal）
func _find_modal_ancestor() -> Node:
	var p: Node = get_parent()
	while p != null:
		if p.has_method("is_modal_open"):
			return p
		p = p.get_parent()
	return null


## 灌入世界缩略数据（布局顺序西→东，保证绘制稳定）
func _populate() -> void:
	if _minimap == null:
		return
	_minimap.call("clear_all")
	var layout: Dictionary = XiuMapThumbs.get_layout()
	var maps: Dictionary = XiuMapThumbs.get_maps()
	var portals: Dictionary = XiuMapThumbs.get_portals()
	for alias in XiuMapThumbs.get_ordered_aliases():
		var info: Dictionary = layout.get(alias, {})
		var m: Dictionary = maps.get(alias, {})
		if m.is_empty():
			continue
		_minimap.call("add_map", alias, str(info.get("name", alias)),
			int(info.get("x", 0)), int(info.get("y", 0)),
			int(m["w"]), int(m["h"]),
			PackedInt32Array(m["cells"]), PackedStringArray(m["terrains"]))
		for p in portals.get(alias, []):
			var c: Array = p["cell"]
			var tc: Array = p["to_cell"]
			_minimap.call("add_portal", alias,
				Vector2i(int(c[0]), int(c[1])),
				str(p["to"]),
				Vector2i(int(tc[0]), int(tc[1])),
				str(p.get("label", "")))


## 刷新当前地图高亮 + 玩家光点 + 标题
func _refresh() -> void:
	if _minimap == null:
		return
	# MapManager 在 process 中自注册，面板 _ready 时可能尚未就绪 → 惰性重查
	# + 补接 s_map_changed（_ready 时缓存落空后此处自愈）
	if _map_mgr == null:
		_map_mgr = _find_map_manager()
		if _map_mgr != null and not _map_mgr.is_connected("s_map_changed", _on_map_changed):
			_map_mgr.connect("s_map_changed", _on_map_changed)
	var alias := ""
	if _map_mgr != null:
		alias = str(_map_mgr.get_current_alias())
	if alias.is_empty():
		alias = "xiuxian_home"  # 地图未开时兜底（首帧/切场景间隙，与出生点一致）
	_minimap.call("set_current", alias)
	_minimap.call("set_player_cell", _resolve_player_cell(alias))
	if _cur_label != null:
		_cur_label.text = "当前：%s" % XiuMapThumbs.get_map_name(alias)


## 玩家在当前地图内的格子（玩家/地图未就绪 → 隐藏光点）
func _resolve_player_cell(_alias: String) -> Vector2i:
	if not is_inside_tree() or _map_mgr == null:
		return PLAYER_HIDDEN
	var map: Node = _map_mgr.get_current_map()
	# 玩家挂在 GdScene（Index）下，与 GDCORE 全局场景根无父子关系
	# （探针手动挂载场景时尤其如此）→ 从场景树根递归找
	var player: Node = get_tree().root.find_child("Player", true, false)
	if map == null or player == null or not map.has_method("world_to_cell"):
		return PLAYER_HIDDEN
	return map.call("world_to_cell", player.global_position)


## 场景根（GdSceneRoot）：经 GDCORE 全局节点表获取
func _find_scene_root() -> Node:
	if not Engine.has_singleton("GDCORE"):
		return null
	var gdcore: Object = Engine.get_singleton("GDCORE")
	var node = gdcore.get_global_node("default")
	return node if node is Node else null


## MapManager 经场景根组件表获取（MapManager 在 process 中自注册，可能晚到）
func _find_map_manager() -> Node:
	var scene_root: Node = _find_scene_root()
	if scene_root == null:
		return null
	var mgr: Node = scene_root.get_component("MapManager")
	if mgr != null and mgr.has_method("get_current_map"):
		return mgr
	return null


func _on_map_changed(_alias: String) -> void:
	_refresh()
