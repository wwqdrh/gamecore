# 测试存档隔离守卫 —— demo 无头测试专用
#
# 背景（2026-10-11 事故）：所有 demo 无头测试曾直接跑在玩家真实默认档
# （user://coredata.data，save_id ""）上——reset_demo / 场景流测试的天数写入 /
# 强制更新会整档覆盖玩家进度（玩家游戏内 30s playtime flush 与测试写入
# 互相 last-writer-wins），表现为「存档列表里境界/天数全变默认值」。
#
# 用法：每个 demo 无头测试在 _run() 开头（任何 XiuState.ins() 之前）调用
#   isolate()，在退出前调用 restore()：
#
#   var tsg: GDScript = load("res://example/demo/xiuxian/state/test_save_guard.gd")
#   tsg.isolate()
#   ...
#   tsg.restore()
#   quit(0)
#
# isolate() 幂等且自愈：先切回默认档（离开可能残留的 "_test"），删除上次
# 残留的测试档文件，再切到 "_test"——上次测试崩溃未 restore 也不会脏数据。
# 注意：测试内不要再对 save_id "" 有任何写入；check_title_save 是存档 UI
# 专项测试，自带清理逻辑，不使用本守卫。
extends RefCounted

## 测试专用存档 id（文件 user://coredata__test.data）
const TEST_SAVE_ID := "_test"
## 测试档文件路径
const TEST_PATH := "user://coredata__test.data"


## 切到测试专用档（并清掉上次残留），保证测试不碰玩家真实存档
static func isolate() -> void:
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("")  # 先离开可能残留的 "_test"（set_save_id 同值会早退）
	_remove_test_file()
	core.set_save_id(TEST_SAVE_ID)


## 切回默认档并删除测试档文件（幂等）
static func restore() -> void:
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("")
	_remove_test_file()


static func _remove_test_file() -> void:
	if FileAccess.file_exists(TEST_PATH):
		DirAccess.remove_absolute(TEST_PATH)
