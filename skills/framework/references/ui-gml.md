# UI：GML 标记语言、列表控件、弹窗、主题

对应 Rust 源码：`rust/src/ui/`（parser / builder / gdui_builder / ui_theme / 各控件）
可运行示例：`example/ui/scene_gallery.gd`、`example/ui/scene_main_bean.gd`

## GML 是什么

类 HTML 的声明式 UI 描述语言：解析器 → AST → Godot Control 节点树。
标签名即控件，属性即节点属性，支持主题变量与 `{{key}}` 模板插值。

```gdscript
# scene_gallery.gd —— 页面控制器继承 GdGmlScene
extends GdGmlScene

var UI = """
<ui theme="cartoon">
  <VList>
    <Label text="图鉴" />
    <Button name="BackBtn" text="返回" on_pressed="close:Self" />
  </VList>
</ui>
"""

func _ready():
    load_from_string(UI)          # 或 load_gml("res://ui/gallery.gml") 加载文件
```

常用 GML 写法（摘自示例）：

```xml
<Button name="GalleryBtn" on_pressed="show:GalleryPopup" />   <!-- 信号绑定到弹出层 -->
<PopupPanel popup_title="Gallery" close_on_overlay="true">...</PopupPanel>
<UIGrid data="weapon_data" count="6" columns="3">              <!-- 数据驱动列表 -->
    <Label text="{{name}}" />   <!-- 模板插值每条数据 -->
</UIGrid>
```

数据绑定语法：`data="bean:scene_main:equip_data"`（bean_id:字段），配合 GdBean（见 state-data.md）。

## GdUiBuilder — 构建 API

```gdscript
var builder = GdUiBuilder.new()

var root: Control = builder.parse_string(markup)   # 解析字符串 -> Control 树
var root2: Control = builder.parse_file("res://ui/shop.gml")
builder.connect_signals(root, self)                # 按 on_xxx 属性连接信号到 target
builder.validate(markup)                           # 校验 -> 错误信息（空串为合法）
builder.set_theme("cartoon")                       # 内置主题
builder.set_theme_var("primary", Color.RED)        # 主题变量
builder.get_builtin_themes()                       # -> PackedStringArray
```

## GdGmlScene — GML 场景节点

```gdscript
extends GdGmlScene

load_gml("res://ui/gallery.gml")     # 加载文件
load_from_string(UI)                 # 加载字符串
connect_signals(self)                # 连接 on_xxx 信号
get_content()                        # 内容根节点
find_node("GalleryBtn")              # 按名字查找控件
clear_content()
is_loaded()
apply_theme("cartoon")               # 换主题
refresh_anchors()                    # 视口变化后刷新锚点
```

## 列表控件

| 控件 | 方向 | 特有方法 |
|---|---|---|
| `UIHList` | 横向 | `set_width_times(idx, n)` |
| `UIVList` | 纵向 | `set_height_times(idx, n)` |
| `UIGrid` / `UIGridList` | 网格 | `columns` 属性 |

```gdscript
var list: UIGrid = find_node("WeaponGrid")
list.update(data_array, true)             # 全量刷新 (Array[Dictionary])
list.patch_item(2, {"name": "改名"})       # 增量更新单条
list.update_all({"name": "新名"})          # 所有条目更新同字段
list.get_at(id)                            # -> Control（条目节点）
list.get_meta_value(idx, key, default)     # 读条目元数据
list.allbind_signal("pressed", "pressed", _on_item_pressed)  # 批量绑条目信号
list.initial()
```

## GdPopupPanel — 弹窗面板

内置模态遮罩、标题栏+关闭按钮、内容区域：

```gdscript
var popup: GdPopupPanel = find_node("GalleryPopup")
popup.set_popup_title_text("图鉴")
popup.add_content_child(some_control)     # 动态塞内容
popup.show_popup()
popup.hide_popup()
popup.toggle_popup()
popup.is_popup_visible()
```

## 其他控件

- `GdUIDrawer`：抽屉面板，从屏幕边缘滑入/滑出。
- `GdUINavMenu`：多级级联导航菜单。
- `GdUITooltip`：鼠标跟随提示框。

## 推荐架构

页面控制器 `extends GdGmlScene`（GML 内嵌字符串或文件）+ 数据源继承 `GdBean`，
属性变更 `emit([keys])` 自动刷新绑定 UI——业务代码只操作数据，不碰控件。
参考 `example/ui/scene_main_bean.gd`。
