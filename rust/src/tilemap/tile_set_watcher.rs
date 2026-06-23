// TileSetWatcher - 监视 TileSet 的变化并发送信号
// 移植自 GDScript TileMapDual/addons/TileMapDual/tile_set_watcher.gd
// 检测 TileSet 的添加/替换/删除、tile_size 变化、grid_shape 变化、atlas 添加/自动生成、地形变化
// 使用标志位系统在同一帧内累积变化，下一帧 check_flags() 时统一发射信号
// 内部使用 Rust HashMap 存储 _cached_sids，比原版 Dictionary 更高效

use std::collections::HashMap;

use godot::prelude::*;
use godot::classes::{
    IResource, Resource, TileSet, TileSetAtlasSource,
};
use godot::classes::object::ConnectFlags;
use godot::builtin::Vector2i;

use super::atlas_watcher::AtlasWatcher;
use super::grid_shape;

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct TileSetWatcher {
    /// 缓存上一次的 tile_set，用于检测变化
    #[var(pub)]
    tile_set: Option<Gd<TileSet>>,
    /// 缓存上一次的 tile_size，用于检测变化
    #[var(pub)]
    tile_size: Vector2i,
    /// 缓存上一次的 grid_shape（存储为 GridShape 序数的 i32）
    #[var(pub)]
    grid_shape: i32,

    /// 内部标志位
    flag_tileset_deleted: bool,
    flag_tileset_created: bool,
    flag_tileset_resized: bool,
    flag_tileset_reshaped: bool,
    flag_atlas_added: bool,
    flag_terrains_changed: bool,
    flag_tileset_changed: bool,

    /// 缓存上一次的 source 数量
    cached_source_count: i32,
    /// 缓存的 sid -> AtlasWatcher 映射
    cached_sids: HashMap<i32, Gd<AtlasWatcher>>,

    base: Base<Resource>,
}

#[godot_api]
impl IResource for TileSetWatcher {
    fn init(base: Base<Resource>) -> Self {
        Self {
            tile_set: None,
            tile_size: Vector2i::ZERO,
            grid_shape: grid_shape::GridShape::Square.ord(),
            flag_tileset_deleted: false,
            flag_tileset_created: false,
            flag_tileset_resized: false,
            flag_tileset_reshaped: false,
            flag_atlas_added: false,
            flag_terrains_changed: false,
            flag_tileset_changed: false,
            cached_source_count: 0,
            cached_sids: HashMap::new(),
            base,
        }
    }
}

#[godot_api]
impl TileSetWatcher {
    // ===== 信号定义 =====

    /// tile_set 被清除或替换时发射
    #[signal]
    fn tileset_deleted();

    /// tile_set 被创建或替换时发射
    #[signal]
    fn tileset_created();

    /// tile_set.tile_size 变化时发射
    #[signal]
    fn tileset_resized();

    /// TileSet 的 GridShape 变化时发射
    #[signal]
    fn tileset_reshaped();

    /// 新的 Atlas 被添加到 TileSet 时发射
    #[signal]
    fn atlas_added(source_id: i32, atlas: Gd<TileSetAtlasSource>);

    /// 检测到自动生成贴图操作时发射
    #[signal]
    fn atlas_autotiled(source_id: i32, atlas: Gd<TileSetAtlasSource>);

    /// atlas 添加/删除或地形变化时发射
    #[signal]
    fn terrains_changed();

    // ===== 公开方法 =====

    /// 创建 TileSetWatcher 并初始化
    /// 原版 _init(tile_set) 在 gdext 中无法自定义构造函数参数，改用 new_watcher() 静态方法
    #[func]
    fn new_watcher(tile_set: Option<Gd<TileSet>>) -> Gd<Self> {
        let mut gd = Gd::<Self>::from_init_fn(|base| TileSetWatcher::init(base));
        // 连接 atlas_added 信号到 _atlas_added 回调（延迟调用）
        let callable = Callable::from_object_method(&gd, "_atlas_added");
        let _ = gd.connect_flags("atlas_added", &callable, ConnectFlags::DEFERRED);
        gd.bind_mut().update(tile_set);
        gd
    }

    /// 检查 TileMapDual 的 tile_set 是否发生变化
    /// 必须每帧由 TileMapDual 调用
    #[func]
    fn update(&mut self, tile_set: Option<Gd<TileSet>>) {
        self.check_tile_set(tile_set);
        self.check_flags();
    }

    /// 如果对应标志位被设置，则发射更新信号
    /// 每帧只能运行一次
    #[func]
    fn check_flags(&mut self) {
        if self.flag_tileset_changed {
            self.flag_tileset_changed = false;
            self.update_tileset();
        }
        if self.flag_tileset_deleted {
            self.flag_tileset_deleted = false;
            self.flag_tileset_reshaped = true;
            self.base_mut().emit_signal("tileset_deleted", &[]);
        }
        if self.flag_tileset_created {
            self.flag_tileset_created = false;
            self.flag_tileset_reshaped = true;
            self.base_mut().emit_signal("tileset_created", &[]);
        }
        if self.flag_tileset_resized {
            self.flag_tileset_resized = false;
            self.base_mut().emit_signal("tileset_resized", &[]);
        }
        if self.flag_tileset_reshaped {
            self.flag_tileset_reshaped = false;
            self.flag_terrains_changed = true;
            self.base_mut().emit_signal("tileset_reshaped", &[]);
        }
        if self.flag_atlas_added {
            self.flag_atlas_added = false;
            self.flag_terrains_changed = true;
        }
        if self.flag_terrains_changed {
            self.flag_terrains_changed = false;
            self.base_mut().emit_signal("terrains_changed", &[]);
        }
    }

    /// 检查 tile_set 是否被添加、替换或删除
    #[func]
    fn check_tile_set(&mut self, tile_set: Option<Gd<TileSet>>) {
        // 比较新传入的 tile_set 和缓存的 tile_set 是否相同
        let same = match (&self.tile_set, &tile_set) {
            (None, None) => true,
            (Some(a), Some(b)) => a.instance_id() == b.instance_id(),
            _ => false,
        };
        if same {
            return;
        }

        // 断开旧 tile_set 的 changed 信号连接
        let old_tile_set = self.tile_set.clone();
        if let Some(mut old_tile_set) = old_tile_set {
            let callable = Callable::from_object_method(&*self.base_mut(), "_set_tileset_changed");
            old_tile_set.disconnect("changed", &callable);
            self.cached_source_count = 0;
            self.cached_sids.clear();
            self.flag_tileset_deleted = true;
        }

        self.tile_set = tile_set.clone();

        if let Some(mut new_tile_set) = tile_set {
            let callable = Callable::from_object_method(&*self.base_mut(), "_set_tileset_changed");
            let _ = new_tile_set.connect_flags("changed", &callable, ConnectFlags::DEFERRED);
            new_tile_set.emit_changed();
            self.flag_tileset_created = true;
            self.tile_set = Some(new_tile_set);
        }

        self.base_mut().emit_signal("changed", &[]);
    }

    // ===== 信号回调方法 =====

    /// atlas_added 信号回调，设置 atlas_added 标志
    #[func]
    fn _atlas_added(&mut self, _source_id: i32, _atlas: Gd<TileSetAtlasSource>) {
        self.flag_atlas_added = true;
    }

    /// tile_set.changed 信号回调，设置 tileset_changed 标志
    #[func]
    fn _set_tileset_changed(&mut self) {
        self.flag_tileset_changed = true;
    }
}

// ===== 内部实现 =====
impl TileSetWatcher {
    /// 当 flag_tileset_changed 时调用，提供更详细的变化检测
    fn update_tileset(&mut self) {
        let tile_set = self.tile_set.clone();
        let Some(tile_set) = tile_set else { return };

        let tile_size = tile_set.get_tile_size();
        if self.tile_size != tile_size {
            self.tile_size = tile_size;
            self.flag_tileset_resized = true;
        }

        let grid_shape = grid_shape::tileset_gridshape(&tile_set);
        let grid_shape_ord = grid_shape.ord();
        if self.grid_shape != grid_shape_ord {
            self.grid_shape = grid_shape_ord;
            self.flag_tileset_reshaped = true;
        }

        self.update_tileset_atlases();
    }

    /// 检查是否有新的 atlas 被添加
    /// 不检查哪些 atlas 被删除
    fn update_tileset_atlases(&mut self) {
        let tile_set = self.tile_set.clone();
        let Some(tile_set) = tile_set else { return };
        let source_count = tile_set.get_source_count();

        // 仅在 atlas 添加或删除时处理
        if self.cached_source_count == source_count {
            return;
        }
        self.cached_source_count = source_count;

        let mut new_sids: HashMap<i32, Gd<AtlasWatcher>> = HashMap::new();

        for i in 0..source_count {
            let sid = tile_set.get_source_id(i);
            if self.cached_sids.contains_key(&sid) {
                let watcher = self.cached_sids.get(&sid).cloned();
                if let Some(w) = watcher {
                    new_sids.insert(sid, w);
                }
                continue;
            }

            let source = tile_set.get_source(sid);
            let Some(source) = source else { continue };

            // 检查是否为 TileSetAtlasSource
            if !source.is_class("TileSetAtlasSource") {
                godot_warn!(
                    "Non-Atlas TileSet found at index {}, source id {}.\nDual Grids only support Atlas TileSets.",
                    i, sid
                );
                continue;
            }

            let atlas = source.cast::<TileSetAtlasSource>();

            // 创建 AtlasWatcher 并初始化
            let mut watcher = Gd::<AtlasWatcher>::from_init_fn(|base| AtlasWatcher::init(base));
            let self_gd = self.base_mut().clone();
            watcher.bind_mut().setup(
                self_gd.cast::<Self>(),
                sid,
                atlas.clone(),
            );

            new_sids.insert(sid, watcher);

            // 发射 atlas_added 信号
            self.base_mut().emit_signal(
                "atlas_added",
                &[sid.to_variant(), atlas.to_variant()],
            );
        }

        self.flag_terrains_changed = true;
        self.cached_sids = new_sids;
    }

    /// 设置 terrains_changed 标志（供 AtlasWatcher 调用）
    pub fn flag_terrains_changed(&mut self) {
        self.flag_terrains_changed = true;
    }

    /// 发射 atlas_autotiled 信号（供 AtlasWatcher 调用）
    pub fn emit_atlas_autotiled(&mut self, sid: i32, atlas: Gd<TileSetAtlasSource>) {
        self.base_mut().emit_signal(
            "atlas_autotiled",
            &[sid.to_variant(), atlas.to_variant()],
        );
    }

    /// 创建 TileSetWatcher（公开包装方法，供 TileMapDual 调用）
    pub fn new_watcher_public(tile_set: Option<Gd<TileSet>>) -> Gd<Self> {
        Self::new_watcher(tile_set)
    }

    /// 更新 TileSet 检查（公开包装方法，供 TileMapDual 调用）
    pub fn update_public(&mut self, tile_set: Option<Gd<TileSet>>) {
        self.update(tile_set);
    }
}
