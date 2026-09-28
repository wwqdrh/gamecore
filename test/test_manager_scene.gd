# suite: manager - 场景状态管理测试（部分异步）
# 覆盖: GdScene 状态机（change_state/back_state/状态栈）、s_state_changed 信号、
#        独立模式自动创建 GdSceneRoot、get_manager
# 注意: 不要 free 测试场景 —— GdSceneRoot 持有 GdScene 引用并注册在 GDCORE 全局表中，
#        手动 free 会留下悬空引用（间歇性引发 GDCORE 单例访问崩溃）
extends "res://test/test_case.gd"


func _make_scene(scene_name: String) -> GdScene:
	var tree := Engine.get_main_loop() as SceneTree
	var scene := GdScene.new()
	scene.name = scene_name
	tree.root.add_child(scene)
	# ready() 在下一帧触发（独立模式会自动 on_enter / on_ready）
	return scene


func test_state_stack_and_signal() -> void:
	var scene := _make_scene("SceneProbe1")
	await wait_frames(2)  # 等待 _ready 触发

	var transitions := []
	scene.s_state_changed.connect(func(state: String, _data): transitions.append(state))

	# 初始状态栈应为空（未设置 current_state 时）
	assert_eq(Array(scene.get_state_stack()).size(), 0, "未设置状态时栈应为空")

	# change_state 压栈并发射信号
	var data := {"level": 1}
	scene.change_state("menu", data)
	scene.change_state("playing", {})

	assert_eq(Array(scene.get_state_stack()), ["menu", "playing"], "状态栈应为 [menu, playing]")
	assert_eq(transitions.size(), 2, "应发射两次 s_state_changed")
	if transitions.size() >= 2:
		assert_eq(String(transitions[0]), "menu", "第一次切换应为 menu")
		assert_eq(String(transitions[1]), "playing", "第二次切换应为 playing")

	# 状态初始化数据应被记录
	var init_data: Dictionary = scene.get_state_init_data()
	assert_true(init_data.has("menu"), "应记录 menu 的初始数据")
	if init_data.has("menu"):
		assert_eq(int(init_data["menu"].get("level", 0)), 1, "menu 初始数据应含 level=1")



func test_back_state() -> void:
	var scene := _make_scene("SceneProbe2")
	await wait_frames(2)

	scene.change_state("a", {})
	scene.change_state("b", {})
	scene.change_state("c", {})
	assert_eq(Array(scene.get_state_stack()), ["a", "b", "c"], "三次入栈后应为 [a, b, c]")

	# 回退: 弹出 c 回到 b
	scene.back_state()
	assert_eq(Array(scene.get_state_stack()), ["a", "b"], "back_state 后应为 [a, b]")

	# 回退到栈底时拒绝再退
	scene.back_state()
	assert_eq(Array(scene.get_state_stack()), ["a"], "再次回退应为 [a]")
	scene.back_state()
	assert_eq(Array(scene.get_state_stack()), ["a"], "栈底再回退不应变化")


func test_standalone_manager_autocreate() -> void:
	# 独立 GdScene（无 __managed meta）应自动查找/创建默认 GdSceneRoot
	var scene := _make_scene("SceneProbe3")
	await wait_frames(2)

	var manager = scene.get_manager()
	assert_not_null(manager, "独立模式应自动创建/关联 GdSceneRoot")
	if manager != null:
		assert_true(manager is GdSceneRoot, "manager 应为 GdSceneRoot 类型")


func test_scene_properties() -> void:
	var scene := _make_scene("SceneProbe4")
	await wait_frames(2)

	# 初始属性
	assert_eq(scene.current_state, "", "初始 current_state 应为空字符串")
	assert_eq(scene.get_state_init_data().size(), 0, "初始状态数据应为空字典")
