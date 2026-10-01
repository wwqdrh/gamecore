# UI：GML 标记语言、列表控件、弹窗、主题

对应 Rust 源码：`rust/src/ui/`（parser / builder / gdui_builder / ui_theme / 各控件）
可运行示例：`example/ui/scene_gallery.gd`、`example/ui/scene_main_bean.gd`、`example/ui/sample_ui.gml`

## 编辑器工作流：纯文本编辑 .gml → 自动生成 .gml.tscn（重要）

`.gml` 是**纯文本源码**（注册在 `docks/filesystem/textfile_extensions`，双击用文本编辑器打开编辑）。
编辑器插件 `addons/gamecore/core.gd` 每秒轮询扫描项目内全部 `.gml`，调用 Rust `GdUiBuilder.build_scene_file`
构建并保存为**同名 `.gml.tscn`**（真实场景文件，含完整节点树，可运行/可实例化/可挂载）：

- `.gml` 保存后 1 秒内对应 `.gml.tscn` **自动重新生成**；若该 tscn 正打开在场景编辑器中会自动 reload
- **GML 是唯一源码，`.gml.tscn` 是生成产物**——不要手改 tscn，改动会在下次 gml 保存时被覆盖
- GML 语法错误时只在输出面板报错（`[GmlAutoGen] xxx 生成失败: ...`），保留旧 tscn 不中断
- 删除 `.gml` 前手动删掉对应 `.gml.tscn`（插件不做自动删除，防止误删手动挂载的引用）
- 运行时（非编辑器）加载 GML 仍走 `GdUiBuilder.parse_file` / `GdGmlScene.load_gml`

需要把 GML 转成场景资源（运行中动态构建）时：

```gdscript
var builder = GdUiBuilder.new()
var scene: PackedScene = builder.build_scene_file("res://ui/shop.gml")   # 失败返回 null
var scene2: PackedScene = builder.build_scene_string("<ui>...</ui>")
if scene == null:
    print(builder.last_error())   # 最近一次构建失败的错误信息
```

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
var scene: PackedScene = builder.build_scene_file(path)  # -> PackedScene（编辑器预览用）
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

## 多 GML 组合实战（example/ui/task/ 宗门任务面板）

复杂界面按区块拆成多个 `.gml`，用 **`<Gml>` 标签直接引用**组合成完整 UI，
控制器脚本只负责数据与信号。可运行示例 `example/ui/task/task_panel.tscn`，
验收脚本 `check_task_ui.gd`（无头跑：`godot --headless --path . -s res://example/ui/task/check_task_ui.gd`）。

拆分方式：`task_panel.gml` 骨架布局，各功能区块独立成文件，条目模板再单独一个文件。

### `<Gml>` 标签：引用另一个 gml 文件（构建期嫁接，推荐）

```xml
<ui theme="cartoon">
  <Panel name="WindowPanel" class="window-bg" anchor="full">
    <VBoxContainer>
      <Gml src="task_topbar.gml" />                              <!-- 相对路径 -->
      <Gml src="res://ui/shop/shop_list.gml" />                  <!-- 也可用 res:// 绝对路径 -->
      <Gml src="task_list.gml" size_flags_vertical="expand_fill" /> <!-- 其余属性覆盖式应用到被引用根节点 -->
    </VBoxContainer>
  </Panel>
</ui>
```

- 构建期解析 src 指向的文件、构建子树、**自动剥掉 UiRoot 包装层**并嫁接到引用位置
  （`parse_file` 手动组合时才需要自己剥壳，见下方坑 1）
- src 支持相对路径（基于引用方文件所在目录）与 `res://` 绝对路径；同一文件可引用多次（各自独立实例）
- Gml 标签上的其余属性（`name`/`anchor`/`margin`/`size_flags_*`/`class`/`on_xxx`）
  会覆盖式应用到被引用文件的根节点上
- 主题与样式继承：子文件继承引用方的主题变量与 `<style>` class；子文件自己的
  `theme` 属性 / `<theme>` 块 / `<style>` 块优先
- 信号（`on_pressed` 等）照常写在子文件里，由控制器的 `connect_signals` / `allbind_signal` 统一连接
- 循环引用（A 引 B、B 引 A、自引用）构建期报错，不会卡死

### 列表模板注入：条目 gml 复用为 UIVList slot 模板

直接在列表 gml 内用 `<Gml>` 引用条目文件，构建期即完成模板注入：

```xml
<ScrollContainer name="TaskScroll" size_flags_vertical="expand_fill">
  <UIVList name="TaskList" size_flags_horizontal="expand_fill">
    <Gml src="task_item.gml" />
  </UIVList>
</ScrollContainer>
```

### 数据驱动刷新

条目 GML 内用 `{{key}}` 占位（`text="{{title}}"`），`update(Array[Dictionary])` 按字段填充；
未被模板使用的字段自动存为条目根节点 `__item_data` meta，回调里 `get_meta("__item_data")` 取回整行数据。
列表条目内的按钮信号用 `allbind_signal(条目内NodePath, 信号名, 回调)` 批量连接
（duplicate 出的实例不会继承 connect_signals 的连接，每次 update 后重新 allbind）。

```gdscript
list.update(data, false)
list.allbind_signal("ItemMargin/ItemRow/ItemBtn", "pressed", _on_item_btn)
```

### 页签：直接复用 TabContainer/Tab

```xml
<TabContainer name="TaskTabs" anchor="full" tabs_visible="true" current_tab="0">
  <Tab title="日常"><Gml src="task_list.gml" /></Tab>
  <Tab title="主线"><Gml src="task_list.gml" /></Tab>
</TabContainer>
```

- Tab 的 `title` 即页签名（也是节点名，可用 `find_node("日常")` 定位页面）
- 切换是原生行为，控制器只需 `tabs.tab_changed.connect(_on_tab_changed)`
- 当前引擎 TabContainer 仅支持顶部/底部页签（`tabs_position` 无 LEFT/RIGHT）；
  设计图样式的左侧竖排页签需用自定义按钮列表 + 数据驱动高亮实现

### 三个必踩的坑（重要）

1. **`parse_string/parse_file` 返回 UiRoot 包装层**：真正的根控件是它的子节点。
   包装层是普通 Control、无尺寸语义——直接当 slot 子节点或列表模板用会导致
   锚点失效、条目高度塌陷为 0；内部未命名节点是 `@Class@id` 形式，NodePath 无法命中。
   手动组合场景一律剥壳（`wrapper.get_child(0)`）；`<Gml>` 标签已内置剥壳，无需处理。
   结构节点在 GML 中务必**显式命名**。
2. **`UIVList/UIGrid.update(data, force)` 的 force 语义**：`force=true` 会把内部 count
   固定为数据长度——首次调用时（列表为空）两个增删分支都不命中，**一个条目都不会创建**。
   纯数据驱动列表保持 `count<=0`（GML 不写 count 属性），统一用 `update(data, false)`。
3. **allbind_signal 的 path 是相对条目根节点的完整 NodePath**（如 `ItemMargin/ItemRow/ItemBtn`），
   路径上每个节点都要显式命名；未命名节点在 duplicate/add 后变成 `_Class_N`，路径必失效。
