# 第一阶段「家族篇 · 萧宅起步」设计与实现对照

> 源设计：`story/04_stage1_design.md`（本文不重复叙事细节，只做**设计 → 实现位置**
> 的对照索引，供后续检查与阶段二复用）。实施记录见 `stage1_milestones.md`，
> 实施引出的框架新增能力已沉淀到框架 skill 文档（`skills/framework/references/`：
> ui-gml.md 响应式显隐/Toast/Banner、state-data.md 删档语义、SKILL.md 测试红线）。

---

## 1. 阶段定义

**第一阶段（家族篇）= 萧宅起步 → 青石镇历练 → 突破斗者 → 宗门选拔前十。**
**第二阶段（宗门篇）= 录入青云宗外门 → 宗门任务线。**

舞台：`home`（萧宅农田）→ `xiaozhai`（萧宅）→ `xiaozhen`（青石镇）→
`town`（青云坊）四张固定图 + 秘境随机副本；青草岭（map_a）/幽竹林（map_b）
为第二阶段舞台（传送点门控，突破斗者后 map_a 解锁）。

### 通关三硬条件（T1/T2/T3）

| 条件 | 实现位置 |
|---|---|
| T1 主线链完成（`stage1_clear`） | `state/task/stage1.json`（main_101→…→sect_tournament 链）+ `state/task/task_state.gd`（complete_task 收束）；flag 置位在 `arena_timeline.txt` 夺魁领奖选项 |
| T2 突破斗者 | `state/character/character_state.gd` `try_breakthrough`（等阶表 breakthrough 判定） |
| T3 宗门选拔前十（token_sect 引荐令） | `role/npc/xiao_master/master_timeline.txt`（真相段赠聚气散×3）→ `arena_master/arena_timeline.txt`（报名+擂台夺魁） |

验收：`test/check_stage1_clear.gd`（真实 index.tscn 全链冒烟，T1+T2+T3 断言）。

### 出生点（2026-10-15 修正）

设计 beat 0 = home 农田「清晨醒来」，实现曾配在 xiaozhai，已纠正：

| 项 | 值 | 位置 |
|---|---|---|
| 新档出生图 | `xiuxian_home`（GdDigLayer 挖地图） | `scenes/main/index.tscn` MapManager `initial_map` |
| 出生格 | `Vector2i(9, 6)`（农田区可行走格，旁 (10,6) 即教学锄地位） | 同上 `initial_spawn` |
| 开图恢复 | 无记录→initial_map；有记录→存档位置图（两级恢复） | `scenes/main/map_restore.gd` + `role/player/player.gd _resolve_spawn_position` |
| 小地图兜底 | 地图未开时 alias 兜底 `xiuxian_home` | `scenes/main/ui/map/map_modal.gd` |
| 传送门控 | 落格命中传送点先查目标图锁，未解锁 Toast 拦截；解锁走 `unlock_map`（突破斗者自动解锁 map_a） | `player.gd` 前置只读查锁（rust `get_portal_target_at`）+ `state/map/map_state.gd` |
| 出生格原则 | 压边线格+旁一格；远离本图传送点（落本体=二次触发弹回）；无头探针实测可行走 | 各图改动后跑 `test/check_map_portal.gd` + `regen_map_thumbs.gd` |

---

## 2. 剧本 beat → 实现位置（2026-10-16 任务重排后）

| Beat | 场景 | 内容 | 实现位置 |
|---|---|---|---|
| 0 ★ | home | 农田劳作教学（移动→锄→浇→种→收→开地图） | `mainhud_guide.gml/.gd`（教学 4 连，`tutorial_shown` 门控）+ `side_104` 灵田初耕（`map/home_farm_task.gd` 监听 GdDigLayer s_dug/s_planted/s_watered 计步） |
| 1 | xiaozhai | main_101 初来乍到（拜见萧老爷）+ side_101 家主嘱托 | `xiao_master/master_timeline.txt`（master_meet / master_side_offer 段） |
| 2 | xiaozhai | main_102 宅中众人（拜会萧夫人 + 小翠） | `xiao_lady/lady_timeline.txt`（lady_main102 段接取+推进）+ `xiao_maid/maid_timeline.txt`（maid_main102 段） |
| 3 | home | main_103 灵田风波（杀地灵鼠×3 → 回报小翠） | `xiao_master`（master_main103_offer 接取）+ `map/home_pest.gd`（kill 步）+ maid 回报段（step1 门控） |
| 4 | xiaozhen | side_102 采药济世 / side_103 铸剑问道（采集/锻造教学，collect auto 自动推进） | `herb_shop/herb_timeline.txt`、`blacksmith/smith_timeline.txt`（接取/交付/取剑段） |
| 5 | home→镇 | main_104 坊市风声（贾先生失窃说书置 theft_rumor → 外门弟子令牌段置 token_clue） | `storyteller/story_timeline.txt`（story_main104 段）+ `disciple/disciple_timeline.txt`（disciple_main104 段）+ `xiao_maid`（maid_main104_offer 接取） |
| 6 | xiaozhai | main_105 破关之资（master_gift 真相段：吐父亲旧事 + 赠聚气散×3）→ 突破斗者 → 复命 | `master_timeline.txt`（master_gift / master_break 段）+ `character_state.gd _on_rank_entered`（rank_up 步联动） |
| 7 ★ | town | sect_tournament 宗门选拔（第 7 天日历自动接取 → 秦重报名 → 擂台夺魁前十） | `calendar_manager.gd`（day7 unlock+accept）+ `arena_master/arena_timeline.txt` + `role/npc/arena_master/arena_master.gd`（start_match/夺魁计步）；夺魁领奖置 `stage1_clear` |

**timeline 红线**（本阶段实测沉淀）：变更本段 gate 状态的 flag/counter
必须写进**选项动作**（exec_response 内变更+goto 连续执行不经 stage 检查）；
行尾函数在行显示时执行，写在行尾会断掉本段选项链。静态 lint：
`test/check_stage1_dialog.gd`（goto 目标/flag 溯源/危险变更/链式 @）。
**entry_stage 必须置空**（2026-10-16）：entry 直达段（尤其无条件兜底段）
会绕过 gate 优先级扫描，任务段永远进不去——置空后 start_from 从头扫描取
第一个满足段。

---

## 3. NPC 清单（10 位）→ 目录对照

| NPC | 目录（`role/npc/`） | 场景 | 本阶段任务位 |
|---|---|---|---|
| 萧老爷（萧远山） | `xiao_master` | xiaozhai | main_101 接取 / side_101 嘱托 / main_103 接取 / main_105 真相段赠药+复命 |
| 萧夫人（柳氏） | `xiao_lady` | xiaozhai | main_102 接取+step1 / side_104 灵田初耕接取（送种子） |
| 小翠 | `xiao_maid` | xiaozhai | main_102 step2 / main_103 回报 / main_104 接取 / side_101 完成 |
| 说书先生（贾先生） | `storyteller` | xiaozhen | main_104 step1（失窃说书，置 theft_rumor） |
| 外门弟子（苏小棠） | `disciple` | town | main_104 step2（令牌段，置 token_clue）+ 选拔规矩 |
| 白芷 | `herb_shop` | xiaozhen | side_102 接取/交付（collect auto 教学） |
| 铁牛 | `blacksmith` | xiaozhen | side_103 接取/交料/取剑 |
| 秦重 | `arena_master` | town | sect_tournament 报名+夺魁领奖（置 stage1_clear） |
| 孙半城 | `merchant` | town | 失窃闲谈氛围段（main_104 期间） |
| 执事长老（玄诚） | `elder` | town | **第二阶段**：main_201 报到 / daily_201 签到 |

NPC 注册契约：各目录 `*_timeline.txt`（对话）+ `schedule.json`（日程），
零代码接入；组件层 `rust/src/role/npc_schedule.rs` / `npc_chat.rs`。

---

## 4. 敌人与战斗位 → 实现位置

| 敌人 | 数值 | 实现位置 |
|---|---|---|
| 地灵鼠 | 攻3/血20/经40，快/近视 | `role/enemy/mouse.tscn`（slime 换皮） |
| 地灵鼠事件 | main_103 accepted 才刷 ×3，清剿推 1 步 | `map/home_pest.gd`（SPAWN_CELLS (7,5)/(12,8)/(9,10)，挂 home.tscn PestEvent 节点） |
| 草妖 slime | 攻5/血30/经60 | `map/dungeon/dungeon_map.gd` |
| 二层缩放怪 | 血×1.6 攻×1.5 经×2 | 同上 `_spawn_enemy(mult)` |
| 妖王（三层 boss） | 攻14/血320/防6/经500，首杀掉 `map_secret` | 同上 |

---

## 5. 任务链与解锁 → 实现位置（2026-10-16 重排）

### 第一阶段（家族篇）`state/task/stage1.json`——10 条

| 任务 | 流程 | 关键实现 |
|---|---|---|
| main_101 初来乍到 | talk 萧老爷（教学：对话） | master_meet 段接取+推进 |
| main_102 宅中众人 | talk 萧夫人 + talk 小翠（教学：寻路） | lady_main102 / maid_main102 段 |
| main_103 灵田风波 | kill 地灵鼠×3 → talk 小翠（教学：战斗） | `home_pest.gd` kill 步 + maid 回报段（step1 门控） |
| main_104 坊市风声 | talk 贾先生 → talk 外门弟子（教学：探索；theft_rumor→token_clue 暗线） | story/disciple timeline 段 |
| main_105 破关之资 | collect 聚气散×3 → rank_up 斗者 → talk 萧老爷 | master_gift 段赠药+接取；`character_state` rank_up 联动；奖 token_sect |
| sect_tournament 宗门选拔 ★ | talk 秦重报名 → kill 比武弟子（**前 10 名**，stage1 收官） | `calendar_manager.gd` 第 7 天自动接取；`arena_master.gd` 计步；夺魁领奖置 `stage1_clear` |
| side_101 家主嘱托 | talk 小翠 | master_side_offer / maid_side101 段 |
| side_102 采药济世 | collect herb_xueshen（**auto**）→ talk 白芷（教学：采集） | herb_timeline 段；背包监听自动推进 |
| side_103 铸剑问道 | collect ore_coldiron（**auto**）→ talk 铁牛 ×2（教学：装备） | smith_timeline 三段 |
| side_104 灵田初耕 | 锄地→播种→浇灌（教学：农事，GdDigLayer 信号计步） | `map/home_farm_task.gd`（挂 home.tscn FarmTaskEvent） |

### 第二阶段（宗门篇）`state/task/stage2.json`——5 条

| 任务 | 流程 |
|---|---|
| main_201 宗门报到 | goto 山门 → talk 执事长老（elder_report 段）→ goto 弟子舍；前置 sect_tournament |
| main_202 秘境探幽 | 秘境三层（原 main_003 内容）；前置 main_201 |
| daily_201 每日签到 | elder_daily 段接取+推进 |
| guild_201 宗门巡逻 | 演示级（goto/kill 步） |
| bounty_201 悬赏·血衣楼 | 前置 guild_201（演示级） |

### 机制

- **定义表拆分**：`stage1.json` / `stage2.json`（各带 pipeline:gjson）→
  `XiuTaskTable` 扫描 `state/task/` 目录合并 *.gjson 为一张总表
  （范式同 XiuItemTable；同 id 后者覆盖告警；新增篇章文件零代码接入）。
- **解锁链**：`task_state.gd complete_task` 尾 `_unlock_successors`
  （解锁状态按前置链推导并随 progress 持久化，定义表无 unlock 字段）。
- **任务 flag 三态**：`task_<id>_accepted`（接取）/ `task_<id>_step<N>`
  （advance_task 自动置——供 timeline 步级门控，如「杀完鼠才能回报」）/
  `task_<id>_done`（提交）。
- **collect(auto) 自动推进**：objectives 步带 `auto:true` 时，`XiuTaskState`
  watch `XiuItemState.bag`（call_deferred 防借用重入）→ 背包获得目标物品即
  advance（采集/购买/赠药统一走 add_item，一条监听全覆盖）。剧情赠药型
  collect（main_105）不加 auto，由 timeline 显式 advance 防跳步。
- **任务目录引导**：可接取页签只列 available（📍 找〈giver〉接取），
  locked 不进任何页签；三页签流转 + HUD 横幅三级优先级（进行中→待领取→可接取）。

**突破联动**：`character_state.gd try_breakthrough` 成功 →
`_on_rank_entered`：置 `rank_<阶id>` flag；斗者阶时 `unlock_map(xiuxian_map_a)`
+ 推进 main_105 rank_up 步。地图门控：`state/map/map_state.gd`
`map_unlocked`（持久化）+ STARTER_MAPS 恒解锁；玩家落格查锁
`player.gd`（rust 侧只读 `get_portal_target_at`，未解锁发 `s_portal_locked`）。

---

## 6. 资源与经济 → 实现位置

| 资源 | 位置 |
|---|---|
| 聚气散 break_qizhe | beat 7 长老对话赠 3（商店未接购买，不进关键路径） |
| item 获得方式字段 | 定义表 `source` 字段，背包详情卡「📍 获得方式」 |
| 定义表管线 | `state/xxx.json`（"pipeline":"gjson"）→ 编辑器实时转；CLI 批转 `test/regen_gjson.gd` v2（内容比对判新鲜 + 单文件 + 回读校验） |
| 经验闭环 | 斗之气 1→10 共需 5500 修为：任务 ~1800 + 副本一轮 ~1200×2~3 + 签到 80/日 |

---

## 7. 引导与演出 → 实现位置

| 演出 | 触发 | 实现位置 |
|---|---|---|
| 教学 4 连 | 首次进主场景（`tutorial_shown` 门控） | `mainhud_guide.gd`（0.5s 节流轮询状态层快照，非 Bean watch——装配期重入规避） |
| 传送门控提示 | `s_portal_locked` 信号 | mainhud_guide 接信号 → Toast |
| 突破演出 | `get_realm_title()` 变化（跨大阶才播，防每级连灌） | 同上 → Banner |
| 通关字幅 | flag `stage1_clear` | 同上 → Banner「第二阶段开启」 |

UI 组件（rust 侧 gml 框架）：`<Toast>`（`rust/src/ui/ui_toast.rs`）、
`<Banner>`（`ui_banner.rs`，播放队列上限 4）。设计原则：演出逻辑在
gd 侧薄接，组件能力全在 rust 侧（见 framework_ui_additions.md）。

---

## 8. 验收测试矩阵

| 测试 | 覆盖 |
|---|---|
| `test/check_stage1_clear.gd` | 真实 index.tscn 全链通关冒烟（教学→任务→刷鼠→突破演出→复命→通关字幅→map_a 切换）；**轮询用墙钟时限**（无头帧率不节流） |
| `test/check_stage1_flow.gd` | 状态层端到端（接→做→交→collect auto→突破→flag→选拔→stage2 点亮） |
| `test/check_stage1_dialog.gd` | 10 份 timeline 静态 lint + 动态 |
| `test/check_stage1_guide.gd` | Toast/Banner 组件单测 + mainhud 集成 |
| `test/check_stage1_enemy.gd` | 杀 3 鼠推进任务 |
| `test/probe_stage1_visual.gd` | 真实渲染器截图 Toast/Banner |
| `test/check_map_flow.gd` | 出生点 home + 萧宅建筑 + 传送流转（2026-10-15 跟进出生点） |
| `test/check_title_save.gd` | 存档选择 + 删除存档闭环 |

回归基线：test_runner **166/166**（32 套件，2026-10-16）。

---

## 9. 第二阶段（宗门篇）入口

- main_201 报到（elder_report 段已接好；goto 步推进待换图钩子或演示直调）
- main_202 秘境探幽（map_secret 三层 + 妖王首杀掉残图）
- daily_201 每日签到 / guild_201 宗门巡逻 / bounty_201 悬赏·血衣楼
- 幽竹林 map_b 开放
- 孙半城失窃案展开（token_clue 伏笔接三幕剧本）
