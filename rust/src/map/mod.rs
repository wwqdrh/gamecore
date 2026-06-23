// 地图模块入口
// 提供双网格地图系统，包含方形网格和等距网格两种实现

mod dual_grid;
mod dual_grid_iso;
mod gd_map_basic;
mod gd_map_isometric;

pub use dual_grid::{
    TerrainType, TerrainRegistry, TerrainThresholds, TerrainThresholdEntry,
    PropConfig, PropPlacement,
    DualGrid, place_props,
};
pub use dual_grid_iso::{
    IsoTerrainType, IsoTerrainRegistry, IsoTerrainThresholds, IsoTerrainThresholdEntry,
    IsoPropConfig, IsoPropPlacement,
    DualGridIso, iso_place_props,
};
pub use gd_map_basic::GdMapBasic;
pub use gd_map_isometric::GdMapIsometric;
