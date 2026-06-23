// TerrainDual - 读取 TileSet 并决定显示地图中哪些贴图与世界地图中的邻居匹配
// 移植自 GDScript TileMapDual/addons/TileMapDual/terrain_dual.gd
// 管理 Neighborhood 类型、地形映射表和 TerrainLayer 列表
// 当 TileSetWatcher 发出 terrains_changed 信号时重新读取 TileSet

use godot::prelude::*;
use godot::classes::{
    IResource, Resource, TileData, TileSet, TileSetAtlasSource, TileSetSource,
};
use godot::classes::object::ConnectFlags;
use godot::classes::tile_set::CellNeighbor;
use godot::builtin::{Variant, VarArray, VarDictionary, Vector2i};
use godot::obj::EngineEnum;

use super::grid_shape::{self, GridShape};
use super::terrain_layer::TerrainLayer;
use super::tile_set_watcher::TileSetWatcher;

/// 邻居类型：决定显示贴图查看世界网格的哪些邻居
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Neighborhood {
    /// 方形
    Square,
    /// 等距
    Isometric,
    /// 水平三角形
    TriangleHorizontal,
    /// 垂直三角形
    TriangleVertical,
}

impl Neighborhood {
    pub fn ord(self) -> i32 {
        self as i32
    }

    pub fn from_ord(n: i32) -> Self {
        match n {
            0 => Neighborhood::Square,
            1 => Neighborhood::Isometric,
            2 => Neighborhood::TriangleHorizontal,
            3 => Neighborhood::TriangleVertical,
            _ => Neighborhood::Square,
        }
    }
}

/// 层配置：地形邻居和显示到世界的邻居映射
struct LayerConfig {
    terrain_neighborhood: Vec<CellNeighbor>,
    display_to_world_neighborhood: Vec<Vec<CellNeighbor>>,
}

/// 根据 GridShape 返回对应的 Neighborhood
pub fn grid_to_neighborhood(grid_shape: GridShape) -> Neighborhood {
    match grid_shape {
        GridShape::Square => Neighborhood::Square,
        GridShape::Iso => Neighborhood::Isometric,
        GridShape::HalfOffHori => Neighborhood::TriangleHorizontal,
        GridShape::HalfOffVert => Neighborhood::TriangleVertical,
        GridShape::HexHori => Neighborhood::TriangleHorizontal,
        GridShape::HexVert => Neighborhood::TriangleVertical,
    }
}

/// 根据 TileSet 返回对应的 Neighborhood
pub fn tileset_neighborhood(tile_set: &Gd<TileSet>) -> Neighborhood {
    let grid_shape = grid_shape::tileset_gridshape(tile_set);
    grid_to_neighborhood(grid_shape)
}

/// 返回指定 Neighborhood 的层配置
fn neighborhood_layers(neighborhood: Neighborhood) -> Vec<LayerConfig> {
    match neighborhood {
        Neighborhood::Square => vec![
            LayerConfig {
                terrain_neighborhood: vec![
                    CellNeighbor::TOP_LEFT_CORNER,
                    CellNeighbor::TOP_RIGHT_CORNER,
                    CellNeighbor::BOTTOM_LEFT_CORNER,
                    CellNeighbor::BOTTOM_RIGHT_CORNER,
                ],
                display_to_world_neighborhood: vec![
                    vec![CellNeighbor::TOP_LEFT_CORNER],
                    vec![CellNeighbor::TOP_SIDE],
                    vec![CellNeighbor::LEFT_SIDE],
                    vec![],
                ],
            },
        ],
        Neighborhood::Isometric => vec![
            LayerConfig {
                terrain_neighborhood: vec![
                    CellNeighbor::TOP_CORNER,
                    CellNeighbor::RIGHT_CORNER,
                    CellNeighbor::LEFT_CORNER,
                    CellNeighbor::BOTTOM_CORNER,
                ],
                display_to_world_neighborhood: vec![
                    vec![CellNeighbor::TOP_CORNER],
                    vec![CellNeighbor::TOP_RIGHT_SIDE],
                    vec![CellNeighbor::TOP_LEFT_SIDE],
                    vec![],
                ],
            },
        ],
        Neighborhood::TriangleHorizontal => vec![
            LayerConfig {
                // v
                terrain_neighborhood: vec![
                    CellNeighbor::BOTTOM_CORNER,
                    CellNeighbor::TOP_LEFT_CORNER,
                    CellNeighbor::TOP_RIGHT_CORNER,
                ],
                display_to_world_neighborhood: vec![
                    vec![],
                    vec![CellNeighbor::TOP_LEFT_SIDE],
                    vec![CellNeighbor::TOP_RIGHT_SIDE],
                ],
            },
            LayerConfig {
                // ^
                terrain_neighborhood: vec![
                    CellNeighbor::TOP_CORNER,
                    CellNeighbor::BOTTOM_LEFT_CORNER,
                    CellNeighbor::BOTTOM_RIGHT_CORNER,
                ],
                display_to_world_neighborhood: vec![
                    vec![CellNeighbor::TOP_LEFT_SIDE],
                    vec![CellNeighbor::LEFT_SIDE],
                    vec![],
                ],
            },
        ],
        Neighborhood::TriangleVertical => vec![
            LayerConfig {
                // >
                terrain_neighborhood: vec![
                    CellNeighbor::RIGHT_CORNER,
                    CellNeighbor::TOP_LEFT_CORNER,
                    CellNeighbor::BOTTOM_LEFT_CORNER,
                ],
                display_to_world_neighborhood: vec![
                    vec![],
                    vec![CellNeighbor::TOP_LEFT_SIDE],
                    vec![CellNeighbor::BOTTOM_LEFT_SIDE],
                ],
            },
            LayerConfig {
                // <
                terrain_neighborhood: vec![
                    CellNeighbor::LEFT_CORNER,
                    CellNeighbor::TOP_RIGHT_CORNER,
                    CellNeighbor::BOTTOM_RIGHT_CORNER,
                ],
                display_to_world_neighborhood: vec![
                    vec![CellNeighbor::TOP_LEFT_SIDE],
                    vec![CellNeighbor::TOP_SIDE],
                    vec![],
                ],
            },
        ],
    }
}

/// 将 CellNeighbor 列表转为 VarArray（存储 ord 值）
fn cell_neighbors_to_var_array(neighbors: &[CellNeighbor]) -> VarArray {
    let mut arr = VarArray::new();
    for n in neighbors {
        arr.push(&n.ord().to_variant());
    }
    arr
}

/// 将 CellNeighbor 二维列表转为 VarArray（每个元素也是 VarArray）
fn cell_neighbors_2d_to_var_array(neighbors: &[Vec<CellNeighbor>]) -> VarArray {
    let mut arr = VarArray::new();
    for row in neighbors {
        arr.push(&cell_neighbors_to_var_array(row).to_variant());
    }
    arr
}

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct TerrainDual {
    /// 此 TerrainDual 的 Neighborhood 类型（存储为 Neighborhood 序数的 i32）
    #[var]
    neighborhood: i32,
    /// 地形类型到贴图的映射：terrain -> {sid, tile}
    #[var]
    terrains: VarDictionary,
    /// TerrainLayer 列表
    #[var]
    layers: VarArray,

    /// 关联的 TileSetWatcher
    tileset_watcher: Option<Gd<TileSetWatcher>>,
    /// 内部使用的 Neighborhood 枚举值
    neighborhood_enum: Neighborhood,
    /// 内部使用的 TerrainLayer 实例列表
    layer_objects: Vec<Gd<TerrainLayer>>,
    base: Base<Resource>,
}

#[godot_api]
impl IResource for TerrainDual {
    fn init(base: Base<Resource>) -> Self {
        Self {
            neighborhood: Neighborhood::Square.ord(),
            terrains: VarDictionary::new(),
            layers: VarArray::new(),
            tileset_watcher: None,
            neighborhood_enum: Neighborhood::Square,
            layer_objects: Vec::new(),
            base,
        }
    }
}

#[godot_api]
impl TerrainDual {
    /// 创建 TerrainDual 并连接到 TileSetWatcher 的 terrains_changed 信号
    /// 原版 _init(tileset_watcher) 在 gdext 中无法自定义构造函数参数，改用 new_dual() 静态方法
    #[func]
    fn new_dual(mut tileset_watcher: Gd<TileSetWatcher>) -> Gd<Self> {
        let mut gd = Gd::<Self>::from_init_fn(|base| TerrainDual::init(base));
        gd.bind_mut().tileset_watcher = Some(tileset_watcher.clone());

        // 连接 terrains_changed 信号到 _changed 回调（延迟调用）
        let callable = Callable::from_object_method(&gd, "_changed");
        let _ = tileset_watcher.connect_flags("terrains_changed", &callable, ConnectFlags::DEFERRED);

        // 立即触发一次 _changed
        gd.bind_mut().changed();
        gd
    }

    /// terrains_changed 信号回调，重新读取 TileSet
    #[func]
    fn changed(&mut self) {
        let tile_set = self.tileset_watcher.as_ref().and_then(|w| w.bind().get_tile_set());
        self.read_tileset(tile_set);
        self.base_mut().emit_signal("changed", &[]);
    }

    /// 读取 TileSet 中的所有 atlas，创建地形规则
    #[func]
    fn read_tileset(&mut self, tile_set: Option<Gd<TileSet>>) {
        self.terrains = VarDictionary::new();
        self.layers = VarArray::new();
        self.layer_objects.clear();
        self.neighborhood_enum = Neighborhood::Square; // 默认值

        let Some(tile_set) = tile_set else { return };

        self.neighborhood_enum = tileset_neighborhood(&tile_set);
        self.neighborhood = self.neighborhood_enum.ord();

        // 为每个层配置创建 TerrainLayer 实例
        let configs = neighborhood_layers(self.neighborhood_enum);
        for config in &configs {
            let mut layer = Gd::<TerrainLayer>::from_init_fn(|base| TerrainLayer::init(base));
            layer.bind_mut().set_terrain_neighborhood(cell_neighbors_to_var_array(&config.terrain_neighborhood));
            layer.bind_mut().set_display_to_world_neighborhood(cell_neighbors_2d_to_var_array(&config.display_to_world_neighborhood));
            self.layers.push(&layer.to_variant());
            self.layer_objects.push(layer);
        }

        // 读取所有 atlas 中的贴图
        let source_count = tile_set.get_source_count();
        for i in 0..source_count {
            let sid = tile_set.get_source_id(i);
            let source = tile_set.get_source(sid);
            let Some(source) = source else { continue };
            if !source.is_class("TileSetAtlasSource") {
                continue;
            }
            let atlas = source.cast::<TileSetAtlasSource>();
            self.read_atlas(&atlas, sid);
        }
    }

    /// 读取 atlas 中的所有贴图
    fn read_atlas(&mut self, atlas: &Gd<TileSetAtlasSource>, sid: i32) {
        let size = atlas.get_atlas_grid_size();
        for y in 0..size.y {
            for x in 0..size.x {
                let tile = Vector2i::new(x, y);
                if !atlas.has_tile(tile) {
                    continue;
                }
                self.read_tile(atlas, sid, tile);
            }
        }
    }

    /// 为 atlas 中的单个贴图添加规则
    fn read_tile(&mut self, atlas: &Gd<TileSetAtlasSource>, sid: i32, tile: Vector2i) {
        let data = atlas.get_tile_data(tile, 0);
        let Some(data) = data else { return };

        let terrain_set = data.get_terrain_set();
        if terrain_set != 0 {
            godot_warn!(
                "The tile at {:?} has a terrain set of {}. Only terrain set 0 is supported.",
                tile, terrain_set
            );
            return;
        }

        let terrain = data.get_terrain();
        if terrain != -1 {
            let terrain_key = terrain.to_variant();
            if !self.terrains.contains_key(&terrain_key) {
                let mut mapping = VarDictionary::new();
                mapping.set(&"sid".to_variant(), &sid.to_variant());
                mapping.set(&"tile".to_variant(), &tile.to_variant());
                self.terrains.set(&terrain_key, &mapping.to_variant());
            }
        }

        // 为每个 TerrainLayer 注册贴图规则
        for layer in &mut self.layer_objects {
            layer.bind_mut().register_tile(&data, sid, tile);
        }
    }
}

// ===== 内部实现 =====
impl TerrainDual {
    /// 获取 Neighborhood 枚举值
    pub fn get_neighborhood_enum(&self) -> Neighborhood {
        self.neighborhood_enum
    }

    /// 获取 TerrainLayer 实例列表
    pub fn get_layer_objects(&self) -> &[Gd<TerrainLayer>] {
        &self.layer_objects
    }

    /// 创建 TerrainDual（公开包装方法，供 Display 调用）
    pub fn new_dual_public(tileset_watcher: Gd<TileSetWatcher>) -> Gd<Self> {
        Self::new_dual(tileset_watcher)
    }

    /// 获取 terrains 字典（供 TileMapDual.draw_cell 调用）
    pub fn get_terrains_public(&self) -> VarDictionary {
        self.terrains.clone()
    }
}
