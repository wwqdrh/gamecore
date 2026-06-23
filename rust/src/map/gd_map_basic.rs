// GdMapBasic - 双网格地图节点（继承 Node2D）
// 提供双网格地形过渡、噪声地图生成、资源配置等封装方法
// 支持动态注册地形，不再硬编码地形类型
// 内部结构：Node2D (GdMapBasic)
//   ├── TileMapLayer (子节点, 按节点名匹配地形, 半格偏移显示过渡贴图)
//   ├── TileMapLayer (子节点, ...)
//   └── PropLayer (TileMapLayer, 资源层)
// 子节点在场景文件中预定义，Rust 代码直接处理而非动态创建

use godot::prelude::*;
use godot::classes::{INode2D, Node2D, TileMapLayer, TileSet};
use godot::builtin::Vector2i;

use std::collections::HashMap;

use super::dual_grid::{
    DualGrid, TerrainType, TerrainRegistry, TerrainThresholds, TerrainThresholdEntry,
    PropConfig, place_props,
};

/// 资源配置（从 JSON 加载）
#[derive(Debug, Clone)]
struct ResourceConfig {
    /// 显示层配置：地形名 → source_id（过渡贴图所在的 TileSet source）
    display_layers: HashMap<String, i32>,
    /// 资源配置列表
    props: Vec<PropConfig>,
}

#[derive(GodotClass)]
#[class(base = Node2D, tool)]
pub struct GdMapBasic {
    base: Base<Node2D>,

    /// TileSet 资源（同步到所有子层）
    #[var(get = get_tile_set, set = set_tile_set)]
    tile_set: Option<Gd<TileSet>>,

    /// 双网格算法核心
    dual_grid: DualGrid,

    /// 地形注册表
    terrain_registry: TerrainRegistry,

    /// 资源配置
    resource_config: Option<ResourceConfig>,

    /// 显示层子节点引用（按子节点顺序，层级关系由节点顺序控制）
    display_layers: Vec<Gd<TileMapLayer>>,

    /// 资源层子节点引用
    prop_layer: Option<Gd<TileMapLayer>>,

    /// 是否可以设置格子
    #[var]
    can_set_tile: bool,

    /// 地形阈值
    thresholds: TerrainThresholds,
}

#[godot_api]
impl INode2D for GdMapBasic {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            tile_set: None,
            dual_grid: DualGrid::new(),
            terrain_registry: TerrainRegistry::new(),
            resource_config: None,
            display_layers: Vec::new(),
            prop_layer: None,
            can_set_tile: true,
            thresholds: TerrainThresholds::default(),
        }
    }

    fn ready(&mut self) {
        self.scan_child_layers();
        self.sync_tile_set_to_layers();
    }
}

#[godot_api]
impl GdMapBasic {
    // ---- tile_set 属性 getter/setter ----

    #[func]
    fn get_tile_set(&self) -> Option<Gd<TileSet>> {
        self.tile_set.clone()
    }

    #[func]
    fn set_tile_set(&mut self, value: Option<Gd<TileSet>>) {
        self.tile_set = value;
        if !self.display_layers.is_empty() {
            self.sync_tile_set_to_layers();
        }
    }

    // ---- 地形注册 ----

    /// 注册地形类型，返回分配的 ID。若已存在则返回已有 ID
    #[func]
    fn register_terrain(&mut self, name: GString) -> i32 {
        let id = self.terrain_registry.register(&name.to_string());
        id.to_i32()
    }

    /// 通过名称获取地形 ID，不存在返回 0
    #[func]
    fn get_terrain_id(&self, name: GString) -> i32 {
        self.terrain_registry
            .get_id(&name.to_string())
            .map(|id| id.to_i32())
            .unwrap_or(0)
    }

    /// 通过 ID 获取地形名称，不存在返回空字符串
    #[func]
    fn get_terrain_name(&self, terrain_id: i32) -> GString {
        let id = TerrainType::from_i32(terrain_id);
        self.terrain_registry
            .get_name(id)
            .map(|s| GString::from(s))
            .unwrap_or_default()
    }

    /// 获取所有已注册地形名称列表
    #[func]
    fn get_all_terrain_names(&self) -> PackedStringArray {
        let mut arr = PackedStringArray::new();
        for name in self.terrain_registry.get_all_names() {
            arr.push(name.as_str());
        }
        arr
    }

    // ---- 资源配置 ----

    /// 从 JSON 文件加载资源集配置
    #[func]
    fn load_resource_config(&mut self, json_path: String) -> bool {
        let Ok(file) = std::fs::read_to_string(&json_path) else {
            godot_error!("GdMapBasic: 无法读取配置文件: {}", json_path);
            return false;
        };

        let Ok(json) = serde_json::from_str::<serde_json::Value>(&file) else {
            godot_error!("GdMapBasic: JSON 解析失败: {}", json_path);
            return false;
        };

        let config = match Self::parse_resource_config(&json, &mut self.terrain_registry) {
            Ok(c) => c,
            Err(e) => {
                godot_error!("GdMapBasic: 配置解析失败: {}", e);
                return false;
            }
        };

        self.resource_config = Some(config);
        true
    }

    /// 从 JSON 字符串加载资源集配置
    #[func]
    fn load_resource_config_from_string(&mut self, json_string: String) -> bool {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_string) else {
            godot_error!("GdMapBasic: JSON 解析失败");
            return false;
        };

        let config = match Self::parse_resource_config(&json, &mut self.terrain_registry) {
            Ok(c) => c,
            Err(e) => {
                godot_error!("GdMapBasic: 配置解析失败: {}", e);
                return false;
            }
        };

        self.resource_config = Some(config);
        true
    }

    // ---- 格子操作 ----

    /// 通过地形类型设置格子（更新 DualGrid 世界数据 + 刷新显示贴图）
    #[func]
    fn set_terrain(&mut self, coords: Vector2i, terrain_type: i32) {
        if !self.can_set_tile {
            return;
        }

        let pos = (coords.x, coords.y);
        let terrain = TerrainType::from_i32(terrain_type);

        if terrain.is_null() {
            self.erase_tile(coords);
            return;
        }

        self.dual_grid.set_world_tile(pos, terrain);
        self.update_display_for_world_pos(pos);
    }

    /// 擦除世界格子
    #[func]
    fn erase_tile(&mut self, coords: Vector2i) {
        let pos = (coords.x, coords.y);
        self.dual_grid.erase_world_tile(pos);
        self.update_display_for_world_pos(pos);
    }

    /// 获取指定坐标的地形类型
    #[func]
    fn get_terrain_type(&self, coords: Vector2i) -> i32 {
        self.dual_grid
            .get_world_tile((coords.x, coords.y))
            .to_i32()
    }

    // ---- 地图生成 ----

    /// 使用噪声生成指定大小的随机地图
    #[func]
    fn generate_map(&mut self, width: i32, height: i32, seed: i64) {
        self.clear_map();
        self.scan_child_layers();
        self.sync_tile_set_to_layers();
        self.apply_display_offset();

        let noise_values = self.generate_noise_values(width, height, seed);
        let terrains = DualGrid::generate_terrain_from_noise(
            width, height, &noise_values, &self.thresholds,
        );

        for (pos, terrain) in &terrains {
            self.dual_grid.set_world_tile(*pos, *terrain);
        }

        self.refresh_all_display_tiles();
    }

    /// 使用噪声生成带资源的随机地图
    #[func]
    fn generate_map_with_resources(&mut self, width: i32, height: i32, seed: i64) {
        self.clear_map();
        self.scan_child_layers();
        self.sync_tile_set_to_layers();
        self.apply_display_offset();

        let noise_values = self.generate_noise_values(width, height, seed);
        let terrains = DualGrid::generate_terrain_from_noise(
            width, height, &noise_values, &self.thresholds,
        );

        for (pos, terrain) in &terrains {
            self.dual_grid.set_world_tile(*pos, *terrain);
        }

        self.refresh_all_display_tiles();
        self.place_props_from_config(&noise_values, &terrains, seed);
    }

    /// 从子 TileMapLayer 层上已画的占位符生成地图（编辑器手动绘制模式）
    /// 遍历子层，通过层名匹配地形类型，读取占位格子填充双网格
    /// 然后清除占位符、应用半格偏移、绘制显示过渡贴图
    #[func]
    fn generate_map_from_tiles(&mut self) {
        self.scan_child_layers();
        self.sync_tile_set_to_layers();

        // 从子层读取占位符（世界坐标）
        let mut terrains: HashMap<(i32, i32), TerrainType> = HashMap::new();
        let mut max_x = 0i32;
        let mut max_y = 0i32;

        let layers: Vec<Gd<TileMapLayer>> = self.display_layers.iter().cloned().collect();
        for layer in &layers {
            let layer_name = layer.get_name().to_string();

            let terrain = match self.terrain_registry.get_id(&layer_name) {
                Some(t) => t,
                None => continue,
            };

            let used_cells: Array<Vector2i> = layer.get_used_cells();
            for cell in used_cells.iter_shared() {
                let pos = (cell.x, cell.y);
                self.dual_grid.set_world_tile(pos, terrain);
                terrains.insert(pos, terrain);
                max_x = max_x.max(cell.x);
                max_y = max_y.max(cell.y);
            }
        }

        // 清除子层占位符
        self.clear_child_layers();

        // 应用半格偏移并绘制显示贴图
        self.apply_display_offset();
        self.refresh_all_display_tiles();

        // 放置资源：根据地块范围生成噪声值
        if !terrains.is_empty() {
            let width = max_x + 1;
            let height = max_y + 1;
            let noise_values = self.generate_noise_values(width, height, 0);
            self.place_props_from_config(&noise_values, &terrains, 0);
        }
    }

    /// 清除整个地图
    #[func]
    fn clear_map(&mut self) {
        self.clear_child_layers();
        self.dual_grid = DualGrid::new();
    }

    // ---- 阈值与查询 ----

    /// 设置地形阈值参数（动态地形版本）
    /// terrain_names 和 threshold_maxs 长度必须一致，按顺序对应
    #[func]
    fn set_thresholds(
        &mut self,
        terrain_names: PackedStringArray,
        threshold_maxs: PackedFloat64Array,
    ) {
        let mut entries = Vec::new();
        for (name, max_val) in terrain_names.as_slice().iter().zip(threshold_maxs.as_slice().iter()) {
            if let Some(id) = self.terrain_registry.get_id(&name.to_string()) {
                entries.push(TerrainThresholdEntry {
                    terrain_id: id,
                    max_value: *max_val,
                });
            }
        }
        self.thresholds = TerrainThresholds { entries };
    }

    /// 获取已使用的世界格子列表
    #[func]
    fn get_used_terrain_cells(&self) -> Array<Vector2i> {
        let mut arr = Array::new();
        for (x, y) in self.dual_grid.get_used_cells() {
            arr.push(Vector2i::new(x, y));
        }
        arr
    }

    /// 刷新所有显示格子
    #[func]
    fn refresh_display(&mut self) {
        self.refresh_all_display_tiles();
    }

    // ---- 动态配置 ----

    /// 动态添加地形配置
    /// terrain_name: 地形名称（需与子层节点名一致）
    /// display_source_id: 显示过渡贴图所在的 TileSet source_id
    #[func]
    fn add_terrain_config(
        &mut self,
        terrain_name: String,
        display_source_id: i32,
    ) {
        self.terrain_registry.register(&terrain_name);

        if self.resource_config.is_none() {
            self.resource_config = Some(ResourceConfig {
                display_layers: HashMap::new(),
                props: Vec::new(),
            });
        }

        if let Some(ref mut config) = self.resource_config {
            config.display_layers.insert(terrain_name, display_source_id);
        }
    }

    /// 动态添加资源配置
    #[func]
    fn add_prop_config(
        &mut self,
        name: String,
        source_id: i32,
        alternative_tile: i32,
        probability: f64,
        allowed_terrains: PackedStringArray,
        noise_min: f64,
        noise_max: f64,
    ) {
        if self.resource_config.is_none() {
            self.resource_config = Some(ResourceConfig {
                display_layers: HashMap::new(),
                props: Vec::new(),
            });
        }

        if let Some(ref mut config) = self.resource_config {
            let terrains: Vec<TerrainType> = allowed_terrains
                .as_slice()
                .iter()
                .filter_map(|s| {
                    self.terrain_registry.get_id(&s.to_string())
                })
                .collect();

            config.props.push(PropConfig {
                name: name.to_string(),
                source_id,
                alternative_tile,
                probability,
                allowed_terrains: terrains,
                noise_range: (noise_min, noise_max),
            });
        }
    }
}

// ---- 内部实现方法 ----
impl GdMapBasic {
    /// 解析 JSON 资源配置
    fn parse_resource_config(
        json: &serde_json::Value,
        registry: &mut TerrainRegistry,
    ) -> Result<ResourceConfig, String> {
        let mut display_layers = HashMap::new();
        let mut props = Vec::new();

        // 解析 terrains（仅注册地形名称，无世界层配置）
        if let Some(terrains) = json.get("terrains").and_then(|t| t.as_object()) {
            for (name, _value) in terrains {
                registry.register(name);
            }
        }

        // 解析 display_layers（仅 source_id，无 priority）
        if let Some(layers) = json.get("display_layers").and_then(|l| l.as_object()) {
            for (name, value) in layers {
                registry.register(name);
                let source_id = value
                    .get("source_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                display_layers.insert(name.clone(), source_id);
            }
        }

        // 解析 props
        if let Some(props_arr) = json.get("props").and_then(|p| p.as_array()) {
            for prop_val in props_arr {
                let name = prop_val
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let source_id = prop_val
                    .get("source_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                let alternative_tile = prop_val
                    .get("alternative_tile")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                let probability = prop_val
                    .get("probability")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.05);
                let allowed_terrains: Vec<TerrainType> = prop_val
                    .get("allowed_terrains")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| {
                                v.as_str()
                                    .and_then(|s| registry.get_id(s))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let noise_range = prop_val
                    .get("noise_range")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        let min = arr.first()?.as_f64()?;
                        let max = arr.get(1)?.as_f64()?;
                        Some((min, max))
                    })
                    .unwrap_or((-0.3, 0.0));

                props.push(PropConfig {
                    name,
                    source_id,
                    alternative_tile,
                    probability,
                    allowed_terrains,
                    noise_range,
                });
            }
        }

        Ok(ResourceConfig {
            display_layers,
            props,
        })
    }

    /// 扫描子 TileMapLayer 节点，按节点名分类存储引用
    /// 名为 "PropLayer" 的节点作为资源层，其余作为显示层
    fn scan_child_layers(&mut self) {
        self.display_layers.clear();
        self.prop_layer = None;

        let children = self.base().get_children();
        for child in children.iter_shared() {
            if let Ok(layer) = child.clone().try_cast::<TileMapLayer>() {
                let name = layer.get_name().to_string();
                if name == "PropLayer" {
                    self.prop_layer = Some(layer);
                } else {
                    self.display_layers.push(layer);
                }
            }
        }
    }

    /// 将 tile_set 同步到所有子层
    /// set_tile_set 存在 ByValue/ByOption 类型不匹配问题，使用 call() 动态调用
    fn sync_tile_set_to_layers(&mut self) {
        let Some(ref ts) = self.tile_set else { return };
        let ts_var = ts.to_variant();

        for layer in &self.display_layers {
            let mut layer = layer.clone();
            layer.call("set_tile_set", &[ts_var.clone()]);
        }

        if let Some(ref mut layer) = self.prop_layer {
            layer.call("set_tile_set", &[ts_var]);
        }
    }

    /// 给显示层应用半格偏移（双网格系统中显示格子在四个世界格子的交叉点上）
    fn apply_display_offset(&mut self) {
        let Some(ref ts) = self.tile_set else { return };
        let tile_size = ts.get_tile_size();
        let half_x = tile_size.x as f32 / 2.0;
        let half_y = tile_size.y as f32 / 2.0;
        let offset = -Vector2::new(half_x, half_y);

        for layer in &self.display_layers {
            let mut layer = layer.clone();
            layer.set_position(offset);
        }
    }

    /// 清除所有子层的格子
    fn clear_child_layers(&mut self) {
        for layer in &self.display_layers {
            let mut layer = layer.clone();
            let used: Array<Vector2i> = layer.get_used_cells();
            for cell in used.iter_shared() {
                layer.erase_cell(cell);
            }
        }

        if let Some(ref mut layer) = self.prop_layer {
            let used = layer.get_used_cells();
            for cell in used.iter_shared() {
                layer.erase_cell(cell);
            }
        }
    }

    /// 获取地形在显示层的 source_id
    fn get_display_source_id(&self, terrain: TerrainType) -> i32 {
        if let Some(ref config) = self.resource_config {
            if let Some(name) = self.terrain_registry.get_name(terrain) {
                return config.display_layers.get(name).copied().unwrap_or(0);
            }
        }
        0
    }

    /// 更新某个世界格子位置影响的显示格子
    fn update_display_for_world_pos(&mut self, world_pos: (i32, i32)) {
        let affected = self.dual_grid.get_affected_display_positions(world_pos);
        for display_pos in affected {
            self.update_single_display_tile(display_pos);
        }
    }

    /// 更新单个显示格子：遍历所有显示层，按层名匹配地形，计算过渡贴图并绘制
    fn update_single_display_tile(&mut self, display_pos: (i32, i32)) {
        // 先克隆层引用，避免在循环中借用 self
        let layers: Vec<Gd<TileMapLayer>> = self.display_layers.iter().cloned().collect();

        for layer in layers {
            let layer_name = layer.get_name().to_string();

            let terrain_id = match self.terrain_registry.get_id(&layer_name) {
                Some(t) => t,
                None => continue,
            };

            let source_id = self.get_display_source_id(terrain_id);
            let atlas = self.dual_grid.calculate_display_tile(display_pos, terrain_id);

            let mut layer = layer;
            layer
                .set_cell_ex(Vector2i::new(display_pos.0, display_pos.1))
                .source_id(source_id)
                .atlas_coords(Vector2i::new(atlas.0, atlas.1))
                .done();
        }
    }

    /// 刷新所有显示格子
    fn refresh_all_display_tiles(&mut self) {
        let cells = self.dual_grid.get_used_cells();
        let mut all_display_positions: std::collections::HashSet<(i32, i32)> =
            std::collections::HashSet::new();

        for world_pos in &cells {
            for dp in self.dual_grid.get_affected_display_positions(*world_pos) {
                all_display_positions.insert(dp);
            }
        }

        for display_pos in all_display_positions {
            self.update_single_display_tile(display_pos);
        }
    }

    /// 生成噪声值
    fn generate_noise_values(
        &self,
        width: i32,
        height: i32,
        seed: i64,
    ) -> HashMap<(i32, i32), f64> {
        use rand::Rng;
        let mut rng: rand::rngs::StdRng = if seed == 0 {
            rand::SeedableRng::from_entropy()
        } else {
            rand::SeedableRng::seed_from_u64(seed as u64)
        };

        let grid_size = 8;
        let mut gradients: HashMap<(i32, i32), (f64, f64)> = HashMap::new();

        let grid_w = (width / grid_size + 2) as usize;
        let grid_h = (height / grid_size + 2) as usize;
        for gx in 0..grid_w {
            for gy in 0..grid_h {
                let angle: f64 = rng.r#gen::<f64>() * std::f64::consts::PI * 2.0;
                gradients.insert((gx as i32, gy as i32), (angle.cos(), angle.sin()));
            }
        }

        let mut values = HashMap::new();
        for x in 0..width {
            for y in 0..height {
                let fx = x as f64 / grid_size as f64;
                let fy = y as f64 / grid_size as f64;

                let x0 = (fx.floor() as i32).max(0);
                let y0 = (fy.floor() as i32).max(0);
                let x1 = x0 + 1;
                let y1 = y0 + 1;

                let sx = fx - x0 as f64;
                let sy = fy - y0 as f64;

                let sx = sx * sx * (3.0 - 2.0 * sx);
                let sy = sy * sy * (3.0 - 2.0 * sy);

                let g00 = gradients.get(&(x0, y0)).unwrap_or(&(0.0, 0.0));
                let g10 = gradients.get(&(x1, y0)).unwrap_or(&(0.0, 0.0));
                let g01 = gradients.get(&(x0, y1)).unwrap_or(&(0.0, 0.0));
                let g11 = gradients.get(&(x1, y1)).unwrap_or(&(0.0, 0.0));

                let d00 = g00.0 * (fx - x0 as f64) + g00.1 * (fy - y0 as f64);
                let d10 = g10.0 * (fx - x1 as f64) + g10.1 * (fy - y0 as f64);
                let d01 = g01.0 * (fx - x0 as f64) + g01.1 * (fy - y1 as f64);
                let d11 = g11.0 * (fx - x1 as f64) + g11.1 * (fy - y1 as f64);

                let v0 = d00 + sx * (d10 - d00);
                let v1 = d01 + sx * (d11 - d01);
                let value = v0 + sy * (v1 - v0);

                values.insert((x, y), value);
            }
        }

        values
    }

    /// 根据配置放置资源
    fn place_props_from_config(
        &mut self,
        noise_values: &HashMap<(i32, i32), f64>,
        terrains: &HashMap<(i32, i32), TerrainType>,
        seed: i64,
    ) {
        let prop_configs = if let Some(ref config) = self.resource_config {
            if config.props.is_empty() {
                return;
            }
            config.props.clone()
        } else {
            return;
        };

        // prop_layer 从子节点扫描，不存在则跳过
        if self.prop_layer.is_none() {
            return;
        }

        let mut max_x = 0i32;
        let mut max_y = 0i32;
        for (x, y) in terrains.keys() {
            max_x = max_x.max(*x);
            max_y = max_y.max(*y);
        }

        let mut prop_rng: rand::rngs::StdRng = if seed == 0 {
            rand::SeedableRng::from_entropy()
        } else {
            rand::SeedableRng::seed_from_u64((seed as u64).wrapping_add(12345))
        };

        let placements = place_props(
            max_x + 1,
            max_y + 1,
            noise_values,
            terrains,
            &prop_configs,
            &mut || rand::Rng::r#gen::<f64>(&mut prop_rng),
        );

        if let Some(ref mut layer) = self.prop_layer {
            for placement in placements {
                layer
                    .set_cell_ex(Vector2i::new(placement.coords.0, placement.coords.1))
                    .source_id(placement.source_id)
                    .atlas_coords(Vector2i::new(0, 0))
                    .alternative_tile(placement.alternative_tile)
                    .done();
            }
        }
    }
}
