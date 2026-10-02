# suite: ui_filter - 列表分类过滤（filter_key + filter_value_var）
# 覆盖: 静态 <script> 变量过滤、<Gml data-category> 具名映射注入过滤值、
#       全量数据 update 后过滤保持、空过滤值显示全部、无 filter 声明兼容
extends "res://test/test_case.gd"


# 运行时构造 fixture：父文件定义分类变量 + data-category 注入子列表文件
func _write_fixture(dir: String) -> void:
	DirAccess.make_dir_recursive_absolute(dir)
	# 子文件：列表组件（filter_value_var 引用本文件 category 变量）
	var fa := FileAccess.open(dir + "/item_list.gml", FileAccess.WRITE)
	fa.store_string("""
<ui theme="cartoon">
	<script>
		var category = ""
		var tasks = [
			{ category: "daily", title: "日常甲" },
			{ category: "daily", title: "日常乙" },
			{ category: "main", title: "主线甲" },
		]
	</script>
	<ScrollContainer name="Scroll" size_flags_vertical="expand_fill">
		<UIVList name="TaskList" size_flags_horizontal="expand_fill"
				 data="tasks" filter_key="category" filter_value_var="category">
			<Panel name="ItemRoot" custom_minimum_size="0,40">
				<Label name="Title" text="{{title}}" />
			</Panel>
		</UIVList>
	</ScrollContainer>
</ui>
""")
	fa.close()
	# 父文件：两个页签各引用一份，注入不同分类
	fa = FileAccess.open(dir + "/main.gml", FileAccess.WRITE)
	fa.store_string("""
<ui theme="cartoon">
	<script>
		var cat_daily = "daily"
		var cat_main = "main"
	</script>
	<VBoxContainer>
		<HBoxContainer>
			<Gml src="item_list.gml" data-category="cat_daily" />
		</HBoxContainer>
		<HBoxContainer>
			<Gml src="item_list.gml" data-category="cat_main" />
		</HBoxContainer>
	</VBoxContainer>
</ui>
""")
	fa.close()


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


func test_filter_var_injection() -> void:
	var dir := "user://gml_filter"
	_write_fixture(dir)
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_file(dir + "/main.gml")
	assert_not_null(root, "解析不应失败")
	# 两个实例分属不同父节点（Godot 不允许兄弟同名）
	var lists := root.find_children("TaskList", "", true, false)
	assert_eq(lists.size(), 2, "应存在两份列表实例")
	if lists.size() == 2:
		var daily: Control = lists[0]
		var main: Control = lists[1]
		# 过滤 meta 已注入（具名映射变量值）
		assert_eq(daily.get_meta("__filter_key"), "category", "__filter_key 应为数据字段名")
		assert_eq(daily.get_meta("__filter_value"), "daily", "第一实例过滤值应为注入的 daily")
		assert_eq(main.get_meta("__filter_value"), "main", "第二实例过滤值应为注入的 main")
		# 构建期静态数据按分类过滤：daily 2 条 / main 1 条
		assert_not_null(daily.get_at(1), "daily 列表应有 2 条匹配条目")
		assert_null(daily.get_at(2), "daily 列表不应有第 3 条")
		assert_not_null(main.get_at(0), "main 列表应有 1 条匹配条目")
		assert_null(main.get_at(1), "main 列表不应有第 2 条")
		var label: Label = main.get_at(0).find_child("Title", true, false)
		if label:
			assert_eq(label.text, "主线甲", "main 列表应显示 main 分类条目")
	root.free()
	_cleanup(dir)


func test_filter_kept_on_update() -> void:
	var dir := "user://gml_filter_upd"
	_write_fixture(dir)
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_file(dir + "/main.gml")
	var lists := root.find_children("TaskList", "", true, false)
	if lists.size() < 2:
		root.free()
		_cleanup(dir)
		return
	var daily: Control = lists[0]
	# 运行时推全量数据（模拟 bean watch 回调）：过滤声明应持续生效
	var full: Array = [
		{ "category": "daily", "title": "日常丙" },
		{ "category": "main", "title": "主线乙" },
		{ "category": "bounty", "title": "悬赏甲" },
		{ "category": "daily", "title": "日常丁" },
	]
	daily.update(full, false)
	assert_not_null(daily.get_at(1), "全量数据应过滤出 2 条 daily 条目")
	assert_null(daily.get_at(2), "过滤后不应有第 3 条")
	var label: Label = daily.get_at(0).find_child("Title", true, false)
	if label:
		assert_eq(label.text, "日常丙", "过滤条目应展示匹配数据")
	root.free()
	_cleanup(dir)


func test_filter_empty_shows_all() -> void:
	var dir := "user://gml_filter_empty"
	_write_fixture(dir)
	var builder := GdUiBuilder.new()
	# 独立打开列表组件：category 默认空 = 不过滤，显示全部 3 条
	var root: Control = builder.parse_file(dir + "/item_list.gml")
	assert_not_null(root, "解析不应失败")
	var list: Control = root.find_child("TaskList", true, false)
	if list == null:
		root.free()
		_cleanup(dir)
		return
	assert_not_null(list.get_at(2), "空过滤值应显示全部 3 条")
	assert_null(list.get_at(3), "不应有第 4 条")
	root.free()
	_cleanup(dir)


func test_filter_no_decl_compat() -> void:
	# 无 filter 声明的列表：数据照常驱动，不受过滤机制影响
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<script>
		var tasks = [
			{ category: "daily", title: "甲" },
			{ category: "main", title: "乙" },
		]
	</script>
	<UIVList name="List" data="tasks">
		<Panel name="ItemRoot" custom_minimum_size="0,40">
			<Label text="{{title}}" />
		</Panel>
	</UIVList>
</ui>
""")
	assert_not_null(root, "解析不应失败")
	var list: Control = root.find_child("List", true, false)
	if list == null:
		root.free()
		return
	assert_not_null(list.get_at(1), "无过滤声明应显示全部条目")
	root.free()
