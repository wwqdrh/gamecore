// GridShape - 网格形状枚举和检测函数
// 移植自 GDScript TileMapDual/addons/TileMapDual/display.gd 中的 GridShape 枚举和 tileset_gridshape 静态函数
// 提前到独立文件以便 tile_set_watcher.rs 使用，避免与 display.rs 的循环依赖
// 当 display.rs 移植完成后，Display 类将复用此模块的枚举

use godot::classes::TileSet;
use godot::classes::tile_set::{TileOffsetAxis, TileShape};
use godot::obj::Gd;

/// 网格形状：TileSet.tile_shape * TileSet.tile_offset_axis 的所有有意义组合
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridShape {
    /// 方形
    Square,
    /// 等距
    Iso,
    /// 水平半偏移
    HalfOffHori,
    /// 垂直半偏移
    HalfOffVert,
    /// 水平六边形
    HexHori,
    /// 垂直六边形
    HexVert,
}

impl GridShape {
    /// 转为 i32 序数
    pub fn ord(self) -> i32 {
        self as i32
    }

    /// 从 i32 序数转换，无效值默认返回 Square
    pub fn from_ord(n: i32) -> Self {
        match n {
            0 => GridShape::Square,
            1 => GridShape::Iso,
            2 => GridShape::HalfOffHori,
            3 => GridShape::HalfOffVert,
            4 => GridShape::HexHori,
            5 => GridShape::HexVert,
            _ => GridShape::Square,
        }
    }
}

/// 根据 TileSet 的 tile_shape 和 tile_offset_axis 返回对应的 GridShape
/// 默认返回 Square
pub fn tileset_gridshape(tile_set: &Gd<TileSet>) -> GridShape {
    let shape = tile_set.get_tile_shape();
    let offset_axis = tile_set.get_tile_offset_axis();

    match shape {
        TileShape::SQUARE => GridShape::Square,
        TileShape::ISOMETRIC => GridShape::Iso,
        TileShape::HALF_OFFSET_SQUARE => {
            if offset_axis == TileOffsetAxis::HORIZONTAL {
                GridShape::HalfOffHori
            } else {
                GridShape::HalfOffVert
            }
        }
        TileShape::HEXAGON => {
            if offset_axis == TileOffsetAxis::HORIZONTAL {
                GridShape::HexHori
            } else {
                GridShape::HexVert
            }
        }
        _ => GridShape::Square,
    }
}
