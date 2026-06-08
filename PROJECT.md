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

## 项目结构

```
core/
├── Cargo.toml              # Workspace 配置
├── rust-toolchain.toml     # Rust 工具链配置（nightly）
├── build.sh                # GDExtension 构建脚本
├── project.godot           # Godot 项目配置
├── addons/gamecore/            # Godot 插件目录
│   ├── core.gdextension    # GDExtension 配置（统一入口）
│   ├── core.gd             # EditorPlugin 脚本
│   ├── plugin.cfg          # 插件元信息
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
├── example/
│   ├── test_from_gd_script.gd  # GDScript 测试脚本
│   ├── fish_procedural_anim.gd # 鱼的程序化动画示例
│   ├── ui_test.gd              # UI图片生成测试脚本
│   ├── ui_test_scene.tscn      # UI图片生成测试场景
│   └── test_scene.tscn         # 测试场景
│   └── rogue/                  # 肉鸽卡牌游戏示例
│       ├── rogue_game.gd       # 示例游戏脚本
│       └── rogue_game.tscn     # 示例游戏场景
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
