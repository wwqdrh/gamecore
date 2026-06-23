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

## 扫描子节点并按名称分类（Node2D 管理子节点模式）

当 Node2D 类需要管理场景文件中预定义的子节点时，通过 `get_children()` + `try_cast()` + 节点名匹配：

```rust
fn scan_child_layers(&mut self) {
    self.display_layers.clear();
    self.prop_layer = None;

    let children = self.base().get_children();
    for child in children.iter_shared() {
        if let Ok(layer) = child.clone().try_cast::<TileMapLayer>() {
            let name = layer.get_name().to_string();
            if name == "PropLayer" {
                self.prop_layer = Some(layer);
            } else {
                self.display_layers.push(layer);
            }
        }
    }
}
```

**关键点**：
- `self.base().get_children()` 返回 `Array<Node>`，需通过 `iter_shared()` 遍历
- `try_cast::<T>()` 消费所有权，必须先 `clone()` 子节点引用
- `get_name()` 返回 `StringName`，需 `.to_string()` 转为 Rust `String`
- 此模式适用于"场景文件定义子节点 + Rust 代码管理"的架构

## 批量同步属性到子节点（call() + Variant clone）

当需要将一个属性同步到多个子节点时，注意 `Variant` 在循环中会被 move：

```rust
fn sync_tile_set_to_layers(&mut self) {
    let Some(ref ts) = self.tile_set else { return };
    let ts_var = ts.to_variant();

    for layer in &self.display_layers {
        let mut layer = layer.clone();
        // ts_var 在循环中被 move，需要 clone
        layer.call("set_tile_set", &[ts_var.clone()]);
    }

    if let Some(ref mut layer) = self.prop_layer {
        // 最后一次使用，无需 clone
        layer.call("set_tile_set", &[ts_var]);
    }
}
```

**关键点**：
- `Variant` 不是 `Copy`，在循环中重复使用需要 `.clone()`
- 最后一次使用时无需 clone
- `call("set_tile_set", &[variant])` 用于绕过 ByValue/ByOption 类型不匹配（见下方）

## 动态创建子节点并添加到场景树

当 Node2D 类需要动态创建子节点（而非从场景文件获取）时，使用 `new_alloc()` 创建 + `add_child()` 添加：

```rust
use godot::obj::NewGd;

fn ensure_prop_layer(&mut self) {
    if self.prop_layer.is_some() {
        return;
    }

    // 检查是否已存在同名子节点（避免重复创建）
    let children = self.base().get_children();
    for child in children.iter_shared() {
        if let Ok(layer) = child.clone().try_cast::<TileMapLayer>() {
            if layer.get_name().to_string() == "PropLayer" {
                self.prop_layer = Some(layer);
                return;
            }
        }
    }

    // 动态创建
    let mut prop_layer = TileMapLayer::new_alloc();
    prop_layer.set_name("PropLayer");

    // 同步 tile_set（使用 call 绕过类型不匹配）
    if let Some(ref ts) = self.tile_set {
        prop_layer.call("set_tile_set", &[ts.to_variant()]);
    }

    // 添加为子节点
    self.base_mut().add_child(&prop_layer);

    // 设置 owner 为场景根节点（编辑器中可见）
    if let Some(owner) = self.base().get_owner() {
        prop_layer.set_owner(&owner);
    }

    self.prop_layer = Some(prop_layer);
}
```

**关键点**：
- `new_alloc()` 创建堆分配的 Godot 对象（Node 类型必须用 `new_alloc`，不能用 `new_gd`）
- `add_child()` 后对象所有权转移给场景树，但 `Gd<T>` 引用仍可用
- `set_owner()` 使节点在编辑器场景树中可见（运行时非必须）
- 创建前应检查是否已存在同名子节点，避免重复创建

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
