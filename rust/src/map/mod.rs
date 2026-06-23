// 地图模块入口
// 提供双网格地图系统，包含方形网格和等距网格两种实现

mod dual_grid;
mod gd_map_basic;

pub use dual_grid::{
    TerrainType, TerrainRegistry, TerrainThresholds, TerrainThresholdEntry,
    PropConfig, PropPlacement,
    DualGrid, place_props,
};

pub use gd_map_basic::GdMapBasic;
