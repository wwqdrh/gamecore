# suite: ui_signal - @pressed 信号声明与自动绑定
# 覆盖: 静态节点 @pressed 连接、列表条目构建期自动连接（connect_signals 走树）、
#       update() 重建条目后 bind_events 自动重连、回调补绑发出按钮、
#       缺失方法容错
extends "res://test/test_case.gd"


# 运行时构造目标脚本：含 0 参回调与 1 参条目回调
func _make_target() -> Node:
	var src := """
extends Node
var calls: Array = []
var last_btn: Control = null

func _on_close() -> void:
	calls.append("close")

func _on_task_action(btn: Control) -> void:
	calls.append("task")
	last_btn = btn
"""
	var script := GDScript.new()
	script.source_code = src
	script.reload()
	var node := Node.new()
	node.set_script(script)
	return node


func test_at_pressed_static() -> void:
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<VBoxContainer>
		<Button name="CloseBtn" @pressed="_on_close" />
	</VBoxContainer>
</ui>
""")
	assert_not_null(root, "解析不应失败")
	var target := _make_target()
	var btn: Button = root.find_child("CloseBtn", true, false)
	assert_not_null(btn, "按钮应存在")
	builder.connect_signals(root, target)
	if btn:
		btn.emit_signal("pressed")
	var calls: Array = target.get("calls")
	assert_eq(calls.size(), 1, "@pressed 声明应自动连接到目标方法")
	if calls.size() > 0:
		assert_eq(calls[0], "close", "回调方法应被正确调用")
	root.free()
	target.free()


func test_at_pressed_list_items_build_time() -> void:
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<script>
		var tasks = [
			{ title: "甲" },
			{ title: "乙" },
		]
	</script>
	<UIVList name="List" data="tasks">
		<Panel name="ItemRoot" custom_minimum_size="0,40">
			<Button name="ItemBtn" text="{{title}}" @pressed="_on_task_action" />
		</Panel>
	</UIVList>
</ui>
""")
	assert_not_null(root, "解析不应失败")
	var target := _make_target()
	var list: GdUIVList = root if String(root.name) == "List" else root.find_child("List", true, false)
	assert_not_null(list, "列表应存在")
	if list == null:
		root.free()
		target.free()
		return
	assert_not_null(list.get_at(1), "构建期数据应创建 2 条条目")
	# connect_signals：记录列表 __signal_target + 连接构建期条目内按钮
	builder.connect_signals(root, target)
	var item_btn: Button = list.get_at(0).find_child("ItemBtn", true, false)
	assert_not_null(item_btn, "条目按钮应存在")
	if item_btn:
		item_btn.emit_signal("pressed")
	var calls: Array = target.get("calls")
	assert_eq(calls.size(), 1, "条目按钮 pressed 应自动连接 _on_task_action")
	if calls.size() > 0:
		assert_eq(calls[0], "task", "条目回调应被调用")
	var last = target.get("last_btn")
	assert_eq(last, item_btn, "1 参回调应自动补绑发出按钮引用")
	root.free()
	target.free()


func test_at_pressed_list_update_rebind() -> void:
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<script>
		var tasks = [
			{ title: "甲" },
			{ title: "乙" },
		]
	</script>
	<UIVList name="List" data="tasks">
		<Panel name="ItemRoot" custom_minimum_size="0,40">
			<Button name="ItemBtn" text="{{title}}" @pressed="_on_task_action" />
		</Panel>
	</UIVList>
</ui>
""")
	assert_not_null(root, "解析不应失败")
	var target := _make_target()
	var list: GdUIVList = root.find_child("List", true, false)
	if list == null:
		root.free()
		target.free()
		return
	builder.connect_signals(root, target)
	# 运行时更新数据：新条目由 duplicate 创建，应被 bind_events 自动重连
	var new_data: Array = [{ "title": "丙" }]
	list.update(new_data, false)
	var item_btn: Button = list.get_at(0).find_child("ItemBtn", true, false)
	assert_not_null(item_btn, "更新后的条目按钮应存在")
	if item_btn:
		item_btn.emit_signal("pressed")
	var calls: Array = target.get("calls")
	assert_eq(calls.size(), 1, "update 重建条目后应自动重连信号")
	if calls.size() > 0:
		assert_eq(calls[0], "task", "重建条目的回调应被调用")
	root.free()
	target.free()


func test_at_pressed_missing_method_no_crash() -> void:
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<Button name="Btn" @pressed="_on_not_exist" />
</ui>
""")
	assert_not_null(root, "解析不应失败")
	var target := _make_target()
	# 方法不存在：告警但不崩溃、不连接
	builder.connect_signals(root, target)
	var btn: Button = root.find_child("Btn", true, false)
	if btn:
		btn.emit_signal("pressed")
	var calls: Array = target.get("calls")
	assert_eq(calls.size(), 0, "缺失方法不应触发任何回调")
	root.free()
	target.free()


# 运行时构造条目脚本：挂载到条目根节点，记录回调来源
func _make_item_script(path: String) -> void:
	var src := """
extends Panel
var calls: Array = []
var last_btn: Control = null

func _on_item_click(btn: Control) -> void:
	calls.append("item")
	last_btn = btn
"""
	var fa := FileAccess.open(path, FileAccess.WRITE)
	fa.store_string(src)
	fa.close()


func test_ui_script_mount_near_bind() -> void:
	# 三层结构（与 task_item 示例一致）：item.gml 顶层 Panel 声明 <ui script="item.gd">
	# → 作为 <Gml> 注入 sub.gml 的 UIVList slot 模板 → 构建期数据驱动 duplicate 出条目。
	# 脚本随条目根节点挂载与复制，条目内 @pressed 就近绑定到条目脚本而非外部场景目标
	var dir := "user://gml_signal_mount"
	DirAccess.make_dir_recursive_absolute(dir)
	_make_item_script(dir + "/item.gd")
	var fa := FileAccess.open(dir + "/item.gml", FileAccess.WRITE)
	fa.store_string("""
<ui script="item.gd">
	<Panel name="ItemRoot" custom_minimum_size="0,40">
		<Button name="ItemBtn" text="{{title}}" @pressed="_on_item_click" />
	</Panel>
</ui>
""")
	fa.close()
	fa = FileAccess.open(dir + "/sub.gml", FileAccess.WRITE)
	fa.store_string("""
<ui>
	<script>
		var tasks = [ { title: "甲" }, { title: "乙" } ]
	</script>
	<UIVList name="List" data="tasks">
		<Gml src="item.gml" />
	</UIVList>
</ui>
""")
	fa.close()
	fa = FileAccess.open(dir + "/main.gml", FileAccess.WRITE)
	fa.store_string("""
<ui>
	<VBoxContainer>
		<Gml src="sub.gml" />
	</VBoxContainer>
</ui>
""")
	fa.close()

	var builder := GdUiBuilder.new()
	# parse_file 自动以 gml 所在目录为 <Gml>/<ui script> 相对路径基准
	var root: Control = builder.parse_file(dir + "/main.gml")
	assert_not_null(root, "解析不应失败")
	var list: GdUIVList = root.find_child("List", true, false)
	if list == null:
		root.free()
		return
	var item: Control = list.get_at(0)
	assert_not_null(item, "构建期条目应存在")
	if item:
		var script: Script = item.get_script()
		assert_not_null(script, "<ui script> 应自动挂载到条目根节点")
		if script:
			# 场景级目标不应收到条目回调
			var target := _make_target()
			builder.connect_signals(root, target)
			var btn: Button = item.find_child("ItemBtn", true, false)
			assert_not_null(btn, "条目按钮应存在")
			if btn:
				btn.emit_signal("pressed")
			var t_calls: Array = target.get("calls")
			assert_eq(t_calls.size(), 0, "条目 @pressed 应回调条目脚本而非场景目标")
			target.free()
			# 条目脚本实例收到回调
			assert_eq(item.get("calls").size(), 1, "条目脚本回调应被调用")
			assert_eq(item.get("last_btn"), btn, "条目脚本应拿到发出按钮引用")
	# update 重建条目：duplicate 自带脚本，bind_events 就近重连
	var new_data: Array = [{ "title": "丙" }]
	list.update(new_data, false)
	var item2: Control = list.get_at(0)
	assert_not_null(item2, "重建条目应存在")
	if item2:
		var script2: Script = item2.get_script()
		assert_not_null(script2, "duplicate 条目应继承挂载脚本")
		var btn2: Button = item2.find_child("ItemBtn", true, false)
		if btn2:
			btn2.emit_signal("pressed")
		# index 0 条目被复用（同一节点/脚本实例），calls 累计：此前 1 次 + 本次 1 次
		assert_eq(item2.get("calls").size(), 2, "重建条目的回调应连到条目自身脚本（复用累计）")
	root.free()
	_cleanup(dir)


func _cleanup(dir: String) -> void:
	var d := DirAccess.open(dir)
	if d:
		d.list_dir_begin()
		var f := d.get_next()
		while f != "":
			if not d.current_is_dir():
				d.remove(f)
			f = d.get_next()
		d.list_dir_end()
		DirAccess.remove_absolute(dir)
