# 玩家角色 —— 四向网格移动 + 地图传送联动 + 相机跟随绑定
#
# 组合框架能力（零自绘移动逻辑）：
#   - GdRoleMover MODE_GRID：WASD/方向键逐格四向移动（框架内禁止斜向），
#     grid_map_path 绑定当前地图（duck-type is_walkable 判定通行）
#   - 鼠标左键点击地面：GdQuickMap.find_path BFS（四方向最短路）→ set_grid_path 逐格走
#   - GdMapManager：落格后 try_teleport(世界坐标) 命中传送点即切图；
#     s_map_changed 后重定位出生点、重绑地图、刷新相机边界
#   - GdViewCamera（GdSceneRoot 自动挂载并注册进组件表）：follow 跟随玩家 +
#     update_limit 限制相机边界在地图矩形内 + 放大到视野小于地图的档位
#
# 组件查找：一律经 GdSceneRoot 组件表（get_component），禁止 get_parent
# 链遍历 / find_child 魔法查找——场景根经 GDCORE 全局节点表获取。
#
# 场景装配（scenes/main/index.tscn）：Player 与 MapManager 平级，玩家不在
# MapLayer 内——地图切换只替换 MapLayer 下的地图实例，玩家常驻。
extends GdRoleMover

var map_mgr: Node = null
var camera: Camera2D = null
var scene_root: Node = null

# 上次落格判定用格子（-9999 = 未初始化，首帧强制判定一次）
var last_cell := Vector2i(-9999, -9999)
var path_line: Line2D
var _wired := false


func _ready() -> void:
	# 加入 "player" 分组：GdDialogTrigger 等系统经分组解析玩家节点
	add_to_group("player")
	_make_visual()
	# 路径预览折线（黄色），走完清除
	path_line = Line2D.new()
	path_line.name = "PathPreview"
	path_line.width = 2.0
	path_line.default_color = Color(1.0, 0.85, 0.25, 0.75)
	path_line.z_index = 1
	add_child(path_line)
	s_grid_path_finished.connect(func() -> void: path_line.clear_points())
	# 接线由 _process 驱动（每帧重试直到完成）。
	# 红线：禁止 call_deferred 自重试——deferred 队列 flush 到空才结束，
	# 自重试会让队列永不为空，主循环卡死在 flush 内（process 永不执行）。


# ---- 接线 ----

func _wire() -> void:
	if _wired:
		return
	scene_root = _find_scene_root()
	if scene_root == null:
		return
	# 组件表查询：MapManager 由管理器自动注册，ViewCamera 由 GdSceneRoot 自动注册
	map_mgr = scene_root.get_component("MapManager")
	if map_mgr == null or not map_mgr.has_method("get_current_map"):
		return
	if map_mgr.get_current_map() == null:
		# 初始地图尚未加载完（同帧级联顺序问题），下帧再试
		return
	_wired = true
	map_mgr.connect("s_map_changed", _on_map_changed)
	camera = scene_root.get_component("ViewCamera")
	_apply_map(map_mgr.get_current_map(), true)


## 场景根（GdSceneRoot）：经 GDCORE 全局节点表获取（manager_id 默认 "default"）
func _find_scene_root() -> Node:
	if not Engine.has_singleton("GDCORE"):
		return null
	var gdcore: Object = Engine.get_singleton("GDCORE")
	var node = gdcore.get_global_node("default")
	return node if node is Node else null


# ---- 地图应用：网格绑定 + 相机跟随/边界 + 出生点 ----

func _apply_map(map: Node, snap: bool) -> void:
	if map == null:
		return
	# 清掉旧地图遗留的路径指令（下一格在新图上无意义）
	stop()
	path_line.clear_points()
	# 网格绑定：指向当前地图实例（换图后 grid_map 缓存失效自动重解析）
	var alias: String = map_mgr.get_current_alias()
	grid_map_path = NodePath("../MapManager/MapLayer/" + alias)
	# 相机：跟随玩家 + 边界限制在地图矩形内
	_apply_camera(map, snap)
	# 出生点（MapManager 已把不可行走出生格吸附到可行走格）
	if snap:
		global_position = map_mgr.get_spawn_point()
	last_cell = Vector2i(-9999, -9999)


func _apply_camera(map: Node, snap: bool) -> void:
	if camera == null and scene_root != null:
		# 相机可能晚于玩家接线创建（默认 GdSceneRoot 延迟添加），组件表重查
		camera = scene_root.get_component("ViewCamera")
	if camera == null or not camera.has_method("follow"):
		return  # 相机缺失，_process 中重试
	# 边界限制在地图矩形内 + 视野放大（zoom 2.5 → 视野 768x432，
	# 小于最小地图 832x512，边界限制才能生效不越界）
	camera.follow(self, snap, true)
	var w: int = map.get_map_width()
	var h: int = map.get_map_height()
	var cs: int = map.get_cell_size_px()
	camera.update_limit(Vector4(0, w * cs, 0, h * cs))
	if camera.zoom_max < 2.5:
		camera.zoom_max = 2.5
	camera.start_zoom(2, -1.0, -1.0)


func _on_map_changed(_alias: String) -> void:
	_apply_map(map_mgr.get_current_map(), true)


# ---- 传送判定：落格（含点击寻路途经）后查询地图管理器 ----

func _process(_delta: float) -> void:
	# 注意：不覆写 _physics_process（会遮蔽 GdRoleMover 的 Rust 侧网格驱动）
	# 接线重试在 process 阶段驱动（MapManager 组件注册也在其 process 中完成，
	# 同为 process 阶段即可收敛，且不会卡死 deferred flush）
	if not _wired:
		_wire()
		if not _wired:
			return
	if map_mgr == null:
		return
	# 相机缺失重试：默认 GdSceneRoot 延迟添加，找到后补一次跟随/边界绑定
	if camera == null:
		_apply_camera(map_mgr.get_current_map(), true)
	if map_mgr.get_current_map() == null:
		return
	if is_moving():
		return
	var map: Node = map_mgr.get_current_map()
	if map == null or not map.has_method("world_to_cell"):
		return
	var cell: Vector2i = map.world_to_cell(global_position)
	if cell != last_cell:
		last_cell = cell
		# 命中传送点：管理器切图并经 s_map_changed 回调重定位玩家
		map_mgr.try_teleport(global_position)


# ---- 点击寻路 ----

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed \
			and event.button_index == MOUSE_BUTTON_LEFT:
		_on_map_clicked()


func _on_map_clicked() -> void:
	if not _wired or map_mgr == null:
		return
	var map: Node = map_mgr.get_current_map()
	if map == null or not map.has_method("find_path"):
		return
	var target: Vector2i = map.world_to_cell(map.get_global_mouse_position())
	if not map.is_walkable(target):
		return
	var from: Vector2i = map.world_to_cell(global_position)
	if from == target:
		return
	var path: PackedVector2Array = map.find_path(from, target)
	if path.is_empty():
		return
	set_grid_path(path)
	# 预览线：当前位置 → 依次各格中心
	var pts := PackedVector2Array([global_position])
	pts.append_array(path)
	path_line.points = pts


# ---- 视觉：身体 + 朝向指示（正式素材就位后替换） ----

func _make_visual() -> void:
	var body := ColorRect.new()
	body.name = "Body"
	body.color = Color(0.3, 0.6, 1.0)
	body.size = Vector2(22, 22)
	body.position = -body.size / 2.0
	body.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(body)
	var eye := ColorRect.new()
	eye.name = "Eye"
	eye.color = Color(0.95, 0.98, 1.0)
	eye.size = Vector2(6, 6)
	eye.position = Vector2(2, -2)
	eye.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(eye)
	s_facing_changed.connect(_on_facing_changed)


func _on_facing_changed(facing: String) -> void:
	var eye: ColorRect = get_node_or_null("Eye")
	if eye == null:
		return
	match facing:
		"left":
			eye.position = Vector2(-8, -2)
		"right":
			eye.position = Vector2(2, -2)
		"up":
			eye.position = Vector2(-2, -8)
		_:
			eye.position = Vector2(-2, 2)
