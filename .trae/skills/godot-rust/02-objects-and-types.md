# 对象创建与类型转换

## 创建 Godot 对象

```rust
use godot::obj::NewGd;

// 创建内置类型
let mut mat = ShaderMaterial::new_gd();
let mut shader = Shader::new_gd();

// 创建自定义类
let mut node = Gd::<MyClass>::from_init_fn(|base| {
    <MyClass as INode2D>::init(base)
});
```

**易错点**：`init` 是 trait 方法，必须用完全限定语法 `<MyClass as ITrait>::init(base)` 或确保 trait 在作用域内 `ITileMapLayer::init(base)`。

## 类型转换（upcast/downcast）

```rust
// upcast：子类 → 父类（安全）
let layer: Gd<TileMapLayer> = tilemap_dual.clone().upcast::<TileMapLayer>();

// downcast：父类 → 子类（可能失败）
let child = self.base().get_children();
for c in child.iter_shared() {
    if let Ok(display_layer) = c.clone().try_cast::<DisplayLayer>() {
        // 转换成功
    }
}
```

## call() 动态调用

```rust
// 当类型不匹配或方法在子类中定义时，使用 call() 动态调用
let result = obj.call("method_name", &[arg1.to_variant(), arg2.to_variant()]);

// call_deferred 延迟调用（第一个参数是方法名字符串，不是 Callable！）
self.base_mut().call_deferred("_changed", &[]);
```

**易错点**：`call_deferred` 的第一个参数是 `impl AsArg<StringName>`（方法名字符串），不是 `Callable`。

```rust
// 错误！
let callable = Callable::from_object_method(&*self.base_mut(), "_changed");
self.base_mut().call_deferred(&callable, &[]);

// 正确
self.base_mut().call_deferred("_changed", &[]);
```

## ByValue/ByOption 类型不匹配

### 问题

某些 Godot 方法的参数类型是 `ByValue`，但 `Option<Gd<T>>` 的 `Pass = ByOption`，导致类型不匹配。

### 受影响的方法

- `set_tile_set(tile_set: Option<Gd<TileSet>>)` → `ByValue` 不匹配
- `set_material(material: Option<Gd<Material>>)` → `ByValue` 不匹配

### 解决方案：使用 call() 动态调用

```rust
// set_tile_set 类型不匹配，改用 call
let tile_set = watcher.bind().get_tile_set();
self.base_mut().call("set_tile_set", &[tile_set.to_variant()]);

// set_material 同理
let material_var = parent.call("get_display_material", &[]);
self.base_mut().call("set_material", &[material_var]);
```
