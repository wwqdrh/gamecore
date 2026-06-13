# core 项目文档

## 项目概述

core 是一个基于 Rust 的 Godot 4 GDExtension 项目，使用 gdext 库与 Godot 引擎交互。项目整合了协程系统、图像生成工具等功能，通过单一 crate 构建出 GDExtension 动态库。

## 核心功能

1. **协程系统**
   - 在 Godot 4.4+ 中运行 Rust 协程和异步函数
   - 支持 yield 等待（帧数/秒数/条件/信号）
   - 协程暂停、恢复、终止
   - 异步任务在后台线程运行

2. **图像生成工具**（ImageTool 类）
   - 背景图生成：随机或指定参数生成图案背景，返回 Texture2D
   - 手绘图形生成：随机或指定参数生成手绘风格图形，返回 Texture2D
   - UI图片生成：圆角矩形、渐变矩形、按钮（带阴影）、面板、胶囊、圆形等游戏UI元素，基于imagetool的ui模块
   - 基于 imagetool 库的 bg 和 handdraw 模块

3. **HUD UI组件**（hud 模块）
   - UiButton：程序化样式按钮组件，基于Control + 内部TextureButton，支持圆角矩形/胶囊/圆形，可配置背景色/边框/阴影/文字，自动生成normal/pressed/hover/disabled四种状态纹理
   - UiCard：卡片布局组件，包含圆角背景+标题+图片容器+描述文字，支持圆角/边框/阴影/内边距/间距配置
   - UiPanel：程序化样式面板组件，基于Control + 内部TextureRect，支持圆角矩形/胶囊/圆形，可配置背景色/边框
   - 所有组件在Godot编辑器Inspector中可调节属性，调用refresh_style()刷新样式

4. **测试类**（TestClass）
   - 协程功能测试，暴露给 GDScript 调用
   - 包含异步任务示例

5. **程序化动画**（AnimGraph 类）
   - 基于 anim-kit 库的数据驱动骨骼动画系统
   - 支持 JSON 配置加载，7 种运动基元（正弦波、三角波、方波、噪声、弹簧阻尼、脉冲、步行周期）
   - 支持风力、速度因子等全局参数运行时调整
   - 支持 IK 解算器（FABRIK、CCD、LookAt）
   - 每帧自动更新骨骼姿态，GDScript 按骨骼名获取变换

6. **状态管理**（state 模块）
   - GdDataLinkList：数据链表，封装 Dictionary<String, Array> 的增删查序列化
   - GJson：纯 Rust 实现的 JSON 文档存储，支持路径查询（`;` 分隔嵌套路径）、XOR 加密、文件持久化、变更订阅通知
   - GdCoreData：核心数据管理器（继承 Resource），基于 GJson 实现数据存取、加密存档、作用域隔离、订阅通知
   - GdBean：数据绑定 Bean（继承 RefCounted），持有 GdCoreData 引用，支持属性监听、UI 绑定、表达式更新、存档管理
   - GDCore：全局核心单例（继承 RefCounted），注册为 Engine singleton "GDCORE"，支持存档 ID 管理（set_save_id/get_save_id），根据 save_id 切换不同存档文件（user://coredata_{id}.data），切换时自动通知所有 GdBean 实例更新数据

7. **肉鸽引擎**（rogue 模块）
   - 基于 gamealgo 库的肉鸽游戏核心算法 Godot 暴露层
   - RogueEngine：核心引擎类（继承 RefCounted），管理 RogueContext（种子/深度）和 EntityPool（实体模板池）
   - 支持 JSON 格式初始化实体模板和卡堆配置，GDScript 无需了解 Rust API 细节
   - RogueCard：卡牌包装类（继承 RefCounted），暴露卡牌数据给 GDScript
   - RogueCardPile：牌堆包装类（继承 RefCounted），暴露牌堆数据和顶牌查询
   - 支持种子可复现、快照/恢复（存档读档）、过程化实体生成（模板+缩放曲线）

8. **后台控制台**（console 模块）
   - 基于 mlua (Lua 5.1) 的后台控制台，支持运行时执行 Lua 脚本
   - GdConsole：全局控制台单例（继承 RefCounted），注册为 Engine singleton "GdConsole"
   - 内置函数：fps()、memory()、gc_info()、cpu_info()、help()
   - 支持 GDScript 注册命令函数（register_command），在 Lua 中直接按名称调用
   - 支持 execute（执行代码）和 eval（执行表达式返回结果）
   - console_output 信号：每次执行后触发，输出内容回传给 GDScript
   - 内置控制台 UI 面板：按 ` 键打开/关闭，输入框+日志输出，命令历史导航（上下键）
   - EditorPlugin 自动加载：插件启用时在编辑器中显示控制台

9. **对话系统**（dialog 模块）
   - 基于 gamedialog 库的对话脚本引擎，支持结构化对话脚本解析和执行
   - GdDialogue：对话控制节点（继承 Node），管理 Timeline 和对话推进
   - 支持多角色对话、分支控制流（跳转/条件/循环）、场景变量、全局变量、条件入口
   - 属性：dialogue_control_path、timeline_path、click_next、skip、skip_can_next、skip_time、handle_fn
   - 方法：next、exec_response、is_registered_role、register_role_node、get_role_pos、initial、goto_stage、all_stages、has_next、stage_index
   - 信号：s_finished（对话结束时触发）
   - gamedialog 库（vendor/gamedialog/）：纯 Rust 实现，包含 DialogueWord、ControlFlow、DiaStage、Timeline、SceneManager

10. **UI 标记语言**（ui 模块）
    - 类 HTML 的声明式 UI 描述语言，用于快速构建 Godot Control 节点树
    - GdUiBuilder：UI 构建器类（继承 RefCounted），暴露给 GDScript 的 API
    - 解析器：自写 HTML 子集解析器，支持标签/属性/样式块/自闭合标签/注释
    - 构建器：AST → Godot Control 节点树，支持容器/控件实例化、属性设置、StyleBoxFlat 样式、信号绑定
    - 主题系统：内置卡通风格配色方案（cartoon），GML 中通过 `$var_name` 引用主题变量，GDScript 通过 `apply_theme("cartoon")` 一键切换主题
    - 支持的容器：VBoxContainer、HBoxContainer、GridContainer、MarginContainer、ScrollContainer、TabContainer、CenterContainer、PanelContainer、Tab
    - 支持的控件：Label、Button、TextureButton、CheckButton、HSlider、ColorRect、OptionButton、Panel、TextureRect、RichTextLabel、LineEdit、ProgressBar、SpinBox、HSeparator、VSeparator、NinePatchRect、PopupPanel、Tooltip、Drawer、NavMenu
    - 样式系统：通过 `<style>` 块定义 CSS 类样式，映射到 Godot StyleBoxFlat，支持 `$var` 主题变量引用
    - 信号绑定：通过 `on_xxx` 属性声明，`connect_signals()` 方法批量连接
    - 方法：parse_string、parse_file、connect_signals、validate、set_theme、get_theme、get_builtin_themes、set_theme_var、clear_custom_theme_vars
    - 列表扩展节点（翻译自 C++ gmlc/）：
      - GdUIHList：水平列表（继承 HBoxContainer），支持 slot 模板复制、点击高亮、填充效果
      - GdUIVList：垂直列表（继承 VBoxContainer），同上 + 鼠标进入/离开事件 + 随机高度
      - GdUIGrid：网格列表（继承 GridContainer），同上 + 移动端触摸长按 + patch_item
      - GdListHelper：列表辅助工具（初始化/更新容器/节点值设置/信号批量绑定）
      - GdSlotHighlight/GdSlotFill：方形/圆形高亮和填充效果（Shader）
    - GML 标签：`<UIHList>`、`<UIVList>`、`<UIGrid>`、`<PopupPanel>`、`<Tooltip>`、`<Drawer>`、`<NavMenu>`、`<NavItem>`、`<Tab>`
    - 新增属性：size_flags_horizontal、size_flags_vertical、color（ColorRect）、toggle_mode、button_pressed、items（OptionButton）、selected、popup_title、popup_width、close_on_overlay、tooltip_title、tooltip_content、delay、offset_x、offset_y、max_width、direction、slide_width、animation_duration、drawer_title、title（Tab）、current_tab（TabContainer）、tabs_visible（TabContainer）、texture_normal/texture_pressed/texture_hover/texture_disabled（TextureButton）
    - 百分比自适应语法：size、custom_minimum_size、margin、popup_width、slide_width、menu_width、sub_menu_width 属性支持百分比值（如 "80%,50%"、"5%"、"50%"），基于父容器大小自动计算实际像素值，窗口大小变化时自动刷新
    - TextureButton 支持：GML 中使用 `<TextureButton>` 标签，text 属性自动叠加居中 Label，样式系统中 texture 属性加载纹理到 texture_normal，支持 texture_normal/texture_pressed/texture_hover/texture_disabled 属性直接指定纹理路径
    - 模板绑定语法：GML 属性值中使用 `{{key}}` 格式（如 `text="{{icon}}"`），builder 阶段记录绑定关系到 meta（`__tpl_{key}`、`__tpl_keys`、`__tpl_attr`），update 阶段 `resolve_template_bindings_recursive` 递归解析并设置值
    - Tooltip 自动绑定：UIHList/UIGrid 的 `tooltip` 属性指定 Tooltip 节点名，鼠标进入/离开子节点时自动从 item 的 meta 读取 name/desc 显示提示框，无需在 GDScript 中手动绑定信号
    - Drawer 初始隐藏：`ready()` 中直接设置 `visible=false` 和 overlay 透明，避免初始状态灰色全屏遮挡
    - 数据格式简化：update 数据 Dictionary 的 key 不再需要 `VBoxContainer/IconLabel:text` 路径前缀，直接使用简单 key（如 `icon`、`name`），含 `:` 或 `/` 的 key 兼容旧路径格式
    - 数据自动绑定：UIHList/UIGrid 的 `data` 属性支持两种格式：
      - 简单变量名（如 `data="equip_data"`）：从 GDScript 脚本变量读取（受 gdext 限制可能返回 NIL）
      - GdBean 引用（如 `data="bean:scene_main:equip_data"`）：从 GdBean 实例读取属性值，支持响应式更新（bean 属性变更时自动更新绑定的 UI 节点）
    - GdBean 响应式绑定：`data="bean:bean_id:property_key"` 格式通过 `get_bean_by_id()` 查找 GdBean 实例，调用 `get_value_by_key()` 获取数据，并通过 `bean.watch()` 注册回调，属性变更时自动调用 `on_bean_data_changed_bound()` 更新节点。watch 回调延迟注册（call_deferred），避免 bean 立即触发回调时与 `parse_and_build` 的 `&mut self` 借用冲突
    - 内部信号绑定方法自动适配：Toggle/Show/Hide 动作通过 `has_method()` 检测目标节点方法，Drawer 使用 `toggle`/`open`/`close`，PopupPanel 使用 `toggle_popup`/`show_popup`/`hide_popup`
    - GDScript 继承 GdGmlScene 时需在 `_ready()` 中调用 `load_gml()` 加载 GML（gdext 的 `IControl::ready()` 不可从 GDScript `super._ready()` 调用）
    - 测试用例：12 个解析器测试（含列表标签、错误处理、深度嵌套等）

## 项目结构

```
core/
├── Cargo.toml              # Workspace 配置
├── rust-toolchain.toml     # Rust 工具链配置（nightly）
├── build.sh                # GDExtension 构建脚本
├── project.godot           # Godot 项目配置
├── addons/gamecore/            # Godot 插件目录
│   ├── core.gdextension    # GDExtension 配置（统一入口）
│   ├── core.gd             # EditorPlugin 脚本（自动加载控制台面板+注册GML扩展名）
│   ├── plugin.cfg          # 插件元信息
│   ├── ui/                 # 内置 UI 组件
│   │   ├── console_panel.gd # 控制台面板（输入框+日志输出）
│   │   └── dialogue_panel.gd # 对话框面板（说话人+文本+选项）
│   └── bin/               # 构建产物输出目录
│       ├── macos/         # macOS framework 产物
│       ├── linux/         # Linux .so 产物
│       ├── windows/       # Windows .dll 产物
│       ├── ios/           # iOS framework 产物
│       ├── android/       # Android .so 产物
│       └── web/           # Web .wasm 产物
├── rust/
│   ├── Cargo.toml          # core crate 配置
│   └── src/                # 源码
│       ├── lib.rs          # crate 入口，GDExtension 注册
│       ├── coroutine.rs    # 协程执行器
│       ├── yielding.rs     # yield 类型定义
│       ├── builder.rs      # 协程构建器
│       ├── start_coroutine.rs  # StartCoroutine trait
│       ├── start_async_task.rs # StartAsyncTask trait
│       ├── image_tool.rs   # ImageTool 类实现
│       ├── test_class.rs   # TestClass 类实现
│       ├── anim_graph.rs   # AnimGraph 类实现
│       └── state/           # 状态管理模块
│           ├── mod.rs       # 模块入口
│           ├── linklist.rs  # GdDataLinkList 数据链表
│           ├── gjson.rs     # GJson 纯 Rust JSON 文档存储
│           ├── coredata.rs  # GdCoreData 核心数据管理器
│           ├── bean.rs      # GdBean 数据绑定 Bean
│           └── gdcore.rs    # GDCore 全局核心单例
│       └── hud/            # HUD UI组件模块
│           ├── mod.rs      # 模块入口
│           ├── ui_button.rs # UiButton 按钮组件
│           ├── ui_card.rs  # UiCard 卡片布局组件
│           └── ui_panel.rs # UiPanel 面板组件
│       └── rogue/          # 肉鸽引擎模块
│           ├── mod.rs      # 模块入口
│           ├── engine.rs   # RogueEngine 核心引擎类
│           ├── card.rs     # RogueCard 卡牌包装类
│           └── card_pile.rs # RogueCardPile 牌堆包装类
│       └── console/        # 后台控制台模块
│           ├── mod.rs      # 模块入口
│           └── gdconsole.rs # GdConsole 全局控制台单例
│       └── dialog/          # 对话系统模块
│           ├── mod.rs      # 模块入口
│           └── gddialogue.rs # GdDialogue 对话控制节点
│       └── ui/              # UI标记语言模块
│           ├── mod.rs      # 模块入口
│           ├── parser.rs   # 类HTML标记解析器（含12个测试用例）
│           ├── builder.rs  # AST→Control节点树构建器
│           ├── gdui_builder.rs # GdUiBuilder GDScript API类
│           ├── ui_popup_panel.rs # GdPopupPanel 弹窗面板节点
│           ├── ui_tooltip.rs # GdUITooltip 鼠标跟随提示框节点
│           ├── ui_drawer.rs  # GdUIDrawer 抽屉面板节点
│           ├── ui_nav_menu.rs # GdUINavMenu 导航菜单节点
│           ├── ui_gml_scene.rs   # GdGmlScene GML文件加载节点
│           ├── ui_list_helper.rs # 列表辅助工具（GdListHelper/GdSlotHighlight/GdSlotFill）
│           ├── ui_hlist.rs # GdUIHList水平列表节点
│           ├── ui_vlist.rs # GdUIVList垂直列表节点
│           └── ui_grid.rs  # GdUIGrid网格列表节点
├── example/
│   ├── test_from_gd_script.gd  # GDScript 测试脚本
│   ├── fish_procedural_anim.gd # 鱼的程序化动画示例
│   ├── ui_test.gd              # UI图片生成测试脚本
│   ├── ui_test_scene.tscn      # UI图片生成测试场景
│   └── test_scene.tscn         # 测试场景
│   └── rogue/                  # 肉鸽卡牌游戏示例
│       ├── rogue_game.gd       # 示例游戏脚本
│       └── rogue_game.tscn     # 示例游戏场景
│   └── console/                # 控制台示例
│       ├── console_example.gd  # 命令注册示例脚本
│       └── console_example.tscn # 命令注册示例场景
│   └── dialogue/               # 对话系统示例
│       ├── chat1.txt           # 对话脚本文件
│       ├── dialogue_example.gd # 对话示例脚本
│       └── dialogue_example.tscn # 对话示例场景
│   └── ui/                     # UI标记语言示例
│       ├── ui_example.gd       # UI标记语言示例脚本
│       ├── ui_example.tscn     # UI标记语言示例场景
│       ├── sample_ui.gml       # 示例.gml文件
│       ├── scene_title.gd      # 游戏标题界面根节点脚本
│       ├── scene_title_gml.gd  # 游戏标题界面GML控制器（继承GdGmlScene）
│       ├── scene_title.gml     # 游戏标题界面GML布局
│       ├── scene_main.gml      # 游戏主界面GML布局（装备栏+Tooltip+Drawer）
│       └── scene_gallery.gd    # 图鉴界面GML控制器（PopupPanel+TabContainer+UIGrid）
├── PROJECT.md              # 本文档
└── FILES.md                # 文件功能索引
```

## ImageTool API

ImageTool 是一个继承 RefCounted 的 Godot 类，在 GDScript 中通过 `ImageTool.new()` 创建实例。

### 背景图生成

| 方法 | 说明 |
|------|------|
| `generate_random_bg(width: int, height: int) -> ImageTexture` | 生成随机背景图（随机图案类型和颜色） |
| `generate_bg(width: int, height: int, pattern_type: int, bg_color: Color, fg_color: Color, rotation: float) -> ImageTexture` | 生成指定参数的背景图 |

pattern_type 取值：0=纯色, 1=线条, 2=圆点, 3=波浪, 4=方块

### 手绘图形生成

| 方法 | 说明 |
|------|------|
| `generate_random_handdraw(width: int, height: int) -> ImageTexture` | 生成随机手绘图形（随机形状和参数） |
| `generate_handdraw_rectangle(width, height, x, y, rect_w, rect_h, roughness, stroke_color, fill_color, seed) -> ImageTexture` | 手绘矩形 |
| `generate_handdraw_circle(width, height, cx, cy, diameter, roughness, stroke_color, fill_color, seed) -> ImageTexture` | 手绘圆形 |
| `generate_handdraw_line(width, height, x1, y1, x2, y2, roughness, stroke_color, seed) -> ImageTexture` | 手绘直线 |
| `generate_handdraw_ellipse(width, height, cx, cy, ew, eh, roughness, stroke_color, fill_color, seed) -> ImageTexture` | 手绘椭圆 |

seed=0 时使用随机种子。

### UI图片生成

| 方法 | 说明 |
|------|------|
| `generate_ui_rounded_rect(width, height, corner_radius, bg_color, border_width, border_color) -> ImageTexture` | 生成圆角矩形 |
| `generate_ui_gradient_rect(width, height, corner_radius, color1, color2, angle, border_width, border_color) -> ImageTexture` | 生成渐变圆角矩形 |
| `generate_ui_button(width, height, corner_radius, bg_color, border_width, border_color, shadow_offset_x, shadow_offset_y, shadow_blur, shadow_color) -> ImageTexture` | 生成带阴影的按钮 |
| `generate_ui_panel(width, height, corner_radius, bg_color, border_width, border_color) -> ImageTexture` | 生成面板 |
| `generate_ui_capsule(width, height, bg_color, border_width, border_color) -> ImageTexture` | 生成胶囊形状 |
| `generate_ui_circle(size, bg_color, border_width, border_color) -> ImageTexture` | 生成圆形 |

border_width=0 时不绘制边框，shadow_blur=0 且偏移为0时不绘制阴影。

## HUD UI组件 API

HUD模块提供三个预定义的Godot UI组件，在编辑器中添加对应节点即可使用。所有组件基于imagetool程序化生成背景纹理，无需美术资源。

### UiButton

继承 Control，内部包含 TextureButton + Label。支持圆角矩形/胶囊/圆形三种形状，自动生成 normal/pressed/hover/disabled 四种状态纹理。

**导出属性：**

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `corner_radius` | float | 12.0 | 圆角半径 |
| `bg_color` | Color | (0.2, 0.6, 0.9) | 背景颜色 |
| `border_width` | float | 2.0 | 边框宽度（0=无边框） |
| `border_color` | Color | (0.1, 0.3, 0.6) | 边框颜色 |
| `shadow_offset_x` | float | 3.0 | 阴影X偏移 |
| `shadow_offset_y` | float | 4.0 | 阴影Y偏移 |
| `shadow_blur` | float | 8.0 | 阴影模糊半径 |
| `shadow_color` | Color | (0,0,0,0.4) | 阴影颜色 |
| `shape_type` | int | 0 | 形状：0=圆角矩形, 1=胶囊, 2=圆形 |
| `button_text` | String | "Button" | 按钮文字 |
| `font_size` | int | 18 | 字体大小 |
| `text_color` | Color | (1,1,1) | 文字颜色 |

**信号：** `pressed()`

**方法：**

| 方法 | 说明 |
|------|------|
| `refresh_style()` | 重新生成纹理（修改属性后调用） |
| `update_button_text(text: String)` | 更新按钮文字 |
| `get_internal_button() -> Variant` | 获取内部TextureButton节点 |

### UiCard

继承 Control，内部包含 TextureRect(背景) + MarginContainer + VBoxContainer(标题+图片+描述)。固定为圆角矩形形状。

**导出属性：**

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `corner_radius` | float | 16.0 | 圆角半径 |
| `bg_color` | Color | (0.15,0.15,0.2,0.92) | 背景颜色 |
| `border_width` | float | 2.0 | 边框宽度 |
| `border_color` | Color | (0.4,0.4,0.5) | 边框颜色 |
| `shadow_offset_x` | float | 4.0 | 阴影X偏移 |
| `shadow_offset_y` | float | 6.0 | 阴影Y偏移 |
| `shadow_blur` | float | 12.0 | 阴影模糊半径 |
| `shadow_color` | Color | (0,0,0,0.35) | 阴影颜色 |
| `content_padding` | int | 16 | 内容内边距 |
| `title_text` | String | "Card Title" | 标题文字 |
| `title_font_size` | int | 22 | 标题字体大小 |
| `title_color` | Color | (1,1,1) | 标题颜色 |
| `desc_text` | String | "Description..." | 描述文字 |
| `desc_font_size` | int | 14 | 描述字体大小 |
| `desc_color` | Color | (0.8,0.8,0.8) | 描述颜色 |
| `image_min_height` | int | 120 | 图片区域最小高度 |
| `image_placeholder_color` | Color | (0.2,0.2,0.28) | 图片占位符颜色 |
| `spacing` | int | 10 | 子元素间距 |

**方法：**

| 方法 | 说明 |
|------|------|
| `refresh_style()` | 重新生成纹理 |
| `set_card_image(texture: Texture2D)` | 设置卡片图片 |
| `set_title(text: String)` | 设置标题 |
| `set_description(text: String)` | 设置描述 |

### UiPanel

继承 Control，内部包含 TextureRect(背景)。支持圆角矩形/胶囊/圆形三种形状。

**导出属性：**

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `corner_radius` | float | 16.0 | 圆角半径 |
| `bg_color` | Color | (0.12,0.12,0.16,0.9) | 背景颜色 |
| `border_width` | float | 2.0 | 边框宽度 |
| `border_color` | Color | (0.35,0.35,0.45) | 边框颜色 |
| `shape_type` | int | 0 | 形状：0=圆角矩形, 1=胶囊, 2=圆形 |

**方法：**

| 方法 | 说明 |
|------|------|
| `refresh_style()` | 重新生成纹理 |

### GDScript UI生成示例

```gdscript
var tool = ImageTool.new()

# 生成圆角矩形按钮
var btn = tool.generate_ui_rounded_rect(200, 60, 12.0, Color(0.2, 0.6, 0.9), 2.0, Color(0.1, 0.3, 0.6))
$ButtonRect.texture = btn

# 生成渐变按钮
var grad_btn = tool.generate_ui_gradient_rect(200, 60, 12.0, Color(0.9, 0.3, 0.3), Color(0.6, 0.1, 0.5), 90.0, 2.0, Color(0.4, 0.1, 0.2))
$GradientRect.texture = grad_btn

# 生成带阴影按钮
var shadow_btn = tool.generate_ui_button(200, 60, 12.0, Color(0.3, 0.8, 0.4), 2.0, Color(0.15, 0.4, 0.2), 3.0, 4.0, 8.0, Color(0, 0, 0, 0.4))
$ShadowButton.texture = shadow_btn

# 生成面板
var panel = tool.generate_ui_panel(300, 200, 16.0, Color(0.15, 0.15, 0.2, 0.9), 2.0, Color(0.4, 0.4, 0.5))
$PanelRect.texture = panel

# 生成胶囊
var capsule = tool.generate_ui_capsule(160, 50, Color(0.9, 0.7, 0.2), 2.0, Color(0.6, 0.5, 0.1))
$CapsuleRect.texture = capsule

# 生成圆形
var circle = tool.generate_ui_circle(80, Color(0.7, 0.3, 0.8), 3.0, Color(0.4, 0.15, 0.5))
$CircleRect.texture = circle
```

### GDScript 使用示例

```gdscript
var tool = ImageTool.new()

# 生成随机背景图
var bg_texture = tool.generate_random_bg(800, 600)
$TextureRect.texture = bg_texture

# 生成指定参数的背景图（波浪图案）
var wave_bg = tool.generate_bg(800, 600, 3, Color.WHITE, Color.BLUE, 15.0)
$TextureRect.texture = wave_bg

# 生成随机手绘图形
var handdraw_texture = tool.generate_random_handdraw(400, 400)
$TextureRect.texture = handdraw_texture

# 生成手绘矩形
var rect_texture = tool.generate_handdraw_rectangle(
    400, 400,       # 画布宽高
    50.0, 50.0,     # 矩形位置
    300.0, 200.0,   # 矩形宽高
    1.5,            # 粗糙度
    Color.BLACK,    # 描边颜色
    Color.RED,      # 填充颜色
    42              # 种子（0=随机）
)
$TextureRect.texture = rect_texture
```

## RogueEngine API

RogueEngine 是一个继承 RefCounted 的 Godot 类，在 GDScript 中通过 `RogueEngine.new()` 创建实例。基于 gamealgo 库，提供肉鸽游戏核心算法的 Godot 暴露层，支持 JSON 配置驱动。

### 方法

| 方法 | 说明 |
|------|------|
| `init_with_seed(seed: int)` | 使用指定种子初始化引擎，同种子可复现 |
| `get_seed() -> int` | 获取当前种子 |
| `get_depth() -> int` | 获取当前深度 |
| `set_depth(depth: int)` | 设置当前深度 |
| `advance_depth()` | 深度+1 |
| `load_entities_from_json(json: String) -> bool` | 从 JSON 字符串加载实体模板，返回是否成功 |
| `generate_piles(json: String) -> Variant` | 根据 JSON 配置生成卡堆布局，返回 Dictionary |
| `generate_entity(template_id: String) -> Variant` | 根据模板ID生成实体，返回 Dictionary |
| `roll_entity(type_name: String) -> Variant` | 随机生成指定类型的实体，返回 Dictionary |
| `get_snapshot_json() -> String` | 获取当前状态快照（JSON），用于存档 |
| `restore_from_json(json: String) -> bool` | 从 JSON 快照恢复状态，用于读档 |

### 实体模板 JSON 格式

```json
{
  "entities": [
    {
      "id": "goblin",
      "name": "哥布林",
      "type": "monster",
      "weight": 5.0,
      "min_depth": 1,
      "stats": {
        "hp": {"scale": "linear", "base": 20, "per_level": 8, "variance": 0.1},
        "atk": {"scale": "linear", "base": 5, "per_level": 2, "variance": 0.1},
        "def": {"scale": "fixed", "value": 1}
      },
      "tags": ["melee"]
    }
  ]
}
```

缩放类型：`fixed`（固定值）、`linear`（线性增长）、`exponential`（指数增长）

### 卡堆配置 JSON 格式

```json
{
  "pile_count": 3,
  "cards_per_pile_min": 3,
  "cards_per_pile_max": 6,
  "type_weights": {
    "monster": 5.0,
    "weapon": 2.0,
    "armor": 1.5,
    "item": 1.5
  }
}
```

`type_weights` 支持自定义类型名称，不限于 monster/weapon/armor/item。

### generate_piles 返回数据结构

```gdscript
{
  "exit_pile_id": 1,  # 出口所在的牌堆ID
  "piles": [
    {
      "id": 0,
      "position_x": 0.0,
      "position_y": 0.0,
      "cards": [
        {
          "id": 0,
          "face_up": true,
          "entity": {
            "template_id": "goblin",
            "name": "哥布林",
            "type": "monster",
            "depth": 1,
            "stats": {"hp": 22.0, "atk": 5.5, "def": 1.0}
          }
        }
      ]
    }
  ]
}
```

### GDScript 使用示例

```gdscript
var engine = RogueEngine.new()
engine.init_with_seed(42)

# 加载实体模板
var entities_json = """
{
  "entities": [
    {"id": "goblin", "name": "哥布林", "type": "monster", "weight": 5.0,
     "stats": {"hp": {"scale":"linear","base":20,"per_level":8,"variance":0.1},
               "atk": {"scale":"linear","base":5,"per_level":2,"variance":0.1}}},
    {"id": "heal_potion", "name": "治疗药水", "type": "item", "weight": 5.0,
     "stats": {"heal": {"scale":"linear","base":15,"per_level":5,"variance":0.1}}}
  ]
}
"""
engine.load_entities_from_json(entities_json)

# 生成卡堆
var config = """{
  "pile_count": 3,
  "cards_per_pile_min": 3,
  "cards_per_pile_max": 5,
  "type_weights": {"monster": 5.0, "weapon": 2.0, "armor": 1.5, "item": 1.5}
}"""
var result = engine.generate_piles(config)

# 访问卡堆数据
for pile in result["piles"]:
    var cards = pile["cards"]
    if cards.size() > 0:
        var top_card = cards[cards.size() - 1]
        var entity = top_card["entity"]
        print(entity["name"], " - ", entity["type"])

# 存档/读档
var snapshot = engine.get_snapshot_json()
engine.restore_from_json(snapshot)
```

## AnimGraph API

AnimGraph 是一个继承 Node 的 Godot 类，在 GDScript 中通过 `AnimGraph.new()` 创建实例并添加到场景树。支持 JSON 配置驱动，每帧自动更新骨骼姿态。

### 导出属性

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `config_json` | String | "" | JSON 配置字符串，ready 时自动加载 |
| `auto_update` | bool | true | 是否每帧自动更新动画 |

### 方法

| 方法 | 说明 |
|------|------|
| `load_config(json: String) -> bool` | 从 JSON 字符串加载配置，返回是否成功 |
| `update(delta: float)` | 手动更新一帧动画 |
| `get_bone_rotation(bone_name: String) -> Quaternion` | 获取骨骼旋转（四元数） |
| `get_bone_position(bone_name: String) -> Vector3` | 获取骨骼位置 |
| `get_bone_rotation_euler(bone_name: String) -> Vector3` | 获取骨骼旋转（欧拉角） |
| `get_bone_names() -> PackedStringArray` | 获取所有骨骼名称列表 |
| `set_wind_strength(strength: float)` | 设置全局风力强度 |
| `get_wind_strength() -> float` | 获取当前风力强度 |
| `set_speed_factor(factor: float)` | 设置全局速度因子 |
| `get_speed_factor() -> float` | 获取当前速度因子 |
| `set_ik_target(ik_index: int, target: Vector3)` | 设置 IK 目标位置 |
| `set_bone_parameter(bone_name: String, parameter: int, value: float)` | 设置骨骼参数（0=角度最小值, 1=角度最大值, 2=刚度, 3=阻尼） |
| `get_global_time() -> float` | 获取全局动画时间 |

### JSON 配置格式

```json
{
    "bones": [
        {
            "name": "Body",
            "primitive": {"type": "sine", "amp": 0.5, "freq": 2.0, "axis": "y"},
            "angle_min": -0.5,
            "angle_max": 0.5
        }
    ],
    "ik": [
        {"type": "look_at", "bone_name": "Head", "forward_axis": "y", "weight": 0.8}
    ],
    "wind_strength": 0.0,
    "speed_factor": 1.0
}
```

基元类型：`sine`, `triangle`, `square`, `noise`, `spring`, `pulse`, `walk_cycle`

### GDScript 使用示例

```gdscript
var anim = AnimGraph.new()
add_child(anim)

var config = {
    "bones": [
        {"name": "Tail", "primitive": {"type": "sine", "amp": 1.0, "freq": 4.0, "phase": 1.2}},
        {"name": "Body", "primitive": {"type": "sine", "amp": 0.3, "freq": 2.0, "axis": "y"}}
    ]
}
anim.load_config(JSON.stringify(config))

func _process(delta):
    var tail_rot = anim.get_bone_rotation_euler("Tail")
    $Tail.rotation = tail_rot.z
```

## GdConsole API

GdConsole 是一个全局控制台单例，注册为 Engine singleton "GdConsole"。基于 mlua (Lua 5.1)，支持运行时执行 Lua 脚本和 GDScript 命令注册。

### 方法

| 方法 | 说明 |
|------|------|
| `execute(code: String) -> String` | 执行 Lua 代码，返回输出内容（print 输出或错误信息） |
| `eval(code: String) -> Variant` | 执行 Lua 表达式并返回结果 |
| `register_command(name: String, callable: Callable, description: String)` | 注册 GDScript 命令，可在 Lua 中按名称调用 |
| `unregister_command(name: String)` | 注销已注册的命令 |
| `list_commands() -> PackedStringArray` | 列出所有已注册命令及其描述 |

### 信号

| 信号 | 说明 |
|------|------|
| `console_output(text: String)` | 每次执行后触发，输出内容回传 |

### 内置 Lua 函数

| 函数 | 返回值 | 说明 |
|------|--------|------|
| `fps()` | number | 获取当前帧率 |
| `memory()` | table | 获取内存信息（static, message_buffer_max） |
| `gc_info()` | table | 获取 Godot 对象信息（object_count, resource_count, node_count） |
| `cpu_info()` | table | 获取 CPU 信息（processor_count） |
| `help()` | nil | 列出所有可用命令 |
| `print(...)` | nil | 重定向输出到控制台缓冲区 |

### GDScript 使用示例

```gdscript
# 获取 GdConsole 单例
var console = Engine.get_singleton("GdConsole")

# 执行 Lua 代码
var output = console.execute("print('Hello from Lua!')")
print(output)  # Hello from Lua!

# 获取运行信息
var fps_result = console.eval("fps()")
print("FPS: ", fps_result)

var mem_info = console.eval("memory()")
print("Static memory: ", mem_info["static"])

# 注册 GDScript 命令
func _ready():
    var console = Engine.get_singleton("GdConsole")
    console.register_command("heal_player", heal_player, "Heal the player by amount")
    console.register_command("get_pos", get_player_pos, "Get player position")

func heal_player(amount: int):
    player_hp += amount
    return "Healed player by %d, HP: %d" % [amount, player_hp]

func get_player_pos():
    return {"x": position.x, "y": position.y}

# 在控制台中调用注册的命令
# console.execute("heal_player(50)")
# console.execute("get_pos()")

# 监听控制台输出
func _ready():
    var console = Engine.get_singleton("GdConsole")
    console.console_output.connect(_on_console_output)

func _on_console_output(text: String):
    $OutputLabel.text += text + "\n"

# 注销命令
console.unregister_command("heal_player")
```

## GdUiBuilder API

GdUiBuilder 是一个继承 RefCounted 的 Godot 类，在 GDScript 中通过 `GdUiBuilder.new()` 创建实例。提供类 HTML 声明式 UI 描述语言的解析和构建功能。

### 方法

| 方法 | 说明 |
|------|------|
| `parse_string(markup: String) -> Control` | 解析标记字符串，返回 Control 节点树 |
| `parse_file(path: String) -> Control` | 解析 .gml 文件，返回 Control 节点树 |
| `connect_signals(root: Control, target: Object)` | 递归连接 UI 节点树中的信号到目标脚本 |
| `validate(markup: String) -> String` | 验证标记语法，返回错误信息（空字符串表示无错误） |
| `set_theme(theme_name: String)` | 设置内置主题名称（cartoon） |
| `get_theme() -> String` | 获取当前主题名称 |
| `get_builtin_themes() -> PackedStringArray` | 获取所有内置主题名称列表 |
| `set_theme_var(key: String, value: String)` | 设置自定义主题变量（覆盖内置主题同名变量） |
| `clear_custom_theme_vars()` | 清除所有自定义主题变量 |

### 标记语言语法

```html
<ui theme="cartoon">
  <style>
    .button-primary {
        background: $bg_button_primary;
        color: $text_white;
        border_radius: 12;
    }
  </style>
  <VBoxContainer anchor="full" margin="12">
    <Label text="欢迎" font_size="24" align="center" />
    <Button text="开始" class="button-primary" on_pressed="_on_start" />
  </VBoxContainer>
</ui>
```

### 支持的属性

| 属性 | 说明 |
|------|------|
| `text` | 文字内容 |
| `font_size` | 字体大小 |
| `align` | 对齐方式（left/center/right/fill） |
| `anchor` | 锚点预设（full/top_left/center 等） |
| `margin` | 边距（"12" / "10 20" / "10 20 30 40"），支持百分比（如 "5%" / "5% 3%" / "5% 3% 5% 3%"） |
| `size` | 尺寸（"width,height"），支持百分比（如 "80%,50%"）或混合（如 "80%,400"） |
| `custom_minimum_size` | 最小尺寸（"width,height"），支持百分比（如 "10%,10%"） |
| `class` | 应用 `<style>` 中定义的样式 |
| `on_xxx` | 信号绑定（如 on_pressed="_on_click"） |
| `bbcode` | RichTextLabel 的 BBCode 文本 |
| `texture` | TextureRect/NinePatchRect/TextureButton 的纹理路径 |
| `texture_normal` | TextureButton 的正常状态纹理路径 |
| `texture_pressed` | TextureButton 的按下状态纹理路径 |
| `texture_hover` | TextureButton 的悬停状态纹理路径 |
| `texture_disabled` | TextureButton 的禁用状态纹理路径 |
| `stretch_mode` | TextureRect 拉伸模式 |
| `columns` | GridContainer 列数 |
| `visible` | 是否可见 |
| `disabled` | 是否禁用（Button 等） |
| `size_flags_horizontal` | 水平尺寸标志（fill=1, expand=2, expand_fill=3, shrink=4, shrink_center=5, shrink_end=6） |
| `size_flags_vertical` | 垂直尺寸标志（同上） |
| `toggle_mode` | 是否切换模式（CheckButton/Button/TextureButton） |
| `button_pressed` | 按钮是否按下（CheckButton/Button/TextureButton） |
| `color` | 颜色（ColorRect 的填充颜色） |
| `items` | 选项列表，逗号分隔（OptionButton） |
| `selected` | 选中索引（OptionButton） |
| `popup_title` | 弹窗标题（PopupPanel） |
| `width` | PopupPanel 弹窗宽度，支持百分比（如 "50%"） |
| `height` | PopupPanel 弹窗高度，支持百分比（如 "60%"） |
| `close_on_overlay` | 点击遮罩关闭弹窗（PopupPanel，true/false） |
| `title` | Tab 页标题（Tab 标签，TabContainer 用节点名作为 tab 标题） |
| `current_tab` | 当前选中 tab 索引（TabContainer） |
| `tabs_visible` | 是否显示 tab 栏（TabContainer，true/false） |

### 样式属性

| 属性 | 说明 |
|------|------|
| `background` / `bg_color` | 背景颜色（#RRGGBB / #RRGGBBAA / 颜色名 / $主题变量） |
| `color` | 文字颜色（同上） |
| `border_radius` | 圆角半径 |
| `border_color` | 边框颜色（同上） |
| `border_width` | 边框宽度 |
| `padding` | 内边距 |
| `texture` | 纹理路径（TextureButton 的 texture_normal） |

### 主题系统

GML 支持通过主题变量引用颜色值，实现一键切换配色方案。

**内置主题：** cartoon（卡通亮色风格，默认）

**主题变量列表：**

| 变量 | 说明 | cartoon 值 |
|------|------|-----------|
| `$bg_primary` | 主背景色 | #f8f4ff |
| `$bg_secondary` | 次背景色 | #eee8f8 |
| `$bg_panel` | 面板背景色 | #ffffff |
| `$bg_button` | 按钮背景色 | #e8dff5 |
| `$bg_button_primary` | 主按钮背景色 | #7c4dff |
| `$bg_button_danger` | 危险按钮背景色 | #ff5252 |
| `$border_default` | 默认边框色 | #c5b3e6 |
| `$border_accent` | 强调边框色 | #7c4dff |
| `$border_highlight` | 高亮边框色 | #ffab40 |
| `$text_primary` | 主文字色 | #3a2d5c |
| `$text_secondary` | 次文字色 | #7b6fa0 |
| `$text_muted` | 弱化文字色 | #a99cc4 |
| `$text_accent` | 强调文字色 | #7c4dff |
| `$text_title` | 标题文字色 | #5c3dbd |
| `$text_white` | 白色文字 | white |
| `$overlay` | 遮罩色 | #3a2d5c60 |
| `$popup_bg` | 弹窗背景色 | #fffffffa |
| `$popup_border` | 弹窗边框色 | #c5b3e6 |
| `$highlight` | 高亮色 | #7c4dff30 |
| `$accent` | 强调色 | #7c4dff |

**组件默认颜色变量（builder 自动应用，无需 GML 中显式声明）：**

| 变量 | 说明 | cartoon 值 |
|------|------|-----------|
| `$panel_bg` | Panel 背景色 | → $bg_panel |
| `$button_bg` | Button 背景色 | → $bg_button |
| `$button_font_color` | Button 文字色 | → $text_primary |
| `$label_font_color` | Label 文字色 | → $text_primary |
| `$input_bg` | LineEdit 背景色 | #ffffff |
| `$input_font_color` | LineEdit 文字色 | → $text_primary |
| `$separator_color` | 分隔线颜色 | → $border_default |
| `$tab_bg` | TabContainer 背景色 | → $bg_secondary |
| `$tab_font_color` | Tab 文字色 | → $text_secondary |
| `$tab_selected_font_color` | 选中 Tab 文字色 | → $text_accent |
| `$progress_bg` | ProgressBar 背景 | → $bg_button |
| `$progress_fill` | ProgressBar 填充色 | → $accent |
| `$optionbutton_bg` | OptionButton 背景色 | → $bg_button |
| `$optionbutton_font_color` | OptionButton 文字色 | → $text_primary |
| `$popup_title_color` | PopupPanel 标题色 | → $text_title |
| `$drawer_title_color` | Drawer 标题色 | → $text_title |
| `$tooltip_title_color` | Tooltip 标题色 | → $text_accent |
| `$tooltip_content_color` | Tooltip 内容色 | → $text_primary |
| `$nav_item_color` | NavMenu 项文字色 | → $text_primary |
| `$nav_item_hover_color` | NavMenu 项悬停色 | → $text_accent |
| `$nav_item_active_color` | NavMenu 项激活色 | #ff6d00 |
| `$nav_item_hover_bg` | NavMenu 项悬停背景 | #7c4dff18 |
| `$nav_item_pressed_bg` | NavMenu 项按下背景 | #7c4dff28 |

**GML 中使用主题变量：**

```html
<ui theme="cartoon">
  <style>
    .my-panel {
      background: $bg_primary;
      border_color: $border_default;
      color: $text_primary;
    }
  </style>
  <Panel class="my-panel">
    <Label text="Themed content" />
  </Panel>
</ui>
```

**GML 中自定义主题变量（覆盖内置主题）：**

```html
<ui theme="cartoon">
  <theme>
    bg_primary: #f0e6ff;
    my_custom_color: #ff8800;
  </theme>
  <style>
    .my-panel {
      background: $bg_primary;
      accent: $my_custom_color;
    }
  </style>
</ui>
```

**GDScript 中切换主题：**

```gdscript
# 方式1：GdGmlScene（最简单）
extends GdGmlScene

func _ready():
    load_from_string(UI)

func _on_switch_theme():
    apply_theme("cartoon")  # 一键切换为 cartoon 主题（自动修改 GML 中的 theme 属性并重新加载）

# 方式2：GdUiBuilder
var builder = GdUiBuilder.new()
builder.set_theme("cartoon")
var ui = builder.parse_string(gml_content)

# 方式3：自定义变量覆盖
builder.set_theme("cartoon")
builder.set_theme_var("bg_primary", "#f0e6ff")  # 覆盖内置变量
builder.set_theme_var("my_color", "#ff8800")     # 新增自定义变量
var ui = builder.parse_string(gml_content)
```

### GDScript 使用示例

```gdscript
var builder = GdUiBuilder.new()

# 从字符串解析
var ui = builder.parse_string("""
<ui>
  <VBoxContainer>
    <Label text="Hello" />
    <Button text="Click" on_pressed="_on_click" />
  </VBoxContainer>
</ui>
""")
add_child(ui)
builder.connect_signals(ui, self)

# 从文件解析
var file_ui = builder.parse_file("res://ui/main_menu.gml")
add_child(file_ui)

# 验证语法
var error = builder.validate("<ui><Label text='test' /></ui>")
if error != "":
    print("Parse error: ", error)
```

## GdPopupPanel API

GdPopupPanel 是一个继承 Control 的 Rust 弹窗面板节点，在 GML 中通过 `<PopupPanel>` 标签使用。支持模态遮罩、标题栏+关闭按钮、内容区域。替代了旧版 GDScript 实现的 popup_panel.gd。

### 导出属性

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `popup_width` | int | 400 | 弹窗宽度（GML 中使用 width 属性，支持百分比如 width="50%"） |
| `popup_height` | int | 400 | 弹窗高度（GML 中使用 height 属性，支持百分比如 height="60%"） |
| `popup_title` | String | "" | 弹窗标题 |
| `close_on_overlay` | bool | true | 点击遮罩是否关闭弹窗 |
| `popup_bg_color` | Color | (0.08,0.08,0.14,0.95) | 弹窗背景色 |
| `popup_border_color` | Color | (0.35,0.4,0.55) | 弹窗边框颜色 |
| `overlay_color` | Color | (0,0,0,0.5) | 遮罩颜色 |
| `title_font_size` | int | 20 | 标题字体大小 |
| `title_color` | Color | (0.4,0.8,1.0) | 标题颜色 |
| `close_button_text` | String | "X" | 关闭按钮文字 |
| `corner_radius` | int | 8 | 圆角半径 |

### 方法

| 方法 | 说明 |
|------|------|
| `set_title(text: String)` | 设置弹窗标题 |
| `set_close_on_overlay(enabled: bool)` | 设置点击遮罩是否关闭 |
| `set_content(node: Control)` | 设置弹窗内容区域节点 |
| `get_content_node(name: String) -> Node` | 获取内容区域中的节点（按名称查找） |
| `connect_content_signals(target: Object)` | 连接内容区域中的信号到目标对象 |
| `show_popup()` | 显示弹窗 |
| `hide_popup()` | 隐藏弹窗 |
| `is_popup_visible() -> bool` | 弹窗是否可见 |
| `toggle_popup()` | 切换弹窗显示/隐藏 |

### GML 使用示例

```html
<PopupPanel popup_title="Settings" popup_width="500" close_on_overlay="true">
  <VBoxContainer>
    <Label text="Volume" />
    <HSlider min_value="0" max_value="100" value="80" />
    <CheckButton text="Fullscreen" toggle_mode="true" />
    <OptionButton items="English,Chinese,Japanese" selected="0" />
    <Button text="Apply" on_pressed="_on_apply" />
  </VBoxContainer>
</PopupPanel>
```

### GDScript 使用示例

```gdscript
# 通过 GdUiBuilder 解析含 PopupPanel 的 GML
var builder = GdUiBuilder.new()
var ui = builder.parse_file("res://ui/scene_title.gml")
add_child(ui)
builder.connect_signals(ui, self)

# 获取 PopupPanel 节点并操作
var popup = ui.find_child("PopupPanel", true, false)
popup.show_popup()
popup.hide_popup()
popup.toggle_popup()

# 获取内容区域中的节点
var slider = popup.get_content_node("HSlider")
```

## GdUITooltip API

GdUITooltip 是一个继承 Control 的 Rust 鼠标跟随提示框节点，在 GML 中通过 `<Tooltip>` 标签使用。浮动面板跟随鼠标位置显示，支持延迟显示、自动位置调整、标题+内容布局。

### 导出属性

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `tooltip_title_text` | String | "" | 提示框标题 |
| `tooltip_content_text` | String | "" | 提示框内容 |
| `delay` | float | 0.3 | 延迟显示时间（秒） |
| `offset_x` | float | 12.0 | 鼠标X偏移 |
| `offset_y` | float | 12.0 | 鼠标Y偏移 |
| `max_width` | int | 300 | 最大宽度 |
| `bg_color` | Color | (0.1,0.1,0.18,0.95) | 背景色 |
| `border_color` | Color | (0.4,0.5,0.7) | 边框颜色 |
| `title_color` | Color | (0.5,0.85,1.0) | 标题颜色 |
| `content_color` | Color | (0.85,0.85,0.9) | 内容颜色 |
| `corner_radius` | int | 6 | 圆角半径 |

### 方法

| 方法 | 说明 |
|------|------|
| `show_tooltip()` | 显示提示框（开始延迟计时） |
| `hide_tooltip()` | 隐藏提示框 |
| `set_tooltip_title(text: String)` | 设置标题 |
| `set_tooltip_content(text: String)` | 设置内容 |

### 信号

| 信号 | 说明 |
|------|------|
| `s_tooltip_shown()` | 提示框显示 |
| `s_tooltip_hidden()` | 提示框隐藏 |

### GML 使用示例

```html
<Tooltip name="EquipTooltip" tooltip_title="Item" tooltip_content="Item description" delay="0.3" max_width="250" />
```

### GDScript 使用示例

```gdscript
var tooltip = ui.find_child("EquipTooltip", true, false)
tooltip.set_tooltip_title("Sword")
tooltip.set_tooltip_content("A sharp blade\nATK +10")
tooltip.show_tooltip()
tooltip.hide_tooltip()
```

## GdUIDrawer API

GdUIDrawer 是一个继承 Control 的 Rust 抽屉面板节点，在 GML 中通过 `<Drawer>` 标签使用。从屏幕边缘滑入/滑出，支持动画过渡、模态遮罩、内容区域。

### 导出属性

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `direction` | int | 0 | 方向：0=right, 1=left, 2=top, 3=bottom |
| `slide_width` | int | 320 | 抽屉宽度/高度（支持百分比，GML 中如 slide_width="35%"） |
| `overlay_color` | Color | (0,0,0,0.5) | 遮罩颜色 |
| `drawer_bg_color` | Color | (0.08,0.08,0.14,0.97) | 抽屉背景色 |
| `drawer_border_color` | Color | (0.35,0.4,0.55) | 边框颜色 |
| `corner_radius` | int | 0 | 圆角半径 |
| `animation_duration` | float | 0.25 | 动画时长（秒） |
| `close_on_overlay` | bool | true | 点击遮罩是否关闭 |
| `drawer_title_text` | String | "" | 抽屉标题 |

### 方法

| 方法 | 说明 |
|------|------|
| `open()` | 打开抽屉 |
| `close()` | 关闭抽屉 |
| `toggle()` | 切换开关 |
| `is_drawer_open() -> bool` | 抽屉是否打开 |
| `set_drawer_title(text: String)` | 设置标题 |

### 信号

| 信号 | 说明 |
|------|------|
| `s_drawer_opened()` | 抽屉打开完成 |
| `s_drawer_closed()` | 抽屉关闭 |

### GML 使用示例

```html
<Drawer name="InventoryDrawer" direction="right" slide_width="360" drawer_title="Inventory" close_on_overlay="true">
  <UIGrid name="InventoryGrid" count="12" columns="3">
    <MarginContainer custom_minimum_size="96,96">
      <Label name="ItemName" text="Item" />
    </MarginContainer>
  </UIGrid>
</Drawer>
```

### 内部信号绑定

Drawer 支持 `open:DrawerName`、`close:DrawerName`、`toggle:DrawerName` 格式的内部信号绑定：

```html
<Button text="Open" on_pressed="toggle:InventoryDrawer" />
```

## GdUINavMenu API

GdUINavMenu 是一个继承 Control 的 Rust 导航菜单节点，在 GML 中通过 `<NavMenu>` 标签使用。支持多级级联菜单（NavItem 递归嵌套），从屏幕左侧/右侧滑入，支持动画过渡和模态遮罩。

### 导出属性

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `direction` | int | 0 | 方向：0=left, 1=right |
| `menu_width` | int | 160 | 一级菜单宽度（支持百分比，GML 中如 menu_width="15%"） |
| `sub_menu_width` | int | 180 | 二级菜单宽度（支持百分比，GML 中如 sub_menu_width="20%"） |
| `menu_bg_color` | Color | (0.08,0.08,0.14,0.97) | 菜单背景色 |
| `menu_border_color` | Color | (0.35,0.4,0.55) | 菜单边框颜色 |
| `overlay_color` | Color | (0,0,0,0.5) | 遮罩颜色 |
| `corner_radius` | int | 0 | 圆角半径 |
| `animation_duration` | float | 0.2 | 动画时长（秒） |
| `close_on_overlay` | bool | true | 点击遮罩是否关闭 |
| `item_font_size` | int | 16 | 一级菜单项字体大小 |
| `item_color` | Color | (0.85,0.85,0.9) | 一级菜单项文字颜色 |
| `item_hover_color` | Color | (1,1,1) | 一级菜单项悬停颜色 |
| `item_active_color` | Color | (0.4,0.8,1.0) | 一级菜单项激活颜色 |
| `sub_item_font_size` | int | 14 | 二级菜单项字体大小 |
| `sub_item_color` | Color | (0.75,0.75,0.8) | 二级菜单项文字颜色 |
| `sub_item_hover_color` | Color | (1,1,1) | 二级菜单项悬停颜色 |

### 方法

| 方法 | 说明 |
|------|------|
| `open()` | 打开菜单 |
| `close()` | 关闭菜单 |
| `toggle()` | 切换开关 |
| `is_menu_open() -> bool` | 菜单是否打开 |
| `ensure_ui_built()` | 确保内部 UI 已构建（供 builder 调用） |

### 信号

| 信号 | 说明 |
|------|------|
| `s_menu_opened()` | 菜单打开完成 |
| `s_menu_closed()` | 菜单关闭 |
| `s_item_clicked(path: GString)` | 叶子项被点击，path 为从根到叶的索引路径（如 "0,1,2"） |

### GML 使用示例

```html
<NavMenu name="NavMenu" direction="left" menu_width="160" sub_menu_width="200" close_on_overlay="true">
  <NavItem text="Audio">
    <NavItem text="Volume">
      <NavItem text="Master" />
      <NavItem text="Music" />
    </NavItem>
    <NavItem text="Output" />
  </NavItem>
  <NavItem text="Display">
    <NavItem text="Resolution" />
    <NavItem text="Fullscreen" />
    <NavItem text="VSync" />
  </NavItem>
</NavMenu>
```

### 内部信号绑定

NavMenu 支持 `open:NavMenuName`、`close:NavMenuName`、`toggle:NavMenuName` 格式的内部信号绑定：

```html
<Button text="Settings" on_pressed="toggle:NavMenu" />
```

## GdGmlScene API

GdGmlScene 是一个继承 Control 的 GML 文件加载节点，位于 `rust/src/ui/ui_gml_scene.rs`。设置 `gml_file` 属性即可加载 .gml 文件并显示为 Control 节点树。信号自动连接到 GdGmlScene 自身脚本，可创建继承 GdGmlScene 的 GDScript 定义回调方法。

### 导出属性

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `gml_file` | String (FILE) | "" | GML 文件路径（编辑器中显示 .gml 文件选择器），设置后在 ready 时自动加载 |
| `auto_connect` | bool | true | 是否自动连接 GML 中的信号到自身脚本 |

### 方法

| 方法 | 说明 |
|------|------|
| `load_gml()` | 手动加载当前 gml_file 指定的文件 |
| `load_from_string(gml_content: String)` | 从字符串加载 GML 内容 |
| `reload()` | 重新加载 GML 文件 |
| `connect_signals(target: Object)` | 连接 GML 中定义的信号到目标对象 |
| `get_content() -> Control` | 获取内容根节点 |
| `find_node(name: String) -> Control` | 按 name 查找内容中的子节点 |
| `clear_content()` | 清除已加载的内容 |
| `is_loaded() -> bool` | 是否已加载 |
| `apply_theme(theme_name: String)` | 切换主题并重新加载（修改 GML 中的 theme 属性，重新解析构建） |
| `get_builtin_themes() -> PackedStringArray` | 获取所有内置主题名称列表 |
| `on_bean_data_changed(node_name: String, data: Variant)` | GdBean 响应式回调，属性变更时自动更新对应节点 |
| `on_bean_data_changed_bound(data: Variant, _metas: Variant, node_name: String)` | GdBean 响应式回调（bind 版），通过 callable.bind() 注册时使用 |

### 信号

| 信号 | 说明 |
|------|------|
| `s_gml_loaded()` | GML 加载完成 |
| `s_gml_load_failed(error: String)` | GML 加载失败 |

### GDScript 使用示例

```gdscript
# 方式1：在场景中添加 GmlScene 节点，设置 gml_file 属性
# 创建继承 GdGmlScene 的脚本，在其中定义回调方法：
#   extends GdGmlScene
#   func _on_start_game(): ...
# 将脚本挂载到 GmlScene 节点即可

# 方式2：代码创建
var gml = GmlScene.new()
gml.gml_file = "res://example/ui/scene_title.gml"
add_child(gml)
# 信号自动连接到 GdGmlScene 自身脚本

# 方式3：从字符串加载
var gml = GmlScene.new()
gml.auto_connect = false
add_child(gml)
gml.load_from_string("<ui><Label text='Hello' /></ui>")
gml.connect_signals(self)

# 查找子节点
var btn = gml.find_node("StartBtn")
```

## 开发命令

```bash
# 构建 GDExtension（debug 版本，默认）
./build.sh

# 构建 GDExtension（release 版本）
./build.sh release

# 仅编译 Rust 扩展（不安装到 bin 目录）
cargo build -p core
cargo build -p core --release
```

构建脚本会将编译产物安装到 `addons/gamecore/bin/` 对应平台目录下：
- macOS: `addons/gamecore/bin/macos/libgamecore.macos.template_{debug|release}.framework/`
- Linux: `addons/gamecore/bin/linux/libgamecore.linux.template_{debug|release}.x86_64.so`
- Windows: `addons/gamecore/bin/windows/libgamecore.windows.template_{debug|release}.x86_64.dll`

## 文件更新记录

| 日期 | 文件 | 变更内容 |
|------|------|----------|
| 2026-05-29 | rust/Cargo.toml | 重命名为 core crate，添加 smol/async-compat/imagetool/rand 依赖 |
| 2026-05-29 | rust/src/lib.rs | 添加 #[gdextension] 入口、image_tool 和 test_class 模块 |
| 2026-05-29 | rust/src/image_tool.rs | 从 integration_tests 移入 |
| 2026-05-29 | rust/src/test_class.rs | 从 integration_tests 提取 TestClass |
| 2026-05-29 | Cargo.toml | 移除 integration_tests workspace 成员 |
| 2026-05-29 | build.sh | CRATE_NAME 改为 core |
| 2026-05-29 | example/test_from_gd_script.gd | 添加 ImageTool 随机背景图示例 |
| 2026-05-29 | rust/integration_tests/ | 删除整个目录 |
| 2026-05-29 | PROJECT.md | 更新项目结构说明 |
| 2026-05-29 | FILES.md | 更新文件功能索引 |
| 2026-05-30 | rust/src/image_tool.rs | 新增6个UI图片生成方法（generate_ui_rounded_rect/generate_gradient_rect/generate_ui_button/generate_ui_panel/generate_ui_capsule/generate_ui_circle），新增 color_to_rgba 辅助函数 |
| 2026-05-30 | example/ui_test.gd | 新建UI图片生成测试脚本，展示各种UI元素 |
| 2026-05-30 | example/ui_test_scene.tscn | 新建UI图片生成测试场景 |
| 2026-05-30 | rust/src/hud/mod.rs | 新建HUD UI组件模块入口 |
| 2026-05-30 | rust/src/hud/ui_button.rs | 新建UiButton按钮组件 |
| 2026-05-30 | rust/src/hud/ui_card.rs | 新建UiCard卡片布局组件 |
| 2026-05-30 | rust/src/hud/ui_panel.rs | 新建UiPanel面板组件 |
| 2026-05-30 | rust/src/lib.rs | 添加hud模块 |
| 2026-05-30 | rust/src/image_tool.rs | rgba_image_to_texture和color_to_rgba改为pub(crate) |
| 2026-05-30 | rust/src/hud/ui_button.rs | 添加 #[class(tool)] 编辑器预览支持，#[var(set)] 自定义setter实现属性变更自动刷新，on_notification处理RESIZED |
| 2026-05-30 | rust/src/hud/ui_card.rs | 同上 |
| 2026-05-30 | rust/src/hud/ui_panel.rs | 同上 |
| 2026-05-30 | rust/src/state/mod.rs | 新建状态管理模块入口，导出 linklist/gjson/coredata/bean 子模块 |
| 2026-05-30 | rust/src/state/linklist.rs | 新建GdDataLinkList数据链表（继承Resource），封装Dictionary<String,Array> |
| 2026-05-30 | rust/src/state/gjson.rs | 新建GJson纯Rust实现，路径查询/XOR加密/文件持久化/订阅通知 |
| 2026-05-30 | rust/src/state/coredata.rs | 新建GdCoreData核心数据管理器（继承Resource），基于GJson |
| 2026-05-30 | rust/src/state/bean.rs | 新建GdBean数据绑定Bean（继承RefCounted），属性监听/UI绑定/表达式更新 |
| 2026-05-30 | rust/src/lib.rs | 添加state模块 |
| 2026-05-30 | rust/Cargo.toml | 添加serde_json依赖 |
| 2026-05-30 | rust/src/state/gdcore.rs | 新建GDCore全局核心单例（继承RefCounted），注册为Engine singleton "GDCORE" |
| 2026-05-30 | rust/src/state/coredata.rs | build方法改为pub |
| 2026-05-30 | rust/src/lib.rs | 添加on_stage_init/on_stage_deinit注册/注销GDCORE单例 |
| 2026-05-31 | rust/src/state/gdcore.rs | 增加存档ID管理：save_id字段、core_data_cache缓存、set_save_id/get_save_id方法，切换存档时自动通知所有Bean |
| 2026-05-31 | rust/src/state/bean.rs | 新增switch_core方法（GDScript调用）和do_switch_core方法（Rust内部调用），响应存档切换重新加载属性值并触发回调；新增get_all_bean_instances公开函数 |
| 2026-05-31 | rust/Cargo.toml | 添加 gamealgo 和 serde 依赖 |
| 2026-05-31 | rust/src/rogue/mod.rs | 新建肉鸽引擎模块入口，导出 engine/card/card_pile 子模块 |
| 2026-05-31 | rust/src/rogue/engine.rs | 新建RogueEngine核心引擎类（继承RefCounted），管理RogueContext和EntityPool，支持JSON初始化实体模板和卡堆配置 |
| 2026-05-31 | rust/src/rogue/card.rs | 新建RogueCard卡牌包装类（继承RefCounted），暴露卡牌数据给GDScript |
| 2026-05-31 | rust/src/rogue/card_pile.rs | 新建RogueCardPile牌堆包装类（继承RefCounted），暴露牌堆数据和顶牌查询 |
| 2026-05-31 | rust/src/lib.rs | 添加rogue模块 |
| 2026-05-31 | example/rogue/rogue_game.gd | 新建肉鸽卡牌游戏示例脚本，展示完整的卡牌肉鸽游戏逻辑 |
| 2026-05-31 | example/rogue/rogue_game.tscn | 新建肉鸽卡牌游戏示例场景 |
| 2026-06-08 | rust/src/state/coredata.rs | 修复 initial 方法中目录创建 bug：使用 std::path::Path 解析 user:// 路径会错误创建 user: 文件夹，改为字符串解析 Godot 路径 |
| 2026-06-09 | rust/Cargo.toml | 添加 mlua 依赖（lua51/send/vendored） |
| 2026-06-09 | rust/src/console/mod.rs | 新建后台控制台模块入口 |
| 2026-06-09 | rust/src/console/gdconsole.rs | 新建GdConsole全局控制台单例（继承RefCounted），基于mlua的Lua控制台，内置fps/memory/gc_info/cpu_info/help函数，支持GDScript注册命令和Lua脚本执行 |
| 2026-06-09 | rust/src/lib.rs | 添加console模块，注册/注销GdConsole单例 |
| 2026-06-09 | addons/gamecore/ui/console_panel.gd | 新建控制台UI面板（CanvasLayer），输入框+日志输出，按`键切换，命令历史导航 |
| 2026-06-09 | addons/gamecore/core.gd | EditorPlugin自动加载控制台面板 |
| 2026-06-09 | example/console/console_example.gd | 新建控制台示例脚本，演示命令注册（heal/damage/status/set_name/add_score/reset） |
| 2026-06-09 | example/console/console_example.tscn | 新建控制台示例场景 |
| 2026-06-09 | addons/gamecore/ui/dialogue_panel.gd | 新建对话框UI面板，说话人+文本+选项按钮，点击推进/选项选择 |
| 2026-06-09 | example/dialogue/dialogue_example.gd | 新建对话系统示例脚本，加载chat1.txt并启动对话 |
| 2026-06-09 | example/dialogue/dialogue_example.tscn | 新建对话系统示例场景 |
| 2026-06-09 | rust/src/ui/mod.rs | 新建UI标记语言模块入口，导出parser/builder/gdui_builder子模块 |
| 2026-06-09 | rust/src/ui/parser.rs | 新建类HTML标记解析器，将标记文本解析为AST节点树，支持标签/属性/样式块/自闭合标签/注释 |
| 2026-06-09 | rust/src/ui/builder.rs | 新建UI构建器，将AST转换为Godot Control节点树，支持容器/控件实例化、属性设置、StyleBoxFlat样式、信号绑定元数据 |
| 2026-06-09 | rust/src/ui/gdui_builder.rs | 新建GdUiBuilder类（继承RefCounted），暴露parse_string/parse_file/connect_signals/validate API给GDScript |
| 2026-06-09 | rust/src/lib.rs | 添加ui模块 |
| 2026-06-09 | example/ui/ui_example.gd | 新建UI标记语言示例脚本，演示基础布局/样式/信号绑定/复杂布局 |
| 2026-06-09 | example/ui/ui_example.tscn | 新建UI标记语言示例场景 |
| 2026-06-09 | example/ui/sample_ui.gml | 新建示例.gml文件，演示从外部文件加载UI |
| 2026-06-09 | rust/src/ui/ui_list_helper.rs | 新建列表辅助工具，翻译自C++ gmlc/ui_list_helper，包含GdListHelper/GdSlotHighlight/GdSlotFill |
| 2026-06-09 | rust/src/ui/ui_hlist.rs | 新建GdUIHList水平列表节点，翻译自C++ gmlc/ui_list，继承HBoxContainer |
| 2026-06-09 | rust/src/ui/ui_vlist.rs | 新建GdUIVList垂直列表节点，翻译自C++ gmlc/ui_list_v，继承VBoxContainer |
| 2026-06-09 | rust/src/ui/ui_grid.rs | 新建GdUIGrid网格列表节点，翻译自C++ gmlc/ui_list_grid，继承GridContainer |
| 2026-06-09 | rust/src/ui/builder.rs | 更新：添加UIHList/UIVList/UIGrid标签支持和列表属性处理 |
| 2026-06-09 | rust/src/ui/parser.rs | 更新：添加列表标签解析测试用例（共12个测试） |
| 2026-06-09 | example/ui/ui_example.gd | 更新：添加列表扩展节点示例（UIHList/UIVList/UIGrid） |
| 2026-06-10 | addons/gamecore/ui/popup_panel.gd | 新建通用弹窗面板组件（继承CanvasLayer），支持模态遮罩、标题栏+关闭按钮、GML内容构建、显示/隐藏切换 |
| 2026-06-10 | example/ui/scene_title.gml | 新建游戏标题界面GML布局，包含居中按钮组、右上角设置按钮 |
| 2026-06-10 | example/ui/scene_title.gd | 新建游戏标题界面控制器，演示GdUiBuilder+PopupPanel组合使用，包含设置弹窗（音量/全屏/语言表单） |
| 2026-06-10 | rust/src/ui/builder.rs | 新增GML标签支持：CheckButton、HSlider、ColorRect、OptionButton、PopupPanel；新增属性：size_flags_horizontal/vertical、color、toggle_mode、button_pressed、items、selected、popup_title、popup_width、close_on_overlay |
| 2026-06-10 | rust/src/ui/ui_popup_panel.rs | 新建GdPopupPanel弹窗面板节点（继承Control），模态遮罩+标题栏+关闭按钮+内容区域，GML标签<PopupPanel>，替代旧版GDScript popup_panel.gd |
| 2026-06-10 | example/ui/scene_title.gml | 更新：使用PopupPanel标签替代旧版GDScript弹窗，使用HSlider/CheckButton/OptionButton替代SpinBox |
| 2026-06-10 | example/ui/scene_title.gd | 简化：不再依赖popup_panel.gd，直接使用GML中的PopupPanel节点 |
| 2026-06-10 | addons/gamecore/ui/popup_panel.gd | 删除：已被Rust实现的GdPopupPanel替代 |
| 2026-06-10 | rust/src/ui/ui_gml_scene.rs | 新建GdGmlScene节点（继承Control），设置gml_file属性即可加载.gml文件并显示为Control节点树，支持自动信号连接 |
| 2026-06-11 | rust/src/ui/ui_gml_scene.rs | 优化：gml_file属性改为文件引用类型（PropertyHint::FILE + *.gml过滤）；auto_connect改为连接信号到自身脚本而非父节点 |
| 2026-06-11 | example/ui/scene_title_gml.gd | 新建继承GdGmlScene的GDScript，将事件回调函数从scene_title.gd移入 |
| 2026-06-11 | example/ui/scene_title.gd | 简化：移除已迁移到scene_title_gml.gd的回调函数 |
| 2026-06-11 | example/ui/scene_title.tscn | 更新：GmlScene节点挂载scene_title_gml.gd脚本，父节点移除脚本 |
| 2026-06-11 | addons/gamecore/gml_import_plugin.gd | 删除：改用 EditorSettings textfile_extensions 方式注册 .gml 扩展名 |
| 2026-06-11 | addons/gamecore/core.gd | 更新：改用 _register_gml_extension() 将 .gml 添加到编辑器文本文件扩展名列表 |
| 2026-06-11 | rust/src/ui/ui_tooltip.rs | 新建GdUITooltip鼠标跟随提示框节点（继承Control），支持延迟显示、自动位置调整、标题+内容布局，GML标签<Tooltip> |
| 2026-06-11 | rust/src/ui/ui_drawer.rs | 新建GdUIDrawer抽屉面板节点（继承Control），从屏幕边缘滑入/滑出，支持动画过渡、模态遮罩、内容区域，GML标签<Drawer> |
| 2026-06-11 | rust/src/ui/mod.rs | 添加ui_tooltip和ui_drawer模块 |
| 2026-06-11 | rust/src/ui/builder.rs | 注册Tooltip/Drawer标签；处理Drawer/Tooltip子节点添加到内容区域；扩展内部信号绑定支持open:/close:动作；新增Tooltip属性（tooltip_title/tooltip_content/delay/offset_x/offset_y/max_width）和Drawer属性（direction/slide_width/animation_duration/drawer_title） |
| 2026-06-11 | example/ui/scene_main.gml | 新建游戏主界面GML布局，包含底部装备栏（UIHList）、Tooltip提示框、右侧Drawer抽屉面板（含UIGrid） |
| 2026-06-11 | rust/src/ui/ui_hlist.rs | 更新：添加 s_mouse_enter_item / s_mouse_exit_item 信号，绑定 mouse_entered/mouse_exited 事件到子节点 |
| 2026-06-11 | example/ui/scene_main_gml.gd | 新建游戏主界面GML控制器脚本，定义装备栏/背包数据，通过 UIHList.update() 渲染，监听鼠标事件控制 Tooltip |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 `{{key}}` 模板语法检测：apply_attribute 开头检测 `{{...}}` 格式，记录 `__tpl_{key}` 和 `__tpl_keys` 元数据 |
| 2026-06-11 | rust/src/ui/ui_list_helper.rs | 新增模板绑定解析：update_container 中分离简单 key 和路径 key，简单 key 通过 resolve_template_bindings_recursive 递归解析 |
| 2026-06-11 | rust/src/ui/ui_hlist.rs | 新增 tooltip 属性和自动 Tooltip 绑定：鼠标进入/离开子节点时自动从 meta 读取 name/desc 显示提示框 |
| 2026-06-11 | rust/src/ui/ui_grid.rs | 新增 tooltip 属性和自动 Tooltip 绑定（与 UIHList 一致） |
| 2026-06-11 | rust/src/ui/ui_drawer.rs | 修复初始显示灰色全屏遮挡：ready() 中直接设置 visible=false 和 overlay 透明 |
| 2026-06-11 | example/ui/scene_main_gml.gd | 简化数据定义：使用简单 key（icon/count/name/desc），移除手动 Tooltip 信号绑定代码 |
| 2026-06-11 | rust/src/ui/ui_list_helper.rs | 修复 get_meta("__tpl_keys") 报错：添加 has_meta() 检查 |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 tooltip 属性处理（UIHList/UIVList/UIGrid）；新增 data 属性处理（存储为 __data_var 元数据）；列表容器子节点跳过 set_owner 修复 add_child 警告 |
| 2026-06-11 | rust/src/ui/ui_gml_scene.rs | 新增数据自动绑定：auto_bind_data 递归扫描 __data_var 元数据，从脚本读取变量并调用 update() |
| 2026-06-11 | example/ui/scene_main.gml | UIHList/UIGrid 添加 data 属性引用脚本变量名 |
| 2026-06-11 | example/ui/scene_main_gml.gd | 数据内联定义，移除 _update_equip_bar/_update_inventory_grid 函数 |
| 2026-06-11 | rust/src/state/bean.rs | get_bean_by_id 改为 pub(crate) fn，供 ui_gml_scene 模块调用 |
| 2026-06-11 | rust/src/ui/ui_gml_scene.rs | 新增 GdBean 响应式数据绑定：data="bean:bean_id:property_key" 格式从 GdBean 读取数据并注册 watch 回调；新增 on_bean_data_changed 方法 |
| 2026-06-11 | example/ui/scene_main_bean.gd | 新建游戏主界面数据 Bean（继承 GdBean），管理装备栏和背包数据 |
| 2026-06-11 | example/ui/scene_main_gml.gd | 重构：移除内联数据，在 _ready() 中初始化 GdBean 并调用 load_gml() |
| 2026-06-11 | example/ui/scene_main.gml | data 属性改为 bean:scene_main:equip_data 格式 |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 Tab 标签支持（映射为 VBoxContainer）；新增 title 属性处理（Tab 标签的 title 覆盖节点名，TabContainer 用节点名作为 tab 标题）；新增 current_tab/tabs_visible 属性处理（TabContainer） |
| 2026-06-11 | example/ui/scene_gallery.gd | 新建图鉴界面 GML 控制器（继承 GdGmlScene），居中按钮 + PopupPanel + TabContainer（Weapons/Armor/Items 三个 Tab 页，每个含描述文字 + UIGrid） |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 TextureButton 标签支持：实例化、text 属性叠加 Label、texture/texture_normal/texture_pressed/texture_hover/texture_disabled 属性加载纹理、样式系统 texture 属性、文字颜色应用到子 Label、toggle_mode/button_pressed/disabled 属性支持 |
| 2026-06-11 | example/ui/scene_title.gd | 将菜单按钮从 Button 改为 TextureButton，menu-button 样式使用 texture 属性加载 btn_green.png |
| 2026-06-11 | rust/src/ui/ui_nav_menu.rs | 新建GdUINavMenu导航菜单节点（继承Control），支持多级级联菜单、动画过渡、模态遮罩，GML标签<NavMenu>/<NavItem>（递归嵌套） |
| 2026-06-11 | rust/src/ui/mod.rs | 添加ui_nav_menu模块 |
| 2026-06-11 | rust/src/ui/builder.rs | 注册NavMenu/NavItem标签；NavItem的text属性存储为__nav_text meta；新增NavMenu属性（direction/menu_width/sub_menu_width/animation_duration/close_on_overlay）；移除NavSubItem标签 |
| 2026-06-11 | example/ui/scene_setting.gd | 新建设置界面GML控制器（继承GdGmlScene），居中按钮+NavMenu多级级联菜单（Audio含三级/Display二级/Controls二级） |
| 2026-06-12 | example/ui/scene_role.gd | 新建角色界面GML控制器（继承GdGmlScene），居中按钮+PopupPanel角色属性面板（面板左三列装备区+立绘，面板右UIGrid 5x5背包网格+分页） |
| 2026-06-12 | rust/src/ui/ui_gml_scene.rs | 修复find_node无法查找PopupPanel/Drawer/Tooltip内部子节点的bug：改用find_child_ex设置owned=false |
| 2026-06-12 | rust/src/ui/builder.rs | 修复信号绑定中find_child无法查找PopupPanel内部节点的bug：改用find_child_ex设置owned=false |
| 2026-06-12 | rust/src/ui/ui_hlist.rs | 修复Tooltip查找中find_child的owned限制：改用find_child_ex设置owned=false |
| 2026-06-12 | rust/src/ui/ui_grid.rs | 同ui_hlist.rs，修复Tooltip查找中find_child的owned限制 |
| 2026-06-13 | rust/src/ui/builder.rs | 新增百分比自适应语法：parse_percent/parse_size_value辅助函数；apply_size/apply_custom_minimum_size/apply_margin支持百分比，百分比信息存为meta延迟计算；popup_width/slide_width/menu_width/sub_menu_width属性支持百分比 |
| 2026-06-13 | rust/src/ui/ui_gml_scene.rs | 新增百分比布局刷新：refresh_percent_layouts方法处理百分比meta；on_notification监听RESIZED事件自动刷新；refresh_anchors同时刷新百分比布局 |
| 2026-06-13 | example/ui/*.gd/*.gml | 所有GML示例更新为百分比语法（size/custom_minimum_size/margin/popup_width/slide_width/menu_width等） |
| 2026-06-13 | rust/src/ui/ui_theme.rs | 新建UI主题系统：内置配色方案（dark/light/forest/ocean），ThemeVars变量表，resolve_theme_vars变量替换，parse_theme_block解析<theme>块 |
| 2026-06-13 | rust/Cargo.toml | 添加regex-lite依赖 |
| 2026-06-13 | rust/src/ui/parser.rs | 新增<theme>块解析支持；ParseResult新增theme_vars和theme_name字段；新增test_parse_theme_block测试 |
| 2026-06-13 | rust/src/ui/builder.rs | UiBuilder新增theme_vars字段和set_theme_vars方法；apply_class_style中样式属性值替换$var主题变量；apply_root_attribute中theme属性存储为__theme_name meta |
| 2026-06-13 | rust/src/ui/gdui_builder.rs | GdUiBuilder新增theme_name和custom_theme_vars字段；新增set_theme/get_theme/get_builtin_themes/set_theme_var/clear_custom_theme_vars方法；parse_string中注入主题变量 |
| 2026-06-13 | rust/src/ui/ui_gml_scene.rs | GdGmlScene新增theme_name导出属性（默认"dark"）；新增apply_theme一键切换主题方法；新增get_builtin_themes方法；parse_and_build中注入主题变量并存储GML内容到meta |
| 2026-06-13 | rust/src/ui/mod.rs | 添加ui_theme模块 |
| 2026-06-13 | example/ui/*.gd/*.gml | 所有GML示例更新：添加theme="dark"属性，样式颜色值替换为$var主题变量引用 |
| 2026-06-13 | rust/src/ui/ui_theme.rs | 重构主题系统：删除dark/light/forest/ocean四个旧主题，新增cartoon卡通亮色风格主题（淡紫白背景、鲜紫强调、活力橙激活） |
| 2026-06-13 | rust/src/ui/builder.rs | apply_theme_defaults增强卡通风格：所有组件圆角从4提升到12、Panel/Button/LineEdit/OptionButton/TabContainer增加2px边框、Button hover使用accent边框、LineEdit增加focus状态、hover/pressed颜色变化更鲜明 |
| 2026-06-13 | rust/src/ui/parser.rs | 测试用例中theme引用从dark更新为cartoon |
| 2026-06-13 | rust/src/ui/gdui_builder.rs | 注释和文档中theme引用从dark更新为cartoon |
| 2026-06-13 | rust/src/ui/ui_gml_scene.rs | 注释中theme引用从dark/light/forest/ocean更新为cartoon |
| 2026-06-13 | example/ui/*.gd/*.gml | 所有GML示例theme属性从dark/light更新为cartoon |
