// Display - 显示管理节点
// 移植自 GDScript TileMapDual/addons/TileMapDual/display.gd
// Node2D 节点，管理最多 2 个 DisplayLayer 子节点
// 根据 TileSet 的 GridShape 创建对应数量的 DisplayLayer，并管理其更新

use godot::prelude::*;
use godot::classes::{
    INode2D, ITileMapLayer, Node2D, TileMapLayer,
};
use godot::builtin::{Array, Variant, Vector2};
use godot::classes::object::ConnectFlags;

use super::grid_shape::{self, GridShape};
use super::tile_cache::TileCache;
use super::tile_set_watcher::TileSetWatcher;
use super::terrain_dual::TerrainDual;
use super::display_layer::DisplayLayer;

/// 根据 GridShape 返回各层的 offset 配置
/// 对应原版 GRIDS 常量
fn grids_offsets(grid_shape: GridShape) -> Vec<Vector2> {
    match grid_shape {
        GridShape::Square => vec![
            // []
            Vector2::new(-0.5, -0.5),
        ],
        GridShape::Iso => vec![
            // <>
            Vector2::new(0.0, -0.5),
        ],
        GridShape::HalfOffHori => vec![
            // v
            Vector2::new(0.0, -0.5),
            // ^
            Vector2::new(-0.5, -0.5),
        ],
        GridShape::HalfOffVert => vec![
            // >
            Vector2::new(-0.5, 0.0),
            // <
            Vector2::new(-0.5, -0.5),
        ],
        GridShape::HexHori => vec![
            // v
            Vector2::new(0.0, -3.0 / 8.0),
            // ^
            Vector2::new(-0.5, -3.0 / 8.0),
        ],
        GridShape::HexVert => vec![
            // >
            Vector2::new(-3.0 / 8.0, 0.0),
            // <
            Vector2::new(-3.0 / 8.0, -0.5),
        ],
    }
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Display {
    /// 父 TileMapDual（以 TileMapLayer 类型存储以避免循环依赖）
    #[var(pub)]
    world: Option<Gd<TileMapLayer>>,
    /// 关联的 TerrainDual
    terrain: Option<Gd<TerrainDual>>,
    /// 上次 update() 时计算的 TileCache
    cached_cells: Gd<TileCache>,
    /// 关联的 TileSetWatcher
    tileset_watcher: Option<Gd<TileSetWatcher>>,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Display {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            world: None,
            terrain: None,
            cached_cells: Gd::<TileCache>::from_init_fn(|b| TileCache::init(b)),
            tileset_watcher: None,
            base,
        }
    }
}

#[godot_api]
impl Display {
    /// 当显示贴图被编辑时发射
    #[signal]
    fn world_tiles_changed(changed: Array<Variant>);

    /// 初始化 Display
    /// 原版 _init(world, tileset_watcher) 在 gdext 中无法自定义构造函数参数
    /// 改用 setup() 方法在创建后调用
    #[func]
    fn setup(&mut self, world: Gd<TileMapLayer>, tileset_watcher: Gd<TileSetWatcher>) {
        self.world = Some(world);
        self.tileset_watcher = Some(tileset_watcher.clone());

        // 创建 TerrainDual 并连接 changed 信号
        let mut terrain = TerrainDual::new_dual_public(tileset_watcher);
        let callable = Callable::from_object_method(&*self.base_mut(), "_terrain_changed");
        let _ = terrain.connect_flags("changed", &callable, ConnectFlags::DEFERRED);
        self.terrain = Some(terrain);

        // 连接 world_tiles_changed 信号到 _world_tiles_changed 回调
        let callable = Callable::from_object_method(&*self.base_mut(), "_world_tiles_changed");
        let _ = self.base_mut().connect_flags("world_tiles_changed", &callable, ConnectFlags::DEFERRED);

        // 让父材质穿透到 DisplayLayer
        self.base_mut().set_use_parent_material(true);
        // 启用 Y 排序
        self.base_mut().set_y_sort_enabled(true);
    }

    /// 根据世界 TileMapDual 中变化的格子更新显示
    #[func]
    fn update(&mut self, updated: Array<Variant>) {
        let Some(tileset_watcher) = self.tileset_watcher.clone() else { return };
        if tileset_watcher.bind().get_tile_set().is_none() {
            return;
        }

        self.update_properties();

        if !updated.is_empty() {
            if let Some(world) = self.world.clone() {
                let mut edited: Array<Vector2i> = Array::new();
                for v in updated.iter_shared() {
                    edited.push(v.to::<Vector2i>());
                }
                self.cached_cells.bind_mut().update_edited_public(world, edited);
            }
            self.base_mut().emit_signal("world_tiles_changed", &[updated.to_variant()]);
        }
    }

    /// TerrainDual 变化时重建所有 DisplayLayer
    #[func]
    fn _terrain_changed(&mut self) {
        if let Some(world) = self.world.clone() {
            self.cached_cells.bind_mut().update_public(world);
        }
        self.delete_layers();

        let Some(tileset_watcher) = self.tileset_watcher.clone() else { return };
        if tileset_watcher.bind().get_tile_set().is_some() {
            self.create_layers();
        }
    }

    /// world_tiles_changed 信号回调，更新所有 DisplayLayer
    #[func]
    fn _world_tiles_changed(&mut self, changed: Array<Variant>) {
        let children = self.base().get_children();
        for child in children.iter_shared() {
            if let Ok(display_layer) = child.clone().try_cast::<DisplayLayer>() {
                let mut layer = display_layer.clone();
                layer.bind_mut().update_tiles_public(self.cached_cells.clone(), changed.clone());
            }
        }
    }
}

/// 内部方法
impl Display {
    /// 获取 cached_cells（供 TileMapDual 使用）
    pub fn get_cached_cells_public(&self) -> Gd<TileCache> {
        self.cached_cells.clone()
    }

    /// 获取 terrain（供 TileMapDual 使用）
    pub fn get_terrain_public(&self) -> Option<Gd<TerrainDual>> {
        self.terrain.clone()
    }

    /// 初始化 Display（公开方法供 TileMapDual 调用）
    pub fn setup_public(&mut self, world: Gd<TileMapLayer>, tileset_watcher: Gd<TileSetWatcher>) {
        self.setup(world, tileset_watcher);
    }

    /// 更新显示（公开方法供 TileMapDual 调用）
    pub fn update_public(&mut self, updated: Array<Variant>) {
        self.update(updated);
    }

    /// 根据 GridShape 创建并配置新的 DisplayLayer
    fn create_layers(&mut self) {
        let Some(tileset_watcher) = self.tileset_watcher.clone() else { return };
        let Some(terrain) = self.terrain.clone() else { return };
        let Some(world) = self.world.clone() else { return };

        let grid_shape_ord = tileset_watcher.bind().get_grid_shape();
        let grid_shape = GridShape::from_ord(grid_shape_ord);
        let offsets = grids_offsets(grid_shape);

        let layer_count = {
            let terrain_borrowed = terrain.bind();
            terrain_borrowed.get_layer_objects().len()
        };
        if layer_count != offsets.len() {
            godot_warn!(
                "GridShape {:?} expects {} layers but TerrainDual has {}",
                grid_shape, offsets.len(), layer_count
            );
            return;
        }

        for (i, offset) in offsets.iter().enumerate() {
            let mut layer = Gd::<DisplayLayer>::from_init_fn(|base| ITileMapLayer::init(base));
            let terrain_layer = terrain.bind().get_layer_objects()[i].clone();
            layer.bind_mut().setup_public(
                world.clone(),
                tileset_watcher.clone(),
                *offset,
                terrain_layer,
            );
            self.base_mut().add_child(&layer);
            layer.bind_mut().update_tiles_all_public(self.cached_cells.clone());
        }
    }

    /// 删除所有 DisplayLayer 子节点
    fn delete_layers(&mut self) {
        let children = self.base().get_children();
        for child in children.iter_shared() {
            if let Ok(mut display_layer) = child.clone().try_cast::<DisplayLayer>() {
                display_layer.queue_free();
            }
        }
    }

    /// 当父 TileMapDual 的属性被编辑时更新所有 DisplayLayer 的属性
    fn update_properties(&mut self) {
        let Some(world) = self.world.clone() else { return };
        let children = self.base().get_children();
        for child in children.iter_shared() {
            if let Ok(display_layer) = child.clone().try_cast::<DisplayLayer>() {
                let mut layer = display_layer.clone();
                layer.bind_mut().update_properties_public(world.clone());
            }
        }
    }
}
