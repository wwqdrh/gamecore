# core 文件功能索引

## 项目配置文件

### [Cargo.toml](file:///Users/dengronghui/project/gamekit/core/Cargo.toml)
- Workspace 配置，包含 rust 成员

### [rust-toolchain.toml](file:///Users/dengronghui/project/gamekit/core/rust-toolchain.toml)
- Rust 工具链配置，指定 nightly 通道

### [rust/Cargo.toml](file:///Users/dengronghui/project/gamekit/core/rust/Cargo.toml)
- core crate 配置
- 依赖：godot 0.5.2, parking_lot, smol, async-compat, imagetool, anim-kit, gamealgo, glam, rand, serde_json, serde

## Godot 项目文件

### [addons/gamecore/core.gdextension](file:///Users/dengronghui/project/gamekit/core/addons/gamecore/core.gdextension)
- GDExtension 统一配置，指定动态库路径和入口符号
- 入口符号：gdext_rust_init
- macOS 使用 framework 格式，其他平台使用 dylib/so/dll 格式

### [addons/gamecore/core.gd](file:///Users/dengronghui/project/gamekit/core/addons/gamecore/core.gd)
- EditorPlugin 脚本

### [addons/gamecore/plugin.cfg](file:///Users/dengronghui/project/gamekit/core/addons/gamecore/plugin.cfg)
- 插件元信息

### [project.godot](file:///Users/dengronghui/project/gamekit/core/project.godot)
- Godot 项目配置

## core crate 源码

### [rust/src/lib.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/lib.rs)
- crate 入口，GDExtension 注册（GameKitCore）
- 模块导出管理，定义 OnFinishCall 枚举和 prelude 模块

### [rust/src/coroutine.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/coroutine.rs)
- SpireCoroutine 协程执行器节点
- 信号：SIGNAL_FINISHED
- 状态查询：IsRunning, IsFinished, IsPaused
- PollMode：Process / PhysicsProcess
- 控制：resume, pause, kill, force_run_to_completion, finish_with

### [rust/src/yielding.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/yielding.rs)
- SpireYield 类型定义
- yield 函数：seconds, frames, wait_while, wait_until, wait_for_signal, wait_for_signal_untyped
- KeepWaiting, WaitUntilFinished

### [rust/src/builder.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/builder.rs)
- CoroutineBuilder 协程构建器
- 配置：auto_start, process_mode, poll_mode
- 回调：on_finish, on_finish_callable
- 执行：spawn

### [rust/src/start_coroutine.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/start_coroutine.rs)
- StartCoroutine trait，为 Gd<Node> 添加 start_coroutine 方法

### [rust/src/start_async_task.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/start_async_task.rs)
- StartAsyncTask trait，为 Gd<Node> 添加 start_async_task 方法

### [rust/src/image_tool.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/image_tool.rs)
- **ImageTool** 类（继承 RefCounted）
- **generate_random_bg**：生成随机背景图（随机图案类型和颜色），返回 ImageTexture
- **generate_bg**：生成指定参数的背景图（pattern_type/bg_color/fg_color/rotation），返回 ImageTexture
- **generate_random_handdraw**：生成随机手绘图形（随机形状和参数），返回 ImageTexture
- **generate_handdraw_rectangle**：手绘矩形（位置/尺寸/粗糙度/描边色/填充色/种子），返回 ImageTexture
- **generate_handdraw_circle**：手绘圆形（中心/直径/粗糙度/描边色/填充色/种子），返回 ImageTexture
- **generate_handdraw_line**：手绘直线（起终点/粗糙度/描边色/种子），返回 ImageTexture
- **generate_handdraw_ellipse**：手绘椭圆（中心/宽高/粗糙度/描边色/填充色/种子），返回 ImageTexture
- **rgba_image_to_texture**：内部函数，将 imagetool 的 RgbaImage 转换为 Godot ImageTexture
- **generate_ui_rounded_rect**：生成圆角矩形UI图片（宽高/圆角/背景色/边框宽/边框色），返回 ImageTexture
- **generate_ui_gradient_rect**：生成渐变圆角矩形UI图片（宽高/圆角/两色/角度/边框宽/边框色），返回 ImageTexture
- **generate_ui_button**：生成带阴影按钮UI图片（宽高/圆角/背景色/边框/阴影偏移/模糊/阴影色），返回 ImageTexture
- **generate_ui_panel**：生成面板UI图片（宽高/圆角/背景色/边框宽/边框色），返回 ImageTexture
- **generate_ui_capsule**：生成胶囊UI图片（宽高/背景色/边框宽/边框色），返回 ImageTexture
- **generate_ui_circle**：生成圆形UI图片（尺寸/背景色/边框宽/边框色），返回 ImageTexture
- **color_to_rgba**：内部函数，将 Godot Color 转换为 [u8; 4]

### [rust/src/test_class.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/test_class.rs)
- **TestClass** 类（继承 Node）
- **test_routine**：异步任务测试，使用 async-compat 兼容层
- **test_from_other_node**：静态方法，在其他节点上启动异步任务

### [rust/src/anim_graph.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/anim_graph.rs)
- **AnimGraph** 类（继承 Node）
- **load_config**：从 JSON 字符串加载动画配置
- **update**：手动更新一帧动画
- **get_bone_rotation**：获取骨骼旋转（四元数）
- **get_bone_position**：获取骨骼位置
- **get_bone_rotation_euler**：获取骨骼旋转（欧拉角）
- **get_bone_names**：获取所有骨骼名称列表
- **set_wind_strength / get_wind_strength**：风力控制
- **set_speed_factor / get_speed_factor**：速度因子控制
- **set_ik_target**：设置 IK 目标位置
- **set_bone_parameter**：设置骨骼参数
- **get_global_time**：获取全局动画时间

### [rust/src/hud/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/hud/mod.rs)
- HUD UI组件模块入口，导出 ui_button、ui_card、ui_panel 子模块

### [rust/src/state/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/state/mod.rs)
- 状态管理模块入口，导出 linklist、gjson、coredata、bean 子模块

### [rust/src/state/linklist.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/state/linklist.rs)
- **GdDataLinkList** 类（继承 Resource）
- 数据链表，封装 Dictionary<String, Array> 的增删查序列化
- 方法：from_json, to_json, get_list, has, add_one

### [rust/src/state/gjson.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/state/gjson.rs)
- **GJson** 纯 Rust JSON 文档存储（非 Godot 类）
- FileStore：文件持久化，支持自定义 load/save 闭包
- 路径查询：`;` 分隔嵌套路径（如 `"init;player;health"`）
- XOR 加密/解密，ENCRYPT_KEY 常量
- 订阅通知：subscribe/notify 模式
- update 方法：action `"~"` 强制设置

### [rust/src/state/coredata.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/state/coredata.rs)
- **GdCoreData** 类（继承 Resource）
- 核心数据管理器，基于 GJson 实现数据存取
- 加密存档：save/load 支持 XOR 加密
- 作用域隔离：scope 参数区分不同数据域
- 订阅通知：subscribe/unsubscribe/notify
- 方法：value, update, has, change, save_data, load_data, subscribe, unsubscribe, notify, get_root_data

### [rust/src/state/bean.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/state/bean.rs)
- **GdBean** 类（继承 RefCounted）
- 数据绑定 Bean，持有 GdCoreData 引用作为数据后端
- 全局实例管理：BEAN_INSTANCES（LazyLock + Mutex）
- 属性监听：watch/watch_property/check_property_val
- UI 绑定：bind_node_text/emit_node_text
- 表达式更新：update_by_expression（支持 +、-、*、/、= 操作，@ 跨 Bean 引用）
- 存档管理：flush/reinit/to_dict/switch_core
- 存档切换：switch_core（GDScript调用）/ do_switch_core（Rust内部调用），切换GdCoreData后重新加载属性值并触发watch回调和on_save_switch
- 方法：bean, initial, set_excludes, set_force, set_scope, bind_node_text, emit_node_text, to_dict, emit, watch, watch_property, update, updates, get_value_by_key, patch_value, flush, reinit, update_by_expression, switch_core

### [rust/src/state/gdcore.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/state/gdcore.rs)
- **GDCore** 类（继承 RefCounted）
- 全局核心单例，注册为 Engine singleton "GDCORE"
- 在 ExtensionLibrary::on_stage_init(InitStage::Scene) 时注册，on_stage_deinit 时注销
- 使用 std::mem::forget 防止 RefCounted 提前释放（与 C++ memnew 行为一致）
- 存档 ID 管理：save_id 字段、core_data_cache 缓存（HashMap<String, Gd<GdCoreData>>）
- 存档文件路径：user://coredata_{id}.data（id 为空时为 user://coredata.data）
- 切换存档时自动通知所有 GdBean 实例调用 do_switch_core 更新数据
- 方法：get_root_data(), get_save_id(), set_save_id(id)

### [rust/src/hud/ui_button.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/hud/ui_button.rs)
- **UiButton** 类（继承 Control）
- 程序化样式按钮，内部TextureButton + Label
- 支持圆角矩形/胶囊/圆形（shape_type: 0/1/2）
- 自动生成 normal/pressed/hover/disabled 四种状态纹理
- 信号：pressed()
- 方法：refresh_style(), update_button_text(), get_internal_button()

### [rust/src/hud/ui_card.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/hud/ui_card.rs)
- **UiCard** 类（继承 Control）
- 卡片布局组件，内部TextureRect(背景) + MarginContainer + VBoxContainer(标题+图片+描述)
- 支持圆角/边框/阴影/内边距/间距配置
- 方法：refresh_style(), set_card_image(), set_title(), set_description()

### [rust/src/hud/ui_panel.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/hud/ui_panel.rs)
- **UiPanel** 类（继承 Control）
- 程序化样式面板，内部TextureRect(背景)
- 支持圆角矩形/胶囊/圆形（shape_type: 0/1/2）
- 方法：refresh_style()

### [rust/src/rogue/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/rogue/mod.rs)
- 肉鸽引擎模块入口，导出 engine/card/card_pile 子模块

### [rust/src/rogue/engine.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/rogue/engine.rs)
- **RogueEngine** 类（继承 RefCounted）
- 核心引擎类，管理 RogueContext（种子/深度）和 EntityPool（实体模板池）
- 支持通过 JSON 初始化实体模板和卡堆配置
- 方法：init_with_seed, get_seed, get_depth, set_depth, advance_depth, load_entities_from_json, generate_piles, generate_entity, roll_entity, get_snapshot_json, restore_from_json
- 内部函数：parse_entity_template, parse_stat_scale, parse_card_pile_config, pack_layout_to_gd, pack_card_to_gd, pack_entity_to_gd

### [rust/src/rogue/card.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/rogue/card.rs)
- **RogueCard** 类（继承 RefCounted）
- 卡牌包装类，将 gamealgo Card 数据暴露给 Godot
- 方法：get_card_id, get_template_id, get_name, get_entity_type, get_stats, get_stat, is_face_up, is_monster, is_weapon, is_armor, is_item, is_exit
- 工厂方法：from_dict（从 VarDictionary 构造）

### [rust/src/rogue/card_pile.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/rogue/card_pile.rs)
- **RogueCardPile** 类（继承 RefCounted）
- 牌堆包装类，将 gamealgo CardPile 数据暴露给 Godot
- 方法：get_pile_id, get_card_count, get_top_card, has_exit_card, get_all_cards
- 工厂方法：from_dict（从 VarDictionary 构造）

## GDScript 示例

### [example/test_from_gd_script.gd](file:///Users/dengronghui/project/gamekit/core/example/test_from_gd_script.gd)
- GDScript 测试脚本，继承 TestClass
- 协程调用示例（test_routine, test_from_other_node）
- ImageTool 随机背景图生成并赋值给 TextureRect 示例

### [example/fish_procedural_anim.gd](file:///Users/dengronghui/project/gamekit/core/example/fish_procedural_anim.gd)
- 鱼的程序化动画示例，继承 Node2D
- 使用 AnimGraph 加载 JSON 配置驱动鱼身、鱼尾、鱼鳍、鱼嘴动画
- 用 ColorRect 搭建鱼的视觉部件，每帧读取骨骼旋转应用到 2D 节点
- 支持键盘控制速度（↑加速、↓减速）

### [example/ui_test.gd](file:///Users/dengronghui/project/gamekit/core/example/ui_test.gd)
- UI图片生成测试脚本，继承 Node2D
- 展示圆角矩形按钮、渐变按钮、带阴影按钮、面板、胶囊、圆形等UI元素
- 使用 ImageTool 的 UI 生成方法创建纹理，通过 TextureRect 显示

### [example/ui_test_scene.tscn](file:///Users/dengronghui/project/gamekit/core/example/ui_test_scene.tscn)
- UI图片生成测试场景，根节点为 Node2D

### [example/test_scene.tscn](file:///Users/dengronghui/project/gamekit/core/example/test_scene.tscn)
- 测试场景，根节点为 TestClass

### [example/rogue/rogue_game.gd](file:///Users/dengronghui/project/gamekit/core/example/rogue/rogue_game.gd)
- 肉鸽卡牌游戏示例脚本，继承 Control
- 使用 RogueEngine 初始化种子、加载实体模板、生成卡堆
- _draw() 绘制牌堆UI、玩家状态、顶牌信息
- 鼠标点击选牌、键盘操作（R重新开始、N下一层）
- 战斗/装备/药水/出口等事件处理

### [example/rogue/rogue_game.tscn](file:///Users/dengronghui/project/gamekit/core/example/rogue/rogue_game.tscn)
- 肉鸽卡牌游戏示例场景，根节点为 Control，挂载 rogue_game.gd 脚本

## 构建脚本

### [build.sh](file:///Users/dengronghui/project/gamekit/core/build.sh)
- GDExtension 构建脚本，支持 debug/release 两种模式
- 自动检测平台（macOS/Linux/Windows）
- macOS: 构建 framework 格式（含 Info.plist），安装到 addons/gamecore/bin/macos/
- Linux: 构建 .so，安装到 addons/gamecore/bin/linux/
- Windows: 构建 .dll，安装到 addons/gamecore/bin/windows/
- 用法: `./build.sh [debug|release]`

## 文档文件

### [PROJECT.md](file:///Users/dengronghui/project/gamekit/core/PROJECT.md)
- 项目概述和核心功能说明
- ImageTool API 文档和 GDScript 使用示例

### [FILES.md](file:///Users/dengronghui/project/gamekit/core/FILES.md)
- 本文档，提供所有文件的功能索引

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
| 2026-05-30 | rust/src/image_tool.rs | 新增6个UI图片生成方法和 color_to_rgba 辅助函数 |
| 2026-05-30 | example/ui_test.gd | 新建UI图片生成测试脚本 |
| 2026-05-30 | example/ui_test_scene.tscn | 新建UI图片生成测试场景 |
| 2026-05-30 | rust/src/hud/mod.rs | 新建HUD UI组件模块入口 |
| 2026-05-30 | rust/src/hud/ui_button.rs | 新建UiButton按钮组件 |
| 2026-05-30 | rust/src/hud/ui_card.rs | 新建UiCard卡片布局组件 |
| 2026-05-30 | rust/src/hud/ui_panel.rs | 新建UiPanel面板组件 |
| 2026-05-30 | rust/src/hud/ui_button.rs | 添加 #[class(tool)] 编辑器预览、#[var(set)] 自定义setter、on_notification |
| 2026-05-30 | rust/src/hud/ui_card.rs | 同上 |
| 2026-05-30 | rust/src/hud/ui_panel.rs | 同上 |
| 2026-05-30 | rust/src/state/mod.rs | 新建状态管理模块入口 |
| 2026-05-30 | rust/src/state/linklist.rs | 新建GdDataLinkList数据链表 |
| 2026-05-30 | rust/src/state/gjson.rs | 新建GJson纯Rust JSON文档存储 |
| 2026-05-30 | rust/src/state/coredata.rs | 新建GdCoreData核心数据管理器 |
| 2026-05-30 | rust/src/state/bean.rs | 新建GdBean数据绑定Bean |
| 2026-05-30 | rust/src/lib.rs | 添加state模块 |
| 2026-05-30 | rust/Cargo.toml | 添加serde_json依赖 |
| 2026-05-30 | rust/src/state/gdcore.rs | 新建GDCore全局核心单例 |
| 2026-05-30 | rust/src/state/coredata.rs | build方法改为pub |
| 2026-05-30 | rust/src/lib.rs | 添加on_stage_init/on_stage_deinit注册/注销GDCORE单例 |
| 2026-05-31 | rust/src/state/gdcore.rs | 增加存档ID管理：save_id字段、core_data_cache缓存、set_save_id/get_save_id方法，切换存档时自动通知所有Bean |
| 2026-05-31 | rust/src/state/bean.rs | 新增switch_core/do_switch_core方法响应存档切换，新增get_all_bean_instances公开函数 |
| 2026-05-31 | rust/Cargo.toml | 添加 gamealgo 和 serde 依赖 |
| 2026-05-31 | rust/src/rogue/mod.rs | 新建肉鸽引擎模块入口 |
| 2026-05-31 | rust/src/rogue/engine.rs | 新建RogueEngine核心引擎类 |
| 2026-05-31 | rust/src/rogue/card.rs | 新建RogueCard卡牌包装类 |
| 2026-05-31 | rust/src/rogue/card_pile.rs | 新建RogueCardPile牌堆包装类 |
| 2026-05-31 | rust/src/lib.rs | 添加rogue模块 |
| 2026-05-31 | example/rogue/rogue_game.gd | 新建肉鸽卡牌游戏示例脚本 |
| 2026-05-31 | example/rogue/rogue_game.tscn | 新建肉鸽卡牌游戏示例场景 |
| 2026-06-08 | rust/src/state/coredata.rs | 修复 initial 方法中目录创建 bug：使用 std::path::Path 解析 user:// 路径会错误创建 user: 文件夹，改为字符串解析 Godot 路径 |
