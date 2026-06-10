@tool
extends EditorPlugin

var _console_panel: CanvasLayer
var _scan_timer: Timer


func _enter_tree():
	# 自动加载控制台面板
	_console_panel = load("res://addons/gamecore/ui/console_panel.gd").new()
	get_editor_interface().get_base_control().add_child(_console_panel)

	# 清理之前 import plugin 留下的缓存数据，否则 Godot 仍将 .gml 当作导入资源
	_cleanup_gml_import_cache()
	# 注册 .gml 为文本扩展名，使 .gml 文件在 FileSystem 中可见且双击可用脚本编辑器打开
	_register_gml_extension()


func _exit_tree():
	if _scan_timer:
		_scan_timer.queue_free()
		_scan_timer = null
	if _console_panel:
		_console_panel.queue_free()
		_console_panel = null


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


func _register_gml_extension() -> void:
	var editor_settings = get_editor_interface().get_editor_settings()
	var setting_name = "docks/filesystem/textfile_extensions"
	var extensions: PackedStringArray
	if editor_settings.has_setting(setting_name):
		var val = editor_settings.get_setting(setting_name)
		if val is PackedStringArray:
			extensions = val
		elif val is String:
			extensions = PackedStringArray(val.split(","))
		else:
			extensions = PackedStringArray(["txt", "md", "cfg", "ini", "log"])
	if not extensions.has("gml"):
		extensions.append("gml")
		editor_settings.set_setting(setting_name, extensions)
	# 用定时器延迟扫描，避免在编辑器初始化期间调用 scan 导致冲突
	_scan_timer = Timer.new()
	_scan_timer.one_shot = true
	_scan_timer.wait_time = 2.0
	_scan_timer.timeout.connect(_on_scan_timeout)
	add_child(_scan_timer)
	_scan_timer.start()


func _on_scan_timeout() -> void:
	get_editor_interface().get_resource_filesystem().scan()
	_scan_timer.queue_free()
	_scan_timer = null
