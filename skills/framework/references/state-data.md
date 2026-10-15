# 数据与状态：GDCore、GdCoreData、GdBean

对应 Rust 源码：`rust/src/state/`（gdcore / coredata / bean / gjson / linklist）
可运行示例：`example/state/1.gd`、`example/state/progress.gd`

## GDCORE — 全局核心单例

```gdscript
# 直接用全局标识访问（无需 Engine.get_singleton）
GDCORE.set_save_id("2")        # 切换存档槽，路径变为 user://coredata_2.data
var slot = GDCORE.get_save_id()

# 全局节点注册表（跨场景访问节点，替代 autoload）
GDCORE.add_global_node("player", player)
var p = GDCORE.get_global_node("player")
GDCORE.remove_global_node("player")

# 每存档累计游玩时长（真实秒，跨开关应用累积）
var secs = GDCORE.get_play_time()   # 当前存档：历史落盘值 + 本次会话已进行时长
GDCORE.flush_play_time()            # 立即落盘（切存档/退出/每 30 秒自动触发，一般无需手动调）

# 删除存档（游戏内删档正规途径；严禁运行中手动删文件——见下方外部删除保护）
GDCORE.delete_save("2")   # 删文件 + 重置缓存；删当前档时 Bean 同步切到空 core 回默认值
```

> 游玩时长持久化在各存档 GdCoreData 的 `playtime;total`（真实秒）。
> 自动落盘时机：`set_save_id` 切档（并入旧档）、进程退出、每 30 秒兜底
> （崩退最多丢 30 秒）。GdWeatherManager 的 `auto_start_from_ticks` 默认
> 取此值作为初始时间基准。

### 外部删除保护 / 多实例告警（2026-10-12 事故沉淀）

- **30s 自动落盘 = 内存态全量写盘**：存档文件只是运行内存的周期性转储，
  **游戏运行中手动删存档文件会被"复活"**（内存旧态每 30s 写回，覆盖新进度，
  last-writer-wins）。删除存档必须：退出游戏后删文件，或调用
  `GDCORE.delete_save(id)`。
- **外部删除保护**：GdCoreData 加载时文件存在、之后被外部删除 → 该实例
  跳过写盘并 godot_warn 一次（"存档文件在会话期间被外部删除，跳过回写"）。
  新档（加载时文件不存在）不受影响。
- **多实例告警**：非 headless 实例启动后若发现 `user://.game_instance.lock`
  90 秒内有过更新 → godot_warn"另一个游戏实例可能正在运行"（锁文件每 30s
  随自动落盘刷新；两个实例同跑会互踩存档）。

### 存档槽位 / 新游戏与继续游戏

- 存档是否存在 = `FileAccess.file_exists("user://coredata_{id}.data")`（id 空 = `coredata.data`）。
- **切档语义已内建**：`set_save_id(id)` 对所有已注册 Bean 执行 `do_switch_core`——
  新档（空库）→ Bean 字段重置为 `initial_data` 默认值并写盘（= 新游戏）；
  旧档 → 从文件读回全部字段并广播 watch（= 继续游戏）。业务只需切 id 后进场景。
- **读非当前档摘要**（存档列表 UI）：临时 `GdCoreData.build(path, "{}", false, "init")`
  直查文件字段（如 `init;xiuxian_character;level`、`playtime;total`），不影响当前档缓存；
  不存在的文件 build 只给内存默认值，不会建文件。update 每次立即落盘。
- 范本：`example/demo/xiuxian/scenes/title/ui/save_modal.gd`（3 槽位弹窗，
  空档「新游戏」/ 有档「境界·天数·时长 + 继续游戏」）+ `check_title_save.gd`。

### 存档级游戏配置（玩法参数随档，勿放全局配置文件）

玩法参数（时间流速等）随存档走 GdBean（每个存档独立一份、update 立即落盘、
watch 响应变更），**不要**放 game_config.json 静态表（那是构建期全局定义）。

- 范本 `xiuxian/state/game_config/game_config_state.gd`（XiuGameConfigState，
  id `xiuxian_gameconfig`）：`day_length` = 一昼夜对应的现实秒数（默认 600）。
- 时间流速统一时间轴：天气/日历都按「游戏秒」流转，配置换算由日历管理器下发——
  `time_scale（游戏秒/现实秒）= 86400 / day_length`，调
  `WeatherManager.set_time_scale()`（#[var] 自动生成 get/set）；相位/降雨/晨雾
  基于游戏秒自动跟随。天数 = game_time / 86400 + 1，与流速无关。
- 运行时改流速 = `set_day_length(v)` → watch → 重下发，UI 可直接做滑条。
- 验收：`xiuxian/state/check_game_config.gd`（默认值/截断/落盘冷读/流速实测）。

### 测试存档隔离（铁律）

demo 无头测试**严禁**跑在玩家真实默认档（save_id ""）上——reset_demo/场景流
写入/标题入口测试都会整档覆盖玩家进度（与玩家进程 30s playtime flush 互相
last-writer-wins）。规则：

- 测试 `_run()` 开头（任何 `XxxState.ins()` 之前）调用
  `load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()`，
  退出前 `....restore()`（isolate 自愈：清残留测试档再切 "_test"）。
- **走标题 UI 进入游戏的测试**（dungeon_flow / scene_flow 等）必须点
  **空槽三**（Slot3Btn，save_id "3"）进入并清理 coredata_3.data——
  存档一 = 玩家真实活档，点它 = 在玩家进程实时写入的档上断言，必翻车。
- check_title_save（存档 UI 专项）自带零污染协议：槽一只读不写。
- 探针工具：`xiuxian/state/probe_save.gd`（解密查各档 level/day/stones/playtime）。

## GdCoreData — 核心数据引擎（Resource）

底层 GJson 路径查询式 JSON 存储，支持加密、持久化、变更订阅。

```gdscript
var data = GdCoreData.build(...)      # 构建数据引擎
data.initial(...)                     # 初始化

data.add_scope("bag", {"gold": 100})  # 添加作用域
data.value("gold", 0, "bag")          # 读（field, default, scope）
data.update("gold", 250, "bag")       # 写
data.has("gold", "bag")               # -> bool
data.change(...)                      # 带变更元信息的更新
data.watch("gold", func(): ...)       # 字段变更订阅 (field, cb, scope)
data.reload_data(json_str)            # 从 JSON 字符串重载
```

## GdBean — 数据绑定 Bean（推荐业务层用法）

GdBean 子类 + 静态单例模式，配合 GML UI 的 `data="bean:..."` 绑定（见 ui-gml.md）。
**属性变更后调用 `emit([keys])` 自动推送到订阅的 UI**，无需手动刷界面。

```gdscript
# state/progress.gd —— 业务进度数据
class_name SProgress extends GdBean

static func ins() -> SProgress:
    return GdBean.bean("state_progress", func(): return SProgress.new())

var times := 0
func get_update_times():
    update("times", times + 1, {}, false)   # (key, value, metas, force)
```

```gdscript
# 使用侧
var progress = SProgress.ins()
progress.watch("times", on_times_changed)     # 订阅变更
progress.get_update_times()                   # 修改即触发 watch
```

常用 API：

```gdscript
bean.update(key, value, metas, force=true)          # 单字段更新
bean.updates({"a": 1, "b": 2}, metas, force=true)   # 批量更新
bean.emit(PackedStringArray(["times"]), force=true) # 手动推送变更
bean.watch(key, callback)                           # 订阅
bean.unwatch(key, callback)                         # 退订（key/cb 与注册时不匹配静默忽略）
bean.watch_property(key, cb)                        # 属性订阅
bean.get_value_by_key(key)                          # 读
bean.flush(excludes)                                # 立即冲刷待推送变更
bean.reinit(excludes)                               # 重置（保留 excludes）
bean.to_dict(excludes)                              # 导出字典（存档用）
bean.set_scope("bag")                               # 绑定 CoreData 作用域
bean.bind_node_text("name", control)                # 直接绑定控件文本
bean.update_by_expression(expr)                     # 表达式更新
bean.switch_core(core)                              # 切换 CoreData 后端
```

**watch 死监听自动清理（框架行为）**：Bean 是进程级单例，生命周期长于场景——
UI 注册 watch 后随场景释放时，死回调残留在 Bean 的监听表里。框架在每次通知前
按【注册时快照的目标 ObjectID】判活并自动清理死监听（GdBean / GdState / GdEventBus
统一此机制）。**严禁改回用 `Callable::is_valid()` 探活**：GDScript self-lambda
（self 为 Node/Control 等非 RefCounted）在引擎侧持 self 裸指针，对象释放后探活
会解引用悬垂指针读出垃圾 ObjectID，ObjectDB 报 `Condition "slot >= slot_max"`
（engine object.h get_instance 越界断言，报错点在 Bean.update 的通知链上）。
生命周期短于 Bean 的 UI 可在 `_exit_tree` 里 `unwatch` 即时退订（可选，不调用
也会被自动清理）。

## UI 联动（典型闭环）

```gdscript
# 业务 Bean（驱动 GML 界面）
class_name SUIMain extends GdBean
static func ins():
    var res = GdBean.bean("scene_main", func(): return SUIMain.new())
    res.reinit([])
    return res

func add_equip():
    equip_data.append({"name": "铁剑", "desc": "+5 攻击"})
    emit(["equip_data"])          # GML 中 data="bean:scene_main:equip_data" 的列表自动刷新
```

## 其他

- `GJson`（gjson.rs）：GdCoreData 底层的 JSON 文档存储，GDScript 侧不直接使用。
- `GdDataLinkList`（linklist.rs）：基于字典的链表数据容器 `Dictionary<String, Array>`。
- 存档路径规则：`user://coredata_{save_id}.data`，save_id 为空时 `user://coredata.data`。
## 静态定义表管线（明文 .json 源 → 加密 .gjson 产物 + 静态查询类）

策划数值/目录类数据（等阶经验表、道具目录等）不放 GdBean：**定义是只读的**，
不需要持久化/变更通知。规范（范本 `example/demo/xiuxian/state/level/` 与 `item/`）：

1. **源文件**：策划/程序只维护明文 `xxx.json`（可 diff、可版本控制、直接编辑）。
   **必须声明管线标记**：顶层字段 `"pipeline": "gjson"`（opt-in——.json 是通用
   扩展名，运行时配置如 game_config.json 走明文，不可无差别转换；插件与
   regen 工具只转换带标记的文件）。
2. **加密产物**：编辑器插件（addons/gamecore/core.gd `_poll_json_files`）每秒轮询
   源 .json 的 mtime（启动 2 秒后先全扫一次），校验管线标记与可解析后经
   `GdJsonCodec.encrypt_text` 加密生成同名 `.gjson`（XOR 密钥在 Rust
   `GJson::ENCRYPT_KEY`，与运行时存档同一套，GDScript 侧永不接触密钥）。
   **.gjson 是生成物：勿手改、改动会被覆盖**。无头重建（与插件逻辑等价，
   产物 mtime 新于源时跳过）：`godot --headless --path . -s res://test/regen_gjson.gd`。
3. **静态查询类**：`class_name XxxTable extends RefCounted`，
   `static var _cache` + `load_data()`（`FileAccess.get_file_as_bytes` 读产物 →
   `GdJsonCodec.decrypt_to_text` 解密 → JSON.parse_string，失败 push_error 返回空）
   + 便捷查询 static func。**不注册 Bean、不写存档、不发变更**——运行数据由 Bean
   持有，定义表只回答「是什么/需要多少」。读取统一走 `_read_decrypted(gjson_path)`
   私有入口（范本两个 Table 各一份），产物缺失时 push_error 指引重跑 regen 工具。
4. **管线回归**：check_state.gd 5b 断言产物存在、确为密文（首字节非 '{'）、
   `GdJsonCodec.decrypt_to_text` 回环与明文源逐字节一致。
5. **Bean 数据源单一化**：Bean 的目录字段初始化/重置时从定义表派生
   （如 `XiuItemState.items = XiuItemTable.get_items()`），改源文件即改全游戏。
   **旧档升级**：items 快照随存档持久化，定义表后加条目在旧档里缺失——
   Bean `ins()` 时做缺失合并（`merge_new_items()`：确有补充才 update，
   遵守同值短路红线），防止背包显示裸 id。
6. 跨表引用在查询类里做校验（如 level 表的突破材料 id 必须存在于 item 表，
   缺失 push_error）。
7. 新建 `class_name` 后 headless `-s` 测试前需刷全局类缓存：
   `godot --headless --editor --quit-after 100`。
8. **导出发布注意**：.gjson 依赖运行时 FileAccess 直读（Table 类不走
   ResourceLoader.load，未注册依赖），导出预设默认不打包，需在导出预设
   「Filters to export non-resource files」加 `*.gjson`
   （源 .json 若不想随包分发，在过滤器排除）。
9. **定义表可按分类拆分多文件**：同目录多个 .json（各带 pipeline 标记）各自
   生成同名 .gjson，Table 类扫描目录全量合并为一张总表（范本
   `XiuItemTable._list_gjson_files`：DirAccess 扫 SOURCE_DIR 下 *.gjson，
   按文件名排序逐个解密合并 items，同 id 后者覆盖并 push_warning）。
   新增分类文件零代码接入；物理文件按系统分类，逻辑分类仍是条目 type 字段
   （get_items_by_type 跨文件聚合）。策划字段约定见各文件 desc。
10. **定义表不放进度状态字段**：unlock/已获flag 等「随玩家游玩变化」的数据
   属游戏进度，由 Bean 持有并持久化（范本 `XiuItemState.locked_items` +
   `unlock_item()`，初始锁定集为运行时默认值 `DEFAULT_LOCKED`）；定义表
   只回答「这个东西是什么/需要多少」。
11. **.gjson 编辑器打开方式**：Rust 侧 `state/gjson_loader.rs` 注册了
   ResourceFormatLoader（lib.rs Scene stage 注册/注销），双击 .gjson 以
   **GdJson 资源**打开（inspector，当前无属性展示；`get_data()/query()`
   运行时可用）；.gjson 不在 textfile_extensions（密文按文本打开会产生
   无效 UTF-8 错误日志）。与 .gml 相反：.gml 是明文源码必须按文本打开，
   刻意不注册 loader。

## 其他

- `GJson`（gjson.rs）：GdCoreData 底层的 JSON 文档存储，GDScript 侧不直接使用；
  其 XOR 加密仅作用于运行时存档 `user://coredata.data`。
- `GdJsonCodec`（json_codec.rs）：GJson 加密的 Godot 静态方法接口
  （`GdJsonCodec.encrypt_text / decrypt_to_text`），静态定义表管线专用。
- `GdJson` / `GdJsonLoader`（gjson_loader.rs）：.gjson 资源载体与
  ResourceFormatLoader——编辑器双击 .gjson 以 GdJson 资源打开；
  运行时也可 `ResourceLoader.load("res://...gjson")` → `get_data()/query()`。
- `GdDataLinkList`（linklist.rs）：基于字典的链表数据容器 `Dictionary<String, Array>`。
- 存档路径规则：`user://coredata_{save_id}.data`，save_id 为空时 `user://coredata.data`。

## demo 状态模式补充（仙途）

- **存档级游戏配置**（`XiuGameConfigState`，id `xiuxian_gameconfig`）：玩法参数
  随档走 Bean 勿放 game_config.json 静态表。字段：`day_length`（一昼夜现实秒，
  时间流速 = 86400/day_length 统一日历/天气时间轴）、`deco_regrow_secs`
  （deco 资源再生间隔秒，默认 120，DecoHarvest._ready 每图下发 + watch 重下发）。
- **游戏时长轴**（`XiuCalendarState.game_secs`，秒）：CalendarManager 每 0.5s
  按 WeatherManager.get_game_time() 轮询写入（天数 = game_secs/day_length+1）。
  **_ready 即接线恢复**：CalendarManager._wire 时 `_weather.set("start_time", 0.0)`
  + `set_total_time(Bean.game_secs)` 以存档绝对游戏秒恢复 WeatherManager 时间
  基准——不恢复的话 WeatherManager 每局从 start_time 重新累计：日历天数被静默
  重置、deco 冷却时间戳基准循环错乱（重开后挖掉的矿被误判到期立即长回）。
  恢复动作必须在场景装配（_ready）同步完成，严禁留给轮询首跑（期间任何
  set_total_time 都会被覆盖，产生时序竞态）。是 deco 采集冷却的**跨重启
  时钟基准**——冷却记录存「采集时刻的游戏秒」，重开后按差值换算已冷却时长。
- **地图采集状态**（`XiuMapState`，id `xiuxian_map`）：`harvested` 字典存已采集
  待再生格子（key = `(x<<32)|y` 字符串，**值 = 采集时刻游戏秒**，0.0 = 已弹出；
  旧格式 1.0 标记位会被视为早已到期 → 首次加载弹出，无害迁移）——deco_harvest
  进场按到期判定：未到期注入 GdDecoLayer（spawn_deco 跳过）+ 预推进再生时钟，
  到期弹出（矿自然长回）；采集 add_harvested、再生 remove_harvested。挖掉的矿
  重开游戏不会全部长回。整字典 duplicate 替换写法（同 XiuItemState.bag）。
- **玩家位置持久化**（`XiuPlayerState`，id `xiuxian_player`，独立玩家运行态
  Bean，见 state/player/player_state.gd）：`pos_alias` + `pos_x/pos_y` 存
  **精确世界坐标**（非格子坐标，保留格内亚像素）。player.gd _process 节流 2s
  写 `save_pos(alias, world)`（update 同值自动短路，只在移动后落盘）。恢复分
  两级：**主场景 map_restore.gd 先按 pos_alias 恢复「进哪张图」**（注册表命中
  则开该图，副本 alias 不在主场景注册表 → 回退 initial_map）；进图后
  player.gd `_resolve_spawn_position` 恢复图内位置——alias 匹配且落点格
  is_walkable 才原样落位，否则默认出生点（world_to_cell 为纯除法全局语义，
  地图在原点可直接用世界坐标校验）。位置属玩家运行态，独立成 Bean 不混入
  养成状态（XiuCharacterState）。

## 删档语义与写盘抑制（delete_save「复活」坑，2026-10-15）

**根因**：`delete_save` 删**当前激活档**后，`notify_beans_switch_core` 让各
Bean 向重建的空核回填默认值（`action=""` 对新核必写入）→ `GJson::update`
每次写都 `save_by_store()` → **文件同帧复活**。无 Bean 实例的守卫测试测不出。

**修复三件套**（`rust/src/state/coredata.rs` + `gdcore.rs`）：
1. `GdCoreData.suppress_save` 写盘抑制开关（`Rc<Cell<bool>>` 与 save_fn
   闭包共享；抑制期间内存照常更新、不落盘）；`delete_save` 删当前档时
   开启 + 终删双保险
2. `set_save_id` 重新激活该档时解除抑制（**含同 id 早退路径**——否则点
   「新游戏」进入后所有进度更新不落盘，进度全丢）
3. `flush_play_time` 幽灵建档守卫：文件不存在且 `session_secs()<1` 时跳过
   （标题页空转/删档后停留不再默默建档）

**语义红线**：
- 存档存在性判定以**文件是否存在**为准，勿用"内存缓存有无"判定
  （Bean 切核回填后缓存恒非空）
- 删档必须走 `GDCORE.delete_save`，严禁运行中手动删文件——30s 自动落盘
  会"复活"已删文件
- 需要在删当前档后立即做后续写盘的场景，先 `set_save_id` 重新激活再操作
