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
use godot::classes::{Engine, ITileMapLayer, Image, ImageTexture, Material, ResourceLoader, Shader, ShaderMaterial, Sprite2D, TileMapLayer, TileSet, TileSetAtlasSource, TileSetSource, Texture2D};
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
    /// 创建 ActionCursor（GdActionCursor）子节点，宿主（锄类工具）经
    /// set_cursor_cell / clear_cursor 驱动——指示「下一锄将处理的格子」，
    /// 只在 is_diggable 的格子显示（所见即所锄）；未配置贴图则用动作色框样式
    #[export]
    cursor_texture: Option<Gd<Texture2D>>,

    // ---- 农田状态层（状态流转：耕地 --浇→ 湿润 --种→ 作物 --生长→ 成熟）----
    /// 可浇灌地形名（已开垦耕地）；空 = 自动取 dig_targets 的非空目标集合
    #[export]
    waterable_terrains: PackedStringArray,
    /// 作物定义数组（多作物：每个 GdCropDef = 一种作物的贴图/阶段/时长/
    /// 种子/产物，见 crop_def.rs）。播种按 seed_id 匹配 def，各作物独立
    /// 生长节奏与贴图；空数组 = 单一默认作物（程序化占位幼苗）
    #[export]
    crops: Array<Gd<crate::map::crop_def::GdCropDef>>,
    /// 湿润持续秒数（超时自然干燥、作物暂停生长）；0 = 永不干燥
    #[export]
    wet_secs: f64,

    /// 每格湿润标记（与 grid 平行；农场层运行时状态）
    wet: Vec<bool>,
    /// 每格湿润已持续秒（干燥判定累计）
    wet_timer: Vec<f32>,
    /// 每格作物阶段（-1 = 无作物）
    crop_stage: Vec<i32>,
    /// 每格作物种类（crops 数组下标；-1 = 无作物）
    crop_kind: Vec<i32>,
    /// 每格当前阶段已生长秒
    crop_timer: Vec<f32>,
    /// 可浇灌地形下标缓存（sync_tables 填充）
    waterable: Vec<i32>,
    /// 湿润逐格遮罩（shader wet_mask r 通道，命中格基色调暗）
    wet_img: Option<Gd<Image>>,
    /// 湿润遮罩 GPU 纹理
    wet_tex: Option<Gd<ImageTexture>>,
    /// 作物精灵（键 = (cx, cy)）
    crop_sprites: std::collections::HashMap<(i32, i32), Gd<Sprite2D>>,
    /// 各作物序列帧纹理缓存（下标 = crops 数组下标；def.texture 或程序化占位）
    crop_sheets: Vec<Option<Gd<Texture2D>>>,

    /// 显示子层（TileMapLayer）：承接 tileset/格子/双网格偏移缩放/材质。
    /// **架构红线：地图根节点必须保持单位变换（位置 0、缩放 1）**——
    /// 双网格显示需要半格偏移+图集缩放，全部打在本子层上（与 GdQuickMap
    /// 的 layer 子层同范式）。根若带变换，GdMapMarker/光标/作物等一切
    /// 子组件的「格心 = (cell+0.5)*cell_size」局部约定都会被放大漂移
    /// （历史 bug：home 传送点被甩出地图不可见）。
    display: Option<Gd<TileMapLayer>>,

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
    /// 动作预览光标（GdActionCursor 组件，ready 创建，初始隐藏；
    /// 显示/动画/定位在组件内，本层只做农田谓词判定后驱动）
    cursor: Option<Gd<super::gd_action_cursor::GdActionCursor>>,

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
            waterable_terrains: PackedStringArray::new(),
            crops: Array::new(),
            wet_secs: 60.0,
            grid: Vec::new(),
            names: Vec::new(),
            blocked: Vec::new(),
            dig_to: Vec::new(),
            has_dig_area: false,
            fallback_base_color: Color::from_rgb(0.72, 0.55, 0.42),
            mask_img: None,
            mask_tex: None,
            cursor: None,
            wet: Vec::new(),
            wet_timer: Vec::new(),
            crop_stage: Vec::new(),
            crop_kind: Vec::new(),
            crop_timer: Vec::new(),
            waterable: Vec::new(),
            wet_img: None,
            wet_tex: None,
            crop_sprites: std::collections::HashMap::new(),
            crop_sheets: Vec::new(),
            display: None,
            base,
        }
    }

    fn ready(&mut self) {
        // 仅装配自身（tileset/格子/偏移），无全局副作用
        // 显示下沉到 Display 子层：瓦片/双网格偏移/材质全在子层（z=-1），
        // 根保持单位变换——子组件（GdMapMarker/光标/作物）格心定位约定统一
        self.ensure_display();
        self.sync_tables();
        if self.grid.is_empty() {
            self.init_grid();
        }
        self.rebuild_visual();
        self.setup_cursor();
        // 农田层：网格对齐 + 湿润遮罩装配；无湿润/作物时关帧循环（空闲零开销）
        self.resize_farm();
        self.setup_wet_mask();
        self.base_mut().set_process(false);
    }

    fn process(&mut self, delta: f64) {
        self.tick_farm(delta as f32);
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

    // ---- 农田状态流转信号（宿主据此联动存档/演出）----
    /// 浇水成功（耕地 → 湿润）
    #[signal]
    fn s_watered(cell: Vector2i);
    /// 湿润超时自然干燥（wet_secs > 0 时）
    #[signal]
    fn s_dried(cell: Vector2i);
    /// 播种成功（湿润耕地 → 作物阶段 0）
    #[signal]
    fn s_planted(cell: Vector2i);
    /// 作物进入下一生长阶段（stage = 新阶段下标）
    #[signal]
    fn s_grew(cell: Vector2i, stage: i32);
    /// 作物成熟（到达最终阶段，生长停止，可收获）
    #[signal]
    fn s_matured(cell: Vector2i);
    /// 收获成功（作物移除，耕地保留湿润状态）
    #[signal]
    fn s_harvested(cell: Vector2i);

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

    /// 按当前导出配置重建（编辑器调参后手动刷新；会清空运行期地形/农田变化）
    #[func]
    pub fn rebuild(&mut self) {
        self.sync_tables();
        self.grid.clear();
        self.init_grid();
        self.rebuild_visual();
        self.clear_farm_runtime();
        self.resize_farm();
        self.setup_wet_mask();
        self.base_mut().set_process(false);
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

    /// 确保显示子层存在（ready 调达；tileset/偏移/缩放/材质全挂本子层）
    fn ensure_display(&mut self) {
        if self
            .display
            .as_ref()
            .is_some_and(|d| d.is_instance_valid())
        {
            return;
        }
        let mut layer = TileMapLayer::new_alloc();
        layer.set_name("Display");
        // z_index = -1：瓦片画在本子层画布上，gl_compatibility 下运行时
        // 动态挂载的地图会盖住场景树更靠后的角色——与 GdQuickMap 图层约定对齐
        layer.set_z_index(-1);
        self.base_mut().add_child(&layer);
        self.display = Some(layer);
    }

    /// 显示子层可变引用（未初始化时兜底创建，防 ready 前调用的健壮性）
    fn display_mut(&mut self) -> Gd<TileMapLayer> {
        self.ensure_display();
        self.display.clone().unwrap()
    }

    /// 显示子层（tileset/显示格/双网格偏移缩放/材质都在子层上；
    /// 根保持单位变换，格心定位等逻辑一律走根的世界语义）
    #[func]
    pub fn get_display_layer(&self) -> Option<Gd<TileMapLayer>> {
        self.display.clone()
    }

    /// 显示挖地预览光标于 cell（仅 is_diggable 的格子显示——rects 外/
    /// 不可挖地形自动隐藏；宿主每帧同步「下一锄将处理的格子」）
    #[func]
    pub fn set_cursor_cell(&mut self, cell: Vector2i) {
        let show = self.is_diggable(cell);
        if let Some(cur) = self.cursor.as_mut() {
            if show {
                cur.bind_mut().show_at(cell, GString::from("dig"));
            } else {
                cur.bind_mut().hide_cursor();
            }
        }
    }

    /// 隐藏挖地预览光标（卸下工具/换图时调用；无光标配置时无害）
    #[func]
    pub fn clear_cursor(&mut self) {
        if let Some(cur) = self.cursor.as_mut() {
            cur.bind_mut().hide_cursor();
        }
    }

    // ---- 农田状态流转：耕地 --浇→ 湿润 --种→ 作物 --生长→ 成熟 --收→ 耕地 ----

    /// 格子是否可浇灌（地形为已开垦耕地，且（无限制或在 dig_rects 内））
    #[func]
    pub fn is_waterable(&self, cell: Vector2i) -> bool {
        let Some(ti) = self.terrain_index_at(cell.x, cell.y) else {
            return false;
        };
        if !self.waterable.contains(&ti) {
            return false;
        }
        !self.has_dig_area || self.in_dig_rects(cell)
    }

    /// 浇水：耕地 → 湿润（已湿润返回 false）。湿润计时清零重新累计；
    /// 成功发 s_watered，湿润期间该格基色经 wet_mask 调暗
    #[func]
    pub fn water(&mut self, cell: Vector2i) -> bool {
        if !self.is_waterable(cell) {
            return false;
        }
        let i = self.cell_index(cell.x, cell.y);
        if self.wet.get(i).copied().unwrap_or(false) {
            return false;
        }
        self.wet[i] = true;
        self.wet_timer[i] = 0.0;
        self.refresh_wet_mask();
        self.base_mut().set_process(true);
        self.base_mut().emit_signal("s_watered", &[cell.to_variant()]);
        true
    }

    /// 格子当前是否湿润
    #[func]
    pub fn is_wet(&self, cell: Vector2i) -> bool {
        match self.farm_index(cell) {
            Some(i) => self.wet.get(i).copied().unwrap_or(false),
            None => false,
        }
    }

    /// 格子是否可播种（湿润耕地且无作物）
    #[func]
    pub fn is_plantable(&self, cell: Vector2i) -> bool {
        match self.farm_index(cell) {
            Some(i) => self.wet.get(i).copied().unwrap_or(false) && self.crop_stage[i] < 0,
            None => false,
        }
    }

    /// 播种：湿润耕地 → 作物阶段 0（无作物且湿润才成功）。发 s_planted。
    /// 单参版 = crops[0]（默认作物；测试/简单用法）
    #[func]
    pub fn plant(&mut self, cell: Vector2i) -> bool {
        self.plant_kind(cell, 0)
    }

    /// 按种子道具 id 播种：crops 里找 seed_id 匹配的 def（找不到 = false）。
    /// 玩家持不同种子播不同作物走这里
    #[func]
    pub fn plant_seed(&mut self, cell: Vector2i, seed_id: GString) -> bool {
        let kind = self.crop_kind_by_seed(&seed_id);
        if kind < 0 {
            return false;
        }
        self.plant_kind(cell, kind)
    }

    /// seed_id → crops 数组下标（空定义 = 单一默认作物 0；未匹配 = -1）
    fn crop_kind_by_seed(&self, seed_id: &GString) -> i32 {
        if self.crops.is_empty() {
            return 0;
        }
        for (k, def) in self.crops.iter_shared().enumerate() {
            if def.bind().seed_id == *seed_id {
                return k as i32;
            }
        }
        -1
    }

    /// 作物 def 参数（kind 越界/空定义 → 默认值；def 属性实时读取——
    /// 测试/调参改 def 即时生效，无需刷新缓存）
    fn crop_stages_of(&self, kind: i32) -> i32 {
        self.crops
            .get(kind.max(0) as usize)
            .map(|d| d.bind().stages)
            .unwrap_or(4)
            .max(1)
    }

    fn crop_grow_secs_of(&self, kind: i32) -> f64 {
        self.crops
            .get(kind.max(0) as usize)
            .map(|d| d.bind().grow_secs)
            .unwrap_or(15.0)
    }

    /// kind 下 def 的 id（越界/空定义 → ""）
    fn crop_id_of(&self, kind: i32) -> GString {
        self.crops
            .get(kind.max(0) as usize)
            .map(|d| d.bind().id.clone())
            .unwrap_or_default()
    }

    fn plant_kind(&mut self, cell: Vector2i, kind: i32) -> bool {
        if !self.is_plantable(cell) {
            return false;
        }
        let i = self.cell_index(cell.x, cell.y);
        self.crop_stage[i] = 0;
        self.crop_kind[i] = kind;
        self.crop_timer[i] = 0.0;
        self.ensure_crop_sprite(cell, 0, kind);
        self.base_mut().set_process(true);
        self.base_mut().emit_signal("s_planted", &[cell.to_variant()]);
        true
    }

    /// 格子作物阶段（-1 = 无作物；成熟 = 所属作物 stages - 1）
    #[func]
    pub fn get_crop_stage(&self, cell: Vector2i) -> i32 {
        match self.farm_index(cell) {
            Some(i) => self.crop_stage.get(i).copied().unwrap_or(-1),
            None => -1,
        }
    }

    /// 格子作物种类 id（所属 def 的 id 字段；无作物/越界 = ""）
    #[func]
    pub fn get_crop_id(&self, cell: Vector2i) -> GString {
        let kind = match self.farm_index(cell) {
            Some(i) => self.crop_kind.get(i).copied().unwrap_or(-1),
            None => -1,
        };
        if kind < 0 {
            return GString::new();
        }
        self.crop_id_of(kind)
    }

    /// 格子成熟后收获产物道具 id（无作物/未成熟/空定义 = ""；玩家采摘前查询）
    #[func]
    pub fn get_harvest_item(&self, cell: Vector2i) -> GString {
        if !self.is_mature(cell) {
            return GString::new();
        }
        let kind = match self.farm_index(cell) {
            Some(i) => self.crop_kind.get(i).copied().unwrap_or(-1),
            None => -1,
        };
        if kind < 0 {
            return GString::new();
        }
        self.crops
            .get(kind as usize)
            .map(|d| d.bind().harvest_item.clone())
            .unwrap_or_default()
    }

    /// 格子作物是否成熟（按所属作物各自的 stages 判定）
    #[func]
    pub fn is_mature(&self, cell: Vector2i) -> bool {
        let (kind, stage) = match self.farm_index(cell) {
            Some(i) => (
                self.crop_kind.get(i).copied().unwrap_or(-1),
                self.crop_stage.get(i).copied().unwrap_or(-1),
            ),
            None => (-1, -1),
        };
        stage >= 0 && stage >= self.crop_stages_of(kind) - 1
    }

    /// 收获：成熟作物移除（耕地保留湿润状态），发 s_harvested。
    /// 未成熟/无作物返回 false
    #[func]
    pub fn harvest(&mut self, cell: Vector2i) -> bool {
        if !self.is_mature(cell) {
            return false;
        }
        let i = self.cell_index(cell.x, cell.y);
        self.crop_stage[i] = -1;
        self.crop_kind[i] = -1;
        self.crop_timer[i] = 0.0;
        self.remove_crop_sprite(cell);
        self.base_mut().emit_signal("s_harvested", &[cell.to_variant()]);
        true
    }

    /// 农田状态 dump（宿主持久化用）：
    /// {"wet": {"x,y": true}, "crops": {"x,y": {"stage": float, "t": float,
    /// "id": float}}}
    /// （数值一律 float，对齐 GJson 落盘约束；id = crops 数组下标）
    #[func]
    pub fn get_farm_state(&self) -> VarDictionary {
        let mut wet_d = VarDictionary::new();
        let mut crops = VarDictionary::new();
        for cy in 0..self.height.max(0) {
            for cx in 0..self.width.max(0) {
                let i = self.cell_index(cx, cy);
                let key = GString::from(format!("{},{}", cx, cy).as_str());
                if self.wet.get(i).copied().unwrap_or(false) {
                    let kv = key.to_variant();
                    wet_d.set(&kv, true);
                }
                let st = self.crop_stage.get(i).copied().unwrap_or(-1);
                if st >= 0 {
                    let mut cd = VarDictionary::new();
                    cd.set("stage", st as f32);
                    cd.set("t", self.crop_timer.get(i).copied().unwrap_or(0.0));
                    cd.set(
                        "id",
                        self.crop_kind.get(i).copied().unwrap_or(0) as f32,
                    );
                    let kv = key.to_variant();
                    let cv = cd.to_variant();
                    crops.set(&kv, &cv);
                }
            }
        }
        let mut d = VarDictionary::new();
        let wv = wet_d.to_variant();
        let cv = crops.to_variant();
        d.set("wet", &wv);
        d.set("crops", &cv);
        d
    }

    /// 农田状态 restore（进场恢复；先清运行期状态再整体应用。
    /// 越界/非法键忽略；成功后刷新湿润遮罩与作物精灵）
    #[func]
    pub fn set_farm_state(&mut self, state: VarDictionary) {
        self.clear_farm_runtime();
        let w = self.width.max(0);
        let h = self.height.max(0);
        if let Some(wet_d) = state.get("wet").and_then(|v| v.try_to::<VarDictionary>().ok()) {
            for (key, _val) in wet_d.iter_shared() {
                if let Ok(k) = key.try_to::<GString>() {
                    if let Some((cx, cy)) = Self::parse_cell_key(&k.to_string()) {
                        if cx >= 0 && cy >= 0 && cx < w && cy < h {
                            let i = self.cell_index(cx, cy);
                            self.wet[i] = true;
                        }
                    }
                }
            }
        }
        if let Some(crops) = state.get("crops").and_then(|v| v.try_to::<VarDictionary>().ok()) {
            for (key, val) in crops.iter_shared() {
                let Ok(k) = key.try_to::<GString>() else {
                    continue;
                };
                let Some((cx, cy)) = Self::parse_cell_key(&k.to_string()) else {
                    continue;
                };
                if cx < 0 || cy < 0 || cx >= w || cy >= h {
                    continue;
                }
                let Ok(cd) = val.try_to::<VarDictionary>() else {
                    continue;
                };
                // 作物种类：旧档无 id 字段 → 0（历史单作物档无损迁移）；
                // 越界（定义表已缩水）→ 0 兜底
                let kind = cd
                    .get("id")
                    .and_then(|v| v.try_to::<f64>().ok())
                    .map(|v| v as i32)
                    .unwrap_or(0)
                    .clamp(0, self.def_count() as i32 - 1);
                // 数值防御：GDScript 侧可能给 int/float，统一走 f64 再收窄；
                // 阶段夹到该作物自己的阶段域内
                let stage = cd
                    .get("stage")
                    .and_then(|v| v.try_to::<f64>().ok())
                    .map(|v| v as i32)
                    .unwrap_or(0)
                    .clamp(0, self.crop_stages_of(kind) - 1);
                let t = cd
                    .get("t")
                    .and_then(|v| v.try_to::<f64>().ok())
                    .unwrap_or(0.0) as f32;
                let i = self.cell_index(cx, cy);
                self.crop_stage[i] = stage;
                self.crop_kind[i] = kind;
                self.crop_timer[i] = t;
                self.ensure_crop_sprite(Vector2i::new(cx, cy), stage, kind);
            }
        }
        self.refresh_wet_mask();
        let active = self.farm_active();
        self.base_mut().set_process(active);
    }

    /// 光标按动作显示（"dig"=可锄 / "water"=可浇 / "plant"=可播 /
    /// "harvest"=成熟作物可采摘；其他 = dig）。不可操作的目标格自动隐藏
    /// （负反馈：不可锄/已湿润/无耕地/未成熟各有语义）
    #[func]
    pub fn set_cursor_for(&mut self, cell: Vector2i, action: GString) {
        let show = match action.to_string().as_str() {
            "dig" => self.is_diggable(cell),
            "water" => self.is_waterable(cell) && !self.is_wet(cell),
            "plant" => self.is_plantable(cell),
            "harvest" => self.is_mature(cell),
            // 未知动作（如 "mine"，本层无矿石语义）→ 不亮（负反馈）
            _ => false,
        };
        if let Some(cur) = self.cursor.as_mut() {
            if show {
                cur.bind_mut().show_at(cell, action);
            } else {
                cur.bind_mut().hide_cursor();
            }
        }
    }

    // ---- 内部实现 ----

    // ---- 农田层内部 ----

    /// 农层下标（越界 None；网格未初始化 None）
    fn farm_index(&self, cell: Vector2i) -> Option<usize> {
        let n = (self.width.max(0) * self.height.max(0)) as usize;
        if self.width <= 0 || self.height <= 0 || self.wet.len() != n {
            return None;
        }
        if cell.x < 0 || cell.y < 0 || cell.x >= self.width || cell.y >= self.height {
            return None;
        }
        Some(self.cell_index(cell.x, cell.y))
    }

    /// "x,y" 键 → (cx, cy)
    fn parse_cell_key(key: &str) -> Option<(i32, i32)> {
        let (x, y) = key.split_once(',')?;
        let cx = x.trim().parse::<i32>().ok()?;
        let cy = y.trim().parse::<i32>().ok()?;
        Some((cx, cy))
    }

    /// 农层数组对齐网格（ready/rebuild 调；已有内容按原值保留，扩缩部分归零）
    fn resize_farm(&mut self) {
        let n = (self.width.max(0) * self.height.max(0)) as usize;
        self.wet.resize(n, false);
        self.wet_timer.resize(n, 0.0);
        self.crop_stage.resize(n, -1);
        self.crop_kind.resize(n, -1);
        self.crop_timer.resize(n, 0.0);
    }

    /// 是否有活跃农田状态（湿润或存在作物；决定帧循环开关）
    fn farm_active(&self) -> bool {
        self.wet.iter().any(|&b| b) || self.crop_stage.iter().any(|&s| s >= 0)
    }

    /// 湿润遮罩装配（w×h 全透明起步；重复调用按需重建尺寸）
    fn setup_wet_mask(&mut self) {
        let w = self.width.max(1);
        let h = self.height.max(1);
        let rebuild = match &self.wet_img {
            Some(img) => img.get_width() != w || img.get_height() != h,
            None => true,
        };
        if rebuild {
            let img = Image::create_empty(w, h, false, godot::classes::image::Format::RGBA8)
                .expect("create wet mask image");
            let tex = ImageTexture::create_from_image(&img).expect("create wet mask texture");
            self.wet_img = Some(img);
            self.wet_tex = Some(tex);
        }
        self.refresh_wet_mask();
    }

    /// 湿润遮罩重填（内存 Image 权威 + GPU update；小地图全量重填最简）
    fn refresh_wet_mask(&mut self) {
        let (Some(img), Some(tex)) = (&mut self.wet_img, &mut self.wet_tex) else {
            return;
        };
        let w = self.width.max(0);
        let h = self.height.max(0);
        img.fill(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        for cy in 0..h {
            for cx in 0..w {
                // 直接算术取下标（self 已被 img/tex 可变借用，避 cell_index）
                let i = (cy * w + cx) as usize;
                if self.wet.get(i).copied().unwrap_or(false) {
                    // 单通道置位（多通道 bitmask 严禁 WHITE——误触发其他分支）
                    img.set_pixel(cx, cy, Color::from_rgba(1.0, 0.0, 0.0, 1.0));
                }
            }
        }
        tex.update(&*img);
    }

    /// 农田状态机 tick（process 驱动；仅在有湿润/作物时开帧循环）：
    /// 湿润超时干燥 → 生长暂停；作物仅湿润时累计生长 → 阶段推进 → 成熟。
    /// 生长节奏逐格读所属 def（多作物各自 stages/grow_secs 独立）
    fn tick_farm(&mut self, delta: f32) {
        let w = self.width.max(0);
        let h = self.height.max(0);
        if w == 0 || h == 0 || self.wet.len() != (w * h) as usize {
            return;
        }
        let wet_secs = self.wet_secs;
        let mut dried: Vec<Vector2i> = Vec::new();
        let mut grew: Vec<(Vector2i, i32, bool)> = Vec::new();
        for cy in 0..h {
            for cx in 0..w {
                let i = self.cell_index(cx, cy);
                if self.wet[i] && wet_secs > 0.0 {
                    self.wet_timer[i] += delta;
                    if self.wet_timer[i] as f64 >= wet_secs {
                        self.wet[i] = false;
                        self.wet_timer[i] = 0.0;
                        dried.push(Vector2i::new(cx, cy));
                    }
                }
                if self.crop_stage[i] < 0 || !self.wet[i] {
                    continue;
                }
                let kind = self.crop_kind[i];
                let grow_secs = self.crop_grow_secs_of(kind);
                let max_stage = self.crop_stages_of(kind) - 1;
                if self.crop_stage[i] < max_stage {
                    self.crop_timer[i] += delta;
                    if grow_secs > 0.0 && self.crop_timer[i] as f64 >= grow_secs {
                        self.crop_timer[i] = 0.0;
                        self.crop_stage[i] += 1;
                        let mature = self.crop_stage[i] >= max_stage;
                        grew.push((Vector2i::new(cx, cy), self.crop_stage[i], mature));
                    }
                }
            }
        }
        if !dried.is_empty() {
            self.refresh_wet_mask();
            for cell in &dried {
                self.base_mut().emit_signal("s_dried", &[cell.to_variant()]);
            }
        }
        for (cell, stage, mature) in &grew {
            let kind = self
                .crop_kind
                .get(self.cell_index(cell.x, cell.y))
                .copied()
                .unwrap_or(0);
            self.update_crop_sprite(*cell, *stage, kind);
            self.base_mut().emit_signal(
                "s_grew",
                &[cell.to_variant(), (*stage as i32).to_variant()],
            );
            if *mature {
                self.base_mut().emit_signal("s_matured", &[cell.to_variant()]);
            }
        }
        let active = self.farm_active();
        self.base_mut().set_process(active);
    }

    /// 清空运行期农田状态（移除作物精灵、湿润归零；持久化 restore 前置）
    fn clear_farm_runtime(&mut self) {
        let cells: Vec<(i32, i32)> = self.crop_sprites.keys().copied().collect();
        for key in cells {
            if let Some(mut spr) = self.crop_sprites.remove(&key) {
                spr.queue_free();
            }
        }
        for v in self.wet.iter_mut() {
            *v = false;
        }
        for v in self.wet_timer.iter_mut() {
            *v = 0.0;
        }
        for v in self.crop_stage.iter_mut() {
            *v = -1;
        }
        for v in self.crop_kind.iter_mut() {
            *v = -1;
        }
        for v in self.crop_timer.iter_mut() {
            *v = 0.0;
        }
    }

    /// 作物精灵装配（首次创建；z=1 盖瓦片，坐标经 to_local——根带偏移+缩放）。
    /// 贴图/阶段数/tint 均取所属 def（多作物各长各的）
    fn ensure_crop_sprite(&mut self, cell: Vector2i, stage: i32, kind: i32) {
        let key = (cell.x, cell.y);
        if !self.crop_sprites.contains_key(&key) {
            let tex = self.crop_sheet_tex(kind);
            let mut spr = Sprite2D::new_alloc();
            spr.set_name("CropSprite");
            spr.set_texture(&tex);
            spr.set_hframes(self.crop_stages_of(kind).max(1));
            // 贴图模式 tint = 变体色（占位图已按 tint 烘进像素，无需再调）
            if self.crops.get(kind.max(0) as usize).is_some() {
                spr.set_modulate(self.def_tint(kind));
            }
            // 帧宽 = 贴图高（横向序列帧）；局部缩放 = (cell_size / 根缩放) / 帧宽
            let frame_px = tex.get_height().max(1) as f32;
            let root_scale = self.base_mut().get_scale().x.max(0.01);
            let s = self.cell_size.max(1) as f32 / root_scale / frame_px;
            spr.set_scale(Vector2::new(s, s));
            spr.set_z_index(1);
            let world = self.cell_to_center(cell);
            let local = self.base_mut().to_local(world);
            spr.set_position(local);
            self.base_mut().add_child(&spr);
            self.crop_sprites.insert(key, spr);
        }
        let max_stage = self.crop_stages_of(kind) - 1;
        if let Some(spr) = self.crop_sprites.get_mut(&key) {
            spr.set_visible(true);
            spr.set_frame(stage.clamp(0, max_stage));
        }
    }

    /// kind 下 def 的 tint（越界/空定义 → 默认绿）
    fn def_tint(&self, kind: i32) -> Color {
        self.crops
            .get(kind.max(0) as usize)
            .map(|d| d.bind().tint)
            .unwrap_or_else(|| Color::from_rgba(0.35, 0.8, 0.35, 1.0))
    }

    /// crops 数组长度（空定义 = 0；restore 阶段夹取用）
    fn def_count(&self) -> usize {
        self.crops.len()
    }

    /// 作物阶段推进后刷新帧
    fn update_crop_sprite(&mut self, cell: Vector2i, stage: i32, kind: i32) {
        let key = (cell.x, cell.y);
        let max_stage = self.crop_stages_of(kind) - 1;
        if let Some(spr) = self.crop_sprites.get_mut(&key) {
            spr.set_frame(stage.clamp(0, max_stage));
        }
    }

    /// 移除作物精灵（收获/清场；queue_free 即可——键已弹出，同帧重名无碍）
    fn remove_crop_sprite(&mut self, cell: Vector2i) {
        let key = (cell.x, cell.y);
        if let Some(mut spr) = self.crop_sprites.remove(&key) {
            spr.queue_free();
        }
    }

    /// 作物序列帧纹理（按 kind 缓存）：def.texture 优先；未配置 → 程序化
    /// 占位（幼苗逐阶段长大，主色 = def.tint，零素材调试用）
    fn crop_sheet_tex(&mut self, kind: i32) -> Gd<Texture2D> {
        let k = kind.max(0) as usize;
        // 缓存向量按需扩容（def 数量就绪后 kind < len 恒成立）
        while self.crop_sheets.len() <= k {
            self.crop_sheets.push(None);
        }
        if let Some(tex) = self.crop_sheets[k].clone() {
            return tex;
        }
        let stages = self.crop_stages_of(kind);
        let tint = self.def_tint(kind);
        let tex = match self.crops.get(k) {
            Some(d) => match d.bind().texture.clone() {
                Some(t) => t,
                None => Self::build_placeholder_crop_sheet(stages, tint),
            },
            None => Self::build_placeholder_crop_sheet(stages, tint),
        };
        self.crop_sheets[k] = Some(tex.clone());
        tex
    }

    /// 程序化占位序列帧：每帧 16px，茎高随阶段增长 + 叶片展开 + 成熟结果；
    /// 主色由 tint 派生（茎 = tint×0.75、叶 = tint、果 = tint 提亮偏暖）
    fn build_placeholder_crop_sheet(stages: i32, tint: Color) -> Gd<Texture2D> {
        use godot::classes::image::Format;
        const FPX: i32 = 16;
        let mut img = Image::create_empty(FPX * stages, FPX, false, Format::RGBA8)
            .expect("create placeholder crop sheet");
        let green = Color::from_rgba(tint.r * 0.75, tint.g * 0.75, tint.b * 0.75, 1.0);
        let leaf = tint;
        let fruit = Color::from_rgba(
            (tint.r * 1.15).min(1.0),
            tint.g * 0.9,
            tint.b * 0.6,
            1.0,
        );
        let soil = Color::from_rgba(0.45, 0.33, 0.22, 0.9);
        for s in 0..stages {
            let t = (s + 1) as f32 / stages as f32;
            let ox = s * FPX;
            // 土壤点
            img.fill_rect(
                Rect2i::new(Vector2i::new(ox + 5, FPX - 4), Vector2i::new(6, 3)),
                soil,
            );
            // 茎：高随阶段
            let stem_h = (2.0 + t * 9.0).round() as i32;
            img.fill_rect(
                Rect2i::new(Vector2i::new(ox + 7, FPX - 5 - stem_h), Vector2i::new(2, stem_h)),
                green,
            );
            // 叶：后半阶段展开
            if t >= 0.4 {
                let lw = (3.0 + t * 4.0).round() as i32;
                img.fill_rect(
                    Rect2i::new(Vector2i::new(ox + 7 - lw, FPX - 8 - (stem_h / 2)), Vector2i::new(lw, 2)),
                    leaf,
                );
                img.fill_rect(
                    Rect2i::new(Vector2i::new(ox + 9, FPX - 8 - (stem_h / 2)), Vector2i::new(lw, 2)),
                    leaf,
                );
            }
            // 成熟结果
            if s == stages - 1 {
                img.fill_rect(
                    Rect2i::new(Vector2i::new(ox + 6, FPX - 12), Vector2i::new(4, 3)),
                    fruit,
                );
            }
        }
        let tex = godot::classes::ImageTexture::create_from_image(&img)
            .expect("create placeholder crop texture");
        tex.upcast::<Texture2D>()
    }

    /// 装配动作预览光标（GdActionCursor 组件，初始隐藏；纯本地子节点，
    /// 无全局副作用）。贴图可选——未配置时组件画动作色框，协议不变
    fn setup_cursor(&mut self) {
        // 光标是纯运行时交互辅助，编辑器不创建——tool 地图根的 ready 在
        // 编辑器也跑，非 tool 子类在编辑器实例化会触发 godot-rust
        // EditorPlaceholderV06 告警（v0.6 起返回占位实例）
        if Engine::singleton().is_editor_hint() {
            return;
        }
        let mut cur = super::gd_action_cursor::GdActionCursor::new_alloc();
        cur.set_name("ActionCursor");
        cur.bind_mut().set_cell_size(self.cell_size);
        if let Some(tex) = &self.cursor_texture {
            cur.bind_mut().set_texture(Some(tex.clone()));
        }
        // 相对 z=1 → 绝对 0：盖住瓦片（Display 子层 z=-1），树序在玩家前（让位角色）
        cur.set_z_index(1);
        self.base_mut().add_child(&cur);
        self.cursor = Some(cur);
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
        // 可浇灌地形：显式配置优先；空 = 自动取 dig_targets 的非空目标集合
        // （dig 的目标地形 = 已开垦耕地，天然是可浇灌面）
        if self.waterable_terrains.len() > 0 {
            self.waterable = self
                .waterable_terrains
                .to_vec()
                .iter()
                .filter_map(|s| self.name_index(s.to_string()))
                .collect();
        } else {
            self.waterable = self.dig_to.iter().copied().filter(|&t| t >= 0).collect();
        }
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
        let mut display = self.display_mut();
        display.set_tile_set(&ts);
        // 双网格显示：偏移半格 + 图集缩放，全部打在显示子层上（根恒单位变换）
        display.set_position(Vector2::new(-cs / 2.0, -cs / 2.0));
        display.set_scale(Vector2::new(cs / ts_px as f32, cs / ts_px as f32));
        drop(display);
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
        let mut display = self.display_mut();
        display.set_tile_set(&ts);
        display.set_position(Vector2::new(-cs / 2.0, -cs / 2.0));
        display.set_scale(Vector2::new(cs / TPX as f32, cs / TPX as f32));
        drop(display);
        self.repaint_all();
    }

    /// 重画全部显示格（0..=width x 0..=height）
    fn repaint_all(&mut self) {
        let Some(sid) = self.own_source_id() else { return };
        self.display_mut().clear();
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
        self.display_mut()
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
        let display = self.display.as_ref()?;
        let ts = display.get_tile_set()?;
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
        let w = self.width.max(1);
        let h = self.height.max(1);
        // 地形贴图遮罩（无贴图槽位时全零占位——wet 调暗仍需材质挂载）
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
        // 湿润遮罩（农田层；可能先于本函数存在——保留已有实例）
        if self.wet_img.is_none() || self.wet_tex.is_none() {
            self.setup_wet_mask();
        }
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
        // 湿润遮罩（农田层；refresh_wet_mask 局部更新 GPU 纹理）
        if let Some(wt) = &self.wet_tex {
            mat.set_shader_parameter("wet_mask", &wt.clone().upcast::<Texture2D>().to_variant());
        } else {
            mat.set_shader_parameter("wet_mask", &terrain_tex::transparent_tex().to_variant());
        }
        self.mask_img = Some(img);
        self.mask_tex = Some(tex);
        self.display_mut().set_material(&mat);
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
