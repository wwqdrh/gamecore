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
```

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
