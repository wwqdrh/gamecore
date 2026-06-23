// AtlasWatcher - 监视 TileSetAtlasSource 的变化
// 移植自 GDScript TileMapDual/addons/TileMapDual/atlas_watcher.gd
// 当 atlas 发生变化时，通知其父 TileSetWatcher 设置 terrains_changed 标志
// 还会检测自动生成贴图的操作，并发射 parent.atlas_autotiled 信号

use std::sync::LazyLock;

use godot::prelude::*;
use godot::classes::{IRefCounted, Image, TileSetAtlasSource};
use godot::classes::object::ConnectFlags;
use godot::builtin::{Rect2i, Vector2i};

use parking_lot::Mutex;

use super::tile_set_watcher::TileSetWatcher;

/// 防止已见过的 atlas 实例 ID 无限增长
const UNDO_LIMIT: usize = 1024;

/// 全局已注册的 atlas 实例 ID 列表，防止 redo 时误触发 autogen
static REGISTERED_ATLASES: LazyLock<Mutex<Vec<i64>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct AtlasWatcher {
    /// 创建此 AtlasWatcher 的 TileSetWatcher，用于回调信号
    parent: Option<Gd<TileSetWatcher>>,
    /// 此 watcher 监视的 atlas 的 source id
    sid: i32,
    /// 被监视的 atlas
    atlas: Option<Gd<TileSetAtlasSource>>,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for AtlasWatcher {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            parent: None,
            sid: -1,
            atlas: None,
            base,
        }
    }
}

#[godot_api]
impl AtlasWatcher {
    /// 当 atlas 发生变化时被调用，设置父级 terrains_changed 标志
    #[func]
    fn _atlas_changed(&self) {
        if let Some(mut parent) = self.parent.clone() {
            parent.bind_mut().flag_terrains_changed();
        }
    }

    /// 检测自动生成贴图的操作（一次性信号回调）
    /// HACK: 尝试猜测地形自动生成系统会创建哪些贴图
    #[func]
    fn _detect_autogen(&self) {
        let Some(atlas) = self.atlas.clone() else { return };
        let Some(mut texture) = atlas.get_texture() else { return };

        let texture_size: Vector2i = texture.call("get_size", &[]).to();
        let region_size = atlas.get_texture_region_size();
        let size = Vector2i::new(
            texture_size.x / region_size.x,
            texture_size.y / region_size.y,
        );

        let image_var = texture.call("get_image", &[]);
        let image: Option<Gd<Image>> = image_var.try_to().ok();
        let Some(image) = image else { return };

        for y in 0..size.y {
            for x in 0..size.x {
                let tile = Vector2i::new(x, y);
                let has_tile = atlas.has_tile(tile);
                let is_opaque = is_opaque_tile(&image, &atlas, tile, 0.1);
                if has_tile != is_opaque {
                    return;
                }
            }
        }

        // 所有贴图都匹配自动生成模式，发射 atlas_autotiled 信号
        if let Some(mut parent) = self.parent.clone() {
            if let Some(atlas) = self.atlas.clone() {
                parent.bind_mut().emit_atlas_autotiled(self.sid, atlas);
            }
        }
    }
}

// ===== 内部实现 =====
impl AtlasWatcher {
    /// 初始化 AtlasWatcher，连接 atlas.changed 信号
    /// 原版 _init(parent, sid, atlas) 在 gdext 中无法自定义构造函数参数，
    /// 改用 setup() 方法在创建后调用
    pub fn setup(&mut self, parent: Gd<TileSetWatcher>, sid: i32, atlas: Gd<TileSetAtlasSource>) {
        self.parent = Some(parent);
        self.sid = sid;
        self.atlas = Some(atlas.clone());

        // 连接 atlas.changed 信号（延迟调用）
        let callable = Callable::from_object_method(&*self.base_mut(), "_atlas_changed");
        let mut atlas_mut = atlas.clone();
        let _ = atlas_mut.connect_flags("changed", &callable, ConnectFlags::DEFERRED);

        let id = atlas.instance_id().to_i64();

        // 如果 atlas 为空且实例 ID 未注册过，则注册并连接 _detect_autogen 一次性信号
        if atlas_is_empty(&atlas) && !is_registered(id) {
            register_atlas_id(id);
            let autogen_callable = Callable::from_object_method(&*self.base_mut(), "_detect_autogen");
            let _ = atlas_mut.connect_flags(
                "changed",
                &autogen_callable,
                ConnectFlags::DEFERRED | ConnectFlags::ONE_SHOT,
            );
        }
    }
}

/// 检查 atlas 是否为空（没有贴图）
fn atlas_is_empty(atlas: &Gd<TileSetAtlasSource>) -> bool {
    atlas.get_tiles_count() == 0
}

/// 检查 atlas 实例 ID 是否已注册
fn is_registered(id: i64) -> bool {
    let registered = REGISTERED_ATLASES.lock();
    registered.contains(&id)
}

/// 注册 atlas 实例 ID，超出限制时移除最早的
fn register_atlas_id(id: i64) {
    let mut registered = REGISTERED_ATLASES.lock();
    registered.push(id);
    while registered.len() > UNDO_LIMIT {
        registered.remove(0);
    }
}

/// 检查贴图在指定坐标是否有不透明像素
fn is_opaque_tile(image: &Gd<Image>, atlas: &Gd<TileSetAtlasSource>, tile: Vector2i, threshold: f32) -> bool {
    let region_size = atlas.get_texture_region_size();
    let region = Rect2i::new(tile * region_size, region_size);

    // 先检查整个区域是否不可见
    let sprite = image.get_region(region);
    let Some(sprite) = sprite else { return false };
    if sprite.is_invisible() {
        return false;
    }

    // 逐像素检查是否有超过阈值的不透明像素
    for y in region.position.y..region.end().y {
        for x in region.position.x..region.end().x {
            let pixel = image.get_pixel(x, y);
            if pixel.a > threshold {
                return true;
            }
        }
    }
    false
}
