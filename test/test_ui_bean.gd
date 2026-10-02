# suite: ui_bean - data="bean:id:key" GdBean 响应式数据绑定
# 覆盖: bean 绑定声明（构建期不解析、__data_var meta）、GdGmlScene 加载后
#       auto_bind_data 初始填充、bean.emit 变化 → 列表自动刷新、
#       多份同名列表同时刷新、缺失 bean 容错
extends "res://test/test_case.gd"


# 测试用 Bean：items 数组为列表数据源
class TestBean:
	extends GdBean
	var items: Array = [
		{ title = "甲" },
		{ title = "乙" },
	]


# 运行时构造 GdGmlScene 子类脚本并实例化（模拟场景控制器）
func _make_scene() -> GdGmlScene:
	var src := """
extends GdGmlScene
"""
	var script := GDScript.new()
	script.source_code = src
	script.reload()
	var scene := GdGmlScene.new()
	scene.set_script(script)
	return scene


func test_bean_binding_reactive() -> void:
	var bean: GdBean = GdBean.bean("test_bean_react", func(): return TestBean.new())
	# GdBean 属性经 GDCORE 存档持久化（user://coredata.data），跨运行会恢复
	# 上次的数据，测试必须显式重置为初始值
	bean.set("items", [{ title = "甲" }, { title = "乙" }] as Array)
	var scene := _make_scene()
	scene.load_from_string("""
<ui>
	<UIVList name="TestList" data="bean:test_bean_react:items">
		<Panel><Label name="Title" text="{{title}}" /></Panel>
	</UIVList>
</ui>
""")
	# 初始填充：auto_bind_data 在加载流程中同步执行
	var list: Control = scene.find_node("TestList")
	assert_not_null(list, "绑定的列表应存在")
	if list:
		assert_not_null(list.get_at(1), "bean 初始数据应填充 2 条")
		assert_null(list.get_at(2), "初始数据只有 2 条")
	# watch 注册是 call_deferred，等一帧确保注册完成
	await wait_frames(2)
	# bean 属性变化 → watch 回调 → 列表自动刷新
	var items: Array = bean.get_value_by_key("items")
	items.append({ title = "丙" })
	bean.emit(["items"])
	await wait_frames(2)
	if list:
		assert_not_null(list.get_at(2), "bean 变化后列表应自动刷新出第 3 条")
		assert_null(list.get_at(3), "刷新后不应有多余条目")
		var label: Label = list.get_at(2).find_child("Title", true, false)
		if label:
			assert_eq(label.text, "丙", "新条目应展示变化后的数据")
	scene.free()


func test_bean_binding_multi_same_name_nodes() -> void:
	var bean: GdBean = GdBean.bean("test_bean_multi", func(): return TestBean.new())
	bean.set("items", [{ title = "甲" }, { title = "乙" }] as Array)  # 清除存档残留
	var scene := _make_scene()
	# 两份同名列表（模拟多页签各含一份 task_list 实例）
	# 两份实例分别放在不同父节点下（模拟多页签各含一份；Godot 不允许兄弟同名）
	scene.load_from_string("""
<ui>
	<VBoxContainer>
		<HBoxContainer>
			<UIVList name="TwinList" data="bean:test_bean_multi:items">
				<Panel><Label text="{{title}}" /></Panel>
			</UIVList>
		</HBoxContainer>
		<HBoxContainer>
			<UIVList name="TwinList" data="bean:test_bean_multi:items">
				<Panel><Label text="{{title}}" /></Panel>
			</UIVList>
		</HBoxContainer>
	</VBoxContainer>
</ui>
""")
	var found := scene.find_children("TwinList", "", true, false)
	assert_eq(found.size(), 2, "应存在两份同名列表")
	await wait_frames(2)
	var items: Array = bean.get_value_by_key("items")
	items.append({ title = "丙" })
	bean.emit(["items"])
	await wait_frames(2)
	# 同一 bean+key 只注册一次 watch，但回调应刷新全部同名节点
	for i in found.size():
		var list: Control = found[i]
		assert_not_null(list.get_at(2), "第 %d 份列表应同步刷新出第 3 条" % (i + 1))
	scene.free()


func test_bean_missing_no_crash() -> void:
	# bean 未注册：告警但不崩溃、不填充、不连接
	var scene := _make_scene()
	scene.load_from_string("""
<ui>
	<UIVList name="LonelyList" data="bean:no_such_bean:items">
		<Panel><Label text="{{title}}" /></Panel>
	</UIVList>
</ui>
""")
	var list: Control = scene.find_node("LonelyList")
	assert_not_null(list, "列表节点应存在（未绑定时为空列表）")
	if list:
		assert_null(list.get_at(0), "bean 缺失时不应有条目")
	scene.free()


func test_bean_attr_kept_as_meta() -> void:
	# data="bean:..." 构建期不解析为 <script> 变量（不报变量不存在错误），
	# 声明保留在 __data_var meta 供运行时绑定
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<UIVList name="List" data="bean:some_bean:items">
		<Panel><Label text="{{title}}" /></Panel>
	</UIVList>
</ui>
""")
	assert_not_null(root, "解析不应失败")
	var list: Control = root.find_child("List", true, false)
	if list:
		assert_true(list.has_meta("__data_var"), "bean 绑定声明应存为 __data_var meta")
		if list.has_meta("__data_var"):
			assert_eq(list.get_meta("__data_var"), "bean:some_bean:items", "meta 应保留完整声明")
		assert_null(list.get_at(0), "构建期不应有条目（bean 绑定是运行时行为）")
	root.free()
