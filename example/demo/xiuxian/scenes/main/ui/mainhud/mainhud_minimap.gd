extends Panel
# 右上角小地图（mainhud）：GdMiniMap 聚焦模式（fit_current=true，只画当前
# 地图、自适应铺满控件）。数据源同世界舆图 MapModal——XiuMapThumbs 离线
# dump 一次灌入；刷新分两层：
#   · 地图切换：MapManager.s_map_changed 信号（惰性重查补接，同 map_modal.gd）
#   · 玩家移动：_process 0.25s 节流重算玩家格
# 取子节点一律 GdUIManager.get_gml_node（免写长路径，见 ui-gml.md）。

const PLAYER_HIDDEN := Vector2i(-9999, -9999)
const REFRESH_INTERVAL := 0.25

@onready var _minimap: Control = GdUIManager.get_gml_node(self, "MiniMapView")
@onready var _name_label: Label = GdUIManager.get_gml_node(self, "MapName")

var _map_mgr: Node = null
var _acc := 0.0


func _ready() -> void:
	_populate()
	# 红线：mainhud _ready 处在 change_scene 装配期（do_swap_scene rust 栈）
	# 内，此时调 GdSceneRoot.get_component() 会单借用重入 panic
	# （Gd<T>::bind failed, already bound）→ 首刷延迟到帧末树外执行
	_refresh.call_deferred()


func _process(delta: float) -> void:
	_acc += delta
	if _acc < REFRESH_INTERVAL:
		return
	_acc = 0.0
	_heal_dangling_mgr()
	# MapManager 晚到自愈：_ready 时组件表未登记 → 惰性重查走全量
	# _refresh（补接 s_map_changed）；已就绪则只刷玩家光点（轻路径）
	if _map_mgr == null:
		_refresh()
	else:
		_refresh_player()


## 缓存的 MapManager 悬垂自愈（旧场景实例已释放 → 置空待重查；
## 信号连接随旧实例释放自动断开，无需手动 disconnect）
func _heal_dangling_mgr() -> void:
	if _map_mgr != null and not is_instance_valid(_map_mgr):
		_map_mgr = null


## 灌入世界缩略数据（聚焦模式只画 set_current 的那一张，数据仍全量灌入
## ——切图只改高亮，无需重灌；布局顺序西→东保证绘制稳定）
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


## 刷新当前地图高亮 + 标题（玩家光点走 _refresh_player 节流）。
## 可选参：直连 s_map_changed(alias) 信号（带参调用）与本地直调（无参）两用
func _refresh(_changed_alias: String = "") -> void:
	if _minimap == null:
		return
	_heal_dangling_mgr()
	# MapManager 在 process 中自注册，面板 _ready 时可能尚未就绪 → 惰性重查
	# + 补接 s_map_changed（与 map_modal.gd 同款自愈）
	if _map_mgr == null:
		_map_mgr = _find_map_manager()
		if _map_mgr != null and not _map_mgr.is_connected("s_map_changed", _refresh):
			_map_mgr.connect("s_map_changed", _refresh)
	var alias := ""
	if _map_mgr != null:
		alias = str(_map_mgr.get_current_alias())
	if alias.is_empty():
		alias = "xiuxian_xiaozhai"  # 地图未开时兜底（首帧/切场景间隙）
	_minimap.call("set_current", alias)
	if _name_label != null:
		_name_label.text = XiuMapThumbs.get_map_name(alias)
	_refresh_player()


## 玩家光点：当前地图内的格子（玩家/地图未就绪 → 隐藏光点）
func _refresh_player() -> void:
	if _minimap == null:
		return
	_minimap.call("set_player_cell", _resolve_player_cell())


func _resolve_player_cell() -> Vector2i:
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
## 无类型接收 + is_instance_valid 校验：防切场景间隙组件表残留死引用
## （freed 实例赋给 Node 类型变量会报 "Trying to assign invalid previously
## freed instance"；框架侧 get_component 已自愈，此处为双保险）
func _find_map_manager() -> Node:
	var scene_root: Node = _find_scene_root()
	if scene_root == null or not is_instance_valid(scene_root):
		return null
	var mgr = scene_root.get_component("MapManager")
	if mgr != null and is_instance_valid(mgr) and mgr.has_method("get_current_map"):
		return mgr
	return null
