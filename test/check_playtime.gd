# GDCORE 每存档累计游玩时长验收 —— get_play_time / flush_play_time / 落盘持久化
#
# 运行：perl -e 'alarm 120; exec @ARGV' godot --headless --path . \
#   -s res://test/check_playtime.gd
#
# 覆盖：
#   1. get_play_time：当前存档累计真实秒，随时间增长
#   2. 存档隔离：set_save_id 切新档后时长从 0 起算（各存档独立）
#   3. flush_play_time：会话时长并入持久化值并立即写盘
#   4. 持久化：同存档文件新建 GdCoreData 实例能读到已落盘累计值
#      （= 跨开关应用累积的落盘语义）
#   5. 切档自动落盘：切走再切回，时长无缝继续累计（无丢失）
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[PlayTime] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	# 清理历史测试存档
	DirAccess.remove_absolute("user://coredata_pt_test.data")

	var core: Object = Engine.get_singleton("GDCORE")
	check(core != null, "GDCORE 单例应存在")

	# ---- 1. 基础：当前存档时长随时间增长 ----
	var t0: float = core.call("get_play_time")
	check(t0 >= 0.0, "get_play_time 应返回非负秒数，实际 %s" % str(t0))
	await create_timer(0.35).timeout
	var t1: float = core.call("get_play_time")
	check(t1 > t0 + 0.05, "时长应随时间增长（%.2f → %.2f）" % [t0, t1])

	# ---- 2. 存档隔离：新档从 0 起算 ----
	core.call("set_save_id", "pt_test")
	var a0: float = core.call("get_play_time")
	check(a0 < 0.5, "切换到全新存档后累计时长应接近 0（会话刚开始），实际 %.2f" % a0)

	# ---- 3. flush：会话并入持久化值 ----
	await create_timer(0.35).timeout
	var a1: float = core.call("get_play_time")
	check(a1 > a0 + 0.05, "等待后时长应增长（%.2f → %.2f）" % [a0, a1])
	core.call("flush_play_time")
	var a2: float = core.call("get_play_time")
	check(absf(a2 - a1) < 0.3,
		"flush 后读值应与 flush 前接近（会话并入落盘值且起点重置），%.2f vs %.2f" % [a1, a2])

	# ---- 4. 持久化：同文件新实例可读出落盘值 ----
	var fresh: Resource = GdCoreData.build("user://coredata_pt_test.data", "{}", false, "init")
	var stored: float = float(fresh.call("value", "total", 0.0, "playtime"))
	check(stored > 0.2, "落盘后同存档文件新实例应读到累计时长 >0，实际 %s" % str(stored))

	# ---- 5. 切档自动落盘：切走再切回无丢失 ----
	var before: float = core.call("get_play_time")
	await create_timer(0.3).timeout
	core.call("set_save_id", "")          # 切走（触发 pt_test 落盘）
	core.call("set_save_id", "pt_test")   # 切回
	var after: float = core.call("get_play_time")
	check(after > before + 0.05,
		"切档往返后时长应继续累计（自动落盘），%.2f → %.2f" % [before, after])

	# ---- 清理测试存档 ----
	DirAccess.remove_absolute("user://coredata_pt_test.data")

	if ok:
		print("[PlayTime] RESULT=PASS")
	else:
		print("[PlayTime] RESULT=FAIL")
	quit(0 if ok else 1)
