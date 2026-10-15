# suite: ui_fit_label - FitLabel 按文本长度自适应宽度
# 覆盖: fit_width 开/关、文本变化后 custom_minimum_size.x 跟随、
#       PanelContainer 容器随内容撑开（HUD 横幅溢出场景）、fit_padding 余量
# 运行: 由 test_runner.gd 自动发现（扫描 test_*.gd）
extends "res://test/test_case.gd"


func _write_fixture(dir: String) -> void:
	DirAccess.make_dir_recursive_absolute(dir)
	var fa := FileAccess.open(dir + "/main.gml", FileAccess.WRITE)
	fa.store_string("""
<ui>
  <style>
	.box {
	  background: #e8d9a8;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 8;
	  padding: 8 12 8 12;
	}
  </style>
  <HBoxContainer name="Banner" separation="4">
	<Panel class="roller" custom_minimum_size="12,48" />
	<PanelContainer name="Scroll" class="box" custom_minimum_size="0,48">
	  <HBoxContainer separation="8">
		<FitLabel name="Text" text="短文本" font_size="16" valign="center" fit_width="true" fit_padding="6" />
	  </HBoxContainer>
	</PanelContainer>
  </HBoxContainer>
</ui>
""")
	fa.close()
	# fit_width 未开（默认关）的对照样本
	fa = FileAccess.open(dir + "/plain.gml", FileAccess.WRITE)
	fa.store_string("""
<ui>
  <HBoxContainer name="Banner">
	<FitLabel name="Plain" text="短文本" font_size="16" />
  </HBoxContainer>
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


## fit_width 开启：文本变长 → 标签 min 宽跟随增长 → PanelContainer 撑开
func test_fit_width_tracks_text() -> void:
	var dir := "user://gml_fitlabel"
	_write_fixture(dir)
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_file(dir + "/main.gml")
	assert_not_null(root, "解析不应失败")
	if root == null:
		_cleanup(dir)
		return
	# _process 脏检查需要节点在树里
	var tree := Engine.get_main_loop() as SceneTree
	tree.root.add_child(root)
	await wait_frames(3)

	var label: Label = root.find_child("Text", true, false)
	var panel: PanelContainer = root.find_child("Scroll", true, false)
	if label == null or panel == null:
		assert_true(false, "FitLabel/PanelContainer 应存在")
		root.queue_free()
		await wait_frames(2)
		_cleanup(dir)
		return

	var short_w: float = label.custom_minimum_size.x
	assert_true(short_w > 0.0, "fit_width 开启时初始就应测量出文本宽（实际 %f）" % short_w)
	var short_panel_w: float = panel.size.x

	# 文本变长：min 宽应显著增长（等 _process 脏检查生效）
	label.text = "可接取：主线·初入仙途 · 找「执事长老」接取任务"
	await wait_frames(4)
	var long_w: float = label.custom_minimum_size.x
	assert_true(long_w > short_w * 2.0,
		"文本变长后 min 宽应跟随增长（短 %f → 长 %f）" % [short_w, long_w])
	assert_true(panel.size.x > short_panel_w,
		"PanelContainer 应随内容撑开（%f → %f）" % [short_panel_w, panel.size.x])
	# 文本变短：应收缩回接近初始宽（自适应是双向的）
	label.text = "短文本"
	await wait_frames(4)
	assert_true(label.custom_minimum_size.x <= short_w + 1.0,
		"文本变短后 min 宽应收缩回初始值（实际 %f）" % label.custom_minimum_size.x)
	# fit_padding 余量计入
	assert_true(label.custom_minimum_size.x >= 6.0, "fit_padding 应计入 min 宽")

	root.queue_free()
	await wait_frames(2)
	_cleanup(dir)


## fit_width 未开（默认）：行为等同普通 Label，min 宽不跟随
func test_fit_width_off_by_default() -> void:
	var dir := "user://gml_fitlabel_off"
	_write_fixture(dir)
	var builder := GdUiBuilder.new()
	var root: Control = builder.parse_file(dir + "/plain.gml")
	assert_not_null(root, "解析不应失败")
	if root == null:
		_cleanup(dir)
		return
	var tree := Engine.get_main_loop() as SceneTree
	tree.root.add_child(root)
	await wait_frames(3)
	var label: Label = root.find_child("Plain", true, false)
	if label:
		label.text = "一段长了很多很多很多很多很多的文本"
		await wait_frames(3)
		assert_eq(label.custom_minimum_size.x, 0.0,
			"fit_width 未开时 min 宽应保持 0（等同普通 Label）")
	root.queue_free()
	await wait_frames(2)
	_cleanup(dir)
