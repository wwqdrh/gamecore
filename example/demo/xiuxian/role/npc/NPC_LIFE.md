# NPC 生活化设计（XiuXian Demo）

> 核心一句话：**NPC 不是等待玩家触发的对话按钮，而是一个持续运行的微型状态机——玩家只是偶然闯入它的生活。**
>
> 落地路线：脚本化日程 + 有限随机（独立游戏路线），不做自由决策 AI。

## 一、注册契约（一个角色 = 一个目录 + 两个脚本文件）

```
role/npc/blacksmith/
├── blacksmith.gd          # 继承 XiuNpcBase，_init 设置名字/站位
├── blacksmith.tscn
├── smith_timeline.txt     # 对话脚本（GdDialogue timeline 语法）
├── schedule.json          # 日程/行为脚本（新）——放了这个文件即启用日程系统
└── assets/dialog_basic.png
```

- **对话脚本**：`timeline_path` 指向本目录 timeline（既有机制）。
- **行为脚本**：角色目录下约定放置 `schedule.json`（自动探测，与立绘探测同范式），
  也可在 npc.gd 中显式导出 `schedule_path` 覆盖。**放置即启用日程系统**，
  未放置时回退旧的 `ai_behavior`（IDLE/WANDER/PATROL）静态行为。

### schedule.json 格式

```jsonc
{
  "jitter_minutes": 10,          // 每日随机扰动上限（分钟），0 = 逐时不差
  "teleport_far": true,          // 目标点太远（>teleport_cells 格）时直接落位
  "teleport_cells": 10,          // 落位阈值
  "ambient_interval": [8.0, 20.0], // idle 行为池触发间隔（秒，随机区间）
  "greet": {                     // 可选：玩家靠近主动问候（GdNpcGreet）
    "lines": ["客官看看货？", "又见面了。"],  // 预设打招呼台词（随机一条）
    "radius": 140.0,             // 触发距离（像素），默认 140
    "cooldown": 25.0,            // 冷却（秒），默认 20
    "line_secs": 3.0,            // 气泡时长（秒），默认 3
    "initial_cooldown": 2.0      // 进场初始冷却（秒），默认 2
  },
  "entries": [
    { "from": 6.0,  "to": 8.0,  "cell": [9, 6],
      "behavior": "idle",        // idle=站桩+行为池 / wander=原地游走
      "lines": ["先烧壶水……", "该磨磨锤子了"] },
    { "from": 8.0,  "to": 12.0, "cell": [12, 8],
      "behavior": "wander", "radius": 64 },
    { "from": 22.0, "to": 6.0,  "cell": [9, 6],   // 跨午夜：from > to 自动回绕
      "behavior": "idle", "lines": ["呼……呼……"] }
  ],
  "chat": {                      // 可选：与另一 NPC 的相遇闲聊（挂发起方）
    "partner": "坊市商人",        // 对方 npc_id（默认取 role_name）
    "my_lines":     ["今天炉火旺，你那布匹不愁卖。", "改日给你打把好刀。"],
    "partner_lines":["承你吉言，坊市最近热闹。", "一言为定！"],
    "chance": 0.6, "cooldown": 120.0, "line_secs": 2.5
  }
}
```

### 时间源

日程按**游戏内小时**（0.0~24.0 浮点）驱动。时间源经 NodePath 注入
（默认自动解析场景根组件表里的 `WeatherManager`，其 `get_hour()` 即当前时刻）。
rust 组件不依赖任何 demo 单例——鸭子调用 `get_hour()`，测试时可注入假时钟。

## 二、组件分层（rust 侧，全部鸭子类型解耦）

| 组件 | 文件 | 职责 |
|---|---|---|
| `GdNpcSchedule` | rust/src/role/npc_schedule.rs | 日程状态机：按小时选段 → 落位/寻路 → 重配 Brain → idle 行为池 |
| `GdNpcChat` | rust/src/role/npc_chat.rs | NPC 闲聊：同图相遇判定 → 双方暂停 → 头顶气泡对聊 → 冷却 |
| `GdNpcGreet` | rust/src/role/npc_greet.rs | 主动问候：玩家靠近边沿触发 → 预制问候浮动气泡 → 冷却 |
| `GdNpcBrain`（既有） | rust/src/role/npc_ai.rs | 到点后的微观行为：IDLE 站桩 / WANDER 游走（BFS 寻路） |
| `XiuNpcBase`（既有，demo 侧） | role/npc/npc.gd | 装配中枢：对话三件套 + 日程/闲聊/问候接线 + 头顶气泡 |

### GdNpcSchedule 关键机制

1. **选段**：`from <= hour < to`（`from > to` 视为跨午夜段），支持每日随机扰动
   —— 扰动量由 `hash(npc_id, 天数)` 决定，同一天内稳定、跨天不同（"偶尔晚 5 分钟出门"）。
2. **落位策略（场景进入 vs 换段过渡）**：
   - **场景进入**：本组件生命周期内首次应用条目时**直接传送显形**到时段位置——
     玩家切图时 NPC 应"已经在那儿"，而不是从实例化坐标走过去。
   - **近距离换段**（≤ `teleport_cells` 格）：正常走过去（可见移动）。
   - **远距离换段**（> `teleport_cells` 格）：出场/入场过渡——先沿路径走到
     **相机视野外**（上限 `exit_walk_cells` 格），在视野外传送到目标端视野外点，
     再从视野外**走进**目标位置。NPC 永不在玩家眼前凭空消失/出现。
     视野判定用视口 canvas transform 逆映射（世界矩形 + 48px 余量）。
   - 不可达（路径为空）：直接落位（兜底）。
3. **导航**：目标格 BFS 吸附到可行走地形（同 npc.gd 落位逻辑），经地图 `find_path`
   走网格路径；导航期间挂起 Brain，到达后 `restart()` 以新位置为家并恢复。
4. **idle 行为池**（消除木桩感）：到点后按随机间隔触发——随机转身（环顾）、
   头顶气泡自言自语（`lines` 池随机）。不需要玩家触发，自己循环。
5. **暂停友好**：对话期间（mover paused）日程冻结，对话结束后自动补位
   （若聊到跨段，会重新走向当前时段的地点）。

### GdNpcChat 关键机制

- 双方都在 `npc_schedule` 分组里（GdNpcSchedule 自动注册），按 `npc_id` 找同伴。
- 触发条件：距离 < `chat_radius`、双方都不在对话/导航中、冷却结束、随机概率。
- 触发后双方 `set_paused(true)`，逐句头顶气泡（`my_lines`/`partner_lines` 交替），
  播完自动恢复、进入冷却。**玩家不参与照样发生**，靠近即"偷听"。
- 同伴半路被释放/切图：逐帧判活，安全终止。

### GdNpcGreet 关键机制

- 玩家进入 `radius` 的**边沿**触发（待在范围内不刷屏），冷却内重复进出不再问候。
- 问候文案解析顺序：宿主 `resolve_greeting() -> String`（非空即用；
  demo 侧 npc.gd 用它实现**重逢问候优先**——`remeet_flag` 已置时返回
  `remeet_greeting`）→ 组件 `lines` 随机一条 → 都为空则本次不显示。
- 抑制条件：宿主 paused（对话中/闲聊中）、赶路中（grid path 活跃）——
  忙碌时接近不问候；对话开始时立即收起气泡给对话框让位。
- 气泡显示：宿主有 `show_bubble` 鸭子调用（与闲聊/自言自语共用一个气泡位）；
  否则自建 Label 子节点兜底（组件继承 Node2D 保证变换链）。
- 玩家解析：`"player"` 分组 > 宿主兄弟节点第一个 GdRoleMover（同 GdDialogTrigger）。

## 三、记忆与状态（复用现有 flag 体系）

对话 timeline 已支持 `@set_flag:xxx` 落 `XiuDialogState`、`[stage@flag]` 门控舞台、
`condition_fn` 触发门控——记忆层零新增框架代码：

- **短期记忆**：timeline 内用 flag 写分支（例：上次聊到哪，下次入口不同）。
- **长期记忆**：关键事件置 flag，后续对话/台词按 flag 切换（例：`[elder_remeet@met_elder]`）。
- **重逢问候**（新增低成本示例）：`remeet_greeting` 导出——玩家靠近时若
  `XiuDialogState` 已有 `met_<role_name>` flag，NPC 头顶主动气泡打招呼，
  而不是每次都"初次见面"。

## 四、环境响应与个人目标（当前版本留接口，后续迭代）

- **时间/天气**：日程条目已按小时分段；天气维度后续可在条目上加 `"weather": "rain"`
  过滤（GdWeatherManager 有 `get_current_weather()`）。
- **世界事件**：EventBus（`GDEVENTBUS` 单例）可广播事件，日程组件后续可订阅
  临时改写日程（怪物袭击→躲屋）。
- **个人目标**：参考文档的"采药人/寡妇"类目标驱动 NPC，落地为
  条目级 `condition`/事件订阅——**不建议首期实现**（工程量爆炸），优先打磨现有
  10 个角色的日程质感。

## 五、分层策略（参考文档第七节）

| 级别 | 角色 | 配置 |
|---|---|---|
| 主线关键 | 铁牛、执事长老、坊市商人、外门弟子 | 完整日程 + idle 池 + 闲聊 + 记忆 |
| 支线 | 白芷、贾先生、萧家三口 | 轻量日程（2~3 段）+ 偶遇闲聊 |
| 路人 | （后续） | 两点位循环 + 随机 idle，无独立记忆 |

## 六、时间流速

`XiuGameConfigState.day_length`：600 → **1200**（游戏内一天 = 现实 20 分钟，
time_scale 86400/1200 = 72 游戏秒/现实秒）。配置 watch 实时生效，测试时可在
设置面板/调试代码里改回或跳时（`WeatherManager.set_total_time`）。

## 七、扩展路线（后续迭代，本期不做）

1. 天气/世界事件改写日程（EventBus 订阅）。
2. 条目级 `condition`（个人目标：采药人采满回家）。
3. 玩家行为感知（奔跑受惊、拔武器警惕）——GdNpcBrain 已有 FLEE/HUNTER 底子。
4. 对话中被打断：NPC 被怪物追赶时对话触发器走 `condition_fn` 拒绝 +
   逃跑喊话气泡（Brain FLEE + show_bubble）。
