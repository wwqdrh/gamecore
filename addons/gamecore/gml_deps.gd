# GML 引用依赖图 —— 编辑器插件（core.gd）的纯逻辑部分
#
# 职责：扫描 .gml 源码中的 <Gml src="..."> 引用，构建反向依赖索引，
#       供"改了某个 gml → 所有（含多层级）引用它的父级一起重建"使用。
#
# 路径解析规则与 rust 侧 builder.rs resolve_include_path 完全一致：
#   res:// 或 user:// 或 / 开头 = 原样使用；否则相对父文件所在目录拼接。
#
# 均为静态函数，无编辑器依赖——headless 测试可直接调用
# （test/test_gml_deps.gd）。
class_name GmlDeps
extends RefCounted

## 引用正则：<Gml 标签内的 src="..."（跨行属性也命中；
## 注释里的示例引用会多算一条边，无害——只是多重建一次）
const INCLUDE_RE := "<Gml\\s[^>]*?src\\s*=\\s*\"([^\"]+)\""


## 提取一个 gml 文件引用的全部子 gml（解析为 res:// 绝对路径，去重）
static func extract_includes(gml_path: String) -> PackedStringArray:
	var out := PackedStringArray()
	var fa := FileAccess.open(gml_path, FileAccess.READ)
	if fa == null:
		return out
	var content := fa.get_as_text()
	fa.close()
	var re := RegEx.create_from_string(INCLUDE_RE)
	var seen := {}
	for m in re.search_all(content):
		var src := m.get_string(1)
		var resolved := resolve_src(src, gml_path)
		if resolved != "" and not seen.has(resolved):
			seen[resolved] = true
			out.append(resolved)
	return out


## src 属性 → 绝对资源路径（与 rust resolve_include_path 同语义）
static func resolve_src(src: String, parent_path: String) -> String:
	if src.begins_with("res://") or src.begins_with("user://") or src.begins_with("/"):
		return src
	var dir := parent_path.get_base_dir()
	# 根目录文件的 base_dir 是 "res://"（builder 侧 trim 后视为 res: 根）
	if dir == "res://" or dir == "user://" or dir.is_empty() or dir == "res:" or dir == "user:":
		return "res://" + src
	return dir.path_join(src)


## 构建两张索引：
##   fwd: { 父: [子...] }（新鲜度比较用：tscn 需不旧于整条依赖链）
##   rev: { 子: [父...] }（传递重建用：改了子 → 找出所有上游引用者）
static func build_graph(files: PackedStringArray) -> Array:
	var fwd := {}
	var rev := {}
	for parent in files:
		var kids := extract_includes(parent)
		fwd[parent] = kids
		for child in kids:
			if not rev.has(child):
				rev[child] = PackedStringArray()
			if not (parent in rev[child]):
				rev[child].append(parent)
	return [fwd, rev]


## 传递展开：changed 里的每个文件，找出所有直接/间接引用它的父级
## （含 changed 自身；环安全——重复引用天然被 visited 挡住）
static func expand_dependents(changed: Array[String], rev: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var visited := {}
	var stack: Array[String] = []
	for p in changed:
		if not visited.has(p):
			visited[p] = true
			stack.append(p)
	while stack.size() > 0:
		var cur: String = stack.pop_back()
		out.append(cur)
		var parents = rev.get(cur, [])
		for parent in parents:
			if not visited.has(parent):
				visited[parent] = true
				stack.append(parent)
	return out


## path 及其传递依赖闭包的最大 mtime（新鲜度基准：
## tscn 只要不旧于这条链上最年轻的 gml 就不用重建）
static func closure_max_mtime(path: String, fwd: Dictionary) -> int:
	var visited := {}
	var stack: Array[String] = [path]
	visited[path] = true
	var newest := 0
	while stack.size() > 0:
		var cur: String = stack.pop_back()
		if FileAccess.file_exists(cur):
			var t := FileAccess.get_modified_time(cur)
			if t > newest:
				newest = t
		var kids = fwd.get(cur, [])
		for child in kids:
			if not visited.has(child):
				visited[child] = true
				stack.append(child)
	return newest
