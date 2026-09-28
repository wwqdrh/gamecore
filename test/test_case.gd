# GDScript 测试基类
# 所有 test_*.gd 测试脚本继承此类，用 test_ 前缀命名测试方法。
#
# 用法示例:
#   extends "res://test/test_case.gd"
#
#   func test_something() -> void:
#       var v := 1 + 1
#       assert_eq(v, 2, "1+1 应等于 2")
#
# 断言失败会被收集（不中断用例），由 test_runner.gd 汇总输出。
class_name GdTestCase
extends RefCounted

var _failures: PackedStringArray = []
var _current_test: String = ""


# ---------------------------------------------------------------------------
# 执行入口（由 test_runner.gd 调用，测试脚本不需要关心）
# ---------------------------------------------------------------------------

## 运行单个测试方法，返回该方法收集到的失败列表。
## 支持协程测试：若测试方法内部使用 await（等待帧/信号），
## 动态调用会返回 GDScriptFunctionState（运行时类名，无法作为编译期类型引用），
## 这里通过 get_class() 识别并等待其完成后再返回结果。
func run_test(method: String) -> PackedStringArray:
	_failures = []
	_current_test = method
	var ret = call(method)
	if ret is Object and ret.get_class() == "GDScriptFunctionState":
		await ret
	return _failures.duplicate()


## 等待 N 帧（headless 下 SceneTree 主循环正常迭代）
func wait_frames(count: int = 1) -> void:
	var tree := Engine.get_main_loop() as SceneTree
	for i in count:
		await tree.process_frame


## 等待指定信号触发或超时（帧数），返回是否在超时前触发。
## 实现方式: 挂接回调 + 每帧轮询标志位，避免竞态丢失。
func wait_signal(sig: Signal, timeout_frames: int = 300) -> bool:
	var result := [false]
	var on_fire := func(_a = null, _b = null, _c = null, _d = null): result[0] = true
	var sig_obj = sig.get_object()
	if sig_obj == null or not is_instance_valid(sig_obj):
		_fail("wait_signal: 信号所属对象已释放 (%s)" % sig)
		return false
	if sig.connect(on_fire) != OK:
		_fail("wait_signal: 信号连接失败 (%s)" % sig)
		return false
	var tree := Engine.get_main_loop() as SceneTree
	var frames := 0
	while frames < timeout_frames and not result[0]:
		await tree.process_frame
		frames += 1
	# 对象可能在等待期间被释放，断开前需再检查
	if is_instance_valid(sig.get_object()) and sig.is_connected(on_fire):
		sig.disconnect(on_fire)
	return result[0]


## 等待直到条件成立或超时，返回是否满足
func wait_until(cond: Callable, timeout_frames: int = 300) -> bool:
	var tree := Engine.get_main_loop() as SceneTree
	var frames := 0
	while frames < timeout_frames:
		if cond.call():
			return true
		await tree.process_frame
		frames += 1
	return cond.call()


## 返回该测试类中所有 test_ 开头的方法名
func get_test_methods() -> Array[String]:
	var methods: Array[String] = []
	for m in get_method_list():
		if m.name.begins_with("test_"):
			methods.append(m.name)
	methods.sort()
	return methods


# ---------------------------------------------------------------------------
# 断言 API
# ---------------------------------------------------------------------------

func assert_true(cond: bool, msg: String = "条件应为 true") -> void:
	if not cond:
		_fail(msg)


func assert_false(cond: bool, msg: String = "条件应为 false") -> void:
	if cond:
		_fail(msg)


func assert_eq(actual, expected, msg: String = "值应相等") -> void:
	if actual != expected:
		_fail("%s (actual: %s, expected: %s)" % [msg, actual, expected])


func assert_ne(actual, expected, msg: String = "值应不相等") -> void:
	if actual == expected:
		_fail("%s (两者相等: %s)" % [msg, actual])


func assert_near(actual: float, expected: float, eps: float = 0.001, msg: String = "浮点值应近似相等") -> void:
	if absf(actual - expected) > eps:
		_fail("%s (actual: %s, expected: %s, eps: %s)" % [msg, actual, expected, eps])


func assert_null(value, msg: String = "值应为 null") -> void:
	if value != null:
		_fail("%s (actual: %s)" % [msg, value])


func assert_not_null(value, msg: String = "值不应为 null") -> void:
	if value == null:
		_fail(msg)


func assert_between(value: float, low: float, high: float, msg: String = "值应在区间内") -> void:
	if value < low or value > high:
		_fail("%s (value: %s, 区间: [%s, %s])" % [msg, value, low, high])


func assert_contains_str(haystack: String, needle: String, msg: String = "字符串应包含子串") -> void:
	if not haystack.contains(needle):
		_fail("%s (未找到: %s)" % [msg, needle])


func _fail(msg: String) -> void:
	_failures.append("[%s] %s" % [_current_test, msg])
