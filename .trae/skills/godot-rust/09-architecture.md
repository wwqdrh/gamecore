# 架构模式

## 循环依赖处理

### 问题

模块 A 需要引用模块 B 的类型，模块 B 也需要引用模块 A 的类型。

例如：`TileMapDual` 创建 `DisplayLayer`，`DisplayLayer` 需要 `TileMapDual` 来获取属性。

### 解决方案：使用基类类型

```rust
// DisplayLayer 需要 TileMapDual，但 TileMapDual 创建 DisplayLayer
// 解决：DisplayLayer 使用 Gd<TileMapLayer>（基类）而非 Gd<TileMapDual>

// DisplayLayer 中
fn update_properties(&mut self, parent: Gd<TileMapLayer>) {
    // 通过 call() 访问 TileMapDual 特有的属性
    let material_var = parent.call("get_display_material", &[]);
    // ...
}

// Display 中
#[var(pub)]
world: Option<Gd<TileMapLayer>>,  // 而非 Option<Gd<TileMapDual>>
```

### 模式总结

1. **子节点用基类类型**：`Gd<TileMapLayer>` 而非 `Gd<TileMapDual>`
2. **子类特有方法用 call()**：`parent.call("get_display_material", &[])`
3. **upcast 传递**：`self.base_mut().clone().upcast::<TileMapLayer>()`

```rust
// TileMapDual 中创建 Display 时 upcast
let self_as_layer = self.base_mut().clone().upcast::<TileMapLayer>();
display.bind_mut().setup_public(self_as_layer, watcher.clone());
```

## 编辑器 vs 运行时模式

原版 GDScript 在编辑器和运行时使用不同的更新机制：

```rust
fn ready(&mut self) {
    // ...
    if godot::classes::Engine::singleton().is_editor_hint() {
        // 编辑器：用 _process 轮询
        self.base_mut().set_process(true);
    } else {
        // 运行时：用 changed 信号
        let callable = Callable::from_object_method(&*self.base_mut(), "_changed");
        let _ = self.base_mut().connect_flags("changed", &callable, ConnectFlags::DEFERRED);
        self.base_mut().set_process(false);
    }
}
```

## 虚方法 + 信号协同

TileMapDual 使用两种机制协同工作：
1. **虚方法 `update_cells`**：引擎在格子变化时自动调用，传入变化的坐标
2. **`_process` 轮询 → `_changed`**：编辑器中定期更新 TileSetWatcher 和属性
