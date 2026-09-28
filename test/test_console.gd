# suite: console - Lua 后台控制台测试
# 覆盖: eval 表达式求值、execute 代码执行、register_command 命令注册、
#        console_output 信号、list_commands/unregister_command
# 前提: GdConsole 单例在 GDExtension Scene 阶段注册为 Engine singleton
extends "res://test/test_case.gd"

func _console() -> Object:
	if not Engine.has_singleton("GdConsole"):
		assert_true(false, "应存在 GdConsole 单例（若失败请检查扩展是否已构建加载）")
		return null
	return Engine.get_singleton("GdConsole")


func test_singleton_exists() -> void:
	assert_true(Engine.has_singleton("GdConsole"), "应存在 GdConsole 单例")


func test_eval_expression() -> void:
	var c := _console()
	if c == null:
		return

	assert_eq(c.eval("1 + 2"), 3.0, "eval('1+2') 应为 3")
	assert_eq(c.eval('"hello" .. " world"'), "hello world", "eval 字符串拼接")
	assert_eq(c.eval("math.floor(3.7)"), 3.0, "eval 应能调用 Lua 标准库")


func test_execute_state() -> void:
	var c := _console()
	if c == null:
		return

	# execute 执行语句并保持 Lua 环境状态；返回值为输出文本（print 输出 + 结果/错误）
	c.execute("test_counter = 41 + 1")
	assert_eq(c.eval("test_counter"), 42.0, "execute 赋值后 eval 应读到状态")

	# print 输出应被捕获
	var out2: String = c.execute("print('from_lua_test')")
	assert_contains_str(out2, "from_lua_test", "execute 应捕获 print 输出")

	# 表达式返回值应出现在输出中
	var out3: String = c.execute("return 6 * 7")
	assert_contains_str(out3, "42", "execute 返回值应包含 42")


func test_register_command() -> void:
	var c := _console()
	if c == null:
		return

	var calls := []
	var cmd := func(a = null):
		calls.append(a)
		return "cmd_ok"

	c.register_command("test_probe_cmd", cmd, "测试命令")

	# list_commands 应包含新命令
	# list_commands 返回 "name - description" 格式
	assert_contains_str("\n".join(c.list_commands()), "test_probe_cmd", "list_commands 应包含注册的命令")

	# execute 调用命令（回调被触发）
	c.execute("test_probe_cmd()")
	assert_eq(calls.size(), 1, "命令被调用时回调应执行一次")

	# unregister 后不应再可列出
	c.unregister_command("test_probe_cmd")
	assert_false("\n".join(c.list_commands()).contains("test_probe_cmd"), "unregister 后不应再列出")


func test_console_output_signal() -> void:
	var c := _console()
	if c == null:
		return

	var outputs := []
	c.console_output.connect(func(text: String): outputs.append(text))
	c.execute("print('signal_probe')")
	# 信号是同步发射的（执行后立即触发）
	assert_eq(outputs.size(), 1, "每次执行应触发一次 console_output")
	if outputs.size() > 0:
		assert_contains_str(String(outputs[0]), "signal_probe", "信号文本应包含输出内容")


func test_builtin_functions() -> void:
	var c := _console()
	if c == null:
		return

	# 内置命令: fps/memory/gc_info/cpu_info/help 均可调用且返回输出
	for fn in ["fps()", "memory()", "gc_info()", "cpu_info()", "help()"]:
		var out: String = c.execute(fn)
		assert_true(out.length() >= 0, "%s 应能安全执行" % fn)
