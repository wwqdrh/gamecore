# 端到端验收：地图传送点接入 home + 地形 shader 贴图（terrain_tex）
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_map_teleport_tex.gd
#
# 覆盖：
#   A. GdQuickMap shader 贴图（xiaozhai）：grass 图层挂内置 terrain_tex，
#      tex0 = grass_texture.png，逐格遮罩只有 grass 格生效
#   B. GdDigLayer shader 贴图（home）：根图层材质 + 遮罩；挖地后遮罩同步
#   C. 传送链路：MapManager 注册 → xiaozhai 传送点 → home → 回程传送点
#   D. 主场景注册：index.tscn MapManager.map_paths 含 xiuxian_home
extends SceneTree

const XIAOZHAI := "res://example/demo/xiuxian/map/xiaozhai.tscn"
const HOME := "res://example/demo/xiuxian/map/home.tscn"
const MAIN := "res://example/demo/xiuxian/scenes/main/index.tscn"
const TEX_PATH := "res://example/demo/xiuxian/map/assets/grass_texture.png"

var ok := true


func _initialize() -> void:
	# 看门狗：脚本错误中止协程时强制退出（防零输出假死）
	create_timer(120.0).timeout.connect(func() -> void:
		print("[TeleTex] WATCHDOG 超时——测试未完成，强制退出")
		quit(2))
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[TeleTex] OK  ", msg)
	else:
		push_error("[TeleTex] FAIL  " + msg)
		ok = false


func _wait_frames(n: int) -> void:
	for i in n:
		await process_frame


func _finish() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	print("[TeleTex] RESULT: ", "ALL GREEN" if ok else "HAS FAILURES")
	quit(0 if ok else 1)


## 读 ShaderMaterial uniform（texture 参数转 Texture2D）
func _mat_tex(mat: Material, param: String) -> Texture2D:
	var v: Variant = mat.call("get_shader_parameter", param)
	if v == null or not v is Texture2D:
		return null
	return v as Texture2D


func _run() -> void:
	# 存档隔离：home 的 DigSave 会随档写地形网格（挖地同步用例），严禁污染
	# 玩家真实存档；清空存档地形保证从初始配置起步
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	XiuMapState.ins().clear_terrain_grids()
	# ---- A. GdQuickMap shader 贴图（xiaozhai，grass = 下标 0）----
	var xz: Node = load(XIAOZHAI).instantiate()
	root.add_child(xz)
	await _wait_frames(4)
	var layer: TileMapLayer = xz.get_terrain_layer(0)
	_check(layer != null, "xiaozhai grass 图层应存在")
	if layer != null:
		var mat: Material = layer.get_material()
		_check(mat != null and mat is ShaderMaterial, "grass 图层应挂 ShaderMaterial")
		if mat is ShaderMaterial:
			var code: String = str((mat as ShaderMaterial).shader.get_code())
			_check(code.contains("mask_tex"), "shader 应含 mask_tex uniform（内置 terrain_tex）")
			var tex0 := _mat_tex(mat, "tex0")
			_check(tex0 != null and tex0.resource_path == TEX_PATH,
				"tex0 应为 grass_texture.png，实际 %s" % (tex0.resource_path if tex0 else "null"))
			var mask := _mat_tex(mat, "mask_tex")
			_check(mask != null, "mask_tex 遮罩应存在")
			if mask != null:
				var img: Image = mask.get_image()
				# 找一格 grass 与一格 mountain 验证遮罩
				var grass_cell := Vector2i(-1, -1)
				for cy in 14:
					for cx in 20:
						if xz.get_terrain_at(Vector2i(cx, cy)) == 0:
							grass_cell = Vector2i(cx, cy)
							break
					if grass_cell.x >= 0:
						break
				_check(grass_cell.x >= 0, "应存在 grass 格")
				var mtn_r: float = img.get_pixel(9, 3).r
				_check(img.get_pixel(grass_cell.x, grass_cell.y).r > 0.5,
					"grass 格 %s 遮罩应命中" % grass_cell)
				_check(mtn_r < 0.5, "mountain 格 (9,3)（手绘建筑）遮罩不应命中")
	xz.queue_free()
	await _wait_frames(2)

	# ---- B. GdDigLayer shader 贴图（home，grass = 下标 1）----
	var home: Node = load(HOME).instantiate()
	root.add_child(home)
	await _wait_frames(4)
	var hmat: Material = home.get_material()
	_check(hmat != null and hmat is ShaderMaterial, "home 根图层应挂 ShaderMaterial")
	if hmat is ShaderMaterial:
		var tex0 := _mat_tex(hmat, "tex0")
		_check(tex0 != null and tex0.resource_path == TEX_PATH,
			"home tex0 应为 grass_texture.png")
		var mask := _mat_tex(hmat, "mask_tex")
		_check(mask != null, "home mask_tex 应存在")
		if mask != null:
			# 遮罩权威数据源是内存 Image（headless 哑渲染器下 GPU 纹理
			# 不回读），经 get_shader_mask_at 读取
			_check(home.call("get_shader_mask_at", Vector2i(3, 3)).r > 0.5,
				"草地格 (3,3) 遮罩应命中（r 通道）")
			_check(home.call("get_shader_mask_at", Vector2i(16, 1)).r > 0.5,
				"四周装饰草格 (16,1) 遮罩应命中（整图默认草地）")
			# 挖地 → 遮罩同步
			home.dig(Vector2i(9, 6))
			_check(home.call("get_shader_mask_at", Vector2i(9, 6)).r < 0.5,
				"挖掉的草地格 (9,6) 遮罩应即时清除")
			_check(home.call("get_shader_mask_at", Vector2i(10, 6)).r > 0.5,
				"相邻草地格 (10,6) 遮罩应保留")
	home.queue_free()
	await _wait_frames(2)

	# ---- C. 传送链路（MapManager 真实调度）----
	var mgr: GdMapManager = GdMapManager.new()
	mgr.manager_id = "_teletex_smoke"
	root.add_child(mgr)
	mgr.register_map("xiuxian_xiaozhai", XIAOZHAI)
	mgr.register_map("xiuxian_home", HOME)
	_check(mgr.open_map("xiuxian_xiaozhai", Vector2i(10, 8)), "open_map(xiaozhai) 应成功")
	# 收集传送点
	var home_marker: Node2D = null
	for m in mgr.get_markers():
		if str(m.call("get_target_alias")) == "xiuxian_home":
			home_marker = m
			break
	_check(home_marker != null, "xiaozhai 应有 target=xiuxian_home 的传送点")
	if home_marker != null:
		var center: Vector2 = home_marker.call("get_center")
		_check(mgr.try_teleport(center), "try_teleport(传送点格心) 应触发")
		_check(str(mgr.get_current_alias()) == "xiuxian_home",
			"传送后应处于 xiuxian_home，实际 %s" % mgr.get_current_alias())
		_check(mgr.get_spawn_cell() == Vector2i(10, 8),
			"落点应为 target_cell (10,8)，实际 %s" % mgr.get_spawn_cell())
	# 回程：home → xiaozhai
	var back_marker: Node2D = null
	for m in mgr.get_markers():
		if str(m.call("get_target_alias")) == "xiuxian_xiaozhai":
			back_marker = m
			break
	_check(back_marker != null, "home 应有回萧宅的传送点")
	if back_marker != null:
		_check(mgr.try_teleport(back_marker.call("get_center")), "回程传送应触发")
		_check(str(mgr.get_current_alias()) == "xiuxian_xiaozhai"
			and mgr.get_spawn_cell() == Vector2i(14, 12),
			"回程应落回萧宅入口 (14,12)，实际 %s@%s" % [mgr.get_current_alias(), mgr.get_spawn_cell()])
	mgr.queue_free()
	await _wait_frames(2)

	# ---- D. 主场景注册检查（不实例化，读 PackedScene state）----
	var ps: PackedScene = load(MAIN)
	var st := ps.get_state()
	var found := false
	for i in st.get_node_count():
		if st.get_node_name(i) != "MapManager":
			continue
		for p in st.get_node_property_count(i):
			if st.get_node_property_name(i, p) == "map_paths":
				var paths := str(st.get_node_property_value(i, p))
				found = paths.contains("xiuxian_home=") and paths.contains("home.tscn")
	print("[TeleTex] index.tscn map_paths 含 xiuxian_home: ", found)
	_check(found, "主场景 MapManager.map_paths 应注册 xiuxian_home")

	_finish()
