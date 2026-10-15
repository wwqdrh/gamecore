# 无头重建 .gjson 加密产物 —— 静态定义表管线工具（命令行版）
#
# 用途：与编辑器插件（addons/gamecore/core.gd 的 _poll_json_files）等价的
#       命令行工具。源 .json 新增/修改后，不必打开编辑器即可重建加密产物：
#
#   godot --headless --path . -s res://test/regen_gjson.gd            # 全量扫描
#   godot --headless --path . -s res://test/regen_gjson.gd -- state/task/task.json
#                                                      （-- 后跟待转源文件，可多个）
#
# 规则：res:// 下（跳过 .godot / target / bin）**带管线标记**的明文 .json
#   —— 顶层字段 "pipeline": "gjson"（opt-in，通用配置如 game_config.json
#   不进管线）→ JSON 校验 → GdJsonCodec.encrypt_text（rust 侧算法，密钥
#   不出 GDScript）→ 同名 .gjson。
#
# 新鲜判定（v2）：不再依赖 mtime（编辑器/命令行混用时不可靠），改为
#   **内容比对**——decrypt_to_text(现存 gjson) == 源文本则视为最新跳过；
#   产物缺失或内容不一致（含源损坏导致的旧产物）即重建。重建后回读解密
#   校验，密文与源不一致视为失败（退出码 1）。
extends SceneTree

const SKIP_DIRS := [".godot", "target", "bin"]
## 管线标记字段（与编辑器插件一致）
const PIPELINE_KEY := "pipeline"
const PIPELINE_VALUE := "gjson"


func _init() -> void:
	var args := OS.get_cmdline_user_args()  # `--` 之后的参数
	var targets := PackedStringArray()
	for a in args:
		if a.begins_with("res://"):
			targets.append(a)
		elif not a.begins_with("-"):
			# 相对路径按项目根解析
			targets.append("res://" + a.lstrip("/"))
	var sources := PackedStringArray()
	if targets.is_empty():
		_collect("res://", sources)
	else:
		for t in targets:
			if FileAccess.file_exists(t):
				sources.append(t)
			else:
				printerr("[RegenGjson] 指定文件不存在: %s" % t)
				quit(1)
				return
	var generated := 0
	var fresh := 0
	var failed := false
	for src in sources:
		match _process_file(src):
			"generated": generated += 1
			"fresh": fresh += 1
			"fail": failed = true
	print("[RegenGjson] done: %d generated, %d fresh, %d scanned"
			% [generated, fresh, sources.size()])
	quit(1 if failed else 0)


func _collect(dir_path: String, out: PackedStringArray) -> void:
	var dir := DirAccess.open(dir_path)
	if dir == null:
		return
	dir.list_dir_begin()
	var name := dir.get_next()
	while name != "":
		if dir.current_is_dir():
			if name != "." and name != ".." and not name in SKIP_DIRS:
				_collect(dir_path.path_join(name), out)
		elif name.get_extension() == "json":
			out.append(dir_path.path_join(name))
		name = dir.get_next()
	dir.list_dir_end()


## 处理单个源文件，返回 "generated" / "fresh" / "fail"
func _process_file(src: String) -> String:
	var text := FileAccess.get_file_as_string(src)
	var parsed: Variant = JSON.parse_string(text)
	if parsed == null:
		printerr("[RegenGjson] %s JSON 解析失败，跳过" % src)
		return "fail"
	# opt-in：只有声明了管线标记的定义表源文件才生成加密产物
	var is_pipeline := parsed is Dictionary \
			and str((parsed as Dictionary).get(PIPELINE_KEY, "")) == PIPELINE_VALUE
	if not is_pipeline:
		return "fresh"  # 非管线文件不计入统计口径外的失败
	var dst := src.get_basename() + ".gjson"
	if FileAccess.file_exists(dst):
		var f := FileAccess.open(dst, FileAccess.READ)
		var bytes := f.get_buffer(f.get_length())
		f.close()
		# 内容比对：现存产物解密后与源文本一致 → 最新（编辑器已转过的也算）
		if GdJsonCodec.decrypt_to_text(bytes) == text:
			return "fresh"
	var ok := _generate(src, dst, text)
	return "generated" if ok else "fail"


func _generate(src: String, dst: String, text: String) -> bool:
	var enc: PackedByteArray = GdJsonCodec.encrypt_text(text)
	var f := FileAccess.open(dst, FileAccess.WRITE)
	if f == null:
		printerr("[RegenGjson] %s 写入失败" % dst)
		return false
	f.store_buffer(enc)
	f.close()
	# 回读校验：解密必须还原出源文本（防写坏/加密异常）
	var rf := FileAccess.open(dst, FileAccess.READ)
	var back := rf.get_buffer(rf.get_length())
	rf.close()
	if GdJsonCodec.decrypt_to_text(back) != text:
		printerr("[RegenGjson] %s 回读校验失败（密文解密不等于源）" % dst)
		return false
	print("[RegenGjson] %s -> %s" % [src, dst])
	return true
