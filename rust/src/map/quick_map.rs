// GdQuickMap - 快速地图生成器（继承 Node2D，tool）
//
// 用于快速搭建俯视角四方向模式的地图场景：
//   - 种子驱动的多八度值噪声 + 阈值划分地形，同一 seed 必然生成同一张地图
//   - 默认地形用颜色块直接表示（零素材即可跑通玩法）；
//     在编辑器给 terrain_textures 填贴图路径即可替换为真实素材
//   - 提供格子坐标换算与 is_walkable 通行查询，配合 GdRoleMover 的
//     MODE_GRID 网格移动模式使用
//
// 图层铺底规则（paint_cumulative = true 时）：
//   每种地形的图层会把"自己 + 所有比自己更上层（下标更大）的地形"格子都铺满，
//   保证上层贴图（尤其双网格过渡片的圆角透明缺口）下方永远有图，不会露出灰底。
//   例如 grass(2) 的圆角缺口内露出的是 sand(1) 铺的底，sand 的缺口露出 water(0) 的底。
//
// 坐标约定：格子 (cx, cy) 占据世界矩形 [cx*cell, (cx+1)*cell) x [cy*cell, (cy+1)*cell)，
// 格心为 ((cx+0.5)*cell, (cy+0.5)*cell)。地图根节点位于原点。
//
// 地形定义使用并行数组（长度需一致）：
//   terrain_names     地形名（如 grass/water）
//   terrain_colors    地形默认颜色
//   terrain_textures  贴图路径（res://...，留空用颜色）
//   terrain_thresholds 噪声上界（升序，前 n-1 个生效；第 i 档: value < thresholds[i]）

use godot::prelude::*;
use godot::builtin::{Color, GString, PackedColorArray, PackedFloat64Array, PackedStringArray, Rect2, Vector2, Vector2i};
use godot::classes::{INode2D, Image, ImageTexture, Node2D, ResourceLoader, Shader, ShaderMaterial, TileMapLayer, TileSet, TileSetAtlasSource, TileSetSource, Texture2D};
use godot::classes::image::Format as ImageFormat;

use super::dual_grid::dual_grid_atlas_coord;

/// 地形数量上限（防御并行数组异常输入）
const MAX_TERRAINS: usize = 32;

/// 内置水体流动 shader 源码（terrain_shaders 填 "water_flow" 启用）
///
/// 原理：图集贴图提供静态岸线与水色打底；shader 用"世界坐标 UV"采样两层
/// 不同速度、反向滚动的程序化值噪声，叠加出流动感并在波峰处提亮高光。
///
/// 关键点：必须用世界坐标 UV，不能直接滚动图集 UV —— 图集 UV 会滚出
/// 格子边界、采样到相邻 tile（串格）。世界坐标还让过渡片与垫底块的
/// 流动相位天然连续，双网格的半格偏移也不受影响。
const WATER_FLOW_SHADER_SRC: &str = r#"
shader_type canvas_item;

uniform float strength : hint_range(0.0, 1.0) = 0.45;
uniform float scale1 = 64.0;
uniform float scale2 = 96.0;
uniform float speed1 = 0.045;
uniform float speed2 = -0.028;

varying vec2 world_pos;

float hash21(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

float vnoise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	vec2 u = f * f * (3.0 - 2.0 * f);
	float a = hash21(i);
	float b = hash21(i + vec2(1.0, 0.0));
	float c = hash21(i + vec2(0.0, 1.0));
	float d = hash21(i + vec2(1.0, 1.0));
	return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 0.0, 1.0)).xy;
}

void fragment() {
	vec2 uv1 = world_pos / scale1 + vec2(TIME * speed1, TIME * speed1 * 0.7);
	vec2 uv2 = world_pos / scale2 + vec2(TIME * speed2, TIME * speed2 * 0.4);
	float n = vnoise(uv1) * 0.62 + vnoise(uv2) * 0.38;
	float shade = smoothstep(0.40, 0.08, n);
	COLOR.rgb = mix(COLOR.rgb, COLOR.rgb * 0.55, shade * strength * COLOR.a);
	float shine = smoothstep(0.60, 0.92, n);
	COLOR.rgb = mix(COLOR.rgb, vec3(1.0), shine * strength * COLOR.a);
}
"#;

/// 每个地形的渲染图层信息
struct TerrainLayerInfo {
    node: Gd<TileMapLayer>,
    /// 是否为双网格渲染模式
    dual: bool,
}

#[derive(GodotClass)]
#[class(base = Node2D, tool)]
pub struct GdQuickMap {
    base: Base<Node2D>,

    /// 地图宽度（格子数）
    #[export]
    width: i32,
    /// 地图高度（格子数）
    #[export]
    height: i32,
    /// 格子尺寸（像素）
    #[export]
    cell_size: i32,
    /// 随机种子（generate 时 seed <= 0 则使用随机熵）
    #[export]
    seed_value: i64,
    /// 基础噪声波长（格子数，越小地形越碎）
    #[export]
    noise_scale: f64,
    /// 噪声叠加层数
    #[export]
    octaves: i32,
    /// 是否绘制网格参考线
    #[export]
    draw_grid_lines: bool,
    /// 累积铺底：下层图层把上层地形的格子也铺满，避免上层贴图透明缺口露出灰底
    #[export]
    paint_cumulative: bool,

    /// 地形名列表（留空使用内置默认地形）
    #[export]
    terrain_names: PackedStringArray,
    /// 地形颜色（与 names 按下标对应）
    #[export]
    terrain_colors: PackedColorArray,
    /// 地形贴图路径（可选，填了则覆盖颜色）
    #[export]
    terrain_textures: PackedStringArray,
    /// 双网格过渡贴图路径（4x4 = 16 格图集；某地形填了则该层启用双网格渲染）
    #[export]
    terrain_dualgrid_textures: PackedStringArray,
    /// 地形图层 shader（可选，按下标对应 terrain_names）：
    /// 填内置 shader 名（如 "water_flow"，源码编译在 Rust 侧，动态创建），
    /// 或 res://...gdshader 资源路径；留空不挂载
    #[export]
    terrain_shaders: PackedStringArray,
    /// 地形噪声上界（升序，前 n-1 个生效）
    #[export]
    terrain_thresholds: PackedFloat64Array,

    /// 不可通行的地形名列表（is_walkable 用）
    #[export]
    blocked_terrains: PackedStringArray,

    // ---- 运行时状态 ----
    /// 每格地形下标，-1 = 未生成
    grid: Vec<i32>,
    /// 地形缓存：名字 / 颜色 / 贴图
    names: Vec<String>,
    colors: Vec<Color>,
    textures: Vec<Option<Gd<Texture2D>>>,
    /// 不可通行地形下标缓存
    blocked_indices: Vec<i32>,
    /// 每个地形对应的渲染图层（与 names 下标对应）
    layers: Vec<TerrainLayerInfo>,
}

#[godot_api]
impl INode2D for GdQuickMap {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            width: 32,
            height: 32,
            cell_size: 32,
            seed_value: 12345,
            noise_scale: 10.0,
            octaves: 3,
            draw_grid_lines: false,
            paint_cumulative: true,
            terrain_names: PackedStringArray::new(),
            terrain_colors: PackedColorArray::new(),
            terrain_textures: PackedStringArray::new(),
            terrain_dualgrid_textures: PackedStringArray::new(),
            terrain_shaders: PackedStringArray::new(),
            terrain_thresholds: PackedFloat64Array::new(),
            blocked_terrains: PackedStringArray::new(),
            grid: Vec::new(),
            names: Vec::new(),
            colors: Vec::new(),
            textures: Vec::new(),
            blocked_indices: Vec::new(),
            layers: Vec::new(),
        }
    }

    fn ready(&mut self) {
        // 场景一打开（含编辑器）即按当前配置出图，方便所见即所得调参
        if self.grid.is_empty() {
            self.generate_internal(self.seed_value);
        }
    }

    fn draw(&mut self) {
        let cs = self.cell_size as f32;
        if cs <= 0.0 {
            return;
        }

        // 未生成时画整块底色；已生成时地形由各 TileMapLayer 子节点渲染
        if self.grid.is_empty() {
            let rect = Rect2::new(
                Vector2::ZERO,
                Vector2::new(self.width as f32 * cs, self.height as f32 * cs),
            );
            self.base_mut()
                .draw_rect(rect, Color::from_rgb(0.12, 0.12, 0.14));
        }

        // 网格参考线画在自身画布上，图层子节点 z_index = -1 保证线在贴图之上
        if self.draw_grid_lines {
            let line_color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
            let map_w = self.width as f32 * cs;
            let map_h = self.height as f32 * cs;
            let w = self.width;
            let h = self.height;
            let mut base = self.base_mut();
            for cx in 0..=w {
                let x = cx as f32 * cs;
                base.draw_line(Vector2::new(x, 0.0), Vector2::new(x, map_h), line_color);
            }
            for cy in 0..=h {
                let y = cy as f32 * cs;
                base.draw_line(Vector2::new(0.0, y), Vector2::new(map_w, y), line_color);
            }
        }
    }
}

#[godot_api]
impl GdQuickMap {
    /// 使用指定种子生成地图（seed <= 0 时使用随机熵）
    #[func]
    pub fn generate(&mut self, seed: i64) {
        if seed > 0 {
            self.seed_value = seed;
        }
        self.generate_internal(seed);
    }

    /// 以当前 seed_value 重新生成
    #[func]
    pub fn regenerate(&mut self) {
        self.generate_internal(self.seed_value);
    }

    /// 清空地图
    #[func]
    pub fn clear(&mut self) {
        self.grid.clear();
        self.free_layers();
        self.base_mut().queue_redraw();
    }

    /// 获取地形对应的渲染图层（下标对应 terrain_names；未生成为 null）
    #[func]
    pub fn get_terrain_layer(&self, terrain_index: i32) -> Option<Gd<TileMapLayer>> {
        self.layers
            .get(terrain_index as usize)
            .filter(|info| info.node.is_instance_valid())
            .map(|info| info.node.clone())
    }

    /// 地图宽度（格子数）
    #[func]
    pub fn get_map_width(&self) -> i32 {
        self.width
    }

    /// 地图高度（格子数）
    #[func]
    pub fn get_map_height(&self) -> i32 {
        self.height
    }

    /// 格子尺寸
    #[func]
    pub fn get_cell_size_px(&self) -> i32 {
        self.cell_size
    }

    /// 查询格子地形下标（越界返回 -1）
    #[func]
    pub fn get_terrain_at(&self, cell: Vector2i) -> i32 {
        self.terrain_index_at(cell.x, cell.y).unwrap_or(-1)
    }

    /// 查询格子地形名（越界返回空串）
    #[func]
    pub fn get_terrain_name_at(&self, cell: Vector2i) -> GString {
        match self.terrain_index_at(cell.x, cell.y) {
            Some(idx) => self
                .names
                .get(idx as usize)
                .map(|s| GString::from(s.as_str()))
                .unwrap_or_default(),
            None => GString::new(),
        }
    }

    /// 格子是否可通行（越界不可通行；地形在 blocked_terrains 中则不可通行）
    #[func]
    pub fn is_walkable(&self, cell: Vector2i) -> bool {
        match self.terrain_index_at(cell.x, cell.y) {
            Some(idx) => !self.blocked_indices.contains(&idx),
            None => false,
        }
    }

    /// 格子中心的世界坐标
    #[func]
    pub fn cell_to_center(&self, cell: Vector2i) -> Vector2 {
        let cs = self.cell_size as f32;
        Vector2::new((cell.x as f32 + 0.5) * cs, (cell.y as f32 + 0.5) * cs)
    }

    /// 世界坐标 → 格子坐标
    #[func]
    pub fn world_to_cell(&self, world_pos: Vector2) -> Vector2i {
        let cs = self.cell_size as f32;
        if cs <= 0.0 {
            return Vector2i::ZERO;
        }
        Vector2i::new(
            (world_pos.x / cs).floor() as i32,
            (world_pos.y / cs).floor() as i32,
        )
    }

    /// 手动设置某格地形（下标对应 terrain_names）
    #[func]
    pub fn set_terrain_at(&mut self, cell: Vector2i, terrain_index: i32) {
        if self.width <= 0 || self.height <= 0 {
            return;
        }
        if cell.x < 0 || cell.y < 0 || cell.x >= self.width || cell.y >= self.height {
            return;
        }
        if self.grid.is_empty() {
            self.grid = vec![-1; (self.width * self.height) as usize];
        }
        let i = (cell.y * self.width + cell.x) as usize;
        let old_ti = self.grid[i];
        self.grid[i] = terrain_index;
        // 局部刷新：新旧地形之间的所有受影响图层（含被铺底的下层）
        self.refresh_cell(cell, old_ti, terrain_index);
        self.base_mut().queue_redraw();
    }

    /// 设置不可通行的地形名列表
    #[func]
    pub fn set_blocked_terrain_names(&mut self, names: PackedStringArray) {
        self.blocked_terrains = names.clone();
        self.rebuild_blocked_indices();
    }

    /// 获取使用的随机种子
    #[func]
    pub fn get_seed(&self) -> i64 {
        self.seed_value
    }

    /// BFS 寻路：四方向最短步数路径，返回路径各格中心的世界坐标
    /// （不含起点，含终点）。起点=终点、终点不可达或越界时返回空数组。
    #[func]
    pub fn find_path(&self, start: Vector2i, end: Vector2i) -> PackedVector2Array {
        let mut out = PackedVector2Array::new();
        if start == end || !self.is_walkable(start) || !self.is_walkable(end) {
            return out;
        }
        let w = self.width;
        let h = self.height;
        if w <= 0 || h <= 0 {
            return out;
        }
        let total = (w * h) as usize;
        let start_i = (start.y * w + start.x) as usize;
        let end_i = (end.y * w + end.x) as usize;

        let neighbors = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)];
        let mut came: Vec<usize> = vec![usize::MAX; total];
        let mut visited: Vec<bool> = vec![false; total];
        let mut queue = std::collections::VecDeque::new();
        visited[start_i] = true;
        queue.push_back(start_i);

        let mut found = false;
        while let Some(cur) = queue.pop_front() {
            if cur == end_i {
                found = true;
                break;
            }
            let cx = (cur as i32) % w;
            let cy = (cur as i32) / w;
            for (dx, dy) in neighbors {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let ni = (ny * w + nx) as usize;
                if visited[ni] || !self.is_walkable(Vector2i::new(nx, ny)) {
                    continue;
                }
                visited[ni] = true;
                came[ni] = cur;
                queue.push_back(ni);
            }
        }
        if !found {
            return out;
        }

        // 回溯：终点 → 起点（不含起点），反转后逐格输出中心坐标
        let mut cells: Vec<usize> = Vec::new();
        let mut cur = end_i;
        while cur != start_i {
            cells.push(cur);
            cur = came[cur];
        }
        cells.reverse();
        for ci in cells {
            let cx = (ci as i32) % w;
            let cy = (ci as i32) / w;
            out.push(self.cell_to_center(Vector2i::new(cx, cy)));
        }
        out
    }

    // ---- 内部实现 ----

    fn terrain_index_at(&self, cx: i32, cy: i32) -> Option<i32> {
        if self.width <= 0 || self.height <= 0 || self.grid.is_empty() {
            return None;
        }
        if cx < 0 || cy < 0 || cx >= self.width || cy >= self.height {
            return None;
        }
        self.grid.get((cy * self.width + cx) as usize).copied()
    }

    /// 生成主流程：重建地形表 → 噪声 → 阈值划分 → 重绘
    fn generate_internal(&mut self, seed: i64) {
        self.build_terrain_table();
        self.rebuild_blocked_indices();

        let w = self.width.max(1);
        let h = self.height.max(1);
        if self.names.is_empty() {
            godot_error!("GdQuickMap: 无可用地形定义（terrain_names 为空且默认表构建失败）");
            return;
        }

        let values = self.generate_noise(w, h, seed);
        let n = self.names.len();
        self.grid = vec![0; (w * h) as usize];

        for cy in 0..h {
            for cx in 0..w {
                let v = values[(cy * w + cx) as usize];
                let ti = self.pick_terrain(v, n);
                self.grid[(cy * w + cx) as usize] = ti as i32;
            }
        }

        self.rebuild_layers();
        self.base_mut().queue_redraw();
    }

    // ---- 图层渲染 ----

    /// 释放所有地形图层子节点
    fn free_layers(&mut self) {
        for info in self.layers.drain(..) {
            if info.node.is_instance_valid() {
                info.node.free();
            }
        }
    }

    /// 重建全部地形图层：每种地形一个 TileMapLayer，按地形顺序叠放
    fn rebuild_layers(&mut self) {
        self.free_layers();
        if self.grid.is_empty() || self.names.is_empty() {
            return;
        }
        let cs = self.cell_size.max(1) as f32;
        let count = self.names.len().min(MAX_TERRAINS);

        for ti in 0..count {
            let dual_path = self
                .terrain_dualgrid_textures
                .get(ti)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let mut layer = TileMapLayer::new_alloc();
            let name = format!("L{}_{}", ti, self.names.get(ti).map(|s| s.as_str()).unwrap_or("?"));
            layer.set_name(&StringName::from(name.as_str()));
            // z_index = -1：让本节点的网格参考线绘制在贴图之上
            layer.set_z_index(-1);

            let mut info = if !dual_path.is_empty() {
                match Self::load_texture(&dual_path) {
                    Some(tex) if tex.is_instance_valid() => {
                        // 4x4 = 16 格过渡图集，显示网格偏移半格并缩放到 cell_size
                        let ts_px = (tex.get_width().max(4) / 4).max(1);
                        let ts = self.build_tile_set(&tex, ts_px);
                        layer.set_tile_set(&ts);
                        layer.set_position(Vector2::new(-cs / 2.0, -cs / 2.0));
                        layer.set_scale(Vector2::new(cs / ts_px as f32, cs / ts_px as f32));
                        self.fill_dual_layer(&mut layer, ti);
                        TerrainLayerInfo { node: layer, dual: true }
                    }
                    _ => {
                        godot_error!(
                            "GdQuickMap: 双网格贴图加载失败: {}（地形 {} 回退为纯色）",
                            dual_path,
                            ti
                        );
                        self.build_plain_layer(&mut layer, ti, cs);
                        TerrainLayerInfo { node: layer, dual: false }
                    }
                }
            } else {
                self.build_plain_layer(&mut layer, ti, cs);
                TerrainLayerInfo { node: layer, dual: false }
            };

            // 可选 shader：为该地形层挂 ShaderMaterial（如水体流动）
            let shader_name = self
                .terrain_shaders
                .get(ti)
                .map(|s| s.to_string())
                .unwrap_or_default();
            if !shader_name.is_empty() {
                match Self::resolve_shader(&shader_name) {
                    Some(shader) => {
                        let mut mat = ShaderMaterial::new_gd();
                        mat.set_shader(&shader);
                        info.node.set_material(&mat);
                    }
                    None => {
                        godot_error!(
                            "GdQuickMap: shader 无法创建: {}（地形 {}）",
                            shader_name,
                            ti
                        );
                    }
                }
            }

            self.base_mut().add_child(&info.node);
            self.layers.push(info);
        }
    }

    /// 普通整格模式：贴图优先，否则用地形颜色生成纯色贴图
    fn build_plain_layer(&mut self, layer: &mut Gd<TileMapLayer>, ti: usize, cs: f32) {
        let tex: Option<Gd<Texture2D>> = match self.textures.get(ti).and_then(|t| t.clone()) {
            Some(t) if t.is_instance_valid() => Some(t),
            _ => {
                let color = self
                    .colors
                    .get(ti)
                    .copied()
                    .unwrap_or_else(|| Color::from_rgb(0.6, 0.6, 0.6));
                Self::color_texture(color, 16).map(|t| t.upcast())
            }
        };
        let Some(tex) = tex else {
            return;
        };
        let ts_px = tex.get_width().max(1);
        let ts = self.build_tile_set(&tex, ts_px);
        layer.set_tile_set(&ts);
        layer.set_position(Vector2::ZERO);
        layer.set_scale(Vector2::new(cs / ts_px as f32, cs / ts_px as f32));

        let sid = Self::first_source_id(&ts);
        if sid < 0 {
            return;
        }
        for cy in 0..self.height {
            for cx in 0..self.width {
                let g = self.grid[(cy * self.width + cx) as usize];
                // 累积铺底：下层也铺更上层地形的格子，保证上层透明缺口下有图
                let paint = if self.paint_cumulative {
                    g >= ti as i32
                } else {
                    g == ti as i32
                };
                if paint {
                    layer.set_cell_ex(Vector2i::new(cx, cy))
                        .source_id(sid)
                        .atlas_coords(Vector2i::new(0, 0))
                        .done();
                }
            }
        }
    }

    /// 双网格模式填充：显示格 (dx,dy) 覆盖世界格 (dx-1,dy-1)..(dx,dy)，
    /// 按 dual_display_tile 决策放置过渡片或垫底整块
    fn fill_dual_layer(&self, layer: &mut Gd<TileMapLayer>, ti: usize) {
        let Some(ts) = layer.get_tile_set() else {
            return;
        };
        let sid = Self::first_source_id(&ts);
        if sid < 0 {
            return;
        }
        for dy in 0..=self.height {
            for dx in 0..=self.width {
                if let Some((ax, ay)) = self.dual_display_tile(dx, dy, ti as i32) {
                    layer.set_cell_ex(Vector2i::new(dx, dy))
                        .source_id(sid)
                        .atlas_coords(Vector2i::new(ax, ay))
                        .done();
                }
            }
        }
    }

    /// 双网格显示格决策：返回 Some((atlas_x, atlas_y)) 表示放置图块，None 表示保持空
    /// 优先级 1：任一四角是本地形 → 按四角组合取过渡贴图（本地形边缘形状）
    /// 优先级 2（累积铺底）：四角全是更上层地形（且无越界外混入更低地形）→
    ///   取"全地形"整块图块垫底，让上层的圆角缺口不露灰
    fn dual_display_tile(&self, dx: i32, dy: i32, ti: i32) -> Option<(i32, i32)> {
        let quads = [
            self.terrain_index_at(dx - 1, dy - 1),
            self.terrain_index_at(dx, dy - 1),
            self.terrain_index_at(dx - 1, dy),
            self.terrain_index_at(dx, dy),
        ];
        let is_self = |q: Option<i32>| q == Some(ti);
        if quads.iter().any(|q| is_self(*q)) {
            let (ax, ay) = dual_grid_atlas_coord(
                is_self(quads[0]),
                is_self(quads[1]),
                is_self(quads[2]),
                is_self(quads[3]),
            );
            return Some((ax, ay));
        }
        if self.paint_cumulative {
            let mut has_inbounds = false;
            let mut all_upper = true;
            for q in quads.iter().flatten() {
                has_inbounds = true;
                if *q <= ti {
                    all_upper = false;
                    break;
                }
            }
            if has_inbounds && all_upper {
                return Some(dual_grid_atlas_coord(true, true, true, true));
            }
        }
        None
    }

    /// 构建 TileSet：单图集源，格尺寸 = ts_px
    fn build_tile_set(&self, tex: &Gd<Texture2D>, ts_px: i32) -> Gd<TileSet> {
        let mut ts = TileSet::new_gd();
        ts.set_tile_size(Vector2i::new(ts_px, ts_px));
        let mut src = TileSetAtlasSource::new_gd();
        src.set_texture(tex);
        src.set_texture_region_size(Vector2i::new(ts_px, ts_px));
        // 图集内所有可用格都注册为 tile
        let region_w = tex.get_width() / ts_px.max(1);
        let region_h = tex.get_height() / ts_px.max(1);
        for ry in 0..region_h {
            for rx in 0..region_w {
                src.create_tile(Vector2i::new(rx, ry));
            }
        }
        let source: Gd<TileSetSource> = src.upcast();
        ts.add_source_ex(&source).done();
        ts
    }

    /// 取 TileSet 中第一个图集源的 source_id（无源返回 -1）
    fn first_source_id(ts: &Gd<TileSet>) -> i32 {
        if ts.get_source_count() > 0 {
            ts.get_source_id(0)
        } else {
            -1
        }
    }

    /// 地形颜色 → 纯色贴图（16x16，双线性下放大无碍像素观感）
    fn color_texture(color: Color, size: i32) -> Option<Gd<ImageTexture>> {
        let mut img = Image::create_empty(size, size, false, ImageFormat::RGBA8)?;
        img.fill_rect(
            Rect2i::new(Vector2i::ZERO, Vector2i::new(size, size)),
            color,
        );
        ImageTexture::create_from_image(&img)
    }

    /// 局部刷新：世界格 cell 的地形从 old_ti 变为 new_ti 后，
    /// 重算所有受影响图层在该格（双网格层为其 4 个受影响显示格）上的图块。
    /// 受影响图层 = 下标在 (min, max] 区间的层（这些层的铺底/过渡判定会翻转）；
    /// 任一端为 -1（擦除/从空上色）时为 0..=max。
    fn refresh_cell(&mut self, cell: Vector2i, old_ti: i32, new_ti: i32) {
        if self.grid.is_empty() {
            return;
        }
        let hi = old_ti.max(new_ti);
        if hi < 0 {
            return;
        }
        let start = if old_ti < 0 || new_ti < 0 {
            0
        } else {
            old_ti.min(new_ti) + 1
        };
        for ti in start..=hi {
            self.update_layer_cell(cell, ti);
        }
    }

    /// 更新单个图层在世界格 cell 上的图块（普通层动一格；双网格层动 4 个显示格）
    fn update_layer_cell(&mut self, cell: Vector2i, ti: i32) {
        // 先取出图层信息，避免与后续 self 不可变借用冲突
        let (dual, node) = {
            let Some(info) = self.layers.get(ti as usize) else {
                return;
            };
            if !info.node.is_instance_valid() {
                return;
            }
            (info.dual, info.node.clone())
        };
        let mut sid = -1;
        if let Some(s) = node.get_tile_set() {
            sid = Self::first_source_id(&s);
        }
        if sid < 0 {
            return;
        }
        let g = self.terrain_index_at(cell.x, cell.y);
        if !dual {
            // 普通模式：铺底条件与 build_plain_layer 一致
            let paint = if self.paint_cumulative {
                matches!(g, Some(v) if v >= ti)
            } else {
                g == Some(ti)
            };
            let mut layer = node;
            if paint {
                layer.set_cell_ex(cell).source_id(sid)
                    .atlas_coords(Vector2i::new(0, 0)).done();
            } else {
                layer.erase_cell(cell);
            }
            return;
        }
        // 双网格模式：该世界格影响 4 个显示格
        let offsets = [(0i32, 0i32), (1, 0), (0, 1), (1, 1)];
        for (ox, oy) in offsets {
            let dcell = Vector2i::new(cell.x + ox, cell.y + oy);
            let mut layer = node.clone();
            if let Some((ax, ay)) = self.dual_display_tile(dcell.x, dcell.y, ti) {
                layer.set_cell_ex(dcell).source_id(sid)
                    .atlas_coords(Vector2i::new(ax, ay)).done();
            } else {
                layer.erase_cell(dcell);
            }
        }
    }

    /// 构建地形表：无配置时使用内置默认地形（颜色即贴图）
    fn build_terrain_table(&mut self) {
        self.names.clear();
        self.colors.clear();
        self.textures.clear();

        let use_default = self.terrain_names.is_empty();
        if use_default {
            let defaults: [(&str, Color); 5] = [
                ("water", Color::from_rgb(0.25, 0.5, 0.85)),
                ("sand", Color::from_rgb(0.9, 0.85, 0.62)),
                ("grass", Color::from_rgb(0.36, 0.7, 0.38)),
                ("forest", Color::from_rgb(0.2, 0.45, 0.24)),
                ("mountain", Color::from_rgb(0.56, 0.56, 0.6)),
            ];
            for (name, color) in defaults {
                self.names.push(name.to_string());
                self.colors.push(color);
                self.textures.push(None);
            }
        } else {
            let count = (self.terrain_names.len() as usize).min(MAX_TERRAINS);
            for i in 0..count {
                let name = self.terrain_names.get(i).map(|s| s.to_string()).unwrap_or_default();
                let color = self.terrain_colors.get(i).unwrap_or_else(|| {
                    Self::default_color_for(&name)
                });
                let tex_path = self
                    .terrain_textures
                    .get(i)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let tex = if tex_path.is_empty() {
                    None
                } else {
                    Self::load_texture(&tex_path)
                };
                self.names.push(name);
                self.colors.push(color);
                self.textures.push(tex);
            }
        }
    }

    /// 按噪声值挑地形：value < thresholds[i] → i；否则最后一个地形
    fn pick_terrain(&self, value: f64, n: usize) -> usize {
        for i in 0..n.saturating_sub(1) {
            let bound = if i < self.terrain_thresholds.len() {
                self.terrain_thresholds.get(i).unwrap_or(1.0)
            } else {
                ((i + 1) as f64) / (n as f64)
            };
            if value < bound {
                return i;
            }
        }
        n.saturating_sub(1)
    }

    /// 种子驱动的多八度值噪声，输出归一化到 [0, 1]
    fn generate_noise(&self, w: i32, h: i32, seed: i64) -> Vec<f64> {
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;
        let mut rng: StdRng = if seed <= 0 {
            StdRng::from_entropy()
        } else {
            StdRng::seed_from_u64(seed as u64)
        };

        let total = (w * h) as usize;
        let mut out = vec![0.0f64; total];
        let mut amp_sum = 0.0f64;
        let mut amp = 1.0f64;
        let mut wavelength = self.noise_scale.max(2.0);
        let octaves = self.octaves.clamp(1, 6);

        for _ in 0..octaves {
            let gw = ((w as f64 / wavelength).ceil() as i32).max(1) + 2;
            let gh = ((h as f64 / wavelength).ceil() as i32).max(1) + 2;
            let mut lattice = vec![0.0f64; (gw * gh) as usize];
            for v in lattice.iter_mut() {
                *v = rng.gen_range(0.0..1.0);
            }

            for y in 0..h {
                for x in 0..w {
                    let fx = x as f64 / wavelength;
                    let fy = y as f64 / wavelength;
                    let x0 = fx.floor() as i32;
                    let y0 = fy.floor() as i32;
                    let sx = smoothstep(fx - x0 as f64);
                    let sy = smoothstep(fy - y0 as f64);

                    let at = |gx: i32, gy: i32| -> f64 {
                        let gx = gx.clamp(0, gw - 1);
                        let gy = gy.clamp(0, gh - 1);
                        lattice[(gy * gw + gx) as usize]
                    };
                    let v00 = at(x0, y0);
                    let v10 = at(x0 + 1, y0);
                    let v01 = at(x0, y0 + 1);
                    let v11 = at(x0 + 1, y0 + 1);

                    let top = v00 + sx * (v10 - v00);
                    let bottom = v01 + sx * (v11 - v01);
                    let v = top + sy * (bottom - top);

                    out[(y * w + x) as usize] += v * amp;
                }
            }

            amp_sum += amp;
            amp *= 0.5;
            wavelength = (wavelength / 2.0).max(1.5);
        }

        // 归一化到 [0, 1]
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for v in &out {
            min = min.min(*v);
            max = max.max(*v);
        }
        let range = (max - min).max(1e-9);
        for v in out.iter_mut() {
            *v = (*v - min) / range;
        }
        out
    }

    /// 已知地形名的默认颜色（自定义配置缺色时回退）
    fn default_color_for(name: &str) -> Color {
        match name {
            "water" => Color::from_rgb(0.25, 0.5, 0.85),
            "sand" => Color::from_rgb(0.9, 0.85, 0.62),
            "grass" => Color::from_rgb(0.36, 0.7, 0.38),
            "forest" => Color::from_rgb(0.2, 0.45, 0.24),
            "mountain" => Color::from_rgb(0.56, 0.56, 0.6),
            _ => Color::from_rgb(0.6, 0.6, 0.6),
        }
    }

    /// 重建不可通行地形下标缓存
    fn rebuild_blocked_indices(&mut self) {
        self.blocked_indices.clear();
        for i in 0..self.blocked_terrains.len() {
            let name = self.blocked_terrains.get(i).map(|s| s.to_string()).unwrap_or_default();
            if let Some(pos) = self.names.iter().position(|n| *n == name) {
                self.blocked_indices.push(pos as i32);
            }
        }
    }

    fn load_texture(path: &str) -> Option<Gd<Texture2D>> {
        if path.is_empty() {
            return None;
        }
        let res = ResourceLoader::singleton().load_ex(path).done()?;
        res.try_cast::<Texture2D>().ok()
    }

    /// 解析 shader 来源：res:// 路径 → 从资源加载；否则视为内置 shader 名，
    /// 用编译进 Rust 的源码动态创建 Shader 对象
    fn resolve_shader(name_or_path: &str) -> Option<Gd<Shader>> {
        if name_or_path.is_empty() {
            return None;
        }
        if name_or_path.starts_with("res://") {
            return Self::load_shader(name_or_path);
        }
        let src = Self::builtin_shader_source(name_or_path)?;
        let mut shader = Shader::new_gd();
        shader.set_code(src);
        Some(shader)
    }

    /// 内置 shader 源码注册表：名字 → gdshader 源码（编译进 Rust 二进制，
    /// 无需 Godot 侧资源文件）
    fn builtin_shader_source(name: &str) -> Option<&'static str> {
        match name {
            "water_flow" => Some(WATER_FLOW_SHADER_SRC),
            _ => None,
        }
    }

    /// 加载 shader 资源（res://...gdshader）
    fn load_shader(path: &str) -> Option<Gd<Shader>> {
        if path.is_empty() {
            return None;
        }
        let res = ResourceLoader::singleton().load_ex(path).done()?;
        res.try_cast::<Shader>().ok()
    }
}

/// smoothstep 缓动
fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
