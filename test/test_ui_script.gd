# suite: ui_script - <script> 数据块与 data="变量名" 绑定
# 覆盖: script 变量定义、UIVList 构建期数据驱动、meta 暴露、
#       <Gml> 引用 data 覆盖、缺失变量容错
extends "res://test/test_case.gd"


func test_script_array_binds_vlist() -> void:
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<script>
		var tasks = [
			{ title: "任务A", progress: "1/2" },
			{ title: "任务B", progress: "2/2" },
		]
	</script>
	<UIVList name="List" data="tasks">
		<Panel name="ItemRoot" custom_minimum_size="0,40">
			<Label name="ItemTitle" text="{{title}}" />
		</Panel>
	</UIVList>
</ui>
""")
	assert_not_null(root, "解析不应失败")
	var list := root.find_child("List", true, false)
	assert_not_null(list, "UIVList 应存在")
	if list:
		var item0 = list.get_at(0)
		var item1 = list.get_at(1)
		var item2 = list.get_at(2)
		assert_not_null(item0, "第 1 条应由 script 数据构建期创建")
		assert_not_null(item1, "第 2 条应由 script 数据构建期创建")
		assert_null(item2, "数据只有 2 条，不应有多余条目")
		if item0:
			assert_true(item0.has_meta("__item_data"), "条目应携带 __item_data meta")
			var title: Label = item0.find_child("ItemTitle", true, false)
			assert_not_null(title, "条目内模板 Label 应存在")
			if title:
				assert_eq(title.text, "任务A", "{{title}} 模板应由 script 数据填充")
	root.free()


func test_script_vars_meta_and_dict_data() -> void:
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<script>
		var panel_info = { title: "面板标题", level: 3, open: true }
		var version = "1.0"
	</script>
	<Panel name="InfoPanel" data="panel_info">
		<Label text="x" />
	</Panel>
</ui>
""")
	assert_not_null(root, "解析不应失败")
	# 根节点 meta：所有 script 变量
	assert_true(root.has_meta("__script_vars"), "根节点应挂 __script_vars meta")
	var vars: Dictionary = root.get_meta("__script_vars")
	assert_eq(vars.get("version", ""), "1.0", "字符串变量应可从根 meta 读取")
	var info: Dictionary = vars.get("panel_info", {})
	assert_eq(info.get("title", ""), "面板标题", "对象变量应递归转换为 Dictionary")
	assert_eq(info.get("level", 0), 3, "数字变量类型应保留")
	assert_eq(info.get("open", false), true, "布尔变量类型应保留")
	# data 属性节点：变量存为 __script_data meta
	var panel := root.find_child("InfoPanel", true, false)
	assert_not_null(panel, "InfoPanel 应存在")
	if panel:
		assert_true(panel.has_meta("__script_data"), "data 节点应挂 __script_data meta")
		var d: Dictionary = panel.get_meta("__script_data")
		assert_eq(d.get("title", ""), "面板标题", "__script_data 应为完整变量值")
	root.free()


func test_script_missing_var_no_crash() -> void:
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<UIVList name="List" data="not_defined">
		<Label text="tpl" />
	</UIVList>
</ui>
""")
	assert_not_null(root, "变量缺失不应导致构建失败")
	var list := root.find_child("List", true, false)
	if list:
		assert_null(list.get_at(0), "无数据时不应有条目")
	root.free()


func test_script_gml_data_override() -> void:
	var dir := "user://gml_script_test"
	DirAccess.make_dir_recursive_absolute(dir)
	_write(dir, "sub.gml", """
<ui>
	<script>
		var tasks = [ { title: "默认数据" } ]
		var header = "子文件默认标题"
	</script>
	<ScrollContainer name="SubRoot">
		<Label name="SubHeader" data="header" />
		<UIVList name="SubList" data="tasks">
			<Panel name="ItemRoot"><Label name="ItemTitle" text="{{title}}" /></Panel>
		</UIVList>
	</ScrollContainer>
</ui>
""")
	_write(dir, "main.gml", """
<ui>
	<script>
		var main_tasks = [
			{ title: "主线一" },
			{ title: "主线二" },
		]
		var main_header = "主线任务"
	</script>
	<VBoxContainer>
		<Gml src="sub.gml" data-tasks="main_tasks" data-header="main_header" />
	</VBoxContainer>
</ui>
""")
	var builder := GdUiBuilder.new()
	builder.set_base_dir(dir)
	var root: Control = builder.parse_string(_read(dir, "main.gml"))
	assert_not_null(root, "解析不应失败")
	var sub_list := root.find_child("SubList", true, false)
	assert_not_null(sub_list, "被引用文件的 UIVList 应存在")
	if sub_list:
		assert_not_null(sub_list.get_at(1), "data-tasks 覆盖应生效（2 条）")
		assert_null(sub_list.get_at(2), "覆盖数据只有 2 条")
		if sub_list.get_at(0):
			var title: Label = sub_list.get_at(0).find_child("ItemTitle", true, false)
			if title:
				assert_eq(title.text, "主线一", "条目应展示引用方覆盖的数据而非子文件默认值")
	# data-header 具名覆盖：子文件内 data="header" 节点取到的应是引用方变量值
	var header := root.find_child("SubHeader", true, false)
	assert_not_null(header, "被引用文件的 Label 应存在")
	if header:
		assert_eq(header.get_meta("__script_data", ""), "主线任务",
			"data-header 应覆盖子文件 header 变量，子文件节点按名引用")
	root.free()
	_cleanup_dir(dir)


func test_script_gml_named_data_default_kept() -> void:
	# 不带 data-* 属性的 <Gml> 引用：子文件 <script> 默认数据原样生效
	var dir := "user://gml_script_test2"
	DirAccess.make_dir_recursive_absolute(dir)
	_write(dir, "sub.gml", """
<ui>
	<script>
		var tasks = [ { title: "默认任务" } ]
	</script>
	<UIVList name="SubList" data="tasks">
		<Panel name="ItemRoot"><Label name="ItemTitle" text="{{title}}" /></Panel>
	</UIVList>
</ui>
""")
	_write(dir, "main.gml", """
<ui>
	<VBoxContainer>
		<Gml src="sub.gml" />
	</VBoxContainer>
</ui>
""")
	var builder := GdUiBuilder.new()
	builder.set_base_dir(dir)
	var root: Control = builder.parse_string(_read(dir, "main.gml"))
	assert_not_null(root, "解析不应失败")
	var sub_list := root.find_child("SubList", true, false)
	if sub_list:
		if sub_list.get_at(0):
			var title: Label = sub_list.get_at(0).find_child("ItemTitle", true, false)
			if title:
				assert_eq(title.text, "默认任务", "未覆盖时应使用子文件默认数据")
	root.free()
	_cleanup_dir(dir)


func test_script_gml_named_data_missing_parent_var_no_crash() -> void:
	# data-xxx 引用父文件不存在的变量：报错但不崩溃，子文件默认数据仍生效
	var dir := "user://gml_script_test3"
	DirAccess.make_dir_recursive_absolute(dir)
	_write(dir, "sub.gml", """
<ui>
	<script>
		var tasks = [ { title: "兜底默认" } ]
	</script>
	<UIVList name="SubList" data="tasks">
		<Panel name="ItemRoot"><Label name="ItemTitle" text="{{title}}" /></Panel>
	</UIVList>
</ui>
""")
	_write(dir, "main.gml", """
<ui>
	<VBoxContainer>
		<Gml src="sub.gml" data-tasks="not_defined" />
	</VBoxContainer>
</ui>
""")
	var builder := GdUiBuilder.new()
	builder.set_base_dir(dir)
	var root: Control = builder.parse_string(_read(dir, "main.gml"))
	assert_not_null(root, "引用缺失父变量不应导致构建失败")
	var sub_list := root.find_child("SubList", true, false)
	if sub_list:
		if sub_list.get_at(0):
			var title: Label = sub_list.get_at(0).find_child("ItemTitle", true, false)
			if title:
				assert_eq(title.text, "兜底默认", "覆盖失败时应保留子文件默认数据")
	root.free()
	_cleanup_dir(dir)


func _write(dir: String, file_name: String, content: String) -> void:
	var fa := FileAccess.open(dir + "/" + file_name, FileAccess.WRITE)
	fa.store_string(content)
	fa.close()


func _read(dir: String, file_name: String) -> String:
	var fa := FileAccess.open(dir + "/" + file_name, FileAccess.READ)
	return fa.get_as_text() if fa else ""


func _cleanup_dir(dir: String) -> void:
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
