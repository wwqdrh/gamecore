@tool
extends EditorPlugin
## gamecore 编辑器插件
##   - 自动挂载 Lua 控制台面板（` 键呼出）
##   - .gml 按纯文本方式打开/编辑（注册到 textfile_extensions）
##   - 自动扫描项目中的 .gml，调用 Rust 侧 GdUiBuilder 构建并生成
##     同名 .gml.tscn（真实场景文件，可运行/可挂载）；.gml 修改后
##     对应 .gml.tscn 自动重新生成（GML 是唯一源码，tscn 是生成产物）

const GML_POLL_INTERVAL := 1.0
## 生成文件说明（打印在输出面板，避免用户误以为 tscn 可手改）
const GEN_NOTE := "[GmlAutoGen] .gml.tscn 由 .gml 自动生成，请勿手动编辑 tscn（改动会在下次 .gml 保存时被覆盖）"

var _console_panel: CanvasLayer
var _scan_timer: Timer
var _gml_mtimes: Dictionary = {}


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
	# 轮询扫描 .gml 并同步生成 .gml.tscn
	_scan_timer = Timer.new()
	_scan_timer.wait_time = GML_POLL_INTERVAL
	_scan_timer.timeout.connect(_poll_gml_files)
	add_child(_scan_timer)
	_scan_timer.start()
	# 编辑器启动 2 秒后先做一次完整扫描（等文件系统扫描就绪），
	# 为缺失/过期的 .gml 补生成 tscn
	var first_timer := Timer.new()
	first_timer.one_shot = true
	first_timer.wait_time = 2.0
	first_timer.timeout.connect(_poll_gml_files)
	add_child(first_timer)
	first_timer.start()


func _exit_tree():
	if _scan_timer:
		_scan_timer.queue_free()
		_scan_timer = null
	if _console_panel:
		_console_panel.queue_free()
		_console_panel = null


## 轮询：收集项目内全部 .gml 文件，与上次 mtime 比对，
## 对新增/修改的文件重新生成对应 .gml.tscn
func _poll_gml_files() -> void:
	var files := PackedStringArray()
	_collect_gml_files("res://", files)

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
		return

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
		# 通知编辑器文件系统有新 tscn 生成
		EditorInterface.get_resource_filesystem().scan()


## 用 DirAccess 递归收集 .gml 文件（res:// 绝对路径）
## 不走 EditorFileSystem：其树在后台扫描完成前为空，且新文件要等 scan 才可见；
## 目录遍历跳过引擎缓存与构建产物目录
func _collect_gml_files(dir_path: String, out: PackedStringArray) -> void:
	var dir := DirAccess.open(dir_path)
	if dir == null:
		return
	dir.list_dir_begin()
	var name := dir.get_next()
	while name != "":
		if dir.current_is_dir():
			if name != "." and name != ".." and name != ".godot" and name != "target":
				_collect_gml_files(dir_path.path_join(name), out)
		elif name.get_extension() == "gml":
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


## 把 gml 加入纯文本扩展（编辑器中双击用文本编辑器打开源码）。
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
		if e != "" and e != "gml":
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
