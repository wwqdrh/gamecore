# 一次性工具：headless 重建 demo 全部 gml 的 .gml.tscn 产物
# （等效编辑器插件的 mtime 扫描重建，用于 CI/无头环境）
# 运行：godot --headless --path . -s res://test/regen_gml_tscn.gd
extends SceneTree


func _initialize() -> void:
	var builder := GdUiBuilder.new()
	var files: PackedStringArray = []
	_files(files, "res://example/demo/xiuxian")
	var failed := 0
	for f in files:
		var scene: PackedScene = builder.build_scene_file(f)
		if scene == null:
			push_error("[Regen] %s 构建失败: %s" % [f, builder.last_error()])
			failed += 1
			continue
		var err := ResourceSaver.save(scene, f + ".tscn")
		if err != OK:
			push_error("[Regen] %s 保存失败 err=%d" % [f, err])
			failed += 1
	print("[Regen] total=%d failed=%d" % [files.size(), failed])
	quit(0 if failed == 0 else 1)


func _files(out: PackedStringArray, dir_path: String) -> void:
	var dir := DirAccess.open(dir_path)
	if dir == null:
		return
	dir.list_dir_begin()
	var name := dir.get_next()
	while not name.is_empty():
		var full := dir_path + "/" + name
		if dir.current_is_dir():
			if not name.begins_with("."):
				_files(out, full)
		elif name.ends_with(".gml"):
			out.append(full)
		name = dir.get_next()
	dir.list_dir_end()
