// suite: map - 双网格地图核心逻辑测试
// 覆盖: 地形注册、世界网格读写、16 角组合查表、擦除

use crate::dev::framework::TestContext;
use crate::dev::framework;
use crate::map::{DualGrid, TerrainRegistry, TerrainType};

fn terrain_registry(ctx: &mut TestContext) {
	let mut reg = TerrainRegistry::new();

	let grass = reg.register("grass");
	let mud = reg.register("mud");

	ctx.assert_ne(grass, TerrainType::NULL, "注册后的 grass 不应是 NULL 地形");
	ctx.assert_ne(mud, TerrainType::NULL, "注册后的 mud 不应是 NULL 地形");
	ctx.assert_ne(grass, mud, "不同地形应得到不同 ID");

	// 重复注册应返回同一 ID
	ctx.assert_eq(reg.register("grass"), grass, "重复注册同名地形应返回相同 ID");

	ctx.assert_eq(reg.get_id("mud"), Some(mud), "get_id 应能查回 mud");
	ctx.assert_eq(reg.get_name(grass), Some("grass"), "get_name 应能查回 grass");
	ctx.assert_true(reg.get_id("unknown").is_none(), "未注册地形查询应返回 None");
	ctx.assert_true(reg.get_all_names().contains(&"grass".to_string()), "地形名列表应包含 grass");

	ctx.assert_true(TerrainType::NULL.is_null(), "NULL 地形应标记为 null");
	ctx.assert_eq(TerrainType::from_i32(-5).to_i32(), 0, "负数地形 ID 应钳制为 0");
}

fn world_grid_read_write(ctx: &mut TestContext) {
	let mut grid = DualGrid::new();
	let grass = TerrainType(1);
	let mud = TerrainType(2);

	grid.set_world_tile((0, 0), grass);
	grid.set_world_tile((1, 0), grass);
	grid.set_world_tile((0, 1), mud);

	ctx.assert_true(grid.has_terrain_at((0, 0), grass), "(0,0) 应有 grass");
	ctx.assert_false(grid.has_terrain_at((1, 1), grass), "(1,1) 不应有 grass");
	ctx.assert_true(grid.has_terrain_at((0, 1), mud), "(0,1) 应有 mud");

	// 同一坐标可属于多个地形
	grid.set_world_tile((0, 0), mud);
	let terrains = grid.get_terrains_at((0, 0));
	ctx.assert_eq(terrains.len(), 2, "(0,0) 应同时具有 2 种地形");

	let used = grid.get_used_cells();
	ctx.assert_eq(used.len(), 3, "世界网格已用格子数应为 3");

	let grass_cells = grid.get_cells_for_terrain(grass);
	ctx.assert_eq(grass_cells.len(), 2, "grass 地形应有 2 个格子");
}

fn display_tile_lookup(ctx: &mut TestContext) {
	let mut grid = DualGrid::new();
	let grass = TerrainType(1);

	// 单格 grass 位于 (0,0)：显示格 (0,0) 的右下角为 grass，其余为 null
	grid.set_world_tile((0, 0), grass);
	let coord = grid.calculate_display_tile((0, 0), grass);
	ctx.assert_eq(coord, (1, 3), "仅右下角有草应对应图集坐标 (1,3)");

	// 左上角: 显示格 (1,1) 的 up_left 是世界格 (0,0)
	let coord = grid.calculate_display_tile((1, 1), grass);
	ctx.assert_eq(coord, (3, 3), "仅左上角有草应对应图集坐标 (3,3)");

	// 2x2 全草：显示格 (1,1) 四角均为 grass
	grid.set_world_tile((1, 0), grass);
	grid.set_world_tile((0, 1), grass);
	grid.set_world_tile((1, 1), grass);
	let coord = grid.calculate_display_tile((1, 1), grass);
	ctx.assert_eq(coord, (2, 1), "四角全草应对应图集坐标 (2,1)");

	// 非目标地形查询: 以 mud 查询全草显示格应落到默认值 (0,3)
	let mud = TerrainType(2);
	let coord = grid.calculate_display_tile((1, 1), mud);
	ctx.assert_eq(coord, (0, 3), "无 mud 角的显示格应回落到默认全泥坐标");
}

fn erase_tiles(ctx: &mut TestContext) {
	let mut grid = DualGrid::new();
	let grass = TerrainType(1);

	grid.set_world_tile((3, 3), grass);
	ctx.assert_true(grid.has_terrain_at((3, 3), grass), "写入后应有 grass");

	grid.erase_world_tile((3, 3), grass);
	ctx.assert_false(grid.has_terrain_at((3, 3), grass), "擦除后不应有 grass");

	// erase_coord 应清除该坐标全部地形
	grid.set_world_tile((4, 4), grass);
	grid.set_world_tile((4, 4), TerrainType(2));
	grid.erase_coord((4, 4));
	ctx.assert_eq(grid.get_terrains_at((4, 4)).len(), 0, "erase_coord 后该坐标应无任何地形");

	// NULL 地形写入应被忽略
	grid.set_world_tile((5, 5), TerrainType::NULL);
	ctx.assert_eq(grid.get_terrains_at((5, 5)).len(), 0, "NULL 地形不应被写入");
}

pub fn register() {
	framework::register("map", "terrain_registry", terrain_registry);
	framework::register("map", "world_grid_read_write", world_grid_read_write);
	framework::register("map", "display_tile_lookup", display_tile_lookup);
	framework::register("map", "erase_tiles", erase_tiles);
}
