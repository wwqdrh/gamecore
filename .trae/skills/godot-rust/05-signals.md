# 信号连接

## connect_flags

```rust
use godot::classes::object::ConnectFlags;

let callable = Callable::from_object_method(&*self.base_mut(), "my_callback");
let _ = self.base_mut().connect_flags("signal_name", &callable, ConnectFlags::DEFERRED);
```

**易错点**：`connect_flags` 需要 `&mut self`，所以 `watcher` 变量需要声明为 `mut`。

```rust
// 错误
let watcher = TileSetWatcher::new_watcher(tile_set);
watcher.connect_flags(...);  // watcher 不是 mut

// 正确
let mut watcher = TileSetWatcher::new_watcher(tile_set);
watcher.connect_flags(...);
```

## ConnectFlags 枚举

| 值 | 说明 |
|----|------|
| `DEFERRED` | 延迟调用（帧末尾） |
| `PERSIST` | 持久化（保存到场景） |
| `ONE_SHOT` | 只触发一次 |
| `REFERENCE_COUNTED` | 引用计数 |

## emit_signal

```rust
// 无参数信号
self.base_mut().emit_signal("changed", &[]);

// 带参数信号
self.base_mut().emit_signal("custom_signal", &[arg1.to_variant(), arg2.to_variant()]);
```

## 连接信号到方法

```rust
// 连接自身的信号到自身的方法
let callable = Callable::from_object_method(&*self.base_mut(), "_changed");
let _ = self.base_mut().connect_flags("changed", &callable, ConnectFlags::DEFERRED);

// 连接子对象的信号到自身的方法
let callable = Callable::from_object_method(&*self.base_mut(), "reposition");
let _ = watcher.connect_flags("tileset_resized", &callable, ConnectFlags::DEFERRED);
```

## 信号与虚方法的区别

- **信号**：需要手动 `connect`，可以连接到任意方法
- **虚方法**：引擎自动调用，只需在 trait impl 中重写
- **编辑器模式**：原版 GDScript 在编辑器中用 `_process` 轮询代替信号连接，运行时用 `changed.connect(_changed)` 信号
