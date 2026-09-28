# suite: runtime - 协程系统测试（异步）
# 覆盖: wait_frames/wait_seconds/wait_signal 工厂、run(Callable) 步进协程、
#        pause/resume、kill、finished 信号
# 本套件包含异步用例（内部 await），依赖测试框架的协程支持
extends "res://test/test_case.gd"


func _make_node(node_name: String) -> Node:
	# 协程会以 call_deferred 方式挂到该节点下，节点必须在场景树内
	var tree := Engine.get_main_loop() as SceneTree
	var node := Node.new()
	node.name = node_name
	tree.root.add_child(node)
	return node


func test_wait_frames() -> void:
	var node := _make_node("CoroWaitFrames")
	var results := []
	var coro := SpireCoroutine.wait_frames(node, 3)
	coro.finished.connect(func(r): results.append(r))

	assert_false(coro.is_finished(), "刚创建时不应完成")
	assert_true(coro.is_running(), "刚创建时应在运行中")

	# 等待完成（给足帧数余量: deferred 挂载 1 帧 + 3 帧等待）
	var fired := await wait_signal(coro.finished, 60)
	assert_true(fired, "wait_frames(3) 应在 60 帧内触发 finished")

	await wait_frames(1)
	# 协程完成后会自释放节点，可能已不可访问
	assert_true(not is_instance_valid(coro) or coro.is_finished(), "完成后协程应标记完成或已自释放")
	node.free()


func test_wait_seconds() -> void:
	var node := _make_node("CoroWaitSeconds")
	var coro := SpireCoroutine.wait_seconds(node, 0.05)

	var fired := await wait_signal(coro.finished, 120)
	assert_true(fired, "wait_seconds(0.05) 应触发 finished")
	await wait_frames(1)
	assert_true(not is_instance_valid(coro) or coro.is_finished(), "完成后协程应标记完成或已自释放")
	node.free()


func test_wait_signal_trigger() -> void:
	var node := _make_node("CoroWaitSignal")
	# 用 coro_a 的 finished 信号作为 coro_b 的等待目标
	var coro_a := SpireCoroutine.wait_frames(node, 2)
	var coro_b := SpireCoroutine.wait_signal(node, Signal(coro_a, "finished"))

	# coro_a 完成前，coro_b 不应完成
	await wait_frames(1)
	assert_false(coro_b.is_finished(), "信号未触发前 coro_b 不应完成")

	# coro_a 完成后（2 帧），coro_b 应随即完成
	var fired := await wait_signal(coro_b.finished, 120)
	assert_true(fired, "等待的信号触发后 coro_b 应完成")
	await wait_frames(1)
	assert_true(
		(not is_instance_valid(coro_a) or coro_a.is_finished())
		and (not is_instance_valid(coro_b) or coro_b.is_finished()),
		"两个协程均应完成或已自释放")
	node.free()


func test_run_callable_steps() -> void:
	var node := _make_node("CoroRunCallable")
	# step 协程: 每帧收到 delta，累计 3 帧后返回结果
	var counter := {"frames": 0}
	var step := func(delta: float):
		counter["frames"] += 1
		if counter["frames"] >= 3:
			return counter["frames"]
		return null

	var results := []
	var coro := SpireCoroutine.run(node, step)
	coro.finished.connect(func(r): results.append(r))

	var fired := await wait_signal(coro.finished, 120)
	assert_true(fired, "step 协程应在 3 帧后完成")
	await wait_frames(1)
	assert_eq(counter["frames"], 3, "step 应被调用恰好 3 次")
	# 注意: 协程完成后自释放，不要在此后再访问 coro
	assert_eq(results.size(), 1, "finished 应携带一个结果")
	if results.size() == 1:
		assert_eq(int(results[0]), 3, "finished 结果应为 3")
	node.free()


func test_pause_and_resume() -> void:
	var node := _make_node("CoroPauseResume")
	var coro := SpireCoroutine.wait_frames(node, 5)

	# 立即暂停（协程 deferred 挂载，下一帧起生效）
	coro.pause()
	assert_true(coro.is_paused(), "pause 后 is_paused 应为 true")

	# 等 20 帧（远超 5 帧目标），协程应仍未完成
	await wait_frames(20)
	await wait_frames(2)  # 再给 deferred 挂载留时间
	assert_false(coro.is_finished(), "暂停期间协程不应完成")

	# 恢复后应能完成
	coro.resume()
	var fired := await wait_signal(coro.finished, 120)
	assert_true(fired, "resume 后协程应完成")
	node.free()


func test_kill() -> void:
	var node := _make_node("CoroKill")
	var coro := SpireCoroutine.wait_frames(node, 100)
	coro.kill()

	# kill 会立即销毁协程节点，之后不可再访问
	await wait_frames(10)
	assert_true(not is_instance_valid(coro) or not coro.is_finished(), "kill 后协程不应标记为完成")
	node.free()
