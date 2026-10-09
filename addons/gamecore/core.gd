@tool
extends EditorPlugin
## gamecore 编辑器插件
##   - 自动挂载 Lua 控制台面板（` 键呼出）
##   - .gml 按纯文本方式打开/编辑（注册到 textfile_extensions）
##   - 自动扫描项目中的 .gml，调用 Rust 侧 GdUiBuilder 构建并生成
##     同名 .gml.tscn（真实场景文件，可运行/可挂载）；.gml 修改后
##     对应 .gml.tscn 自动重新生成（GML 是唯一源码，tscn 是生成产物）
##   - 自动扫描项目中的明文 .json 定义表，经 GdJsonCodec 加密生成
##     同名 .gjson 产物（静态定义表管线：.json 是唯一源码，.gjson 是
##     加密生成物，运行时 Table 类只读 .gjson；产物勿手改勿提交外发）
##     .gjson 双击以 GdJson 资源打开（Rust 侧 ResourceFormatLoader 接管，
##     不按纯文本打开——密文无文本编辑意义）

const GML_POLL_INTERVAL := 1.0
## 生成文件说明（打印在输出面板，避免用户误以为 tscn 可手改）
const GEN_NOTE := "[GmlAutoGen] .gml.tscn 由 .gml 自动生成，请勿手动编辑 tscn（改动会在下次 .gml 保存时被覆盖）"

var _console_panel: CanvasLayer
var _scan_timer: Timer
var _gml_mtimes: Dictionary = {}
var _json_mtimes: Dictionary = {}


func _enter_tree():
	# 自动加载控制台面板
	_console_panel = load("res://addons/gamecore/ui/console_panel.gd").new()
	get_editor_interface().get_base_control().add_child(_console_panel)

	# 清理之前 import plugin 留下的缓存数据，否则 Godot 仍将 .gml 当作导入资源
	_cleanup_gml_import_cache()
	# 清理编辑器布局中残留的 .gml "场景" 页签（旧版 loader 的工作流产物，
	# 现在 .gml 是纯文本，作为场景重新打开会报 No loader found）
	_cleanup_gml_open_scenes()
	# .gml 按纯文本打开（编辑器可直接编辑源码）
	_register_gml_text_extension()


func _ready() -> void:
	# 轮询扫描 .gml / .json 并同步生成 .gml.tscn / .gjson 产物
	_scan_timer = Timer.new()
	_scan_timer.wait_time = GML_POLL_INTERVAL
	_scan_timer.timeout.connect(_poll_generated_files)
	add_child(_scan_timer)
	_scan_timer.start()
	# 编辑器启动 2 秒后先做一次完整扫描（等文件系统扫描就绪），
	# 为缺失/过期的 .gml / .json 补生成产物
	var first_timer := Timer.new()
	first_timer.one_shot = true
	first_timer.wait_time = 2.0
	first_timer.timeout.connect(_poll_generated_files)
	add_child(first_timer)
	first_timer.start()


func _exit_tree():
	if _scan_timer:
		_scan_timer.queue_free()
		_scan_timer = null
	if _console_panel:
		_console_panel.queue_free()
		_console_panel = null
	# 规避 Godot 4.5/4.6 引擎退出崩溃（见 _remove_editor_doc_cache）
	_remove_editor_doc_cache()


## 规避 Godot 引擎已知崩溃 bug（godotengine/godot#111048 / #111645，
## 修复 PR #123658 截至本注释写入时仍未合并进任何 stable 版本）：
##   崩溃链：编辑器启动时 docs 缓存（editor_doc_cache-<主>.<次>.res）命中 →
##   worker 线程向主线程延迟注册 EditorHelp::_gen_extensions_docs →
##   退出时 EditorNode 析构先跑 EditorHelp::cleanup_doc()（memdelete(doc);
##   doc=nullptr）→ Main::cleanup 中 message_queue->flush() 执行残留回调 →
##   doc->generate() 对空指针解引用 → EXC_BAD_ACCESS(0x8) → macOS
##   「异常退出」弹窗（崩溃报告栈底：DocTools::generate ←
##   EditorHelp::_gen_extensions_docs ← CallQueue::flush ← Main::cleanup）。
##   规避方式：编辑器退出时删除全局 docs 缓存文件，使下次启动必然走
##   「缓存未命中 → 主线程同步 generate」路径——该路径不注册
##   _gen_extensions_docs，其余 deferred 回调（load_script_doc_cache 等）
##   均不直接解引用 doc，退出安全。
##   代价：下次启动文档全量生成（约 1~2 秒）。引擎修复合并后可删除本段。
func _remove_editor_doc_cache() -> void:
	if not Engine.is_editor_hint():
		return
	# EditorPaths / OS.get_cache_path 均未暴露给脚本，按引擎 C++ 逻辑
	# 复现各平台缓存目录（self-contained _sc_ 模式不覆盖，官方发行版不受影响）
	var cache_dir := ""
	match OS.get_name():
		"macOS":
			cache_dir = OS.get_environment("HOME").path_join("Library/Caches/Godot")
		"Linux":
			var xdg := OS.get_environment("XDG_CACHE_HOME")
			if xdg.is_empty():
				xdg = OS.get_environment("HOME").path_join(".cache")
			cache_dir = xdg.path_join("Godot")
		"Windows":
			var local := OS.get_environment("LOCALAPPDATA")
			if local.is_empty():
				local = OS.get_environment("APPDATA")
			cache_dir = local.path_join("Godot")
	if cache_dir.is_empty():
		return
	var vi := Engine.get_version_info()
	var cache_file: String = cache_dir.path_join(
		"editor_doc_cache-%d.%d.res" % [int(vi.major), int(vi.minor)])
	if FileAccess.file_exists(cache_file):
		var err := DirAccess.remove_absolute(cache_file)
		if err != OK:
			printerr("[gamecore] 删除编辑器 docs 缓存失败（%s）: %s" % [cache_file, error_string(err)])


## 轮询：gml→tscn 与 json→gjson 两类产物统一入口
func _poll_generated_files() -> void:
	var need_fs_scan := false
	if _poll_gml_files():
		need_fs_scan = true
	if _poll_json_files():
		need_fs_scan = true
	if need_fs_scan:
		# 通知编辑器文件系统有新产物生成
		EditorInterface.get_resource_filesystem().scan()


## 轮询 .gml：收集项目内全部 .gml 文件，与上次 mtime 比对，
## 对新增/修改的文件重新生成对应 .gml.tscn。有产物写入返回 true。
func _poll_gml_files() -> bool:
	var files := PackedStringArray()
	_collect_files_by_ext("res://", "gml", files)

	var changed: Array[String] = []
	var seen := {}
	for path in files:
		seen[path] = true
		var mtime := FileAccess.get_modified_time(path)
		var last = _gml_mtimes.get(path)
		if last == null:
			# 编辑器首次扫描：tscn 已存在且不旧于 gml 时跳过，避免每次启动全量重建
			_gml_mtimes[path] = mtime
			if _is_tscn_fresh(path):
				continue
			changed.append(path)
		elif last != mtime:
			_gml_mtimes[path] = mtime
			changed.append(path)

	# .gml 被删除时清理对应的生成物
	for path in _gml_mtimes.keys():
		if not seen.has(path):
			_gml_mtimes.erase(path)

	if changed.is_empty():
		return false

	var saved := false
	for path in changed:
		if _generate_tscn(path):
			saved = true
	if saved:
		# .gml.tscn 若正打开在场景编辑器中，重新加载以反映最新结构
		var open_scenes := EditorInterface.get_open_scenes()
		for path in changed:
			var tscn_path: String = path + ".tscn"
			if tscn_path in open_scenes:
				EditorInterface.reload_scene_from_path(tscn_path)
	return saved


## 用 DirAccess 递归收集指定扩展名文件（res:// 绝对路径）
## 不走 EditorFileSystem：其树在后台扫描完成前为空，且新文件要等 scan 才可见；
## 目录遍历跳过引擎缓存与构建产物目录
func _collect_files_by_ext(dir_path: String, ext: String, out: PackedStringArray) -> void:
	var dir := DirAccess.open(dir_path)
	if dir == null:
		return
	dir.list_dir_begin()
	var name := dir.get_next()
	while name != "":
		if dir.current_is_dir():
			if name != "." and name != ".." and name != ".godot" and name != "target":
				_collect_files_by_ext(dir_path.path_join(name), ext, out)
		elif name.get_extension() == ext:
			out.append(dir_path.path_join(name))
		name = dir.get_next()
	dir.list_dir_end()


## 对应 .gml.tscn 是否存在且不旧于 .gml（首次扫描用于跳过未变更文件）
func _is_tscn_fresh(gml_path: String) -> bool:
	var tscn_path := gml_path + ".tscn"
	if not FileAccess.file_exists(tscn_path):
		return false
	return FileAccess.get_modified_time(tscn_path) >= FileAccess.get_modified_time(gml_path)


## 构建 .gml 并保存为同名 .gml.tscn，成功返回 true
func _generate_tscn(gml_path: String) -> bool:
	var builder := GdUiBuilder.new()
	var packed: PackedScene = builder.build_scene_file(gml_path)
	var tscn_path := gml_path + ".tscn"
	if packed == null:
		printerr("[GmlAutoGen] %s 生成失败: %s（保留旧的 %s）" % [gml_path, builder.last_error(), tscn_path])
		return false
	var err := ResourceSaver.save(packed, tscn_path)
	if err != OK:
		printerr("[GmlAutoGen] %s 保存失败: %s" % [tscn_path, error_string(err)])
		return false
	print(GEN_NOTE)
	print("[GmlAutoGen] %s -> %s" % [gml_path, tscn_path])
	return true


# ---------- 静态定义表管线：明文 .json 源 → 加密 .gjson 产物 ----------

const GJSON_GEN_NOTE := "[GjsonAutoGen] .gjson 由 .json 加密生成，请勿手动编辑（改动会在下次 .json 保存时被覆盖）"

## 管线标记字段：定义表源文件顶层声明 "pipeline": "gjson" 才进管线
## （opt-in——.json 是通用扩展名，运行时配置如 game_config.json 走明文，
## 不可无差别转换；与 test/regen_gjson.gd 保持一致）
const GJSON_PIPELINE_KEY := "pipeline"
const GJSON_PIPELINE_VALUE := "gjson"

## 轮询 .json：与上次 mtime 比对，对新增/修改的定义表生成加密 .gjson。
## 有产物写入返回 true。
func _poll_json_files() -> bool:
	var files := PackedStringArray()
	_collect_files_by_ext("res://", "json", files)

	var changed: Array[String] = []
	var seen := {}
	for path in files:
		seen[path] = true
		var mtime := FileAccess.get_modified_time(path)
		var last = _json_mtimes.get(path)
		if last == null:
			_json_mtimes[path] = mtime
			if _is_gjson_fresh(path):
				continue
			changed.append(path)
		elif last != mtime:
			_json_mtimes[path] = mtime
			changed.append(path)

	# 源 .json 被删除时清理对应生成物
	for path in _json_mtimes.keys():
		if not seen.has(path):
			_json_mtimes.erase(path)

	if changed.is_empty():
		return false

	var saved := false
	for path in changed:
		if _generate_gjson(path):
			saved = true
	return saved


## 对应 .gjson 是否存在且不旧于 .json（首次扫描用于跳过未变更文件）
func _is_gjson_fresh(json_path: String) -> bool:
	var gjson_path := json_path.get_basename() + ".gjson"
	if not FileAccess.file_exists(gjson_path):
		return false
	return FileAccess.get_modified_time(gjson_path) >= FileAccess.get_modified_time(json_path)


## 读明文 .json → 校验管线标记与可解析 → GdJsonCodec 加密 → 写同名 .gjson，
## 成功返回 true
func _generate_gjson(json_path: String) -> bool:
	var text := FileAccess.get_file_as_string(json_path)
	var parsed: Variant = JSON.parse_string(text)
	if parsed == null:
		printerr("[GjsonAutoGen] %s JSON 解析失败，跳过生成（保留旧的 .gjson）" % json_path)
		return false
	# opt-in：只有声明了管线标记的定义表源文件才生成加密产物
	if not (parsed is Dictionary) \
			or str((parsed as Dictionary).get(GJSON_PIPELINE_KEY, "")) != GJSON_PIPELINE_VALUE:
		return false
	var gjson_path := json_path.get_basename() + ".gjson"
	var encrypted := GdJsonCodec.encrypt_text(text)
	var f := FileAccess.open(gjson_path, FileAccess.WRITE)
	if f == null:
		printerr("[GjsonAutoGen] %s 写入失败" % gjson_path)
		return false
	f.store_buffer(encrypted)
	f.close()
	print(GJSON_GEN_NOTE)
	print("[GjsonAutoGen] %s -> %s" % [json_path, gjson_path])
	return true


## 把 gml 加入纯文本扩展（FileSystem 面板可见、双击用文本编辑器打开）。
## 注意：gjson 不在纯文本扩展里——它是加密产物（密文无文本编辑意义），
## 双击按文本打开只会产生无效 UTF-8 错误日志；由 Rust 侧注册的
## ResourceFormatLoader（state/gjson_loader.rs）接管，双击以 GdJson 资源
## 打开。此处每次启动强制把 gjson 从既有设置中剔除（清掉历史残留）。
## 注意：Godot 4.6 内部按「逗号分隔的 String」解析该设置
## （editor_file_system.cpp: (String)EDITOR_GET(...).split(",")），
## 必须写回 String；写成 PackedStringArray 会被强转成带引号括号的
## 垃圾 token，导致扩展名匹配失败、文件不被索引、FileSystem 不可见。
func _register_gml_text_extension() -> void:
	var editor_settings := get_editor_interface().get_editor_settings()
	var setting_name := "docks/filesystem/textfile_extensions"
	var extensions := PackedStringArray(["txt", "md", "cfg", "ini", "log"])
	if editor_settings.has_setting(setting_name):
		var val = editor_settings.get_setting(setting_name)
		if val is String:
			extensions = String(val).split(",", false)
		elif val is PackedStringArray or val is Array:
			extensions = PackedStringArray(val)   # 兼容旧格式（本插件早期误写）
	var parts := PackedStringArray()
	for ext in extensions:
		var e := String(ext).strip_edges()
		if e != "" and e != "gml" and e != "gjson":
			parts.append(e)
	parts.append("gml")
	# 始终以 String 形式写回（4.6 引擎按 String 解析）
	editor_settings.set_setting(setting_name, ",".join(parts))


## 从编辑器布局（.godot/editor/editor_layout.cfg，[EditorNode] 小节）中
## 移除 .gml 场景页签，避免编辑器启动时把 .gml 当场景加载报错。
## 另兼容清理 project_metadata.cfg 的 [editor_states] open_scenes（旧版本键位）。
## 注意：插件初始化（first_scan 阶段）早于编辑器布局加载，此处清理时机会生效。
func _cleanup_gml_open_scenes() -> void:
	_layout_cfg_filter(ProjectSettings.globalize_path("res://.godot/editor/editor_layout.cfg"),
		"EditorNode")
	_layout_cfg_filter(ProjectSettings.globalize_path("res://.godot/editor/project_metadata.cfg"),
		"editor_states")


func _layout_cfg_filter(cfg_path: String, section: String) -> void:
	if not FileAccess.file_exists(cfg_path):
		return
	var cfg := ConfigFile.new()
	if cfg.load(cfg_path) != OK:
		return
	var dirty := false
	if cfg.has_section_key(section, "open_scenes"):
		var open_scenes = cfg.get_value(section, "open_scenes")
		if open_scenes is PackedStringArray or open_scenes is Array:
			var filtered := PackedStringArray()
			for path in open_scenes:
				if path.get_extension() != "gml":
					filtered.append(path)
			if filtered.size() != (open_scenes as Array).size():
				cfg.set_value(section, "open_scenes", filtered)
				dirty = true
				# 打开的场景全被过滤时，current_scene 一并清掉
				var current = cfg.get_value(section, "current_scene") if cfg.has_section_key(section, "current_scene") else null
				if current != null and (current as String).get_extension() == "gml":
					cfg.set_value(section, "current_scene", filtered[0] if not filtered.is_empty() else "")
					dirty = true
	if dirty:
		cfg.save(cfg_path)


func _cleanup_gml_import_cache() -> void:
	var project_dir: String = ProjectSettings.globalize_path("res://")
	var imported_dir_path: String = project_dir.path_join(".godot/imported")
	var dir := DirAccess.open(imported_dir_path)
	if dir:
		dir.list_dir_begin()
		var file_name: String = dir.get_next()
		while file_name != "":
			if ".gml-" in file_name:
				dir.remove(file_name)
			file_name = dir.get_next()
		dir.list_dir_end()

	# 同时清理 editor 目录下的 .gml 缓存
	var editor_dir_path: String = project_dir.path_join(".godot/editor")
	var edir := DirAccess.open(editor_dir_path)
	if edir:
		edir.list_dir_begin()
		var file_name: String = edir.get_next()
		while file_name != "":
			if ".gml-" in file_name:
				edir.remove(file_name)
			file_name = edir.get_next()
		edir.list_dir_end()
