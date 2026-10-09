# 无头重建 .gjson 加密产物 —— 静态定义表管线工具
#
# 用途：与编辑器插件（addons/gamecore/core.gd 的 _poll_json_files）等价的
#       命令行工具。源 .json 新增/修改后，不必打开编辑器即可重建加密产物：
#   godot --headless --path . -s res://test/regen_gjson.gd
#
# 规则：res:// 下（跳过 .godot / target / addons/bin）**带管线标记**的明文
#   .json —— 顶层字段 "pipeline": "gjson"（opt-in，通用 .json 配置如
#   game_config.json 不进管线）→ JSON 校验 → GdJsonCodec 加密 →
#   同名 .gjson（mtime 旧于源时跳过）。
# 运行时 Table 类（XiuLevelTable / XiuItemTable）只读 .gjson 产物。
extends SceneTree

const SKIP_DIRS := [".godot", "target", "bin"]
## 管线标记字段（与编辑器插件一致）
const PIPELINE_KEY := "pipeline"
const PIPELINE_VALUE := "gjson"


func _init() -> void:
	var sources := PackedStringArray()
	_collect("res://", sources)
	var generated := 0
	var skipped := 0
	for src in sources:
		var dst := src.get_basename() + ".gjson"
		if FileAccess.file_exists(dst) \
				and FileAccess.get_modified_time(dst) >= FileAccess.get_modified_time(src):
			skipped += 1
			continue
		if _generate(src, dst):
			generated += 1
	print("[RegenGjson] done: %d generated, %d fresh" % [generated, skipped])
	quit(0)


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


func _generate(src: String, dst: String) -> bool:
	var text := FileAccess.get_file_as_string(src)
	var parsed: Variant = JSON.parse_string(text)
	if parsed == null:
		printerr("[RegenGjson] %s JSON 解析失败，跳过" % src)
		return false
	# opt-in：只有声明了管线标记的定义表源文件才生成加密产物
	var is_pipeline := parsed is Dictionary \
			and str((parsed as Dictionary).get(PIPELINE_KEY, "")) == PIPELINE_VALUE
	if not is_pipeline:
		return false
	var f := FileAccess.open(dst, FileAccess.WRITE)
	if f == null:
		printerr("[RegenGjson] %s 写入失败" % dst)
		return false
	f.store_buffer(GdJsonCodec.encrypt_text(text))
	f.close()
	print("[RegenGjson] %s -> %s" % [src, dst])
	return true
