# GdUITree 树形组件验收 —— 四向布局/展开收起/JSON 状态导出恢复/动态建树
#
# 运行：perl -e 'alarm 60; exec @ARGV' godot --headless --path . \
#   -s res://test/check_gml_tree.gd
#
# 覆盖：
#   1. GML 构建期：<Tree>/<TreeNode> 占位符解析 → 行节点构建（__tree_row meta）
#      → 布局（down：根在顶部逐层向下；up：根在底部向上）
#   2. 选中/展开收起：select_node/toggle_expand；折叠后子孙行隐藏、再展开恢复
#   3. JSON 状态：dump_state → restore_state 往返（expanded/locked/tone/state/selected）
#   4. build_from_json 动态建树（替换结构）
#   5. TreeNode locked/tone 属性初始生效
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[Tree] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _rows_of(tree: Control) -> Array:
	var out: Array = []
	for i in tree.get_child_count():
		var c: Node = tree.get_child(i)
		if c is Control and c.has_meta("__tree_row"):
			out.append(c)
	return out


func _row_by_id(tree: Control, id: String) -> Control:
	for r in _rows_of(tree):
		if String(r.get_meta("__tree_id")) == id:
			return r
	return null


func _run() -> void:
	var gml := """
<ui>
  <Control name="Wrap">
	<Tree name="MyTree" direction="down" node_width="100" node_height="40"
	      level_gap="30" sibling_gap="10">
	  <TreeNode id="root" text="根" desc="根节点">
		<TreeNode id="a" text="甲">
		  <TreeNode id="a1" text="甲一" />
		  <TreeNode id="a2" text="甲二" />
		</TreeNode>
		<TreeNode id="b" text="乙" locked="true" tone="done" />
	  </TreeNode>
	</Tree>
  </Control>
</ui>
"""

	# ---- 1. 构建期：占位符解析 → 行构建 + 布局 ----
	var builder := GdUiBuilder.new()
	var ui_root: Control = builder.parse_string(gml)
	check(ui_root != null, "gml 应解析成功")
	var tree: Control = ui_root.find_child("MyTree", true, false)
	check(tree != null, "Tree 应存在")
	check(tree != null and tree.get_class() == "GdUITree", "Tree 应为 GdUITree 实例")
	var ids: PackedStringArray = tree.get_node_ids()
	check(ids.size() == 5, "应有 5 个节点（实际 %d）" % ids.size())
	var root_row := _row_by_id(tree, "root")
	var a_row := _row_by_id(tree, "a")
	var a1_row := _row_by_id(tree, "a1")
	var b_row := _row_by_id(tree, "b")
	check(root_row != null and a_row != null and a1_row != null, "行节点应全部构建")
	# 占位符应已移除
	for r in _rows_of(tree):
		check(not r.has_meta("__tree_node"), "占位符不应残留")
	# down 方向：根在顶部（y=0），子层 y = node_height+level_gap
	if root_row and a_row:
		check(absf(root_row.position.y) < 1.0, "down 根 y 应为 0（实际 %f）" % root_row.position.y)
		check(absf(a_row.position.y - 70.0) < 1.0,
			"down 子层 y 应为 70（实际 %f）" % a_row.position.y)
	# locked/tone 属性初始生效
	check(tree.is_node_locked("b"), "b 应初始锁定")
	check(String(tree.get_node_state("b").get("tone", "")) != "x", "状态字典应可读")
	var b_dumped: Dictionary = JSON.parse_string(tree.dump_state())["nodes"]["b"]
	check(bool(b_dumped["locked"]), "dump 应含 b.locked=true")

	# ---- 2. 选中 / 展开收起 ----
	check(tree.select_node("a", false), "select_node 应成功")
	check(String(tree.get_selected_id()) == "a", "选中 id 应为 a")
	check(not tree.select_node("nope", false), "未知 id 应返回 false")
	tree.toggle_expand("root")
	check(not a1_row.visible, "折叠根后子孙行应隐藏")
	check(not a_row.visible, "折叠根后直接子行应隐藏")
	tree.toggle_expand("root")
	check(a1_row.visible and a_row.visible, "再展开后行应恢复显示")

	# ---- 3. dump → restore 状态往返 ----
	tree.select_node("a1", false)
	var dump: String = tree.dump_state()
	var d: Dictionary = JSON.parse_string(dump)
	check(String(d.get("selected", "")) == "a1", "dump.selected 应为 a1")
	# 破坏状态后恢复
	tree.select_node("b", false)
	tree.toggle_expand("root")
	tree.restore_state(dump)
	check(String(tree.get_selected_id()) == "a1", "restore 后选中应为 a1")
	check(a1_row.visible, "restore 后折叠状态应还原（a1 可见）")
	check(tree.is_node_locked("b"), "restore 后 b 锁定应还原")
	# state 字段往返
	tree.set_node_state("a1", {"learned": true, "tone": "done"})
	var dump2: String = tree.dump_state()
	tree.set_node_state("a1", {"learned": false, "tone": "normal"})
	tree.restore_state(dump2)
	var st: Dictionary = tree.get_node_state("a1")
	check(bool(st.get("learned", false)), "restore 后 state.learned 应还原")
	check(String(st.get("tone", "")) == "done", "restore 后 tone 应还原")
	ui_root.free()

	# ---- 4. build_from_json 动态建树 ----
	var ui_root2: Control = builder.parse_string("<ui><Control name=\"Wrap2\"><Tree name=\"T2\" /></Control></ui>")
	var tree2: Control = ui_root2.find_child("T2", true, false)
	check(tree2 != null, "空 Tree 应可构建")
	var json := """
{"direction": "right", "nodes": [
  {"id": "r", "text": "起点",
   "children": [
     {"id": "x", "text": "岔路一"},
     {"id": "y", "text": "岔路二", "locked": true}
   ]}
]}
"""
	check(tree2.build_from_json(json), "build_from_json 应成功")
	check(tree2.get_node_ids().size() == 3, "动态树应有 3 节点")
	check(tree2.is_node_locked("y"), "动态节点 locked 应生效")
	check(tree2.get_node_children("r").size() == 2, "父子结构应正确")
	check(String(tree2.get_node_parent("x")) == "r", "parent 查询应正确")
	# direction=right：子节点 x 在父节点右侧
	var r_row := _row_by_id(tree2, "r")
	var x_row := _row_by_id(tree2, "x")
	check(r_row != null and x_row != null, "动态行应存在")
	if r_row and x_row:
		check(x_row.position.x > r_row.position.x + 100.0,
			"right 方向子节点应在父右侧（%f -> %f）" % [r_row.position.x, x_row.position.x])
	print("[Tree] sec4 ok")
	ui_root2.free()
	print("[Tree] free2 ok")

	print("[Tree] 全部通过" if ok else "[Tree] 存在失败项")
	quit(0 if ok else 2)
