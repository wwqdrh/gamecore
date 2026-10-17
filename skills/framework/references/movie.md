# GdMoviePlayer — 剧情演出（movie）

对话（GdDialogue）的功能扩展：在台词之外编排**场景加载、人物出场/退场/
移动、相机运镜**。提供节点 + 剧本文本（`.movie`）即可演绎。

## 组件（rust/src/movie/）

- `GdMoviePlayer`（`#[class(base = Node)]`）：播放器节点
  - 导出：`script_path`（剧本 res 路径）/ `dialog_box_path`（台词宿主，**可缺省**）/
    `player_path`（玩家节点）/ `auto_free_actors`（结束释放角色，默认 true）
  - 方法：`play()` / `play_from(label)` / `stop()` / `is_playing()` /
    `get_actor(id)` / `configure(script, dialog_box, player)`（代码动态构造用，
    幂等；tscn 装配走 #[var] 自动 setter + ready 统一解析）
  - 信号：`s_movie_started` / `s_movie_finished`
  - 执行模型：`play()` 启动**协程**（框架 SpireCoroutine）逐指令执行；
    say/move/wait/camera 等指令 await 完成再走下一条，jump/if 驱动分支。
    **协程内 bind_mut 不跨 await 持有**（Rust 借用强制了这一模式）
- `script.rs`：行式 DSL 解析器（`parse(text) -> Result<MovieScript, String>`，
  带行号报错；含 rust 单元测试）

## 节点引用（game_config.json `movie` 段）

```json
"movie": {
  "actors": {
    "xiao_master": {
      "scene": "res://.../xiao_master.tscn",
      "display_name": "萧战"
    },
    "player": { "display_name": "萧炎" }
  }
}
```

- `enter` 按 `scene` 实例化挂到当前地图（demo npc.gd 走 `npc_cell` export
  自吸附最近可行走格）；`exit`/演出结束自动 `queue_free`
- `player` 只配 `display_name`（say 显示名），节点本身经 `player_path` 注入
- MapManager / ViewCamera 经 **GdSceneRoot 组件表**查找（框架标准入口），
  不需要在 movie 段重复声明

## 台词 UI（rust 框架默认实现，开箱即用）

- `rust/src/dialog/movie_ui.rs` 提供两件默认 UI，**demo 侧零配置零代码**：
  - `GdMovieDialogBox`（CanvasLayer，`__MovieDialogBox`）：屏幕下方黑框台词，
    点击 / E / 空格推进（`unhandled_input`），结构与普通对话框一致
    （`Panel/TextLabel`、`Panel/NameLabel`）
  - `GdMovieBubble`（Node2D，`__MovieBubble`）：角色头顶气泡，超时自隐
  - `show_bubble_on(host, text, secs)` / `hide_bubble_on(host)` 共享 helper：
    宿主实现 `show_bubble` 则复用宿主样式，否则自动挂默认气泡
- **宿主注入仍支持**：`dialog_box_path` 指向的节点需满足鸭子契约——
  `show_movie_line(line: Dictionary)` / `movie_line_done: bool`（点击置 true，
  播放器轮询）/ `close_movie_box()`；未注入时 say 自动懒创建默认黑框
- npc 三系统（schedule 闲聊 / greet 问候 / chat 对话气泡）统一走
  `show_bubble_on`：demo npc.gd 的自建气泡 Label 与 greet 的自建 Label 已删

## scene 语义（暂借舞台，自动回迁）

- **同图演出**：`scene` 目标与当前地图相同则跳过重开（避免地图重载/NPC
  重建的整场洗牌，prologue 在 home 图开播即走此路径）
- **跨图演出**：首次真正切图前拍**快照**（原地图 alias + 玩家所在格），
  end/播完/stop 时自动 `open_map` 切回原图并把玩家落回原格（等 2 帧装配
  后校正落位）——演出不把世界永久留在剧本最后一张图上
- 剧本若要演出后**真的换图**（剧情位移），由调用方监听 `s_movie_finished`
  自行 `open_map`（显式优于隐式，框架不做双向猜测）
- `auto_free_actors=false` 保留在场的角色，finish 时自动回调
  `set_movie_actor(false)` 恢复日程/对话/寒暄自治（true 时直接释放无需恢复）

## 常驻 NPC 抑制（演出 dedup，2026-10-17）

地图场景内嵌的常驻 NPC 与剧本 actor（同一角色再实例化一份）会同屏重复。
演出期间播放器自动**隐藏当前地图上的常驻 NPC**，结束/stop 恢复：

- 触发点：`play()` 开局 + `scene` 指令（跨图开图后等装配帧、同图跳过）
  各收一遍（`pending_suppress` 延迟到协程下一指令前消费，新图 NPC 已就位）
- 识别契约：节点带 `npc_id` 属性（鸭子识别，框架不感知 demo 类名）；
  **必须同时 `has_method("hide")`**——NPC 子组件 GdNpcSchedule 同样镜像了
  `npc_id` 但纯逻辑 Node 无 hide（踩过：call("hide") panic 炸协程）
- 演出 actor 本尊及其子树豁免（防同图二段抑制把已出场 actor 藏掉）
- 恢复：finish/stop 先 `restore_npcs()` 再回迁快照——跨图演出回迁重载的
  地图上是新 NPC 实例（本来就可见），旧实例条目按有效性自然跳过

## 演出锁

`play()` 对 `player_path` 节点鸭子 `set_paused(true)`（GdRoleMover 剧情
锁），结束/stop 解锁。相机状态剧本自理（结束前 `camera unfocus` +
`follow player` 是常见收尾）。

## DSL 速查

行式语法，`#` 注释，`:label` 标签，坐标 `(x,y)` 括号内允许空格。
完整语法见 `script.rs` 文件头与 demo `example/demo/xiuxian/movie/README.md`。

| 指令 | 语法 | 等待 |
|---|---|---|
| scene | `scene <别名> [(x,y)]`（坐标兼容保留，**站位统一用 player 声明**） | open_map + 2 帧；**同图跳过重开** |
| player | `player (x,y)` | 玩家瞬移落格（显式演出站位；玩家位置随档持久化会覆盖默认落位，对话前用它保证站位靠近） |
| actor | `actor <id> enter [(x,y)] [from (x,y)]`（画面外走入）/ `actor <id> exit [to (x,y) \| left\|right\|up\|down]`（走出画面才消失）/ `teleport (x,y)` | 走位等走完 |
| move | `move <id> (x,y)` | 寻路走完（20s 超时兜底落格；无 find_path 地图走 L 形手动路径） |
| camera | `follow <t>` / `unfollow` / `pan (x,y) [dur]` / `focus <t> [dur]` / `unfocus` / `shake <amp>` / `snap (x,y)` / `zoom <n>` / `reset_zoom` / `blur <v>` | pan/focus/shake 按时长 |
| say | `say <id> [秒] <台词>`（`dur:秒` 兼容别名） | 等点击；带秒则定时 |
| bubble | `bubble <id> <秒> <台词>`（`dur:秒` 兼容别名） | 等秒数 |
| wait | `wait <秒>` | 定时 |
| flag | `flag <名>` / `if <名> jump <label>` / `jump <label>` | 走 control 的 set_flag/has_flag |
| end | `end` | 收尾发 `s_movie_finished` |

## 范例与测试

- demo 剧本：`example/demo/xiuxian/movie/`（prologue / breakthrough /
  selection_eve / test_smoke，各剧情一文件夹）
- headless 集成测试：`test/test_movie_player.gd`（真实主场景全链：
  scene/actor/camera/say dur+点击/bubble/move/flag-if-jump/end/自动释放/
  玩家锁定解锁）；解析器单测在 `script.rs` `#[cfg(test)]`

## 坑与约定

- **演出 actor 必须屏蔽宿主自治系统**（本框架已内置）：spawn 后鸭子调
  `set_movie_actor(true)`——否则宿主 schedule.json 的日程「场景进入直接
  落位」会在演出数秒后把角色瞬移到日程点（实测 3.2s 跳走），Brain 漫游/
  闲聊/问候同理；站位吸附一并禁用（站位由剧本指令全权落位）。宿主按此
  契约实现（demo 范本 npc.gd：关 schedule/chat/greet/brain 的 enable）。
  **注意组件 disable 分支不能每帧收气泡**——greet 曾在 disable 分支逐帧
  `hide_bubble_on`，演出 bubble 指令刚 show 完下一帧就被隐藏（一闪而过）；
  disable 应只冻结触发，气泡靠超时自隐
- **GdRoleMover move_mode 默认 FOUR_WAY 不进 tick_grid**——actor 场景
  若要 `move` 走格子，宿主必须显式 `move_mode = 2`（MODE_GRID）或复用
  demo npc.tscn（已配置）
- say 无 `dialog_box_path` 宿主时**自动懒创建 rust 默认黑框**
  `GdMovieDialogBox`（CanvasLayer，`layer=100` 高于 demo 全屏层段，演出期
  恒最顶；层级总表 demo `ui/ui_layers.gd` XiuUILayers），结束不销毁（复用）；
  bubble 无宿主 `show_bubble` 方法时挂 rust 默认气泡 `GdMovieBubble`（演出
  结束随角色释放）
- `stop()` 不发 `s_movie_finished`（与自然播完区分），且强制清角色；
  同样会做 scene 快照回迁
- move 期间日程组件若接管 grid_path 会破坏演出——演出 actor 是独立
  实例（不挂日程）即无此问题
