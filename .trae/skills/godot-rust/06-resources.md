# 资源创建与内联 Shader

## 内联创建 ShaderMaterial

```rust
use godot::classes::{Shader, ShaderMaterial};
use godot::obj::NewGd;

const SHADER_CODE: &str = r#"
shader_type canvas_item;
void fragment() {
    COLOR.a = 0.0;
}
"#;

let mut shader = Shader::new_gd();
shader.set_code(SHADER_CODE);
let mut mat = ShaderMaterial::new_gd();
mat.set_shader(&shader);
```

**用途**：替代 GDScript 的 `preload("res://path/to/material.tres")`，避免资源文件依赖。

## 从 init_fn 创建对象

```rust
// 创建自定义 Resource
let cache = Gd::<TileCache>::from_init_fn(|b| {
    <TileCache as IResource>::init(b)
});

// 创建自定义 Node
let display = Gd::<Display>::from_init_fn(|base| {
    <Display as godot::classes::INode2D>::init(base)
});
```

**易错点**：`init` 是 trait 方法，必须用完全限定语法 `<Type as ITrait>::init(base)`。

## 创建内置类型

```rust
use godot::obj::NewGd;

// 所有内置类型都可以用 new_gd() 创建
let mut mat = ShaderMaterial::new_gd();
let mut shader = Shader::new_gd();
let mut tile_set = TileSet::new_gd();
```

## 克隆 Gd<T>

```rust
// Gd<T> 实现了 Clone，克隆是廉价的（引用计数）
let cloned = original.clone();

// 克隆 Option<Gd<T>>
let cloned_opt = self.tileset_watcher.clone(); // Option<Gd<TileSetWatcher>>
```
