// TerrainLayer - 单个 DisplayLayer 使用的地形规则集
// 移植自 GDScript TileMapDual/addons/TileMapDual/terrain_layer.gd
// 使用 trie 结构存储地形邻居到贴图的映射规则，支持随机权重选择
// 内部使用 Rust HashMap 实现 trie，比原版的 Dictionary 嵌套更高效

use std::collections::HashMap;

use godot::prelude::*;
use godot::classes::{IResource, RandomNumberGenerator, Resource, TileData};
use godot::classes::tile_set::CellNeighbor;
use godot::builtin::{GString, PackedFloat32Array, VarArray, VarDictionary, Vector2i};
use godot::obj::{EngineEnum, NewGd};

use super::util;

/// 贴图映射：source_id + atlas_coords + 概率权重
#[derive(Clone, Debug)]
struct Mapping {
    sid: i32,
    tile: Vector2i,
    prob: f32,
}

/// Trie 节点：分支节点有子节点，叶子节点有 mappings
#[derive(Default)]
struct TrieNode {
    branches: HashMap<i32, TrieNode>,
    mappings: Vec<Mapping>,
}

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct TerrainLayer {
    /// 关心的 CellNeighbor 列表（存储为 ord 值的 Array）
    #[var(pub)]
    terrain_neighborhood: VarArray,
    /// 从 display 格子到邻近 world 格子的路径数组
    #[var(pub)]
    display_to_world_neighborhood: VarArray,
    /// 随机种子，用于确定性生成贴图
    #[var]
    global_seed: i32,

    /// 随机数生成器
    rand: Gd<RandomNumberGenerator>,
    /// 规则 trie 根节点
    rules: TrieNode,
    base: Base<Resource>,
}

/// 空贴图常量：{sid: -1, tile: Vector2i(-1, -1)}
fn tile_empty() -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set(&"sid".to_variant(), &(-1i32).to_variant());
    dict.set(&"tile".to_variant(), &Vector2i::new(-1, -1).to_variant());
    dict
}

#[godot_api]
impl IResource for TerrainLayer {
    fn init(base: Base<Resource>) -> Self {
        Self {
            terrain_neighborhood: VarArray::new(),
            display_to_world_neighborhood: VarArray::new(),
            global_seed: 707,
            rand: RandomNumberGenerator::new_gd(),
            rules: TrieNode::default(),
            base,
        }
    }
}

#[godot_api]
impl TerrainLayer {
    /// 将 -1 地形值归一化为 0
    #[func]
    fn normalize_terrain(terrain: i32) -> i32 {
        if terrain == -1 { 0 } else { terrain }
    }

    /// 根据周围地形邻居返回应使用的贴图
    /// terrain_neighbors 为 CellNeighbor 对应的 terrain 值数组
    /// 返回 {sid, tile} 字典，空格子返回 {sid: -1, tile: (-1, -1)}
    #[func]
    fn apply_rule(&mut self, terrain_neighbors: VarArray, cell: Vector2i) -> VarDictionary {
        // 收集地形邻居
        let neighbors: Vec<i32> = terrain_neighbors
            .iter_shared()
            .map(|v| v.to::<i32>())
            .collect();

        // 全部为 -1 时返回空贴图
        let is_empty = neighbors.iter().all(|&t| t == -1);
        if is_empty {
            return tile_empty();
        }

        // 归一化：-1 -> 0
        let normalized: Vec<i32> = neighbors
            .iter()
            .map(|&t| if t == -1 { 0 } else { t })
            .collect();

        // 遍历 trie
        let mappings = {
            let mut current = &self.rules;
            for &terrain in &normalized {
                let t = if current.branches.contains_key(&terrain) {
                    terrain
                } else {
                    0
                };
                match current.branches.get(&t) {
                    Some(next) => current = next,
                    None => return tile_empty(),
                }
            }
            &current.mappings
        };

        if mappings.is_empty() {
            return tile_empty();
        }

        // 随机选择贴图
        let weights: Vec<f32> = mappings.iter().map(|m| m.prob).collect();
        let seed = self.compute_seed(cell);
        self.rand.set_seed(seed);
        let packed_weights: PackedFloat32Array = weights.into();
        let index = self.rand.rand_weighted(&packed_weights);
        let mapping = &mappings[index as usize];

        let mut result = VarDictionary::new();
        result.set(&"sid".to_variant(), &mapping.sid.to_variant());
        result.set(&"tile".to_variant(), &mapping.tile.to_variant());
        result
    }
}

impl TerrainLayer {
    /// 从 TileData 注册一个贴图规则
    pub fn register_tile(&mut self, data: &Gd<TileData>, sid: i32, tile: Vector2i) {
        if data.get_terrain_set() != 0 {
            return;
        }

        // 获取每个 CellNeighbor 的 terrain peering bit
        let neighbors: Vec<CellNeighbor> = self.get_terrain_neighborhood_vec();
        let terrain_neighbors: Vec<i32> = neighbors
            .iter()
            .map(|n| data.get_terrain_peering_bit(*n))
            .collect();

        // 检查无效的邻居组合
        let has_minus_one = terrain_neighbors.iter().any(|&t| t == -1);
        if has_minus_one {
            let has_non_minus_one = terrain_neighbors.iter().any(|&t| t != -1);
            if has_non_minus_one {
                let names: Vec<String> = neighbors.iter().map(|n| util::neighbor_name(*n)).collect();
                godot_print!(
                    "Warning: Invalid Tile Neighborhood at ({}, {}).\nExpected neighborhood: {:?}",
                    tile.x, tile.y, names
                );
            }
            return;
        }

        let mapping = Mapping {
            sid,
            tile,
            prob: data.get_probability(),
        };
        self.register_rule(&terrain_neighbors, mapping);
    }

    /// 注册一组地形邻居到贴图的规则
    pub fn register_rule(&mut self, terrain_neighbors: &[i32], mapping: Mapping) {
        let mut current = &mut self.rules;
        for &terrain in terrain_neighbors {
            current = current.branches.entry(terrain).or_default();
        }
        current.mappings.push(mapping);
    }

    /// 获取 terrain_neighborhood 转为 CellNeighbor 向量
    fn get_terrain_neighborhood_vec(&self) -> Vec<CellNeighbor> {
        self.terrain_neighborhood
            .iter_shared()
            .map(|v| {
                let ord: i32 = v.to();
                CellNeighbor::from_ord(ord)
            })
            .collect()
    }

    /// 计算 cell 和 global_seed 的哈希值作为随机种子
    fn compute_seed(&self, cell: Vector2i) -> u64 {
        // 使用 Godot 的 hash 函数保持与原版一致
        let combined = format!("({}, {}){}", cell.x, cell.y, self.global_seed);
        let hash = combined.to_variant().hash_u32();
        hash as u64
    }
}

/// 公开方法（供 DisplayLayer 调用）
impl TerrainLayer {
    /// 根据周围地形邻居返回应使用的贴图（公开包装方法）
    pub fn apply_rule_public(&mut self, terrain_neighbors: VarArray, cell: Vector2i) -> VarDictionary {
        self.apply_rule(terrain_neighbors, cell)
    }
}
