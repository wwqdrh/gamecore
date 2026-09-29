# suite: view-camera - GdViewCamera 边界限制测试
# 覆盖: update_limit 的 0 值边界（左/上）设置与 NaN 哨兵跳过语义
extends "res://test/test_case.gd"


func test_update_limit_zero_edges() -> void:
	var tree := Engine.get_main_loop() as SceneTree
	var cam := GdViewCamera.new()
	tree.root.add_child(cam)

	# 地图从 (0,0) 开始：左/上边界就是 0，必须被设置（回归：0 曾被当哨兵跳过）
	cam.update_limit(Vector4(0.0, 960.0, 0.0, 576.0))
	assert_eq(cam.get_limit(SIDE_LEFT), 0, "左边界 0 应被设置")
	assert_eq(cam.get_limit(SIDE_TOP), 0, "上边界 0 应被设置")
	assert_eq(cam.get_limit(SIDE_RIGHT), 960, "右边界应被设置")
	assert_eq(cam.get_limit(SIDE_BOTTOM), 576, "下边界应被设置")

	# NaN 表示该边保持不变
	cam.update_limit(Vector4(NAN, 100.0, NAN, 200.0))
	assert_eq(cam.get_limit(SIDE_LEFT), 0, "NaN 应保持左边界不变")
	assert_eq(cam.get_limit(SIDE_TOP), 0, "NaN 应保持上边界不变")
	assert_eq(cam.get_limit(SIDE_RIGHT), 100, "非 NaN 应更新右边界")
	assert_eq(cam.get_limit(SIDE_BOTTOM), 200, "非 NaN 应更新下边界")

	cam.free()
