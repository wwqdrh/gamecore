# 修炼/功法面板验收 —— 功法树学习流（扣点/解锁/加成/存档往返）
#
# 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . \
#   -s res://test/check_level_panel.gd
#
# 覆盖：
#   1. 面板 tscn 加载 → <Tree> 行构建（构建期 initial 已序列化进 tscn）
#   2. 首次进入：根「道基」自动标记已领悟 + 解锁第一层
#   3. 领悟流：点击 gongfa1 → 扣技能点 → tone=done/learned → 子层解锁
#      → max_mp 加成入账 → skill_tree 快照落 Bean
#   4. 校验分支：锁定节点点击不消耗 / 已学节点不重复消耗
#   5. 持久化往返：新面板实例按 skill_tree 快照恢复状态
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[LevelPanel] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	# 存档隔离：headless 测试不碰玩家活档
	var guard: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
	guard.isolate()

	var char_state: GdBean = XiuCharacterState.ins()
	char_state.reset_demo()
	char_state.update("skill_points", 3, {}, true)

	var scene: PackedScene = load(
		"res://example/demo/xiuxian/scenes/main/ui/level/level_panel.gml.tscn")
	check(scene != null, "level_panel.gml.tscn 应存在（先跑 regen_gml_tscn）")
	if scene == null:
		guard.restore()
		quit(2)
		return
	var panel: Control = scene.instantiate()
	root.add_child(panel)
	await process_frame

	var tree: Control = panel.get_node_or_null(^"MainMargin/MainColumn/TreeScroll/GongfaTree")
	check(tree != null, "功法树应存在")
	check(tree.get_node_ids().size() == 13, "树应有 13 节点（实际 %d）" % tree.get_node_ids().size())

	# ---- 2. 首次进入：根已领悟 + 第一层解锁 ----
	var root_st: Dictionary = tree.get_node_state("root")
	check(bool(root_st.get("learned", false)), "根「道基」首次进入应已领悟")
	check(not tree.is_node_locked("gongfa1"), "第一层（吐纳诀）应解锁")
	check(tree.is_node_locked("gongfa2"), "第二层（聚气术）应保持锁定")

	# ---- 3. 领悟流 ----
	var mp_before := int(char_state.max_mp)
	panel._on_node_clicked("gongfa1")
	check(int(char_state.skill_points) == 2, "技能点应扣至 2（实际 %d）" % int(char_state.skill_points))
	var st1: Dictionary = tree.get_node_state("gongfa1")
	check(bool(st1.get("learned", false)), "gongfa1 应标记已学")
	check(String(st1.get("tone", "")) == "done", "gongfa1 tone 应为 done")
	check(not tree.is_node_locked("gongfa2"), "gongfa2 应随父解锁")
	check(int(char_state.max_mp) == mp_before + 20, "灵力上限应 +20（实际 %d）" % int(char_state.max_mp))
	check(not str(char_state.skill_tree).is_empty(), "skill_tree 快照应落 Bean")

	# ---- 4. 校验分支 ----
	var pts := int(char_state.skill_points)
	panel._on_node_clicked("gongfa1")  # 已学：不重复消耗
	check(int(char_state.skill_points) == pts, "已学节点不应重复扣点")
	panel._on_node_clicked("gongfa3")  # 锁定（gongfa2 未学）
	check(int(char_state.skill_points) == pts, "锁定节点不应扣点")
	check(not bool(tree.get_node_state("gongfa3").get("learned", false)), "锁定节点不应被学")
	# 点数不足
	char_state.update("skill_points", 0, {}, true)
	panel._on_node_clicked("gongfa2")
	check(not bool(tree.get_node_state("gongfa2").get("learned", false)), "点数不足不应学成")
	char_state.update("skill_points", 2, {}, true)

	# 连续领悟到第三层，丰富快照
	panel._on_node_clicked("gongfa2")
	panel._on_node_clicked("gongfa3")
	check(bool(tree.get_node_state("gongfa3").get("learned", false)), "gongfa3 应可学成")
	var snapshot := str(char_state.skill_tree)
	check(snapshot.contains("gongfa3"), "快照应含 gongfa3")

	# ---- 5. 持久化往返：新实例按快照恢复 ----
	panel.queue_free()
	await process_frame
	var panel2: Control = scene.instantiate()
	root.add_child(panel2)
	await process_frame
	var tree2: Control = panel2.get_node_or_null(^"MainMargin/MainColumn/TreeScroll/GongfaTree")
	check(tree2 != null, "第二实例树应存在")
	check(bool(tree2.get_node_state("gongfa1").get("learned", false)), "恢复后 gongfa1 应已学")
	check(bool(tree2.get_node_state("gongfa3").get("learned", false)), "恢复后 gongfa3 应已学")
	check(tree2.is_node_locked("body2"), "体修第二层应仍锁定")
	check(not tree2.is_node_locked("gongfa4"), "gongfa4 应随恢复解锁")

	# 清理演示态
	char_state.reset_demo()
	guard.restore()

	print("[LevelPanel] 全部通过" if ok else "[LevelPanel] 存在失败项")
	quit(0 if ok else 2)
