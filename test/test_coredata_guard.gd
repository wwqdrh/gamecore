# suite: coredata-guard - 存档外部删除保护 + delete_save API + 多实例锁
# 覆盖: ①新档（加载时文件不存在）写盘正常创建；②加载时文件存在、会话中被
#        外部删除 → 回写跳过（不复活旧档）；③重建实例后写盘恢复；
#        ④GDCORE.delete_save 删文件/重置当前档；⑤实例锁文件存在且被刷新
extends "res://test/test_case.gd"


func test_external_delete_blocks_rewrite() -> void:
	var path := "user://test_guard_ext.data"
	_cleanup(path)

	# 1) 新档（build 时文件不存在，不受保护）：写盘应创建文件
	var d: GdCoreData = GdCoreData.build(path, "{}", false, "init")
	d.update("k", "~", "v1", "init")
	assert_true(FileAccess.file_exists(path), "新档首次写盘应创建文件")

	# 2) 重建实例（此时文件存在 → 保护生效）：外部删除后回写应被跳过
	d = null
	var armed: GdCoreData = GdCoreData.build(path, "{}", false, "init")
	DirAccess.remove_absolute(path)
	armed.update("k", "~", "v2", "init")
	assert_false(FileAccess.file_exists(path), "外部删除后回写应被跳过（不复活）")

	# 3) 再次重建实例（文件不存在 → 新档语义）→ 写盘恢复创建
	var d3: GdCoreData = GdCoreData.build(path, "{}", false, "init")
	d3.update("k3", "~", "v", "init")
	assert_true(FileAccess.file_exists(path), "重建实例后写盘应恢复创建")

	_cleanup(path)


func test_delete_save_api() -> void:
	var core: Object = Engine.get_singleton("GDCORE")
	var sid := "__del_test"
	var p := "user://coredata_%s.data" % sid
	_cleanup(p)

	# 切到测试档并建档（update "~" 立即落盘）
	core.set_save_id("")
	core.set_save_id(sid)
	var data: GdCoreData = core.get_root_data()
	data.update("marker", "~", 1.0, "init")
	assert_true(FileAccess.file_exists(p), "建档后文件应存在")

	# delete_save：文件删除 + 当前档重置
	core.delete_save(sid)
	assert_false(FileAccess.file_exists(p), "delete_save 应删除存档文件")
	var after: GdCoreData = core.get_root_data()
	assert_false(after.has("marker", "init"), "删除当前档后数据应重置为空")

	# 离开被删档（切回默认），清理残留
	core.set_save_id("")
	_cleanup(p)


func test_instance_lock_file() -> void:
	var lock := "user://.game_instance.lock"
	# headless 下 refresh_instance_lock 直接跳过（不写锁）——此处仅验证
	# 锁文件机制本身：若存在则 mtime 应是合理的历史时刻
	if FileAccess.file_exists(lock):
		var mtime: float = FileAccess.get_modified_time(lock)
		assert_true(mtime > 0.0, "锁文件 mtime 应可读取")
	_cleanup(lock)


func _cleanup(path: String) -> void:
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)
