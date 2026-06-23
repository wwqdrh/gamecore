// CursorDual - 双网格光标
// 移植自 GDScript TileMapDual/addons/TileMapDual/cursor_dual.gd
// Sprite2D 节点，跟随鼠标在 TileMapDual 上绘制贴图
// 支持快捷键切换地形（0/1/2），左键绘制，右键擦除

use godot::prelude::*;
use godot::classes::{ISprite2D, Sprite2D, Input};
use godot::builtin::Vector2;

use super::tile_map_dual::TileMapDual;

#[derive(GodotClass)]
#[class(base = Sprite2D)]
pub struct CursorDual {
    /// 目标 TileMapDual
    #[var(pub)]
    tilemap_dual: Option<Gd<TileMapDual>>,
    /// 当前格子坐标
    cell: godot::builtin::Vector2i,
    /// 瓦片大小
    tile_size: Vector2,
    /// 精灵大小
    sprite_size: Vector2,
    /// 当前地形
    terrain: i32,
    base: Base<Sprite2D>,
}

#[godot_api]
impl ISprite2D for CursorDual {
    fn init(base: Base<Sprite2D>) -> Self {
        Self {
            tilemap_dual: None,
            cell: godot::builtin::Vector2i::ZERO,
            tile_size: Vector2::ZERO,
            sprite_size: Vector2::ZERO,
            terrain: 1,
            base,
        }
    }

    fn ready(&mut self) {
        // 先提取 tile_size，避免持有 tilemap_dual 的借用
        let tile_size_opt = self.tilemap_dual.as_ref().and_then(|tilemap_dual| {
            let tilemap = tilemap_dual.bind();
            tilemap.base().get_tile_set().map(|ts| ts.get_tile_size())
        });

        if let Some(tile_size_i) = tile_size_opt {
            self.tile_size = Vector2::new(tile_size_i.x as f32, tile_size_i.y as f32);
            let texture = self.base().get_texture();
            if let Some(tex) = texture {
                self.sprite_size = tex.get_size();
                let scale = Vector2::new(self.tile_size.y, self.tile_size.y) / self.sprite_size;
                self.base_mut().set_scale(scale);
            }
        }
    }

    fn process(&mut self, _delta: f64) {
        let Some(mut tilemap_dual) = self.tilemap_dual.clone() else { return };
        let tilemap = tilemap_dual.bind();

        // 获取鼠标位置对应的格子坐标
        let local_mouse = tilemap.base().get_local_mouse_position();
        self.cell = tilemap.base().local_to_map(local_mouse);
        let global_pos = tilemap.base().to_global(tilemap.base().map_to_local(self.cell));
        self.base_mut().set_global_position(global_pos);

        // 快捷键切换地形
        let input = Input::singleton();
        if input.is_action_pressed("quick_action_1") {
            self.terrain = 1;
        }
        if input.is_action_pressed("quick_action_2") {
            self.terrain = 2;
        }
        if input.is_action_pressed("quick_action_0") {
            self.terrain = 0;
        }

        // 左键绘制，右键擦除
        if input.is_action_pressed("left_click") {
            drop(tilemap);
            tilemap_dual.bind_mut().draw_cell_public(self.cell, self.terrain);
        } else if input.is_action_pressed("right_click") {
            drop(tilemap);
            tilemap_dual.bind_mut().draw_cell_public(self.cell, 0);
        }
    }
}
