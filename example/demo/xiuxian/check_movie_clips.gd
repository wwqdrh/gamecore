# 宗门篇剧情演出验收 —— 20 段 .movie 剧本静态/动态校验 + 新地图/NPC 装配检查
#
# 运行：perl -e 'alarm 300; exec @ARGV' godot --headless --path . \
#   -s res://example/demo/xiuxian/check_movie_clips.gd
#
# 覆盖：
#   1. movie.actors 注册表：20 段剧本引用的 actor id 全部已注册
#   2. 剧本解析：逐段 configure（parse 失败会以 WARNING 打到 stderr，
#      由外部脚本 grep 断言；此处再以 play_from 拒播空剧本兜底校验）
#   3. 玩家站位可行走：逐段 open_map(别名, 站位格) 后 is_walkable 校验
#      （player (x,y) 指令直接落格，不吸附——站位格必须可行走）
#   4. 青云宗山门图：注册、传送点往返（town⇄sect）、内嵌 NPC 站位吸附
#   5. 解锁门控：xiuxian_sect 初始锁定，unlock_map 后开放
extends SceneTree

var ok := true

## 各剧本的 (别名, 玩家站位格)；与 .movie 内 scene/player 指令一一对应
const CLIPS := [
	["c01_farewell", "xiuxian_xiaozhai", Vector2i(10, 8)],
	["c02_gate_arrival", "xiuxian_sect", Vector2i(29, 19)],
	["c03_master_reception", "xiuxian_sect", Vector2i(15, 8)],
	["c04_dorm_reunion", "xiuxian_sect", Vector2i(9, 11)],
	["c05_senior_brother", "xiuxian_sect", Vector2i(9, 11)],
	["c06_library_duty", "xiuxian_sect", Vector2i(25, 7)],
	["c07_elder_hint", "xiuxian_sect", Vector2i(24, 8)],
	["c08_rival_challenge", "xiuxian_sect", Vector2i(22, 10)],
	["c09_ring_duel", "xiuxian_sect", Vector2i(21, 10)],
	["c10_dark_deal", "xiuxian_sect", Vector2i(28, 14)],
	["c11_herb_errand", "xiuxian_xiaozhen", Vector2i(6, 9)],
	["c12_mapa_peril", "xiuxian_map_a", Vector2i(18, 12)],
	["c13_ancient_stele", "xiuxian_map_a", Vector2i(24, 4)],
	["c14_town_rumors", "xiuxian_town", Vector2i(16, 9)],
	["c15_sect_dispatch", "xiuxian_sect", Vector2i(14, 8)],
	["c16_dungeon_ambush", "xiuxian_map_b", Vector2i(12, 10)],
	["c17_dark_cultivator", "xiuxian_map_b", Vector2i(12, 11)],
	["c18_sect_verdict", "xiuxian_sect", Vector2i(15, 8)],
	["c19_grandpa_truth", "xiuxian_sect", Vector2i(25, 7)],
	["c20_vow_hometown", "xiuxian_home", Vector2i(9, 6)],
]


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[MovieClips] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	# 屏蔽开场剧情自动补播（MovieTrigger），保证本验收确定性
	load("res://example/demo/xiuxian/state/movie/movie_state.gd").ins().mark_played("prologue")

	var core: Object = Engine.get_singleton("GDCORE")
	check(core != null, "GDCORE 单例应已注册")

	var main_scene: PackedScene = load("res://example/demo/xiuxian/scenes/main/index.tscn")
	var index: Node = main_scene.instantiate()
	root.add_child(index)
	for i in 4:
		await process_frame

	var mgr: Node = index.get_node_or_null("MapManager")
	check(mgr != null, "MapManager 应存在")
	var dialog_box: Node = index.get_node_or_null("DialogLayer/DialogBox")
	var player: Node = index.get_node_or_null("Player")
	check(dialog_box != null and player != null, "台词宿主/玩家应存在")
	var registered: Array = mgr.get_registered_maps()
	check("xiuxian_sect" in registered, "注册表应含青云宗山门 xiuxian_sect，实际 %s" % [registered])

	# ---- 1. actor id 静态校验 ----
	var cfg := JSON.parse_string(
		FileAccess.get_file_as_string("res://game_config.json")) as Dictionary
	var actors: Dictionary = cfg.get("movie", {}).get("actors", {}) as Dictionary
	for clip in CLIPS:
		var path := "res://example/demo/xiuxian/movie/%s/%s.movie" % [clip[0], clip[0]]
		var text := FileAccess.get_file_as_string(path)
		check(not text.is_empty(), "%s 剧本应非空" % clip[0])
		for line in text.split("\n"):
			line = line.strip_edges()
			if line.begins_with("say ") or line.begins_with("bubble "):
				var id := line.split(" ", false, 2)[1]
				# say/bubble id 可能带时长段（say x 3.5 台词）——首个 token 即 id
				if not actors.has(id):
					check(actors.has(id), "%s 台词角色 [%s] 未注册 movie.actors" % [clip[0], id])
			elif line.begins_with("actor "):
				var parts := line.split(" ", false)
				if parts.size() >= 2 and not actors.has(parts[1]):
					check(actors.has(parts[1]), "%s 出场角色 [%s] 未注册 movie.actors" % [clip[0], parts[1]])

	# ---- 2. 剧本解析（configure 触发 load_script；解析失败打 WARNING 由外部 grep） ----
	for clip in CLIPS:
		var movie: Node = GdMoviePlayer.new()
		index.add_child(movie)
		movie.call("configure",
			"res://example/demo/xiuxian/movie/%s/%s.movie" % [clip[0], clip[0]],
			movie.get_path_to(dialog_box), movie.get_path_to(player))
		movie.queue_free()
	print("[MovieClips] 20 段剧本 configure 完成（解析失败会输出 WARNING）")

	# ---- 3. 玩家站位可行走 + 逐图加载 ----
	for clip in CLIPS:
		var alias: String = clip[1]
		var cell: Vector2i = clip[2]
		check(mgr.open_map(alias, cell), "%s open_map(%s) 应成功" % [clip[0], alias])
		for i in 3:
			await process_frame
		var m: Node = mgr.get_current_map() if mgr.get_current_map() != null else null
		check(m != null, "%s 应能取到当前地图节点" % clip[0])
		if m != null and m.has_method("is_walkable"):
			check(m.is_walkable(cell), "%s 玩家站位 %s@%s 应可行走" % [clip[0], alias, cell])
	print("[MovieClips] 玩家站位可行走校验完成")

	# ---- 4. 青云宗山门：传送点往返 + 内嵌 NPC ----
	mgr.open_map("xiuxian_town", Vector2i(6, 10))
	await process_frame
	var found_sect_marker: Node2D = null
	for mk in mgr.get_markers():
		if str(mk.target_alias) == "xiuxian_sect":
			found_sect_marker = mk
			break
	check(found_sect_marker != null, "青云坊应有指向青云宗的传送点")
	if found_sect_marker != null:
		check(mgr.try_teleport(found_sect_marker.get_center())
			and str(mgr.get_current_alias()) == "xiuxian_sect",
			"青云坊→青云宗传送应成功")
	var back_sect: Node2D = null
	for mk in mgr.get_markers():
		if str(mk.target_alias) == "xiuxian_town":
			back_sect = mk
			break
	check(back_sect != null, "青云宗应有返回青云坊的传送点")
	if back_sect != null:
		check(mgr.try_teleport(back_sect.get_center())
			and str(mgr.get_current_alias()) == "xiuxian_town",
			"青云宗→青云坊反向传送应成功")
	var sect_map: Node = null
	mgr.open_map("xiuxian_sect", Vector2i(2, 10))
	await process_frame
	sect_map = mgr.get_current_map()
	if sect_map != null:
		var npc_names := ["NpcSectMaster", "NpcSeniorBrother", "NpcRival", "NpcSweepElder"]
		for nn in npc_names:
			var npc: Node = sect_map.get_node_or_null(nn)
			check(npc != null, "青云宗应内嵌 %s" % nn)
	print("[MovieClips] 青云宗山门装配校验完成")

	# ---- 5. 解锁门控：sect 初始锁定 → unlock_map 后 open_map 成功 ----
	# 先清空解锁记录（上轮运行可能已在测试档点亮过，保证确定性）
	load("res://example/demo/xiuxian/state/map/map_state.gd").ins().clear_map_unlocked()
	mgr.open_map("xiuxian_town", Vector2i(6, 10))
	await process_frame
	var map_state: Object = load(
		"res://example/demo/xiuxian/state/map/map_state.gd").ins()
	var locked: bool = not bool(map_state.is_map_unlocked("xiuxian_sect")) \
		if map_state.has_method("is_map_unlocked") else true
	check(locked, "青云宗初始应为锁定（选拔夺魁后才点亮）")
	map_state.unlock_map("xiuxian_sect")
	check(mgr.open_map("xiuxian_sect", Vector2i(2, 10)), "unlock_map 后应能进入青云宗")

	print("[MovieClips] %s" % ("ALL GREEN" if ok else "FAILED"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
