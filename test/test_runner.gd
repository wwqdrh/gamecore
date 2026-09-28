# GDScript 测试运行器
# 用 Godot 命令行运行全部测试（headless）:
#   godot --headless -s res://test/test_runner.gd
# 或在编辑器内直接运行本脚本所在场景均可。
#
# 发现规则: 扫描 res://test/ 下所有 test_*.gd（排除 test_case.gd 基类），
#   每个脚本中所有 test_ 开头的方法会被依次执行。
# 退出码: 0 = 全部通过, 1 = 存在失败。
#
# 也可以只跑指定模块（传参为脚本名不含扩展名，逗号分隔）:
#   godot --headless -s res://test/test_runner.gd -- test_state_linklist,test_rogue
extends SceneTree

const BASE_CLASS_NAME := "GdTestCase"
const EXCLUDE := ["test_case", "test_runner"]


func _initialize() -> void:
	# 整个 _initialize 是协程：异步测试需要主循环跑帧，结束后才 quit
	var only: Array[String] = []
	var args := OS.get_cmdline_user_args()
	if args.size() > 0:
		for part in args[0].split(",", false):
			only.append(part.strip_edges())

	var scripts := _discover_test_scripts(only)
	var total := 0
	var passed := 0
	var failed_suites: Array[String] = []

	print("==================================================")
	print(" GameKit Core GDScript 测试")
	print("==================================================")

	for path in scripts:
		var suite_name := path.get_file().get_basename()
		var results: Array = await _run_suite(path)
		var suite_passed := true
		for r in results:
			total += 1
			if r.passed:
				passed += 1
				print("  [PASS] %s.%s" % [suite_name, r.test_name])
			else:
				suite_passed = false
				print("  [FAIL] %s.%s" % [suite_name, r.test_name])
				for f in r.failures:
					print("         %s" % f)
		if not suite_passed:
			failed_suites.append(suite_name)

	print("--------------------------------------------------")
	print(" 套件数: %d  用例数: %d  通过: %d  失败: %d" % [
		scripts.size(), total, passed, total - passed])
	if failed_suites.is_empty():
		print(" ALL GREEN ✔")
	else:
		print(" 失败套件: %s" % ", ".join(failed_suites))
	print("==================================================")

	quit(0 if total == passed else 1)


## 扫描 test 目录，返回测试脚本路径列表
func _discover_test_scripts(only: Array[String]) -> Array[String]:
	var out: Array[String] = []
	var dir := DirAccess.open("res://test")
	if dir == null:
		push_error("无法打开 res://test 目录")
		return out

	dir.list_dir_begin()
	var fname := dir.get_next()
	while fname != "":
		if not dir.current_is_dir() and fname.begins_with("test_") and fname.ends_with(".gd"):
			var base := fname.get_basename()
			if base in EXCLUDE:
				fname = dir.get_next()
				continue
			if only.is_empty() or base in only:
				out.append("res://test/" + fname)
		fname = dir.get_next()
	out.sort()
	return out


## 运行单个测试脚本中的全部用例
func _run_suite(path: String) -> Array:
	var script: GDScript = load(path)
	if script == null:
		push_error("加载测试脚本失败: " + path)
		return []

	var results: Array = []
	var instance: RefCounted = script.new()
	# 用鸭子类型检查，避免 -s 模式下 class_name 缓存未注册的问题
	if not instance.has_method("run_test") or not instance.has_method("get_test_methods"):
		push_error("脚本未继承 test_case.gd 基类: " + path)
		return results

	for method in instance.get_test_methods():
		# 必须用 call() 动态调用: run_test 内部含 await，
		# 直接方法调用会报 "Trying to call an async function without await"
		var failures = instance.call("run_test", method)
		# 异步测试: run_test 挂起时返回运行时类名为 GDScriptFunctionState 的对象
		if failures is Object and failures.get_class() == "GDScriptFunctionState":
			failures = await failures
		results.append({
			"test_name": method,
			"passed": (failures as PackedStringArray).is_empty(),
			"failures": failures,
		})

	# instance 为 RefCounted，由引用计数自动释放
	return results
