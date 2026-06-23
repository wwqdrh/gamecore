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

## 克隆 Vec<Gd<T>> 避免循环中的借用冲突

当需要在循环中遍历 `self` 的 `Vec<Gd<T>>` 字段，同时访问 `self` 的其他字段时，先克隆整个 Vec：

```rust
// 错误：循环中借用 self.display_layers，同时访问 self.terrain_registry 和 self.dual_grid
for layer in &self.display_layers {
    let name = layer.get_name().to_string();
    let terrain = self.terrain_registry.get_id(&name).unwrap(); // 借用冲突！
    self.dual_grid.set_world_tile(pos, terrain); // 借用冲突！
}

// 正确：先克隆 Vec<Gd<T>>，再在循环中访问 self 的其他字段
let layers: Vec<Gd<TileMapLayer>> = self.display_layers.iter().cloned().collect();
for layer in &layers {
    let name = layer.get_name().to_string();
    let terrain = self.terrain_registry.get_id(&name).unwrap(); // 安全
    self.dual_grid.set_world_tile(pos, terrain); // 安全
}
```

**关键点**：
- `Gd<T>` 是引用计数类型，`clone()` 开销小（仅增加引用计数）
- `Vec<Gd<T>>::iter().cloned().collect()` 克隆整个 Vec
- 此模式适用于"遍历子节点集合 + 读写 self 其他字段"的场景

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

## 虚方法/#[func] 重入 panic（重要！）

当引擎调用虚方法（如 `update_cells`、`ready`）或 `#[func]` 方法时，godot-rust 会 `bind_mut()` 当前对象。如果在此方法内部通过 `call()` 调用同一对象的另一个 `#[func]` 方法，会触发 `Gd<T>::bind() failed, already bound` panic。

```rust
// 场景：TileMapDual 的虚方法 update_cells 被引擎调用（对象已被 bind_mut）
fn update_cells(&mut self, coords: Array<Vector2i>, _forced: bool) {
    // ... 内部调用 DisplayLayer::update_properties
    // DisplayLayer 通过 parent.call("get_display_material") 调用 TileMapDual 的 #[func] 方法
    // → 尝试再次 bind() TileMapDual → panic!
}

// 错误：在已绑定的对象上通过 call() 调用 #[func] getter
fn update_properties(&mut self, mut parent: Gd<TileMapLayer>) {
    // parent 实际是 TileMapDual，get_display_material 是其 #[func] 方法
    let material_var = parent.call("get_display_material", &[]); // 重入 panic!
}

// 正确：通过参数传递数据，避免 call() 跨绑定边界
fn update_properties(&mut self, mut parent: Gd<TileMapLayer>, material: Option<Gd<Material>>) {
    // material 由调用方在 bind 作用域内直接读取字段后传入
    base.call("set_material", &[material.to_variant()]);
}

// 调用方在 &mut self 作用域内直接读取字段并传递
fn update_cells(&mut self, coords: Array<Vector2i>, _forced: bool) {
    let Some(mut display) = self.display.clone() else { return };
    // 直接读 self 字段，无需 call()
    display.bind_mut().set_display_material_public(self.display_material.clone());
    display.bind_mut().update_public(variants);
}
```

**规则**：在虚方法或 `#[func]` 方法内部，不要通过 `call()` 调用同一 GodotClass 的其他 `#[func]` 方法。改为：
1. 直接读取 `self` 字段（在同一 bind 作用域内安全）
2. 通过参数将数据传递给子节点/其他对象

**注意**：`base.get_xxx()` / `base.set_xxx()` 等引擎内置方法不经过 Rust bind，可以安全调用。只有自定义 `#[func]` 方法通过 `call()` 调用时会触发 bind。
