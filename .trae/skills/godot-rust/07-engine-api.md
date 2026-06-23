# 引擎 API

## 版本检测

### Engine.get_version_info()

```rust
fn is_godot_below_4_4() -> bool {
    let version = godot::classes::Engine::singleton().get_version_info();
    // get() 返回 Option<Variant>，用 map + unwrap_or
    let major: i32 = version.get("major").map(|v| v.to()).unwrap_or(4);
    let minor: i32 = version.get("minor").map(|v| v.to()).unwrap_or(0);
    major < 4 || (major == 4 && minor < 4)
}
```

**易错点**：`version.get("major")` 返回 `Option<Variant>`，不能 `.unwrap_or(&Variant::nil())`（类型不匹配），要用 `.map(|v| v.to::<i32>()).unwrap_or(4)`。

## 编辑器检测

```rust
if godot::classes::Engine::singleton().is_editor_hint() {
    // 在编辑器中运行
    self.base_mut().set_process(true);
} else {
    // 在游戏中运行
    self.base_mut().set_process(false);
}
```

## 输入处理

```rust
use godot::classes::Input;

let input = Input::singleton();
if input.is_action_pressed("left_click") {
    // 左键按下
}
if input.is_action_pressed("right_click") {
    // 右键按下
}
```

## 节点树操作

```rust
// 添加子节点
self.base_mut().add_child(&child_node);

// 获取子节点
let children = self.base().get_children();
for child in children.iter_shared() {
    // 处理子节点
}

// 队列释放
child_node.queue_free();  // 需要 &mut self

// 获取本地鼠标位置
let local_mouse = self.base().get_local_mouse_position();

// 坐标转换
let cell = self.base().local_to_map(local_mouse);
let local_pos = self.base().map_to_local(cell);
let global_pos = self.base().to_global(local_pos);
```

## 内部子节点（InternalMode）

在编辑器中，`add_child` 添加的子节点默认会被保存到场景文件，重新加载场景时会出现重复节点。使用 `INTERNAL_MODE_BACK` 可以避免此问题。

```rust
use godot::classes::node::InternalMode;

// 添加内部子节点（不会被保存到场景文件，也不会出现在编辑器节点树中）
self.base_mut().add_child_ex(&child_node)
    .internal(InternalMode::BACK)
    .done();

// 获取包含内部子节点的子节点列表
let children = self.base().get_children_ex()
    .include_internal(true)
    .done();
```

**易错点**：
- `get_children()` 默认不包含内部子节点，需要用 `get_children_ex().include_internal(true).done()` 获取
- `InternalMode::BACK` 将子节点添加到子节点列表末尾，`InternalMode::FRONT` 添加到开头
- 内部子节点不会随父节点被复制（duplicate），适合用于工具脚本中动态创建的辅助节点

## set_process 控制

```rust
// 启用/禁用 _process 回调
self.base_mut().set_process(true);   // 启用
self.base_mut().set_process(false);  // 禁用
```
