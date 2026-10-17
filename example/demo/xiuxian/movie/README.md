# 剧情演出（movie）

`GdMoviePlayer`（rust 侧 `rust/src/movie/`）驱动的剧本化演出，是对话
（dialog）的功能扩展：在台词之外编排**场景加载 / 人物出场退场移动 /
相机运镜**。每个剧情演出一个文件夹（剧本 `.movie` + 说明）。

## 剧本 DSL 速查

行式语法，`#` 注释，`:label` 标签；完整语法见 `rust/src/movie/script.rs` 文件头。

| 指令 | 语法 | 等待语义 |
|---|---|---|
| scene | `scene <地图别名> [(x,y)]`（坐标兼容保留，站位统一用 player 声明） | open_map 后等 2 帧装配；同图跳过重开；跨图演出结束自动回迁原地图 |
| player | `player (x,y)` | 玩家瞬移落格（显式演出站位声明，防存档恢复位置跑偏） |
| actor | `actor <id> enter [(x,y)] [from (x,y)]` / `actor <id> exit [to (x,y) \| left\|right\|up\|down]` / `teleport (x,y)` | enter from 走完到场；exit 走出画面才消失 |
| move | `move <id> (x,y)` | 网格寻路走完（超时 20s 兜底落格；无寻路地图走 L 形路径） |
| camera | `follow <t>` / `unfollow` / `pan (x,y) [dur]` / `focus <t> [dur]` / `unfocus` / `shake <amp>` / `snap (x,y)` / `zoom <n>` / `reset_zoom` / `blur <v>` | pan/focus/shake 按时长等待 |
| say | `say <id> [秒] <台词>` | 等玩家点击推进；带秒则定时继续 |
| bubble | `bubble <id> <秒> <台词>` | 气泡显示，等秒数 |
| wait | `wait <秒>` | 定时 |
| flag | `flag <名>` / `if <名> jump <label>` / `jump <label>` | 走 dialog control 的 set_flag/has_flag |
| end | `end` | 收尾（解锁玩家/清角色/发信号） |

target 统一为 actor id，特殊值 `player` 指向注入的玩家节点。

**scene 快照回迁**：演出是"暂借舞台"——跨图 `scene` 切图前框架自动记录
原地图与玩家所在格，`end`/播完/`stop` 时切回原图并归位玩家（演出后要真的
换图，由调用方监听 `s_movie_finished` 自行 open_map）。

**进出场走位**：`enter ... from (x,y)` 让角色从画面外格走进目标格（L 形
路径：先横后纵），避免闪现登场；`exit to (x,y)` / `exit left|right|up|down`
让角色走出画面外一格后才移除，避免原地消失。方向词按地图半宽/半高取
画面外落点。

## 台词 UI（rust 框架默认，零配置）

say/bubble 的黑框与头顶气泡由 **rust 侧默认实现**提供
（`rust/src/dialog/movie_ui.rs`：`GdMovieDialogBox` + `GdMovieBubble`），
demo 侧不再自建。若 `dialog_box_path` 注入了宿主（鸭子契约
`show_movie_line` / `movie_line_done` / `close_movie_box`）则优先用宿主；
未注入自动懒创建默认黑框。npc 闲聊/问候/对话气泡统一走 `show_bubble_on`
helper：宿主有 `show_bubble` 复用之，否则挂默认气泡。

## 节点引用（game_config.json `movie.actors`）

`id → {scene: tscn路径, display_name: 展示名}`；`enter` 按场景实例化挂到
当前地图（走 npc.gd 的 `npc_cell` 自吸附可行走格），`exit`/演出结束自动
释放。`player` 只配 `display_name`（say 用），节点本身经 `player_path` 注入。

## 接入方式

1. 场景装配 `GdMoviePlayer` 节点（或测试动态 `new`），`player_path` 指向
   Player；`dialog_box_path` 可选（缺省用 rust 默认黑框）；
2. `movie.play()`（或 `play_from(":label")`），await `s_movie_finished`；
3. 演出期玩家被 `set_paused(true)` 锁定，结束自动解锁。

本目录剧本：

| 文件夹 | 剧情 | 触发点 | 接线状态 |
|---|---|---|---|
| `prologue/` | 农田醒来开场 | 新档首次进主场景 | ✅ 已接线：`scenes/main/movie_trigger.gd`（播放记录存 `XiuMovieState`） |
| `breakthrough/` | 突破斗者 | `try_breakthrough` 成功 | 未接线 |
| `selection_eve/` | 宗门选拔前夜 | stage1_clear 后 | 未接线 |
| `c01_farewell` … `c20_vow_hometown` | **宗门篇《青云问道》20 段**（人物/情节/拆分见同目录 `character.md` / `detail.md` / `clip.md`） | 各段触发建议见 `clip.md` | 未接线（新图 `xiuxian_sect` 已注册，经夺魁领奖 `unlock_map` 点亮） |

`test_smoke/` 是 headless 回归剧本（`test/test_movie_player.gd` 使用）。
宗门篇 20 段剧本验收：`example/demo/xiuxian/check_movie_clips.gd`
（actor 注册 / 解析 / 玩家站位可行走 / 山门传送与门控）。
触发器接线回归：`test/test_movie_trigger.gd`（自动开播 / 播完落盘 / 重进不重播）。
