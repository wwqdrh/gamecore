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
```

> 游玩时长持久化在各存档 GdCoreData 的 `playtime;total`（真实秒）。
> 自动落盘时机：`set_save_id` 切档（并入旧档）、进程退出、每 30 秒兜底
> （崩退最多丢 30 秒）。GdWeatherManager 的 `auto_start_from_ticks` 默认
> 取此值作为初始时间基准。

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
6. 跨表引用在查询类里做校验（如 level 表的突破材料 id 必须存在于 item 表，
   缺失 push_error）。
7. 新建 `class_name` 后 headless `-s` 测试前需刷全局类缓存：
   `godot --headless --editor --quit-after 100`。
8. **导出发布注意**：.gjson 依赖运行时 FileAccess 直读（Table 类不走
   ResourceLoader.load，未注册依赖），导出预设默认不打包，需在导出预设
   「Filters to export non-resource files」加 `*.gjson`
   （源 .json 若不想随包分发，在过滤器排除）。
9. **定义表不放进度状态字段**：unlock/已获flag 等「随玩家游玩变化」的数据
   属游戏进度，由 Bean 持有并持久化（范本 `XiuItemState.locked_items` +
   `unlock_item()`，初始锁定集为运行时默认值 `DEFAULT_LOCKED`）；定义表
   只回答「这个东西是什么/需要多少」。
10. **.gjson 编辑器打开方式**：Rust 侧 `state/gjson_loader.rs` 注册了
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
