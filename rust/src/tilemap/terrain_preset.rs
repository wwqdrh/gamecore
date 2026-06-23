// TerrainPreset - 地形预设静态函数
// 移植自 GDScript TileMapDual/addons/TileMapDual/terrain_preset.gd
// 仅移植 neighborhood_preset() 静态方法和相关常量
// 编辑器相关函数（使用 EditorUndoRedoManager 的 write_default_preset/init_terrains/new_terrain/write_preset/clear_and_resize_atlas/clear_and_divide_atlas）
// 按对齐方案放到 addons/gamecore/map/map.gd 中实现

use godot::prelude::*;
use godot::classes::IRefCounted;
use godot::builtin::{GString, VarArray, VarDictionary, Vector2i};

use super::terrain_dual::Neighborhood;
use super::util;

/// 地形拓扑类型：决定可用的预设集合
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topology {
    /// 方形拓扑（SQUARE / ISOMETRIC 邻居）
    Square,
    /// 三角形拓扑（TRIANGLE_HORIZONTAL / TRIANGLE_VERTICAL 邻居）
    Triangle,
}

/// 预设数据结构（内部使用，对应 GDScript 中的 Dictionary）
struct Preset {
    /// 贴图网格大小
    size: Vector2i,
    /// 背景贴图坐标
    bg: Vector2i,
    /// 前景贴图坐标
    fg: Vector2i,
    /// 每层的贴图序列
    layers: Vec<Vec<Vector2i>>,
}

/// 根据 Neighborhood 返回对应的 Topology
fn neighborhood_topology(neighborhood: Neighborhood) -> Topology {
    match neighborhood {
        Neighborhood::Square => Topology::Square,
        Neighborhood::Isometric => Topology::Square,
        Neighborhood::TriangleHorizontal => Topology::Triangle,
        Neighborhood::TriangleVertical => Topology::Triangle,
    }
}

/// 获取指定 Topology 和名称的预设
/// 对应原版 PRESETS 常量
fn get_preset(topology: Topology, preset_name: &str) -> Option<Preset> {
    match (topology, preset_name) {
        // Topology.SQUARE -> 'Standard'
        (Topology::Square, "Standard") => Some(Preset {
            size: Vector2i::new(4, 4),
            bg: Vector2i::new(0, 3),
            fg: Vector2i::new(2, 1),
            layers: vec![
                vec![
                    // []
                    Vector2i::new(0, 3), Vector2i::new(3, 3), Vector2i::new(0, 2),
                    Vector2i::new(1, 2), Vector2i::new(0, 0), Vector2i::new(3, 2),
                    Vector2i::new(2, 3), Vector2i::new(3, 1), Vector2i::new(1, 3),
                    Vector2i::new(0, 1), Vector2i::new(1, 0), Vector2i::new(2, 2),
                    Vector2i::new(3, 0), Vector2i::new(2, 0), Vector2i::new(1, 1),
                    Vector2i::new(2, 1),
                ],
            ],
        }),
        // Topology.TRIANGLE -> 'Standard'
        (Topology::Triangle, "Standard") => Some(Preset {
            size: Vector2i::new(4, 4),
            bg: Vector2i::new(0, 0),
            fg: Vector2i::new(0, 2),
            layers: vec![
                // v
                vec![
                    Vector2i::new(0, 1), Vector2i::new(2, 3), Vector2i::new(3, 1),
                    Vector2i::new(1, 3), Vector2i::new(1, 1), Vector2i::new(3, 3),
                    Vector2i::new(2, 1), Vector2i::new(0, 3),
                ],
                // ^
                vec![
                    Vector2i::new(0, 0), Vector2i::new(2, 2), Vector2i::new(3, 0),
                    Vector2i::new(1, 2), Vector2i::new(1, 0), Vector2i::new(3, 2),
                    Vector2i::new(2, 0), Vector2i::new(0, 2),
                ],
            ],
        }),
        // Topology.TRIANGLE -> 'Winged' (旧模板，对 Brick/Half-Off Square 不太方便)
        (Topology::Triangle, "Winged") => Some(Preset {
            size: Vector2i::new(4, 4),
            bg: Vector2i::new(0, 0),
            fg: Vector2i::new(0, 2),
            layers: vec![
                // v
                vec![
                    Vector2i::new(0, 1), Vector2i::new(2, 1), Vector2i::new(3, 1),
                    Vector2i::new(1, 3), Vector2i::new(1, 1), Vector2i::new(3, 3),
                    Vector2i::new(2, 3), Vector2i::new(0, 3),
                ],
                // ^
                vec![
                    Vector2i::new(0, 0), Vector2i::new(2, 0), Vector2i::new(3, 0),
                    Vector2i::new(1, 2), Vector2i::new(1, 0), Vector2i::new(3, 2),
                    Vector2i::new(2, 2), Vector2i::new(0, 2),
                ],
            ],
        }),
        // Topology.TRIANGLE -> 'Alternating' (旧模板，三角形间隙难以对齐)
        (Topology::Triangle, "Alternating") => Some(Preset {
            size: Vector2i::new(4, 4),
            bg: Vector2i::new(0, 0),
            fg: Vector2i::new(0, 2),
            layers: vec![
                // v
                vec![
                    Vector2i::new(0, 0), Vector2i::new(2, 0), Vector2i::new(3, 1),
                    Vector2i::new(1, 3), Vector2i::new(1, 1), Vector2i::new(3, 3),
                    Vector2i::new(2, 2), Vector2i::new(0, 2),
                ],
                // ^
                vec![
                    Vector2i::new(0, 1), Vector2i::new(2, 1), Vector2i::new(3, 0),
                    Vector2i::new(1, 2), Vector2i::new(1, 0), Vector2i::new(3, 2),
                    Vector2i::new(2, 3), Vector2i::new(0, 3),
                ],
            ],
        }),
        _ => None,
    }
}

/// 将 Preset 转为 VarDictionary（保持与原版 Dictionary 一致的键名）
fn preset_to_dict(preset: &Preset) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set(&"size".to_variant(), &preset.size.to_variant());
    dict.set(&"bg".to_variant(), &preset.bg.to_variant());
    dict.set(&"fg".to_variant(), &preset.fg.to_variant());

    let mut layers = VarArray::new();
    for layer in &preset.layers {
        let mut layer_arr = VarArray::new();
        for tile in layer {
            layer_arr.push(&tile.to_variant());
        }
        layers.push(&layer_arr.to_variant());
    }
    dict.set(&"layers".to_variant(), &layers.to_variant());
    dict
}

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct TerrainPreset {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for TerrainPreset {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl TerrainPreset {
    /// 根据 Neighborhood 和预设名称返回预设字典
    /// 返回 {size, bg, fg, layers} 字典
    /// 不存在时返回 {size: Vector2i.ONE, layers: []}
    /// TRIANGLE_VERTICAL 邻居会自动转置所有 Vector2i
    #[func]
    fn neighborhood_preset(neighborhood: i32, preset_name: GString) -> VarDictionary {
        let neighborhood = Neighborhood::from_ord(neighborhood);
        let topology = neighborhood_topology(neighborhood);
        let preset_name = preset_name.to_string();

        let Some(mut preset) = get_preset(topology, &preset_name) else {
            // 预设不存在时返回默认空预设
            let mut dict = VarDictionary::new();
            dict.set(&"size".to_variant(), &Vector2i::ONE.to_variant());
            dict.set(&"layers".to_variant(), &VarArray::new().to_variant());
            return dict;
        };

        // 所有 Horizontal 邻居可以转置为 Vertical
        if neighborhood == Neighborhood::TriangleVertical {
            preset.size = util::transpose_vec(preset.size);
            preset.fg = util::transpose_vec(preset.fg);
            preset.bg = util::transpose_vec(preset.bg);
            for layer in &mut preset.layers {
                for tile in layer.iter_mut() {
                    *tile = util::transpose_vec(*tile);
                }
            }
        }

        preset_to_dict(&preset)
    }
}
