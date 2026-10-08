# 对话系统

对应 Rust 源码：`rust/src/dialog/`（gddialogue / speaker / dialog_trigger）
可运行示例：`example/role/`（role_demo.gd + dialog_box.gd）、`example/dialogue/dialogue_example.gd`

## 三件套分工

| 类 | 职责 | 挂载位置 |
|---|---|---|
| `GdDialogue` | 对话控制节点：管理 Timeline、推进、选项 | 场景内放**一个**共享实例 |
| `GdRoleSpeaker` | 把人物节点绑定到 timeline 中的角色名 | 每个参与对话的人物节点下 |
| `GdDialogTrigger` | 在合适的时机启动一场对话 | 每个触发对话的 NPC 节点下 |

## 场景组装

```
NPC (GdRoleMover)
├── AnimatedSprite2D / GdRoleAnimator
├── Speaker (GdRoleSpeaker)    <- role_name = timeline 中的角色名
└── Trigger (GdDialogTrigger)  <- 触发配置
Player (GdRoleMover)
└── Speaker (GdRoleSpeaker)    <- 玩家也要 Speaker（role_name 对应 timeline 玩家角色）
Dialogue (GdDialogue)           <- 共享，供各 trigger 用 NodePath 指向
```

## GdDialogue — 对话控制

```gdscript
var dialogue = GdDialogue.new()
dialogue.set_timeline_path("res://example/role/demo_timeline.txt")  # timeline 文本文件
add_child(dialogue)

# 对话 UI 回调：注册一个 control 节点，GdDialogue 会回调它的 handle_line
dialogue.set_dialogue_control_path(dialogue.get_path_to(dialog_box))

dialogue.next("")                 # 推进一行
dialogue.exec_response(item, current_role)  # 执行选项
dialogue.is_playing()             # -> bool
dialogue.s_finished.connect(_on_dialogue_finished)
```

低层用法（无场景组装，直接喂文本）：

```gdscript
var d = ClassDB.instantiate("GdDialogue")
d.initial(text_content)           # 直接注入 timeline 内容
d.next("")
d.s_finished.connect(...)
```

对话 UI 侧实现 `handle_line` 回调（参考 `example/role/dialog_box.gd`）：

```gdscript
func handle_line(line: Dictionary):
    # line = {name, text, stage, response: [{text, fn, stage}, ...]}
    name_label.text = line["name"]
    text_label.text = line["text"]
    _build_responses(line.get("response", []))
```

## GdRoleSpeaker — 角色绑定

```gdscript
var speaker = GdRoleSpeaker.new()
speaker.role_name = "旅人"        # 必须与 timeline 中的角色名一致
npc.add_child(speaker)
```

触发器启动对话时自动把双方 Speaker 注册进 GdDialogue，对话框即可按角色名定位
人物节点（站位、朝向、动画联动）。

## GdDialogTrigger — 触发器

```gdscript
var trigger = GdDialogTrigger.new()
trigger.trigger_mode = 1          # 0=TRIGGER_PROXIMITY 靠近 / 1=TRIGGER_INTERACT 交互键
                                  # 2=TRIGGER_AUTO 延时自动 / 3=TRIGGER_MANUAL 仅代码调用
trigger.trigger_radius = 100.0    # 靠近/交互的判定半径
trigger.interact_action = "interact"  # 交互动作（InputMap 缺失回退 E 键）
trigger.auto_delay = 1.5          # AUTO 模式的延时（秒）

trigger.dialogue_path = NodePath("../../Dialogue")  # 指向共享 GdDialogue
trigger.entry_stage = "elder_first"                 # 入口 stage 名
trigger.timeline_path = "res://.../elder_timeline.txt"  # 本 NPC 专属 timeline（触发时加载）
trigger.condition_fn = "has_flag:met_elder"         # 条件门控（可选）
trigger.pause_roles = true        # 对话期间暂停双方移动、互相面向
npc.add_child(trigger)

# 手动模式（TRIGGER_MANUAL / 剧情脚本驱动）
trigger.start_dialog()            # -> bool
trigger.cancel_dialog()
trigger.is_dialog_active()
```

**条件门控**：`condition_fn = "方法名[:参数1,参数2]"`，依次在
触发器自身 → 宿主人物 → 对话 control 节点上查找并调用，返回 falsy 则不触发。
配合 timeline 内 `@set_flag` 可实现"对话改变状态 → 影响后续触发"的闭环：

```gdscript
# 任意被查找对象上实现
func has_flag(flag: String) -> bool:
    return flags.has(flag)
```

同一场对话期间不会重复触发；对话结束自动恢复移动。

## 选项动作 → 命令字（对话驱动 UI / NPC 行为）

选项动作与行函数都是 `fn[:参数]` 表达式，支持 `;` 链式（`@open_ui:StoreModal;end`）。
执行优先级：**对话 control（DialogBox）→ role 节点（NPC 宿主）**，两层互不引用：

- **全局 UI 命令**写在 DialogBox 上：`open_ui:ID` / `close_ui:ID` 经
  `GdUIManager.find_ui(id).open()/close()` 开合任意注册组件（Modal 等）——
  对话内容只发命令，接收方各自实现，零耦合。UI 里的对应组件需声明 `ui_id`。
- **NPC 专属动作**写成 npc.gd 上的方法（control 没有该方法时回退 role 节点）。

```gdscript
# DialogBox 侧（已内置）
func open_ui(ui_id: String) -> void:
    var ui = GdUIManager.find_ui(ui_id)
    if ui != null: ui.call("open")
```

```text
# timeline 写法（选项动作 / 行函数皆可）
- 让我看看货品。@open_ui:StoreModal;end   # 打开商店并结束对话（end → s_finished → 触发器复位）
@set_flag:met_elder                        # 状态命令（DialogBox 状态函数）
```

选 UI 命令后要接 `;end` 结束对话：`end` 走 `goto_end + next()` → `is_playing=false`
→ `s_finished` → 触发器 pause_roles 自动复位，避免商店开着而双方仍被暂停。

## DialogBox 层级与鼠标推进（UI 装配要点）

- **嵌套 CanvasLayer 全局排序**：DialogBox 作为 DialogLayer（layer=2）子节点，
  自身默认 layer=1 与 UILayer(1) 同层 → 会被 mainhud 道具栏遮挡。
  必须显式 `dialog_box.layer = 3`（高于一切游戏 UI）。
- **全屏点击捕获**：DialogBox 内放一个 `PRESET_FULL_RECT + MOUSE_FILTER_STOP`
  的透明 Control（`click_catcher`，先挂 → 选项按钮后挂绘制在上优先吃输入），
  `gui_input` 左键推进并 `set_input_as_handled()`。对话期间拦截一切点击：
  任意位置可推进 + 防误触世界寻路/HUD 按钮；E/空格仍走 `_unhandled_input`。
  visible 与对话框同步（handle_line 时 show，_close 时 hide）。

## 修仙 demo 完整装配范本（推荐参考）

`example/demo/xiuxian/`：主城 town.tscn + 角色按目录拆分（每个角色一个文件夹，
含 脚本 + 场景 + 专属 timeline），E 键交互完整链路：

```
role/
├── npc/
│   ├── npc.gd         XiuNpcBase 共享基类（对话三件套 + AI Brain + 吸附装配）
│   ├── elder/         执事长老：elder.gd+tscn+elder_timeline.txt，AI_IDLE 站桩
│   ├── merchant/      坊市商人：merchant.gd+tscn+merchant_timeline.txt，AI_WANDER 游走
│   └── disciple/      外门弟子：disciple.gd+tscn+disciple_timeline.txt，AI_PATROL 巡逻
├── player/            玩家（网格四向 + 点击寻路 + 远程射击）
└── enemy/             小怪基类 XiuEnemyBase + 变体场景（视野追击，见 combat.md）
```

- **角色子类**：`extends XiuNpcBase`，`_init()` 设默认导出值
  （display_name/role_name/timeline_path/entry_stage/npc_cell/ai_behavior/
  patrol_cells/body_color）；共享 Dialogue 不预载 timeline——触发对话时经
  `trigger.timeline_path` 自动加载本角色的文件（切换/重复触发均复位）。
- **AI 移动**：网格地图下 Brain 经地图 `find_path` BFS 逐格走（不穿水域/山地，
  见 role.md「网格地图适配」）；落位吸附后 `brain.restart()` 重设游走中心。
- **NPC 组件**：`role/npc/npc.gd`（`class_name XiuNpcBase extends GdRoleMover`，
  control_mode=0 网格走 grid_path 队列）。代码装配 Speaker + Trigger +
  Brain（按 ai_behavior）；站位/巡逻格不可行走时 BFS 吸附；共享 GdDialogue
  经 `"__gd_dialogue"` 分组解析并接线 dialogue_path（_process 重试，
  **禁止 call_deferred 自重试**）；s_trigger_enter/exit 驱动头顶"E 对话"提示。
- **对话层**：`ui/dialog/dialog_layer.gd`（CanvasLayer，layer=2）——创建共享
  GdDialogue（**不预载 timeline**，触发时由 trigger 加载各角色自己的文件）
  + DialogBox（**layer=3 置顶**，全屏 click_catcher 鼠标推进，open_ui/close_ui
  命令分发）。
- **玩家侧**：Player 加入 `"player"` 分组（触发器解析玩家）+ GdRoleSpeaker 子节点。
- **地图侧**：NPC 场景作为地图场景（town.tscn）子节点实例化，随地图加载/释放；
  主场景 MapManager `initial_map = "xiuxian_town"` 默认进主城。
- **验收**：`check_npc_flow.gd`（落位/接线/timeline 归属/AI 行为与网格游走合法性/
  范围边沿/E 触发/推进/复位/解除暂停/商人选项 open_ui 开商店）。注意 E 键边沿：
  release 与 press 同物理帧会丢边沿，测试里两次按键间必须 `await physics_frame`
  消费释放状态。
