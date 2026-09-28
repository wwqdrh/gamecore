// GdQuickMap - 快速地图生成器（继承 Node2D，tool）
//
// 用于快速搭建俯视角四方向模式的地图场景：
//   - 种子驱动的多八度值噪声 + 阈值划分地形，同一 seed 必然生成同一张地图
//   - 默认地形用颜色块直接表示（零素材即可跑通玩法）；
//     在编辑器给 terrain_textures 填贴图路径即可替换为真实素材
//   - 提供格子坐标换算与 is_walkable 通行查询，配合 GdRoleMover 的
//     MODE_GRID 网格移动模式使用
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
use godot::classes::{INode2D, Node2D, ResourceLoader, Texture2D};

/// 地形数量上限（防御并行数组异常输入）
const MAX_TERRAINS: usize = 32;

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

    /// 地形名列表（留空使用内置默认地形）
    #[export]
    terrain_names: PackedStringArray,
    /// 地形颜色（与 names 按下标对应）
    #[export]
    terrain_colors: PackedColorArray,
    /// 地形贴图路径（可选，填了则覆盖颜色）
    #[export]
    terrain_textures: PackedStringArray,
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
            terrain_names: PackedStringArray::new(),
            terrain_colors: PackedColorArray::new(),
            terrain_textures: PackedStringArray::new(),
            terrain_thresholds: PackedFloat64Array::new(),
            blocked_terrains: PackedStringArray::new(),
            grid: Vec::new(),
            names: Vec::new(),
            colors: Vec::new(),
            textures: Vec::new(),
            blocked_indices: Vec::new(),
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
        if cs <= 0.0 || self.grid.is_empty() {
            return;
        }

        for cy in 0..self.height {
            for cx in 0..self.width {
                let idx = self.grid[(cy * self.width + cx) as usize];
                let rect = Rect2::new(
                    Vector2::new(cx as f32 * cs, cy as f32 * cs),
                    Vector2::new(cs, cs),
                );
                if idx >= 0 && (idx as usize) < self.colors.len() {
                    let ti = idx as usize;
                    match self.textures.get(ti).and_then(|t| t.as_ref()) {
                        Some(tex) => {
                            let tex = tex.clone();
                            self.base_mut()
                                .draw_texture_rect(&tex, rect, false);
                        }
                        None => {
                            let color = self.colors[ti];
                            self.base_mut().draw_rect(rect, color);
                        }
                    }
                } else {
                    // 未生成的格子画底色
                    self.base_mut()
                        .draw_rect(rect, Color::from_rgb(0.12, 0.12, 0.14));
                }
            }
        }

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
        self.base_mut().queue_redraw();
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
        self.grid[i] = terrain_index;
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

        self.base_mut().queue_redraw();
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
                let color = self
                    .terrain_colors
                    .get(i)
                    .unwrap_or_else(|| Color::from_rgb(0.6, 0.6, 0.6));
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
}

/// smoothstep 缓动
fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
