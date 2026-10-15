// GdMiniMap - 世界缩略地图（继承 Control，tool）
//
// 渲染「多张地图拼接后的缩略图」：地形色块网格 + 传送点配对连线 +
// 当前地图高亮边框 + 玩家呼吸光点。数据驱动（不耦合具体地图实现）：
//
//   minimap.clear_all()
//   minimap.add_map("xiuxian_town", "青云坊", 98, 6, 32, 20, cells, terrains)
//   minimap.add_portal("xiuxian_town", Vector2i(28,10), "xiuxian_map_a", Vector2i(4,13), "前往青草岭")
//   minimap.set_current("xiuxian_town")     # 当前所在地图（金框 + 玩家锚点）
//   minimap.set_player_cell(Vector2i(16,3)) # 玩家在当前地图内的格子
//
// cells 为行主序地形索引（值 = terrains 下标）；地形名 → 颜色走内置色板，
// 可用 set_terrain_color("grass", Color...) 覆盖。
//
// demo 数据源：test/regen_map_thumbs.gd dump 的 state/map/map_thumbs.json
// （由六张地图场景离线采样，地图改动后重跑）。范本：
// example/demo/xiuxian/scenes/main/ui/map/map_modal.gd

use godot::prelude::*;

use godot::classes::{IControl, Control, ThemeDb};
use std::collections::HashMap;

/// 单张地图缩略数据
#[derive(Clone)]
struct MapThumb {
    alias: String,
    name: String,
    /// 拼接布局偏移（格子单位）
    ox: i32,
    oy: i32,
    w: i32,
    h: i32,
    /// 行主序地形索引（值 = terrains 下标）
    cells: Vec<i32>,
    terrains: Vec<String>,
}

/// 传送点缩略数据
#[derive(Clone)]
struct PortalThumb {
    alias: String,
    cell: Vector2i,
    to_alias: String,
    to_cell: Vector2i,
    #[allow(dead_code)]
    label: String,
}

#[derive(GodotClass)]
#[class(base = Control, tool)]
pub struct GdMiniMap {
    /// 每格像素尺寸（缩略比例）
    #[export]
    cell_px: f32,
    /// 内容与控件边缘的留白（px）
    #[export]
    padding: f32,
    /// 是否绘制地图显示名
    #[export]
    show_names: bool,
    /// 聚焦模式：只绘制当前地图（set_current 指定），按控件尺寸自适应
    /// 缩放并居中（HUD 右上角小地图用；false = 全世界拼接 + 固定 cell_px）
    #[export]
    fit_current: bool,
    /// 玩家光点是否呼吸脉冲
    #[export]
    pulse: bool,

    // ---- 配色 ----
    #[export]
    paper_bg: Color,
    #[export]
    map_border: Color,
    #[export]
    current_border: Color,
    #[export]
    player_color: Color,
    #[export]
    portal_color: Color,
    #[export]
    link_color: Color,
    #[export]
    unknown_color: Color,
    #[export]
    name_color: Color,

    maps: Vec<MapThumb>,
    portals: Vec<PortalThumb>,
    terrain_colors: HashMap<String, Color>,
    current: String,
    player_cell: Vector2i,
    has_player: bool,
    /// 脉冲相位（秒）
    phase: f32,
    /// 重绘节流累计
    redraw_acc: f32,

    base: Base<Control>,
}

/// 内置地形色板（缺省值；set_terrain_color 可覆盖）
fn default_terrain_colors() -> HashMap<String, Color> {
    let mut m = HashMap::new();
    m.insert("grass".to_string(), Color::from_rgb(0.49, 0.71, 0.42));
    m.insert("forest".to_string(), Color::from_rgb(0.29, 0.48, 0.28));
    m.insert("mountain".to_string(), Color::from_rgb(0.55, 0.52, 0.46));
    m.insert("water".to_string(), Color::from_rgb(0.36, 0.56, 0.79));
    m.insert("sand".to_string(), Color::from_rgb(0.85, 0.75, 0.53));
    m.insert("farmland".to_string(), Color::from_rgb(0.66, 0.55, 0.35));
    m.insert("floor".to_string(), Color::from_rgb(0.69, 0.60, 0.45));
    m
}

#[godot_api]
impl IControl for GdMiniMap {
    fn init(base: Base<Control>) -> Self {
        Self {
            cell_px: 6.0,
            padding: 12.0,
            show_names: true,
            fit_current: false,
            pulse: true,
            paper_bg: Color::from_rgb(0.13, 0.15, 0.12),
            map_border: Color::from_rgb(0.45, 0.40, 0.30),
            current_border: Color::from_rgb(0.83, 0.69, 0.22),
            player_color: Color::from_rgb(0.95, 0.30, 0.25),
            portal_color: Color::from_rgb(0.35, 0.95, 1.0),
            link_color: Color::from_rgba(1.0, 0.85, 0.4, 0.45),
            unknown_color: Color::from_rgb(0.6, 0.6, 0.6),
            name_color: Color::from_rgba(0.93, 0.90, 0.82, 0.85),
            maps: Vec::new(),
            portals: Vec::new(),
            terrain_colors: default_terrain_colors(),
            current: String::new(),
            player_cell: Vector2i::ZERO,
            has_player: false,
            phase: 0.0,
            redraw_acc: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        self.update_content_size();
        self.base_mut().queue_redraw();
    }

    fn process(&mut self, delta: f64) {
        // 可见时才驱动呼吸脉冲重绘（节流 ~30fps，modal 关闭后零开销）
        if !self.pulse || !self.base().is_visible_in_tree() {
            return;
        }
        self.phase += delta as f32;
        self.redraw_acc += delta as f32;
        if self.redraw_acc >= 0.033 {
            self.redraw_acc = 0.0;
            self.base_mut().queue_redraw();
        }
    }

    fn draw(&mut self) {
        if self.maps.is_empty() {
            return;
        }
        // 聚焦模式：只保留当前地图（找不到时回退全世界），portal_pair 对
        // 不在集内的对端返回 None——只画本图出口光点，不画跨图连线
        let mut maps = self.maps.clone();
        if self.fit_current {
            let cur = self.current.clone();
            let focused = maps.iter().find(|m| m.alias == cur).cloned();
            if let Some(cm) = focused {
                maps.clear();
                maps.push(cm);
            }
        }
        // 借用红线：先把全部标量/数据取出（拷贝），再借 base_mut 绘制
        let padding = self.padding;
        // 格距：聚焦模式按控件可用区自适应（铺满短边），否则固定 cell_px
        let cell = if self.fit_current {
            let m0 = &maps[0];
            let bs = self.base().get_size();
            let avail_w = (bs.x - padding * 2.0).max(8.0);
            let avail_h = (bs.y - padding * 2.0).max(8.0);
            (avail_w / m0.w.max(1) as f32).min(avail_h / m0.h.max(1) as f32).max(1.0)
        } else {
            self.cell_px.max(1.0)
        };
        // 聚焦模式长边留白 → 整体居中偏移（非聚焦 = 0）
        let center_off = if self.fit_current {
            let m0 = &maps[0];
            let bs = self.base().get_size();
            let pad2 = Vector2::new(padding * 2.0, padding * 2.0);
            let used = Vector2::new(m0.w as f32 * cell, m0.h as f32 * cell) + pad2;
            let half = (bs - used) * 0.5;
            Vector2::new(half.x.max(0.0), half.y.max(0.0))
        } else {
            Vector2::ZERO
        };
        let paper_bg = self.paper_bg;
        let map_border = self.map_border;
        let current_border = self.current_border;
        let player_color = self.player_color;
        let portal_color = self.portal_color;
        let link_color = self.link_color;
        let unknown_color = self.unknown_color;
        let name_color = self.name_color;
        let show_names = self.show_names;
        let current = self.current.clone();
        let has_player = self.has_player;
        let player_cell = self.player_cell;
        let phase = self.phase;
        let portals = self.portals.clone();
        let palette = self.terrain_colors.clone();
        let (min_x, min_y, _, _) = Self::content_bounds_cells(&maps);

        // ---- 每张地图：底板 + 地形色块 + 边框 + 名字 ----
        for m in &maps {
            let origin = Self::map_origin(&maps, &m.alias, min_x, min_y, cell, padding)
                .unwrap_or_default()
                + center_off;
            let size = Vector2::new(m.w as f32 * cell, m.h as f32 * cell);

            // 底板
            self.base_mut().draw_rect(Rect2::new(origin, size), paper_bg);
            // 地形色块
            for y in 0..m.h {
                for x in 0..m.w {
                    let idx = m.cells[(y * m.w + x) as usize];
                    let tname = m
                        .terrains
                        .get(idx as usize)
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    let color = palette.get(tname).copied().unwrap_or(unknown_color);
                    let p = origin + Vector2::new(x as f32 * cell, y as f32 * cell);
                    self.base_mut()
                        .draw_rect(Rect2::new(p, Vector2::new(cell, cell)), color);
                }
            }
            // 边框（当前地图金框加粗）
            let is_current = m.alias == current;
            let border = if is_current { current_border } else { map_border };
            let bw = if is_current { 2.5 } else { 1.2 };
            self.base_mut()
                .draw_rect_ex(Rect2::new(origin, size), border)
                .filled(false)
                .width(bw)
                .done();
            // 显示名（地图上方居中）
            if show_names && !m.name.is_empty() {
                if let Some(font) = ThemeDb::singleton().get_fallback_font() {
                    let fs = 12;
                    let text_gd = GString::from(m.name.as_str());
                    let size = font
                        .get_string_size_ex(&text_gd)
                        .width(-1.0)
                        .font_size(fs)
                        .done();
                    let pos = origin
                        + Vector2::new((m.w as f32 * cell - size.x) * 0.5, -4.0);
                    self.base_mut()
                        .draw_string_ex(&font, pos, &text_gd)
                        .font_size(fs)
                        .modulate(name_color)
                        .done();
                }
            }
        }

        // ---- 传送点：配对连线 + 光点 ----
        for i in 0..portals.len() {
            if let Some((ca, cb)) =
                Self::portal_pair(&maps, &portals, i, min_x, min_y, cell, padding)
            {
                self.base_mut()
                    .draw_line_ex(ca + center_off, cb + center_off, link_color)
                    .width(1.5)
                    .done();
            }
        }
        for p in &portals {
            let Some(mpos) = Self::map_origin(&maps, &p.alias, min_x, min_y, cell, padding) else {
                continue;
            };
            let c = mpos
                + center_off
                + Vector2::new((p.cell.x as f32 + 0.5) * cell, (p.cell.y as f32 + 0.5) * cell);
            let r = (cell * 0.4).max(2.5);
            self.base_mut()
                .draw_circle(c, r + 1.2, Color::from_rgba(0.0, 0.0, 0.0, 0.5));
            self.base_mut().draw_circle(c, r, portal_color);
        }

        // ---- 玩家呼吸光点（当前地图内）----
        if has_player {
            if let Some(mpos) =
                Self::map_origin(&maps, &current.as_str(), min_x, min_y, cell, padding)
            {
                let c = mpos
                    + center_off
                    + Vector2::new(
                        (player_cell.x as f32 + 0.5) * cell,
                        (player_cell.y as f32 + 0.5) * cell,
                    );
                let t = (phase * 2.4).sin() * 0.5 + 0.5;
                let ring_r = 5.0 + 3.0 * t;
                let ring = Color::from_rgba(
                    player_color.r,
                    player_color.g,
                    player_color.b,
                    0.45 * (1.0 - t),
                );
                self.base_mut().draw_circle(c, ring_r, ring);
                self.base_mut()
                    .draw_circle(c, 3.5, Color::from_rgba(1.0, 1.0, 1.0, 0.9));
                self.base_mut().draw_circle(c, 2.5, player_color);
            }
        }
    }
}

#[godot_api]
impl GdMiniMap {
    /// 清空全部数据
    #[func]
    pub fn clear_all(&mut self) {
        self.maps.clear();
        self.portals.clear();
        self.current.clear();
        self.has_player = false;
        self.update_content_size();
        self.base_mut().queue_redraw();
    }

    /// 添加一张地图缩略
    /// alias 别名 / name 显示名 / ox,oy 拼接布局偏移（格子单位）/
    /// cells 行主序地形索引 / terrains 地形名表
    #[func]
    pub fn add_map(
        &mut self,
        alias: GString,
        name: GString,
        ox: i32,
        oy: i32,
        w: i32,
        h: i32,
        cells: PackedInt32Array,
        terrains: PackedStringArray,
    ) {
        self.maps.retain(|m| m.alias != alias.to_string());
        self.maps.push(MapThumb {
            alias: alias.to_string(),
            name: name.to_string(),
            ox,
            oy,
            w,
            h,
            cells: cells.to_vec(),
            terrains: terrains.to_vec().iter().map(|s| s.to_string()).collect(),
        });
        self.update_content_size();
        self.base_mut().queue_redraw();
    }

    /// 添加传送点（alias 地图内 cell 格，通向 to_alias 的 to_cell）
    #[func]
    pub fn add_portal(
        &mut self,
        alias: GString,
        cell: Vector2i,
        to_alias: GString,
        to_cell: Vector2i,
        label: GString,
    ) {
        let alias = alias.to_string();
        let to_alias = to_alias.to_string();
        self.portals.retain(|p| !(p.alias == alias && p.cell == cell));
        self.portals.push(PortalThumb {
            alias,
            cell,
            to_alias,
            to_cell,
            label: label.to_string(),
        });
        self.base_mut().queue_redraw();
    }

    /// 设置当前所在地图（金框高亮 + 玩家锚点图）
    #[func]
    pub fn set_current(&mut self, alias: GString) {
        self.current = alias.to_string();
        self.base_mut().queue_redraw();
    }

    /// 设置玩家在当前地图内的格子（传 (-9999,-9999) 隐藏）
    #[func]
    pub fn set_player_cell(&mut self, cell: Vector2i) {
        self.player_cell = cell;
        self.has_player = cell.x > -9000;
        self.base_mut().queue_redraw();
    }

    /// 覆盖/新增地形颜色
    #[func]
    pub fn set_terrain_color(&mut self, name: GString, color: Color) {
        self.terrain_colors.insert(name.to_string(), color);
        self.base_mut().queue_redraw();
    }

    /// 内容像素尺寸（容器布局用）
    #[func]
    pub fn get_content_size(&mut self) -> Vector2 {
        if self.maps.is_empty() {
            return Vector2::ZERO;
        }
        let (min_x, min_y, max_x, max_y) = Self::content_bounds_cells(&self.maps);
        Vector2::new(
            (max_x - min_x) as f32 * self.cell_px + self.padding * 2.0,
            (max_y - min_y) as f32 * self.cell_px + self.padding * 2.0,
        )
    }
}

impl GdMiniMap {
    /// 拼接内容范围（格子单位）：min_x/min_y/max_x/max_y
    fn content_bounds_cells(maps: &Vec<MapThumb>) -> (i32, i32, i32, i32) {
        if maps.is_empty() {
            return (0, 0, 0, 0);
        }
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for m in maps {
            min_x = min_x.min(m.ox);
            min_y = min_y.min(m.oy);
            max_x = max_x.max(m.ox + m.w);
            max_y = max_y.max(m.oy + m.h);
        }
        (min_x, min_y, max_x, max_y)
    }

    fn update_content_size(&mut self) {
        // 聚焦模式：控件尺寸由宿主布局决定（anchor full），不反推最小尺寸
        if self.fit_current || self.maps.is_empty() {
            self.base_mut().set_custom_minimum_size(Vector2::ZERO);
            return;
        }
        let size = self.get_content_size();
        self.base_mut().set_custom_minimum_size(size);
    }

    /// 地图绘制原点（px）
    fn map_origin(
        maps: &Vec<MapThumb>,
        alias: &str,
        min_x: i32,
        min_y: i32,
        cell: f32,
        padding: f32,
    ) -> Option<Vector2> {
        let m = maps.iter().find(|m| m.alias == alias)?;
        Some(Vector2::new(
            (m.ox - min_x) as f32 * cell + padding,
            (m.oy - min_y) as f32 * cell + padding,
        ))
    }

    /// 传送点 i 的配对点：目标图中「落点旁（曼哈顿 ≤ 2）且指回本图」的传送点
    fn portal_pair(
        maps: &Vec<MapThumb>,
        portals: &Vec<PortalThumb>,
        i: usize,
        min_x: i32,
        min_y: i32,
        cell: f32,
        padding: f32,
    ) -> Option<(Vector2, Vector2)> {
        let a = &portals[i];
        let a_pos = Self::map_origin(maps, &a.alias, min_x, min_y, cell, padding)?;
        for (j, b) in portals.iter().enumerate() {
            if j == i || b.alias != a.to_alias || b.to_alias != a.alias {
                continue;
            }
            let md = (b.cell.x - a.to_cell.x).abs() + (b.cell.y - a.to_cell.y).abs();
            if md > 2 {
                continue;
            }
            let b_pos = Self::map_origin(maps, &b.alias, min_x, min_y, cell, padding)?;
            let ca =
                a_pos + Vector2::new((a.cell.x as f32 + 0.5) * cell, (a.cell.y as f32 + 0.5) * cell);
            let cb =
                b_pos + Vector2::new((b.cell.x as f32 + 0.5) * cell, (b.cell.y as f32 + 0.5) * cell);
            return Some((ca, cb));
        }
        None
    }
}
