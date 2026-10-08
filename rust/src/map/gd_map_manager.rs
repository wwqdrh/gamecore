// GdMapManager - 地图管理器（继承 Node）
//
// 负责地图场景的注册、加载、切换与传送点调度，与 GdQuickMap/GdMapMarker 配套：
//   - 地图注册：map_paths 填 "别名=res://...tscn" 条目（或运行时 register_map），
//     地图场景根需为 Node2D（推荐直接用 GdQuickMap 作根）
//   - 地图切换：open_map(alias, spawn_cell) 释放旧地图、实例化新地图到 MapLayer，
//     发出 s_map_changed 信号；spawn_cell 记录玩家出生格（供后续人物系统取用）
//   - 传送调度：try_teleport(world_pos) 命中当前地图某 GdMapMarker 的格子时，
//     发出 s_teleport_triggered 并切换到 marker.target_alias/target_cell；
//     后续人物网格走动每步调用 try_teleport(角色世界坐标) 即可自动传送
//   - 初始加载：initial_map/spawn_cell 场景上显式配置，ready 自动 open_map
//
// 调试便利：click_teleport = true 时，未被 UI 消费的左键点击若落在传送点格内，
// 直接触发传送（人物系统接入前可点击验证地图流转）。
//
// 场景装配参考：example/demo/xiuxian/scenes/main/index.tscn

use std::collections::HashMap;

use godot::prelude::*;
use godot::builtin::{GString, PackedStringArray, Variant, VarArray, Vector2, Vector2i, StringName};
use godot::classes::{Engine, INode, InputEvent, InputEventMouseButton, Node, Node2D, PackedScene, ResourceLoader};
use godot::global::MouseButton;

use super::gd_map_marker::MAP_MARKER_GROUP;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdMapManager {
    /// 地图注册表（alias -> 场景路径），ready 时从 map_paths 解析
    registry: HashMap<String, String>,

    /// 地图场景容器（ready 时自动创建）
    map_layer: Option<Gd<Node2D>>,

    /// 当前地图实例
    current_map: Option<Gd<Node2D>>,
    /// 当前地图别名
    current_alias: GString,
    /// 当前出生格（open_map 时记录）
    spawn_cell: Vector2i,

    // ---- 可配置导出属性 ----
    /// 地图注册列表，每项格式 "别名=res://...tscn"
    #[export]
    map_paths: PackedStringArray,
    /// 初始加载的地图别名（ready 自动 open_map；空则不自动加载）
    #[export]
    initial_map: GString,
    /// 初始出生格
    #[export]
    initial_spawn: Vector2i,
    /// 是否自动加载 initial_map（关闭后仅注册，由业务手动 open_map）
    #[export]
    auto_load: bool,
    /// 点击传送点格子直接传送（调试便利，人物系统接入前验证地图流转）
    #[export]
    click_teleport: bool,
    /// 管理器 ID（注册到 GDCore 全局节点表）
    #[var]
    manager_id: GString,

    /// 场景组件表中的注册名（GdSceneRoot.get_component 按此查询）
    #[var]
    component_name: GString,

    /// 组件注册重试帧数（GdScene 默认管理器延迟创建，需逐帧重试）
    component_reg_frames: u32,

    base: Base<Node>,
}

/// 组件注册重试上限（约 5 秒 @60fps，超过放弃并告警一次）
const COMPONENT_REG_MAX_FRAMES: u32 = 300;

#[godot_api]
impl INode for GdMapManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            registry: HashMap::new(),
            map_layer: None,
            current_map: None,
            current_alias: GString::new(),
            spawn_cell: Vector2i::ZERO,
            map_paths: PackedStringArray::new(),
            initial_map: GString::new(),
            initial_spawn: Vector2i::ZERO,
            auto_load: true,
            click_teleport: true,
            manager_id: GString::from("map"),
            component_name: GString::from("MapManager"),
            component_reg_frames: 0,
            base,
        }
    }

    fn ready(&mut self) {
        // 解析 "alias=path" 注册表
        self.parse_map_paths();

        // 创建地图容器
        let mut layer = Node2D::new_alloc();
        layer.set_name(&StringName::from("MapLayer"));
        self.base_mut().add_child(&layer);
        self.map_layer = Some(layer);

        // 注册到 GDCore 全局节点表（与 GdSceneRoot 同机制）
        self.register_to_gdcore();

        // 初始加载（仅认场景上显式配置的 initial_map，空则不自动加载）
        if self.auto_load && !self.initial_map.is_empty() {
            let alias = self.initial_map.clone();
            let spawn = self.initial_spawn;
            if !self.open_map(alias, spawn) {
                godot_warn!("GdMapManager: initial map load failed: {}", self.initial_map);
            }
        }
    }

    fn process(&mut self, _delta: f64) {
        // 向所在场景根的组件表注册自身（GdSceneRoot.get_component("MapManager")）。
        // 场景根可能是 GdScene 延迟创建的默认 GdSceneRoot（call_deferred），
        // 因此逐帧重试直到成功或超限。
        if self.component_reg_frames >= COMPONENT_REG_MAX_FRAMES {
            self.base_mut().set_process(false);
            return;
        }
        self.component_reg_frames += 1;

        let Some(gdcore) = Engine::singleton().get_singleton("GDCORE") else {
            return;
        };
        let mut gdcore = gdcore;
        if !gdcore.has_method(&StringName::from("get_global_node")) {
            return;
        }
        // GdSceneRoot 以 manager_id（默认 "default"）注册在 GDCORE 全局节点表
        let root_var = gdcore.call(
            &StringName::from("get_global_node"),
            &[GString::from("default").to_variant()],
        );
        if root_var.get_type() != godot::builtin::VariantType::OBJECT {
            return;
        }
        let Ok(mut scene_root) = root_var.try_to::<Gd<Node>>() else {
            return;
        };
        if !scene_root.has_method("register_component") {
            return;
        }
        let base = self.base().clone();
        scene_root.call(
            &StringName::from("register_component"),
            &[self.component_name.to_variant(), base.to_variant()],
        );
        // 注册成功，不再需要逐帧重试
        self.base_mut().set_process(false);
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if !self.click_teleport {
            return;
        }
        let Ok(mb) = event.try_cast::<InputEventMouseButton>() else {
            return;
        };
        if mb.get_button_index() != MouseButton::LEFT || !mb.is_pressed() {
            return;
        }
        // 视口坐标 → 世界坐标（考虑相机变换）
        let Some(vp) = self.base().get_viewport() else {
            return;
        };
        let mouse = vp.get_mouse_position();
        let world = vp.get_canvas_transform().affine_inverse() * mouse;
        self.try_teleport(world);
    }
}

#[godot_api]
impl GdMapManager {
    /// 地图切换完成信号（参数：新地图别名）
    #[signal]
    fn s_map_changed(alias: GString);

    /// 传送触发信号（参数：来源别名、目标别名、落点格子）
    #[signal]
    fn s_teleport_triggered(from_alias: GString, to_alias: GString, target_cell: Vector2i);

    /// 注册地图：alias 别名，path 场景路径（res://...tscn）
    #[func]
    pub fn register_map(&mut self, alias: GString, path: GString) {
        let key = alias.to_string();
        if self.registry.contains_key(&key) {
            godot_warn!("GdMapManager: map alias '{}' already registered", alias);
            return;
        }
        self.registry.insert(key, path.to_string());
    }

    /// 打开（切换到）指定地图
    /// alias: 地图别名；spawn: 出生格（记录到 spawn_cell 供人物系统取用）
    #[func]
    pub fn open_map(&mut self, alias: GString, spawn: Vector2i) -> bool {
        let mut spawn = spawn;
        let new_map = match self.instantiate_map(&alias) {
            Some(map) => map,
            None => return false,
        };

        // 释放旧地图
        if let Some(old) = self.current_map.take() {
            let mut old = old;
            if let Some(mut parent) = old.get_parent() {
                parent.remove_child(&old);
            }
            old.queue_free();
        }

        // 挂载新地图
        if let Some(ref layer) = self.map_layer {
            let mut layer = layer.clone();
            layer.add_child(&new_map);
        }

        // 传送点吸附：落在不可通行地形上的标记点自动移到最近的可行走格
        self.snap_markers_to_walkable(&new_map);

        // 出生格吸附（与传送点同规则）：落在不可通行地形时移到最近可行走格
        if new_map.has_method("is_walkable") {
            let mut m = new_map.clone();
            if !m.call("is_walkable", &[spawn.to_variant()]).to::<bool>() {
                if let Some(snapped) = Self::nearest_walkable(&mut m, spawn) {
                    godot_warn!(
                        "GdMapManager: spawn cell ({}, {}) not walkable, snapped to ({}, {})",
                        spawn.x, spawn.y, snapped.x, snapped.y
                    );
                    spawn = snapped;
                }
            }
        }

        self.current_map = Some(new_map);
        self.current_alias = alias.clone();
        self.spawn_cell = spawn;

        self.base_mut().emit_signal(
            "s_map_changed",
            &[alias.to_variant()],
        );
        true
    }

    /// 世界坐标传送：命中当前地图某传送点格子时切换到其目标地图。
    /// 返回是否触发传送（人物网格走动每步调用即可）。
    #[func]
    pub fn try_teleport(&mut self, world_pos: Vector2) -> bool {
        let Some(marker) = self.find_marker_at(world_pos) else {
            return false;
        };
        let mut marker = marker;

        marker.emit_signal("s_triggered", &[]);

        let target_alias = marker
            .call("get_target_alias", &[])
            .to::<GString>();
        if target_alias.is_empty() {
            return false; // 仅标记点，不传送
        }
        let target_cell = marker
            .call("get_target_cell", &[])
            .to::<Vector2i>();
        let from = self.current_alias.clone();

        self.base_mut().emit_signal(
            "s_teleport_triggered",
            &[from.to_variant(), target_alias.to_variant(), target_cell.to_variant()],
        );

        self.open_map(target_alias, target_cell)
    }

    /// 当前地图实例（未加载返回 null）
    #[func]
    pub fn get_current_map(&self) -> Variant {
        match &self.current_map {
            Some(map) => map.to_variant(),
            None => Variant::nil(),
        }
    }

    /// 当前地图别名（未加载为空串）
    #[func]
    pub fn get_current_alias(&self) -> GString {
        self.current_alias.clone()
    }

    /// 当前出生格
    #[func]
    pub fn get_spawn_cell(&self) -> Vector2i {
        self.spawn_cell
    }

    /// 出生格的世界坐标（格心）
    #[func]
    pub fn get_spawn_point(&self) -> Vector2 {
        let cs = self.current_cell_size();
        Vector2::new(
            (self.spawn_cell.x as f32 + 0.5) * cs,
            (self.spawn_cell.y as f32 + 0.5) * cs,
        )
    }

    /// 当前地图格子尺寸（无地图返回 0）
    #[func]
    pub fn get_current_cell_size(&self) -> i32 {
        self.current_cell_size() as i32
    }

    /// 别名是否已注册
    #[func]
    pub fn is_registered(&self, alias: GString) -> bool {
        self.registry.contains_key(&alias.to_string())
    }

    /// 已注册别名列表
    #[func]
    pub fn get_registered_maps(&self) -> VarArray {
        let mut out = VarArray::new();
        let mut names: Vec<&String> = self.registry.keys().collect();
        names.sort();
        for name in names {
            out.push(&GString::from(name.as_str()).to_variant());
        }
        out
    }

    /// 当前地图上的传送点列表（GdMapMarker 数组）
    #[func]
    pub fn get_markers(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(ref map) = self.current_map else {
            return out;
        };
        if let Some(tree) = self.base().get_tree_or_null() {
            let nodes = tree.get_nodes_in_group(MAP_MARKER_GROUP);
            for i in 0..nodes.len() {
                let node = nodes.at(i);
                let Ok(n2d) = node.try_cast::<Node2D>() else {
                    continue;
                };
                if map.is_ancestor_of(&n2d.clone().upcast::<Node>()) {
                    out.push(&n2d.to_variant());
                }
            }
        }
        out
    }

    // ---- 内部实现 ----

    /// 解析 map_paths 中 "alias=path" 条目
    fn parse_map_paths(&mut self) {
        for i in 0..self.map_paths.len() {
            let entry = self.map_paths.get(i).map(|s| s.to_string()).unwrap_or_default();
            if entry.is_empty() {
                continue;
            }
            let Some((alias, path)) = entry.split_once('=') else {
                godot_warn!("GdMapManager: invalid map_paths entry '{}'", entry);
                continue;
            };
            self.registry
                .insert(alias.trim().to_string(), path.trim().to_string());
        }
    }

    /// 实例化地图：查注册表 → 加载 PackedScene → 校验根为 Node2D
    fn instantiate_map(&self, alias: &GString) -> Option<Gd<Node2D>> {
        let Some(path) = self.registry.get(&alias.to_string()) else {
            godot_warn!("GdMapManager: map '{}' not registered", alias);
            return None;
        };
        let packed = ResourceLoader::singleton()
            .load_ex(path.as_str())
            .done()?;
        let Ok(packed) = packed.try_cast::<PackedScene>() else {
            godot_warn!("GdMapManager: '{}' is not a PackedScene: {}", alias, path);
            return None;
        };
        let inst = packed.instantiate()?;
        match inst.try_cast::<Node2D>() {
            Ok(map) => {
                let mut map = map;
                map.set_name(&StringName::from(alias.to_string().as_str()));
                Some(map)
            }
            Err(_) => {
                godot_warn!(
                    "GdMapManager: map '{}' root is not a Node2D: {}",
                    alias,
                    path
                );
                None
            }
        }
    }

    /// 查找覆盖世界坐标的传送点
    fn find_marker_at(&self, world_pos: Vector2) -> Option<Gd<Node2D>> {
        let Some(ref map) = self.current_map else {
            return None;
        };
        let tree = self.base().get_tree_or_null()?;
        let nodes = tree.get_nodes_in_group(MAP_MARKER_GROUP);
        for i in 0..nodes.len() {
            let node = nodes.at(i);
            let Ok(n2d) = node.try_cast::<Node2D>() else {
                continue;
            };
            if !map.is_ancestor_of(&n2d.clone().upcast::<Node>()) {
                continue;
            }
            if self.marker_contains(&n2d, world_pos) {
                return Some(n2d);
            }
        }
        None
    }

    /// 传送点是否覆盖世界坐标（读导出字段，不依赖脚本方法）
    fn marker_contains(&self, marker: &Gd<Node2D>, world_pos: Vector2) -> bool {
        // GdMapMarker 的 contains_world_point
        if marker.has_method("contains_world_point") {
            let mut m = marker.clone();
            let r = m.call("contains_world_point", &[world_pos.to_variant()]);
            return r.to::<bool>();
        }
        false
    }

    /// 传送点吸附：标记点所在格不可通行（如山/水）时，BFS 找最近可行走格
    /// 并调用标记点 place_at 移过去（噪声地图种子固定但地形随种子变化，
    /// 场景里手填的格子无法保证永远落在地面）
    fn snap_markers_to_walkable(&self, map: &Gd<Node2D>) {
        if !map.has_method("is_walkable") {
            return;
        }
        let Some(tree) = self.base().get_tree_or_null() else {
            return;
        };
        let nodes = tree.get_nodes_in_group(MAP_MARKER_GROUP);
        for i in 0..nodes.len() {
            let node = nodes.at(i);
            let Ok(mut marker) = node.try_cast::<Node2D>() else {
                continue;
            };
            if !map.is_ancestor_of(&marker.clone().upcast::<Node>()) {
                continue;
            }
            if !marker.has_method("get_cell") || !marker.has_method("place_at") {
                continue;
            }
            let cell = marker.call("get_cell", &[]).to::<Vector2i>();
            let mut m = map.clone();
            if m.call("is_walkable", &[cell.to_variant()]).to::<bool>() {
                continue;
            }
            if let Some(snapped) = Self::nearest_walkable(&mut m, cell) {
                godot_warn!(
                    "GdMapManager: marker cell ({}, {}) not walkable, snapped to ({}, {})",
                    cell.x, cell.y, snapped.x, snapped.y
                );
                marker.call("place_at", &[snapped.to_variant()]);
            }
        }
    }

    /// BFS 距 from 最近的可行走格（含 from 本身；全图不可走返回 None）
    fn nearest_walkable(map: &mut Gd<Node2D>, from: Vector2i) -> Option<Vector2i> {
        let w = map.call("get_map_width", &[]).to::<i64>() as i32;
        let h = map.call("get_map_height", &[]).to::<i64>() as i32;
        if w <= 0 || h <= 0 || from.x < 0 || from.y < 0 || from.x >= w || from.y >= h {
            return None;
        }
        let idx = |c: Vector2i| (c.y * w + c.x) as usize;
        let mut visited = vec![false; (w * h) as usize];
        let mut queue = std::collections::VecDeque::new();
        visited[idx(from)] = true;
        queue.push_back(from);
        while let Some(cur) = queue.pop_front() {
            if map.call("is_walkable", &[cur.to_variant()]).to::<bool>() {
                return Some(cur);
            }
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let nc = Vector2i::new(cur.x + dx, cur.y + dy);
                if nc.x < 0 || nc.y < 0 || nc.x >= w || nc.y >= h {
                    continue;
                }
                let ni = idx(nc);
                if visited[ni] {
                    continue;
                }
                visited[ni] = true;
                queue.push_back(nc);
            }
        }
        None
    }

    /// 当前地图格子尺寸（GdQuickMap 提供 get_cell_size_px）
    fn current_cell_size(&self) -> f32 {
        let Some(ref map) = self.current_map else {
            return 0.0;
        };
        if map.has_method("get_cell_size_px") {
            let mut m = map.clone();
            let r = m.call("get_cell_size_px", &[]);
            if let Ok(v) = r.try_to::<i64>() {
                return v as f32;
            }
        }
        0.0
    }

    /// 注册到 GDCore 全局节点表
    fn register_to_gdcore(&self) {
        let mut engine = Engine::singleton();
        if let Some(gdcore) = engine.get_singleton("GDCORE") {
            let method = StringName::from("add_global_node");
            let mut gdcore = gdcore;
            if gdcore.has_method(&method) {
                let base = self.base().clone();
                gdcore.call(&method, &[self.manager_id.to_variant(), base.to_variant()]);
            }
        }
    }
}
