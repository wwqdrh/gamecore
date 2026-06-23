# 类定义与注册

## 基本类定义

```rust
use godot::prelude::*;
use godot::classes::{ITileMapLayer, TileMapLayer};

#[derive(GodotClass)]
#[class(base = TileMapLayer, tool)]  // tool 启用编辑器模式
pub struct MyNode {
    #[var(pub)]
    my_property: f32,
    base: Base<TileMapLayer>,
}

#[godot_api]
impl ITileMapLayer for MyNode {
    fn init(base: Base<TileMapLayer>) -> Self {
        Self {
            my_property: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        // _ready() 回调
    }

    fn process(&mut self, delta: f64) {
        // _process(delta) 回调
    }
}
```

### 关键点

- **`#[class(tool)]`**：启用编辑器工具模式，节点在编辑器中也会执行 `_ready`/`_process`
- **`Base<T>`**：基类引用，通过 `self.base()` 获取不可变引用，`self.base_mut()` 获取可变引用
- **`init(base)`**：trait 方法，必须通过 trait 调用（如 `ITileMapLayer::init(base)`），不能直接 `MyNode::init(base)`

### 常见基类

| 基类 | trait | init 签名 |
|------|-------|-----------|
| Node | INode | `fn init(base: Base<Node>) -> Self` |
| Node2D | INode2D | `fn init(base: Base<Node2D>) -> Self` |
| Sprite2D | ISprite2D | `fn init(base: Base<Sprite2D>) -> Self` |
| Control | IControl | `fn init(base: Base<Control>) -> Self` |
| TileMapLayer | ITileMapLayer | `fn init(base: Base<TileMapLayer>) -> Self` |
| Resource | IResource | `fn init(base: Base<Resource>) -> Self` |
| RefCounted | IRefCounted | `fn init(base: Base<RefCounted>) -> Self` |

## 属性（#[var]）

### `#[var]` vs `#[var(pub)]`

```rust
#[derive(GodotClass)]
#[class(base = Resource)]
pub struct MyClass {
    // #[var] 生成 deprecated 的 getter/setter，跨模块调用会警告
    #[var]
    old_style: i32,

    // #[var(pub)] 生成非 deprecated 的 getter/setter，推荐使用
    #[var(pub)]
    new_style: i32,
}
```

**易错点**：`#[var]` 单独使用会生成 deprecated 的 getter/setter，跨模块调用时编译器会警告。需要跨模块访问的属性必须用 `#[var(pub)]`。

### 自定义 getter/setter

```rust
#[var(get = get_value, set = set_value)]
value: i32,

fn get_value(&self) -> i32 { self.value }
fn set_value(&mut self, v: i32) { self.value = v; }
```

### 自定义 getter/setter（Option<Gd<T>> 类型）

当属性是 `Option<Gd<T>>` 且需要在 setter 中触发副作用（如同步到子节点）时，使用 `#[var(get = ..., set = ...)]`：

```rust
#[derive(GodotClass)]
#[class(base = Node2D, tool)]
pub struct MapNode {
    #[var(get = get_tile_set, set = set_tile_set)]
    tile_set: Option<Gd<TileSet>>,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for MapNode {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            tile_set: None,
        }
    }
}

#[godot_api]
impl MapNode {
    #[func]
    fn get_tile_set(&self) -> Option<Gd<TileSet>> {
        self.tile_set.clone()  // Gd<T> 需要 clone
    }

    #[func]
    fn set_tile_set(&mut self, value: Option<Gd<TileSet>>) {
        self.tile_set = value;
        // 触发副作用：同步到子节点
        self.sync_tile_set_to_layers();
    }
}
```

**关键点**：
- getter 返回 `Option<Gd<T>>` 时需要 `.clone()`（`Gd<T>` 是引用计数，clone 开销小）
- setter 接收 `Option<Gd<TileSet>>`（ByOption 类型），可在其中触发副作用
- `#[func]` 标注的 getter/setter 在 GDScript 中表现为普通属性访问

### 可选类型属性

```rust
#[var(pub)]
tile_set: Option<Gd<TileSet>>,  // 编辑器中显示为空槽位

#[var(pub)]
display_material: Option<Gd<Material>>,
```

## 方法（#[func]）

### 私有性问题

**重要**：`#[func]` 方法在 Rust 中是**私有的**，不能跨模块直接调用。

```rust
#[godot_api]
impl MyClass {
    #[func]
    fn my_method(&mut self, arg: i32) {
        // 这个方法在 Rust 中是私有的！
        // 其他模块不能直接调用 obj.bind_mut().my_method(42)
    }
}
```

### 解决方案：公开包装方法

```rust
#[godot_api]
impl MyClass {
    #[func]
    fn my_method(&mut self, arg: i32) {
        // 内部实现
    }
}

// 单独的 impl 块放公开包装方法
impl MyClass {
    pub fn my_method_public(&mut self, arg: i32) {
        self.my_method(arg);
    }
}
```

### 可选参数

```rust
#[func]
fn draw_cell(&mut self, cell: Vector2i, #[opt(default = 1)] terrain: i32) {
    // terrain 参数在 GDScript 中可选，默认值为 1
}
```

## 信号定义

```rust
#[godot_api]
impl MyClass {
    #[signal]
    fn changed();

    #[signal]
    fn world_tiles_changed(changed: Array<Variant>);
}
```

## 虚方法重写（Virtual Methods）

**关键**：GDScript 中以 `_` 开头的虚方法（如 `_update_cells`、`_ready`、`_process`），在 gdext 中需要去掉下划线前缀，并在 `impl ITrait for MyClass` 块中重写，而不是在 `#[godot_api] impl MyClass` 块中定义为 `#[func]`。

```rust
// ❌ 错误：定义为 #[func] 方法，引擎不会自动调用
#[godot_api]
impl TileMapDual {
    #[func]
    fn _update_cells(&mut self, coords: Array<Vector2i>, _forced_cleanup: bool) {
        // 这只是一个普通方法，引擎不会在格子变化时调用它！
    }
}

// ✅ 正确：在 ITileMapLayer trait 实现中重写（方法名无下划线前缀）
#[godot_api]
impl ITileMapLayer for TileMapDual {
    fn update_cells(&mut self, coords: Array<Vector2i>, _forced_cleanup: bool) {
        // 引擎在格子变化时自动调用此方法
    }
}
```

**gdext 虚方法命名规则**：GDScript `_method_name` → gdext `method_name`（去掉下划线前缀）

**易错点**：`#[func]` 方法定义的是自定义方法（向 Godot 暴露但不自动调用），虚方法必须在 trait impl 块中重写才能被引擎自动调用。
