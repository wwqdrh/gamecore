# 地图系统：双网格地形、噪声生成、寻路

对应 Rust 源码：`rust/src/map/`（dual_grid / gd_map_basic / quick_map）
可运行示例：`example/map/map_demo.gd`、`example/map/basic.gd`

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
