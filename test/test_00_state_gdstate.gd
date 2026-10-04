# suite: state - GdState 临时状态总线测试（非持久化）
# 文件名 00 前缀确保最先运行（与 test_00_state_coredata 同理：避免 manager 套件
# 创建 GdSceneRoot 后 Rust 侧 Engine::get_singleton 引用计数问题提前释放单例）
# 覆盖: 单例注册、set/get/has/erase、watch 注册即回调、变更通知、等值跳过、
#        unwatch、回调参数约定（1 参 value / 2 参 value+key）、防重注册
# 红线: GDSTATE 是 godot-rust Object 实例（GdCell 单借用），监听回调内【禁止】
#        同步再调用本单例任何方法（set_state/get_state/watch…）——会触发
#        "already bound" panic。需要级联状态时用 call_deferred 包一层。
# 注意: GDSTATE 进程级存活、状态跨用例保留——每个用例必须使用自己的唯一键
extends "res://test/test_case.gd"

var _seq := 0


## 每次调用返回唯一键，杜绝跨用例状态污染
func _key() -> String:
	_seq += 1
	return "test.gdstate.%d.k%d" % [Time.get_ticks_msec(), _seq]


func _state():
	return Engine.get_singleton("GDSTATE")


func test_singleton_registered() -> void:
	assert_true(Engine.has_singleton("GDSTATE"), "应存在 GDSTATE 单例（Scene 阶段注册）")
	assert_not_null(_state(), "Engine.get_singleton('GDSTATE') 不应返回 null")


func test_set_get_has_erase() -> void:
	var st = _state()
	var k := _key()
	assert_false(st.has_state(k), "未设置时 has 应为 false")
	assert_null(st.get_state(k), "未设置时 get 应为 null")

	st.set_state(k, 42)
	assert_true(st.has_state(k), "set 后 has 应为 true")
	assert_eq(st.get_state(k), 42, "set 后 get 应回读")

	st.erase_state(k)
	assert_false(st.has_state(k), "erase 后 has 应为 false")
	assert_null(st.get_state(k), "erase 后 get 应为 null")


func test_watch_initial_callback() -> void:
	var st = _state()
	var k1 := _key()
	var k2 := _key()

	var hits := []
	var cb := func(v): hits.append(v)
	# 注册即用当前值回调一次（未设置 → nil）
	st.watch(k1, cb)
	assert_eq(hits.size(), 1, "watch 注册应立即回调一次")
	assert_null(hits[0], "未设置键的初始回调应为 nil")

	# 已有值的键：初始回调携带当前值
	st.set_state(k2, "hello")
	var hits2 := []
	var cb2 := func(v): hits2.append(v)
	st.watch(k2, cb2)
	assert_eq(hits2.size(), 1, "watch 已有值键应立即回调一次")
	assert_eq(hits2[0], "hello", "初始回调应携带当前值")

	st.unwatch(k1, cb)
	st.unwatch(k2, cb2)


func test_notify_and_equal_skip() -> void:
	var st = _state()
	var k := _key()
	var k2 := _key()
	var hits := []
	var cb := func(v): hits.append(v)
	st.watch(k, cb)

	st.set_state(k, 1)
	assert_eq(hits.size(), 2, "值变化应通知（初始 nil + set 1）")
	assert_eq(hits[1], 1, "通知应携带新值")

	# 等值写入不通知（Variant 深比较）
	st.set_state(k, 1)
	assert_eq(hits.size(), 2, "等值写入不应通知")

	# 写入其他键不影响本键监听
	st.set_state(k2, "x")
	assert_eq(hits.size(), 2, "写入未监听键不应通知")

	# erase 不触发回调
	st.erase_state(k)
	assert_eq(hits.size(), 2, "erase 不应触发回调")

	st.unwatch(k, cb)


func test_watch_dedup() -> void:
	var st = _state()
	var k := _key()
	var hits := []
	var cb := func(v): hits.append(v)
	st.watch(k, cb)
	st.watch(k, cb)  # 重复注册应去重

	st.set_state(k, "once")
	assert_eq(hits.size(), 2, "重复 watch 只应注册一次（初始 + 一次通知）")

	st.unwatch(k, cb)


func test_unwatch() -> void:
	var st = _state()
	var k := _key()
	var k2 := _key()
	var hits := []
	var cb := func(v): hits.append(v)
	st.watch(k, cb)
	st.unwatch(k, cb)

	st.set_state(k, 7)
	assert_eq(hits.size(), 1, "unwatch 后不应再收到通知（仅剩注册时初始回调）")
	assert_eq(st.get_state(k), 7, "unwatch 不影响状态本身")

	# key 不匹配的 unwatch 静默忽略
	var hits2 := []
	var cb2 := func(v): hits2.append(v)
	st.watch(k2, cb2)
	st.unwatch(k, cb2)
	st.set_state(k2, "still-on")
	assert_eq(hits2.size(), 2, "key 不匹配的 unwatch 不应移除监听（初始 + 通知）")
	assert_eq(hits2[1], "still-on", "通知应正常到达")

	st.unwatch(k2, cb2)


func test_callback_arity() -> void:
	var st = _state()
	var k := _key()
	# 1 参回调收 value；2 参回调收 (value, key)
	var seen := {}
	var cb2 := func(v, key): seen[key] = v
	st.watch(k, cb2)

	st.set_state(k, "val")
	assert_eq(seen.get(k, null), "val", "2 参回调应收到 (value, key)")

	st.unwatch(k, cb2)
