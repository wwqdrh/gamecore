// TileMapDual - 双网格 TileMapLayer
// 移植自 GDScript TileMapDual/addons/TileMapDual/tile_map_dual.gd
// 世界网格存储逻辑地形，通过 Display 子节点显示过渡贴图
// 使用 ghost material 使世界网格不可见，只显示 DisplayLayer 的内容

use godot::prelude::*;
use godot::classes::{
    ITileMapLayer, Material, Shader, ShaderMaterial, TileMapLayer, TileSetAtlasSource,
};
use godot::builtin::{Array, Variant, Vector2i};
use godot::classes::object::ConnectFlags;
use godot::obj::NewGd;

use super::display::Display;
use super::tile_set_watcher::TileSetWatcher;

/// Ghost shader 代码：将所有像素的 alpha 设为 0，使世界网格不可见
const GHOST_SHADER_CODE: &str = r#"
shader_type canvas_item;
void fragment() {
    COLOR.a = 0.0;
}
"#;

#[derive(GodotClass)]
#[class(base = TileMapLayer, tool)]
pub struct TileMapDual {
    /// 刷新间隔时间（秒），主要用于编辑器模式
    #[var(pub)]
    refresh_time: f32,
    /// Godot 4.3 兼容模式：双检查所有格子
    #[var(pub)]
    godot_4_3_compatibility: bool,
    /// 显示层材质
    #[var(pub)]
    display_material: Option<Gd<Material>>,

    /// 不可见材质，用于隐藏世界网格
    ghost_material: Option<Gd<ShaderMaterial>>,
    /// TileSet 监视器
    tileset_watcher: Option<Gd<TileSetWatcher>>,
    /// 显示管理节点
    display: Option<Gd<Display>>,
    /// 缓存的 use_parent_material 值，用于检测变化
    cached_use_parent_material: Option<bool>,
    /// 刷新计时器
    timer: f32,
    base: Base<TileMapLayer>,
}

#[godot_api]
impl ITileMapLayer for TileMapDual {
    fn init(base: Base<TileMapLayer>) -> Self {
        // 创建 ghost material
        let mut shader = Shader::new_gd();
        shader.set_code(GHOST_SHADER_CODE);
        let mut ghost_mat = ShaderMaterial::new_gd();
        ghost_mat.set_shader(&shader);

        Self {
            refresh_time: 0.02,
            godot_4_3_compatibility: is_godot_below_4_4(),
            display_material: None,
            ghost_material: Some(ghost_mat),
            tileset_watcher: None,
            display: None,
            cached_use_parent_material: None,
            timer: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        let tile_set = self.base().get_tile_set();

        // 创建 TileSetWatcher
        let mut watcher = TileSetWatcher::new_watcher_public(tile_set);
        self.tileset_watcher = Some(watcher.clone());

        // 创建 Display 并添加为子节点
        let mut display = Gd::<Display>::from_init_fn(|base| {
            <Display as godot::classes::INode2D>::init(base)
        });
        let self_as_layer = self.base_mut().clone().upcast::<TileMapLayer>();
        display.bind_mut().setup_public(self_as_layer, watcher.clone());
        self.base_mut().add_child(&display);
        self.display = Some(display.clone());

        // 使世界网格不可见
        self.make_self_invisible(true);

        // 编辑器中使用 process 轮询，运行时使用信号
        if godot::classes::Engine::singleton().is_editor_hint() {
            // 编辑器：连接 atlas_autotiled 信号
            let callable = Callable::from_object_method(&*self.base_mut(), "_atlas_autotiled");
            let _ = watcher.connect_flags("atlas_autotiled", &callable, ConnectFlags::DEFERRED);
            self.base_mut().set_process(true);
        } else {
            // 运行时：连接 changed 信号
            let callable = Callable::from_object_method(&*self.base_mut(), "_changed");
            let _ = self.base_mut().connect_flags("changed", &callable, ConnectFlags::DEFERRED);
            self.base_mut().set_process(false);
        }

        // 延迟调用一次 _changed 以初始化
        // 原版使用 await get_tree().process_frame，这里用 call_deferred
        self.base_mut().call_deferred("_changed", &[]);
    }

    fn process(&mut self, delta: f64) {
        // 仅在编辑器中使用
        if self.refresh_time < 0.0 {
            return;
        }
        if self.timer > 0.0 {
            self.timer -= delta as f32;
            return;
        }
        self.timer = self.refresh_time;

        self.base_mut().call_deferred("_changed", &[]);
    }

    /// TileMapLayer 虚方法：当格子需要内部更新时由引擎调用
    /// 对应 GDScript 的 _update_cells(coords, forced_cleanup)
    /// 在编辑器中放置图块或 undo/redo 时触发，传入变化的格子坐标
    fn update_cells(&mut self, coords: Array<Vector2i>, _forced_cleanup: bool) {
        let Some(mut display) = self.display.clone() else { return };
        let variants: Array<Variant> = coords.iter_shared().map(|c| c.to_variant()).collect();
        display.bind_mut().update_public(variants);
    }
}

#[godot_api]
impl TileMapDual {
    /// 公开方法：添加或移除贴图
    /// terrain -1 完全移除贴图，默认 terrain 0 是空贴图
    #[func]
    fn draw_cell(&mut self, cell: Vector2i, #[opt(default = 1)] terrain: i32) {
        let Some(display) = self.display.clone() else { return };
        let terrain_dual = display.bind().get_terrain_public();

        let Some(terrain_dual) = terrain_dual else { return };
        let terrains = terrain_dual.bind().get_terrains_public();

        let terrain_key = terrain.to_variant();
        if !terrains.contains_key(&terrain_key) {
            self.base_mut().erase_cell(cell);
            self.base_mut().emit_signal("changed", &[]);
            return;
        }

        let mapping = terrains.get(&terrain_key).unwrap();
        let mapping_dict: godot::builtin::VarDictionary = mapping.to();
        let sid: i32 = mapping_dict.get(&"sid".to_variant()).unwrap().to();
        let tile: Vector2i = mapping_dict.get(&"tile".to_variant()).unwrap().to();

        self.base_mut()
            .set_cell_ex(cell)
            .source_id(sid)
            .atlas_coords(tile)
            .done();
        self.base_mut().emit_signal("changed", &[]);
    }

    /// 获取指定坐标的地形值
    #[func]
    fn get_cell(&self, cell: Vector2i) -> i32 {
        let Some(data) = self.base().get_cell_tile_data(cell) else {
            return -1;
        };
        data.get_terrain()
    }

    /// atlas_autotiled 信号回调（编辑器自动生成地形）
    #[func]
    fn _atlas_autotiled(&mut self, _source_id: i32, _atlas: Gd<TileSetAtlasSource>) {
        // 编辑器专属功能，按对齐方案在 GDScript 中实现
        // 这里仅作为信号回调占位
    }

    /// 使世界网格不可见
    /// 主贴图不需要被看到，只有 DisplayLayer 应该可见
    #[func]
    fn make_self_invisible(&mut self, #[opt(default = true)] startup: bool) {
        let Some(ghost_mat) = self.ghost_material.clone() else { return };
        let ghost_upcast = ghost_mat.upcast::<Material>();

        let current_material = self.base().get_material();
        if current_material.as_ref() != Some(&ghost_upcast) {
            if !startup && godot::classes::Engine::singleton().is_editor_hint() {
                godot_warn!(
                    "Warning! Direct material edit detected.\n\
                    Don't manually edit the real material in the editor!\n\
                    Instead edit the custom 'Display Material' property."
                );
            } else {
                // 复制材质到 display_material（如果被脚本编辑过）
                self.display_material = current_material;
            }
            // 强制 TileMapDual 的材质变为不可见
            self.base_mut().call("set_material", &[ghost_upcast.to_variant()]);
        }

        // 检查 use_parent_material 是否被设置
        let use_parent = self.base().get_use_parent_material();
        if godot::classes::Engine::singleton().is_editor_hint()
            && Some(use_parent) != self.cached_use_parent_material
            && self.cached_use_parent_material == Some(false)
        {
            godot_warn!(
                "Warning: Using Parent Material.\n\
                The parent material will override any other materials used by the TileMapDual,\n\
                including the 'ghost shader' that the world tiles use to hide themselves."
            );
        }
        self.cached_use_parent_material = Some(use_parent);
    }

    /// 当 tileset 变化或编辑器轮询时调用
    #[func]
    fn _changed(&mut self) {
        let Some(mut watcher) = self.tileset_watcher.clone() else { return };
        let tile_set = self.base().get_tile_set();
        watcher.bind_mut().update_public(tile_set);

        let mut updated_cells: Array<Vector2i> = Array::new();

        // Godot 4.3 兼容模式：双检查所有格子
        if self.godot_4_3_compatibility && self.base().get_tile_set().is_some() {
            let mut current_cells = Gd::<super::tile_cache::TileCache>::from_init_fn(|b| {
                <super::tile_cache::TileCache as godot::classes::IResource>::init(b)
            });
            let used_cells = self.base().get_used_cells();
            let self_layer = self.base_mut().clone();
            current_cells.bind_mut().update_edited_public(self_layer, used_cells);
            updated_cells = current_cells.bind().xor_public(self.get_cached_cells());
        }

        let Some(mut display) = self.display.clone() else { return };
        let updated_variants: Array<Variant> = updated_cells
            .iter_shared()
            .map(|c| c.to_variant())
            .collect();
        display.bind_mut().update_public(updated_variants);

        self.make_self_invisible(false);
    }
}

/// 内部方法
impl TileMapDual {
    /// 公开方法：添加或移除贴图（供 CursorDual 调用）
    pub fn draw_cell_public(&mut self, cell: Vector2i, terrain: i32) {
        self.draw_cell(cell, terrain);
    }

    /// 获取 Display 的 cached_cells（用于 Godot 4.3 兼容模式）
    fn get_cached_cells(&self) -> Gd<super::tile_cache::TileCache> {
        if let Some(display) = &self.display {
            display.bind().get_cached_cells_public()
        } else {
            Gd::<super::tile_cache::TileCache>::from_init_fn(|b| {
                <super::tile_cache::TileCache as godot::classes::IResource>::init(b)
            })
        }
    }
}

/// 检测 Godot 版本是否低于 4.4
fn is_godot_below_4_4() -> bool {
    let version = godot::classes::Engine::singleton().get_version_info();
    let major: i32 = version.get("major").map(|v| v.to()).unwrap_or(4);
    let minor: i32 = version.get("minor").map(|v| v.to()).unwrap_or(0);
    major < 4 || (major == 4 && minor < 4)
}
