// 地图模块入口
// 提供双网格地图系统，包含地形过渡、噪声生成、资源配置等功能

mod dual_grid;
mod gd_map_basic;

pub use dual_grid::{
    TerrainType, TerrainRegistry, TerrainThresholds, TerrainThresholdEntry,
    PropConfig, PropPlacement,
    DualGrid, place_props,
};
pub use gd_map_basic::GdMapBasic;
