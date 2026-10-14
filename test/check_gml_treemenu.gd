# GdUITreeMenu 树形菜单组件验收 —— 多级缩进/折叠展开/选中/信号
#
# 运行：perl -e 'alarm 60; exec @ARGV' godot --headless --path . \
#   -s res://test/check_gml_treemenu.gd
#
# 覆盖：
#   1. 构建期：<TreeMenu>/<MenuNode> 占位符解析 → 行构建（__menu_row meta）
#   2. 自动缩进：二级行文字 x = depth*indent + 箭头槽
#   3. 默认展开；toggle_expand 折叠后子行隐藏、再展开恢复
#   4. select_item / get_selected_id / 未知 id
#   5. 行点击：选中 + 发信号 + 有子菜单时联动折叠
#   6. expand_all / collapse_all
#   7. dump_state 快照
extends SceneTree

var ok := true
var sig_count := 0
var sig_last := ""


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[TreeMenu] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _rows_of(menu: Control) -> Array:
	var out: Array = []
	for i in menu.get_child_count():
		var c: Node = menu.get_child(i)
		if c is Control and c.has_meta("__menu_row"):
			out.append(c)
	return out


func _row_by_id(menu: Control, id: String) -> Control:
	for r in _rows_of(menu):
		if String(r.get_meta("__menu_id")) == id:
			return r
	return null


func _run() -> void:
	var gml := """
<ui>
  <Control name="Wrap">
	<Control name="Holder">
	  <TreeMenu name="MyMenu" indent_width="18" row_height="34" row_gap="2"
	            font_size="14">
		<MenuNode id="basics" text="基础知识">
		  <MenuNode id="ability" text="能力体系" />
		</MenuNode>
		<MenuNode id="items" text="物品图鉴" />
	  </TreeMenu>
	</Control>
  </Control>
</ui>
"""
	var builder := GdUiBuilder.new()
	var ui_root: Control = builder.parse_string(gml)
	check(ui_root != null, "gml 应解析成功")
	var menu: Control = ui_root.find_child("MyMenu", true, false)
	check(menu != null, "TreeMenu 应存在")
	check(menu != null and menu.get_class() == "GdUITreeMenu", "应为 GdUITreeMenu 实例")

	# ---- 1. 构建期：行构建 ----
	check(int(menu.call("get_item_count")) == 3, "应有 3 个菜单项")
	var basics := _row_by_id(menu, "basics")
	var ability := _row_by_id(menu, "ability")
	var items := _row_by_id(menu, "items")
	check(basics != null and ability != null and items != null, "行节点应全部构建")
	for r in _rows_of(menu):
		check(not r.has_meta("__menu_node"), "占位符不应残留")

	# ---- 2. 自动缩进（文字标签 x = depth*18 + 20 + 2）----
	if basics and ability:
		var bt: Control = basics.get_node("__MenuText")
		var at: Control = ability.get_node("__MenuText")
		check(absf(bt.position.x - 22.0) < 1.0, "一级缩进 x 应 22（实际 %f）" % bt.position.x)
		check(absf(at.position.x - 40.0) < 1.0, "二级缩进 x 应 40（实际 %f）" % at.position.x)

	# ---- 3. 默认展开 + 折叠 ----
	check(bool(menu.call("is_item_expanded", "basics")), "菜单应默认展开")
	check(ability.visible, "展开态子行应可见")
	check(menu.call("toggle_expand", "basics"), "toggle_expand 应成功")
	check(not ability.visible, "折叠后子行应隐藏")
	check(not bool(menu.call("is_item_expanded", "basics")), "折叠后状态应为 false")
	check(menu.call("toggle_expand", "basics"), "再 toggle 应成功")
	check(ability.visible, "再展开后子行应恢复")

	# ---- 4. 选中 ----
	check(menu.call("select_item", "ability", false), "select_item 应成功")
	check(String(menu.call("get_selected_id")) == "ability", "选中 id 应为 ability")
	check(not menu.call("select_item", "nope", false), "未知 id 应返回 false")

	# ---- 5. 行点击：选中+信号+联动折叠 ----
	menu.connect("s_menu_selected", func(id: String) -> void:
		sig_count += 1
		sig_last = id)
	menu.call("_on_row_pressed", 0)  # basics 行：选中 + 联动折叠
	check(String(menu.call("get_selected_id")) == "basics", "行点击应选中 basics")
	check(sig_count == 1 and sig_last == "basics", "应发一次 s_menu_selected(basics)")
	check(not ability.visible, "有子菜单的行点击应联动折叠")
	menu.call("_on_row_pressed", 2)  # items 行（叶子）：只选中
	check(String(menu.call("get_selected_id")) == "items", "叶子行点击应选中")
	check(sig_count == 2 and sig_last == "items", "叶子点击也应发信号")

	# ---- 6. expand_all / collapse_all ----
	menu.call("expand_all")
	check(ability.visible, "expand_all 后子行应恢复")
	menu.call("collapse_all")
	check(not ability.visible, "collapse_all 后子行应隐藏")
	menu.call("expand_all")

	# ---- 7. dump_state ----
	var dump: Dictionary = menu.call("dump_state")
	check(String(dump.get("selected", "")) == "items", "dump 应含选中项")
	var exp: Dictionary = dump.get("expanded", {})
	check(bool(exp.get("basics", false)), "dump 应含展开状态")

	print("[TreeMenu] 全部通过" if ok else "[TreeMenu] 存在失败项")
	quit(0 if ok else 2)
