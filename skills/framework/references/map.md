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
add_child(map)
map.generate(MAP_SEED)
```

内置 shader 名直接写进 `terrain_shaders`（如 `water_flow`），编译在 Rust 二进制内，无需 .gdshader 文件。

## 寻路与通行判定

```gdscript
map.set_blocked_terrain_names(PackedStringArray(["water", "mountain"]))  # 标记阻挡地形

var walkable: bool = map.is_walkable(cell)
var cell: Vector2i = map.world_to_cell(world_pos)
var center: Vector2 = map.cell_to_center(cell)
var path: PackedVector2Array = map.find_path(from_cell, target_cell)  # BFS

map.clear_map()   # 清空重画
```

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
