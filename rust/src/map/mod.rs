// 地图模块入口
// 提供双网格地图系统，包含方形网格和等距网格两种实现

mod crop_def;
mod dual_grid;
mod gd_action_cursor;
mod gd_dig_layer;
mod gd_map_basic;
mod gd_map_deco;
mod gd_map_manager;
mod gd_map_marker;
mod gd_minimap;
mod quick_map;
mod terrain_tex;

pub use dual_grid::{
    TerrainType, TerrainRegistry, TerrainThresholds, TerrainThresholdEntry,
    PropConfig, PropPlacement,
    DualGrid, place_props,
};

pub use crop_def::GdCropDef;
pub use gd_action_cursor::GdActionCursor;
pub use gd_dig_layer::GdDigLayer;
pub use gd_map_basic::GdMapBasic;
pub use gd_map_deco::GdDecoLayer;
pub use gd_map_manager::GdMapManager;
pub use gd_map_marker::GdMapMarker;
pub use gd_minimap::GdMiniMap;
pub use quick_map::GdQuickMap;
