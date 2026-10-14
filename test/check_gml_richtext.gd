# GdUIRichText 富文本组件验收 —— gdoc 解析/BBCode 转换/图片直通/文件加载
#
# 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . \
#   -s res://test/check_gml_richtext.gd
#
# 覆盖：
#   1. set_doc_text：Markdown 子集 → BBCode（标题/列表/粗斜体/引用/分隔线）
#   2. [img] 单行直通 + ```bbcode 围栏直通（[table]）
#   3. load_doc：真实 .gdoc 文件加载渲染
#   4. gml src 属性 → ready 自动加载（builder 集成路径）
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[RichText] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _make_rt(gml_extra: String = "") -> Control:
	var gml := """
<ui>
  <Control name="Wrap">
	<RichText name="DocView" font_size="15" %s />
  </Control>
</ui>
""" % gml_extra
	var builder := GdUiBuilder.new()
	return builder.parse_string(gml)


func _run() -> void:
	# ---- 1. set_doc_text：格式转换 ----
	var wrap := _make_rt()
	check(wrap != null, "gml 应解析成功")
	root.add_child(wrap)
	await process_frame
	var rt: Control = wrap.find_child("DocView", true, false)
	check(rt != null, "RichText 应存在")
	check(rt != null and rt.get_class() == "GdUIRichText", "应为 GdUIRichText 实例")

	var doc := "# 一级标题\n\n## 二级标题\n\n**粗体** 与 *斜体* 与 `代码`\n\n- 项目甲\n- 项目乙\n\n| 表头一 | 表头二 |\n| --- | --- |\n| 甲一 | 乙一 |\n\n> 引用一句\n\n---\n\n正文段落。"
	rt.call("set_doc_text", doc)
	var bb := String(rt.call("get_bbcode"))
	check(bb.contains("[font_size=23]一级标题"), "h1 应转字号 15+8（%s）" % bb.substr(0, 80))
	check(bb.contains("[font_size=20]二级标题"), "h2 应转字号 15+5")
	check(bb.contains("[b]粗体[/b]"), "粗体应转 [b]")
	check(bb.contains("[i]斜体[/i]"), "斜体应转 [i]")
	check(bb.contains("[code]代码[/code]"), "代码应转 [code]")
	check(bb.contains("[indent]• 项目甲[/indent]"), "列表应转 bullet+indent（引擎无 [li]）")
	check(not bb.contains("[li]"), "不得残留 [li] 字面标签")
	check(bb.contains("[table=2]"), "markdown 表格应转 [table=2]")
	check(bb.contains("[cell][b]表头一[/b][/cell]"), "表格首行应加粗作表头")
	check(bb.contains("[cell]乙一[/cell]"), "表格数据行应转 cell")
	check(not bb.contains("| --- |"), "表格分隔行应被跳过")
	check(bb.contains("[color=#8a7a5a]引用一句"), "引用应有强调色")
	check(bb.contains("[hr]"), "分隔线应转 [hr]")

	# ---- 2. 图片直通 + 围栏直通 ----
	var doc2 := "[img width=300]res://a.png[/img]\n\n```bbcode\n[table=2][cell]甲[/cell][cell]乙[/cell][/table]\n```"
	rt.call("set_doc_text", doc2)
	var bb2 := String(rt.call("get_bbcode"))
	check(bb2.contains("[img width=300]res://a.png[/img]"), "img 行应原样透传")
	check(bb2.contains("[table=2]"), "围栏内 BBCode 应原样透传")

	# ---- 3. load_doc：真实文件 ----
	var real_doc := "res://example/demo/xiuxian/scenes/main/ui/docsbook/docs/basics.gdoc"
	var ok_load: bool = rt.call("load_doc", real_doc)
	check(ok_load, "load_doc 应成功打开手册文件")
	var bb3 := String(rt.call("get_bbcode"))
	check(bb3.contains("修行基础知识"), "手册标题应渲染进 BBCode")
	check(bb3.contains("[indent]• "), "手册列表应转 bullet")
	check(not bb3.contains("[li]"), "手册不得残留 [li]")
	var ok_bad: bool = rt.call("load_doc", "res://nonexist/none.gdoc")
	check(not ok_bad, "不存在的文件应返回 false")

	# ---- 4. gml src 属性 → ready 自动加载 ----
	var wrap2 := _make_rt("src=\"%s\"" % real_doc)
	root.add_child(wrap2)
	await process_frame
	var rt2: Control = wrap2.find_child("DocView", true, false)
	var bb4 := String(rt2.call("get_bbcode"))
	check(bb4.contains("修行基础知识"), "src 属性应在 ready 自动加载并渲染")

	print("[RichText] 全部通过" if ok else "[RichText] 存在失败项")
	quit(0 if ok else 2)
