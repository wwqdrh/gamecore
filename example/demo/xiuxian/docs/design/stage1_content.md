# 第一阶段「萧宅起步 · 入门青云」设计与实现对照

> 源设计：`story/04_stage1_design.md`（本文不重复叙事细节，只做**设计 → 实现位置**
> 的对照索引，供后续检查与阶段二复用）。实施记录见 `stage1_milestones.md`，
> 实施引出的框架新增能力已沉淀到框架 skill 文档（`skills/framework/references/`：
> ui-gml.md 响应式显隐/Toast/Banner、state-data.md 删档语义、SKILL.md 测试红线）。

---

## 1. 阶段定义

**第一阶段 = 萧宅起步 → 青云坊入门 → 突破斗者。**

舞台：`home`（萧宅农田）→ `xiaozhai`（萧宅）→ `xiaozhen`（青石镇）→
`town`（青云坊）四张固定图 + 秘境随机副本；青草岭（map_a）/幽竹林（map_b）
为第二阶段舞台（传送点门控，突破斗者后 map_a 解锁）。

### 通关三硬条件（T1/T2/T3）

| 条件 | 实现位置 |
|---|---|
| T1 主线链完成（`story_p1_done`） | `state/task/task.json`（main_001→002→004 链）+ `state/task/task_state.gd`（complete_task 收束） |
| T2 突破斗者 | `state/character/character_state.gd` `try_breakthrough`（等阶表 breakthrough 判定） |
| T3 长老赠 `token_sect` | `role/npc/elder/elder_timeline.txt`（真相段选项动作 `item_give` + `task_advance`） |

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

## 2. 剧本 8 beat → 实现位置

| Beat | 场景 | 内容 | 实现位置 |
|---|---|---|---|
| 0 ★ | home | 农田劳作教学（移动→锄→浇→种→收→开地图） | `scenes/main/ui/mainhud/mainhud_guide.gml/.gd`（教学 4 连，`tutorial_shown` 门控） |
| 1 | xiaozhai | 萧老爷寒暄 + 托付看顾小翠（side_003） | `role/npc/xiao_master/*_timeline.txt`（已有段） |
| 2 ★ | xiaozhai | 小翠对话（祖产近况） | `role/npc/xiao_maid/*_timeline.txt` 续写段 |
| 3 | xiaozhen | 白芷采药 / 铁牛铸剑支线 | `role/npc/herb_shop`、`blacksmith` timeline（已有） |
| 4 ★ | town | 长老考校 → main_001 报到 | `role/npc/elder/elder_timeline.txt` 考校段（接 main_001） |
| 5 ★ | town/home | 坊市安顿 + main_002 灵田风波 | 商店现为 UI 样机未接购买（不进关键路径）；刷鼠事件 `map/home_pest.gd` |
| 6 ★ | town/xiaozhen | 坊市失窃事件（令牌伏笔） | `role/npc/storyteller/*_timeline.txt`（theft_rumor 段）+ `role/npc/disciple`（token_clue 段） |
| 7 ★ | town | 长老真相段赠聚气散×3 → 突破 → 收束 | `elder_timeline.txt`（`item_give` 赠 3 + 复命段置 `stage1_clear`） |

**timeline 红线**（本阶段实测沉淀）：变更本段 gate 状态的 flag/counter
必须写进**选项动作**（exec_response 内变更+goto 连续执行不经 stage 检查）；
行尾函数在行显示时执行，写在行尾会断掉本段选项链。静态 lint：
`test/check_stage1_dialog.gd`（goto 目标/flag 溯源/危险变更/链式 @）。

---

## 3. NPC 清单（10 位）→ 目录对照

| NPC | 目录（`role/npc/`） | 场景 | 本阶段改动 |
|---|---|---|---|
| 执事长老（玄诚） | `elder` | town | 考校接 main_001 / 真相段赠药接 main_004 / 复命收束置 stage1_clear |
| 萧老爷（萧远山） | `xiao_master` | xiaozhai | 修 4 处行函数断线坑（master_intro 等） |
| 萧夫人 | `xiao_lady` | xiaozhai | 寒暄+临行嘱托段 |
| 小翠 | `xiao_maid` | xiaozhai | beat 2 续写（祖产近况） |
| 孙半城 | `merchant` | town | 失窃闲谈一句 |
| 外门弟子（苏小棠） | `disciple` | town | beat 5 传送教学 + beat 6 token_clue 伏笔 |
| 白芷 | `herb_shop` | xiaozhen | 无改动（已有够用） |
| 铁牛 | `blacksmith` | xiaozhen | 无改动 |
| 说书先生（贾先生） | `storyteller` | xiaozhen | theft_rumor 失窃夜说书段 |
| 秦重 | `arena_master` | town | 修 arena_intro/arena_win 断线坑 |

NPC 注册契约：各目录 `*_timeline.txt`（对话）+ `schedule.json`（日程），
零代码接入；组件层 `rust/src/role/npc_schedule.rs` / `npc_chat.rs`。

---

## 4. 敌人与战斗位 → 实现位置

| 敌人 | 数值 | 实现位置 |
|---|---|---|
| 地灵鼠 | 攻3/血20/经40，快/近视 | `role/enemy/mouse.tscn`（slime 换皮） |
| 地灵鼠事件 | main_002 accepted 才刷 ×3，清剿推 1 步 | `map/home_pest.gd`（SPAWN_CELLS (7,5)/(12,8)/(9,10)，挂 home.tscn PestEvent 节点） |
| 草妖 slime | 攻5/血30/经60 | `map/dungeon/dungeon_map.gd` |
| 二层缩放怪 | 血×1.6 攻×1.5 经×2 | 同上 `_spawn_enemy(mult)` |
| 妖王（三层 boss） | 攻14/血320/防6/经500，首杀掉 `map_secret` | 同上 |

---

## 5. 任务链与解锁 → 实现位置

| 任务 | 流程 | 关键实现 |
|---|---|---|
| main_001 初入仙途 | 山门→报到→入住 | `state/task/task.json` |
| main_002 灵田风波 | 回 home→杀地灵鼠×3→回报 | steps 3→2；kill target_role=地灵鼠 |
| main_004 ★破关之资 | collect break_qizhe×3 → rank_up dou_zhe → talk 执事长老 | **新增**；奖励 exp1000/灵石100/token_sect |
| main_003 秘境探幽 | 秘境三层 | 前置改 main_004（第二阶段入口钩子） |

解锁链机制：`task_state.gd complete_task` 尾部 `_unlock_successors`
（自动解锁所有 precondition 指向刚完成者；accept 只接 available）。
flag 自动同步 `task_<id>_accepted/_done`。

**任务目录引导（2026-10-15 改造）**：任务面板默认页签「可接取」= 全量任务
目录——available 任务带「📍 找〈giver〉接取」指引、locked 任务作链路预览
（🔒 完成前置后开放），玩家不用盲找 NPC，看目录就知道下一步找谁对话；
任务随进度在 可接取→进行中→已完成 三页签间流转。HUD 任务横幅同步：
无进行中任务时显示「可接取：〈任务〉· 找〈giver〉接取」。
实现：`task_state.gd refresh_views`（status_group "avail" + hint 字段）、
`task_tabs.gml`、`mainhud.gd _on_task_views_changed`。

**突破联动**：`character_state.gd try_breakthrough` 成功 →
`_on_rank_entered`：置 `rank_<阶id>` flag；斗者阶时 `unlock_map(xiuxian_map_a)`
+ 推进 main_004 rank_up 步。地图门控：`state/map/map_state.gd`
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
| `test/check_stage1_flow.gd` | 状态层端到端 37 断言（接→做→交→突破→flag） |
| `test/check_stage1_dialog.gd` | 10 份 timeline 静态 lint + 动态 |
| `test/check_stage1_guide.gd` | Toast/Banner 组件单测 + mainhud 集成 |
| `test/check_stage1_enemy.gd` | 杀 3 鼠推进任务 |
| `test/probe_stage1_visual.gd` | 真实渲染器截图 Toast/Banner |
| `test/check_map_flow.gd` | 出生点 home + 萧宅建筑 + 传送流转（2026-10-15 跟进出生点） |
| `test/check_title_save.gd` | 存档选择 + 删除存档闭环 |

回归基线：test_runner **162/162**。

---

## 9. 阶段二候选入口（备忘）

- main_003 秘境探幽（map_secret 三层 + 妖王首杀掉残图）
- 幽竹林 map_b 开放
- 宗门大比（sect_tournament / arena 秦重）
- 孙半城失窃案展开（token_clue 伏笔接三幕剧本）
