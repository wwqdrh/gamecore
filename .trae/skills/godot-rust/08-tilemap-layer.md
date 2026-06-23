# TileMapLayer API

## set_cell 构建器模式

```rust
// set_cell 只接受坐标，需要用 set_cell_ex 构建器设置完整信息
self.base_mut()
    .set_cell_ex(cell)
    .source_id(sid)
    .atlas_coords(tile)
    .done();
```

**易错点**：`set_cell(cell)` 只接受 1 个参数（坐标），设置 source_id 和 atlas_coords 必须用 `set_cell_ex` 构建器。

## 获取格子信息

```rust
let sid = world.get_cell_source_id(cell);        // i32，-1 表示空
let tile = world.get_cell_atlas_coords(cell);     // Vector2i
let data = world.get_cell_tile_data(cell);        // Option<Gd<TileData>>
let used_cells = world.get_used_cells();          // Array<Vector2i>
```

## get_neighbor_cell

```rust
use godot::classes::tile_set::CellNeighbor;
use godot::obj::EngineEnum;

let neighbor = CellNeighbor::CELL_NEIGHBOR_TOP_SIDE;
let neighbor_cell = self.base().get_neighbor_cell(cell, neighbor);

// 从 ord 值创建 CellNeighbor
let ord: i32 = variant.to();
let neighbor = CellNeighbor::from_ord(ord);
```

## get_children()

```rust
// get_children() 不接受参数（没有 include_internal 参数）
let children = self.base().get_children();
for child in children.iter_shared() {
    // 处理子节点
}
```

**易错点**：gdext 的 `get_children()` 不接受 `include_internal` 参数，与 GDScript 不同。

## 虚方法重写

**关键**：GDScript 中以 `_` 开头的虚方法，在 gdext 中需要去掉下划线前缀，并在 `impl ITileMapLayer for MyClass` 块中重写。

```rust
// ❌ 错误：定义为 #[func] 方法，引擎不会自动调用
#[godot_api]
impl TileMapDual {
    #[func]
    fn _update_cells(&mut self, coords: Array<Vector2i>, _forced_cleanup: bool) {
        // 引擎不会在格子变化时调用它！
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

### TileMapLayer 虚方法列表

| GDScript | gdext Rust | 说明 |
|----------|-----------|------|
| `_update_cells(coords, forced_cleanup)` | `update_cells(coords, forced_cleanup)` | 格子变化时调用 |
| `_tile_data_runtime_update(coords, tile_data)` | `tile_data_runtime_update(coords, tile_data)` | 运行时更新 TileData |
| `_use_tile_data_runtime_update(coords)` | `use_tile_data_runtime_update(coords)` | 是否使用运行时 TileData 更新 |

**易错点**：`#[func]` 方法定义的是自定义方法（向 Godot 暴露但不自动调用），虚方法必须在 trait impl 块中重写才能被引擎自动调用。

## update_cells 虚方法与编辑器实时更新

**重要**：`update_cells` 虚方法在 gdext 0.5.3 中**可以正常被引擎调用**（编辑器放置图块、undo/redo 时触发）。如果编辑器中无法实时显示，通常是以下原因：

1. **初始化时序问题**：`call_deferred` 在子节点完全 ready 前执行，导致初始更新失败
2. **子节点未使用 INTERNAL_MODE**：动态创建的子节点被保存到场景文件，重新加载时出现重复节点

**解决方案**：

1. 使用标志位 + `process` 实现下一帧初始更新（等价于 `await get_tree().process_frame`）
2. 动态创建的子节点使用 `INTERNAL_MODE_BACK` 避免保存到场景文件
3. 作为 fallback，编辑器模式下可启用双检查模式主动检测变化

```rust
// 双检查模式作为 fallback（不依赖 update_cells）
let is_editor = Engine::singleton().is_editor_hint();
let should_double_check = self.godot_4_3_compatibility || is_editor;

if should_double_check && self.base().get_tile_set().is_some() {
    let mut current_cells = TileCache::new();
    let used_cells = self.base().get_used_cells();
    current_cells.bind_mut().update_edited_public(self.base_mut().clone(), used_cells);
    updated_cells = current_cells.bind().xor_public(self.get_cached_cells());
}
```

**关键点**：
- `update_cells` 虚方法在编辑器中正常工作，传入变化的格子坐标
- 双检查模式作为 fallback，通过 `TileCache::xor` 主动发现变化
- 初始化时序和 INTERNAL_MODE 是编辑器实时更新的关键

## 编辑器实时更新模式

GDScript 中 `await get_tree().process_frame` 在 gdext 中无法直接使用。替代方案：

```rust
// 方案：使用标志位在下一帧 process 中执行初始更新
struct MyLayer {
    needs_initial_update: bool,
    // ...
}

impl ITileMapLayer for MyLayer {
    fn ready(&mut self) {
        // ...
        if Engine::singleton().is_editor_hint() {
            self.base_mut().set_process(true);
            self.needs_initial_update = true;  // 标记需要初始更新
        } else {
            self.base_mut().set_process(false);
            self.base_mut().call_deferred("_changed", &[]);  // 运行时直接延迟调用
        }
    }

    fn process(&mut self, delta: f64) {
        // 下一帧执行初始更新（对应 await get_tree().process_frame）
        if self.needs_initial_update {
            self.needs_initial_update = false;
            self.base_mut().call_deferred("_changed", &[]);
            return;
        }
        // ... 正常的 process 逻辑
    }
}
```

**关键点**：
- `call_deferred` 在当前帧末尾执行，可能过早（子节点未完全初始化）
- 使用标志位 + `process` 可以确保在下一帧执行，等价于 `await get_tree().process_frame`
- 编辑器模式下 `set_process(true)` 是必须的，否则 `process` 不会被调用

## 属性复制模式

从父节点复制属性到子节点的常见模式：

```rust
fn update_properties(&mut self, mut parent: Gd<TileMapLayer>) {
    let mut base = self.base_mut();
    // Rendering
    base.set_y_sort_origin(parent.get_y_sort_origin());
    base.set_x_draw_order_reversed(parent.is_x_draw_order_reversed());
    base.set_rendering_quadrant_size(parent.get_rendering_quadrant_size());
    // Physics
    base.set_collision_enabled(parent.is_collision_enabled());
    base.set_use_kinematic_bodies(parent.is_using_kinematic_bodies());
    // ...
}
```

## API 名称差异

| GDScript | gdext Rust | 说明 |
|----------|-----------|------|
| `use_kinematic_bodies` | `is_using_kinematic_bodies()` | getter 前缀 is_ |
| `show_behind_parent` | `is_draw_behind_parent_enabled()` | 名称不同 |
| `use_parent_material` | `get_use_parent_material()` | getter 用 get_ 而非 is_ |
| `top_level` | `is_set_as_top_level()` | 名称不同 |
