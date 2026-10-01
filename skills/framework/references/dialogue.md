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
