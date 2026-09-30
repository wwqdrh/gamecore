# GameKitCore 游戏框架结构说明

本文档描述 `rust/src` 中游戏框架的功能模块划分。框架以 godot-rust 0.5（Godot 4.6）实现，
通过 GDExtension 暴露给 GDScript 使用；入口与单例注册见 `lib.rs`。

```
rust/src/
├── lib.rs              # 扩展入口：模块声明 + 5 个引擎单例注册（InitStage::Scene）
├── runtime/            # 协程运行时
├── state/              # 数据与状态
├── manager/            # 场景与应用管理器
├── role/               # 角色系统
├── dialog/             # 对话系统（含触发器、角色绑定，已整合）
├── components/         # 实体战斗组件库
├── map/                # 地图系统
├── ui/                 # UI 标记语言与控件库
├── anim/               # 动画与缓动
├── drawer/             # 2D 绘图辅助
├── environment/        # 天气环境系统
├── event/              # 全局事件总线
├── pool/               # 对象池
├── debug/              # 运行时调试绘制
├── console/            # Lua 后台控制台
├── rogue/              # 肉鸽核心算法
└── dev/                # Rust 侧单元测试框架
```

---

## 1. 运行时基础设施（runtime）

基于标准 `Future` 的协程系统，替代 GDScript 的 `await` 链，供 Rust 侧逻辑编排异步流程。

| 文件 | 职责 |
|---|---|
| `runtime/coroutine.rs` | `SpireCoroutine` 协程管理器核心（运行/暂停/完成信号） |
| `runtime/yielding.rs` | 等待原语：`seconds` / `frames` / `wait_while` / `wait_until` / `wait_for_signal` |
| `runtime/builder.rs` | `CoroutineBuilder` 链式协程构建器 |
| `runtime/start_coroutine.rs` | `StartCoroutine` 挂载辅助类 |
| `runtime/start_async_task.rs` | `StartAsyncTask` 异步任务挂载辅助类 |

公开 API 经 `lib.rs` 的 `pub mod prelude` 导出。

## 2. 数据与状态（state）

提供带加密、持久化与变更订阅能力的 JSON 数据存储，以及数据绑定层。

| 文件 | 职责 |
|---|---|
| `state/gjson.rs` | `GJson` 路径查询式 JSON 文档存储（加密/持久化/订阅，等价 gamedb::GJson） |
| `state/coredata.rs` | `GdCoreData` 核心数据引擎（Resource，底层 GJson） |
| `state/bean.rs` | `GdBean` 数据绑定 Bean（持有 GdCoreData 作后端） |
| `state/linklist.rs` | `GdDataLinkList` 基于字典的链表数据容器 |
| `state/gdcore.rs` | `GDCore` 全局核心单例 **"GDCORE"**：全局节点注册表、存档 ID 管理与数据缓存 |

## 3. 场景与应用管理器（manager）

| 文件 | 职责 |
|---|---|
| `manager/gd_scene.rs` | `GdScene` 场景页面节点（Control，状态管理与生命周期回调） |
| `manager/gd_scene_root.rs` | `GdSceneRoot` 场景管理器：页面切换、转场动画、全局暂停控制 |
| `manager/config_manager.rs` | `GdConfigManager` 通用配置管理器（读取 `res://game_config.json`） |
| `manager/input.rs` | `GdInput` 输入管理器：虚拟动作注册、组合按键、鼠标点击/滑动、轴映射 |
| `manager/audio.rs` | `GdViewAudio` 音频管理器：BGM 与 SFX（含播放器池、`play_sfx(path, volume)`） |
| `manager/camera.rs` | `GdViewCamera` 相机管理器：跟随、边界约束、鼠标/滚轮缩放 |
| `manager/setting.rs` | `GdViewSetting` 游戏设置管理器：音频/窗口/自定义设置的读取、应用与持久化（依赖 `state/coredata`） |

## 4. 角色系统（role）

开箱即用的角色三件套，配合节点树使用：
`Player (GdRoleMover) └── Animator (GdRoleAnimator) └── Brain (GdNpcBrain)`。

| 文件 | 职责 |
|---|---|
| `role/movement.rs` | `GdRoleMover`（CharacterBody2D）：四向/横向/网格移动，键盘/鼠标/AI 控制模式，暂停接口 |
| `role/animator.rs` | `GdRoleAnimator` 依据移动状态自动驱动精灵动画 |
| `role/npc_ai.rs` | `GdNpcBrain` NPC 行为决策：待命/游走/巡逻/跟随/逃跑/追击（HUNTER 含视野、攻击相位与冷却） |

## 5. 对话系统（dialog）

对话全链路三件套（原 `speaker.rs` / `dialog_trigger.rs` 位于 `role/`，因与对话引擎紧耦合，已整合至本目录）：

| 文件 | 职责 |
|---|---|
| `dialog/gddialogue.rs` | `GdDialogue` 对话控制节点：管理 Timeline 与对话推进（等价 C++ dialogue.cpp 接口） |
| `dialog/speaker.rs` | `GdRoleSpeaker` 人物与对话角色绑定（挂在人物节点下，按 role_name 注册进 GdDialogue） |
| `dialog/dialog_trigger.rs` | `GdDialogTrigger` 对话触发器：接近/交互/自动/手动四种模式 + 条件门控 + 对话期间暂停移动互相面向 |

## 6. 实体战斗组件库（components）

原型可复用的挂载式组件，任意 Node/Node2D 宿主按需组合。

| 文件 | 职责 |
|---|---|
| `components/health.rs` | `GdHealth` 血量组件：伤害/治疗/无敌帧/复活 + `s_damaged` / `s_healed` / `s_died` 信号 |
| `components/combat.rs` | `GdHitbox` 攻击判定 + `GdHurtbox` 受击判定（物理帧主动扫描、自动解析宿主 Health） |
| `components/shooter.rs` | `GdShooter` 射击组件：`fire` / `fire_at_point` / `fire_toward_mouse` + 按住鼠标连发；子弹场景懒注册进对象池 |
| `components/bullet.rs` | `GdBullet` 池化子弹（Area2D）：直线飞行、命中受击盒即毁、寿命到期归池 |
| `components/attribute.rs` | `GdAttributeSet` 通用属性集（HashMap + 变更信号） |
| `components/input_buffer.rs` | `GdInputBuffer` 输入缓冲（按键预输入窗口期内消费） |

碰撞层约定：layer2 = 玩家受击盒（值 2）、layer3 = 敌方受击盒（值 4）、子弹 mask = 4、敌方 Hitbox mask = 2。

## 7. 地图系统（map）

| 文件 | 职责 |
|---|---|
| `map/dual_grid.rs` | 双网格算法核心：世界网格按地形层存储坐标集合，支持同一坐标多地形与地形自动过渡 |
| `map/gd_map_basic.rs` | `GdMapBasic` 双网格地图节点（Node2D）：地形过渡绘制、噪声地图生成、资源配置 |
| `map/quick_map.rs` | `GdQuickMap` 快速地图生成器（Node2D，tool） |

## 8. UI 框架（ui）

类 HTML 声明式 UI 标记语言（GML）+ 开箱控件库。

| 文件 | 职责 |
|---|---|
| `ui/parser.rs` | GML 解析器：类 HTML 文本 → AST 节点树 |
| `ui/builder.rs` | UI 构建器：AST → Godot Control 节点树 |
| `ui/gdui_builder.rs` | `GdUiBuilder` 暴露给 GDScript 的解析/构建 API |
| `ui/ui_theme.rs` | UI 主题系统：内置卡通风格配色与变量替换 |
| `ui/ui_gml_scene.rs` | `GdGmlScene` GML 文件加载节点（设 file 属性即显示） |
| `ui/ui_drawer.rs` | `GdUIDrawer` 抽屉面板（屏幕边缘滑入/滑出） |
| `ui/ui_nav_menu.rs` | `GdUINavMenu` 多级级联导航菜单 |
| `ui/ui_popup_panel.rs` | `GdPopupPanel` 通用弹窗面板（模态遮罩 + 标题栏 + 内容区） |
| `ui/ui_tooltip.rs` | `GdUITooltip` 鼠标跟随提示框 |
| `ui/ui_hlist.rs` / `ui_vlist.rs` / `ui_grid.rs` | 横向 / 纵向 / 网格列表节点 |
| `ui/ui_list_helper.rs` | 列表辅助工具（等价 C++ gmlc/ui_list_helper） |

## 9. 动画（anim）

移植自 C++ juice 库的动画效果集（`ui`、`manager/camera`、`manager/audio` 内部均有依赖）。

| 文件 | 职责 |
|---|---|
| `anim/easing.rs` | `Easing` 缓动函数集（供 UI 相机音频等复用） |
| `anim/juice.rs` | 核心动画效果（缩放弹跳、震动等 juice 手感） |
| `anim/easy_move.rs` | `EaseMover` 数值平滑移动器（相机跟随、音量渐变） |
| `anim/transition.rs` | 场景/界面过渡效果 |

## 10. 表现辅助：绘图与环境（drawer / environment）

| 文件 | 职责 |
|---|---|
| `drawer/geometry.rs` | 纯几何轮廓生成（树叶/圆/点集旋转平移），无 Godot 类依赖 |
| `drawer/colors.rs` | 调色板与颜色工具（hex 解析、插值、秋季叶色）+ `GdColorUtil` 静态接口 |
| `drawer/canvas_drawer.rs` | `GdCanvasDrawer` 队列式画布绘制节点（矩形/圆/线/多边形/树叶） |
| `environment/shaders.rs` | 内置天气 shader 源码注册表（字符串动态创建，编译进 Rust 二进制） |
| `environment/weather.rs` | `GdWeather` 天气管理节点：白天/夜晚/雨/雪/风（全屏后期覆盖层 + 刮风树叶，依赖 drawer） |

## 11. 全局服务单例（event / pool / debug / console / state::gdcore）

5 个进程级单例，统一在 `lib.rs` 的 `InitStage::Scene` 注册。
**基类均为 `Object`（手动内存）**，严禁改回 `RefCounted + mem::forget`（会被引擎回收引发 UB）。

| 文件 | 单例名 | 职责 |
|---|---|---|
| `event/event_bus.rs` | **GDEVENTBUS** | `GdEventBus` 全局发布/订阅总线，跨场景解耦广播 |
| `pool/spawn_pool.rs` | **GDSPAWNPOOL** | `GdSpawnPool` 对象池：预缓存 PackedScene，spawn/despawn 复用节点 |
| `debug/debug_draw.rs` | **GDDEBUGDRAW** | `GdDebugDraw` 运行时 2D 调试绘制（线/射线/框/圆，自动过期） |
| `console/gdconsole.rs` | **GdConsole** | 基于 mlua 的后台控制台（GDScript 注册命令、执行 Lua） |
| `state/gdcore.rs` | **GDCORE** | 全局核心（见 §2 数据与状态） |

## 12. 肉鸽核心算法（rogue）

将 `gamealgo` 肉鸽算法库暴露给 Godot。

| 文件 | 职责 |
|---|---|
| `rogue/engine.rs` | `RogueEngine` 肉鸽引擎单例（JSON 配置初始化） |
| `rogue/card.rs` | `RogueCard` 卡牌包装类 |
| `rogue/card_pile.rs` | 卡牌堆（抽牌/洗牌/弃牌） |

## 13. 开发调试（dev）

Rust 侧轻量测试基础设施，可被 `cargo test` 与 Godot 内运行器复用。

| 文件 | 职责 |
|---|---|
| `dev/framework.rs` | 测试注册表 + 断言上下文 + 结果收集 |
| `dev/gd_test_runner.rs` | `DevTestRunner` 暴露给 GDScript 的引擎内测试触发器 |
| `dev/suites/*.rs` | 纯逻辑测试套件（`tests_easing` / `tests_gjson` / `tests_map`） |

> GDScript 侧测试位于项目 `test/` 目录（`test_runner.gd` 统一调度），与 dev 模块互补。

---

## 模块依赖关系速览

- `ui` → `anim`（缓动）、`state/bean`（GML 场景数据）
- `manager`（camera/audio）→ `anim/easy_move`；`manager/setting` → `state/coredata`；`manager/camera` → `manager/input`
- `environment` → `drawer`（叶形与调色板）
- `dialog` → `role`（触发器需要定位 GdRoleMover）
- `components/shooter` → `pool`（子弹懒注册对象池）
- `dev/suites` → `map` / `state/gjson` / `anim`（被测对象）

## 约定与红线

1. 引擎单例基类必须为 `Object`（手动内存），见 §11。
2. 新增单例：实现类 + 注册/注销函数，并在 `lib.rs` 两个钩子中成对登记。
3. godot-rust 0.5 API 注意：`_ex()` builder 写法、`#[export]` 字段自动生成 getter/setter（勿手写同名方法）、`bind()/bind_mut()` 仅适用于用户自定义类。
