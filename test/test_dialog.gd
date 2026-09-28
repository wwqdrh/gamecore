# suite: dialog - 对话脚本引擎测试
# 覆盖: 时间线文本解析（initial）、stage 枚举/索引/跳转、角色注册、属性访问器
# 说明: next()/exec_response() 依赖 dialogue_control UI 节点，
#        UI 交互流程建议在编辑器场景内人工验证，这里覆盖数据层 API
extends "res://test/test_case.gd"

const TIMELINE := """
[stage_start]
(narrator)
欢迎来到测试对话
- 去街道@goto:street
- 去学校@goto:school
.
..

[street]
(user,bona)
这里是街道分支
...
..

[school]
(user,bona)
这里是学校分支
...
"""


func _make_dialogue() -> GdDialogue:
	var d := GdDialogue.new()
	d.initial(TIMELINE)
	return d


func test_all_stages() -> void:
	var d := _make_dialogue()
	var stages := Array(d.all_stages())
	assert_eq(stages.size(), 3, "应解析出 3 个 stage")
	assert_true(stages.has("stage_start"), "应包含 stage_start")
	assert_true(stages.has("street"), "应包含 street")
	assert_true(stages.has("school"), "应包含 school")
	d.free()


func test_stage_index() -> void:
	var d := _make_dialogue()
	assert_eq(d.stage_index("stage_start"), 0, "stage_start 索引应为 0")
	assert_eq(d.stage_index("street"), 1, "street 索引应为 1")
	assert_eq(d.stage_index("school"), 2, "school 索引应为 2")
	assert_eq(d.stage_index("nonexistent"), -1, "不存在的 stage 索引应为 -1")
	d.free()


func test_goto_stage() -> void:
	var d := _make_dialogue()
	d.goto_stage("school")
	assert_true(d.has_next(), "跳转到 school 后应有后续对话")

	d.goto_stage("nonexistent_stage")
	# 跳转到不存在的 stage 不应崩溃
	assert_true(true, "goto 非法 stage 不应崩溃")
	d.free()


func test_has_next() -> void:
	var d := _make_dialogue()
	assert_true(d.has_next(), "初始解析后应有后续对话")

	# 空时间线
	var empty := GdDialogue.new()
	empty.initial("")
	assert_false(empty.has_next(), "空时间线 has_next 应为 false")
	empty.free()
	d.free()


func test_role_registration() -> void:
	var tree := Engine.get_main_loop() as SceneTree

	# 对话节点必须先入树（is_registered_role 内部通过节点路径解析角色节点）
	var d := GdDialogue.new()
	d.initial(TIMELINE)
	tree.root.add_child(d)
	await wait_frames(1)

	# is_registered_role 只查询手动注册的映射，时间线角色不会自动注册
	assert_false(d.is_registered_role("probe_role"), "未注册角色应返回 false")

	# 角色节点必须已在场景树内（register_role_node 内部会取 node path）
	var holder := Node.new()
	holder.name = "RoleHolder"
	tree.root.add_child(holder)
	await wait_frames(1)
	d.register_role_node("probe_role", holder)
	assert_true(d.is_registered_role("probe_role"), "注册后角色应可查询")

	# 未注册角色的位置应返回 null
	assert_null(d.get_role_pos("nobody"), "未注册角色位置应为 null")
	d.free()
	holder.free()


func test_property_accessors() -> void:
	var d := GdDialogue.new()

	d.set_click_next(true)
	assert_true(d.get_click_next(), "click_next setter/getter 应一致")

	d.set_skip(true)
	assert_true(d.get_skip(), "skip setter/getter 应一致")

	d.set_skip_time(1.5)
	assert_near(d.get_skip_time(), 1.5, 0.001, "skip_time setter/getter 应一致")

	d.set_timeline_path("res://test/timeline.txt")
	assert_eq(d.get_timeline_path(), "res://test/timeline.txt", "timeline_path 应一致")

	d.set_handle_fn("my_handler")
	assert_eq(d.get_handle_fn(), "my_handler", "handle_fn 应一致")
	d.free()
