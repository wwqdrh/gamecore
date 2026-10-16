# suite: gml_deps - GML 引用依赖图（编辑器自动重建的纯逻辑部分）
# 覆盖: <Gml src> 引用提取（相对/同目录/绝对路径）、反向索引、
#       传递展开（多层级引用者 + 环安全）、闭包 mtime 基准
extends "res://test/test_case.gd"

const GmlDepsRes := preload("res://addons/gamecore/gml_deps.gd")


func _write(path: String, content: String) -> void:
	var dir := path.get_base_dir()
	DirAccess.make_dir_recursive_absolute(dir)
	var fa := FileAccess.open(path, FileAccess.WRITE)
	fa.store_string(content)
	fa.close()


func _write_fixture(dir: String) -> void:
	# 三层链：main -> sub/b -> c；main 还直接引用同目录 panel
	_write(dir + "/a/main.gml", """
<ui>
  <!-- 注释里也出现一次 <Gml src="panel.gml">（应去重） -->
  <Gml src="sub/b.gml" data-grp="grp_active" />
  <Gml src="panel.gml" />
</ui>
""")
	_write(dir + "/a/sub/b.gml", """
<ui>
  <Gml src="c.gml" />
</ui>
""")
	_write(dir + "/a/sub/c.gml", """
<ui>
  <Label text="leaf" />
</ui>
""")
	_write(dir + "/a/panel.gml", """
<ui>
  <Panel />
</ui>
""")
	# 环引用（构建期会报错，但依赖图展开必须安全终止）
	_write(dir + "/cyc/x.gml", """
<ui>
  <Gml src="y.gml" />
</ui>
""")
	_write(dir + "/cyc/y.gml", """
<ui>
  <Gml src="x.gml" />
</ui>
""")


func _cleanup(dir: String) -> void:
	var d := DirAccess.open(dir)
	if d:
		d.list_dir_begin()
		var f := d.get_next()
		while f != "":
			if d.current_is_dir():
				var sub := dir.path_join(f)
				_cleanup(sub)
				DirAccess.remove_absolute(sub)
			else:
				d.remove(dir.path_join(f))
			f = d.get_next()
		d.list_dir_end()
		DirAccess.remove_absolute(dir)


func test_extract_includes_resolves_paths() -> void:
	var dir := "user://gml_deps"
	_write_fixture(dir)
	var inc := GmlDepsRes.extract_includes(dir + "/a/main.gml")
	assert_eq(inc.size(), 2, "main 应引用 2 个子文件（注释内重复已去重），实际 %d" % inc.size())
	assert_true(dir + "/a/sub/b.gml" in inc, "相对 src 应按父文件目录解析")
	assert_true(dir + "/a/panel.gml" in inc, "同目录 src 应解析")
	var inc_b := GmlDepsRes.extract_includes(dir + "/a/sub/b.gml")
	assert_eq(inc_b.size(), 1, "b 应引用 1 个子文件")
	assert_true(dir + "/a/sub/c.gml" in inc_b, "b 的同目录引用应相对 b 解析")
	_cleanup(dir)


func test_expand_dependents_multilevel() -> void:
	var dir := "user://gml_deps2"
	_write_fixture(dir)
	var files := PackedStringArray([
		dir + "/a/main.gml", dir + "/a/sub/b.gml", dir + "/a/sub/c.gml",
		dir + "/a/panel.gml", dir + "/cyc/x.gml", dir + "/cyc/y.gml",
	])
	var graph: Array = GmlDepsRes.build_graph(files)
	var fwd: Dictionary = graph[0]
	var rev: Dictionary = graph[1]
	# 改叶子 c：main（两级上游）+ b（一级上游）都应重建
	var changed: Array[String] = [dir + "/a/sub/c.gml"]
	var out := GmlDepsRes.expand_dependents(changed, rev)
	assert_eq(out.size(), 3, "改 c 应展开 c+b+main 三文件，实际 %d" % out.size())
	assert_true(dir + "/a/sub/b.gml" in out, "一级上游 b 应入闭包")
	assert_true(dir + "/a/main.gml" in out, "两级上游 main 应入闭包（多层级传递）")
	# 改 panel：只有 main 引用它
	var out2 := GmlDepsRes.expand_dependents([dir + "/a/panel.gml"] as Array[String], rev)
	assert_eq(out2.size(), 2, "改 panel 应展开 panel+main，实际 %d" % out2.size())
	# 环安全：改 x 应终止并覆盖 x+y
	var out3 := GmlDepsRes.expand_dependents([dir + "/cyc/x.gml"] as Array[String], rev)
	assert_eq(out3.size(), 2, "环引用展开应安全终止（x+y），实际 %d" % out3.size())
	# fwd 图用于闭包 mtime：c 的闭包最大 mtime >= 自身 mtime
	var mt := GmlDepsRes.closure_max_mtime(dir + "/a/sub/c.gml", fwd)
	assert_true(mt >= FileAccess.get_modified_time(dir + "/a/sub/c.gml"),
		"闭包最大 mtime 应不小于自身")
	_cleanup(dir)
