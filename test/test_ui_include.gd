# suite: ui_include - <Gml> 标签引用另一个 gml 文件
# 覆盖: 相对路径解析、剥壳嫁接、属性覆盖、循环引用检测
# 注意: fixture 写到 user:// 临时目录，避免污染项目（res:// 下的 .gml 会被
#       编辑器插件自动生成 .gml.tscn）
extends "res://test/test_case.gd"

const FIXTURE_DIR := "user://gml_include_test"


func _setup_fixtures() -> void:
	DirAccess.make_dir_recursive_absolute(FIXTURE_DIR)
	_write("sub.gml", """
<ui>
	<Panel name="SubRoot">
		<Label name="sub_label" text="子视图"/>
	</Panel>
</ui>
""")
	_write("main.gml", """
<ui>
	<VBoxContainer>
		<Gml src="sub.gml" name="IncludedView"/>
	</VBoxContainer>
</ui>
""")
	_write("cycle_a.gml", """
<ui>
	<VBoxContainer>
		<Gml src="cycle_b.gml"/>
	</VBoxContainer>
</ui>
""")
	_write("cycle_b.gml", """
<ui>
	<VBoxContainer>
		<Label text="B"/>
		<Gml src="cycle_a.gml"/>
	</VBoxContainer>
</ui>
""")


func _write(file_name: String, content: String) -> void:
	var fa := FileAccess.open(FIXTURE_DIR + "/" + file_name, FileAccess.WRITE)
	fa.store_string(content)
	fa.close()


func _cleanup_fixtures() -> void:
	var dir := DirAccess.open(FIXTURE_DIR)
	if dir:
		dir.list_dir_begin()
		var f := dir.get_next()
		while f != "":
			if not dir.current_is_dir():
				dir.remove(f)
			f = dir.get_next()
		dir.list_dir_end()
		DirAccess.remove_absolute(FIXTURE_DIR)


func test_include_relative_path_graft() -> void:
	_setup_fixtures()
	var builder := GdUiBuilder.new()
	# parse_file 携带目录上下文，main.gml 中 src="sub.gml" 按相对路径解析
	var root: Control = builder.parse_file(FIXTURE_DIR + "/main.gml")
	assert_not_null(root, "解析结果不应为 null")
	var included := root.find_child("IncludedView", true, false)
	assert_not_null(included, "被引用子视图应嫁接到引用位置（含 name 覆盖）")
	if included:
		var label := included.find_child("sub_label", true, false)
		assert_not_null(label, "被引用文件的内部节点应完整保留")
		if label is Label:
			assert_eq((label as Label).text, "子视图", "Label 文本应来自被引用文件")
	root.free()
	_cleanup_fixtures()


func test_include_cycle_detected() -> void:
	_setup_fixtures()
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_file(FIXTURE_DIR + "/cycle_a.gml")
	assert_not_null(root, "解析结果不应为 null（失败时返回空根）")
	assert_eq(root.get_child_count(), 0, "循环引用应构建失败，根下无节点")
	assert_true(builder.last_error().contains("循环引用"), "错误信息应提示循环引用")
	root.free()
	_cleanup_fixtures()
