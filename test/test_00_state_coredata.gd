# suite: state - GdCoreData 核心数据管理器测试
# 文件名用 00 前缀确保最先运行: manager 套件创建 GdSceneRoot 后，
# GDCORE 单例会因 Rust 侧 Engine::get_singleton 引用计数问题被提前释放（疑似扩展层 bug）
# 覆盖: build 工厂、change/value/has、watch 订阅回调、持久化恢复、GDCORE 单例
# 注意: GdCoreData 是 Resource（RefCounted），由引用计数释放，不要手动 free()
extends "res://test/test_case.gd"

# 每次运行使用独立存档文件，避免污染
# 注意: GdCoreData 的文档结构为 {"{scope}": {data}}，字段读写必须带非空 scope
#       （scope="" 时写入落在 "" 键下，而查询会跳过空路径段，导致读写失配）
const SCOPE := "main"
var _file_seq := 0


func _unique_file() -> String:
	_file_seq += 1
	return "user://test_coredata_%d_%d.data" % [Time.get_ticks_msec(), _file_seq]


func _make_core(data: String = "{}") -> GdCoreData:
	return GdCoreData.build(_unique_file(), data, true, SCOPE)


func test_change_value_roundtrip() -> void:
	var core := _make_core('{"hp":100}')

	# 初始值读取
	assert_eq(core.value("hp", 0, SCOPE), 100, "初始 hp 应为 100")
	assert_true(core.has("hp", SCOPE), "has(hp) 应为 true")
	assert_false(core.has("mp", SCOPE), "has(mp) 应为 false")

	# change 强制写入（action="~"）
	core.change("hp", "~", "50", SCOPE)
	assert_eq(core.value("hp", 0, SCOPE), 50, "change 后 hp 应为 50")

	# update 用 Variant 写入（内部 stringify）
	core.update("mp", "~", 30, SCOPE)
	assert_eq(core.value("mp", 0, SCOPE), 30, "update 后 mp 应为 30")

	# 非强制写入已存在字段不应覆盖（GJson 语义: action != "~" 且路径已存在则跳过）
	core.change("hp", "set", "10", SCOPE)
	assert_eq(core.value("hp", 0, SCOPE), 50, "非强制写入已存在字段不应覆盖")

	# 默认值: 不存在的字段返回 default
	assert_eq(core.value("missing", "fallback", SCOPE), "fallback", "缺失字段应返回默认值")


func test_scope_isolation() -> void:
	var core := _make_core("{}")

	core.change("gold", "~", "10", "player1")
	core.change("gold", "~", "20", "player2")

	assert_eq(core.value("gold", 0, "player1"), 10, "player1 作用域 gold 应为 10")
	assert_eq(core.value("gold", 0, "player2"), 20, "player2 作用域 gold 应为 20")


func test_watch_callback() -> void:
	var core := _make_core("{}")

	var hits := []
	core.watch("gold", func(path: String): hits.append(path), SCOPE)

	# 强制写入触发订阅
	core.change("gold", "~", "1", SCOPE)
	assert_eq(hits.size(), 1, "强制写入应触发一次 watch 回调")

	# 非强制写入已存在字段不触发
	core.change("gold", "set", "2", SCOPE)
	assert_eq(hits.size(), 1, "非强制写入已存在字段不应触发回调")

	# 写入其他字段不触发
	core.change("gems", "~", "3", SCOPE)
	assert_eq(hits.size(), 1, "写入未订阅字段不应触发回调")


func test_persistence_restore() -> void:
	# 两个实例使用同一个存档文件: 先写入并保存，再加载验证
	var filename := _unique_file()

	var writer := GdCoreData.build(filename, '{"coins":7}', true, SCOPE)
	writer.change("coins", "~", "99", SCOPE)

	var reader := GdCoreData.build(filename, "{}", false, SCOPE)
	assert_eq(reader.value("coins", 0, SCOPE), 99, "新实例应从存档恢复 coins=99")


func test_force_overwrite() -> void:
	var filename := _unique_file()

	var writer := GdCoreData.build(filename, '{"coins":7}', true, SCOPE)
	writer.change("coins", "~", "99", SCOPE)

	# force=true: 初始数据覆盖旧存档
	var reinit := GdCoreData.build(filename, '{"coins":1}', true, SCOPE)
	assert_eq(reinit.value("coins", 0, SCOPE), 1, "force=true 应以初始数据覆盖存档")

	# force=false: 保留旧存档
	var keeper := GdCoreData.build(filename, '{"coins":2}', false, SCOPE)
	# force=true 只覆盖内存态（下次 change 才落盘），故 keeper 读到 writer 写入的 99
	assert_eq(keeper.value("coins", 0, SCOPE), 99, "force=false 应保留旧存档数据")


func test_is_inited_and_duplicate() -> void:
	var core := _make_core('{"hp":100}')
	assert_true(core.is_inited(), "build 后应处于已初始化状态")
	assert_contains_str(core.duplicate_all_string(), "100", "duplicate_all_string 应含初始值")

	# reload_data 重载
	core.reload_data('{"main":{"hp":55}}')
	assert_eq(core.value("hp", 0, SCOPE), 55, "reload_data 后 hp 应为新值")


func test_gdcore_singleton() -> void:
	# GDCORE 单例在 GDExtension Scene 阶段注册
	if not Engine.has_singleton("GDCORE"):
		assert_true(false, "应存在 GDCORE 单例（若失败请检查扩展是否已构建加载）")
		return

	var gdcore = Engine.get_singleton("GDCORE")
	if gdcore == null:
		assert_true(false, "Engine.get_singleton('GDCORE') 不应返回 null")
		return

	# get_save_id / set_save_id
	var old_id: String = gdcore.get_save_id()
	assert_true(old_id is String, "get_save_id 应返回字符串")
	gdcore.set_save_id("test_slot")
	assert_eq(gdcore.get_save_id(), "test_slot", "set_save_id 后应可查回")
	gdcore.set_save_id(old_id)

	# 全局节点管理
	var probe := Node.new()
	probe.name = "TestGlobalProbe"
	gdcore.add_global_node("test_probe", probe)
	assert_not_null(gdcore.get_global_node("test_probe"), "add_global_node 后应可获取")
	gdcore.remove_global_node("test_probe")
	assert_null(gdcore.get_global_node("test_probe"), "remove_global_node 后应获取为 null")
	probe.free()
