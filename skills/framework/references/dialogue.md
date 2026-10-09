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
    # line = {name, text, stage, response: [{text, fn, stage}, ...],
    #         display_name: 展示名(可选, Speaker 注册时随行下发),
    #         portrait: 立绘路径(可选, 空键=未配置)}
    name_label.text = line.get("display_name", "") or line["name"]
    text_label.text = line["text"]
    _show_portrait(line.get("portrait", ""))
    _build_responses(line.get("response", []))
```

## GdRoleSpeaker — 角色绑定

```gdscript
var speaker = GdRoleSpeaker.new()
speaker.role_name = "旅人"        # 必须与 timeline 中的角色名一致
speaker.display_name = "老旅人"   # 可选：UI 显示名（非空时替代角色名）
speaker.portrait = "res://.../assets/dialog_basic.png"  # 可选：对话立绘
npc.add_child(speaker)
```

触发器启动对话时自动把双方 Speaker 注册进 GdDialogue（register_role_node +
register_role_meta），对话框即可按角色名定位人物节点（站位、朝向、动画联动）。

## 角色立绘（portrait）

- 链路：Speaker.portrait → 触发器 `register_role_meta(role, display_name,
  portrait)` → `next()` 组行时注入 `line.display_name / line.portrait` →
  DialogBox `_set_portrait()` 按行切换（空路径/加载失败隐藏，无立绘角色
  如玩家不显示）。
- 查询 API：`GdDialogue.get_role_portrait(role)` / `get_role_display_name(role)`
  （未注册返回 ""）。
- 显示参数（DialogBox 范本）：TextureRect `EXPAND_IGNORE_SIZE +
  STRETCH_KEEP_ASPECT_CENTERED`（任意尺寸素材等比自适应居中），槽位 400x696
  竖版比例（素材 1152x2048 / 1520x2720；2x 大小），悬浮在对话框**外部左侧**
  （panel 子节点负坐标 position(-416,-490)，右缘距面板左缘 16px、底边与面板
  底边齐平），不挤压面板内文本区；1920x1080 下左缘 x≈94、顶边 y≈360 不出屏；**mouse_filter 必须 IGNORE**（点击穿透到 click_catcher
  推进对话，否则立绘吞点击）。
- 约定：立绘放角色脚本同目录 `assets/dialog_basic.png`——demo NPC 由
  XiuNpcBase 自动探测填入（`portrait` 导出可显式覆盖），零配置。

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

**最近触发**：多个 NPC 触发圈重叠时（触发器 ready 自动入 `"dialog_trigger"`
分组），只有离玩家最近的那个能启动对话——让位方经 PROXIMITY 重试 /
INTERACT 下次按键自然后继。`cancel_dialog()` 会同时停掉共享 GdDialogue
的播放（否则 `is_playing=true` 会卡住其他触发器）。

## 对话进度记录（stage flag 门控）

timeline 的 stage 可声明 flag 条件，GdDialogue 启动/推进时经 precheck
（control 的 `has_flag`）过滤，**不满足条件的段落自动跳过**——这是
"对话看过一次、之后不再重复" 的实现基础：

```text
[stage名@flag1;!flag2]        # 全部满足才可进入；! 前缀取反
[stage名@has_flag:flag1]      # method:args 形式（裸标记等价 has_flag:xxx）
```

- **选段机制** `GdDialogue.start_from(entry)`：entry 的 flag 满足则直达；
  否则从 timeline 开头扫描取**第一个满足条件的 stage**（一个都不满足则
  结束）。触发器 start_dialog 内部走的就是这个入口。
- **写作红线**（违反会断对话/选错段）：
  1. 置 flag 的 stage（如初见段 `@set_flag:met_xxx`），本段结尾必须是
     选项或 goto——行推进时当前 stage 的 flag 已失配会被判定过期直接
     goto_end；
  2. 门控段落必须排在无条件段落**之前**（扫描取第一个满足的 stage，
     无条件 stage 恒通过）；无条件分支段只能放文件尾部经 goto 进入。
- **持久化**：flag 存对话 control 的 `has_flag`/`set_flag`。demo 里
  DialogBox 委托 `XiuDialogState`（GdBean，随存档持久化）→ 对话进度跨
  运行保留；框架侧 control 是任意实现 has_flag 的节点。
- 典型结构（初见 / 日常再访 两段式）：

```text
[npc_intro@!met_npc]
(角色,玩家)
首次见面的自我介绍……
@set_flag:met_npc
- 选项A@goto:npc_branch
- 告辞。@goto:npc_end

[npc_catchup@met_npc]
(角色)
熟络后的简短寒暄……
- 选项A@goto:npc_branch
- 告辞。@goto:npc_end
```

## 任务联动（demo 命令字）

`XiuTaskState`（见 state-data.md「静态定义表管线」）接取/完成任务时把
`task_<id>_accepted` / `task_<id>_done` 写入 XiuDialogState → 台词本用
flag 门控任务段落，任务结束后对应选项自动消失：

```text
[npc_task_offer@met_npc;!task_side_003_accepted;!task_side_003_done]
(角色,玩家)
交代任务的台词……
- 接受。@goto:npc_task_ok          # 任务落点行挂 @task_accept
- 改日再说。@goto:npc_end

[npc_task_ok]
(角色)
应承台词。
@task_accept:side_003              # DialogBox → XiuTaskState.accept_task
:goto:npc_end

[npc_task_pending@met_npc;task_side_003_accepted;!task_side_003_done]
(角色)
任务进行中的催促……

# 完成方 NPC（对话内直接结算奖励）
[npc_task_done_chat@task_side_003_accepted;!task_side_003_done]
(角色,玩家)
交付台词……
@task_complete:side_003            # 完成并发放奖励（coins/exp/items → 各 Bean）
- 收尾选项。@goto:npc_end
```

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
- **玩家侧**：Player 加入 `"player"` 分组（触发器解析玩家）+ GdRoleSpeaker 子节点
  （无 display_name/portrait → 对话回退角色名、不显示立绘）。
- **立绘**：三 NPC 各自 `assets/dialog_basic.png`（XiuNpcBase 自动探测），
  DialogBox 左侧槽位按行切换，验证脚本 `test/check_dialog_portrait.gd`。
- **对话进度 + 任务**：萧宅/青石镇 6 NPC，萧老爷 timeline 四段门控
  （初见→可接任务→进行中→完成日常），验证脚本
  `example/demo/xiuxian/check_task_flow.gd`（最近触发 / 进度跳过 /
  接取 / 50 金币奖励 / 幂等 / 重置）。
- **地图侧**：NPC 场景作为地图场景（town.tscn）子节点实例化，随地图加载/释放；
  主场景 MapManager `initial_map = "xiuxian_town"` 默认进主城。
- **验收**：`check_npc_flow.gd`（落位/接线/timeline 归属/AI 行为与网格游走合法性/
  范围边沿/E 触发/推进/复位/解除暂停/商人选项 open_ui 开商店）。注意 E 键边沿：
  release 与 press 同物理帧会丢边沿，测试里两次按键间必须 `await physics_frame`
  消费释放状态。
