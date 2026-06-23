# 导入与最佳实践

## 常用导入

```rust
// 基础
use godot::prelude::*;
use godot::obj::{NewGd, EngineEnum};

// 内置类型
use godot::builtin::{Array, Variant, VarArray, VarDictionary, Vector2, Vector2i, GString};

// 引擎类
use godot::classes::{
    Engine, Input, Node, Node2D, Sprite2D, Control, CanvasItem,
    TileMapLayer, TileSet, TileSetSource, TileSetAtlasSource, TileData,
    Material, Shader, ShaderMaterial, Resource, RefCounted,
    ITileMapLayer, INode2D, INode, IResource, IRefCounted, ISprite2D, IControl,
};

// 枚举
use godot::classes::tile_set::CellNeighbor;
use godot::classes::object::ConnectFlags;
```

## 最佳实践

1. **每个类一个文件**：遵循项目规范，耦合度低的类放独立文件，文件开头写功能注释
2. **`#[var(pub)]` 优先**：需要跨模块访问的属性用 `#[var(pub)]` 避免 deprecated 警告
3. **公开包装方法**：`#[func]` 方法是私有的，跨模块调用需添加 `*_public()` 包装方法
4. **先克隆再借用**：在 `self.base_mut()` 之前克隆需要的 `Gd<T>`、`Option<Gd<T>>`、`Vector2` 等值
5. **用 call() 绕过类型不匹配**：`set_tile_set`、`set_material` 等 ByValue/ByOption 不匹配的方法用 `call()` 动态调用
6. **逐文件编译检查**：每写完一个文件就 `cargo check`，不要堆到最后一起检查
7. **内联 Shader**：用 `Shader::new_gd()` + `set_code()` 替代 `preload(.tres)`，避免资源文件依赖
8. **基类类型避免循环依赖**：跨模块引用时用基类 `Gd<BaseClass>` + `call()` 访问子类特有方法

## gdext 源码位置

当需要查找 gdext API 时，按以下顺序查找：

1. 先在本 SKILL 文档中查找
2. 再到 gdext 源码中阅读：`~/.cargo/registry/src/*/godot-core-0.5.3/`
3. 编译后的生成代码在：`target/debug/build/godot-core-*/out/`

### 常用源码路径

| 内容 | 路径 |
|------|------|
| 类定义（如 TileMapLayer） | `out/classes/tile_map_layer.rs` |
| 虚方法映射 | `out/virtuals.rs` |
| 方法签名 | `out/classes/*.rs` |
