## 设计目标

设计一套类似 HTML 的声明式语言，用于快速描述 Godot 游戏的 UI 结构、布局与样式。该语言将解析并生成 Godot 的 `Control` 节点树，可输出为 `.tscn` 场景文件或直接由 GDScript 运行时构建。目标是降低 UI 编写的冗余代码，提高可读性与维护性。

## 语法设计（伪代码示例）

```html
<ui theme="default">
  <style>
    .button-primary {
        background: #2e7d32;
        color: white;
        padding: 8 16;
    }
    .panel-card {
        margin: 10;
        bg_color: #333333;
        border_radius: 4;
    }
  </style>

  <VBoxContainer anchor="full" margin="12" class="main-container">
    <Label text="欢迎, 勇者" font_size="24" align="center" />

    <HBoxContainer margin="0 0 20 0">
      <Button text="开始游戏" class="button-primary" on_pressed="_on_start_pressed" />
      <Button text="设置" class="button-primary" on_pressed="_on_settings_pressed" />
    </HBoxContainer>

    <Panel class="panel-card" size="300,200">
      <MarginContainer margin="all:15">
        <RichTextLabel bbcode="[center]最新消息[/center]" />
      </MarginContainer>
    </Panel>

    <TextureRect texture="res://ui/logo.png" stretch_mode="keep_aspect" />
  </VBoxContainer>
</ui>
```

## 语言要素说明

| 要素 | 作用 |
|------|------|
| `<ui>` | 根元素，可声明全局主题、导入资源 |
| `<style>` | 定义类似 CSS 的类样式，映射到 Godot 的 `Theme` / 样式盒 |
| 容器标签 (`VBoxContainer` / `HBoxContainer` / `GridContainer` 等) | 对应 Godot 的布局容器，自动管理子节点排列 |
| 控件标签 (`Button`, `Label`, `Panel` 等) | 直接生成 Godot 内置控件 |
| 属性 | 支持 Godot 常用属性（`text`, `margin`, `size`, `anchor` 等）以及布局特有属性 |
| `class` | 应用 `<style>` 中定义的类样式 |
| `on_pressed` / `on_mouse_entered` 等 | 连接信号到脚本方法 |

## 转换伪代码（解析器核心逻辑）

```pseudocode
FUNCTION parse_ui_markup(ast_node, parent_godot_node):
    FOR EACH node in ast_node.children:
        IF node.tag == "style":
            theme = parse_style(node)   // 构建一个 Godot Theme 资源
        ELSE:
            // 1. 根据标签名实例化 Godot 控件
            control = instantiate_godot_control(node.tag)

            // 2. 设置直接属性
            FOR EACH attr IN node.attributes:
                IF attr.name == "class" AND theme EXISTS:
                    apply_theme_style(control, theme, attr.value)
                ELSE IF attr.name.startswith("on_"):
                    connect_signal(control, attr.name[3:], attr.value)
                ELSE:
                    control.set(attr.name, attr.value)

            // 3. 应用内联样式/边距
            IF node.has_property("margin"):
                apply_margin(control, node.margin)
            IF node.has_property("padding"):
                apply_padding(control, node.padding)   // 通过 MarginContainer 实现

            // 4. 递归处理子节点
            parse_ui_markup(node, control)

            // 5. 添加到父容器
            parent_godot_node.add_child(control)

FUNCTION compile_to_tscn(ast_root):
    scene = instantiate_Control()
    parse_ui_markup(ast_root, scene)
    return PackedScene(scene).save("output.tscn")
```

## 运行时集成（GDScript 示例）

```gdscript
# 加载并实例化 UI 文件（类似加载场景）
var ui_root = preload("res://ui/main_menu.tscn").instantiate()
add_child(ui_root)

# 或动态解析字符串
var ui_builder = UIBuilder.new()
var dynamic_ui = ui_builder.parse_string("
    <VBoxContainer>
        <Label text='动态创建' />
        <Button text='点我' on_pressed='_on_click' />
    </VBoxContainer>
")
add_child(dynamic_ui)
```

## 用途与优势

1. **快速原型**：用几行标签描述复杂布局，避免手写数十行 `add_child` 和 `set_anchor`。
2. **可读性**：树形结构与 HTML 相似，设计师或策划也能理解基本结构。
3. **样式复用**：`<style>` 块统一管理主题，符合现代 UI 工作流。
4. **信号绑定**：直接在标签上挂接事件，减少手动 `connect` 代码。
5. **跨场景共享**：可将 UI 片段编译为 `.tscn` 文件，无缝融入 Godot 资源管线。
6. **热重载支持**：解析器可监听 `.gml` 文件变化并实时重建 UI，加速迭代。
