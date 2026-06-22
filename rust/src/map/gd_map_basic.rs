// GdMapBasic - 双网格地图节点（继承 TileMapLayer）
// 提供双网格地形过渡、噪声地图生成、资源配置等封装方法
// 支持动态注册地形，不再硬编码地形类型
// 内部结构：TileMapLayer (GdMapBasic, 自身即世界层, self_modulate 透明)
//   ├── DisplayLayer_xxx (TileMapLayer, 半格偏移, 每种地形一个, 支持 priority 排序)
//   └── PropLayer (TileMapLayer)

use godot::prelude::*;
use godot::classes::{ITileMapLayer, TileMapLayer, TileSet};
use godot::builtin::Vector2i;

use std::collections::HashMap;

use super::dual_grid::{
    DualGrid, TerrainType, TerrainRegistry, TerrainThresholds, TerrainThresholdEntry,
    PropConfig, place_props,
};

/// 资源配置（从 JSON 加载）
#[derive(Debug, Clone)]
struct ResourceConfig {
    /// 地形配置：地形名 → (atlas_coord, source_id)
    terrain_atlas: HashMap<String, (Vector2i, i32)>,
    /// 显示层配置：地形名 → (source_id, priority)
    display_layers: HashMap<String, DisplayLayerConfig>,
    /// 资源配置列表
    props: Vec<PropConfig>,
}

/// 显示层配置
#[derive(Debug, Clone)]
struct DisplayLayerConfig {
    /// TileSet source_id
    source_id: i32,
    /// 渲染优先级，数值越大越在上面
    priority: i32,
}

#[derive(GodotClass)]
#[class(base = TileMapLayer)]
pub struct GdMapBasic {
    base: Base<TileMapLayer>,

    /// 双网格算法核心
    dual_grid: DualGrid,

    /// 地形注册表
    terrain_registry: TerrainRegistry,

    /// 资源配置
    resource_config: Option<ResourceConfig>,

    /// 显示层子节点引用
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
impl ITileMapLayer for GdMapBasic {
    fn init(base: Base<TileMapLayer>) -> Self {
        Self {
            base,
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
        // 自身即世界层，设置 self_modulate 为透明（alpha=0），不使用 hide()
        // 这样编辑器中仍然可以看到图集配置，且不影响子节点渲染
        self.base_mut().set_self_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));
        self.ensure_display_layers();
    }
}

#[godot_api]
impl GdMapBasic {
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
        self.ensure_display_layers();
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
        self.ensure_display_layers();
        true
    }

    /// 设置世界格子（通过 atlas_coords）
    #[func]
    fn set_tile(&mut self, coords: Vector2i, atlas_coords: Vector2i) {
        if !self.can_set_tile {
            return;
        }

        let pos = (coords.x, coords.y);
        let atlas = (atlas_coords.x, atlas_coords.y);

        let terrain = self.atlas_to_terrain(atlas);
        self.dual_grid.set_world_tile(pos, terrain);

        // 设置世界层格子（自身即世界层，self_modulate 透明）
        let source_id = self.get_terrain_source_id(terrain);
        self.base_mut()
            .set_cell_ex(coords)
            .source_id(source_id)
            .atlas_coords(atlas_coords)
            .done();

        self.update_display_for_world_pos(pos);
    }

    /// 通过地形类型设置格子
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

        let (atlas, source_id) = self.get_terrain_atlas_and_source(&terrain);
        self.base_mut()
            .set_cell_ex(coords)
            .source_id(source_id)
            .atlas_coords(Vector2i::new(atlas.0, atlas.1))
            .done();

        self.update_display_for_world_pos(pos);
    }

    /// 擦除世界格子
    #[func]
    fn erase_tile(&mut self, coords: Vector2i) {
        let pos = (coords.x, coords.y);
        self.dual_grid.erase_world_tile(pos);
        self.base_mut().erase_cell(coords);
        self.update_display_for_world_pos(pos);
    }

    /// 获取指定坐标的地形类型
    #[func]
    fn get_terrain_type(&self, coords: Vector2i) -> i32 {
        self.dual_grid
            .get_world_tile((coords.x, coords.y))
            .to_i32()
    }

    /// 使用噪声生成指定大小的随机地图
    #[func]
    fn generate_map(&mut self, width: i32, height: i32, seed: i64) {
        self.clear_map();
        self.sync_tile_set_to_layers();

        let noise_values = self.generate_noise_values(width, height, seed);
        let terrains = DualGrid::generate_terrain_from_noise(
            width, height, &noise_values, &self.thresholds,
        );

        for (pos, terrain) in &terrains {
            self.dual_grid.set_world_tile(*pos, *terrain);
            let (atlas, source_id) = self.get_terrain_atlas_and_source(terrain);
            self.base_mut()
                .set_cell_ex(Vector2i::new(pos.0, pos.1))
                .source_id(source_id)
                .atlas_coords(Vector2i::new(atlas.0, atlas.1))
                .done();
        }

        self.refresh_all_display_tiles();
    }

    /// 使用噪声生成带资源的随机地图
    #[func]
    fn generate_map_with_resources(&mut self, width: i32, height: i32, seed: i64) {
        self.clear_map();
        self.sync_tile_set_to_layers();

        let noise_values = self.generate_noise_values(width, height, seed);
        let terrains = DualGrid::generate_terrain_from_noise(
            width, height, &noise_values, &self.thresholds,
        );

        for (pos, terrain) in &terrains {
            self.dual_grid.set_world_tile(*pos, *terrain);
            let (atlas, source_id) = self.get_terrain_atlas_and_source(terrain);
            self.base_mut()
                .set_cell_ex(Vector2i::new(pos.0, pos.1))
                .source_id(source_id)
                .atlas_coords(Vector2i::new(atlas.0, atlas.1))
                .done();
        }

        self.refresh_all_display_tiles();
        self.place_props_from_config(&noise_values, &terrains, seed);
    }

    /// 从自身 TileMapLayer 上已画的地块生成地图（编辑器手动绘制模式）
    /// 读取自身已放置的格子，通过 atlas_coords 反查地形类型，填充双网格并刷新显示层
    /// 同时根据噪声值和概率放置资源
    #[func]
    fn generate_map_from_tiles(&mut self) {
        self.sync_tile_set_to_layers();

        let used_cells: Array<Vector2i> = self.base().get_used_cells();

        // 收集已画地块的地形信息
        let mut terrains: HashMap<(i32, i32), TerrainType> = HashMap::new();
        let mut max_x = 0i32;
        let mut max_y = 0i32;

        for cell in used_cells.iter_shared() {
            let atlas_coords = self.base().get_cell_atlas_coords(cell);
            let atlas = (atlas_coords.x, atlas_coords.y);
            let terrain = self.atlas_to_terrain(atlas);

            if terrain.is_null() {
                continue;
            }

            let pos = (cell.x, cell.y);
            self.dual_grid.set_world_tile(pos, terrain);
            terrains.insert(pos, terrain);
            max_x = max_x.max(cell.x);
            max_y = max_y.max(cell.y);
        }

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
        // 清除世界层（自身）
        {
            let mut base = self.base_mut();
            let used: Array<Vector2i> = base.get_used_cells();
            for cell in used.iter_shared() {
                base.erase_cell(cell);
            }
        }

        // 清除显示层
        for layer in &self.display_layers {
            let mut layer = layer.clone();
            let used = layer.get_used_cells();
            for cell in used.iter_shared() {
                layer.erase_cell(cell);
            }
        }

        // 清除资源层
        if let Some(ref mut layer) = self.prop_layer {
            let used = layer.get_used_cells();
            for cell in used.iter_shared() {
                layer.erase_cell(cell);
            }
        }

        self.dual_grid = DualGrid::new();
    }

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

    /// 动态添加地形集配置
    #[func]
    fn add_terrain_config(
        &mut self,
        terrain_name: String,
        atlas_x: i32,
        atlas_y: i32,
        world_source_id: i32,
        display_source_id: i32,
        priority: i32,
    ) {
        // 确保地形已注册
        self.terrain_registry.register(&terrain_name);

        if self.resource_config.is_none() {
            self.resource_config = Some(ResourceConfig {
                terrain_atlas: HashMap::new(),
                display_layers: HashMap::new(),
                props: Vec::new(),
            });
        }

        if let Some(ref mut config) = self.resource_config {
            config.terrain_atlas.insert(
                terrain_name.clone(),
                (Vector2i::new(atlas_x, atlas_y), world_source_id),
            );
            config.display_layers.insert(terrain_name, DisplayLayerConfig {
                source_id: display_source_id,
                priority,
            });
        }

        self.ensure_display_layers();
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
                terrain_atlas: HashMap::new(),
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

    /// 设置 TileSet（自身即世界层，和显示层共用）
    #[func]
    fn set_tile_set(&mut self, tile_set: Gd<TileSet>) {
        self.base_mut().set_tile_set(&tile_set);
        self.sync_tile_set_to_layers();
    }
}

// 内部实现方法
impl GdMapBasic {
    /// 解析 JSON 资源配置
    fn parse_resource_config(
        json: &serde_json::Value,
        registry: &mut TerrainRegistry,
    ) -> Result<ResourceConfig, String> {
        let mut terrain_atlas = HashMap::new();
        let mut display_layers = HashMap::new();
        let mut props = Vec::new();

        // 解析 terrains
        if let Some(terrains) = json.get("terrains").and_then(|t| t.as_object()) {
            for (name, value) in terrains {
                // 注册地形
                registry.register(name);

                let atlas_coord = value
                    .get("atlas_coord")
                    .or_else(|| value.get("atlas_coords"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        let x = arr.first().and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let y = arr.get(1).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        Vector2i::new(x, y)
                    })
                    .ok_or_else(|| format!("terrain '{}' 缺少 atlas_coord", name))?;

                let source_id = value
                    .get("source_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                terrain_atlas.insert(name.clone(), (atlas_coord, source_id));
            }
        }

        // 解析 display_layers（支持 priority 字段）
        if let Some(layers) = json.get("display_layers").and_then(|l| l.as_object()) {
            for (name, value) in layers {
                let source_id = value
                    .get("source_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                let priority = value
                    .get("priority")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                display_layers.insert(name.clone(), DisplayLayerConfig {
                    source_id,
                    priority,
                });
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
            terrain_atlas,
            display_layers,
            props,
        })
    }

    /// 确保显示层子节点已创建（按 priority 排序，低 priority 在下）
    fn ensure_display_layers(&mut self) {
        // 收集显示层配置并按 priority 排序
        let sorted_layers: Vec<(String, DisplayLayerConfig)> = if let Some(ref config) = self.resource_config {
            let mut entries: Vec<(String, DisplayLayerConfig)> = config.display_layers
                .iter()
                .map(|(name, cfg)| (name.clone(), cfg.clone()))
                .collect();
            entries.sort_by_key(|(_, cfg)| cfg.priority);
            entries
        } else {
            self.terrain_registry.get_all_names()
                .into_iter()
                .enumerate()
                .map(|(i, name)| (name, DisplayLayerConfig {
                    source_id: 0,
                    priority: i as i32,
                }))
                .collect()
        };

        let existing_count = self.display_layers.len();
        if existing_count >= sorted_layers.len() {
            // 即使数量足够，也需要更新 z_index
            for (i, layer) in self.display_layers.iter().enumerate() {
                let mut layer = layer.clone();
                layer.set_z_index(i as i32);
            }
            return;
        }

        // 收集需要创建的层信息，避免借用冲突
        let new_layers: Vec<(usize, String, i32)> = sorted_layers
            .iter()
            .enumerate()
            .skip(existing_count)
            .map(|(i, (name, cfg))| (i, name.clone(), cfg.priority))
            .collect();

        for (i, name, _priority) in new_layers {
            let mut layer = TileMapLayer::new_alloc();
            layer.set_name(&format!("DisplayLayer_{}", name));
            layer.set_z_index(i as i32);
            self.base_mut().add_child(&layer);
            self.display_layers.push(layer);
        }

        // 同步 TileSet 到显示层
        self.sync_tile_set_to_layers();
    }

    /// 确保资源层子节点已创建
    fn ensure_prop_layer(&mut self) {
        if self.prop_layer.is_some() {
            return;
        }

        let mut layer = TileMapLayer::new_alloc();
        layer.set_name("PropLayer");
        layer.set_z_index(100);
        self.base_mut().add_child(&layer);
        self.prop_layer = Some(layer);

        // 同步 TileSet 到资源层
        self.sync_tile_set_to_layers();
    }

    /// 将自身的 TileSet 同步到所有显示层和资源层
    /// 同时设置显示层的半格偏移（双网格系统中显示格子在四个世界格子的交叉点上）
    fn sync_tile_set_to_layers(&mut self) {
        let tile_set = self.base().get_tile_set();
        if let Some(ref ts) = tile_set {
            let tile_size = ts.get_tile_size();
            let half_x = tile_size.x as f32 / 2.0;
            let half_y = tile_size.y as f32 / 2.0;

            for layer in &self.display_layers {
                let mut layer = layer.clone();
                layer.set_tile_set(ts);
                // 显示层偏移半个格子
                layer.set_position(-Vector2::new(half_x, half_y));
            }
            if let Some(ref mut layer) = self.prop_layer {
                layer.set_tile_set(ts);
            }
        }
    }

    /// 根据 atlas_coords 判断地形类型
    fn atlas_to_terrain(&self, atlas: (i32, i32)) -> TerrainType {
        if let Some(ref config) = self.resource_config {
            for (name, (atlas_coord, _)) in &config.terrain_atlas {
                if atlas_coord.x == atlas.0 && atlas_coord.y == atlas.1 {
                    return self.terrain_registry
                        .get_id(name)
                        .unwrap_or(TerrainType::NULL);
                }
            }
        }
        TerrainType::NULL
    }

    /// 获取地形在世界层的 source_id
    fn get_terrain_source_id(&self, terrain: TerrainType) -> i32 {
        if let Some(ref config) = self.resource_config {
            if let Some(name) = self.terrain_registry.get_name(terrain) {
                return config
                    .terrain_atlas
                    .get(name)
                    .map(|(_, sid)| *sid)
                    .unwrap_or(0);
            }
        }
        0
    }

    /// 获取地形的 atlas_coord 和 source_id
    fn get_terrain_atlas_and_source(&self, terrain: &TerrainType) -> ((i32, i32), i32) {
        if let Some(ref config) = self.resource_config {
            if let Some(name) = self.terrain_registry.get_name(*terrain) {
                if let Some((atlas, source_id)) = config.terrain_atlas.get(name) {
                    return ((atlas.x, atlas.y), *source_id);
                }
            }
        }
        ((0, 0), 0)
    }

    /// 获取地形在显示层的 source_id
    fn get_display_source_id(&self, terrain: TerrainType) -> i32 {
        if let Some(ref config) = self.resource_config {
            if let Some(name) = self.terrain_registry.get_name(terrain) {
                return config.display_layers.get(name).map(|cfg| cfg.source_id).unwrap_or(0);
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

    /// 更新单个显示格子
    /// 低优先级图层（z_index 较小）在边界处额外渲染1格邻居，避免连接处露出背景
    fn update_single_display_tile(&mut self, display_pos: (i32, i32)) {
        let sorted_layers = self.get_sorted_display_layer_info();

        for (i, _name, is_low_priority) in &sorted_layers {
            if *i >= self.display_layers.len() {
                break;
            }

            let terrain_id = self.terrain_registry
                .get_id(_name)
                .unwrap_or(TerrainType::NULL);

            let source_id = self.get_display_source_id(terrain_id);

            // 渲染主格子
            let atlas = self.dual_grid.calculate_display_tile(display_pos, terrain_id);
            let mut layer = self.display_layers[*i].clone();
            layer
                .set_cell_ex(Vector2i::new(display_pos.0, display_pos.1))
                .source_id(source_id)
                .atlas_coords(Vector2i::new(atlas.0, atlas.1))
                .done();

            // 低优先级图层额外渲染周围1格邻居，避免边界漏出背景
            if *is_low_priority {
                for neighbor in &[(0, -1), (0, 1), (-1, 0), (1, 0)] {
                    let neighbor_pos = (display_pos.0 + neighbor.0, display_pos.1 + neighbor.1);
                    let neighbor_atlas = self.dual_grid.calculate_display_tile(neighbor_pos, terrain_id);
                    layer
                        .set_cell_ex(Vector2i::new(neighbor_pos.0, neighbor_pos.1))
                        .source_id(source_id)
                        .atlas_coords(Vector2i::new(neighbor_atlas.0, neighbor_atlas.1))
                        .done();
                }
            }
        }
    }

    /// 获取按 priority 排序的显示层信息：(display_layers索引, 地形名, 是否低优先级)
    /// 低优先级定义：priority 低于最高 priority 的图层
    fn get_sorted_display_layer_info(&self) -> Vec<(usize, String, bool)> {
        if let Some(ref config) = self.resource_config {
            let mut entries: Vec<(String, i32)> = config.display_layers
                .iter()
                .map(|(name, cfg)| (name.clone(), cfg.priority))
                .collect();
            entries.sort_by_key(|(_, p)| *p);

            let max_priority = entries.iter().map(|(_, p)| *p).max().unwrap_or(0);

            entries.into_iter()
                .enumerate()
                .map(|(i, (name, priority))| (i, name, priority < max_priority))
                .collect()
        } else {
            self.terrain_registry.get_all_names()
                .into_iter()
                .enumerate()
                .map(|(i, name)| (i, name, i > 0)) // 无配置时，第一个最低，其余低优先级
                .collect()
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

        self.ensure_prop_layer();

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
