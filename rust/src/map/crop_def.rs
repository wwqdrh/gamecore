// GdCropDef — 作物定义资源（GdDigLayer 多作物支持）
//
// 一份资源 = 一种作物的完整生长定义（贴图/阶段/时长/种子/产物），编辑器
// 里可存 .tres 独立文件，也可在地图 .tscn 内嵌 sub_resource（demo 用内嵌，
// 见 home.tscn 的 crops 数组）。GdDigLayer.crops = Array[GdCropDef]，
// 播种时按 seed_id 匹配 def，格子按 def 各自的阶段数/时长/贴图独立生长。
//
// 贴图约定：横向序列帧，帧宽 = 贴图高，宽 = stages × 帧宽，左→右 =
// 阶段 0 → 成熟（如 4 阶段 = 128×32）。texture 空 = 程序化占位幼苗
// （tint 决定主色，零素材调试用；tint 同时也是贴图模式的 modulate，
// 同一贴图可配不同 tint 做变体色）。
use godot::classes::{Resource, Texture2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct GdCropDef {
    /// 作物 id（持久化与查询键，如 "veg_green"）
    #[export]
    pub id: GString,
    /// 显示名（如 "青菜"）
    #[export]
    pub name: GString,
    /// 对应种子道具 id（item.json；播种时按它匹配 def，如 "seeds_veg"）
    #[export]
    pub seed_id: GString,
    /// 收获产物道具 id（item.json；harvest 后玩家 add_item 入包）
    #[export]
    pub harvest_item: GString,
    /// 生长阶段数（= 序列帧帧数；成熟 = stages-1；1 = 种下即成熟）
    #[export]
    pub stages: i32,
    /// 每阶段生长秒数（仅湿润时累计；0 = 瞬间成熟）
    #[export]
    pub grow_secs: f64,
    /// 横向序列帧贴图（可选；空 = 程序化占位幼苗，主色 = tint）
    #[export]
    pub texture: Option<Gd<Texture2D>>,
    /// 占位图主色 / 贴图 modulate（同贴图配不同 tint = 变体色）
    #[export]
    pub tint: Color,
}

#[godot_api]
impl IResource for GdCropDef {
    fn init(_base: Base<Resource>) -> Self {
        Self {
            id: GString::new(),
            name: GString::new(),
            seed_id: GString::new(),
            harvest_item: GString::new(),
            stages: 4,
            grow_secs: 15.0,
            texture: None,
            tint: Color::from_rgba(0.35, 0.8, 0.35, 1.0),
        }
    }
}
