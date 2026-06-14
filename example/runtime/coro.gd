# 协程示例 - 演示 SpireCoroutine 的 GDScript 侧 API
# SpireCoroutine 提供静态工厂方法供 GDScript 创建协程，
# 返回的对象支持暂停、恢复、销毁等操作，并监听 finished 信号。
extends Node


func _ready() -> void:
	print("=== SpireCoroutine 示例 ===")
	_demo_wait_frames()
	_demo_wait_seconds()
	_demo_wait_signal()
	_demo_run_callable()
	_demo_pause_resume()
	_demo_kill()


# 等待指定帧数
func _demo_wait_frames() -> void:
	var coro = SpireCoroutine.wait_frames(self, 60)
	coro.finished.connect(func(_r): print("[frames] 60帧后完成"))


# 等待指定秒数
func _demo_wait_seconds() -> void:
	var coro = SpireCoroutine.wait_seconds(self, 3.0)
	coro.finished.connect(func(_r): print("[seconds] 3秒后完成"))


# 等待信号
func _demo_wait_signal() -> void:
	# 等待自身进入场景树的信号（仅作演示，_ready 时已进入）
	var sig = Signal(self, "tree_entered")
	var coro = SpireCoroutine.wait_signal(self, sig)
	coro.finished.connect(func(_r): print("[signal] 信号已发射"))


# 从 Callable 创建协程
# callable 每帧调用一次，参数为 delta_time (float)
# 返回 null → 继续等待，返回非 null → 协程完成
var _elapsed: float = 0.0

func _demo_run_callable() -> void:
	_elapsed = 0.0
	var coro = SpireCoroutine.run(self, _step_elapsed)
	coro.finished.connect(func(r): print("[run] 累计 %.1f 秒后完成" % r))


func _step_elapsed(delta: float):
	_elapsed += delta
	if _elapsed >= 2.0:
		return _elapsed  # 非 null = 完成，值作为结果
	return null          # null = 继续等待


# 暂停与恢复
func _demo_pause_resume() -> void:
	var coro = SpireCoroutine.wait_seconds(self, 5.0)
	coro.pause()
	print("[pause/resume] 已暂停, is_paused: ", coro.is_paused())

	# 2秒后恢复
	var timer = get_tree().create_timer(2.0)
	timer.timeout.connect(func():
		coro.resume()
		print("[pause/resume] 已恢复, is_running: ", coro.is_running())
	)


# 销毁协程（不触发 finished 信号）
func _demo_kill() -> void:
	var coro = SpireCoroutine.wait_seconds(self, 999.0)
	# 1秒后销毁
	var timer = get_tree().create_timer(1.0)
	timer.timeout.connect(func():
		coro.kill()
		print("[kill] 已销毁, is_finished: ", coro.is_finished())
	)
