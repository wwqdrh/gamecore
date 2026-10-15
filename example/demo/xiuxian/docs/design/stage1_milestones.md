# 第一阶段里程碑实施记录（M1~M6）

> 对应设计：`story/04_stage1_design.md` §9；内容对照见 `stage1_content.md`。
> 每步走「改表/改码 → headless 测试 → 修」小循环，M1→M6 串行。

---

## M1 内容表

**做了**：
- `state/task/task.json`：新增 main_004「破关之资」（collect break_qizhe×3 /
  rank_up dou_zhe / talk 执事长老，奖 token_sect）；main_002 steps 3→2；
  main_003 前置改 main_004
- gjson 转换工具升级：`test/regen_gjson.gd` v2 —— **内容比对**（decrypt_to_text
  现存==源文本）判新鲜（mtime 不可靠：编辑器开着会实时转），支持 `-- 单文件`，
  重建后回读校验

**验收**：check_task_flow 全绿 + 表加载断言。

---

## M2 敌人与战斗位

**做了**：
- 地灵鼠：`role/enemy/mouse.tscn`（slime 换皮，血20/攻3/经40/棕色/快/近视）
- home 刷鼠事件：`map/home_pest.gd` + home.tscn 挂 PestEvent 节点
  （main_002 accepted 才刷 ×3，清剿推 1 步；0.5s 节流轮询非 Bean watch——
  装配期重入规避）
- 副本数值：`map/dungeon/dungeon_map.gd` slime 30/5/60；二层起楼层缩放
  血×1.6 攻×1.5 经×2；妖王 320/14/6（血 150 低于二层小怪 192 不成立，保留 320）

**验收**：`test/check_stage1_enemy.gd`（杀 3 鼠推进任务）+ check_dungeon_flow。
**教训**：改 tscn 数值必 grep 测试硬断言（check_enemy_exp 曾硬编码旧经验 20 连带
升级边界错）。

---

## M3 对话剧本

**做了**：
- `ui/dialog/dialog_box.gd` 新行命令：`task_advance` / `item_give` /
  `task_status`（参数 Variant 再 int() 转）
- 10 份 timeline 续写/修复：elder（考校接 main_001 / 真相段赠药接 main_004 /
  复命收束置 stage1_clear）、小翠、萧夫人、说书先生（theft_rumor）、
  外门弟子（token_clue）、孙半城、master/arena/maid
- **修 4 处既有断线坑**：master_intro/arena_intro/arena_win/maid_task 的
  选项从未显示——行函数（@set_flag 等）在行显示时执行，变更本段 gate 的
  flag 必须写进**选项动作**（exec_response 内变更+goto 连续执行不经 stage 检查）

**验收**：`test/check_stage1_dialog.gd`（静态 lint：goto 目标/flag 溯源/
CODE_FLAGS 危险变更/链式 @ + 动态）+ check_npc_greet / check_task_flow 跟进。

---

## M4 任务闭环

**做了**：
- `state/task/task_state.gd`：complete_task 尾部 `_unlock_successors`
  （解锁所有 precondition 指向刚完成者——否则 main_004 locked 致
  task_accept 静默失败）
- `state/character/character_state.gd`：try_breakthrough 尾接
  `_on_rank_entered`（按新阶置 `rank_<阶id>` flag；dou_zhe 时
  unlock_map(map_a) + 推进 main_004 rank_up 步）
- `state/map/map_state.gd`：`map_unlocked` 字典（持久化）+
  is_map_unlocked/unlock_map；主线四图 STARTER_MAPS 恒解锁
- `role/player/player.gd`：落格命中传送点先查目标图锁（rust 侧新增只读
  `get_portal_target_at` 供前置查询，不在 try_teleport 回调里反向调管理器——
  重入规避），未解锁 emit `s_portal_locked` 拦截

**验收**：`test/check_stage1_flow.gd`（37 断言状态层端到端）+ check_map_flow。

---

## M5 引导 + 演出（rust 侧 gml 框架组件）

**用户要求**：引导和演出 UI 在 rust 侧 gml 框架实现，gd 侧薄接。

**做了**：
- `<Toast>` 提示条：`rust/src/ui/ui_toast.rs`（PanelContainer，show_toast/
  show_toast_secs 队列 + 相位机 0空闲/1入/2停/3出 + s_toast_shown/s_toast_done +
  is_busy/clear_toasts；grow_horizontal 走 base.set 属性——gdext 0.5 无
  set_grow_horizontal 方法）
- `<Banner>` 演出字幅：`rust/src/ui/ui_banner.rs`（Control，play_banner 队列
  上限 4 无缝接播，淡入(1.25→1.0 缩放)→停留→淡出，dim_bg，播放期 STOP
  结束回 IGNORE，s_banner_finished）
- 组件注册：`rust/src/ui/mod.rs + builder.rs`（instantiate_control /
  鼠标穿透白名单 / apply_attribute 属性段；derive GodotClass 自动注册无需清单）
- 引导控制器：`scenes/main/ui/mainhud/mainhud_guide.gml/.gd`（挂 mainhud.gml
  根部最后=最上层）：beat0 教学 4 连（tutorial_shown 门控）、突破演出按
  **get_realm_title() 变化**判定（按 level 变化会 1→10 每升段连灌 9 条）、
  通关字幅（stage1_clear）、s_portal_locked→Toast

**验收**：`test/check_stage1_guide.gd`（组件单测+mainhud 集成）+
`test/probe_stage1_visual.gd`（真实渲染器截图）。
**坑**：测试 lambda 按值捕获标量（counter+=1 改副本），计数断言用 Array.append。

---

## M6 全链通关验收

**做了**：
- Banner 播放队列（上限 4 接播）
- guide 突破判定改等阶头衔（见上）
- `test/check_stage1_clear.gd`：真实 index.tscn 全链通关冒烟
  （教学→任务→刷鼠→突破演出→复命→通关字幅→青草岭切换）
- **无头帧率红线**：await process_frame 帧数≠墙钟时间（900 帧 <1s）——
  等真实时间的轮询必须 `Time.get_ticks_msec()` 墙钟时限
  （budget_ms = max_frames/60*1000）

**验收**：check_stage1_clear ALL PASS；runner 162/162（多轮）。

---

## M6.5 删除存档 + 响应式显隐（2026-10-15 追加）

**做了**：
- 框架：`rust/src/ui/ui_reactive.rs` —— GML `show_if`/`hide_if` 条件显隐 +
  `GdUIReactive.set_var/get_var/apply_conditions`（详见
  `skills/framework/references/ui-gml.md`（响应式显隐/Toast/Banner）与 `state-data.md`（删档语义））
- `scenes/title/ui/save_modal.gml/.gd`：有档槽「继续游戏+删除存档」，
  两步确认（CONFIRM_MS=3000 墙钟），删后 set_var(false) 自动回「新游戏」
- **框架级修复**：delete_save 删当前档被 Bean 默认值回填同帧复活——
  GdCoreData suppress_save 开关 + set_save_id 重新激活解除 +
  flush_play_time 幽灵建档守卫（详见 skills/framework/references/state-data.md「删档语义」）

**验收**：check_title_save.gd 31 断言 + probe_title_delbtn.gd 截图。

## M6.6 出生点修正（2026-10-15 追加）

设计 beat 0 出生点=home 农田，实现曾配 xiaozhai。修正：
`scenes/main/index.tscn` MapManager `initial_map="xiuxian_home"`、
`initial_spawn=Vector2i(9,6)`（农田区可行走格）；`map_modal.gd` 兜底同步；
`check_map_flow.gd`/`check_npc_flow.gd` 跟进（npc 测试显式切图到萧宅；
home 地图根是 GdDigLayer 非 GdQuickMap，类型断言放宽）。runner 162/162。
