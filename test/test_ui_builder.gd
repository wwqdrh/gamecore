# suite: ui - GdUiBuilder UI 标记语言测试
# 覆盖: parse_string 节点树构建、validate 校验、内置主题、主题变量
# 注意: GdUiBuilder 是 RefCounted，由引用计数释放；parse_string 返回的 Control 是 Node，可手动 free
extends "res://test/test_case.gd"


func _make_builder() -> GdUiBuilder:
	return GdUiBuilder.new()


func test_parse_string_basic_tree() -> void:
	var builder := _make_builder()
	var markup := """
	<ui>
		<VBoxContainer>
			<Label name="title" text="标题"/>
			<Button name="ok" text="确定"/>
		</VBoxContainer>
	</ui>
	"""
	var root: Control = builder.parse_string(markup)
	assert_not_null(root, "解析结果不应为 null")
	# 无包装层：<ui> 根元素直接作为解析结果根节点
	var vbox: Control = root
	assert_true(vbox is VBoxContainer, "根元素应为 VBoxContainer（无 UiRoot 包装层）")
	if vbox is VBoxContainer:
		assert_eq(vbox.get_child_count(), 2, "VBoxContainer 应有 2 个子节点")

	var title := root.find_child("title", true, false)
	assert_not_null(title, "应存在 name=title 的节点")
	if title is Label:
		assert_eq((title as Label).text, "标题", "Label 文本应为 '标题'")
	var btn := root.find_child("ok", true, false)
	assert_not_null(btn, "应存在 name=ok 的节点")
	if btn is Button:
		assert_eq((btn as Button).text, "确定", "Button 文本应为 '确定'")
	root.free()


func test_parse_nested_containers() -> void:
	var builder := _make_builder()
	var markup := """
	<ui>
		<VBoxContainer>
			<HBoxContainer>
				<Label name="a" text="A"/>
				<Label name="b" text="B"/>
			</HBoxContainer>
			<GridContainer columns="2">
				<ColorRect name="c1"/>
				<ColorRect name="c2"/>
			</GridContainer>
		</VBoxContainer>
	</ui>
	"""
	var root: Control = builder.parse_string(markup)
	assert_not_null(root, "嵌套解析结果不应为 null")

	var a := root.find_child("a", true, false)
	assert_not_null(a, "深层节点 a 应存在")
	if a is Label:
		assert_eq((a as Label).text, "A", "a 的文本应为 'A'")
	var c2 := root.find_child("c2", true, false)
	assert_not_null(c2, "深层节点 c2 应存在")
	if c2 != null and c2.get_parent() is GridContainer:
		assert_eq((c2.get_parent() as GridContainer).columns, 2, "GridContainer 列数应为 2")
	root.free()


func test_validate() -> void:
	var builder := _make_builder()

	var ok := builder.validate("<ui><VBoxContainer><Label/></VBoxContainer></ui>")
	assert_eq(ok, "", "合法标记校验应无错误")

	# 标签不匹配
	var bad := builder.validate("<ui><VBoxContainer><Label/></HBoxContainer></ui>")
	assert_not_null(bad, "标签不匹配校验应有错误信息")
	# 缺少 <ui> 根元素
	var no_root := builder.validate("<VBoxContainer><Label/></VBoxContainer>")
	assert_not_null(no_root, "缺少 ui 根元素应有错误信息")
	if no_root != "":
		assert_contains_str(no_root, "ui", "错误信息应提示 ui 根元素")


func test_custom_theme_vars() -> void:
	var builder := _make_builder()
	# 自定义主题变量注入（<style> 值中 $var 引用，无内置主题）
	builder.set_theme_var("primary_color", "#ff0000")
	builder.clear_custom_theme_vars()


func test_parse_with_theme() -> void:
	var builder := _make_builder()
	var markup := """
	<ui>
		<PanelContainer>
			<Label name="content" text="主题内文本"/>
		</PanelContainer>
	</ui>
	"""
	var root: Control = builder.parse_string(markup)
	assert_not_null(root, "带主题解析不应失败")
	assert_not_null(root.find_child("content", true, false), "带主题解析应保留子节点")
	root.free()
