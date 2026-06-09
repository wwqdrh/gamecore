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
- 自动加载控制台面板（_enter_tree 时实例化 console_panel.gd）

### [addons/gamecore/ui/console_panel.gd](file:///Users/dengronghui/project/gamekit/core/addons/gamecore/ui/console_panel.gd)
- 控制台 UI 面板（继承 CanvasLayer）
- 按 ` 键打开/关闭，输入框+日志输出
- 命令历史导航（上下键），监听 GdConsole 的 console_output 信号
- 可配置：toggle_key、console_height_ratio、max_log_lines、font_size

### [addons/gamecore/ui/dialogue_panel.gd](file:///Users/dengronghui/project/gamekit/core/addons/gamecore/ui/dialogue_panel.gd)
- 对话框 UI 面板（继承 CanvasLayer）
- 底部显示说话人名称+对话文本+选项按钮
- 作为 GdDialogue 的 dialogue_control 节点，接收 handle_line 回调
- 点击对话区域推进下一条，选项按钮触发 exec_response
- 可配置：panel_height_ratio、font_size、name_font_size、option_font_size、面板/文字颜色

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

### [rust/src/console/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/console/mod.rs)
- 后台控制台模块入口，导出 gdconsole 子模块

### [rust/src/console/gdconsole.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/console/gdconsole.rs)
- **GdConsole** 类（继承 RefCounted）
- 全局控制台单例，注册为 Engine singleton "GdConsole"
- 基于 mlua (Lua 5.1) 的 Lua 控制台，支持运行时执行 Lua 脚本
- 内置 Lua 函数：fps(), memory(), gc_info(), cpu_info(), help(), print()
- 支持 GDScript 注册命令函数（register_command），在 Lua 中直接按名称调用
- 信号：console_output(text: String)
- 方法：execute, eval, register_command, unregister_command, list_commands

### [rust/src/dialog/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/dialog/mod.rs)
- 对话系统模块入口，导出 gddialogue 子模块

### [rust/src/dialog/gddialogue.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/dialog/gddialogue.rs)
- **GdDialogue** 类（继承 Node）
- 对话控制节点，管理 Timeline 和对话推进
- 基于 gamedialog 库，支持多角色对话、分支控制流、场景变量、条件入口
- 属性：dialogue_control_path, timeline_path, click_next, skip, skip_can_next, skip_time, handle_fn
- 信号：s_finished()
- 方法：next, exec_response, is_registered_role, register_role_node, get_role_pos, initial, goto_stage, all_stages, has_next, stage_index

### [rust/src/ui/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/mod.rs)
- UI标记语言模块入口，导出parser/builder/gdui_builder子模块

### [rust/src/ui/parser.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/parser.rs)
- **类HTML标记解析器**
- 将标记文本解析为AST节点树（UiNode）
- 支持标签/属性/样式块/自闭合标签/注释
- StyleRule：CSS类样式定义
- ParseResult：包含根节点和样式规则

### [rust/src/ui/builder.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/builder.rs)
- **UI构建器**
- 将AST转换为Godot Control节点树
- 支持容器/控件实例化（VBox/HBox/Grid/Margin/Scroll/Tab/Center/PanelContainer + Label/Button/Panel/TextureRect/RichTextLabel/LineEdit/ProgressBar/SpinBox/HSeparator/VSeparator/NinePatchRect）
- 属性设置：text/font_size/align/anchor/margin/size/bbcode/texture/stretch_mode/columns/visible/disabled等
- StyleBoxFlat样式应用：background/border_radius/border_color/border_width/padding/color
- 信号绑定元数据：on_xxx属性存储为__signal_xxx元数据

### [rust/src/ui/gdui_builder.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/gdui_builder.rs)
- **GdUiBuilder** 类（继承 RefCounted）
- UI标记语言GDScript API
- 方法：parse_string, parse_file, connect_signals, validate
- connect_signals：递归遍历节点树，将__signal_xxx元数据连接为信号

### [rust/src/ui/ui_list_helper.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_list_helper.rs)
- 列表辅助工具，翻译自C++ gmlc/ui_list_helper
- GdListHelper：list_initial/update_container/update_data_alias/update_node_value/allbind_signal/update_slot_fill
- GdSlotHighlight：create_square_highlight_node/create_circle_highlight_node（Shader高亮效果）
- GdSlotFill：create_square_fill_node/create_circle_fill_node（Shader填充效果）

### [rust/src/ui/ui_hlist.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_hlist.rs)
- **GdUIHList** 水平列表节点（继承 HBoxContainer），翻译自C++ gmlc/ui_list
- 支持slot模板复制、点击高亮、填充效果
- 属性：count, highlight_mode, highlight_color, fill_mode, fill_color, space_left, space_right
- 信号：s_click_item
- 方法：initial, update, update_all, get_at, get_meta_value, set_width_times, allbind_signal

### [rust/src/ui/ui_vlist.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_vlist.rs)
- **GdUIVList** 垂直列表节点（继承 VBoxContainer），翻译自C++ gmlc/ui_list_v
- 支持slot模板复制、点击高亮、填充效果、鼠标进入/离开事件、随机高度
- 属性：count, highlight_mode, highlight_color, fill_mode, fill_color, enable_random_pos, random_rotate
- 信号：s_click_item, s_mouse_enter_item, s_mouse_exit_item
- 方法：initial, update, update_all, get_at, get_meta_value, set_height_times, allbind_signal

### [rust/src/ui/ui_grid.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_grid.rs)
- **GdUIGrid** 网格列表节点（继承 GridContainer），翻译自C++ gmlc/ui_list_grid
- 支持slot模板复制、点击高亮、填充效果、鼠标进入/离开事件、移动端触摸长按
- 属性：count, highlight_mode, highlight_color, fill_mode, fill_color
- 信号：s_click_item, s_mouse_enter_item, s_mouse_exit_item
- 方法：initial, update, update_all, patch_item, get_at, get_meta_value, allbind_signal

### [vendor/gamedialog/Cargo.toml](file:///Users/dengronghui/project/gamekit/core/vendor/gamedialog/Cargo.toml)
- gamedialog crate 配置，纯 Rust 对话脚本引擎库

### [vendor/gamedialog/src/lib.rs](file:///Users/dengronghui/project/gamekit/core/vendor/gamedialog/src/lib.rs)
- gamedialog 库入口，导出 word/flow/stage/timeline/scene_manager 子模块

### [vendor/gamedialog/src/word.rs](file:///Users/dengronghui/project/gamekit/core/vendor/gamedialog/src/word.rs)
- **DialogueWord** 对话词条结构体
- 包含说话人(name)、文本(text)、所属stage、选项列表(responses)、触发函数列表(functions)

### [vendor/gamedialog/src/flow.rs](file:///Users/dengronghui/project/gamekit/core/vendor/gamedialog/src/flow.rs)
- **ControlFlow** 控制流枚举
- 4种变体：Start(回到开头)、End(终止)、Skip(跳过N个stage)、Goto(跳转到指定stage)
- create_from_string 工厂方法：解析 ":start"/":end"/":skip:N"/":goto:name"

### [vendor/gamedialog/src/stage.rs](file:///Users/dengronghui/project/gamekit/core/vendor/gamedialog/src/stage.rs)
- **DiaStage** 场景阶段结构体
- 核心解析和执行单元，支持脚本语法解析、变量块、条件入口、标签跳转
- **LineVariant** 枚举：Word(DialogueWord) / Flow(ControlFlow)
- **Condition** 条件表达式结构体：variable/op/value/is_global

### [vendor/gamedialog/src/timeline.rs](file:///Users/dengronghui/project/gamekit/core/vendor/gamedialog/src/timeline.rs)
- **Timeline** 时间线结构体
- 管理 DiaStage 序列，提供全局导航（next/has_next/goto_stage/goto_begin/goto_end）
- 支持 precheck 回调（flag 过滤）、控制流执行

### [vendor/gamedialog/src/scene_manager.rs](file:///Users/dengronghui/project/gamekit/core/vendor/gamedialog/src/scene_manager.rs)
- **SceneManager** 场景管理器结构体
- 管理多个 Timeline 和全局变量（非单例设计）

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

### [example/console/console_example.gd](file:///Users/dengronghui/project/gamekit/core/example/console/console_example.gd)
- 控制台命令注册示例脚本，继承 Node2D
- 注册6个GDScript命令：heal/damage/status/set_name/add_score/reset
- 运行时加载控制台面板，按`键打开

### [example/console/console_example.tscn](file:///Users/dengronghui/project/gamekit/core/example/console/console_example.tscn)
- 控制台示例场景，根节点为 Node2D，挂载 console_example.gd 脚本

### [example/dialogue/chat1.txt](file:///Users/dengronghui/project/gamekit/core/example/dialogue/chat1.txt)
- 对话脚本文件，包含 stage0/meetinstreet/meetinschool 三个 stage
- 演示多角色对话、选项分支跳转

### [example/dialogue/dialogue_example.gd](file:///Users/dengronghui/project/gamekit/core/example/dialogue/dialogue_example.gd)
- 对话系统示例脚本，继承 Control
- 创建 GdDialogue 节点和 dialogue_panel 面板
- 加载 chat1.txt 并启动对话，监听 s_finished 信号

### [example/dialogue/dialogue_example.tscn](file:///Users/dengronghui/project/gamekit/core/example/dialogue/dialogue_example.tscn)
- 对话系统示例场景，根节点为 Control，挂载 dialogue_example.gd 脚本

### [example/ui/ui_example.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/ui_example.gd)
- UI标记语言示例脚本，继承 Control
- 演示4个示例：基础布局/带样式UI/信号绑定/复杂布局
- 使用 GdUiBuilder.parse_string() 解析标记字符串
- 使用 GdUiBuilder.connect_signals() 连接信号

### [example/ui/ui_example.tscn](file:///Users/dengronghui/project/gamekit/core/example/ui/ui_example.tscn)
- UI标记语言示例场景，根节点为 Control，挂载 ui_example.gd 脚本

### [example/ui/sample_ui.gml](file:///Users/dengronghui/project/gamekit/core/example/ui/sample_ui.gml)
- 示例 .gml 文件，演示从外部文件加载 UI
- 包含样式定义、面板布局、按钮信号绑定

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
| 2026-06-09 | rust/Cargo.toml | 添加 mlua 依赖（lua51/send/vendored） |
| 2026-06-09 | rust/src/console/mod.rs | 新建后台控制台模块入口 |
| 2026-06-09 | rust/src/console/gdconsole.rs | 新建GdConsole全局控制台单例，基于mlua的Lua控制台 |
| 2026-06-09 | rust/src/lib.rs | 添加console模块，注册/注销GdConsole单例 |
| 2026-06-09 | addons/gamecore/ui/console_panel.gd | 新建控制台UI面板，输入框+日志输出，按`键切换 |
| 2026-06-09 | addons/gamecore/core.gd | EditorPlugin自动加载控制台面板 |
| 2026-06-09 | example/console/console_example.gd | 新建控制台命令注册示例脚本 |
| 2026-06-09 | example/console/console_example.tscn | 新建控制台示例场景 |
| 2026-06-09 | addons/gamecore/ui/dialogue_panel.gd | 新建对话框UI面板，说话人+文本+选项按钮，点击推进/选项选择 |
| 2026-06-09 | example/dialogue/dialogue_example.gd | 新建对话系统示例脚本，加载chat1.txt并启动对话 |
| 2026-06-09 | example/dialogue/dialogue_example.tscn | 新建对话系统示例场景 |
| 2026-06-09 | rust/src/ui/mod.rs | 新建UI标记语言模块入口 |
| 2026-06-09 | rust/src/ui/parser.rs | 新建类HTML标记解析器 |
| 2026-06-09 | rust/src/ui/builder.rs | 新建UI构建器，AST→Control节点树 |
| 2026-06-09 | rust/src/ui/gdui_builder.rs | 新建GdUiBuilder GDScript API类 |
| 2026-06-09 | example/ui/ui_example.gd | 新建UI标记语言示例脚本 |
| 2026-06-09 | example/ui/ui_example.tscn | 新建UI标记语言示例场景 |
| 2026-06-09 | example/ui/sample_ui.gml | 新建示例.gml文件 |
| 2026-06-09 | rust/src/ui/ui_list_helper.rs | 新建列表辅助工具，翻译自C++ gmlc/ui_list_helper |
| 2026-06-09 | rust/src/ui/ui_hlist.rs | 新建GdUIHList水平列表节点，翻译自C++ gmlc/ui_list |
| 2026-06-09 | rust/src/ui/ui_vlist.rs | 新建GdUIVList垂直列表节点，翻译自C++ gmlc/ui_list_v |
| 2026-06-09 | rust/src/ui/ui_grid.rs | 新建GdUIGrid网格列表节点，翻译自C++ gmlc/ui_list_grid |
| 2026-06-09 | rust/src/ui/builder.rs | 更新：添加UIHList/UIVList/UIGrid标签支持 |
| 2026-06-09 | rust/src/ui/parser.rs | 更新：添加列表标签解析测试用例（共12个测试） |
| 2026-06-09 | example/ui/ui_example.gd | 更新：添加列表扩展节点示例 |
