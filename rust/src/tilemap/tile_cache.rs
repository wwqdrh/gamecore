// TileCache - 缓存 TileMapDual 世界网格中每个格子的贴图位置和地形
// 移植自 GDScript TileMapDual/addons/TileMapDual/tile_cache.gd
// cells 属性：Dictionary[Vector2i, Dictionary]，存储 {sid, tile, terrain}
// 注意：原版 update(world, edited=cells.keys()) 的默认参数在 gdext 中无法直接实现，
// 拆分为 update(world) 更新所有格子，update_edited(world, edited) 更新指定格子

use godot::prelude::*;
use godot::classes::{IResource, Resource, TileMapLayer, TileSet, TileSetSource};
use godot::builtin::{Array, Variant, VarDictionary, Vector2i};

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct TileCache {
    /// 格子缓存：Vector2i -> Dictionary{sid, tile, terrain}
    #[var]
    cells: VarDictionary,
    base: Base<Resource>,
}

#[godot_api]
impl IResource for TileCache {
    fn init(base: Base<Resource>) -> Self {
        Self {
            cells: VarDictionary::new(),
            base,
        }
    }
}

#[godot_api]
impl TileCache {
    /// 更新 TileCache 中所有缓存格子的数据
    /// 若用户意外放置了无效贴图（terrain=-1 或 terrain_set!=0），会重置为之前的值
    #[func]
    fn update(&mut self, mut world: Gd<TileMapLayer>) {
        let cells_to_update: Vec<Vector2i> = self
            .cells
            .keys_array()
            .iter_shared()
            .map(|v| v.to::<Vector2i>())
            .collect();
        self.update_cells(&mut world, cells_to_update);
    }

    /// 更新 TileCache 中指定格子的数据
    #[func]
    fn update_edited(&mut self, mut world: Gd<TileMapLayer>, edited: Array<Vector2i>) {
        let cells_to_update: Vec<Vector2i> = edited.iter_shared().collect();
        self.update_cells(&mut world, cells_to_update);
    }

    /// 返回两个缓存的对称差集 (XOR)
    #[func]
    fn xor(&self, other: Gd<TileCache>) -> Array<Vector2i> {
        let mut out: Array<Vector2i> = Array::new();
        let other_cells = &other.bind().cells;

        for (key, value) in self.cells.iter_shared() {
            let entry: VarDictionary = value.to();
            let terrain: i32 = entry.get("terrain").unwrap().to();
            let cell: Vector2i = key.to();
            if !other_cells.contains_key(&key) {
                out.push(cell);
            } else {
                let other_entry: VarDictionary = other_cells.get(&key).unwrap().to();
                let other_terrain: i32 = other_entry.get("terrain").unwrap().to();
                if terrain != other_terrain {
                    out.push(cell);
                }
            }
        }
        for key in other_cells.keys_array().iter_shared() {
            if !self.cells.contains_key(&key) {
                out.push(key.to::<Vector2i>());
            }
        }
        out
    }

    /// 返回指定坐标的地形值，空格子返回 -1
    #[func]
    fn get_terrain_at(&self, cell: Vector2i) -> i32 {
        let key = cell.to_variant();
        if !self.cells.contains_key(&key) {
            return -1;
        }
        let entry: VarDictionary = self.cells.get(&key).unwrap().to();
        entry.get("terrain").unwrap().to()
    }
}

impl TileCache {
    /// 获取所有缓存格子的键列表（供 DisplayLayer.update_tiles_all 调用）
    pub fn get_cells_keys(&self) -> Array<Variant> {
        self.cells.keys_array()
    }

    /// 返回指定坐标的地形值，空格子返回 -1（公开方法供 DisplayLayer 调用）
    pub fn get_terrain_at_public(&self, cell: Vector2i) -> i32 {
        self.get_terrain_at(cell)
    }

    /// 更新所有缓存格子（公开方法供 Display 调用）
    pub fn update_public(&mut self, world: Gd<TileMapLayer>) {
        self.update(world);
    }

    /// 更新指定缓存格子（公开方法供 Display 调用）
    pub fn update_edited_public(&mut self, world: Gd<TileMapLayer>, edited: Array<Vector2i>) {
        self.update_edited(world, edited);
    }

    /// 返回两个缓存的对称差集（公开方法供 TileMapDual 调用）
    pub fn xor_public(&self, other: Gd<TileCache>) -> Array<Vector2i> {
        self.xor(other)
    }
}

impl TileCache {
    /// 内部方法：更新指定格子的缓存数据
    fn update_cells(&mut self, world: &mut Gd<TileMapLayer>, cells_to_update: Vec<Vector2i>) {
        let Some(tile_set) = world.get_tile_set() else {
            godot_error!("Attempted to update TileCache while tile set was null");
            return;
        };

        for cell in cells_to_update {
            let sid = world.get_cell_source_id(cell);
            if sid == -1 {
                self.cells.remove(&cell.to_variant());
                continue;
            }
            if !tile_set.has_source(sid) {
                continue;
            }
            let Some(src) = tile_set.get_source(sid) else {
                continue;
            };
            let tile = world.get_cell_atlas_coords(cell);
            if !src.has_tile(tile) {
                continue;
            }
            let Some(data) = world.get_cell_tile_data(cell) else {
                continue;
            };

            let terrain = data.get_terrain();
            let terrain_set = data.get_terrain_set();

            // 意外放置的格子应重置为之前的值
            if terrain == -1 || terrain_set != 0 {
                let cell_key = cell.to_variant();
                if !self.cells.contains_key(&cell_key) {
                    world.erase_cell(cell);
                    continue;
                }
                let cached = self.cells.get(&cell_key).unwrap();
                let cached_dict: VarDictionary = cached.to();
                let cached_sid: i32 = cached_dict.get("sid").unwrap().to();
                let cached_tile: Vector2i = cached_dict.get("tile").unwrap().to();
                world
                    .set_cell_ex(cell)
                    .source_id(cached_sid)
                    .atlas_coords(cached_tile)
                    .done();
                let Some(new_data) = world.get_cell_tile_data(cell) else {
                    continue;
                };
                let new_terrain = new_data.get_terrain();
                let mut entry = VarDictionary::new();
                entry.set(&"sid".to_variant(), &cached_sid.to_variant());
                entry.set(&"tile".to_variant(), &cached_tile.to_variant());
                entry.set(&"terrain".to_variant(), &new_terrain.to_variant());
                self.cells.set(&cell_key, &entry);
            } else {
                let mut entry = VarDictionary::new();
                entry.set(&"sid".to_variant(), &sid.to_variant());
                entry.set(&"tile".to_variant(), &tile.to_variant());
                entry.set(&"terrain".to_variant(), &terrain.to_variant());
                self.cells.set(&cell.to_variant(), &entry);
            }
        }
    }
}
