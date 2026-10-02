# suite: ui_contract - 数据契约（@export 变量注入）
# 覆盖: 条目脚本 @export 同名注入、未声明 key 不注入（meta 兜底不受影响）、
#       update 刷新契约变量、缺失 key 保留当前值、契约回调直读变量、
#       __data_contract meta 记录
extends "res://test/test_case.gd"


# 运行时构造条目脚本：@export 契约 + 回调直读契约变量
func _make_item_script(path: String) -> void:
	var src := """
extends Panel

@export var title: String = ""
@export var count: int = 0
@export var btn_state: String = ""

var calls: Array = []

func _on_item_click(_btn: Control) -> void:
	calls.append("%s|%d|%s" % [title, count, btn_state])
"""
	var fa := FileAccess.open(path, FileAccess.WRITE)
	fa.store_string(src)
	fa.close()


func _write_fixture(dir: String) -> void:
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
	fa = FileAccess.open(dir + "/main.gml", FileAccess.WRITE)
	fa.store_string("""
<ui>
	<script>
		var tasks = [
			{ title: "甲", count: 3, btn_state: "go", ghost: "未声明字段" },
			{ title: "乙", count: 7, btn_state: "wait" },
		]
	</script>
	<UIVList name="List" data="tasks">
		<Gml src="item.gml" />
	</UIVList>
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


func test_contract_injection() -> void:
	var dir := "user://gml_contract"
	_write_fixture(dir)
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_file(dir + "/main.gml")
	assert_not_null(root, "解析不应失败")
	var list: GdUIVList = root.find_child("List", true, false)
	if list == null:
		root.free()
		_cleanup(dir)
		return
	var item: Control = list.get_at(0)
	assert_not_null(item, "条目应存在")
	if item:
		# 契约变量：同名字段自动注入
		assert_eq(item.get("title"), "甲", "@export title 应被注入")
		assert_eq(item.get("count"), 3, "@export int count 应被注入")
		assert_eq(item.get("btn_state"), "go", "@export btn_state 应被注入")
		# 未声明的 key 不注入脚本（保持 null），仅走 meta 兜底
		assert_null(item.get("ghost"), "未声明变量不应被注入")
		assert_eq(item.get_meta("ghost"), "未声明字段", "未声明 key 应保留 meta 兜底")
		# 契约 meta：挂载时记录 @export 变量列表
		var contract: PackedStringArray = item.get_meta("__data_contract")
		assert_not_null(contract, "条目根应记录 __data_contract meta")
		if contract != null:
			assert_true(contract.has("title") and contract.has("count") and contract.has("btn_state"),
				"契约 meta 应包含全部 @export 变量名")
			assert_false(contract.has("ghost"), "契约 meta 不应包含未声明字段")
	root.free()
	_cleanup(dir)


func test_contract_callback_reads_vars() -> void:
	var dir := "user://gml_contract_cb"
	_write_fixture(dir)
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_file(dir + "/main.gml")
	var list: GdUIVList = root.find_child("List", true, false)
	if list == null:
		root.free()
		_cleanup(dir)
		return
	builder.connect_signals(root, Node.new())
	var item: Control = list.get_at(0)
	var btn: Button = item.find_child("ItemBtn", true, false)
	assert_not_null(btn, "条目按钮应存在")
	if btn:
		btn.emit_signal("pressed")
		# 回调直接读契约变量，无需解析 __item_data meta
		var calls: Array = item.get("calls")
		assert_eq(calls.size(), 1, "条目回调应被调用")
		if calls.size() > 0:
			assert_eq(calls[0], "甲|3|go", "回调应直读注入后的契约变量")
		# 第二条目各自持有独立脚本实例与数据
		var item2: Control = list.get_at(1)
		if item2:
			assert_eq(item2.get("title"), "乙", "第二条目应注入各自数据")
			assert_eq(item2.get("calls").size(), 0, "条目间脚本实例应相互独立")
	root.free()
	_cleanup(dir)


func test_contract_update_refresh() -> void:
	var dir := "user://gml_contract_upd"
	_write_fixture(dir)
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_file(dir + "/main.gml")
	var list: GdUIVList = root.find_child("List", true, false)
	if list == null:
		root.free()
		_cleanup(dir)
		return
	# 运行时更新：契约变量随数据刷新，缺失 key 保留当前值
	var new_data: Array = [{ "title": "丙", "count": 9, "btn_state": "go" }, { "count": 1 }]
	list.update(new_data, false)
	var item: Control = list.get_at(0)
	assert_not_null(item, "更新后条目应存在")
	if item:
		assert_eq(item.get("title"), "丙", "update 后契约变量应刷新")
		assert_eq(item.get("count"), 9, "update 后 int 契约变量应刷新")
	# 第二条目缺 title/btn_state：契约变量保留当前值（不重置、不告警）
	var item2: Control = list.get_at(1)
	if item2:
		assert_eq(item2.get("count"), 1, "部分更新应注入存在的字段")
		assert_ne(item2.get("title"), "", "缺失契约字段应保留当前值而非清空")
	root.free()
	_cleanup(dir)


func test_contract_no_script_compat() -> void:
	# 无 <ui script> 的普通条目：数据仍走模板绑定，契约机制不干扰
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_string("""
<ui>
	<script>
		var tasks = [ { title: "甲" } ]
	</script>
	<UIVList name="List" data="tasks">
		<Panel name="ItemRoot" custom_minimum_size="0,40">
			<Button name="ItemBtn" text="{{title}}" />
		</Panel>
	</UIVList>
</ui>
""")
	assert_not_null(root, "解析不应失败")
	var list: GdUIVList = root.find_child("List", true, false)
	if list == null:
		root.free()
		return
	var item: Control = list.get_at(0)
	assert_not_null(item, "条目应存在")
	if item:
		var btn: Button = item.find_child("ItemBtn", true, false)
		assert_not_null(btn, "条目按钮应存在")
		if btn:
			assert_eq(btn.text, "甲", "无脚本条目模板绑定应照常工作")
	root.free()
