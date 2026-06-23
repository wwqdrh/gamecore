// 双网格地图算法核心逻辑
// 世界网格按地形层存储坐标集合，支持同一坐标属于多个地形
// 显示网格根据四角组合查表得到过渡贴图
// TerrainType 使用 u16 newtype，支持动态注册任意地形
use godot::prelude::*;

use std::collections::{HashMap, HashSet};

/// 地形类型（动态 ID，0 = Null，1+ = 用户注册的地形）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrainType(pub u16);

impl TerrainType {
    pub const NULL: TerrainType = TerrainType(0);

    pub fn from_i32(v: i32) -> Self {
        TerrainType(v.max(0) as u16)
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
pub enum CornerState {
    Null = 0,
    NotNull = 1,
}

/// 四角组合键：(up_left, up_right, down_left, down_right)
type CornerKey = [CornerState; 4];

/// 16种四角组合到 atlas_coord 的查找表
/// 对应 GDScript 中的 TILE 字典
fn build_tile_lookup() -> HashMap<CornerKey, (i32, i32)> {
    let n = CornerState::Null;
    let y = CornerState::NotNull;

    let entries: Vec<(CornerKey, (i32, i32))> = vec![
        ([y, y, y, y], (2, 1)),         // 全草
        ([n, n, n, y], (1, 3)),         // 右下角草
        ([n, n, y, n], (0, 0)),         // 左下角草
        ([n, y, n, n], (0, 2)),         // 右上角草
        ([y, n, n, n], (3, 3)),         // 左上角草
        ([n, y, n, y], (1, 0)),         // 右半边草
        ([y, n, y, n], (3, 2)),         // 左半边草
        ([n, n, y, y], (3, 0)),         // 下半边草
        ([y, y, n, n], (1, 2)),         // 上半边草
        ([n, y, y, y], (1, 1)),         // 左上角泥
        ([y, n, y, y], (2, 0)),         // 右上角泥
        ([y, y, n, y], (2, 2)),         // 左下角泥
        ([y, y, y, n], (3, 1)),         // 右下角泥
        ([n, y, y, n], (2, 3)),         // 左上与右下泥
        ([y, n, n, y], (0, 1)),         // 左上与右下草
        ([n, n, n, n], (0, 3)),         // 全泥
    ];

    entries.into_iter().collect()
}

/// 四个偏移量：用于从世界格子坐标计算显示格子四角对应的世界格子
/// FOUR_CELLS = [(0,0), (1,0), (0,1), (1,1)]
/// 对应四角：[左上, 右上, 左下, 右下]
const FOUR_CELLS: [(i32, i32); 4] = [(0, 0), (1, 0), (0, 1), (1, 1)];

/// 双网格算法核心
/// 世界网格按地形层存储坐标集合，支持同一坐标属于多个地形
pub struct DualGrid {
    /// 世界网格：地形类型 → 坐标集合（同一坐标可属于多个地形）
    world_tiles: HashMap<TerrainType, HashSet<(i32, i32)>>,
    /// 四角组合查找表
    tile_lookup: HashMap<CornerKey, (i32, i32)>,
}

impl DualGrid {
    pub fn new() -> Self {
        Self {
            world_tiles: HashMap::new(),
            tile_lookup: build_tile_lookup(),
        }
    }

    /// 设置世界格子的地形（将坐标添加到对应地形层）
    pub fn set_world_tile(&mut self, coords: (i32, i32), terrain: TerrainType) {
        if terrain.is_null() {
            return;
        }
        self.world_tiles
            .entry(terrain)
            .or_insert_with(HashSet::new)
            .insert(coords);
    }

    /// 从指定地形层移除坐标
    pub fn erase_world_tile(&mut self, coords: (i32, i32), terrain: TerrainType) {
        if let Some(set) = self.world_tiles.get_mut(&terrain) {
            set.remove(&coords);
            // 如果该地形层已空，移除整个条目
            if set.is_empty() {
                self.world_tiles.remove(&terrain);
            }
        }
    }

    /// 从所有地形层移除该坐标
    pub fn erase_coord(&mut self, coords: (i32, i32)) {
        for set in self.world_tiles.values_mut() {
            set.remove(&coords);
        }
        // 移除空的地形层
        self.world_tiles.retain(|_, set| !set.is_empty());
    }

    /// 查询某坐标是否属于指定地形
    pub fn has_terrain_at(&self, coords: (i32, i32), terrain: TerrainType) -> bool {
        self.world_tiles
            .get(&terrain)
            .map(|set| set.contains(&coords))
            .unwrap_or(false)
    }

    /// 获取某坐标的所有地形类型
    pub fn get_terrains_at(&self, coords: (i32, i32)) -> Vec<TerrainType> {
        self.world_tiles
            .iter()
            .filter(|(_, set)| set.contains(&coords))
            .map(|(terrain, _)| *terrain)
            .collect()
    }

    /// 获取所有有地形的坐标（去重）
    pub fn get_used_cells(&self) -> Vec<(i32, i32)> {
        let mut all: HashSet<(i32, i32)> = HashSet::new();
        for set in self.world_tiles.values() {
            all.extend(set);
        }
        all.into_iter().collect()
    }

    /// 获取某地形层的所有坐标
    pub fn get_cells_for_terrain(&self, terrain: TerrainType) -> Vec<(i32, i32)> {
        self.world_tiles
            .get(&terrain)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    /// 计算某个显示格子位置的四角组合对应的 atlas_coord
    /// display_pos: 显示格子的坐标
    /// target_terrain: 目标地形类型（用于确定查找哪一层的过渡贴图）
    /// 返回 atlas_coord (x, y)
    pub fn calculate_display_tile(
        &self,
        display_pos: (i32, i32),
        target_terrain: TerrainType,
    ) -> (i32, i32) {
        let up_left = (display_pos.0 - 1, display_pos.1 - 1);
        let up_right = (display_pos.0, display_pos.1 - 1);
        let down_left = (display_pos.0 - 1, display_pos.1);
        let down_right = (display_pos.0, display_pos.1);

        // 检查四角是否属于目标地形
        let corner_key: CornerKey = [
            self.has_terrain_at(up_left, target_terrain),
            self.has_terrain_at(up_right, target_terrain),
            self.has_terrain_at(down_left, target_terrain),
            self.has_terrain_at(down_right, target_terrain),
        ]
        .map(|has| {
            if has {
                CornerState::NotNull
            } else {
                CornerState::Null
            }
        });

        self.tile_lookup
            .get(&corner_key)
            .copied()
            .unwrap_or((0, 3)) // 默认全泥
    }

    /// 当世界格子变更时，返回需要更新的显示格子坐标列表
    /// 一个世界格子的变更会影响周围4个显示格子
    pub fn get_affected_display_positions(&self, world_pos: (i32, i32)) -> Vec<(i32, i32)> {
        FOUR_CELLS
            .iter()
            .map(|offset| (world_pos.0 + offset.0, world_pos.1 + offset.1))
            .collect()
    }

    /// 噪声生成地形（每地形独立判断，一坐标可属多地形）
    /// 返回：坐标 → 该坐标的所有地形列表
    pub fn generate_terrain_from_noise(
        width: i32,
        height: i32,
        noise_values: &HashMap<(i32, i32), f64>,
        thresholds: &TerrainThresholds,
    ) -> HashMap<(i32, i32), Vec<TerrainType>> {
        let mut result: HashMap<(i32, i32), Vec<TerrainType>> = HashMap::new();
        for x in 0..width {
            for y in 0..height {
                let value = noise_values.get(&(x, y)).copied().unwrap_or(0.0);
                // 每个地形独立判断，一个坐标可属于多个地形
                for entry in &thresholds.entries {
                    if value >= entry.min_value && value < entry.max_value {
                        result
                            .entry((x, y))
                            .or_insert_with(Vec::new)
                            .push(entry.terrain_id);
                    }
                }
            }
        }
        result
    }
}

/// 地形阈值条目：地形 ID + 噪声值范围 [min, max)
#[derive(Debug, Clone)]
pub struct TerrainThresholdEntry {
    pub terrain_id: TerrainType,
    pub min_value: f64,
    pub max_value: f64,
}

/// 地形阈值配置，用于噪声生成
/// 每个地形独立判断：噪声值在 [min_value, max_value) 范围内则属于该地形
#[derive(Debug, Clone)]
pub struct TerrainThresholds {
    pub entries: Vec<TerrainThresholdEntry>,
}

impl Default for TerrainThresholds {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

/// 地形注册表：维护 name ↔ id 的双向映射
#[derive(Debug, Clone)]
pub struct TerrainRegistry {
    /// 名称 → ID
    name_to_id: HashMap<String, TerrainType>,
    /// ID → 名称
    id_to_name: HashMap<u16, String>,
    /// 下一个可分配的 ID
    next_id: u16,
}

impl TerrainRegistry {
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: HashMap::new(),
            next_id: 1, // 0 保留给 Null
        }
    }

    /// 注册地形，返回分配的 ID。若已存在则返回已有 ID
    pub fn register(&mut self, name: &str) -> TerrainType {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }
        let id = TerrainType(self.next_id);
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.insert(self.next_id, name.to_string());
        self.next_id += 1;
        id
    }

    /// 通过名称获取 ID
    pub fn get_id(&self, name: &str) -> Option<TerrainType> {
        self.name_to_id.get(name).copied()
    }

    /// 通过 ID 获取名称
    pub fn get_name(&self, id: TerrainType) -> Option<&str> {
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
pub struct PropConfig {
    /// 资源名称
    pub name: String,
    /// TileSet source_id
    pub source_id: i32,
    /// alternative_tile 值
    pub alternative_tile: i32,
    /// 放置概率 (0.0 ~ 1.0)
    pub probability: f64,
    /// 可放置的地形类型
    pub allowed_terrains: Vec<TerrainType>,
    /// 噪声值范围 (min, max)
    pub noise_range: (f64, f64),
}

/// 资源放置结果
#[derive(Debug, Clone)]
pub struct PropPlacement {
    pub coords: (i32, i32),
    pub source_id: i32,
    pub alternative_tile: i32,
}

/// 根据噪声值和概率在地图上放置资源
/// terrains: 坐标 → 该坐标的所有地形列表（支持同一坐标属于多个地形）
pub fn place_props(
    width: i32,
    height: i32,
    noise_values: &HashMap<(i32, i32), f64>,
    terrains: &HashMap<(i32, i32), Vec<TerrainType>>,
    prop_configs: &[PropConfig],
    rng: &mut impl FnMut() -> f64,
) -> Vec<PropPlacement> {
    let mut placements = Vec::new();

    // 统计计数器
    let mut total_coords = 0i32;
    let mut coords_with_terrain = 0i32;
    let mut terrain_match_count = 0i32;
    let mut noise_match_count = 0i32;
    let mut prob_pass_count = 0i32;

    for x in 0..width {
        for y in 0..height {
            total_coords += 1;
            let noise_val = noise_values.get(&(x, y)).copied().unwrap_or(0.0);
            let coord_terrains = terrains.get(&(x, y)).cloned().unwrap_or_default();
            if !coord_terrains.is_empty() {
                coords_with_terrain += 1;
            }

            for prop in prop_configs {
                // 检查该坐标的任一地形是否允许放置此资源
                let terrain_allowed = coord_terrains
                    .iter()
                    .any(|t| prop.allowed_terrains.contains(t));
                if !terrain_allowed {
                    continue;
                }
                terrain_match_count += 1;

                // 检查噪声范围
                if noise_val < prop.noise_range.0 || noise_val >= prop.noise_range.1 {
                    continue;
                }
                noise_match_count += 1;

                // 概率检查
                if rng() < prop.probability {
                    prob_pass_count += 1;
                    placements.push(PropPlacement {
                        coords: (x, y),
                        source_id: prop.source_id,
                        alternative_tile: prop.alternative_tile,
                    });
                }
            }
        }
    }

    godot_print!(
        "[place_props] 统计: 总坐标={}, 有地形={}, 地形匹配={}, 噪声匹配={}, 概率通过={}, 最终放置={}",
        total_coords, coords_with_terrain, terrain_match_count, noise_match_count, prob_pass_count, placements.len()
    );

    placements
}
