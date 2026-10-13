# 地图系统：双网格地形、噪声生成、寻路、地图管理

对应 Rust 源码：`rust/src/map/`（dual_grid / gd_map_basic / quick_map / gd_map_manager / gd_map_marker）
可运行示例：`example/map/map_demo.gd`、`example/map/basic.gd`、
`example/demo/xiuxian/check_map_flow.gd`（地图管理器端到端）

## GdQuickMap — 快速地图生成器（Node2D）

运行时创建 + 属性注入 + 生成，适合程序化地图：

```gdscript
var map = GdQuickMap.new()
map.width = 30
map.height = 18
map.cell_size = 32
map.seed_value = 20260928

# 地形层：三个数组的下标一一对应
map.terrain_names = PackedStringArray(["sand", "grass", "forest", "mountain", "water"])
map.terrain_thresholds = PackedFloat64Array([0.2, 0.4, 0.6, 0.8])   # 噪声阈值切分
map.terrain_dualgrid_textures = PackedStringArray([
    "",                                       # sand：空串 = 不渲染
    "res://assets/tiles/tileset_grass.png",
    "", "",
    "res://assets/tiles/tileset_water.png",
])
map.terrain_shaders = PackedStringArray(["", "", "", "", "water_flow"])  # 内置水体 shader
map.terrain_shader_textures = PackedStringArray(["", "res://assets/tex/grass_detail.png", "", "", ""])  # 地形贴图
add_child(map)
map.generate(MAP_SEED)
```

内置 shader 名直接写进 `terrain_shaders`（如 `water_flow`），编译在 Rust 二进制内，无需 .gdshader 文件。

**地形 shader 贴图（`terrain_shader_textures`，按下标对应 terrain_names）**：
给该地形的 shader 提供 `tex0` uniform（世界坐标 1:1 重复采样，相邻格/过渡片
纹丝对接）。同时自动构建逐格遮罩 `mask_tex`（每世界格 1 像素）——双网格图层的
累积垫底/过渡片混有其他地形像素，必须按格遮罩，只有本地形格子生效。
`terrain_shaders` 对应项为空时自动挂内置 `terrain_tex`；自定义 shader 按内置
uniform 契约（`mask_tex/map_cells/cell_px/tex0..2/tex_size0..2`）声明同名参数即可。
GdDigLayer 同样支持该字段（单层结构，材质挂根图层，最多 3 个贴图地形按
mask r/g/b 通道区分；挖地/恢复后遮罩自动同步，调试读口 `get_shader_mask_at`）。
编辑器改值后需重开场景或调 `regenerate()`/`rebuild()` 生效。

## 连通性保证（connected_terrain）

噪声阈值切分的地图会出现被山/水隔断的孤岛可行走区（看着能走却到不了）。设置
`connected_terrain = "grass"`（任意 terrain_names 中的地形名）后，generate/regenerate
自动把**全部可通行区域挖通成一片**（同一 seed 结果确定）：

- 算法：可通行格连通分量标记 → 取「主地形格最多」的分量为主分量 →
  其余分量按大小降序，0-1 BFS（可通行代价 0 / 阻挡代价 1）找最短桥 →
  桥上阻挡格改成主地形（视觉上即打通的通道）→ 分量并入主集合，直到全连通
- 渲染前执行，桥格贴图/双网格过渡自然一致；传送点/NPC 吸附照常工作
- 手动补挖：`map.ensure_connected()`（有改动时重建图层并重绘）

```gdscript
map.connected_terrain = "grass"   # tscn 里直接设属性即可
map.generate(MAP_SEED)
```

## 手绘建筑块（paint_rects）

噪声地形只适合野外；宅院/小镇这类需要确定布局的地图，用 `paint_rects` 在
噪声生成后叠加手绘地形矩形（典型用法：**山地地形当建筑**——矩形是屋子、
矩形间缝隙是门，山地的不可行走 + 高度场（子弹撞毁）语义自动继承）：

```ini
# tscn 配置（GdQuickMap 节点属性）
paint_terrain = "mountain"
paint_rects = Array[Rect2i]([Rect2i(7, 2, 6, 3), Rect2i(4, 5, 2, 4)])
```

- 应用时机：噪声填充**之后**、连通性挖洞**之前**——建筑墙不会被桥格打通
- 想要"纯手绘布局"：把 terrain_thresholds 拉满（如 2 地形填 `(1.01)` →
  全图 grass，无水无噪声山），再只用 paint_rects 造型
- 越界部分自动裁剪；paint_terrain 不在 terrain_names 中时整批跳过并告警
- 范本：`example/demo/xiuxian/map/xiaozhai.tscn`（萧宅：正房/东西厢/南墙带门）、
  `xiaozhen.tscn`（青石镇：北街三铺面 + 南街三宅院）


## 寻路与通行判定

```gdscript
map.set_blocked_terrain_names(PackedStringArray(["water", "mountain"]))  # 标记阻挡地形

var walkable: bool = map.is_walkable(cell)
var cell: Vector2i = map.world_to_cell(world_pos)
var center: Vector2 = map.cell_to_center(cell)
var path: PackedVector2Array = map.find_path(from_cell, target_cell)  # BFS

map.clear_map()   # 清空重画
```

## 高度场（terrain_provider 协议）

GdQuickMap 支持每地形海拔高度（高度场是一等地图数据，不做「山体」特判）：

```gdscript
# terrain_heights 与 terrain_names 下标对应（缺省 0；名为 mountain 的地形内置海拔 3）
map.terrain_heights = PackedInt32Array([0, 3])   # grass=0, mountain=3

map.get_cell_height(cell)             # 格子海拔（越界 0）
map.get_height_at_world(world_pos)    # 世界坐标海拔（协议入口）
```

**协议**：GdQuickMap ready 时自动加入 `terrain_provider` 分组。场景树内该分组中
实现 `get_height_at_world(world) -> i32` 的节点即为高度场数据源——投射物
（GdBullet 撞山检测）、飞行单位越障、视线判断等消费方 duck-type 查询，
与地图实现零耦合。判定语义：`地形海拔 > 单位飞行海拔` → 撞毁/阻挡
（海拔相等不挡，草地 0 不挡地面单位）。

## 配合 GdRoleMover 网格移动

```gdscript
# 玩家走格子 + 自动寻路
player.move_mode = 2                      # MODE_GRID
player.grid_cell_size = 32
player.grid_map_path = NodePath("../QuickMap")

var path: PackedVector2Array = map.find_path(from_cell, to_cell)
player.set_grid_path(path)
player.s_grid_path_finished.connect(func(): print("到达"))
```

## GdMapBasic — 双网格地图节点（预置于场景）

地形过渡渲染为主，适合 tscn 内预配置：

```gdscript
# 节点已在场景中（$Map）
$Map.generate_map_with_seed(randi())
$Map.clear_map()
```

双网格核心（`dual_grid.rs`）：世界网格按地形层存储坐标集合，同一坐标可属于多个地形，
渲染时自动计算地形边缘过渡瓦片——业务层只管"哪些格子是什么地形"，过渡由框架处理。

## 灯光示例

`example/map/rotating_light.gd` 演示了地图上挂旋转 PointLight2D 的做法，可参考。

## GdMapManager — 地图管理器（Node）

地图场景的注册、加载、切换与传送点调度。地图场景根须为 Node2D（推荐直接 GdQuickMap 作根）：

```gdscript
# 场景装配（tscn 上显式配置，与 entry_scene 同风格）：
# [node name="MapManager" type="GdMapManager" parent="."]
# map_paths = PackedStringArray("xiuxian_map_a=res://.../map_a.tscn", ...)
# initial_map = "xiuxian_map_a"     # ready 自动加载（空则不自动）
# initial_spawn = Vector2i(4, 9)

MapManager.open_map("xiuxian_map_b", Vector2i(3, 9))  # 切换地图（旧图释放，新图入 MapLayer）
MapManager.get_current_map()      # 当前地图实例（GdQuickMap/Node2D）
MapManager.get_current_alias()    # 当前别名
MapManager.get_spawn_cell()       # 出生格（open_map 时记录）
MapManager.get_spawn_point()      # 出生格中心世界坐标
MapManager.get_markers()          # 当前地图上的传送点数组
```

- 跨组件查找：MapManager 自动注册进所在场景根组件表（`GdSceneRoot.get_component("MapManager")`），
  业务禁止 get_parent 链遍历 / find_child 查找，规范见 scene-manager.md「组件注册表」

- 信号：`s_map_changed(alias)` / `s_teleport_triggered(from, to, target_cell)`
- 传送点/出生格吸附：open_map 后自动把落在不可通行地形上的 GdMapMarker 与 spawn_cell
  BFS 吸附到最近可行走格（噪声图种子固定但格子随机，手填坐标不可靠）
- `click_teleport = true`（默认）时，未被 UI 消费的左键点击落在传送点格内直接触发传送
- 注册到 GDCORE 全局节点表（manager_id 默认 "map"，`GDCORE.get_global_node` 可取）

## GdMapMarker — 地图传送点/高亮标记（Node2D，tool）

放在地图场景内（建议 GdQuickMap 子节点），指定格子绘制呼吸亮框（填充+光晕环+描边+四角高亮+中心光点，可配 label 文字）：

```gdscript
# tscn 节点属性：
# cell = Vector2i(26, 9)        # ready 自动定位到格心（父级=地图原点约定）
# cell_size = 32                # 与地图 cell_size 一致
# target_alias = "xiuxian_map_b"  # 空 = 仅标记不传送
# target_cell = Vector2i(3, 9)
# label = "前往幽竹林"

# 后续人物网格走动到达格子时：每步调用
MapManager.try_teleport(角色全局坐标)  # 命中传送点格子即触发切换，返回是否传送
```

- ready 自动加入 `__gdmap_marker` 分组（GdMapManager 经此收集）
- `contains_world_point(world_pos)` 命中判定 / `get_center()` 格心全局坐标 / `place_at(cell)` 重定位
- 信号 `s_triggered`（管理器命中时发出）

## GdDecoLayer — 地图装饰层/可采集物（Node2D）

挂在 GdQuickMap 地图场景下，按配置在指定地形上随机分布装饰/可采集物品（范本：
xiaozhai.tscn / xiaozhen.tscn 的 DecoLayer + deco_harvest.gd）。配置为**并行数组**
（下标一一对应，以 deco_item_ids 为主长度，风格同 GdQuickMap 地形数组），tscn 手写零门槛：

```ini
# tscn 节点属性（GdDecoLayer）：
deco_terrains = PackedStringArray("grass,forest", "forest")  # 目标地形（逗号分隔多个；空=所有可行走地形）
deco_item_ids = PackedStringArray("ore_coldiron", "ore_amethyst")  # 物品 id（回调/背包联动标识）
deco_textures = PackedStringArray("", "")       # 贴图路径（留空用 deco_colors 色块绘制）
deco_colors = PackedColorArray(...)             # 回退颜色（菱形色块）
deco_frequencies = PackedFloat64Array(0.02, 0.012)  # 单格生成概率 0..1
deco_mineable = PackedInt32Array(1, 1)          # 1 = 可开采
deco_hits = PackedInt32Array(3, 3)              # 采集所需碰撞次数（默认 3）
deco_callbacks = PackedStringArray("on_ore_harvested", "...")  # 采集回调函数名
callback_target_path = NodePath("../DecoHarvest")  # 回调宿主（空 = 父节点）
deco_seed = 0                                   # 0 = 取父地图 get_seed()（与地形同源确定）
deco_regrow_interval = 0.0                      # 资源再生间隔秒（0 = 禁用；demo 默认 120 随档下发）
```

- **可开采协议**（复用战斗组件，零新语义）：矿石 = Node2D 根 + GdHealth
  （max_health = deco_hits，无敌帧 0）+ GdHurtbox（layer5=16 可采集层）。
  采矿工具（GdHitbox/GdMelee，target_mask=16）每次命中 take_damage(1)，
  血量归零即「碰撞满 N 次」→ 发 `s_harvested(item_id, node)` → 按名调用
  `callback_target.<deco_callbacks 名>(item_id, node)` → 矿石移除
- **碰撞层规划**：layer2(2)=玩家受击盒 / layer3(4)=敌受击盒 / **layer5(16)=可采集物受击盒**
- 采集回调在 gd 侧实现（框架零业务感知）：`func on_ore_harvested(item_id: String, node: Node)`
  ——demo 里 `XiuItemState.ins().add_item(item_id, 1)` 入包，背包 UI watch bag_views 自动刷新
- 时序：子节点 ready 先于父地图生成噪声，ready 自动 call_deferred spawn_deco；
  spawn_deco 幂等（先清旧再铺新，同种子同结果），可换种子/配置后手动重铺。
  **清旧为立即 free，严禁在采集回调内调用 spawn_deco**（信号栈中释放节点不安全）
- 随机性：每条目独立 RNG（base_seed + 条目下标 × 7919 混入），格内散布偏移由
  格子哈希决定——同一地图 + 同一种子分布完全确定（可复现/可测试）
- `get_spawned_count()` 最近一次铺设数量（测试断言用）

### 资源再生（挖掉不复活 + 定时随机补种）

```gdscript
layer.deco_regrow_interval = 120.0            # 秒；0 = 禁用（宿主每图 _ready 下发存档级配置）
layer.set_harvested_cells(PackedInt64Array)   # 进场注入冷却格子（key = (x<<32)|y），随档持久化
layer.get_harvested_cells() -> PackedInt64Array
layer.set_regrow_elapsed(secs: float)         # 预推进再生时钟（秒）——恢复存档时传入
                                              # 已冷却时长，重启/换图不再重置再生节拍
layer.s_regrowned(item_id: String, cell_key: int)  # 补种成功信号——cell_key = 被消费的冷却记录
```

- 采集后格子进 **harvested FIFO 队列**：spawn_deco（重开游戏进场重铺）跳过
  这些格——矿不会"重开就全部长回"；`spawn_node` 在矿石上挂 `deco_cell` /
  `deco_entry` meta（回调宿主读取入库）
- 每 `deco_regrow_interval` 秒补种一批：缺额条目（当前存活 < 初始铺设数）各
  随机挑一个可生长空格长一个（**生长位置不固定**，非确定性 RNG）；成功后消费
  一条冷却记录——长回原格弹原记录、长到新格弹最早一条（缺口记录数守恒 =
  初始数 − 存活数）；**冷却中的格子允许被补种选中**（freq 高的图可能没有其他
  空位，排除会死锁）
- **s_regrowned 发的是被消费的记录 key**（新格 key 从未持久化，发它宿主会对
  不上）——宿主 `remove_harvested(cell_key)` 同步持久化集合
- demo 侧（deco_harvest.gd 挂图内，每图 _ready 自动接线）：进场
  `XiuMapState.get_harvested()`（值 = 采集时刻游戏秒）到期判定——到期的弹出
  （矿本次铺设自然长回）、未到期的按采集时间升序 `set_harvested_cells` 注入 +
  `set_regrow_elapsed(最早格已冷却秒)` 预推进；`XiuGameConfigState.deco_regrow_secs`
  （默认 120s）下发 + watch 配置重下发；采集 `add_harvested(key, 游戏秒)`、
  再生 `remove_harvested`。
  **注意铺设语义**：spawn_deco 逐格消耗 RNG 且跳过冷却格不消耗 → 注入任一格
  会使同条目后续铺设位置漂移（生长位置本就不固定，矿数守恒由 regrow 补足目标
  = 本次实际铺设数保证）。

## GdDigLayer — 挖地地形层（TileMapLayer，tool，可作地图根）

继承 TileMapLayer 的单层双网格地形系统（参考 example/map/Jess.tscn 的 TileMapDual
语义，范本：demo/xiuxian/map/home.tscn）：显示网格偏移半格，每个显示格按其覆盖的
4 个世界格角点地形组合查 4x4 过渡图集（16 组合，图集把铺底地形背景烘进贴图，
如 grass_sand.png = 沙底 + 草形状）——与 GdQuickMap 多层堆叠不同，单层即可渲染。
定位**手工布置的小地图**（home/室内/农田），核心能力 `dig()`：把某格地形挖成
目标地形，局部刷新受影响显示格并通知宿主。

```ini
# tscn 节点属性（GdDigLayer，可直接作为地图场景根节点）：
width = 20; height = 14; cell_size = 32        # 世界格数 / 格尺寸
atlas_path = "res://.../grass_sand.png"        # 4x4 过渡图集（空 = 程序化双色图集调试）
terrain_names = PackedStringArray("sand", "grass")  # 下标0=铺底(填满整图) 下标1=形状(图集圆角)
dig_targets = PackedStringArray("", "sand")    # 与 terrain_names 平行：挖后地形名（空=不可挖）
init_rects = Array[Rect2i]([Rect2i(1,1,12,7)]) # 初始地形补丁（与 init_terrains 平行，后覆盖先）
init_terrains = PackedStringArray("grass")
dig_rects = Array[Rect2i]([Rect2i(1,1,12,7), Rect2i(15,3,4,9), Rect2i(3,10,9,3)])  # 可挖区域限制（与草地补丁对齐；空 = 全图按 dig_targets 允许）
blocked_terrains = PackedStringArray()         # 不可通行地形名
```

- **API**：`dig(cell) -> bool`（成功发 `s_dug(cell, from, to)` +
  `s_terrain_changed`）/ `is_diggable(cell)` / `get_terrain_name_at(cell)` /
  `set_terrain_name_at(cell, name)` / `is_walkable(cell)`
- **挖地预览光标**：导出 `cursor_texture`（Texture2D，如 cursor.svg）→ ready 时
  创建 DigCursor 精灵子节点（相对 z=1 盖瓦片、初始隐藏）；宿主经
  `set_cursor_cell(cell)` 驱动（**只在 is_diggable 的格子显示**——rects 外/
  不可挖地形自动隐藏，所见即所锄）、`clear_cursor()` 隐藏。光标格换算与实际
  dig 共用一套（挥击方向前方一格），锄具每帧同步（player.gd _update_dig_cursor）
- **默认地形改法**：想整图默认某地形（如草地）不必换铺底下标/重做图集——
  init_rects 加一条全图矩形即可（铺底被盖住；图集自带双向过渡片，沙洞在
  草地上渲染正常）。可交互区域（dig_rects）与视觉区域须对齐，玩家按所见即所点
- **地图根协议**（MapManager/player 鸭子类型）：`is_walkable` /
  `world_to_cell`（纯除法全局语义）/ `cell_to_center` / `get_map_width` /
  `get_map_height` / `get_cell_size_px`——已齐备，home.tscn 根节点直接是 GdDigLayer
- **持久化**：宿主 `get_terrain_grid()` / `set_terrain_grid(arr)` 整网格
  dump/restore（行主序地形下标；小地图整存最简且覆盖任意变化路径）；
  编辑器调参后 `rebuild()` 重出图（清运行期变化）。
  demo 范式（dig_save.gd，范本 home.tscn 的 DigSave 节点）：图内宿主节点
  `@export map_alias` → 进场 deferred 读 Bean（**子 ready 先于父地图 ready，
  恢复必须 call_deferred 等网格初始化完**）`set_terrain_grid` 覆盖初始补丁；
  `s_terrain_changed` → 整网格写回 Bean（update 现值比对天然去重）。
  验收：check_hoe_till.gd E 节（新实例进场自动恢复锄过的地形）
- 时序：ready 时按导出配置初始化网格 + 装配 tileset + 全量铺设（编辑器
  WYSIWYG）；世界格变化只重画受影响 4 个显示格（O(1) 局部刷新）
- **工具联动范式（锄地，范本 demo/xiuxian 4 号槽玄铁锄）**：地形不是 Area2D，
  不能走 GdMelee 判定盒——装备切 till 模式时 `melee.target_mask = 0`（判定盒
  扫描不到任何目标），锄挖逻辑挂 `melee.s_swing(dir)` 信号：目标格 =
  `map.world_to_cell(player.global_position + dir.normalized() * cell_size)`
  （挥击方向前方一格），`map.has_method("dig")` 守卫后调 `dig(cell)`——
  无 dig 协议的地图（GdQuickMap 等）静默跳过。对照：挖矿镐 mine 模式走
  判定盒打 deco 受击盒（矿石是 Area2D），两种"挖"的目标结构不同，勿混用。
  验收：check_hoe_till.gd（挥击→dig 全链 + rects 外拒绝 + 单格精度）。

## 场景分类与随机副本（demo 范本：xiuxian 秘境）

场景分两类：**固定场景**（手工 tscn，注册进 map_paths 即可，如萧宅/青石镇/青云坊）
与**随机场景**（一张"壳" tscn，运行时按种子重新出图，如秘境 dungeon.tscn）。
随机场景在注册表里仍是普通别名——「随机」体现在种子与刷怪，不走特殊通道。

实现要点（范本 example/demo/xiuxian/map/dungeon/ + scenes/main/dungeon_manager.gd）：

```gdscript
# 随机地图 = GdQuickMap 根（静态基础参数）+ 楼层控制器子节点（dungeon_map.gd）：
# _process 首帧（父地图 ready 之后）：
map.generate(int(state.floor_seed()))          # 层种子出图（同种子同图，可复现）
# 随机刷怪：变体 tscn instantiate → 覆写导出值 → map.add_child（enemy 自吸附可行走格）
# boss 变体：同 tscn 覆写 display_name/max_health/... + scale
# 清场判定：每只怪 health.s_died 计数，清零发 s_floor_cleared
```

- 流程分两个场景（进入秘境 = **整场景切换** change_scene，走 GdScene 转场遮罩）：
  · main 场景 DungeonManager：start_run() 记录入口（固定地图+玩家所在格）→
    change_scene("探索场景")；从探索回来后 wire 时按 from_map 送回入口
  · explore 场景 ExploreManager：wire 后 open_map(随机别名) → 接楼层
    s_floor_cleared → 非 boss 层延时 advance_floor 重开 → boss 层清空发奖 +
    弹结算窗 → 关闭后 change_scene 回主场景；中途 abandon() 放弃
  · **清场信号防重放**：楼层控制器 `_alive <= 0` 即发 s_floor_cleared，
    同帧多杀会重放信号——结算/换层处理器必须带「run 进行中」守卫
    （`_state.status != RUNNING` 直接忽略），否则结算后迟到的信号会二次发奖
  · 场景注册表分离：主场景 MapManager 只注册固定场景；随机场景只在
    explore 场景的 MapManager 注册（auto_load=false，业务手动开图）
  · 转场防重入红线：GdSceneRoot.change_scene 在转场动画期间（is_changing）
    拒绝切换——业务侧同步调用可能失败，必须 _process 逐帧重试（带次数上限）；
    场景节点 _exit_tree 注销组件表必须 call_deferred（场景替换发生在
    GdSceneRoot 的 Rust 调用栈内，同步回调 unregister_component 会重入 panic）
- 层种子 = run_seed + floor * 7919（同次挑战内每层不同、单层可复现）；
  刷怪位置/掉落共用该种子 → 结算结果可追溯
- 状态数据进 GdBean（XiuDungeonState：status/floor/run_seed/from_map/
  settle_view/boss_cleared），存档恢复残留 running 由控制器 _wire 作废
- 进度持久化注意：boss_cleared 这类"一次性"标记必须放 Bean 并提供
  reset_demo（跨运行确定性测试依赖）
