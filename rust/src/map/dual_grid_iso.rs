// 等距双网格地图算法核心逻辑
// 与方形双网格（dual_grid.rs）独立，针对等距（Isometric）网格的邻域结构设计
// 等距网格的4角方向为：上(TOP_CORNER)、右(RIGHT_CORNER)、左(LEFT_CORNER)、下(BOTTOM_CORNER)
// 显示层偏移为 (0, -0.5) * tile_size，而非方形网格的 (-0.5, -0.5) * tile_size
use godot::prelude::*;
use std::collections::HashMap;

/// 地形类型（动态 ID，0 = Null，1+ = 用户注册的地形）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IsoTerrainType(pub u16);

impl IsoTerrainType {
    pub const NULL: IsoTerrainType = IsoTerrainType(0);

    pub fn from_i32(v: i32) -> Self {
        IsoTerrainType(v.max(0) as u16)
    }

    pub fn to_i32(self) -> i32 {
        self.0 as i32
    }

    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// 四角状态：NotNull 或 Null
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsoCornerState {
    Null = 0,
    NotNull = 1,
}

/// 等距四角组合键：(top, right, left, bottom)
/// 对应等距网格的4个角方向
type IsoCornerKey = [IsoCornerState; 4];

/// 16种等距四角组合到 atlas_coord 的查找表
/// 等距网格的4角顺序：[top, right, left, bottom]
fn build_iso_tile_lookup() -> HashMap<IsoCornerKey, (i32, i32)> {
    let n = IsoCornerState::Null;
    let y = IsoCornerState::NotNull;

    // 等距网格的16种四角组合，参考 TileMapDual 的 Isometric preset
    let entries: Vec<(IsoCornerKey, (i32, i32))> = vec![
        ([y, y, y, y], (2, 1)),         // 全前景
        ([n, n, n, y], (1, 3)),         // 仅下角前景
        ([n, n, y, n], (0, 0)),         // 仅左角前景
        ([n, y, n, n], (0, 2)),         // 仅右角前景
        ([y, n, n, n], (3, 3)),         // 仅上角前景
        ([n, y, n, y], (1, 0)),         // 右+下前景
        ([y, n, y, n], (3, 2)),         // 上+左前景
        ([n, n, y, y], (3, 0)),         // 左+下前景
        ([y, y, n, n], (1, 2)),         // 上+右前景
        ([n, y, y, y], (1, 1)),         // 上角背景（右+左+下前景）
        ([y, n, y, y], (2, 0)),         // 右角背景（上+左+下前景）
        ([y, y, n, y], (2, 2)),         // 左角背景（上+右+下前景）
        ([y, y, y, n], (3, 1)),         // 下角背景（上+右+左前景）
        ([n, y, y, n], (2, 3)),         // 上+下背景（右+左前景）
        ([y, n, n, y], (0, 1)),         // 左+右背景（上+下前景）
        ([n, n, n, n], (0, 3)),         // 全背景
    ];

    entries.into_iter().collect()
}

/// 等距网格中，一个世界格子变更时影响的显示格子偏移量
/// 等距网格的显示格子位于世界格子的上角位置
/// 世界格子 (wx, wy) 影响的显示格子为：
///   (wx, wy)     — 自身（BOTTOM_CORNER 反转路径为空）
///   (wx+1, wy+1) — 通过 BOTTOM_CORNER 偏移(1,1)
///   (wx, wy+1)   — 通过 BOTTOM_LEFT_SIDE 偏移(0,1)
///   (wx+1, wy)   — 通过 BOTTOM_RIGHT_SIDE 偏移(1,0)
/// 与方形网格 [(0,0),(1,0),(0,1),(1,1)] 不同
const ISO_FOUR_CELLS: [(i32, i32); 4] = [(0, 0), (1, 1), (0, 1), (0, 2)];

/// 等距双网格算法核心
pub struct DualGridIso {
    /// 世界网格：坐标 → 地形类型
    world_tiles: HashMap<(i32, i32), IsoTerrainType>,
    /// 四角组合查找表
    tile_lookup: HashMap<IsoCornerKey, (i32, i32)>,
}

impl DualGridIso {
    pub fn new() -> Self {
        Self {
            world_tiles: HashMap::new(),
            tile_lookup: build_iso_tile_lookup(),
        }
    }

    /// 设置世界格子的地形类型
    pub fn set_world_tile(&mut self, coords: (i32, i32), terrain: IsoTerrainType) {
        if terrain.is_null() {
            self.world_tiles.remove(&coords);
        } else {
            self.world_tiles.insert(coords, terrain);
        }
    }

    /// 获取世界格子的地形类型
    pub fn get_world_tile(&self, coords: (i32, i32)) -> IsoTerrainType {
        self.world_tiles.get(&coords).copied().unwrap_or(IsoTerrainType::NULL)
    }

    /// 清除世界格子
    pub fn erase_world_tile(&mut self, coords: (i32, i32)) {
        self.world_tiles.remove(&coords);
    }

    /// 获取所有已使用的世界格子坐标
    pub fn get_used_cells(&self) -> Vec<(i32, i32)> {
        self.world_tiles.keys().copied().collect()
    }

    /// 计算某个显示格子位置的四角组合对应的 atlas_coord
    /// 等距网格的四角对应的世界格子：
    ///   top    = (display_x - 1, display_y - 1)  上方世界格（TOP_CORNER偏移(-1,-1)）
    ///   right  = (display_x, display_y - 1)       右方世界格（TOP_RIGHT_SIDE偏移(0,-1)）
    ///   left   = (display_x - 1, display_y)       左方世界格（TOP_LEFT_SIDE偏移(-1,0)）
    ///   bottom = (display_x, display_y)            自身（BOTTOM_CORNER无偏移）
    ///
    /// 参考 TileMapDual 的 ISOMETRIC neighborhood:
    ///   terrain_neighborhood: [TOP_CORNER, RIGHT_CORNER, LEFT_CORNER, BOTTOM_CORNER]
    ///   display_to_world_neighborhood:
    ///     TOP_CORNER    → [TOP_CORNER]     即 get_neighbor_cell(dx,dy,TOP_CORNER)    = (dx-1, dy-1)
    ///     RIGHT_CORNER  → [TOP_RIGHT_SIDE] 即 get_neighbor_cell(dx,dy,TOP_RIGHT_SIDE) = (dx, dy-1)
    ///     LEFT_CORNER   → [TOP_LEFT_SIDE]  即 get_neighbor_cell(dx,dy,TOP_LEFT_SIDE)  = (dx-1, dy)
    ///     BOTTOM_CORNER → []               即自身 = (dx, dy)
    pub fn calculate_display_tile(
        &self,
        display_pos: (i32, i32),
        target_terrain: IsoTerrainType,
    ) -> (i32, i32) {
        // 等距网格中，显示格子的四角对应的世界格子
        // 参考 GDScript: display_to_world_neighborhood + Godot get_neighbor_cell() isometric offsets
        //   TOP_CORNER    → follow_path(cell, [TOP_CORNER])     = (dx-1, dy-1)
        //   RIGHT_CORNER  → follow_path(cell, [TOP_RIGHT_SIDE]) = (dx, dy-1)
        //   LEFT_CORNER   → follow_path(cell, [TOP_LEFT_SIDE])  = (dx-1, dy)
        //   BOTTOM_CORNER → follow_path(cell, [])               = (dx, dy) 自身
        let top = (display_pos.0, display_pos.1 - 2);
        let right = (display_pos.0 + 1, display_pos.1);
        let left = (display_pos.0 - 1, display_pos.1);
        let bottom = (display_pos.0, display_pos.1 + 2);

        let corners = [
            self.get_world_tile(top),
            self.get_world_tile(right),
            self.get_world_tile(left),
            self.get_world_tile(bottom),
        ];

        godot_print!("display_pos: {:?}", display_pos);
        godot_print!("corners: {:?}", corners);

        // 将地形类型转换为针对目标地形的 NotNull/Null
        let corner_key: IsoCornerKey = corners.map(|t| {
            if t == target_terrain {
                IsoCornerState::NotNull
            } else {
                IsoCornerState::Null
            }
        });

        self.tile_lookup
            .get(&corner_key)
            .copied()
            .unwrap_or((0, 3)) // 默认全背景
    }

    /// 当世界格子变更时，返回需要更新的显示格子坐标列表
    /// 等距网格中，一个世界格子的变更同样影响周围4个显示格子
    pub fn get_affected_display_positions(&self, world_pos: (i32, i32)) -> Vec<(i32, i32)> {
        ISO_FOUR_CELLS
            .iter()
            .map(|offset| (world_pos.0 + offset.0, world_pos.1 + offset.1))
            .collect()
    }

    /// 噪声生成地形
    /// 返回所有世界格子的地形类型映射
    pub fn generate_terrain_from_noise(
        width: i32,
        height: i32,
        noise_values: &HashMap<(i32, i32), f64>,
        thresholds: &IsoTerrainThresholds,
    ) -> HashMap<(i32, i32), IsoTerrainType> {
        let mut result = HashMap::new();
        for x in 0..width {
            for y in 0..height {
                let value = noise_values.get(&(x, y)).copied().unwrap_or(0.0);
                let terrain = thresholds.classify(value);
                if !terrain.is_null() {
                    result.insert((x, y), terrain);
                }
            }
        }
        result
    }
}

/// 地形阈值条目：地形 ID + 噪声值上限
#[derive(Debug, Clone)]
pub struct IsoTerrainThresholdEntry {
    pub terrain_id: IsoTerrainType,
    pub max_value: f64,
}

/// 地形阈值配置，用于噪声生成
#[derive(Debug, Clone)]
pub struct IsoTerrainThresholds {
    pub entries: Vec<IsoTerrainThresholdEntry>,
}

impl Default for IsoTerrainThresholds {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl IsoTerrainThresholds {
    /// 根据噪声值分类地形
    pub fn classify(&self, value: f64) -> IsoTerrainType {
        for entry in &self.entries {
            if value < entry.max_value {
                return entry.terrain_id;
            }
        }
        IsoTerrainType::NULL
    }
}

/// 地形注册表：维护 name ↔ id 的双向映射
#[derive(Debug, Clone)]
pub struct IsoTerrainRegistry {
    /// 名称 → ID
    name_to_id: HashMap<String, IsoTerrainType>,
    /// ID → 名称
    id_to_name: HashMap<u16, String>,
    /// 下一个可分配的 ID
    next_id: u16,
}

impl IsoTerrainRegistry {
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: HashMap::new(),
            next_id: 1, // 0 保留给 Null
        }
    }

    /// 注册地形，返回分配的 ID。若已存在则返回已有 ID
    pub fn register(&mut self, name: &str) -> IsoTerrainType {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }
        let id = IsoTerrainType(self.next_id);
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.insert(self.next_id, name.to_string());
        self.next_id += 1;
        id
    }

    /// 通过名称获取 ID
    pub fn get_id(&self, name: &str) -> Option<IsoTerrainType> {
        self.name_to_id.get(name).copied()
    }

    /// 通过 ID 获取名称
    pub fn get_name(&self, id: IsoTerrainType) -> Option<&str> {
        self.id_to_name.get(&id.0).map(|s| s.as_str())
    }

    /// 获取所有已注册地形名称（按注册顺序）
    pub fn get_all_names(&self) -> Vec<String> {
        let mut entries: Vec<(u16, &str)> = self
            .id_to_name
            .iter()
            .map(|(id, name)| (*id, name.as_str()))
            .collect();
        entries.sort_by_key(|(id, _)| *id);
        entries.into_iter().map(|(_, name)| name.to_string()).collect()
    }
}

/// 资源放置配置
#[derive(Debug, Clone)]
pub struct IsoPropConfig {
    /// 资源名称
    pub name: String,
    /// TileSet source_id
    pub source_id: i32,
    /// alternative_tile 值
    pub alternative_tile: i32,
    /// 放置概率 (0.0 ~ 1.0)
    pub probability: f64,
    /// 可放置的地形类型
    pub allowed_terrains: Vec<IsoTerrainType>,
    /// 噪声值范围 (min, max)
    pub noise_range: (f64, f64),
}

/// 资源放置结果
#[derive(Debug, Clone)]
pub struct IsoPropPlacement {
    pub coords: (i32, i32),
    pub source_id: i32,
    pub alternative_tile: i32,
}

/// 根据噪声值和概率在地图上放置资源
pub fn iso_place_props(
    width: i32,
    height: i32,
    noise_values: &HashMap<(i32, i32), f64>,
    terrains: &HashMap<(i32, i32), IsoTerrainType>,
    prop_configs: &[IsoPropConfig],
    rng: &mut impl FnMut() -> f64,
) -> Vec<IsoPropPlacement> {
    let mut placements = Vec::new();

    for x in 0..width {
        for y in 0..height {
            let noise_val = noise_values.get(&(x, y)).copied().unwrap_or(0.0);
            let terrain = terrains.get(&(x, y)).copied().unwrap_or(IsoTerrainType::NULL);

            for prop in prop_configs {
                if !prop.allowed_terrains.contains(&terrain) {
                    continue;
                }
                if noise_val < prop.noise_range.0 || noise_val >= prop.noise_range.1 {
                    continue;
                }
                if rng() < prop.probability {
                    placements.push(IsoPropPlacement {
                        coords: (x, y),
                        source_id: prop.source_id,
                        alternative_tile: prop.alternative_tile,
                    });
                }
            }
        }
    }

    placements
}
