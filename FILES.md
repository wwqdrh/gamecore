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
- 注册 .gml 扩展名到编辑器文本文件列表（使 .gml 文件在 FileSystem 中可见）

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

### [rust/src/anim/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/anim/mod.rs)
- 动画模块入口，导出 easing/juice/easy_move/transition 子模块
- 公开导出：Easing 枚举、juice::* 所有动画函数、EaseMover、TransitionFade、FadeType

### [rust/src/anim/easing.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/anim/easing.rs)
- **Easing** 枚举（31种缓动类型）
- 方法：get_progress(self, x: f32) -> f32
- 变体：Linear, SineIn/Out/InOut, QuadIn/Out/InOut, CubicIn/Out/InOut, QuartIn/Out/InOut, QuintIn/Out/InOut, ExpoIn/Out/InOut, CircIn/Out/InOut, BackIn/Out/InOut, ElasticIn/Out/InOut, BounceIn/Out/InOut

### [rust/src/anim/juice.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/anim/juice.rs)
- **核心动画系统**，移植自 C++ juice/juice.h/cpp
- 所有动画基于 Godot Tween 系统，使用 tween_property + 内置缓动类型（gdext 0.5 的 tween_method 不支持 Rust 闭包）
- 防重入机制：check_is_handle/mark_handle/anim_handle_finished（基于 Godot meta）
- 清理回调：make_cleanup_callable（使用 Callable::from_fn，因为 Gd<Node> 不是 Send）
- Easing 映射：easing_to_trans（Easing→TransitionType）、easing_to_ease_type（Easing→EaseType）
- 公开动画函数（简化签名，移除 Easing 参数，使用 Godot 内置缓动）：
  - anim_wave_simple(target, wave_offset, duration)：波浪摆动（循环，SINE/IN_OUT）
  - anim_rotate_circle_simple(target, duration)：360度旋转（循环，LINEAR）
  - anim_shake_simple(target, duration)：左右抖动（循环，SINE/IN_OUT）
  - anim_move_straight_simple(target, to_pos, duration, ease_type)：直线移动
  - anim_bounce_simple(target, dire_pos, distance, duration)：弹跳（BACK/OUT）
  - do_scale_simple(target, target_scale, duration)：缩放（BACK/OUT）
  - anim_breath_simple(target, breath_factor, anim_span, duration)：呼吸缩放（循环，SINE/IN_OUT）
  - anim_walk_simple(target, walk_span, duration)：走路摆动（循环，rotation+scale，SINE/IN_OUT）
  - anim_enter(target, container, direction, duration, delay, mode)：入场动画（从界面外移入+透明度变化，按分组依次执行）
  - anim_explosion(target, max_distance, duration)：爆炸散开+淡出（BACK/IN，子节点动画）
  - anim_collect(target, target_position, is_global, duration)：收集汇聚（SINE/IN_OUT，子节点动画）
  - hit_label(target, duration)：打击标签（缩放+颜色闪烁+上浮淡出）
  - popup_enter(target: &Gd<Control>, duration)：弹窗弹入（BACK/OUT 缩放+淡入）
  - popup_exit(target: &Gd<Control>, duration)：弹窗弹出（SINE/IN 缩放+淡出）
  - click_feedback(target: &Gd<Control>)：点击缩放反馈（BACK/OUT 弹回）
  - tooltip_fade_in(target: &Gd<Control>, duration)：Tooltip淡入（SINE/OUT 透明+BACK/OUT 缩放）
  - tooltip_fade_out(target: &Gd<Control>, duration)：Tooltip淡出（SINE/IN 透明+缩放）

### [rust/src/anim/easy_move.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/anim/easy_move.rs)
- **EaseMover** 平滑移动工具

### [rust/src/anim/transition.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/anim/transition.rs)
- **TransitionFade** 过渡效果
- **FadeType** 淡入淡出类型枚举

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

### [rust/src/manager/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/manager/mod.rs)
- 场景管理模块入口，导出 gd_scene、gd_scene_root、config_manager 子模块

### [rust/src/manager/gd_scene.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/manager/gd_scene.rs)
- **GdScene** 类（继承 Control）
- 场景页面节点，提供生命周期回调和状态管理
- 生命周期：on_enter(data) → on_ready() → on_exit()
- 独立模式：没有 GdSceneRoot 时自动搜索或创建默认实例，避免崩溃
- 受管模式：on_ready 由 GdSceneRoot 在转场完成后调用
- 属性：scene_id, current_state, init_data（均为 #[export]）
- 信号：s_state_changed(state, data)
- 方法：change_state, back_state, root_call_ready, get_manager, get_state_stack, get_state_init_data, change_scene, change_scene_with_state, change_scene_no_anim, back_scene, restart_scene

### [rust/src/manager/gd_scene_root.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/manager/gd_scene_root.rs)
- **GdSceneRoot** 类（继承 Node）
- 场景管理器，负责场景注册、切换、转场动画
- 转场动画：ColorRect 遮罩 + Tween 淡入淡出，通过 _process 驱动状态机
- 防重入：is_changing 标志防止并发场景切换
- 场景历史栈：支持 back_scene 回退和 restart_scene 重启
- 从 GdConfigManager 自动加载 scenes 配置
- 属性：trans_duration, manager_id
- 信号：s_trans_closed, s_trans_opened
- 方法：register_scene, register_scenes, change_scene, back_scene, restart_scene, get_current_scene, get_registered_scenes, get_scene_stack, is_changing_scene

### [rust/src/manager/config_manager.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/manager/config_manager.rs)
- **GdConfigManager** 类（继承 Node）
- 通用配置管理器，启动时自动读取 res://game_config.json
- 内置 scenes 属性（alias → PackedScene），支持自定义配置项
- 通过 GDCORE 的 add_global_node 注册为全局节点（alias: "config"）
- 在 GDCore 单例注册时自动创建并加载配置，确保 GdSceneRoot 能获取到配置
- 属性：config_path（#[export]，默认 "res://game_config.json"）
- 方法：get_config, set_config, get_scene, get_scene_aliases, get_config_data, reload_config

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
- UI标记语言模块入口，导出parser/builder/gdui_builder/ui_theme/ui_popup_panel/ui_tooltip/ui_drawer/ui_nav_menu/ui_gml_scene/ui_list_helper/ui_hlist/ui_vlist/ui_grid子模块

### [rust/src/ui/ui_theme.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_theme.rs)
- **UI 主题系统**
- 内置配色方案：dark/light/forest/ocean
- 主题变量定义（ThemeVars = HashMap<String, String>）
- 变量替换：resolve_theme_vars() 将 $var_name 替换为变量值
- 解析 <theme> 块：parse_theme_block() 解析自定义主题变量
- 获取内置主题：get_builtin_theme() / builtin_theme_names()

### [rust/src/ui/parser.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/parser.rs)
- **类HTML标记解析器**
- 将标记文本解析为AST节点树（UiNode）
- 支持标签/属性/样式块/主题块/自闭合标签/注释
- StyleRule：CSS类样式定义
- ParseResult：包含根节点、样式规则和主题变量

### [rust/src/ui/builder.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/builder.rs)
- **UI构建器**
- 将AST转换为Godot Control节点树
- 支持容器/控件实例化（VBox/HBox/Grid/Margin/Scroll/Tab/Center/PanelContainer/Tab + Label/Button/TextureButton/CheckButton/HSlider/ColorRect/OptionButton/Panel/TextureRect/RichTextLabel/LineEdit/ProgressBar/SpinBox/HSeparator/VSeparator/NinePatchRect/PopupPanel/Tooltip/Drawer/NavMenu/NavItem）
- 属性设置：text/font_size/align/anchor/margin/size/bbcode/texture/texture_normal/texture_pressed/texture_hover/texture_disabled/stretch_mode/columns/visible/disabled/size_flags_horizontal/size_flags_vertical/color/toggle_mode/button_pressed/items/selected/popup_title/popup_width/close_on_overlay/tooltip_title/tooltip_content/delay/offset_x/offset_y/max_width/direction/slide_width/animation_duration/drawer_title/title/current_tab/tabs_visible/patch_margin等
- TextureButton 子 Label 自动配置：当 Label 作为 TextureButton 的子节点时，自动设置锚点填满父节点、文字居中、鼠标穿透
- NinePatchRect 按钮模式：有 on_pressed 属性时自动添加不可见 Button 子节点处理点击事件；子 Label 自动配置锚点填满、文字居中、鼠标穿透；class style texture 属性设置纹理；patch_margin 属性设置九宫格边距
- 模板绑定：`{{key}}` 语法检测，记录 `__tpl_{key}`/`__tpl_keys`/`__tpl_attr` 元数据
- StyleBoxFlat样式应用：background/border_radius/border_color/border_width/padding/color/texture，支持 `$var` 主题变量替换
- 信号绑定元数据：on_xxx属性存储为__signal_xxx元数据
- 主题变量：UiBuilder 持有 ThemeVars，构建时自动替换样式属性中的 $var 引用

### [rust/src/ui/gdui_builder.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/gdui_builder.rs)
- **GdUiBuilder** 类（继承 RefCounted）
- UI标记语言GDScript API
- 方法：parse_string, parse_file, connect_signals, validate, set_theme, get_theme, get_builtin_themes, set_theme_var, clear_custom_theme_vars
- connect_signals：递归遍历节点树，将__signal_xxx元数据连接为信号
- 主题支持：set_theme 设置内置主题，set_theme_var 设置自定义变量，parse 时自动注入

### [rust/src/ui/ui_popup_panel.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_popup_panel.rs)
- **GdPopupPanel** 类（继承 Control）
- 弹窗面板节点，替代旧版GDScript popup_panel.gd
- 模态遮罩（点击外部关闭）+ 标题栏 + 关闭按钮 + 内容区域
- GML标签：`<PopupPanel>`
- 属性：popup_width, popup_title, close_on_overlay, popup_bg_color, popup_border_color, overlay_color, title_font_size, title_color, corner_radius
- 方法：show_popup, hide_popup, is_popup_visible, toggle_popup, update_popup_title, get_content_path
- 信号：s_popup_shown, s_popup_hidden

### [rust/src/ui/ui_tooltip.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_tooltip.rs)
- **GdUITooltip** 类（继承 Control）
- 鼠标跟随提示框节点，浮动面板跟随鼠标位置显示
- 支持延迟显示、自动位置调整（避免超出屏幕）、标题+内容布局
- 支持自定义子节点（GML 中定义 Label 等），通过 update_data 解析 {{key}} 模板绑定
- 添加自定义子节点时自动移除内置 title/content/separator
- GML标签：`<Tooltip>`
- 属性：tooltip_title_text, tooltip_content_text, delay, offset_x, offset_y, max_width, max_height, bg_color, border_color, title_color, content_color, corner_radius
- 方法：show_tooltip, hide_tooltip, set_tooltip_title, set_tooltip_content, update_data, ensure_ui_built, add_content_child
- 信号：s_tooltip_shown, s_tooltip_hidden

### [rust/src/ui/ui_drawer.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_drawer.rs)
- **GdUIDrawer** 类（继承 Control）
- 抽屉面板节点，从屏幕边缘滑入/滑出
- 支持动画过渡（ease-out cubic）、模态遮罩、标题栏+关闭按钮、内容区域
- GML标签：`<Drawer>`
- 属性：direction, slide_width, overlay_color, drawer_bg_color, drawer_border_color, corner_radius, animation_duration, close_on_overlay, drawer_title_text
- 方法：open, close, toggle, is_drawer_open, set_drawer_title
- 信号：s_drawer_opened, s_drawer_closed

### [rust/src/ui/ui_nav_menu.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_nav_menu.rs)
- **GdUINavMenu** 类（继承 Control）
- 导航菜单节点，支持多级级联菜单
- 从屏幕左侧/右侧滑入，支持动画过渡（ease-out cubic）和模态遮罩
- GML标签：`<NavMenu>`，子标签：`<NavItem>`（递归嵌套，支持多级菜单）
- 属性：direction, menu_width, sub_menu_width, menu_bg_color, menu_border_color, overlay_color, corner_radius, animation_duration, close_on_overlay, item_font_size, item_color, item_hover_color, item_active_color, sub_item_font_size, sub_item_color, sub_item_hover_color
- 方法：open, close, toggle, is_menu_open, ensure_ui_built
- 信号：s_menu_opened, s_menu_closed, s_item_clicked(path: GString)

### [rust/src/ui/ui_gml_scene.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_gml_scene.rs)
- **GdGmlScene** 类（继承 Control）
- GML 文件加载节点，设置 gml_file 属性即可加载 .gml 文件并显示为 Control 节点树
- 属性：gml_file（GML文件路径，编辑器中显示 .gml 文件选择器）, auto_connect（自动连接信号到自身脚本）
- 主题来源：由 GML 中 <ui theme="xxx"> 属性决定，不暴露 theme_name 导出属性
- 主题切换：apply_theme() 修改 GML 中的 theme 属性并重新加载，get_builtin_themes() 获取内置主题列表
- 数据自动绑定：加载后扫描 __data_var 元数据，支持两种格式：
  - 简单变量名（如 `data="equip_data"`）：从脚本对象读取变量
  - GdBean 引用（如 `data="bean:scene_main:equip_data"`）：从 GdBean 实例读取属性值，支持响应式更新
- GdBean 响应式绑定：通过 bean.watch() 注册回调，属性变更时自动调用 on_bean_data_changed_bound() 更新节点
- 动画自动配置：加载后扫描 __anim_enter/__anim_hover/__anim_click 元数据，自动设置入场/悬停/点击动画
  - anim_enter：入场动画（滑入+淡入，方向 bottom/top/left/right，同级兄弟自动 stagger 递增延迟）
  - anim_hover：悬停动画（鼠标进入放大+提亮，离开恢复，可配置缩放倍数）
  - anim_click：点击反馈（缩小弹回，NinePatchRect 通过 __click_handler 子节点检测交互）
- 方法：load_gml, load_from_string, reload, connect_signals, get_content, find_node, clear_content, is_loaded, on_bean_data_changed, on_bean_data_changed_bound
- 信号：s_gml_loaded, s_gml_load_failed

### [rust/src/ui/ui_list_helper.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_list_helper.rs)
- 列表辅助工具，翻译自C++ gmlc/ui_list_helper
- GdListHelper：list_initial/update_container/update_data_alias/update_node_value/allbind_signal/update_slot_fill
- 模板绑定解析：update_container 中分离简单 key 和路径 key（含 `:` 或 `/` 的为路径 key），简单 key 通过 resolve_template_bindings_recursive 递归解析
- 未被模板绑定使用的简单 key 自动存储为 meta（供 Tooltip 读取 name/desc）
- 完整数据字典存储为 __item_data meta（供 Tooltip 的 update_data 方法使用）
- GdSlotHighlight：create_square_highlight_node/create_circle_highlight_node（Shader高亮效果）
- GdSlotFill：create_square_fill_node/create_circle_fill_node（Shader填充效果）

### [rust/src/ui/ui_hlist.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_hlist.rs)
- **GdUIHList** 水平列表节点（继承 HBoxContainer），翻译自C++ gmlc/ui_list
- 支持slot模板复制、点击高亮、填充效果、鼠标进入/离开事件
- 属性：count, highlight_mode, highlight_color, fill_mode, fill_color, space_left, space_right, tooltip
- 信号：s_click_item, s_mouse_enter_item, s_mouse_exit_item
- 方法：initial, update, update_all, get_at, get_meta_value, set_width_times, allbind_signal
- Tooltip 自动绑定：tooltip 属性指定 Tooltip 节点名，鼠标进入/离开子节点时自动从 __item_data meta 读取完整数据字典调用 update_data，兼容内置 title/content label

### [rust/src/ui/ui_vlist.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_vlist.rs)
- **GdUIVList** 垂直列表节点（继承 VBoxContainer），翻译自C++ gmlc/ui_list_v
- 支持slot模板复制、点击高亮、填充效果、鼠标进入/离开事件、随机高度
- 属性：count, highlight_mode, highlight_color, fill_mode, fill_color, enable_random_pos, random_rotate
- 信号：s_click_item, s_mouse_enter_item, s_mouse_exit_item
- 方法：initial, update, update_all, get_at, get_meta_value, set_height_times, allbind_signal

### [rust/src/ui/ui_grid.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/ui/ui_grid.rs)
- **GdUIGrid** 网格列表节点（继承 GridContainer），翻译自C++ gmlc/ui_list_grid
- 支持slot模板复制、点击高亮、填充效果、鼠标进入/离开事件、移动端触摸长按
- 属性：count, highlight_mode, highlight_color, fill_mode, fill_color, tooltip
- 信号：s_click_item, s_mouse_enter_item, s_mouse_exit_item
- 方法：initial, update, update_all, patch_item, get_at, get_meta_value, allbind_signal
- Tooltip 自动绑定：tooltip 属性指定 Tooltip 节点名，鼠标进入/离开子节点时自动从 __item_data meta 读取完整数据字典调用 update_data，兼容内置 title/content label

### [rust/src/map/mod.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/map/mod.rs)
- 双网格地图模块入口，导出 dual_grid/dual_grid_iso/gd_map_basic/gd_map_isometric 子模块
- 公开导出：TerrainType, TerrainThresholds, PropConfig, PropPlacement, DualGrid, place_props, GdMapBasic, IsoTerrainType, IsoTerrainThresholds, IsoPropConfig, IsoPropPlacement, DualGridIso, iso_place_props, GdMapIsometric

### [rust/src/map/dual_grid.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/map/dual_grid.rs)
- **方形双网格算法核心逻辑**（纯 Rust，不依赖 Godot API）
- TerrainType 枚举：动态 ID，0 = Null，1+ = 用户注册
- CornerState 枚举：Null/NotNull，用于四角组合
- 16种四角组合查找表：build_tile_lookup()，映射 CornerKey → atlas_coord
- DualGrid 结构体：世界网格按地形层存储坐标集合（HashMap<TerrainType, HashSet<(i32,i32)>>，同一坐标可属于多个地形），显示格子计算、受影响位置计算、噪声地形生成
- TerrainThresholdEntry 结构体：地形阈值条目（terrain_id/min_value/max_value），每地形独立范围 [min, max)
- TerrainThresholds 结构体：地形阈值配置，每地形独立判断（非互斥）
- TerrainRegistry 结构体：地形注册表，维护 name ↔ id 双向映射
- PropConfig 结构体：资源配置（名称/source_id/alternative_tile/概率/允许地形/噪声范围）
- place_props 函数：根据噪声值和概率在地图上放置资源（支持同一坐标多个地形）

### [rust/src/map/gd_map_basic.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/map/gd_map_basic.rs)
- **GdMapBasic** 类（继承 Node2D）
- 双网格地图节点，通过导出属性 tile_set 配置 TileSet，场景文件中定义子 TileMapLayer 显示层（节点名对应地形名），PropLayer 资源层由 Rust 动态创建（ensure_prop_layer）
- 世界网格支持同一坐标属于多个地形（按地形层存储坐标集合）
- 渲染顺序由子节点顺序决定（自下而上），无 priority 概念
- 子层在编辑器中位于 (0,0) 可直接绘制占位符，运行时 generate_map_from_tiles 清除占位符、应用半格偏移并绘制过渡贴图
- 资源配置：通过 JSON 文件或字符串加载，仅配置 display source_id（无 atlas_coord/world source_id/priority）
- 方法：load_resource_config, load_resource_config_from_string, set_terrain, erase_tile, has_terrain, get_terrains_at, generate_map, generate_map_with_resources, generate_map_from_tiles, clear_map, set_thresholds, get_used_terrain_cells, refresh_display, add_terrain_config, add_prop_config, register_terrain, get_terrain_id, get_terrain_name, get_all_terrain_names
- 噪声生成：内置 Perlin-like 噪声算法，支持种子可复现，每地形独立范围判断
- 资源放置：根据噪声值和概率自动放置资源（Flower/Tree 等）

### [rust/src/map/dual_grid_iso.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/map/dual_grid_iso.rs)
- **等距双网格算法核心逻辑**（纯 Rust，不依赖 Godot API）
- IsoTerrainType 枚举：动态 ID，0 = Null，1+ = 用户注册
- IsoCornerState 枚举：Null/NotNull，用于等距四角组合
- 16种等距四角组合查找表：build_iso_tile_lookup()，四角顺序 [top, right, left, bottom]
- DualGridIso 结构体：等距世界网格存储、显示格子计算（等距邻域映射）、受影响位置计算、噪声地形生成
- 等距邻域映射：top=(x-1,y-1), right=(x,y-1), left=(x-1,y), bottom=(x,y)
- IsoTerrainThresholds 结构体：等距地形阈值配置
- IsoTerrainRegistry 结构体：等距地形注册表（name ↔ id 双向映射）
- IsoPropConfig 结构体：等距资源配置
- iso_place_props 函数：根据噪声值和概率在等距地图上放置资源

### [rust/src/map/gd_map_isometric.rs](file:///Users/dengronghui/project/gamekit/core/rust/src/map/gd_map_isometric.rs)
- **GdMapIsometric** 类（继承 TileMapLayer）
- 等距双网格地图节点，自身即世界层（self_modulate alpha=0 透明隐藏），内部持有显示层 TileMapLayer 子节点和1个资源层子节点
- 显示层偏移为 (0, -0.5)*tile_size（等距网格），启用 Y 排序
- 显示层支持 priority 优先级配置，低优先级图层在边界处额外渲染1格邻居避免露出背景
- 资源配置：通过 JSON 文件或字符串加载，支持地形 atlas_coord/source_id 和显示层 source_id/priority
- 方法：register_terrain, get_terrain_id, get_terrain_name, get_all_terrain_names, load_resource_config, load_resource_config_from_string, set_tile, set_terrain, erase_tile, get_terrain_type, generate_map, generate_map_with_resources, generate_map_from_tiles, clear_map, set_thresholds, get_used_terrain_cells, refresh_display, add_terrain_config, add_prop_config, set_tile_set
- 噪声生成：内置 Perlin-like 噪声算法，支持种子可复现
- 资源放置：根据噪声值和概率自动放置资源

## tilemap 模块（TileMapDual 双网格地形过渡系统）

### [rust/src/tilemap/mod.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/mod.rs)
- TileMapDual 模块入口，移植自 GDScript TileMapDual 插件 v5.0.2
- 提供双网格地形过渡系统：世界网格存储逻辑地形，显示网格根据四角组合查表得到过渡贴图

### [rust/src/tilemap/util.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/util.rs)
- Util 静态工具函数
- 方法：reverse_neighbor（反转 CellNeighbor）、cell_neighbors_to_var_array、cell_neighbors_2d_to_var_array

### [rust/src/tilemap/grid_shape.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/grid_shape.rs)
- GridShape 枚举（SQUARE/ISO/HALF_OFF_HORI/HALF_OFF_VERT/HEX_HORI/HEX_VERT）
- tileset_gridshape 函数：根据 TileSet 的 tile_shape 和 tile_offset_axis 返回 GridShape

### [rust/src/tilemap/tile_cache.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/tile_cache.rs)
- **TileCache** 类（继承 Resource）
- 缓存世界网格中每个格子的贴图位置和地形（cells: Dictionary[Vector2i, Dictionary{sid, tile, terrain}]）
- 方法：update、update_edited、xor（对称差集）、get_terrain_at

### [rust/src/tilemap/terrain_layer.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/terrain_layer.rs)
- **TerrainLayer** 类（继承 Resource）
- 单层地形规则，存储 terrain_neighbors → {sid, tile} 映射和 display_to_world_neighborhood 路径
- 方法：apply_rule、register_rule

### [rust/src/tilemap/atlas_watcher.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/atlas_watcher.rs)
- **AtlasWatcher** 类（继承 Resource）
- 图集监视器，监视 TileSetAtlasSource 的图集变化

### [rust/src/tilemap/tile_set_watcher.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/tile_set_watcher.rs)
- **TileSetWatcher** 类（继承 Resource）
- 监视 TileSet 变化（tile_size/tile_shape/terrain set 0）
- 信号：tileset_resized、atlas_autotiled
- 属性：tile_set、tile_size、grid_shape

### [rust/src/tilemap/terrain_dual.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/terrain_dual.rs)
- **TerrainDual** 类（继承 Resource）
- 地形系统核心，管理 TerrainLayer 集合
- 根据 TileSet 的 terrain set 0 自动生成地形过渡规则
- 信号：changed
- 属性：terrains、layers

### [rust/src/tilemap/terrain_preset.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/terrain_preset.rs)
- **TerrainPreset** 类（继承 Resource）
- 地形预设，neighborhood_preset 静态方法返回预设地形配置

### [rust/src/tilemap/display_layer.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/display_layer.rs)
- **DisplayLayer** 类（继承 TileMapLayer）
- 显示层，根据父 TileMapDual 的内容和 TerrainLayer 规则自动计算和更新显示贴图
- 方法：setup、reposition、update_properties、update_tiles_all、update_tiles、update_tile、follow_path

### [rust/src/tilemap/display.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/display.rs)
- **Display** 类（继承 Node2D）
- 显示管理节点，管理最多 2 个 DisplayLayer 子节点
- 信号：world_tiles_changed
- 方法：setup、update

### [rust/src/tilemap/tile_map_dual.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/tile_map_dual.rs)
- **TileMapDual** 类（继承 TileMapLayer，tool 模式）
- 双网格 TileMapLayer 主类，世界网格存储逻辑地形，通过 Display 子节点显示过渡贴图
- 使用 ghost material（内联 ShaderMaterial，alpha=0）使世界网格不可见
- 属性：refresh_time、godot_4_3_compatibility、display_material
- 方法：draw_cell、get_cell

### [rust/src/tilemap/cursor_dual.rs](file:///Users/dengronghui/project/gamekit/gamecore/rust/src/tilemap/cursor_dual.rs)
- **CursorDual** 类（继承 Sprite2D）
- 光标节点，跟随鼠标在 TileMapDual 上绘制贴图
- 属性：tilemap_dual
- 支持快捷键切换地形（0/1/2），左键绘制，右键擦除

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

### [example/ui/scene_title.gml](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_title.gml)
- 游戏标题界面 GML 布局
- 居中按钮组（Start Game/Continue/Quit）+ 右上角设置按钮
- 使用 PopupPanel 标签构建设置弹窗（HSlider/CheckButton/OptionButton）
- 使用 CSS 类样式定义按钮外观

### [example/ui/scene_title.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_title.gd)
- 游戏标题界面 GML 控制器 - 继承 GdGmlScene，处理 GML 中的事件回调
- 使用 NinePatchRect 替代 TextureButton 实现按钮背景九宫格拉伸

### [example/ui/scene_title_gml.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_title_gml.gd)
- 游戏标题界面 GML 控制器（继承 GdGmlScene）
- 处理 GML 中的事件回调：_on_start_game、_on_continue_game、_on_quit_game、_on_fullscreen_toggle
- PopupPanel 的显示/隐藏由 GML 内部信号绑定自动处理

### [example/ui/scene_main.gml](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_main.gml)
- 游戏主界面 GML 布局
- 底部居中装备栏（UIHList，6个槽位）
- Tooltip 提示框（鼠标跟随显示）
- 右侧 Drawer 抽屉面板（含 UIGrid 背包网格）
- 使用 CSS 类样式定义装备槽和网格项外观

### [example/ui/scene_main_gml.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_main_gml.gd)
- 游戏主界面 GML 控制器（继承 GdGmlScene）
- 在 _ready() 中初始化 GdBean（SceneMainBean）并调用 load_gml()
- 数据通过 GdBean 响应式绑定，GML 中 data="bean:scene_main:equip_data" 格式引用
- Tooltip 显示由 Rust 内置的 tooltip 属性自动处理

### [example/ui/scene_main_bean.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_main_bean.gd)
- 游戏主界面数据 Bean（继承 GdBean）
- 管理装备栏数据（equip_data）和背包数据（inventory_data）
- 属性变更时自动触发 watch 回调，更新绑定的 UI 节点

### [example/ui/scene_gallery.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_gallery.gd)
- 图鉴界面 GML 控制器（继承 GdGmlScene）
- 居中按钮点击打开 PopupPanel 弹窗
- 弹窗内含 TabContainer（Weapons/Armor/Items 三个 Tab 页）
- 每个 Tab 页包含顶部描述文字 + UIGrid 网格列表
- 数据通过脚本变量自动绑定（weapon_data/armor_data/item_data）

### [example/ui/scene_setting.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_setting.gd)
- 设置界面 GML 控制器（继承 GdGmlScene）
- 居中按钮点击后左侧弹出 NavMenu 多级级联菜单
- 一级菜单3项（Audio含三级/Display二级/Controls二级），NavItem递归嵌套

### [example/ui/scene_role.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/scene_role.gd)
- 角色界面 GML 控制器（继承 GdGmlScene）
- 居中按钮点击弹出 PopupPanel 角色属性面板
- 面板左：三列装备区（左侧装备列3槽+中间角色立绘面板+右侧装备列3槽），装备槽带 label 标注类型
- 面板右：UIGrid 5x5 背包网格，支持分页（Prev/Next 按钮+页码显示）

### [example/ui/tudouxiongdi/levelup.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/tudouxiongdi/levelup.gd)
- 土豆兄弟升级弹窗界面 GML 控制器（继承 GdGmlScene），严格匹配 levelup.html 设计稿
- 2D像素风生存割草游戏升级选择界面，纯深灰背景(#333333)+扁平化极简黑色卡片UI
- 左上角状态栏：红色HP进度条(#c91919,40px高,文字在条内右侧)、绿色经验进度条(#24c442,32px高)、绿色圆形金币图标(#48d05e)+数字388
- 顶部居中：关卡标题"第16关"(42px)+"升级!!!"(60px)
- 中部3个横向并排黑色圆角天赋卡片(240px宽,padding 20px)，卡片图标用Panel色块(白色#eee/绿色#48d05e,60x60)，描述文字绿色(#48d05e)，选择按钮白底黑字
- 卡片下方刷新按钮：黑底白字+白色播放图标Panel(26x26)
- 右侧竖版黑色属性面板(240px宽)：剩余升级点数+目前等级(右对齐22px)+11项属性列表(灰色图标#666 22x22+属性名+数值)
- 右下角浅灰色"九游"水印
- 使用 `<theme>` 块覆盖 cartoon 主题为暗色调配色，ProgressBar 使用 class 样式设置 fill/track 颜色

### [example/ui/tudouxiongdi/pause.gd](file:///Users/dengronghui/project/gamekit/core/example/ui/tudouxiongdi/pause.gd)
- 土豆兄弟暂停/结算面板 GML 控制器（继承 GdGmlScene），严格匹配 pause.html 设计稿
- 1657x960px 暗色主题面板(#222222)，左侧属性栏(260px,#111111)+右侧武器/物品区+底部按钮
- 顶部标题栏：居中"胜利"(36px,绿色#4cd964)+"危险5"(24px,红色#ff3b30)
- 左侧属性面板(PanelContainer class="attr-box")：16项属性行(HBoxContainer)，标签+数值，绿色(#4cd964)/红色(#ff3b30)数值
- 右侧武器区：6个80x80 Panel色块，红色(#ff3b30)/紫色(#af52cc)边框，emoji图标
- 右侧物品区：3行HBoxContainer，每行10个72x72 Panel色块，黑/紫/蓝/灰色边框，部分带堆叠标签(X2/X3)
- 底部3个按钮(320x60)：深色(#333333)/浅色(#555555)背景，白色文字
- 堆叠标签定位：anchor_left/right/top/bottom + offset 实现右下角定位

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
| 2026-06-10 | addons/gamecore/ui/popup_panel.gd | 新建通用弹窗面板组件（继承CanvasLayer），模态遮罩+标题栏+GML内容构建+显示/隐藏 |
| 2026-06-10 | example/ui/scene_title.gml | 新建游戏标题界面GML布局，居中按钮组+右上角设置按钮 |
| 2026-06-10 | example/ui/scene_title.gd | 新建游戏标题界面控制器，GdUiBuilder+PopupPanel组合，设置弹窗（音量/全屏/语言） |
| 2026-06-10 | rust/src/ui/builder.rs | 新增GML标签：CheckButton/HSlider/ColorRect/OptionButton/PopupPanel；新增属性：size_flags_horizontal/vertical/color/toggle_mode/button_pressed/items/selected/popup_title/popup_width/close_on_overlay |
| 2026-06-10 | rust/src/ui/ui_popup_panel.rs | 新建GdPopupPanel弹窗面板节点（继承Control），模态遮罩+标题栏+关闭按钮+内容区域，GML标签<PopupPanel>，替代旧版GDScript popup_panel.gd |
| 2026-06-10 | example/ui/scene_title.gml | 更新：使用PopupPanel标签替代旧版GDScript弹窗，使用HSlider/CheckButton/OptionButton替代SpinBox |
| 2026-06-10 | example/ui/scene_title.gd | 简化：不再依赖popup_panel.gd，直接使用GML中的PopupPanel节点 |
| 2026-06-10 | addons/gamecore/ui/popup_panel.gd | 删除：已被Rust实现的GdPopupPanel替代 |
| 2026-06-10 | rust/src/ui/ui_gml_scene.rs | 新建GdGmlScene节点（继承Control），设置gml_file属性即可加载.gml文件并显示为Control节点树，支持自动信号连接 |
| 2026-06-11 | rust/src/ui/ui_gml_scene.rs | 优化：gml_file属性改为文件引用类型；auto_connect改为连接信号到自身脚本 |
| 2026-06-11 | example/ui/scene_title_gml.gd | 新建继承GdGmlScene的GDScript，事件回调从scene_title.gd移入 |
| 2026-06-11 | example/ui/scene_title.gd | 简化：移除已迁移到scene_title_gml.gd的回调函数 |
| 2026-06-11 | example/ui/scene_title.tscn | 更新：GmlScene节点挂载scene_title_gml.gd脚本 |
| 2026-06-11 | addons/gamecore/gml_import_plugin.gd | 删除：改用 EditorSettings textfile_extensions 方式 |
| 2026-06-11 | addons/gamecore/core.gd | 更新：改用 _register_gml_extension() 注册 .gml 扩展名 |
| 2026-06-11 | rust/src/ui/ui_tooltip.rs | 新建GdUITooltip鼠标跟随提示框节点，GML标签<Tooltip> |
| 2026-06-11 | rust/src/ui/ui_drawer.rs | 新建GdUIDrawer抽屉面板节点，GML标签<Drawer> |
| 2026-06-11 | rust/src/ui/mod.rs | 添加ui_tooltip和ui_drawer模块 |
| 2026-06-11 | rust/src/ui/builder.rs | 注册Tooltip/Drawer标签；扩展内部信号绑定支持open:/close:动作；新增Tooltip和Drawer属性处理 |
| 2026-06-11 | example/ui/scene_main.gml | 新建游戏主界面GML布局（装备栏+Tooltip+Drawer） |
| 2026-06-11 | rust/src/ui/ui_hlist.rs | 更新：添加 s_mouse_enter_item / s_mouse_exit_item 信号 |
| 2026-06-11 | example/ui/scene_main_gml.gd | 新建游戏主界面GML控制器脚本 |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 `{{key}}` 模板语法检测和元数据记录 |
| 2026-06-11 | rust/src/ui/ui_list_helper.rs | 新增模板绑定解析（resolve_template_bindings_recursive），分离简单 key 和路径 key |
| 2026-06-11 | rust/src/ui/ui_hlist.rs | 新增 tooltip 属性和自动 Tooltip 绑定 |
| 2026-06-11 | rust/src/ui/ui_grid.rs | 新增 tooltip 属性和自动 Tooltip 绑定 |
| 2026-06-11 | rust/src/ui/ui_drawer.rs | 修复初始显示灰色全屏遮挡 |
| 2026-06-11 | example/ui/scene_main_gml.gd | 简化数据定义，移除手动 Tooltip 信号绑定 |
| 2026-06-11 | rust/src/ui/ui_list_helper.rs | 修复 get_meta("__tpl_keys") 报错：添加 has_meta() 检查 |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 tooltip/data 属性处理；列表容器子节点跳过 set_owner |
| 2026-06-11 | rust/src/ui/ui_gml_scene.rs | 新增数据自动绑定（auto_bind_data） |
| 2026-06-11 | example/ui/scene_main.gml | UIHList/UIGrid 添加 data 属性 |
| 2026-06-11 | example/ui/scene_main_gml.gd | 数据内联定义，移除 update 函数 |
| 2026-06-11 | rust/src/state/bean.rs | get_bean_by_id 改为 pub(crate) fn |
| 2026-06-11 | rust/src/ui/ui_gml_scene.rs | 新增 GdBean 响应式数据绑定和 on_bean_data_changed 方法 |
| 2026-06-11 | example/ui/scene_main_bean.gd | 新建游戏主界面数据 Bean |
| 2026-06-11 | example/ui/scene_main_gml.gd | 重构：移除内联数据，在 _ready() 中初始化 GdBean |
| 2026-06-11 | example/ui/scene_main.gml | data 属性改为 bean: 前缀格式 |
| 2026-06-11 | rust/src/ui/ui_tooltip.rs | 新增 max_height 属性、update_data 方法、自定义子节点支持（添加时移除内置label）、resolve_template_bindings 辅助方法 |
| 2026-06-11 | rust/src/ui/ui_list_helper.rs | update_container 中存储完整数据字典为 __item_data meta |
| 2026-06-11 | rust/src/ui/ui_hlist.rs | show_tooltip_for_item 优先读取 __item_data 调用 update_data，降级兼容旧格式 |
| 2026-06-11 | rust/src/ui/ui_grid.rs | 同 UIHList，show_tooltip_for_item 优先读取 __item_data |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 max_height 属性解析 |
| 2026-06-11 | rust/src/state/bean.rs | 修复 watch 回调参数：统一为 2 参数调用 (value, metas)，删除 1 参数调用，修复 Callable.bind() 追加参数导致类型转换错误 |
| 2026-06-11 | rust/src/ui/ui_gml_scene.rs | 修复 GdBean 响应式回调：新增 on_bean_data_changed_bound() 适配 callable.bind() 追加参数顺序，注册回调改用新函数 |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 Tab 标签支持（映射为 VBoxContainer）；新增 title 属性处理（Tab 标签的 title 覆盖节点名）；新增 current_tab/tabs_visible 属性处理（TabContainer） |
| 2026-06-11 | example/ui/scene_gallery.gd | 新建图鉴界面 GML 控制器（继承 GdGmlScene），居中按钮 + PopupPanel + TabContainer（Weapons/Armor/Items 三个 Tab 页，每个含描述文字 + UIGrid） |
| 2026-06-11 | rust/src/ui/builder.rs | 新增 TextureButton 标签支持：实例化、text 叠加 Label、texture/texture_normal/texture_pressed/texture_hover/texture_disabled 属性、样式 texture 属性、文字颜色子 Label 应用 |
| 2026-06-11 | example/ui/scene_title.gd | 将菜单按钮从 Button 改为 TextureButton，menu-button 样式使用 texture 属性加载 btn_green.png |
| 2026-06-11 | rust/src/ui/ui_nav_menu.rs | 新建GdUINavMenu导航菜单节点，GML标签<NavMenu>/<NavItem>（递归嵌套） |
| 2026-06-11 | rust/src/ui/mod.rs | 添加ui_nav_menu模块 |
| 2026-06-11 | rust/src/ui/builder.rs | 注册NavMenu/NavItem标签；新增NavMenu属性处理；移除NavSubItem标签 |
| 2026-06-11 | example/ui/scene_setting.gd | 新建设置界面GML控制器，居中按钮+NavMenu多级级联菜单 |
| 2026-06-12 | example/ui/scene_role.gd | 新建角色界面GML控制器，居中按钮+PopupPanel角色属性面板（面板左三列装备区+立绘，面板右UIGrid 5x5背包网格+分页） |
| 2026-06-12 | rust/src/ui/ui_gml_scene.rs | 修复find_node无法查找PopupPanel/Drawer/Tooltip内部子节点bug：改用find_child_ex设置owned=false |
| 2026-06-12 | rust/src/ui/builder.rs | 修复信号绑定中find_child无法查找PopupPanel内部节点bug：改用find_child_ex设置owned=false |
| 2026-06-12 | rust/src/ui/ui_hlist.rs | 修复Tooltip查找中find_child的owned限制：改用find_child_ex设置owned=false |
| 2026-06-12 | rust/src/ui/ui_grid.rs | 同ui_hlist.rs，修复Tooltip查找中find_child的owned限制 |
| 2026-06-13 | rust/src/ui/builder.rs | 新增百分比自适应语法：parse_percent/parse_size_value辅助函数；apply_size/apply_custom_minimum_size/apply_margin支持百分比（如"80%,50%"、"5%"），百分比信息存为meta延迟计算；popup_width/slide_width/menu_width/sub_menu_width属性支持百分比 |
| 2026-06-13 | rust/src/ui/ui_gml_scene.rs | 新增百分比布局刷新：refresh_percent_layouts/refresh_percent_layouts_recursive方法，处理__pct_size/__pct_min_size/__pct_margin/__pct_popup_width/__pct_slide_width/__pct_menu_width/__pct_sub_menu_width元数据；on_notification监听RESIZED事件自动刷新百分比布局；refresh_anchors同时刷新百分比布局 |
| 2026-06-13 | example/ui/sample_ui.gml | 更新：margin改为"2%"，Panel size改为"60%,30%"，Button/Panel custom_minimum_size改为百分比 |
| 2026-06-13 | example/ui/scene_title.gd | 更新：TextureButton custom_minimum_size改为"30%,6%"，PopupPanel popup_width改为"50%"，HSlider custom_minimum_size改为"15%,0"，Button custom_minimum_size改为"12%,5%" |
| 2026-06-13 | example/ui/scene_gallery.gd | 更新：Button custom_minimum_size改为"30%,6%"，PopupPanel popup_width改为"65%"，TabContainer custom_minimum_size改为"90%,80%"，MarginContainer custom_minimum_size改为"12%,12%" |
| 2026-06-13 | example/ui/scene_role.gd | 更新：PopupPanel popup_width改为"80%"，equip-slot custom_minimum_size改为"10%,10%"，portrait-panel custom_minimum_size改为"25%,0"，grid-item custom_minimum_size改为"7%,7%" |
| 2026-06-13 | example/ui/scene_setting.gd | 更新：NavMenu menu_width改为"15%"，sub_menu_width改为"20%" |
| 2026-06-13 | rust/src/ui/ui_theme.rs | 新建UI主题系统：内置配色方案（dark/light/forest/ocean），ThemeVars变量表，resolve_theme_vars变量替换，parse_theme_block解析<theme>块 |
| 2026-06-13 | rust/Cargo.toml | 添加regex-lite依赖 |
| 2026-06-13 | rust/src/ui/parser.rs | 新增<theme>块解析支持；ParseResult新增theme_vars和theme_name字段 |
| 2026-06-13 | rust/src/ui/builder.rs | UiBuilder新增theme_vars字段和set_theme_vars方法；样式属性值替换$var主题变量 |
| 2026-06-13 | rust/src/ui/gdui_builder.rs | GdUiBuilder新增set_theme/get_theme/get_builtin_themes/set_theme_var/clear_custom_theme_vars方法 |
| 2026-06-13 | rust/src/ui/ui_gml_scene.rs | GdGmlScene新增theme_name属性和apply_theme/get_builtin_themes方法 |
| 2026-06-13 | rust/src/ui/mod.rs | 添加ui_theme模块 |
| 2026-06-13 | example/ui/*.gd/*.gml | 所有GML示例更新：添加theme="dark"属性，样式颜色值替换为$var主题变量引用 |
| 2026-06-13 | example/ui/scene_main.gd | 更新：Button custom_minimum_size改为"30%,6%"，equip-slot custom_minimum_size改为"8%,8%"，Drawer slide_width改为"35%"，grid-item custom_minimum_size改为"12%,12%" |
| 2026-06-14 | rust/src/anim/juice.rs | 完全重写：移除所有 callable_from_fn（返回 Callable::invalid()）和 tween_method+闭包模式（gdext 0.5 不支持），改用 tween_property + Godot 内置 TransitionType/EaseType；移除大部分公开函数的 Easing 参数（简化签名）；新增 easing_to_trans/easing_to_ease_type 辅助映射函数；清理回调改用 Callable::from_fn（Gd<Node> 非 Send）；属性参数从 &StringName 改为 &NodePath，final_val 从 Variant 改为 &Variant |
| 2026-06-14 | rust/src/anim/transition.rs | 修复 gdext 0.5 兼容：移除 tween.bind()，StringName→NodePath，添加缓动映射 |
| 2026-06-14 | rust/src/anim/easy_move.rs | 新建 EaseMover 平滑移动工具，移植自 C++ juice/easy_move |
| 2026-06-14 | rust/src/ui/ui_popup_panel.rs | 添加弹入/弹出动画（缩放+淡入淡出+遮罩动画） |
| 2026-06-14 | rust/src/ui/ui_tooltip.rs | 添加淡入淡出动画（缩放+透明度） |
| 2026-06-14 | rust/src/ui/ui_hlist.rs | 添加点击缩放反馈动画 |
| 2026-06-14 | rust/src/ui/ui_vlist.rs | 添加点击缩放反馈动画 |
| 2026-06-14 | rust/src/ui/ui_grid.rs | 添加点击缩放反馈动画 |
| 2026-06-14 | rust/src/ui/ui_drawer.rs | 使用 anim 模块缓动函数替代手写 ease_out_cubic |
| 2026-06-14 | rust/src/ui/ui_nav_menu.rs | 使用 anim 模块缓动函数替代手写 ease_out_cubic |
| 2026-06-14 | rust/src/ui/builder.rs | 移除 TextureButton 的 text 属性处理，改为子 Label 自动配置（锚点填满、文字居中、鼠标穿透） |
| 2026-06-14 | example/ui/scene_title.gd | TextureButton 的 text 属性拆成子 Label 元素 |
| 2026-06-14 | rust/src/ui/builder.rs | NinePatchRect 按钮模式：有 on_pressed 时添加不可见 Button 子节点处理点击；子 Label 自动配置；class style texture 支持；patch_margin 属性；主题默认颜色和文字颜色支持 |
| 2026-06-14 | example/ui/scene_title.gd | TextureButton 改为 NinePatchRect 实现九宫格拉伸按钮 |
| 2026-06-15 | rust/src/manager/mod.rs | 新建场景管理模块入口 |
| 2026-06-15 | rust/src/manager/gd_scene.rs | 新建GdScene场景页面节点（继承Control），生命周期回调+状态管理+自动创建GdSceneRoot |
| 2026-06-15 | rust/src/manager/gd_scene_root.rs | 新建GdSceneRoot场景管理器（继承Node），场景注册/切换/转场动画/历史栈 |
| 2026-06-15 | rust/src/lib.rs | 添加manager模块 |
| 2026-06-15 | rust/src/manager/gd_scene.rs | id→scene_id，属性改为#[export]，添加change_scene/back_scene/restart_scene便捷方法 |
| 2026-06-15 | rust/src/manager/gd_scene.rs | 修复：ensure_manager使用call_deferred添加GdSceneRoot，get_tree→get_tree_or_null |
| 2026-06-15 | rust/src/manager/gd_scene_root.rs | 修复：get_tree→get_tree_or_null避免不在场景树时panic |
| 2026-06-15 | rust/src/state/gdcore.rs | 添加global_nodes字典和add_global_node/get_global_node/remove_global_node方法 |
| 2026-06-15 | rust/src/manager/config_manager.rs | 新建GdConfigManager通用配置管理器，读取res://game_config.json，内置scenes属性，通过GDCORE注册 |
| 2026-06-15 | rust/src/manager/gd_scene_root.rs | 添加load_scenes_from_config从GdConfigManager加载场景配置 |
| 2026-06-15 | example/ui/tudouxiongdi/levelup.gd | 重写升级弹窗界面严格匹配HTML设计稿；builder.rs新增ProgressBar fill颜色支持(background→fill,track→background)和theme默认值 |
| 2026-06-15 | example/ui/tudouxiongdi/pause.gd | 新建土豆兄弟暂停/结算面板GML控制器，严格匹配pause.html设计稿 |
| 2026-06-23 | rust/src/map/dual_grid_iso.rs | 修复等距双网格偏移量Bug：calculate_display_tile中top=(0,-1)→(-1,-1), right=(1,-1)→(0,-1), left=(-1,-1)→(-1,0)；ISO_FOUR_CELLS从[(0,0),(-1,1),(0,1),(1,1)]修正为[(0,0),(1,1),(0,1),(1,0)]；更新相关注释 |
| 2026-06-24 | rust/src/tilemap/ | 新建tilemap模块，移植自GDScript TileMapDual插件v5.0.2，包含util/grid_shape/tile_cache/terrain_layer/atlas_watcher/tile_set_watcher/terrain_dual/terrain_preset/display_layer/display/tile_map_dual/cursor_dual共12个文件 |
| 2026-06-24 | rust/src/tilemap/mod.rs | 新建模块入口，导出所有子模块 |
| 2026-06-24 | rust/src/tilemap/util.rs | 新建Util静态工具函数（reverse_neighbor/cell_neighbors_to_var_array/cell_neighbors_2d_to_var_array） |
| 2026-06-24 | rust/src/tilemap/grid_shape.rs | 新建GridShape枚举与tileset_gridshape函数 |
| 2026-06-24 | rust/src/tilemap/tile_cache.rs | 新建TileCache类（继承Resource），缓存格子贴图位置和地形，支持XOR对称差集 |
| 2026-06-24 | rust/src/tilemap/terrain_layer.rs | 新建TerrainLayer类（继承Resource），单层地形规则，terrain_neighbors→{sid,tile}映射 |
| 2026-06-24 | rust/src/tilemap/atlas_watcher.rs | 新建AtlasWatcher类（继承Resource），图集监视器 |
| 2026-06-24 | rust/src/tilemap/tile_set_watcher.rs | 新建TileSetWatcher类（继承Resource），监视TileSet变化，发射tileset_resized/atlas_autotiled信号 |
| 2026-06-24 | rust/src/tilemap/terrain_dual.rs | 新建TerrainDual类（继承Resource），地形系统核心，管理TerrainLayer集合，自动生成地形过渡规则 |
| 2026-06-24 | rust/src/tilemap/terrain_preset.rs | 新建TerrainPreset类（继承Resource），地形预设，neighborhood_preset静态方法 |
| 2026-06-24 | rust/src/tilemap/display_layer.rs | 新建DisplayLayer类（继承TileMapLayer），显示层，自动计算和更新显示贴图 |
| 2026-06-24 | rust/src/tilemap/display.rs | 新建Display类（继承Node2D），显示管理节点，管理DisplayLayer子节点 |
| 2026-06-24 | rust/src/tilemap/tile_map_dual.rs | 新建TileMapDual类（继承TileMapLayer,tool），双网格主类，ghost material内联创建，draw_cell/get_cell方法 |
| 2026-06-24 | rust/src/tilemap/cursor_dual.rs | 新建CursorDual类（继承Sprite2D），光标节点，跟随鼠标绘制贴图 |
| 2026-06-24 | PROJECT.md | 添加tilemap模块说明（第12节）和项目结构中的tilemap目录 |
| 2026-06-24 | FILES.md | 添加tilemap模块文件索引和变更记录 |
| 2026-06-24 | API.md | 新建API使用手册，包含TileMapDual模块API文档 |
| 2026-06-24 | rust/src/map/gd_map_basic.rs | 重构 GdMapBasic：继承 Node2D，新增 tile_set 导出属性，移除世界层/priority/set_tile，新增 generate_map_from_tiles，add_terrain_config 签名简化 |
| 2026-06-24 | addons/gamecore/map/basic/terrain_config.gd | 移除 atlas_coord/source_id/priority 字段 |
| 2026-06-24 | addons/gamecore/map/basic/index.gd | 适配新 add_terrain_config 签名，tile_set 从 GdMapBasic 属性获取 |
| 2026-06-24 | addons/gamecore/map/basic/index.tscn | 新增 5 个子 TileMapLayer 节点（dirt/grass/sand/water/PropLayer） |
| 2026-06-24 | example/map/basic.tscn | 移除 tile_map_data（Node2D 不支持） |
| 2026-06-24 | FILES.md | 更新 GdMapBasic 文件描述（Node2D 架构、新方法列表） |
| 2026-06-24 | rust/src/map/gd_map_basic.rs | PropLayer 改为动态创建：新增 ensure_prop_layer()，scan_child_layers 不再扫描 PropLayer |
| 2026-06-24 | addons/gamecore/map/basic/index.tscn | 移除 PropLayer 子节点（改为 Rust 动态创建） |
| 2026-06-24 | FILES.md | 更新 GdMapBasic 文件描述（PropLayer 动态创建） |
| 2026-06-24 | rust/src/map/dual_grid.rs | 重构世界网格为多层结构（HashMap<TerrainType, HashSet>），支持同一坐标多地形 |
| 2026-06-24 | rust/src/map/gd_map_basic.rs | 适配多层世界网格：erase_tile 增加 terrain_type，新增 has_terrain/get_terrains_at，set_thresholds 增加 min 参数 |
| 2026-06-24 | addons/gamecore/map/basic/terrain_config.gd | 新增 threshold_min 属性 |
| 2026-06-24 | addons/gamecore/map/basic/index.gd | _setup_thresholds 适配新签名 |
| 2026-06-24 | addons/gamecore/map/basic/index.tscn | TerrainConfig 添加 threshold_min 值 |
| 2026-06-24 | FILES.md | 更新 dual_grid.rs 和 gd_map_basic.rs 文件描述 |
