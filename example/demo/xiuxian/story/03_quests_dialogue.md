# 03 任务与对话

支线、任务表与关键对话脚本。对话均用项目 GdDialogue timeline 语法
（`[stage]` 舞台、`(角色名)` 说话人、`- 选项@动作`、`@函数:参数`、
`:goto:stage`），可直接落地为各 NPC 的 `*_timeline.txt`。

## 支线剧情

| 支线 | 发布人 | 触发 | 简述 | 与主线关系 |
|---|---|---|---|---|
| 寻药换丹 | 孙半城 | 坊市对话 | 帮万宝斋在青草岭采灵芝血参，换修为丹 | 讲坊市货源与散修生态 |
| 外门比试 | 苏小棠 | 东街对话 | 击败三只草妖证明外门实力，赢青锋剑 | 深化挚友，镜像主角缺口 |
| 残图之谜 | 玄诚长老 | 持 `map_secret` 对话 | 长老鉴定残图，讲百年封印旧事 | 世界观补完，指引 map_b |

## 任务表

| ID | 类型 | 发布人/触发 | 目标 | 流程 | 奖励 | 叙事作用 |
|---|---|---|---|---|---|---|
| `main_p1` | 主线 | Beat1 长老对话 | 突破斗者 | ① 领 `break_qizhe` ② 坊市购辅材 ③ 服药冲关 | 解锁第一幕后续对话 flag | 打破日常 |
| `main_p2` | 主线 | Beat4 长老对话 | 青草岭立威 | ① 击杀 slime×5 ② 服 `break_doushi` 晋斗师 | 开放 map_a | 承幕入口 |
| `main_p3` | 主线 | Beat6 灵狐战后 | 取回残图 | ① 幽竹林缘 fox 精英战 ② 获 `map_secret` ③ 突破大斗师 | 开放 map_b 传送 | 中点前铺垫 |
| `main_p4` | 主线 | Beat8 东街事件 | 落霞秘境救人 | ① 持图入 map_b ② 破 golem×3 ③ 服 `break_douwang` 晋斗王 | 进入最终战 | 最低点→决断 |
| `main_p5` | 主线 | Beat10 最终战 | 守府决战 | ① 交图对话 ② golem 守府战 ③ 诀别对话 | `pill_break`×3 + 执事令 flag | 三幕收束 |
| `side_herb` | 支线 | 孙半城 | 采药换丹 | ① map_a 采 `herb_lingzhi`/`herb_xueshen` ② 回坊市交货 | `pill_exp_s`×2 | 坊市生态 |
| `side_spar` | 支线 | 苏小棠 | 外门比试 | ① 击杀 slime×3 ② 回东街复命 | `sword_qingfeng` | 挚友线 |
| `side_map` | 支线 | 玄诚长老（持图） | 残图鉴定 | ① 对话听旧事 ② 获得 map_b 背景说明 | flag `lore_seal` | 世界观补完 |

需新增系统清单：无（全部可用现有对话触发器 + 等阶经验 + 道具表实现；
「拾取手札」可先用固定对话触发位代替道具化，后续再提案文档收集品系统）。

## 关键对话脚本

### Beat1+4：执事长老（续写 `elder_timeline.txt`）

```text
[main_p1_start]
(执事长老,云舟)
你入门半年，斗之气停在九级——四十天了。
@set_flag:main_p1_ready
- 弟子不才，愿听长老教训。@goto:main_p1_gift
- 是宗门冷眼，不是弟子不努力。@goto:main_p1_grit

[main_p1_gift]
(执事长老)
嗯，肯认短处，比嘴硬的强。
聚气散一颗。缺的不是修为，是心气——去坊市凑齐辅材，把它服下去。
@set_counter:main_step:1
:goto:main_p1_end

[main_p1_grit]
(执事长老)
这话留着对你自己说。心气要向外使，不是向内怨。
聚气散一颗。凑齐辅材服下去，用突破堵住他们的嘴。
@set_counter:main_step:1
:goto:main_p1_end

[main_p1_end]
(执事长老)
突破了再来见我。有旧事，须当面说。
```

```text
[main_p4_start]
(执事长老,云舟)
江湖人称那一位「客卿供奉」，入宗三年，无人知其来路。
你父亲十年前追的，就是这枚棋子。
@set_flag:main_p4_ready
- 我去落霞秘境。@goto:main_p4_send
- 弟子修为尚浅……@goto:main_p4_steady

[main_p4_send]
(执事长老)
带上残图。记住，秘境认血，钥匙在你身上——这话只说一遍。
:update_counter:main_step:4
:goto:main_p4_end

[main_p4_steady]
(执事长老)
浅不浅，打过才知道。石傀不会因你年轻而手下留情。
:update_counter:main_step:4
:goto:main_p4_end
```

### Beat3：苏小棠（东街夜，`disciple_timeline.txt` 新增段）

```text
[main_p3_night]
(外门弟子,云舟)
出大事了！万宝斋的库房被翻了个底朝天，丢的却是宗门旧档——
你猜管档的是谁？执事堂！你爹当年的差事！
@set_flag:met_token
- 我捡到一枚玄色令牌。@goto:token_show
- 先别声张。@goto:token_quiet

[token_show]
(外门弟子)
……这纹路邪性。走，找玄诚长老，这事瞒不住。
:update_counter:main_step:3
:goto:token_end

[token_quiet]
(外门弟子)
你倒是沉得住气。行，那更得找长老——他的脸色比你这张脸好看不了。
:update_counter:main_step:3
:goto:token_end

[token_end]
(外门弟子)
对了，若真要动身，把我的剑也带上。外门弟子的剑，不比内门的差。
```

### Beat10：墨渊（遗府前，最终战引导）

```text
[final_deal]
(墨渊,云舟)
云家的小辈。图留下，人你带走——本座只要遗府之物，与你们父子无关。
- 图可以给，人你必须放。@goto:final_give
- 做梦。@goto:final_fight

[final_give]
(墨渊)
爽快。可惜——封印认血，你交出的本就是空壳。
@set_counter:main_step:5
:goto:final_boss

[final_fight]
(墨渊)
硬骨头。本座最厌硬骨头，好在石傀没有牙齿方面的顾虑。
@set_counter:main_step:5
:goto:final_boss

[final_boss]
(墨渊)
血祭既启，遗府自开。拦在本座面前的，都算祭品。
```

### Beat11：诀别（收束）

```text
[final_farewell]
(云澈,云舟)
舟儿。阵拖了十年，为父的力气，也只剩这最后一封了。
(云舟)
爹——跟我回去！长老有办法，一定有……
(云澈)
封印重合的那一刻，为父很骄傲。不是为了你赢，是你先去牵小棠的手。
拿好这执事令。以后轮到你，站在山门里替别人撑伞了。
@set_flag:story_clear
```

## 文案规范

- 命名/称谓/禁用见 `README.md` 摘要；对话脚本同规。
- timeline 书写约定：每段首行注释用途；选项 ≤ 3；分支必须
  `:goto:` 收回到主线；`@set_flag`/`@set_counter` 只用于
  任务进度，不复用作其它逻辑，避免 flag 污染。
