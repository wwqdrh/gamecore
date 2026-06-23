// DisplayLayer - 显示层 TileMapLayer
// 移植自 GDScript TileMapDual/addons/TileMapDual/display_layer.gd
// 单个 TileMapLayer，根据父 TileMapDual 的内容和 TerrainLayer 的规则自动计算和更新显示贴图
// 维护双网格错觉：世界网格存储逻辑地形，显示层根据四角组合查表得到过渡贴图

use std::collections::HashSet;

use godot::prelude::*;
use godot::classes::{
    ITileMapLayer, Material, TileMapLayer,
};
use godot::classes::tile_set::CellNeighbor;
use godot::builtin::{Array, Variant, VarArray, Vector2, Vector2i};
use godot::obj::EngineEnum;

use super::tile_cache::TileCache;
use super::tile_set_watcher::TileSetWatcher;
use super::terrain_layer::TerrainLayer;
use super::util;

#[derive(GodotClass)]
#[class(base = TileMapLayer)]
pub struct DisplayLayer {
    /// 相对于主 TileMapDual 网格的偏移量（与 tile_size 无关）
    #[var(pub)]
    offset: Vector2,
    /// 关联的 TileSetWatcher
    tileset_watcher: Option<Gd<TileSetWatcher>>,
    /// 关联的 TerrainLayer
    terrain: Option<Gd<TerrainLayer>>,
    base: Base<TileMapLayer>,
}

#[godot_api]
impl ITileMapLayer for DisplayLayer {
    fn init(base: Base<TileMapLayer>) -> Self {
        Self {
            offset: Vector2::ZERO,
            tileset_watcher: None,
            terrain: None,
            base,
        }
    }
}

#[godot_api]
impl DisplayLayer {
    /// 初始化 DisplayLayer
    /// 原版 _init(world, tileset_watcher, fields, layer) 在 gdext 中无法自定义构造函数参数
    /// 改用 setup() 方法在创建后调用
    #[func]
    fn setup(
        &mut self,
        world: Gd<TileMapLayer>,
        tileset_watcher: Gd<TileSetWatcher>,
        offset: Vector2,
        layer: Gd<TerrainLayer>,
    ) {
        self.update_properties(world);
        self.offset = offset;
        self.tileset_watcher = Some(tileset_watcher.clone());
        self.terrain = Some(layer);

        // 设置 tile_set（使用 call 避免 gdext 0.5.3 的 ByValue/ByOption 类型不匹配）
        let tile_set = tileset_watcher.bind().get_tile_set();
        self.base_mut().call("set_tile_set", &[tile_set.to_variant()]);

        // 连接 tileset_resized 信号到 reposition（延迟调用）
        let callable = Callable::from_object_method(&*self.base_mut(), "reposition");
        let mut watcher_mut = tileset_watcher.clone();
        let _ = watcher_mut.connect_flags("tileset_resized", &callable, godot::classes::object::ConnectFlags::DEFERRED);

        self.reposition();
    }

    /// 根据 tile_set 的 tile_size 调整此 DisplayLayer 的位置
    #[func]
    fn reposition(&mut self) {
        if let Some(watcher) = &self.tileset_watcher {
            let tile_size = watcher.bind().get_tile_size();
            let offset = self.offset;
            self.base_mut().set_position(offset * Vector2::new(tile_size.x as f32, tile_size.y as f32));
        }
    }

    /// 从父 TileMapDual 复制属性到子显示层
    /// parent 为 TileMapDual（继承自 TileMapLayer），传入 TileMapLayer 类型以避免循环依赖
    /// display_material 通过 call("get_display_material") 动态获取
    #[func]
    fn update_properties(&mut self, mut parent: Gd<TileMapLayer>) {
        let mut base = self.base_mut();
        // Rendering
        base.set_y_sort_origin(parent.get_y_sort_origin());
        base.set_x_draw_order_reversed(parent.is_x_draw_order_reversed());
        base.set_rendering_quadrant_size(parent.get_rendering_quadrant_size());
        // Physics
        base.set_collision_enabled(parent.is_collision_enabled());
        base.set_use_kinematic_bodies(parent.is_using_kinematic_bodies());
        base.set_collision_visibility_mode(parent.get_collision_visibility_mode());
        // Navigation
        base.set_navigation_enabled(parent.is_navigation_enabled());
        base.set_navigation_visibility_mode(parent.get_navigation_visibility_mode());
        // Canvas item properties
        base.set_draw_behind_parent(parent.is_draw_behind_parent_enabled());
        base.set_as_top_level(parent.is_set_as_top_level());
        base.set_light_mask(parent.get_light_mask());
        base.set_visibility_layer(parent.get_visibility_layer());
        base.set_y_sort_enabled(parent.is_y_sort_enabled());
        base.set_modulate(parent.get_modulate());
        base.set_self_modulate(parent.get_self_modulate());
        // NOTE: parent material takes priority over the current shaders,
        // causing the world tiles to show up
        base.set_use_parent_material(parent.get_use_parent_material());

        // display_material - 通过动态调用获取（TileMapDual 特有属性）
        let material_var = parent.call("get_display_material", &[]);
        // 使用 call 设置 material（避免 ByValue/ByOption 类型不匹配）
        base.call("set_material", &[material_var]);
    }

    /// 更新所有显示贴图以反映当前变化
    #[func]
    fn update_tiles_all(&mut self, cache: Gd<TileCache>) {
        let cells = cache.bind().get_cells_keys();
        self.update_tiles(cache, cells);
    }

    /// 更新受世界格子变化影响的所有显示贴图
    #[func]
    fn update_tiles(&mut self, cache: Gd<TileCache>, updated_world_cells: Array<Variant>) {
        let Some(terrain) = self.terrain.clone() else { return };
        let terrain_borrowed = terrain.bind();

        // 获取 display_to_world_neighborhood（使用自动生成的 getter）
        let display_to_world: Vec<Vec<CellNeighbor>> = terrain_borrowed
            .get_display_to_world_neighborhood()
            .iter_shared()
            .map(|v| -> Vec<CellNeighbor> {
                let row: VarArray = v.to();
                row.iter_shared()
                    .map(|n| -> CellNeighbor {
                        let ord: i32 = n.to();
                        CellNeighbor::from_ord(ord)
                    })
                    .collect()
            })
            .collect();

        // 收集需要更新的 display cells（去重）
        let mut already_updated: HashSet<Vector2i> = HashSet::new();
        let cells_to_update: Vec<Vector2i> = {
            let mut cells = Vec::new();
            for path in &display_to_world {
                // 反转路径
                let reversed_path: Vec<CellNeighbor> = path.iter().map(|n| util::reverse_neighbor(*n)).collect();
                for world_cell_var in updated_world_cells.iter_shared() {
                    let world_cell: Vector2i = world_cell_var.to();
                    let display_cell = self.follow_path_slice(world_cell, &reversed_path);
                    if already_updated.insert(display_cell) {
                        cells.push(display_cell);
                    }
                }
            }
            cells
        };

        // 更新每个 display cell
        drop(terrain_borrowed);
        for cell in cells_to_update {
            self.update_tile(cache.clone(), cell);
        }
    }

    /// 更新指定的世界格子对应的显示贴图
    #[func]
    fn update_tile(&mut self, cache: Gd<TileCache>, cell: Vector2i) {
        let Some(terrain) = self.terrain.clone() else { return };
        let terrain_borrowed = terrain.bind();

        // 获取 display_to_world_neighborhood 并计算 terrain_neighbors
        let display_to_world: Vec<Vec<CellNeighbor>> = terrain_borrowed
            .get_display_to_world_neighborhood()
            .iter_shared()
            .map(|v| -> Vec<CellNeighbor> {
                let row: VarArray = v.to();
                row.iter_shared()
                    .map(|n| -> CellNeighbor {
                        let ord: i32 = n.to();
                        CellNeighbor::from_ord(ord)
                    })
                    .collect()
            })
            .collect();

        // 对每条路径，获取对应的 terrain 值
        let mut terrain_neighbors = VarArray::new();
        for path in &display_to_world {
            let target_cell = self.follow_path_slice(cell, path);
            let terrain_value = cache.bind().get_terrain_at_public(target_cell);
            terrain_neighbors.push(&terrain_value.to_variant());
        }

        drop(terrain_borrowed);

        // 调用 apply_rule 获取贴图映射
        let mut terrain_mut = terrain.clone();
        let mapping = terrain_mut.bind_mut().apply_rule_public(terrain_neighbors, cell);
        let sid: i32 = mapping.get(&"sid".to_variant()).unwrap().to::<i32>();
        let tile: Vector2i = mapping.get(&"tile".to_variant()).unwrap().to::<Vector2i>();

        // 设置显示贴图（使用 set_cell_ex 构建器模式）
        self.base_mut()
            .set_cell_ex(cell)
            .source_id(sid)
            .atlas_coords(tile)
            .done();
    }

    /// 沿 CellNeighbor 路径查找邻居格子
    #[func]
    fn follow_path(&self, cell: Vector2i, path: Array<Variant>) -> Vector2i {
        let mut current = cell;
        for neighbor_var in path.iter_shared() {
            let ord: i32 = neighbor_var.to();
            let neighbor = CellNeighbor::from_ord(ord);
            current = self.base().get_neighbor_cell(current, neighbor);
        }
        current
    }
}

/// 内部辅助方法
impl DisplayLayer {
    /// 沿 CellNeighbor 切片路径查找邻居格子（内部使用，避免 Array 转换开销）
    fn follow_path_slice(&self, cell: Vector2i, path: &[CellNeighbor]) -> Vector2i {
        let mut current = cell;
        for neighbor in path {
            current = self.base().get_neighbor_cell(current, *neighbor);
        }
        current
    }
}

/// 公开方法（供 Display 调用）
impl DisplayLayer {
    /// 初始化 DisplayLayer（公开方法）
    pub fn setup_public(
        &mut self,
        world: Gd<TileMapLayer>,
        tileset_watcher: Gd<TileSetWatcher>,
        offset: Vector2,
        layer: Gd<TerrainLayer>,
    ) {
        self.setup(world, tileset_watcher, offset, layer);
    }

    /// 更新所有显示贴图（公开方法）
    pub fn update_tiles_all_public(&mut self, cache: Gd<TileCache>) {
        self.update_tiles_all(cache);
    }

    /// 更新受影响的显示贴图（公开方法）
    pub fn update_tiles_public(&mut self, cache: Gd<TileCache>, updated_world_cells: Array<Variant>) {
        self.update_tiles(cache, updated_world_cells);
    }

    /// 更新属性（公开方法）
    pub fn update_properties_public(&mut self, parent: Gd<TileMapLayer>) {
        self.update_properties(parent);
    }
}
