# 借用检查器（Borrow Checker）

## 常见借用冲突

```rust
// 错误：self.base_mut() 借用 self 的同时访问 self.offset
fn reposition(&mut self) {
    let tile_size = self.watcher.bind().get_tile_size();
    self.base_mut().set_position(self.offset * tile_size); // 冲突！
}

// 正确：先克隆需要的值
fn reposition(&mut self) {
    let tile_size = self.watcher.bind().get_tile_size();
    let offset = self.offset;  // 克隆 Vector2（Copy 类型）
    self.base_mut().set_position(offset * tile_size);
}
```

## Gd<T> 的 bind() 借用

```rust
// bind() 返回不可变借用，bind_mut() 返回可变借用
let value = obj.bind().some_field;      // 不可变
obj.bind_mut().some_field = new_value;  // 可变

// 不能同时持有 bind() 和 bind_mut()
let borrowed = obj.bind();
obj.bind_mut();  // 错误！已经借用了

// 需要先释放借用
let value = obj.bind().some_field;
drop(borrowed);  // 或离开作用域
obj.bind_mut().some_field = new_value;
```

## 克隆 Gd<T> 避免借用冲突

```rust
// 克隆 Option<Gd<T>> 或 Gd<T> 后再操作
let Some(mut watcher) = self.tileset_watcher.clone() else { return };
let tile_set = self.base().get_tile_set();
watcher.bind_mut().update(tile_set);
```

## let-else 模式提前返回

```rust
let Some(mut display) = self.display.clone() else { return };
display.bind_mut().update_public(data);
```

## 临时值生命周期

```rust
// 错误：terrain.bind().get_layer_objects() 返回临时值的引用
let layers = terrain.bind().get_layer_objects(); // 临时值立即释放
for layer in layers { /* ... */ }

// 正确：用作用域块先获取需要的值
let layer_count = terrain.bind().get_layer_objects().len();
for i in 0..layer_count {
    let layer = terrain.bind().get_layer_objects()[i].clone();
    // 使用 layer
}
```

## parent.call() 需要 mut

```rust
// 错误：call() 需要 &mut self
fn update_properties(&mut self, parent: Gd<TileMapLayer>) {
    let material_var = parent.call("get_display_material", &[]); // parent 不是 mut
}

// 正确：声明为 mut
fn update_properties(&mut self, mut parent: Gd<TileMapLayer>) {
    let material_var = parent.call("get_display_material", &[]);
}
```
