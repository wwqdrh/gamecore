# 协程运行时

对应 Rust 源码：`rust/src/runtime/`（coroutine / yielding / builder / start_coroutine / start_async_task）
可运行示例：`example/runtime/coro.gd`

Rust 侧 API 经 `pub mod prelude` 导出（`SpireCoroutine`、`seconds`/`frames`/`wait_until` 等）。
**GDScript 侧统一走静态工厂** `SpireCoroutine.等待方法(宿主节点, ...)`，宿主节点用于生命周期绑定
（宿主销毁协程自动取消）。

## 等待类协程

```gdscript
var coro = SpireCoroutine.wait_frames(self, 60)        # 等 60 帧
var coro = SpireCoroutine.wait_seconds(self, 3.0)      # 等 3 秒
var coro = SpireCoroutine.wait_signal(self, Signal(self, "tree_entered"))  # 等信号
```

## 执行类协程

```gdscript
# callable(delta) -> null 继续下一帧 / 返回非 null 则完成并带回结果
func _step_elapsed(delta):
    elapsed += delta
    if elapsed >= 3.0:
        return elapsed          # 完成值
    return null

var coro = SpireCoroutine.run(self, _step_elapsed)
```

## 控制

```gdscript
coro.finished.connect(func(result): print("完成: ", result))

coro.pause()
coro.resume()
coro.kill()              # kill 不触发 finished

coro.is_paused()
coro.is_running()
coro.is_finished()
```

## 与原生 await 的取舍

- 简单一次性等待：直接用原生 `await get_tree().create_timer(3.0).timeout` 即可。
- 需要**暂停/恢复/取消/复用/拿到完成值**的流程（过场编排、技能前摇、AI 时序）：用 SpireCoroutine。
- Rust 侧代码内使用 `crate::prelude` 的 `seconds()/frames()/wait_until()/wait_for_signal()` 组合。
