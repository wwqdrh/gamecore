// Util - 工具函数模块
// 移植自 GDScript TileMapDual/addons/TileMapDual/util.gd
// 提供 Vector2i 转置、CellNeighbor 反转/命名等静态工具函数

use godot::builtin::Vector2i;
use godot::classes::tile_set::CellNeighbor;
use godot::obj::EngineEnum;

/// 交换 Vector2i 的 X 和 Y 轴
pub fn transpose_vec(v: Vector2i) -> Vector2i {
    Vector2i::new(v.y, v.x)
}

/// 反转 CellNeighbor 的方向： (neighbor + 8) % 16
pub fn reverse_neighbor(neighbor: CellNeighbor) -> CellNeighbor {
    let n = ord(neighbor);
    from_ord((n + 8) % 16)
}

/// 返回 CellNeighbor 的简写名称 (E/SE/S/SW/W/NW/N/NE)
pub fn neighbor_name(neighbor: CellNeighbor) -> String {
    const DIRECTIONS: [&str; 8] = ["E", "SE", "S", "SW", "W", "NW", "N", "NE"];
    let idx = (ord(neighbor) >> 1) as usize;
    DIRECTIONS[idx].to_string()
}

/// 将 CellNeighbor 转为 i32 序数
pub fn ord(neighbor: CellNeighbor) -> i32 {
    neighbor.ord()
}

/// 将 i32 序数转为 CellNeighbor
pub fn from_ord(n: i32) -> CellNeighbor {
    CellNeighbor::from_ord(n)
}
