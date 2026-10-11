## 编辑器写盘守卫验证（2026-10-12）：编辑器进程严禁写存档。
## 用法：分别在 --editor 与非 editor 模式下跑本脚本，断言行为差异。
##   编辑器模式：godot --headless --editor -s res://test/test_editor_no_write.gd
##     （编辑器模式下引擎会继续运行，脚本用 quit 主动退出）
## 预期：--editor 模式 update 不落盘（守卫生效）；普通模式正常落盘（无回归）。
extends SceneTree

const PATH := "user://test_editor_guard.data"

func _init() -> void:
	var is_editor: bool = Engine.is_editor_hint()
	# 清场
	if FileAccess.file_exists(PATH):
		DirAccess.remove_absolute(PATH)

	var data: GdCoreData = GdCoreData.build(PATH, "{}", false, "init")
	data.update("probe_key", "~", 42.0, "init")
	# update 触发的 save_fn 是同步调用，无需等帧
	var written := FileAccess.file_exists(PATH)

	if is_editor:
		if written:
			printerr("[EditorGuard] FAIL: 编辑器进程竟然写出了存档文件！")
		else:
			print("[EditorGuard] PASS: 编辑器进程未写盘（守卫生效）")
	else:
		if written:
			print("[EditorGuard] PASS: 普通进程正常写盘（无回归）")
		else:
			printerr("[EditorGuard] FAIL: 普通进程写盘失败（守卫误伤）")
	# 冷读验证（普通模式）
	if written:
		var back: GdCoreData = GdCoreData.build(PATH, "{}", false, "init")
		var v: Variant = back.value("probe_key", 0.0, "init")
		if absf(float(v) - 42.0) < 0.001:
			print("[EditorGuard] PASS: 冷读回读值正确")
		else:
			printerr("[EditorGuard] FAIL: 冷读值不符: %s" % str(v))

	if FileAccess.file_exists(PATH):
		DirAccess.remove_absolute(PATH)
	quit(0)
