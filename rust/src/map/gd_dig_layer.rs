// GdDigLayer - 支持挖地（地形变化）的双网格 TileMapLayer 地图系统
//
// 参考 example/map/Jess.tscn（TileMapDual 插件）的双网格渲染语义：
//   显示网格相对世界网格偏移半格，每个显示格按其覆盖的 4 个世界格角点
//   的地形组合查 4x4 过渡图集（16 组合，dual_grid_atlas_coord），
//   圆角过渡天然平滑；图集需把铺底地形背景烘进贴图（如 grass_sand.png
//   = 沙底 + 草形状），单层即可完成渲染，无需 GdQuickMap 的多层堆叠。
//
// 与 GdQuickMap 的分工：
//   GdQuickMap  = 噪声生成的大地图（多地形多层、连通性、高度场、deco 挂载）
//   GdDigLayer  = 手工布置的小地图（home/室内/农田）：铺底地形 + 形状地形
//                 + init_rects 手绘补丁；核心能力是 dig()——把某格地形挖成
//                 目标地形（如草→沙），局部刷新受影响显示格，信号通知宿主。
//
// 地图根协议（与 GdMapManager / GdRoleMover / player.gd 的 duck-type 对接）：
//   is_walkable / world_to_cell / cell_to_center / get_map_width /
//   get_map_height / get_cell_size_px——可直接作为地图场景根节点使用。
//
// 持久化：宿主（demo 侧 Bean）经 get_terrain_grid / set_terrain_grid 整网格
// dump/restore——地图小（百格级）整存最简且天然覆盖任意地形变化路径。
//
// 坐标约定（同 GdQuickMap）：格子 (cx,cy) 占据世界矩形
// [cx*cell,(cx+1)*cell) x [cy*cell,(cy+1)*cell)，格心 ((cx+0.5)*cell,...)。
// 显示格 (dx,dy) 覆盖世界格 (dx-1,dy-1)..(dx,dy) 四角，自身节点在 ready
// 时偏移半格（-cell/2），显示格图块中心恰落在世界格角点上。

use godot::prelude::*;
use godot::builtin::{Array, GString, PackedInt32Array, PackedStringArray, Rect2i, Vector2, Vector2i};
use godot::classes::{ITileMapLayer, Image, ImageTexture, Material, ResourceLoader, Shader, ShaderMaterial, Sprite2D, TileMapLayer, TileSet, TileSetAtlasSource, TileSetSource, Texture2D};
use godot::global::godot_warn;
use godot::obj::WithBaseField;

use super::dual_grid::dual_grid_atlas_coord;
use super::terrain_tex;

#[derive(GodotClass)]
#[class(base = TileMapLayer, tool)]
pub struct GdDigLayer {
    /// 地图宽度（世界格数）
    #[export]
    width: i32,
    /// 地图高度（世界格数）
    #[export]
    height: i32,
    /// 世界格尺寸（像素）
    #[export]
    cell_size: i32,
    /// 4x4 双网格过渡图集路径（16 格；铺底地形背景 + 形状地形圆角，
    /// 如 grass_sand.png）。空 = 纯色模式（形状地形用高亮色块渲染，零素材调试用）
    #[export]
    atlas_path: GString,
    /// 地形名：下标 0 = 铺底地形（填满整图），下标 1 = 形状地形（图集圆角形状）。
    /// 3 个以上名字仅作为模型数据（渲染一律按"非铺底 = 形状"处理，
    /// 单图集无法区分多种形状地形，语义见类注释）
    #[export]
    terrain_names: PackedStringArray,
    /// 挖掘目标地形名（与 terrain_names 平行；空 = 该地形不可挖）
    #[export]
    dig_targets: PackedStringArray,
    /// 初始地形补丁矩形（与 init_terrains 平行，按顺序应用后者覆盖前者；
    /// 铺底地形无需配置——天然填满）
    #[export]
    init_rects: Array<Rect2i>,
    /// 初始地形补丁地形名（与 init_rects 平行）
    #[export]
    init_terrains: PackedStringArray,
    /// 可挖区域限制（世界格矩形）；空 = 全图按 dig_targets 允许挖掘
    #[export]
    dig_rects: Array<Rect2i>,
    /// 不可通行地形名
    #[export]
    blocked_terrains: PackedStringArray,
    /// 纯色模式下形状地形的颜色（atlas_path 为空时生效）
    #[export]
    fallback_overlay_color: Color,
    /// 地形 shader 贴图路径（可选，与 terrain_names 按下标对应，如草地草纹）：
    /// 内置 "terrain_tex" shader 挂本图层（单层无法按层分材质，遮罩内建——
    /// 最多 3 个贴图地形按 mask r/g/b 通道区分），贴图以世界坐标 1:1
    /// 重复采样，只有遮罩命中的格子生效。编辑器改值后需调 rebuild() 生效
    #[export]
    terrain_shader_textures: PackedStringArray,
    /// 挖地预览光标贴图（可选，编辑器设置；如 cursor.svg）：配置后 ready 时
    /// 创建 DigCursor 精灵子节点，宿主（锄类工具）经 set_cursor_cell /
    /// clear_cursor 驱动——指示「下一锄将处理的格子」，只在 is_diggable
    /// 的格子显示（所见即所锄）
    #[export]
    cursor_texture: Option<Gd<Texture2D>>,

    /// 每格地形下标（行主序，width*height；ready 时按导出配置初始化）
    grid: Vec<i32>,
    /// 地形名缓存（与 terrain_names 同步）
    names: Vec<String>,
    /// 不可通行地形下标缓存
    blocked: Vec<i32>,
    /// 每地形挖掘目标下标缓存（-1 = 不可挖）
    dig_to: Vec<i32>,
    /// dig_rects 是否为空缓存（false = 全图允许）
    has_dig_area: bool,
    /// 纯色模式渲染用的铺底/形状颜色
    fallback_base_color: Color,
    /// shader 贴图逐格遮罩（r/g/b 通道 = 第 1/2/3 个贴图地形）
    mask_img: Option<Gd<Image>>,
    /// 遮罩对应的 GPU 纹理（挂进 ShaderMaterial）
    mask_tex: Option<Gd<ImageTexture>>,
    /// 挖地预览光标精灵（cursor_texture 配置后 ready 创建，初始隐藏）
    cursor: Option<Gd<Sprite2D>>,

    base: Base<TileMapLayer>,
}

#[godot_api]
impl ITileMapLayer for GdDigLayer {
    fn init(base: Base<TileMapLayer>) -> Self {
        Self {
            width: 20,
            height: 14,
            cell_size: 32,
            atlas_path: GString::new(),
            terrain_names: PackedStringArray::new(),
            dig_targets: PackedStringArray::new(),
            init_rects: Array::new(),
            init_terrains: PackedStringArray::new(),
            dig_rects: Array::new(),
            blocked_terrains: PackedStringArray::new(),
            fallback_overlay_color: Color::from_rgb(0.85, 0.65, 0.35),
            terrain_shader_textures: PackedStringArray::new(),
            cursor_texture: None,
            grid: Vec::new(),
            names: Vec::new(),
            blocked: Vec::new(),
            dig_to: Vec::new(),
            has_dig_area: false,
            fallback_base_color: Color::from_rgb(0.72, 0.55, 0.42),
            mask_img: None,
            mask_tex: None,
            cursor: None,
            base,
        }
    }

    fn ready(&mut self) {
        // 仅装配自身（tileset/格子/偏移），无全局副作用
        // z_index = -1：瓦片画在本层自身画布上，gl_compatibility 下运行时
        // 动态挂载的地图会盖住场景树更靠后的角色（GdQuickMap 的瓦片子层
        // 同为 z=-1 所以无此问题）——与 GdQuickMap 图层约定对齐
        self.base_mut().set_z_index(-1);
        self.sync_tables();
        if self.grid.is_empty() {
            self.init_grid();
        }
        self.rebuild_visual();
        self.setup_cursor();
    }
}

#[godot_api]
impl GdDigLayer {
    /// 挖地成功信号（参数：格子、原地形名、新地形名）——宿主据此联动入包/存档
    #[signal]
    fn s_dug(cell: Vector2i, from_terrain: GString, to_terrain: GString);

    /// 地形变化信号（dig 与 set_terrain_name_at 统一发出）
    #[signal]
    fn s_terrain_changed(cell: Vector2i, from_terrain: GString, to_terrain: GString);

    /// 挖地：把 cell 的地形变成 dig_targets 配置的目标地形。
    /// 返回是否成功（越界 / 不可挖地形 / 不在可挖区域内均失败）。
    /// 成功时局部刷新受影响显示格并发出 s_dug + s_terrain_changed。
    #[func]
    pub fn dig(&mut self, cell: Vector2i) -> bool {
        let Some(ti) = self.terrain_index_at(cell.x, cell.y) else {
            return false;
        };
        let target = match self.dig_to.get(ti as usize).copied().unwrap_or(-1) {
            t if t >= 0 => t,
            _ => return false, // 该地形不可挖
        };
        if self.has_dig_area && !self.in_dig_rects(cell) {
            return false; // 不在可挖区域内
        }
        let i = self.cell_index(cell.x, cell.y);
        if self.grid[i] == target {
            return false; // 已是目标地形
        }
        self.grid[i] = target;
        self.refresh_display_around(cell);
        self.refresh_shader_mask();
        let from = self.gstr_name(ti);
        let to = self.gstr_name(target);
        self.base_mut().emit_signal(
            "s_dug",
            &[cell.to_variant(), from.to_variant(), to.to_variant()],
        );
        self.emit_changed_signal(cell, from, to);
        true
    }

    /// 查询格子是否可挖（可挖地形且（无限制或在 dig_rects 内））
    #[func]
    pub fn is_diggable(&self, cell: Vector2i) -> bool {
        let Some(ti) = self.terrain_index_at(cell.x, cell.y) else {
            return false;
        };
        if self.dig_to.get(ti as usize).copied().unwrap_or(-1) < 0 {
            return false;
        }
        !self.has_dig_area || self.in_dig_rects(cell)
    }

    /// 查询格子地形名（越界返回空串）
    #[func]
    pub fn get_terrain_name_at(&self, cell: Vector2i) -> GString {
        match self.terrain_index_at(cell.x, cell.y) {
            Some(ti) => self.gstr_name(ti),
            None => GString::new(),
        }
    }

    /// 手动设置格子地形（按名；越界/未知地形名忽略）。地形变化统一入口，
    /// 与 dig 共用刷新与信号（s_terrain_changed）
    #[func]
    pub fn set_terrain_name_at(&mut self, cell: Vector2i, terrain_name: GString) {
        let Some(ti) = self.terrain_index_at(cell.x, cell.y) else {
            return;
        };
        let Some(new_ti) = self.name_index(terrain_name.to_string()) else {
            godot_warn!(
                "GdDigLayer: 未知地形名 \"{}\"（set_terrain_name_at 忽略）",
                terrain_name
            );
            return;
        };
        let i = self.cell_index(cell.x, cell.y);
        if self.grid[i] == new_ti {
            return;
        }
        let old = self.gstr_name(ti);
        self.grid[i] = new_ti;
        self.refresh_display_around(cell);
        self.refresh_shader_mask();
        self.emit_changed_signal(cell, old, self.gstr_name(new_ti));
    }

    /// 格子是否可通行（越界不可通行；blocked_terrains 中的地形不可通行）
    #[func]
    pub fn is_walkable(&self, cell: Vector2i) -> bool {
        match self.terrain_index_at(cell.x, cell.y) {
            Some(ti) => !self.blocked.contains(&ti),
            None => false,
        }
    }

    /// 地图宽度（世界格数；MapManager duck-type 协议）
    #[func]
    pub fn get_map_width(&self) -> i32 {
        self.width
    }

    /// 地图高度（世界格数；MapManager duck-type 协议）
    #[func]
    pub fn get_map_height(&self) -> i32 {
        self.height
    }

    /// 世界格尺寸（MapManager duck-type 协议）
    #[func]
    pub fn get_cell_size_px(&self) -> i32 {
        self.cell_size
    }

    /// 世界坐标 → 格子坐标（纯除法全局语义，同 GdQuickMap）
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

    /// 格子中心的世界坐标
    #[func]
    pub fn cell_to_center(&self, cell: Vector2i) -> Vector2 {
        let cs = self.cell_size as f32;
        Vector2::new((cell.x as f32 + 0.5) * cs, (cell.y as f32 + 0.5) * cs)
    }

    /// 整网格 dump（行主序地形下标；宿主持久化用）
    #[func]
    pub fn get_terrain_grid(&self) -> PackedInt32Array {
        self.grid.iter().copied().collect()
    }

    /// 整网格 restore（长度须等于 width*height；成功后全量重铺）。
    /// 宿主在 ready 之后调用（覆盖导出配置的初始地形）
    #[func]
    pub fn set_terrain_grid(&mut self, grid: PackedInt32Array) -> bool {
        let total = (self.width.max(0) * self.height.max(0)) as usize;
        if grid.len() != total {
            godot_warn!(
                "GdDigLayer: set_terrain_grid 长度 {} != 期望 {}，忽略",
                grid.len(),
                total
            );
            return false;
        }
        self.grid = grid.to_vec();
        // 防御越界值（地形下标非法按铺底处理）
        let n = self.names.len().max(1) as i32;
        for v in self.grid.iter_mut() {
            if *v < 0 || *v >= n {
                *v = 0;
            }
        }
        self.rebuild_visual();
        true
    }

    /// 按当前导出配置重建（编辑器调参后手动刷新；会清空运行期地形变化）
    #[func]
    pub fn rebuild(&mut self) {
        self.sync_tables();
        self.grid.clear();
        self.init_grid();
        self.rebuild_visual();
    }

    /// 调试/测试：读 shader 贴图遮罩某格的通道颜色
    /// （r/g/b = 第 1/2/3 个贴图地形；headless 哑渲染器下 GPU 纹理不更新，
    /// 权威数据源是内存 Image 本体，经此读取）
    #[func]
    pub fn get_shader_mask_at(&self, cell: Vector2i) -> Color {
        let Some(img) = &self.mask_img else {
            return Color::from_rgba(0.0, 0.0, 0.0, 0.0);
        };
        if cell.x < 0 || cell.y < 0 || cell.x >= self.width || cell.y >= self.height {
            return Color::from_rgba(0.0, 0.0, 0.0, 0.0);
        }
        img.get_pixel(cell.x, cell.y)
    }

    /// 显示挖地预览光标于 cell（仅 is_diggable 的格子显示——rects 外/
    /// 不可挖地形自动隐藏；宿主每帧同步「下一锄将处理的格子」）
    #[func]
    pub fn set_cursor_cell(&mut self, cell: Vector2i) {
        if self.cursor.is_none() {
            return;
        }
        let show = self.is_diggable(cell);
        // 世界语义格心 → 根局部坐标：根带半格偏移 + 图集缩放（cs/ts_px），
        // 格心直接当局部坐标用会随根缩放 2 倍飘移（双网格下光标飞远+压多格）
        let world = self.cell_to_center(cell);
        let local = self.base_mut().to_local(world);
        let Some(spr) = self.cursor.as_mut() else {
            return;
        };
        spr.set_visible(show);
        if show {
            spr.set_position(local);
        }
    }

    /// 隐藏挖地预览光标（卸下工具/换图时调用；无光标配置时无害）
    #[func]
    pub fn clear_cursor(&mut self) {
        if let Some(spr) = self.cursor.as_mut() {
            spr.set_visible(false);
        }
    }

    // ---- 内部实现 ----

    /// 装配挖地预览光标（cursor_texture 配置后创建，初始隐藏；
    /// 纯本地子节点，无全局副作用）
    fn setup_cursor(&mut self) {
        let Some(tex) = self.cursor_texture.clone() else {
            return;
        };
        let mut spr = Sprite2D::new_alloc();
        spr.set_name("DigCursor");
        spr.set_texture(&tex);
        spr.set_visible(false);
        // 缩放到单格大小（世界像素）：精灵在根局部空间，根带 cs/ts_px 缩放，
        // 局部缩放 = (cell_size / 根缩放) / 贴图长边——任意尺寸素材 + 任意
        // 图集分辨率都恰好铺满一个世界格
        let tw = tex.get_width().max(1) as f32;
        let th = tex.get_height().max(1) as f32;
        let root_scale = self.base_mut().get_scale().x.max(0.01);
        let s = self.cell_size.max(1) as f32 / root_scale / tw.max(th);
        spr.set_scale(Vector2::new(s, s));
        // 相对 z=1 → 绝对 0：盖住瓦片（本层根 z=-1），树序在玩家前（让位角色）
        spr.set_z_index(1);
        self.base_mut().add_child(&spr);
        self.cursor = Some(spr);
    }

    /// 解析导出并行数组 → 运行时缓存（names/blocked/dig_to/has_dig_area）
    fn sync_tables(&mut self) {
        self.names = self
            .terrain_names
            .to_vec()
            .iter()
            .map(|s| s.to_string())
            .collect();
        if self.names.is_empty() {
            // 零配置兜底：单铺底地形（纯色渲染仍可跑通）
            self.names.push("base".to_string());
        }
        self.blocked = self
            .blocked_terrains
            .to_vec()
            .iter()
            .filter_map(|s| self.name_index(s.to_string()))
            .collect();
        self.dig_to = self
            .dig_targets
            .to_vec()
            .iter()
            .map(|s| match self.name_index(s.to_string()) {
                Some(ti) => ti,
                None => -1,
            })
            .collect();
        self.dig_to.resize(self.names.len(), -1);
        self.has_dig_area = self.dig_rects.len() > 0;
    }

    /// 初始网格：铺底填满 + init_rects 按序叠加
    fn init_grid(&mut self) {
        let w = self.width.max(0) as usize;
        let h = self.height.max(0) as usize;
        self.grid = vec![0; w * h];
        for (ri, rect) in self.init_rects.iter_shared().enumerate() {
            let Some(ti) = self
                .init_terrains
                .get(ri)
                .and_then(|s| self.name_index(s.to_string()))
            else {
                godot_warn!(
                    "GdDigLayer: init_terrains[{}] 未知名/缺失，跳过补丁 {}",
                    ri,
                    rect
                );
                continue;
            };
            let x0 = rect.position.x.max(0);
            let y0 = rect.position.y.max(0);
            let x1 = (rect.position.x + rect.size.x).min(self.width.max(0));
            let y1 = (rect.position.y + rect.size.y).min(self.height.max(0));
            for cy in y0..y1 {
                for cx in x0..x1 {
                    self.grid[(cy * self.width.max(0) + cx) as usize] = ti;
                }
            }
        }
    }

    /// 全量重铺：tileset 装配 + 所有显示格重画 + shader 贴图材质
    fn rebuild_visual(&mut self) {
        let cs = self.cell_size.max(1) as f32;
        if self.atlas_path.is_empty() {
            self.rebuild_visual_plain(cs);
        } else {
            self.rebuild_visual_atlas(cs);
        }
        self.apply_terrain_shader_material();
    }

    /// 图集模式：4x4 过渡图集，显示网格偏移半格 + 缩放到 cell_size
    fn rebuild_visual_atlas(&mut self, cs: f32) -> Option<()> {
        let path = self.atlas_path.to_string();
        let tex = Self::load_texture(&path)?;
        let ts_px = (tex.get_width().max(4) / 4).max(1);
        let ts = Self::build_tile_set(&tex, ts_px);
        self.base_mut().set_tile_set(&ts);
        self.base_mut()
            .set_position(Vector2::new(-cs / 2.0, -cs / 2.0));
        self.base_mut().set_scale(Vector2::new(
            cs / ts_px as f32,
            cs / ts_px as f32,
        ));
        self.repaint_all();
        Some(())
    }

    /// 纯色模式（atlas_path 空）：程序化 2 色 4x4 图集，零素材调试用。
    /// 每个过渡 tile 按“四角有无形状地形”填充对应象限（方块近似，无圆角）
    fn rebuild_visual_plain(&mut self, cs: f32) {
        const TPX: i32 = 16;
        let mut img = Self::color_image(self.fallback_base_color, TPX * 4);
        let overlay = self.fallback_overlay_color;
        // 遍历 16 组合 → 查表得图集坐标 → 按标记填充该 tile 的对应象限
        // （显示 tile 的四象限 = 四个世界格角点：左上/右上/左下/右下）
        for ul in [false, true] {
            for ur in [false, true] {
                for dl in [false, true] {
                    for dr in [false, true] {
                        let (ax, ay) = dual_grid_atlas_coord(ul, ur, dl, dr);
                        let (ox, oy) = (ax * TPX, ay * TPX);
                        let half = TPX / 2;
                        if ul {
                            img.fill_rect(Rect2i::new(Vector2i::new(ox, oy), Vector2i::new(half, half)), overlay);
                        }
                        if ur {
                            img.fill_rect(Rect2i::new(Vector2i::new(ox + half, oy), Vector2i::new(half, half)), overlay);
                        }
                        if dl {
                            img.fill_rect(Rect2i::new(Vector2i::new(ox, oy + half), Vector2i::new(half, half)), overlay);
                        }
                        if dr {
                            img.fill_rect(Rect2i::new(Vector2i::new(ox + half, oy + half), Vector2i::new(half, half)), overlay);
                        }
                    }
                }
            }
        }
        let tex = godot::classes::ImageTexture::create_from_image(&img)
            .map(|t| t.upcast::<Texture2D>());
        let Some(tex) = tex else { return };
        let ts = Self::build_tile_set(&tex, TPX);
        self.base_mut().set_tile_set(&ts);
        self.base_mut()
            .set_position(Vector2::new(-cs / 2.0, -cs / 2.0));
        self.base_mut()
            .set_scale(Vector2::new(cs / TPX as f32, cs / TPX as f32));
        self.repaint_all();
    }

    /// 重画全部显示格（0..=width x 0..=height）
    fn repaint_all(&mut self) {
        let Some(sid) = self.own_source_id() else { return };
        self.base_mut().clear();
        for dy in 0..=self.height {
            for dx in 0..=self.width {
                self.paint_display(sid, dx, dy);
            }
        }
    }

    /// 画单个显示格：四角地形组合 → 过渡图块（铺底填满，恒有图可画）
    fn paint_display(&mut self, sid: i32, dx: i32, dy: i32) {
        // 显示格 (dx,dy) 覆盖世界格 (dx-1,dy-1)..(dx,dy)
        let quads = [
            self.terrain_index_at(dx - 1, dy - 1),
            self.terrain_index_at(dx, dy - 1),
            self.terrain_index_at(dx - 1, dy),
            self.terrain_index_at(dx, dy),
        ];
        // 形状标记：角点为非铺底地形（越界视为铺底）
        let is_shape = |q: Option<i32>| matches!(q, Some(t) if t >= 1);
        let (ax, ay) = dual_grid_atlas_coord(
            is_shape(quads[0]),
            is_shape(quads[1]),
            is_shape(quads[2]),
            is_shape(quads[3]),
        );
        self.base_mut()
            .set_cell_ex(Vector2i::new(dx, dy))
            .source_id(sid)
            .atlas_coords(Vector2i::new(ax, ay))
            .done();
    }

    /// 世界格 (cx,cy) 地形变化后，重画受影响显示格（其覆盖的 4 格）
    fn refresh_display_around(&mut self, cell: Vector2i) {
        let Some(sid) = self.own_source_id() else { return };
        for (dx, dy) in [
            (cell.x, cell.y),
            (cell.x + 1, cell.y),
            (cell.x, cell.y + 1),
            (cell.x + 1, cell.y + 1),
        ] {
            self.paint_display(sid, dx, dy);
        }
    }

    /// 本节点 tileset 的第一个图集 source id（无 tileset/源返回 None）
    fn own_source_id(&self) -> Option<i32> {
        let ts = self.base().get_tile_set()?;
        if ts.get_source_count() > 0 {
            Some(ts.get_source_id(0))
        } else {
            None
        }
    }

    /// 格子地形下标（越界 None）
    fn terrain_index_at(&self, cx: i32, cy: i32) -> Option<i32> {
        if self.width <= 0 || self.height <= 0 || self.grid.is_empty() {
            return None;
        }
        if cx < 0 || cy < 0 || cx >= self.width || cy >= self.height {
            return None;
        }
        self.grid.get((cy * self.width + cx) as usize).copied()
    }

    fn cell_index(&self, cx: i32, cy: i32) -> usize {
        (cy * self.width + cx) as usize
    }

    fn name_index(&self, name: String) -> Option<i32> {
        self.names.iter().position(|s| *s == name).map(|i| i as i32)
    }

    fn gstr_name(&self, ti: i32) -> GString {
        self.names
            .get(ti as usize)
            .map(|s| GString::from(s.as_str()))
            .unwrap_or_default()
    }

    /// 是否在任一 dig_rect 内
    fn in_dig_rects(&self, cell: Vector2i) -> bool {
        for rect in self.dig_rects.iter_shared() {
            if cell.x >= rect.position.x
                && cell.y >= rect.position.y
                && cell.x < rect.position.x + rect.size.x
                && cell.y < rect.position.y + rect.size.y
            {
                return true;
            }
        }
        false
    }

    fn emit_changed_signal(&mut self, cell: Vector2i, from: GString, to: GString) {
        self.base_mut().emit_signal(
            "s_terrain_changed",
            &[cell.to_variant(), from.to_variant(), to.to_variant()],
        );
    }

    // ---- 地形 shader 贴图（terrain_shader_textures）----

    /// 收集配置了贴图的地形：[(地形下标, 贴图路径)]，按 terrain_names 顺序，
    /// 最多 3 个（内置遮罩只有 r/g/b 三个通道）
    fn shader_slots(&self) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        for ti in 0..self.names.len() {
            let path = self
                .terrain_shader_textures
                .get(ti)
                .map(|s| s.to_string())
                .unwrap_or_default();
            if !path.is_empty() {
                if out.len() >= 3 {
                    godot_warn!(
                        "GdDigLayer: terrain_shader_textures 超过 3 个（遮罩仅 r/g/b 通道），多余项忽略"
                    );
                    break;
                }
                out.push((ti, path));
            }
        }
        out
    }

    /// 给根图层挂内置 terrain_tex 材质：逐格遮罩（r/g/b = 第 1/2/3 个
    /// 贴图地形）+ 各贴图世界坐标平铺。无配置时清掉材质
    fn apply_terrain_shader_material(&mut self) {
        let slots = self.shader_slots();
        if slots.is_empty() {
            self.mask_img = None;
            self.mask_tex = None;
            self.base_mut().set_material(Gd::<Material>::null_arg());
            return;
        }
        let w = self.width.max(1);
        let h = self.height.max(1);
        let mut img = Image::create_empty(w, h, false, godot::classes::image::Format::RGBA8)
            .expect("create mask image");
        for cy in 0..h {
            for cx in 0..w {
                let ti = self.terrain_index_at(cx, cy);
                if let Some(slot) = slots.iter().position(|(t, _)| Some(*t as i32) == ti) {
                    let c = match slot {
                        0 => Color::from_rgba(1.0, 0.0, 0.0, 1.0),
                        1 => Color::from_rgba(0.0, 1.0, 0.0, 1.0),
                        _ => Color::from_rgba(0.0, 0.0, 1.0, 1.0),
                    };
                    img.set_pixel(cx, cy, c);
                }
            }
        }
        let tex = ImageTexture::create_from_image(&img).expect("create mask texture");
        let mut mat = ShaderMaterial::new_gd();
        let mut shader = Shader::new_gd();
        shader.set_code(terrain_tex::TERRAIN_TEX_SHADER_SRC);
        mat.set_shader(&shader);
        let mask_v = tex.clone().upcast::<Texture2D>().to_variant();
        mat.set_shader_parameter("mask_tex", &mask_v);
        let cells_v = Vector2::new(w as f32, h as f32).to_variant();
        mat.set_shader_parameter("map_cells", &cells_v);
        let cs = self.cell_size.max(1) as f32;
        let cell_v = Vector2::new(cs, cs).to_variant();
        mat.set_shader_parameter("cell_px", &cell_v);
        for (slot, (_, path)) in slots.iter().enumerate() {
            match Self::load_texture(path) {
                Some(tex) => {
                    let tex_v = tex.clone().to_variant();
                    mat.set_shader_parameter(format!("tex{}", slot).as_str(), &tex_v);
                    let size_v = Vector2::new(tex.get_width() as f32, tex.get_height() as f32)
                        .to_variant();
                    mat.set_shader_parameter(format!("tex_size{}", slot).as_str(), &size_v);
                }
                None => {
                    godot_warn!("GdDigLayer: shader 贴图加载失败: {}", path);
                }
            }
        }
        // 未配置的 tex 槽位占位为全透明纹理（防 unset sampler 默认白被采样）
        for slot in slots.len()..3 {
            let v = terrain_tex::transparent_tex().to_variant();
            mat.set_shader_parameter(format!("tex{}", slot).as_str(), &v);
        }
        self.mask_img = Some(img);
        self.mask_tex = Some(tex);
        self.base_mut().set_material(&mat);
    }

    /// 地形归属变化后刷新遮罩（dig/set_terrain_name_at 调达；地图小全量重填）
    fn refresh_shader_mask(&mut self) {
        if self.shader_slots().is_empty() {
            return;
        }
        let slots = self.shader_slots();
        let grid = self.grid.clone();
        let (Some(img), Some(tex)) = (&mut self.mask_img, &mut self.mask_tex) else {
            return;
        };
        let w = self.width.max(1);
        let h = self.height.max(1);
        img.fill(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        for cy in 0..h {
            for cx in 0..w {
                let ti = grid.get((cy * w + cx) as usize).copied();
                if let Some(slot) = slots.iter().position(|(t, _)| Some(*t as i32) == ti) {
                    let c = match slot {
                        0 => Color::from_rgba(1.0, 0.0, 0.0, 1.0),
                        1 => Color::from_rgba(0.0, 1.0, 0.0, 1.0),
                        _ => Color::from_rgba(0.0, 0.0, 1.0, 1.0),
                    };
                    img.set_pixel(cx, cy, c);
                }
            }
        }
        tex.update(&*img);
    }

    fn load_texture(path: &str) -> Option<Gd<Texture2D>> {
        if path.is_empty() {
            return None;
        }
        let res = ResourceLoader::singleton().load_ex(path).done()?;
        res.try_cast::<Texture2D>().ok()
    }

    /// 构建 TileSet：单图集源，格尺寸 = ts_px，图集所有格注册为 tile（同 GdQuickMap）
    fn build_tile_set(tex: &Gd<Texture2D>, ts_px: i32) -> Gd<TileSet> {
        let mut ts = TileSet::new_gd();
        ts.set_tile_size(Vector2i::new(ts_px, ts_px));
        let mut src = TileSetAtlasSource::new_gd();
        src.set_texture(tex);
        src.set_texture_region_size(Vector2i::new(ts_px, ts_px));
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

    fn color_image(color: Color, size: i32) -> Gd<godot::classes::Image> {
        let mut img = godot::classes::Image::create_empty(size, size, false, godot::classes::image::Format::RGBA8)
            .expect("create image");
        img.fill_rect(Rect2i::new(Vector2i::ZERO, Vector2i::new(size, size)), color);
        img
    }
}
