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

### 无包装层运行（推荐）：`<ui script>` 根挂载，直开 tscn 即完整应用

gml 根声明 `<ui script="面板控制器.gd">`（脚本 extends gml 根元素的实际类型，如根是
`<Panel>` 则 `extends Panel`），构建期脚本自动挂到根元素——**不要再用 GdGmlScene
壳场景包装面板**，`bag_panel.gml.tscn` 直开 F6 运行即完整可用：

- 信号绑定（`@pressed` / `@s_click_item` / …）**挂树自动连接**：GDCORE 监听
  `node_added`，识别带 `__gml_root` 标记的 gml 树根后自动连接整树（运行时自举，
  编辑器下跳过）；子文件 `<ui script>` 的回调按就近解析落在各自脚本。
  兜底：钩子在首帧才连上 `node_added`，F6/主场景在首帧前已挂树会错过——
  钩子连接成功后立即对现有树做一次补扫，两种时序均覆盖
- 回调就近解析：声明节点沿父链向上找最近"挂脚本且实现该方法"的节点，区块回调落在
  区块脚本，场景级回调（如关闭按钮）回退面板根脚本——控制器零绑定代码
- 注意：控制器脚本类型必须与 gml 根元素匹配（根 `<Panel>` → `extends Panel`），
  类型不符会在脚本挂载时报错
- GdGmlScene 壳场景仅保留给**动态加载**场景（运行中 `load_gml` 换页面 / bean 响应式绑定）

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
<ui>
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
builder.set_theme_var("primary", "#ff0000")        # 主题变量（<style> 值中 $primary 引用）
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

- `GdUIDrawer`：抽屉面板，从屏幕边缘滑入/滑出（`<Drawer direction="left" slide_width="50%" drawer_title="任务">`，open/close/toggle，close_on_overlay 点击遮罩关闭）。
- `GdUINavMenu`：多级级联导航菜单。
- `GdUITooltip`：鼠标跟随提示框。

## GdUIModal — 模态弹窗 + GdUIManager 跨组件调用

`<Modal>`：全屏遮罩 + 内容区，遮罩淡入 + 内容缩放动画（居中弹窗版 Drawer）：

```xml
<!-- 组件声明（在自己文件里声明 ui_id 唯一身份，零外部引用） -->
<Modal name="BagModal" ui_id="BagModal" anchor="full"
       content_margin="56" close_on_overlay="true">
  <Gml src="bag_panel.gml" />   <!-- 内容子节点进入 ContentArea（content_margin 四周留白） -->
</Modal>
```

- `open()/close()/toggle()/is_modal_open()`；信号 `s_modal_opened` / `s_modal_closed`
- `content_margin`：内容区四周像素留白；留白区点击穿透到遮罩 → 关闭
- **内容关闭联动**：内容根声明 `signal s_close_requested` 并在自身关闭按钮里 emit
  （独立 F6 仍走自身 hide），Modal 自动连接并整体 close——现成面板无需改动即可弹窗化
- 编辑器内静态预览展开布局（遮罩透明），运行期初始隐藏

**跨组件调用（GdUIManager 统一 UI 管理层）**：组件分属不同目录时零 import、
零 find_child 翻树，全部按 ui_id 查注册表（组合根把组件装配进同一棵树后自动注册）：

```xml
<!-- GML 内部动作：运行期按下时按 id 延迟解析（本地树节点名优先 → ui_id 注册表） -->
<Panel @pressed="show:BagModal" />
```

```gdscript
# GD 侧按需调用
GdUIManager.find_ui("BagModal").open()
GdUIManager.has_ui("TaskDrawer") / get_ui_ids()
```

范本：`example/demo/xiuxian/`（main.gml 组合根装配 mainhud + task 抽屉 + 三个 Modal，
mainhud 功能按钮 `show:BagModal`/`show:StoreModal`、头像 `show:ProfileModal`），
验收 `check_demo_ui.gd`。

## 推荐架构

**静态面板（推荐）**：gml 根声明 `<ui script="面板.gd">` 挂控制器，直开 `.gml.tscn`
运行，区块拆分 `<Gml>` 引用 + 各自 `<ui script>` 回调就近绑定，数据源按需继承 `GdBean`。
**动态页面**（运行中换页 / bean 响应式绑定）：页面控制器 `extends GdGmlScene`
（GML 内嵌字符串或文件）+ 数据源继承 `GdBean`，属性变更 `emit([keys])` 自动刷新绑定
UI——业务代码只操作数据，不碰控件。参考 `example/ui/scene_main_bean.gd`。

## 多 GML 组合实战（example/ui/task/ 宗门任务面板）

复杂界面按区块拆成多个 `.gml`，用 **`<Gml>` 标签直接引用**组合成完整 UI，
控制器脚本只负责数据与信号。可运行示例 `example/ui/task/task_panel.tscn`，
验收脚本 `check_task_ui.gd`（无头跑：`godot --headless --path . -s res://example/ui/task/check_task_ui.gd`）。

拆分方式：`task_panel.gml` 骨架布局，各功能区块独立成文件，条目模板再单独一个文件。
运行入口推荐无包装层模式：骨架根 `<ui script="task_panel.gd">` 挂控制器、直开生成的
tscn 运行（参考 `example/ui/bag/bag_panel.gml`）；task/profile 示例中的 GdGmlScene
壳场景为历史写法，仍兼容。

### `<Gml>` 标签：引用另一个 gml 文件（构建期嫁接，推荐）

```xml
<ui>
  <Panel name="WindowPanel" class="window-bg" anchor="full">
    <VBoxContainer>
      <Gml src="task_topbar.gml" />                              <!-- 相对路径 -->
      <Gml src="res://ui/shop/shop_list.gml" />                  <!-- 也可用 res:// 绝对路径 -->
      <Gml src="task_list.gml" size_flags_vertical="expand_fill" /> <!-- 其余属性覆盖式应用到被引用根节点 -->
    </VBoxContainer>
  </Panel>
</ui>
```

- 构建期解析 src 指向的文件、构建子树并嫁接到引用位置（无包装层，嫁接的就是
  被引用文件的根元素节点；`parse_string/parse_file` 返回的也是根元素节点本身）
- src 支持相对路径（基于引用方文件所在目录）与 `res://` 绝对路径；同一文件可引用多次（各自独立实例）
- Gml 标签上的其余属性（`name`/`anchor`/`margin`/`size_flags_*`/`stretch_ratio`/`class`/`on_xxx`）
  会覆盖式应用到被引用文件的根节点上
- **容器内多列/多行比例分配用 `stretch_ratio`**（配合 `size_flags_*="expand_fill"`）：
  `<Gml src="a.gml" size_flags_horizontal="expand_fill" stretch_ratio="3" />`——
  构建期写入 `size_flags_stretch_ratio`，静态生效（编辑器直开 tscn 也正确）。
  **不要用 `custom_minimum_size="30%,0"` 做容器内列宽**：百分比 min-size 存为
  `__pct_min_size` meta 延迟解析，仅在 GdGmlScene 场景链路生效（相对场景根），
  直开 tscn / 非 GdGmlScene 场景下解析为 0 宽
- 主题与样式继承：子文件继承引用方的主题变量与 `<style>` class；子文件自己的
  `<theme>` 块 / `<style>` 块优先
- 信号（`on_pressed` 等）照常写在子文件里，由控制器的 `connect_signals` / `allbind_signal` 统一连接
- 循环引用（A 引 B、B 引 A、自引用）构建期报错，不会卡死

### 跨区块联动：GdState 临时状态总线（推荐，单向数据流）

多个 `<Gml>` 区块组合成一个面板时，区块间的联动（点选物品更新详情卡、切分类
过滤网格…）**不要让区块互相调用**（A 沿父链 find_child 找 B 再调它的方法——
强耦合、链路隐蔽、加区块要改所有旧代码），也**不要用 GDScript signal**——
用框架的 **GdState 临时状态总线**（Engine 单例 `GDSTATE`，Rust 侧
`state/state_store.rs`）：键值状态 + 按键监听，专为 UI 联动等**进程内临时状态**
设计，不落盘（与 GdBean 分工：GdBean 存可持久化游戏状态，随存档恢复）。
参考 `example/ui/bag/`：

```gdscript
# ---- 写入方（命令上行）：把用户操作写进总线，不碰兄弟区块 ----
# bag_categories.gd：点击分类
_state().set_state("bag.category", NAME_TO_CAT.get(String(btn.name), ""))
# bag_grid.gd：点选物品（__item_data 是列表写入条目的完整数据字典）
_state().set_state("bag.selected_item", item.get_meta("__item_data", {}))

func _state():
    return Engine.get_singleton("GDSTATE")
```

```gdscript
# ---- 监听方（状态下行）：只 watch 自己关心的键，各自更新自己 ----
# bag_grid.gd：watch 分类 -> 更新过滤；bag_detail.gd：watch 选中 -> 填详情
func _ready() -> void:
    _state().watch(KEY_CATEGORY, _on_category_changed)

func _on_category_changed(value: Variant) -> void:
    if value == null:
        return  # 注册即回调当前值，未初始化时为 nil，跳过等写入方
    grid.set_meta("__filter_value", str(value))
    grid.call("update", grid.get_meta("__script_data"), false)
```

GdState API 语义（与 GdBean.watch 对齐）：

- `set_state(key, value)`：写入并通知该键全部监听者；**值未变化（Variant 深比较）
  自动跳过通知**——写入方无需判重；`get_state`（未设置返回 nil）/ `has_state` /
  `erase_state`（移除值，不触发回调）
- `watch(key, callback)`：**注册即用当前值回调一次**（未设置为 nil）——UI 初始
  填充零样板；同一 Callable 重复注册自动去重；回调参数 1 参收 value，
  2+ 参收 `(value, key)`；`unwatch(key, callback)` 取消
- 键名约定 `"<面板>.<状态>"`（如 `bag.category`），跨面板不冲突

时序要点：

- 子区块 `_ready` 自底向上**先于面板根**执行：面板根 `_ready` 写初始状态
  （如 bag_panel 写 `bag.category`），此时监听已全部就位
- 网格上报"初始选中"用 `call_deferred`（等面板初始写入与详情卡监听就绪）

**红线：watch 回调内禁止同步再调用 GDSTATE 任何方法**（set_state/get_state/watch
…）——godot-rust 实例方法执行期间持有 GdCell 单借用，重入直接 panic
（GdBean/GdEventBus 同限制）。需要级联状态时用 `call_deferred` 包一层：

```gdscript
func _on_selection_changed(item: Variant) -> void:
    # 想在响应选中时联动写别的状态 -> 延迟一拍
    _state_other.call_deferred("set_state", "bag.xxx", item)
```

数据完整性归数据管：展示所需字段（name/desc/…）直接放进数据源（`<script>`
块条目 / 配置表 / bean），**消费方不做映射补全**（不要写 `ICON_INFO` 这类
"图标猜名字"表）。展示规则（locked→封印态、缺省→普通）才归展示视图所有。

好处：新增联动区块只需"watch 键"，零改现有代码；面板状态天然支持重开恢复
（GDSTATE 进程级存活，watch 注册即回调上次值）。

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

```gdscript
list.update(data, false)
```

### 信号绑定：`@信号名=方法名`（推荐，零控制器绑定代码）

GML 内直接声明信号绑定，挂树时由框架自动连接（GDCORE `node_added` 自举，见工作流
小节；GdGmlScene 动态加载路径兼容），无需在控制器里写 `connect_signals` / `allbind_signal`：

```xml
<!-- 静态节点：0 参方法 -->
<Button name="CloseBtn" @pressed="_on_close_pressed" />

<!-- 列表条目内按钮：1 参方法自动补绑发出按钮，可经 __item_data 读整行数据 -->
<Button name="ItemBtn" text="{{btn_text}}" @pressed="_on_task_action" />
```

- 任意信号均可：`@pressed` / `@text_changed` / `@toggled` …（等价于旧 `on_pressed` 写法）
- **非按钮控件的点击回退**：`@pressed` 声明在 Panel/Control 等没有原生 `pressed` 信号的
  控件上时，框架自动监听 `gui_input` 模拟 Button（左键按下触发，回调补绑语义不变）——
  可点击面板直接写 `<Panel @pressed="_on_x" mouse_default_cursor_shape="pointing_hand">`，
  无需包一层 Button（注意面板内子节点不能是 mouse_filter=STOP 的满铺节点，会挡点击）
- **条目内声明**会随 slot 复制自动存活：构建期条目由 `connect_signals` 走树连接；
  运行时 `update()` 重建条目后由列表 `bind_events` 自动重连——控制器全程零绑定代码
- 参数自动匹配：方法要求的参数多于信号提供时，补绑发出节点
  （`_on_task_action(btn)` 收到按钮；`_on_close()` 0 参照常）
- 方法不存在时构建期告警但不崩溃；`on_xxx` 旧写法仍兼容
- **绑定目标就近解析**：从声明节点沿父链向上找最近一个"挂了脚本且实现了该方法"的节点，
  找不到才回退场景脚本——配合 `<ui script>`（见下节），条目回调可落在条目自身脚本
- 仅当需要**运行时动态目标**（如回调对象不是场景脚本）时才用 `allbind_signal` 手动连接

### `<ui script="xxx.gd">`：脚本声明与自动挂载（组件自带控制器）

gml 根标签声明 `script` 属性，构建期自动把脚本挂到本文件的**根元素节点**
（gml 根即内容根，无包装层；`<Gml>` 引用嫁接与 tscn 打包均保留该节点）：

```xml
<!-- task_item.gml：条目组件自带控制器，可独立预览/独立回调 -->
<ui script="task_item.gd">
  <Panel name="ItemRoot" class="item-bg">
    <Button name="ItemBtn" text="{{btn_text}}" @pressed="_on_task_action" />
  </Panel>
</ui>
```

```gdscript
# task_item.gd —— 挂载方式完全由 GML 声明，无需手动 add script
extends Panel

# 数据契约：@export 变量 = 条目接受的外部注入字段（见下一节）
@export var title: String = ""
@export var btn_state: String = ""

func _on_task_action(_btn: Control) -> void:
	print("点击: ", title, " state=", btn_state)  # 直读契约变量
```

要点：
- **条目组件标准用法**：声明了 script 的 gml 被 `<Gml>` 注入列表 slot 模板后，
  脚本随条目根节点进入模板；条目 `duplicate()` 时脚本随节点复制（duplicate 复制 script），
  `update()` 重建后 `bind_events` 就近重连到**每个条目自己的脚本实例**——
  条目回调写在条目脚本里，外部场景控制器完全不感知
- **就近解析回退**：若祖先链上没有实现该方法的脚本（如顶栏按钮），回退连接到
  `GdGmlScene` 场景脚本——一个面板可混合"条目自带回调 + 场景级回调"
- 挂载目标已有脚本时（如场景根），跳过并告警（外部脚本优先）
- 路径相对当前 gml 所在目录解析；`GdUiBuilder.set_base_dir("user://xxx")` 可为
  `parse_string` 提供相对路径基准（`parse_file` 自动推导）
- 挂载时把脚本 @export 变量名列表记录为条目根 meta `__data_contract`
  （PackedStringArray），编辑器/检查工具可读取

### 数据契约：条目脚本 `@export` 变量自动注入（推荐）

条目脚本中用 `@export var` 声明的变量 = 该条目接受的外部数据字段。
列表 `update(data)` 时，data 字典中与 @export 变量**同名**的 key 自动写入脚本实例
（`@export` 即契约：**声明了才注入**，类型即文档——编辑条目 gml 时对照
`<ui script>` 脚本的 @export 列表即知会传入哪些数据）：

```gdscript
# task_item.gd
extends Panel
@export var icon: String = ""
@export var title: String = ""
@export var btn_state: String = ""
```

```xml
<!-- task_item.gml：{{模板}} 负责把数据渲染到子节点，@export 负责把数据交给脚本 -->
<ui script="task_item.gd">
  <Panel name="ItemRoot">
    <Button name="ItemBtn" text="{{title}}" @pressed="_on_task_action" />
  </Panel>
</ui>
```

规则：
- **注入优先级**：@export 同名注入与 `{{模板}}` 绑定互不冲突（一个写给脚本、
  一个写给节点属性）；未声明的 key 仍走模板绑定与 `__item_data` meta 兜底（向后兼容）
- **部分更新友好**：data 中缺失的契约字段保留当前值（不重置），适合增量 update
- **类型匹配**：注入走 `node.set()`，data 值类型需与 @export 类型一致
  （如 `@export var count: int` 配数字）；普通 `var`（无 @export）不会被注入
- 无脚本的条目完全不受影响；回调可直接读契约变量，无需再沿父链解析 `__item_data`

### `<script>` 数据块与 data 绑定

gml 内 `<script>` 块用 JSON 风格字面量定义数据变量（支持字符串/数字/布尔/null/数组/对象、
行注释与尾逗号），节点用 `data="变量名"` 绑定：

- **列表控件（UIVList/UIHList/UIGrid）+ 数组变量**：构建期直接 `update(data, false)` 驱动条目，
  无需控制器代码
- **其他节点**：变量值存为节点 meta `__script_data`，控制器按需读取
- 全部变量同时挂根节点 meta `__script_vars`（Dictionary），控制器可整体读取

```xml
<ui>
  <script>
    // 各页签任务数据（数据与 UI 同文件声明，构建期直接绑定到列表）
    var daily_tasks = [
      { icon: "🌿", title: "采集灵草", progress: "3/5", btn_text: "前往" },
    ]
  </script>
  <UIVList name="TaskList" data="daily_tasks">
    <Gml src="task_item.gml" />
  </UIVList>
</ui>
```

### `<Gml>` 具名数据映射：data-子变量名="父变量名"

被引用文件 `<script>` 定义的是**默认数据**；引用方用 `data-xxx="父变量"` 把自己的变量
映射进子文件同名变量 `xxx`（`data-` 后跟的是**子文件**的变量名），子文件内部节点继续用
自己的变量名（`data="tasks"` 等）引用——子文件不知道、也不需要知道父变量的名字：

```xml
<!-- task_tabs.gml（父）：data-tasks 中的 tasks 是 task_list.gml 的变量名 -->
<Tab title="日常"><Gml src="task_list.gml" data-tasks="daily_tasks" /></Tab>
<Tab title="主线"><Gml src="task_list.gml" data-tasks="main_tasks" /></Tab>

<!-- task_list.gml（子）：保持 data="tasks" 不变，被引用时被覆盖，独立打开用默认数据 -->
<UIVList name="TaskList" data="tasks"> ... </UIVList>
```

- 可同时映射多个变量：`<Gml src="sub.gml" data-tasks="a" data-title="b" />`
- 引用的父变量不存在时构建期报错但不中断，子文件默认数据原样生效
- 不写 `data-*` 时子文件默认数据原样生效（组件可独立预览/复用）
- 旧版整体覆盖语法 `data="父变量"`（不指定子变量名）仍兼容，推荐迁移到 `data-*` 具名映射

### GdBean 响应式绑定：data="bean:bean_id:属性名"

`data` 值以 `bean:` 开头时走 **GdBean 运行时响应式绑定**（不走 `<script>` 变量机制）：
`GdGmlScene` 场景加载后 `auto_bind_data` 自动完成初始填充 + `bean.watch()` 注册，
Bean 属性 `emit([key])` 变化时**更新全部同名绑定节点**（多页签共享数据源时全部同步刷新）。
数据源由任意脚本注册（`GdBean.bean(id, 工厂)`，工厂返回继承 GdBean 的数据类），
通常配合 `<ui script>` 组件脚本一并声明（见 example/ui/task/task_list.gd）。

```xml
<!-- task_list.gml：<script> 默认数据 + 组件脚本注册 Bean -->
<ui script="task_list.gd">
  <script>
    var tasks = [ { title: "示例任务", btn_state: "go" }, ]
  </script>
  <UIVList name="TaskList" data="bean:task_list:tasks"> ... </UIVList>
</ui>
```

```gdscript
# task_list.gd（挂载在内容根上）：注册 GdBean 数据源 + 驱动数据
class TaskBean:
	extends GdBean
	var tasks: Array = []

func _ready() -> void:
	_bean = GdBean.bean("task_list", _create_bean)  # 工厂从 __script_vars meta 取 GML 默认数据
	_bean.watch("tasks", _on_tasks_changed)          # 独立打开（无场景脚本）时兜底绑定
```

- **注意**：GdBean 属性经 GDCORE 存档持久化（`user://coredata_*.data`），跨运行会恢复上次
  数据；演示类场景可在注册后 `bean.set("tasks", 默认数据)` 重置
- 列表刷新统一用 `update(data, false)`（count<=0 走动态分支按数据增删条目）；
  **`update(data, true)` 在 `count == data.len()` 时（如首次填充）两个分支都不满足，
  不会创建条目**——这是已知坑
- 同一 bean+key 的 watch 每个 GmlScene 只注册一次（回调内部更新全部同名节点）；
  独立打开组件 tscn（无 GdGmlScene）时由组件脚本兜底绑定

### 列表分类过滤：filter_key + filter_value_var（配合 bean: 绑定与 <Gml> 复用）

列表控件（UIVList/UIHList/UIGrid）可声明过滤：`update_container` 建条目前按
`item[filter_key] == filter_value` 过滤数据，声明后列表即成为"只显示匹配分类"的视图
（bean: 响应式推来的全量数据、直接 `update()` 的外部数据都在建条目前过滤）：

```xml
<!-- task_list.gml：绑定全量数据 + 声明分类过滤；filter_value_var 引用本文件
     <script> 的 category 变量（过滤值），引用方用 data-category 注入各自分类 -->
<ui script="task_list.gd">
  <script>
    var category = ""   <!-- 独立打开时为空 = 不过滤（显示全部） -->
    var tasks = [ { category: "daily", title: "..." }, ... ]
  </script>
  <UIVList name="TaskList" data="bean:task_list:tasks"
           filter_key="category" filter_value_var="category"> ... </UIVList>
</ui>
```

```xml
<!-- task_tabs.gml：每个页签注入不同分类，同一份列表文件按分类各显示各的 -->
<script>
  var cat_daily = "daily"
  var cat_main = "main"
</script>
<Tab title="日常"><Gml src="task_list.gml" data-category="cat_daily" /></Tab>
<Tab title="主线"><Gml src="task_list.gml" data-category="cat_main" /></Tab>
```

规则：
- `filter_key` = 数据字段名字面量；`filter_value_var` = 本文件 `<script>` 变量名
  （构建期解析为过滤值，**必须在 data 绑定前注入**，首次填充即过滤）
- 过滤值匹配用 Variant 相等比较，任务数据字段类型需与过滤值一致（同为字符串）
- 过滤值为**空串**时不过滤（组件独立打开的合理默认）；`filter_value_var` 引用的
  变量不存在时告警并不过滤
- 典型组合：bean 存全量数据（单一数据源）+ 各实例 filter 出自己的分类视图，
  数据变化 emit 后所有实例自动刷新（各显示各的分类）

### 页签：直接复用 TabContainer/Tab

```xml
<TabContainer name="TaskTabs" anchor="full" tabs_visible="true" current_tab="0">
  <Tab title="日常"><Gml src="task_list.gml" data-tasks="daily_tasks" /></Tab>
  <Tab title="主线"><Gml src="task_list.gml" data-tasks="main_tasks" /></Tab>

</TabContainer>
```

- Tab 的 `title` 即页签名（也是节点名，可用 `find_node("日常")` 定位页面）
- 切换是原生行为，控制器只需 `tabs.tab_changed.connect(_on_tab_changed)`
- 当前引擎 TabContainer 仅支持顶部/底部页签（`tabs_position` 无 LEFT/RIGHT）；
  设计图样式的左侧竖排页签需用自定义按钮列表 + 数据驱动高亮实现

### HUD 覆盖层：全屏 UI 的鼠标穿透（mainhud 模式）

覆盖在游戏世界之上的 HUD（主界面/战斗 UI）用无包装层模式编写：根 `<Control anchor="full">`
+ 多个 `anchor="full"` 的 MarginContainer 布局层（margin 收窄各层活动范围），
用 HBox/VBox + `expand_fill` spacer + `shrink_begin/shrink_center` 实现四角与居中停靠。
**GML 无 mouse_filter 属性**，穿透在根控制器 `_ready` 里统一设置：

```gdscript
func _ready() -> void:
    mouse_filter = Control.MOUSE_FILTER_IGNORE          # 根穿透
    for layer in ["TopLayer", "MidLayer", "BottomLayer"]:  # 各布局层穿透
        (find_child(layer, true, false) as Control).mouse_filter = Control.MOUSE_FILTER_IGNORE
```

- `IGNORE` 只作用于节点自身，**子交互节点（功能按钮/技能格）不受影响照常可点**
- 行内子项用 `size_flags_vertical="shrink_begin"` 防止被行高（最高子项）拉伸
- `size_flags_*` 支持值：`fill` / `expand` / `expand_fill` / `shrink_begin` /
  `shrink_center` / `shrink_end`；**不认识的值静默回退 FILL**（容器子节点默认
  FILL 会被拉伸到容器尺寸——组件"撑满整屏高"先查这里是不是值拼错/框架过旧）
- 布局断言用 `global_position`（子节点 `position` 是父容器相对坐标，跨容器比较会错）
- 参考 `example/ui/mainhud/`（含 check_mainhud_ui.gd 端到端验收）

### 必踩的坑（重要）

1. **`<ui>` 下必须恰好一个根元素**（`<style>`/`<script>` 是数据块不算元素），
   该根元素直接作为 `parse_string/parse_file`/`build_scene_*` 的返回根节点——
   **无包装层**，布局属性（anchor/margin）写在根元素上即完整约定布局，
   产物 `.gml.tscn` 可直接在编辑器打开预览（根锚点相对视口生效）。
   多根/无根会构建报错；结构节点在 GML 中务必**显式命名**（未命名节点是
   `@Class@id` 形式，NodePath 无法命中）。GdGmlScene 场景加载时：根元素未声明
   anchor/margin 则默认占满场景（兼容旧语义），声明了则完全尊重 gml 的布局。
2. **`UIVList/UIGrid.update(data, force)` 的 force 语义**：`force=true` 会把内部 count
   固定为数据长度——首次调用时（列表为空）两个增删分支都不命中，**一个条目都不会创建**。
   纯数据驱动列表保持 `count<=0`（GML 不写 count 属性），统一用 `update(data, false)`。
3. **allbind_signal 的 path 是相对条目根节点的完整 NodePath**（如 `ItemMargin/ItemRow/ItemBtn`），
   路径上每个节点都要显式命名；未命名节点在 duplicate/add 后变成 `_Class_N`，路径必失效。
   条目信号优先用 `@pressed` 声明（自动重连），仅动态目标场景才用 allbind_signal。
4. **`Node.duplicate()` 不复制 meta**：GML 声明（@pressed/信号绑定）依赖 `__signal_*` meta
   随条目存活，列表在 duplicate 后会自动从 slot 模板同步这些 meta；自己写 duplicate
   相关逻辑时注意同样的问题。
5. **`node.call("method", [a, b])` 是错的**：`call` 的参数是变长逐个传的
   （`call("update", data, true)`），包成数组等于只传 1 个 Array 参数，
   对 Rust `#[func]` 方法直接报"N parameters, M arguments"。类型明确时直接
   `node.update(data, true)` 动态调用，别绕 call。
6. **`{{模板}}` 只支持整值插值**：`text="{{key}}"` 整个属性值替换；混合文本
   `⏱ {{countdown}}` 不会解析——前缀写在数据值里，或拆成兄弟节点。
7. **条目契约 setter 的 `is_node_ready()` 时序**：列表构建期注入发生在条目挂树前，
   setter 里的 `is_node_ready()` 为 false 会跳过 UI 联动——`_ready()` 必须统一
   补偿调用全部 `_apply_xxx()`（注入状态才能生效）。
8. **点/边锚（top_right/bottom_right/bottom_wide…）+ margin 会把节点推出父容器**：
   margin 语义是 `(left, top, -right, -bottom)` 直接写 offset——对 l/r 锚=1 的
   预设，`offset_left=正数` 起点已在父容器右边界之外（如 top_right 的红点
   `margin="0 6 10 0"` 实际 pos.x=父宽，完全看不见）；bottom_wide 的文字
   `margin top=0` 起点 y=父高，落在容器正下方。**角标/文字优先用容器结构**
   （`anchor="full"` 的 VBox：图标 `expand_fill` + 文字贴底），确实要锚点时：
   - 右上/右下角标 → `anchor="top_wide"/"bottom_wide"` + `align="right"` + 负 right margin
   - bottom_wide 贴底文字 → 负 top margin（如 `margin="14 -24 14 2"`，
     注意 Label 行高比 font_size 大，留余量）
