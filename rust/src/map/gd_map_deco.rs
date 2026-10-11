// GdDecoLayer - 地图装饰层（继承 Node2D）
//
// 挂在 GdQuickMap 地图场景下，按配置在指定地形上随机分布装饰/可采集物品：
//   - 配置为并行数组（下标一一对应，以 deco_item_ids 为主长度，风格同 GdQuickMap
//     地形并行数组），文本 .tscn 手写配置零门槛：
//       deco_terrains     目标地形名（逗号分隔多个，如 "grass,forest"；空 = 所有可行走地形）
//       deco_item_ids     物品 id（采集回调 / 背包联动的唯一标识）
//       deco_textures     物品贴图路径（res://...；留空用 deco_colors 色块绘制）
//       deco_colors       回退颜色（无贴图时绘制菱形色块）
//       deco_frequencies  单格生成概率 0..1（种子驱动，同地图+同种子结果确定）
//       deco_mineable     1 = 可开采（挂受击盒；被打满 deco_hits 次触发采集）
//       deco_hits         采集所需碰撞次数（默认 3；仅可开采条目生效）
//       deco_callbacks    采集回调函数名（在 callback_target_path 节点上定义，
//                         签名 func on_xxx(item_id: String, node: Node)，gd 侧实现入库等逻辑）
//
// 可开采协议（复用框架战斗组件，零新语义）：
//   矿石节点 = Node2D 根 + GdHealth（max_health = deco_hits，无敌帧 0）
//              + GdHurtbox（collision_layer = harvest_collision_layer，默认 16 = layer5）
//   采矿工具（GdHitbox/GdMelee，target_mask = 16）每次命中 take_damage(1)；
//   血量归零即「碰撞满 N 次」→ 发出 s_harvested → 按名调用采集回调 → 矿石消失。
//
// 资源再生（2026-10-12）：
//   采集后格子进 harvested 队列——spawn_deco 重铺/进场时跳过（挖掉的矿不会
//   重开游戏就全部长回）；deco_regrow_interval > 0 时每到达间隔随机挑空格
//   补种一个缺口（生长位置不固定），成功后弹出一条最早的 harvested 记录
//   （集合有界，随矿再生逐步缩短）。harvested 队列经 get/set_harvested_cells
//   交给宿主持久化（框架零业务感知；demo 侧存 XiuMapState Bean 随档落盘）。
//   再生时钟可经 set_regrow_elapsed 预推进（2026-10-12 晚）：宿主按持久化的
//   采集时间戳换算已冷却时长，重启/换图不再重置再生节拍。
//
// 碰撞层规划（demo 全局约定）：
//   layer2(2)=玩家受击盒  layer3(4)=敌受击盒  layer5(16)=可采集物受击盒
//
// 时序：子节点 ready 先于父地图（噪声未生成），ready 时 call_deferred
// spawn_deco——延迟到整棵树 ready 完成后执行，此时格子地形已就绪。
// spawn_deco 幂等（先清旧再铺新），可反复调用（换种子重铺等）。

use godot::prelude::*;
use godot::builtin::{Color, GString, NodePath, PackedColorArray, PackedFloat64Array, PackedInt32Array, PackedInt64Array, PackedStringArray, VarArray, Vector2, Vector2i};
use godot::classes::{CollisionShape2D, CircleShape2D, INode2D, Node2D, Polygon2D, ResourceLoader, Sprite2D, Texture2D};
use godot::global::godot_warn;

use crate::components::{GdHealth, GdHurtbox};

use std::collections::{HashSet, VecDeque};

/// 一条 deco 配置（从并行数组解析）
struct DecoEntry {
    terrains: Vec<String>,
    item_id: String,
    texture: String,
    color: Color,
    frequency: f64,
    mineable: bool,
    hits: i32,
    callback: String,
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdDecoLayer {
    /// 目标地形名（逗号分隔；空 = 所有可行走地形）
    #[export]
    deco_terrains: PackedStringArray,
    /// 物品 id（采集回调 / 背包联动标识）
    #[export]
    deco_item_ids: PackedStringArray,
    /// 物品贴图路径（留空用颜色）
    #[export]
    deco_textures: PackedStringArray,
    /// 回退颜色（与 item_ids 下标对应）
    #[export]
    deco_colors: PackedColorArray,
    /// 单格生成概率 0..1
    #[export]
    deco_frequencies: PackedFloat64Array,
    /// 1 = 可开采（挂受击盒 + 血量承载碰撞计数）
    #[export]
    deco_mineable: PackedInt32Array,
    /// 采集所需碰撞次数（默认 3）
    #[export]
    deco_hits: PackedInt32Array,
    /// 采集回调函数名（gd 侧在回调宿主上定义）
    #[export]
    deco_callbacks: PackedStringArray,
    /// 可采集物受击盒碰撞层（位值，默认 16 = layer5）
    #[export]
    harvest_collision_layer: i64,
    /// 采集回调宿主节点路径（空 = 父节点）
    #[export]
    callback_target_path: NodePath,
    /// 生成随机种子（0 = 取父地图 get_seed()；同一结果确定）
    #[export]
    deco_seed: i64,
    /// 资源再生间隔（秒；0 = 禁用再生）。到点随机挑空格补种一个缺口，
    /// 生长位置不固定；由宿主/demo 配置（demo 默认 120s，存档级配置下发）
    #[export]
    deco_regrow_interval: f64,

    /// 已采集格子队列（FIFO，补种成功弹出最早一条；持久化经 get/set_harvested_cells）
    harvested: VecDeque<i64>,
    /// harvested 判重集合（与队列同内容）
    harvested_set: HashSet<i64>,
    /// 每条目初始铺设数量（regrow 补足目标）
    initial_counts: Vec<i64>,
    /// 再生计时累计
    regrow_acc: f64,
    /// 再生随机源（非确定性：位置不固定）
    regrow_rng: Option<rand::rngs::StdRng>,
    /// 最近一次 spawn_deco 生成的节点数（测试断言用）
    spawned_count: i64,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdDecoLayer {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            deco_terrains: PackedStringArray::new(),
            deco_item_ids: PackedStringArray::new(),
            deco_textures: PackedStringArray::new(),
            deco_colors: PackedColorArray::new(),
            deco_frequencies: PackedFloat64Array::new(),
            deco_mineable: PackedInt32Array::new(),
            deco_hits: PackedInt32Array::new(),
            deco_callbacks: PackedStringArray::new(),
            harvest_collision_layer: 16,
            callback_target_path: NodePath::default(),
            deco_seed: 0,
            deco_regrow_interval: 0.0,
            harvested: VecDeque::new(),
            harvested_set: HashSet::new(),
            initial_counts: Vec::new(),
            regrow_acc: 0.0,
            regrow_rng: None,
            spawned_count: 0,
            base,
        }
    }

    fn ready(&mut self) {
        // 子节点 ready 先于父地图生成噪声，延迟到本帧末（整树 ready 完成）再铺
        let mut base = self.base().clone();
        base.call_deferred("spawn_deco", &[]);
    }

    fn process(&mut self, delta: f64) {
        if self.deco_regrow_interval > 0.0 && !self.harvested.is_empty() {
            self.regrow_acc += delta;
            if self.regrow_acc >= self.deco_regrow_interval {
                self.regrow_acc = 0.0;
                self.try_regrow();
            }
        }
    }
}

#[godot_api]
impl GdDecoLayer {
    /// 采集完成信号（参数：物品 id、矿石节点）
    #[signal]
    fn s_harvested(item_id: GString, node: Variant);

    /// 再生完成信号（参数：物品 id、格子 key）——宿主据此同步持久化集合
    #[signal]
    fn s_regrowned(item_id: GString, cell_key: i64);

    /// 按当前配置铺设 deco（幂等：先清旧再铺新）。
    /// ready 自动 call_deferred 调用；运行时换配置/换种子后可手动重铺。
    /// 已采集格子（harvested 队列）跳过——挖掉的矿不会重开游戏全部长回。
    /// 注意：清旧为立即 free，严禁在采集回调内调用（信号栈中释放节点不安全）。
    #[func]
    pub fn spawn_deco(&mut self) {
        // 1. 清旧（立即 free：避免与后续新节点重名触发引擎自动改名，
        //    导致按名清理/识别失效；本节点自有子节点，无外部信号栈风险）
        let old: Vec<Gd<Node2D>> = self
            .base()
            .get_children()
            .iter_shared()
            .filter_map(|n| n.try_cast::<Node2D>().ok())
            .filter(|n| {
                n.get_name()
                    .to_string()
                    .starts_with(&self.deco_node_prefix())
            })
            .collect();
        for mut n in old {
            n.free();
        }

        // 2. 解析地图（父节点 duck-type：GdQuickMap 或实现同协议的自定义地图）
        let Some(mut map) = self.resolve_map() else {
            godot_warn!(
                "GdDecoLayer: 父节点不是地图（需 get_map_width/get_terrain_name_at/is_walkable/cell_to_center），跳过铺设"
            );
            return;
        };

        // 3. 解析配置条目
        let entries = self.parse_entries();
        if entries.is_empty() {
            return;
        }

        let w = map.call("get_map_width", &[]).to::<i64>().max(0) as i32;
        let h = map.call("get_map_height", &[]).to::<i64>().max(0) as i32;
        if w <= 0 || h <= 0 {
            return;
        }

        // 4. 种子：显式配置优先，否则取父地图种子（与地形生成同源，确定性一致）
        let base_seed = if self.deco_seed > 0 {
            self.deco_seed
        } else {
            map.call("get_seed", &[]).to::<i64>()
        };

        // 5. 逐条目铺设：每条目独立 RNG（条目间分布互不相关）
        use rand::Rng;
        let mut spawned = 0i64;
        let mut per_entry: Vec<i64> = vec![0; entries.len()];
        for (ei, entry) in entries.iter().enumerate() {
            let mut rng = entry_rng(base_seed, ei);
            for cy in 0..h {
                for cx in 0..w {
                    let cell = Vector2i::new(cx, cy);
                    if !self.cell_matches(&map, cell, entry) {
                        continue;
                    }
                    if self.is_harvested(cell) {
                        continue; // 已采集待再生：不铺（含进场恢复）
                    }
                    if rng.gen_range(0.0..1.0) >= entry.frequency {
                        continue;
                    }
                    self.spawn_node(entry, ei, cell, &map);
                    spawned += 1;
                    per_entry[ei] += 1;
                }
            }
        }
        self.spawned_count = spawned;
        self.initial_counts = per_entry;
    }

    /// 已采集格子队列（i64 key = (x << 32) | y）——宿主进场注入/持久化
    #[func]
    pub fn get_harvested_cells(&self) -> PackedInt64Array {
        let mut out = PackedInt64Array::new();
        for k in &self.harvested {
            out.push(*k);
        }
        out
    }

    /// 注入已采集格子（进场恢复持久化；内部清旧队列后重建）
    #[func]
    pub fn set_harvested_cells(&mut self, cells: PackedInt64Array) {
        self.harvested.clear();
        self.harvested_set.clear();
        for k in cells.to_vec() {
            self.record_harvest_key(k);
        }
    }

    /// 预推进再生时钟（秒）：宿主恢复存档时传入冷却已流逝时长，
    /// 使首个补种 tick 对齐「最早采集格的剩余冷却」——重开游戏/换图
    /// 不再把再生节拍清零（重启后挖掉的矿不会重新等满一个间隔）。
    /// 仅影响下次 tick 时刻；传 0 恢复默认从头计时。
    #[func]
    pub fn set_regrow_elapsed(&mut self, secs: f64) {
        self.regrow_acc = secs.max(0.0);
    }

    /// 最近一次 spawn_deco 生成数量
    #[func]
    pub fn get_spawned_count(&self) -> i64 {
        self.spawned_count
    }

    // ---- 内部实现 ----

    /// 到点补种：所有缺额条目（当前存活 < 初始铺设数）各随机挑一个可生长
    /// 空格长一个——生长位置不固定；成功后弹出最早的 harvested 记录
    /// （缺口记录数守恒 = 初始数 - 存活数，随矿再生逐步清空）。
    fn try_regrow(&mut self) {
        let Some(mut map) = self.resolve_map() else {
            return;
        };
        let entries = self.parse_entries();
        if entries.is_empty() || self.initial_counts.is_empty() {
            return;
        }
        let w = map.call("get_map_width", &[]).to::<i64>().max(0) as i32;
        let h = map.call("get_map_height", &[]).to::<i64>().max(0) as i32;
        if w <= 0 || h <= 0 {
            return;
        }

        // 当前存活统计（按条目）与占用格子集合（children meta 快照）
        let mut alive: Vec<i64> = vec![0; entries.len()];
        let mut occupied: HashSet<i64> = HashSet::new();
        for n in self.base().get_children().iter_shared() {
            let Ok(obj) = n.try_cast::<Node2D>() else {
                continue;
            };
            let obj = obj.upcast::<godot::classes::Object>();
            if !obj.has_meta(&godot::builtin::StringName::from("deco_cell")) {
                continue;
            }
            let cell = obj
                .get_meta(&godot::builtin::StringName::from("deco_cell"))
                .try_to::<Vector2i>()
                .unwrap_or_default();
            occupied.insert(cell_to_key(cell));
            let ei = obj
                .get_meta(&godot::builtin::StringName::from("deco_entry"))
                .try_to::<i64>()
                .unwrap_or(-1);
            if ei >= 0 && (ei as usize) < alive.len() {
                alive[ei as usize] += 1;
            }
        }

        // 非确定性随机源（惰性创建；生长位置不固定）。
        // take 出来本地使用，避免与 self 方法调用的借用冲突
        use rand::{Rng, SeedableRng, rngs::StdRng};
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64 ^ (d.as_secs() << 20))
            .unwrap_or(12345);
        let mut rng = self
            .regrow_rng
            .take()
            .unwrap_or_else(|| StdRng::seed_from_u64(seed));

        for (ei, entry) in entries.iter().enumerate() {
            let initial = self.initial_counts.get(ei).copied().unwrap_or(0);
            if alive.get(ei).copied().unwrap_or(0) >= initial {
                continue; // 该条目无缺口
            }
            // 随机试 K 次找可生长空格（地形匹配 + 未占用；冷却中的格子
            // 允许长回——freq 高的图可能没有其他空位，长回即弹出该记录）
            for _ in 0..24 {
                let cell = Vector2i::new(rng.gen_range(0..w.max(1)), rng.gen_range(0..h.max(1)));
                if !self.cell_matches(&map, cell, entry) {
                    continue;
                }
                let key = cell_to_key(cell);
                if occupied.contains(&key) {
                    continue;
                }
                self.spawn_node(entry, ei, cell, &map);
                // 消费一条冷却记录：长回原格弹原记录；长到新格弹最早一条
                // （缺口记录数守恒 = 初始数 - 存活数）。信号发「被消费的记录」
                // ——宿主据其从持久化移除（新格 key 从未持久化，发它会对不上）
                let consumed = if self.harvested_set.contains(&key) {
                    if let Some(pos) = self.harvested.iter().position(|&k| k == key) {
                        self.harvested.remove(pos);
                    }
                    self.harvested_set.remove(&key);
                    key
                } else if let Some(front) = self.harvested.pop_front() {
                    self.harvested_set.remove(&front);
                    front
                } else {
                    key // 防御：缺口>0 意味着队列非空，正常不会走到
                };
                self.base_mut().emit_signal(
                    "s_regrowned",
                    &[GString::from(entry.item_id.as_str()).to_variant(), consumed.to_variant()],
                );
                break; // 每条目每轮最多补一个
            }
        }
        self.regrow_rng = Some(rng);
    }

    /// 格子是否在已采集冷却队列
    fn is_harvested(&self, cell: Vector2i) -> bool {
        self.harvested_set.contains(&cell_to_key(cell))
    }

    /// 记录已采集格子（去重）
    fn record_harvest_key(&mut self, key: i64) {
        if key < 0 || !self.harvested_set.insert(key) {
            return;
        }
        self.harvested.push_back(key);
    }

    /// 生成节点名前缀（清旧识别用）
    fn deco_node_prefix(&self) -> String {
        "Deco_".to_string()
    }

    /// 解析父节点为地图（duck-type 协议方法必须齐全）
    fn resolve_map(&self) -> Option<Gd<Node2D>> {
        let parent = self.base().get_parent()?;
        let map = parent.try_cast::<Node2D>().ok()?;
        for m in ["get_map_width", "get_terrain_name_at", "is_walkable", "cell_to_center"] {
            if !map.has_method(m) {
                return None;
            }
        }
        Some(map)
    }

    /// 并行数组 → 条目列表（以 item_ids 为主长度；空条目跳过）
    fn parse_entries(&self) -> Vec<DecoEntry> {
        let n = self.deco_item_ids.len();
        let mut out = Vec::new();
        for i in 0..n {
            let item_id = self
                .deco_item_ids
                .get(i)
                .map(|s| s.to_string())
                .unwrap_or_default();
            if item_id.is_empty() {
                continue;
            }
            let terrains = self
                .deco_terrains
                .get(i)
                .map(|s| s.to_string())
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let texture = self
                .deco_textures
                .get(i)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let color = self.deco_colors.get(i).unwrap_or(Color::from_rgb(0.7, 0.7, 0.75));
            let frequency = self.deco_frequencies.get(i).unwrap_or(0.0).clamp(0.0, 1.0);
            let mineable = self.deco_mineable.get(i).unwrap_or(0) != 0;
            let hits = self.deco_hits.get(i).unwrap_or(3).max(1);
            let callback = self
                .deco_callbacks
                .get(i)
                .map(|s| s.to_string())
                .unwrap_or_default();
            out.push(DecoEntry {
                terrains,
                item_id,
                texture,
                color,
                frequency,
                mineable,
                hits,
                callback,
            });
        }
        out
    }

    /// 格子是否命中条目（地形匹配 + 可行走；不可行走的山/水里不长矿石）
    fn cell_matches(&self, map: &Gd<Node2D>, cell: Vector2i, entry: &DecoEntry) -> bool {
        let mut m = map.clone();
        if !m.call("is_walkable", &[cell.to_variant()]).to::<bool>() {
            return false;
        }
        let terrain = m
            .call("get_terrain_name_at", &[cell.to_variant()])
            .to::<GString>()
            .to_string();
        if terrain.is_empty() {
            return false;
        }
        entry.terrains.is_empty() || entry.terrains.iter().any(|t| *t == terrain)
    }

    /// 生成单个 deco 节点：视觉（贴图/色块）+ 可开采组件（Hurtbox/Health）
    fn spawn_node(&mut self, entry: &DecoEntry, entry_index: usize, cell: Vector2i, map: &Gd<Node2D>) {
        let mut m = map.clone();
        let cs = m.call("get_cell_size_px", &[]).to::<i64>().max(1) as f32;
        let mut center = m
            .call("cell_to_center", &[cell.to_variant()])
            .to::<Vector2>();
        // 自然散布：格内小偏移（RNG 由调用方推进，此处用格子哈希保证确定性）
        let jitter = cell_jitter(cell);
        center += Vector2::new(jitter.0 * cs, jitter.1 * cs);

        let mut root = Node2D::new_alloc();
        let name = format!("Deco_{}_{}_{}", entry.item_id, cell.x, cell.y);
        root.set_name(&godot::builtin::StringName::from(name.as_str()));
        root.set_position(center);

        // 视觉：贴图优先，无贴图用菱形色块（双多边形做出明暗立体感）
        let tex = Self::load_texture(&entry.texture);
        match &tex {
            Some(t) if t.is_instance_valid() => {
                let mut sprite = Sprite2D::new_alloc();
                sprite.set_texture(&t.clone());
                // 等比缩放到约一格大小
                let tw = t.get_width().max(1) as f32;
                let s = (cs / tw) * 0.7;
                sprite.set_scale(Vector2::new(s, s));
                root.add_child(&sprite.upcast::<Node2D>());
            }
            _ => {
                let s = cs * 0.28;
                let mut body = Polygon2D::new_alloc();
                body.set_polygon(&diamond_points(s));
                body.set_color(entry.color);
                root.add_child(&body.upcast::<Node2D>());
                let mut shine = Polygon2D::new_alloc();
                shine.set_polygon(&diamond_points(s * 0.45));
                shine.set_color(entry.color.lightened(0.3));
                shine.set_position(Vector2::new(-s * 0.15, -s * 0.15));
                root.add_child(&shine.upcast::<Node2D>());
            }
        }

        if entry.mineable {
            // 血量承载碰撞计数：每被采矿工具命中 1 次扣 1 点，归零即采集完成
            // （#[export] 自动生成的 set_ 在结构体上，须经 bind_mut 调用）
            let mut health = GdHealth::new_alloc();
            health.bind_mut().set_max_health(entry.hits as f64);
            health.bind_mut().set_invincible_time(0.0);
            let mut health_node: Gd<godot::classes::Node> = health.clone().upcast();
            health_node.set_name(&godot::builtin::StringName::from("Health"));
            root.add_child(&health_node);

            // 受击盒：layer = 可采集物层（默认 16），采矿工具 hitbox mask 指向这里
            let mut hurt = GdHurtbox::new_alloc();
            hurt.set_collision_layer(self.harvest_collision_layer.max(0) as u32);
            hurt.set_collision_mask(0);
            let mut shape = CollisionShape2D::new_alloc();
            let mut circle = CircleShape2D::new_gd();
            circle.set_radius(cs * 0.38);
            shape.set_shape(&circle);
            hurt.add_child(&shape.upcast::<godot::classes::Node>());
            let mut hurt_node: Gd<godot::classes::Node2D> = hurt.clone().upcast();
            hurt_node.set_name(&godot::builtin::StringName::from("Hurtbox"));
            root.add_child(&hurt_node);

            // 血量归零 → 采集（绑定物品 id 与节点，回调回 deco 层统一分发）
            let item_var = GString::from(entry.item_id.as_str()).to_variant();
            let node_var = root.clone().upcast::<godot::classes::Object>().to_variant();
            let mut bind_args = VarArray::new();
            bind_args.push(&item_var);
            bind_args.push(&node_var);
            let cb = Callable::from_object_method(&*self.base_mut(), "on_ore_died").bindv(&bind_args);
            health.connect("s_died", &cb);
        }

        // 采集信息挂 meta（回调名/物品 id/格子，on_ore_died 分发与再生记录用）
        let id_meta = GString::from(entry.item_id.as_str()).to_variant();
        let cb_meta = GString::from(entry.callback.as_str()).to_variant();
        let cell_meta = cell.to_variant();
        let entry_meta = (entry_index as i64).to_variant();
        root.set_meta("deco_item_id", &id_meta);
        root.set_meta("deco_callback", &cb_meta);
        root.set_meta("deco_cell", &cell_meta);
        root.set_meta("deco_entry", &entry_meta);

        self.base_mut().add_child(&root);
    }

    /// 矿石血量归零回调（s_died 绑定 item_id + node）：发信号 → 按名调 gd 回调 → 移除矿石
    #[func]
    pub fn on_ore_died(&mut self, item_id: GString, ore: Variant) {
        // 记录已采集格子（再生队列；spawn_deco/补种跳过，宿主持久化）
        let cell_key = ore
            .try_to::<Gd<godot::classes::Object>>()
            .ok()
            .map(|o| {
                let v = o.get_meta(&godot::builtin::StringName::from("deco_cell"));
                cell_to_key(v.try_to::<Vector2i>().unwrap_or_default())
            })
            .unwrap_or(-1);
        if cell_key >= 0 {
            self.record_harvest_key(cell_key);
        }

        self.base_mut()
            .emit_signal("s_harvested", &[item_id.to_variant(), ore.to_variant()]);

        // 回调宿主：显式路径优先，否则父节点（地图根）
        let target = if self.callback_target_path.is_empty() {
            self.base().get_parent().map(|p| p.upcast::<godot::classes::Object>())
        } else {
            self.base()
                .get_node_or_null(&self.callback_target_path.clone())
                .map(|n| n.upcast::<godot::classes::Object>())
        };
        let Some(mut target) = target else {
            godot_warn!(
                "GdDecoLayer: 采集回调宿主不存在（path='{}'，物品 {} 已消失但未入库）",
                self.callback_target_path.to_string(),
                item_id
            );
            Self::free_ore(&ore);
            return;
        };
        // 回调名从矿石 meta 读取（spawn 时写入）
        let cb_name = ore
            .try_to::<Gd<godot::classes::Object>>()
            .map(|o| {
                let v = o.get_meta(&godot::builtin::StringName::from("deco_callback"));
                v.try_to::<GString>().unwrap_or_default()
            })
            .unwrap_or_default();
        if cb_name.is_empty() {
            godot_warn!("GdDecoLayer: 矿石 {} 缺少 deco_callback meta，采集未入库", item_id);
        } else if !target.has_method(&godot::builtin::StringName::from(cb_name.to_string().as_str())) {
            godot_warn!(
                "GdDecoLayer: 回调宿主 {} 上不存在方法 {}（物品 {} 未入库）",
                target.call("get_path", &[])
                    .to::<godot::builtin::NodePath>()
                    .to_string(),
                cb_name,
                item_id
            );
        } else {
            target.call(
                &godot::builtin::StringName::from(cb_name.to_string().as_str()),
                &[item_id.to_variant(), ore.clone()],
            );
        }
        Self::free_ore(&ore);
    }

    /// 矿石消失（queue_free 安全：可能在信号发射栈内）
    fn free_ore(ore: &Variant) {
        if let Ok(mut node) = ore.clone().try_to::<Gd<godot::classes::Node>>() {
            node.queue_free();
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

/// 条目独立 RNG：基础种子 + 条目下标混入（与副本层种子同款 7919 质数步进）
fn entry_rng(base_seed: i64, entry_index: usize) -> rand::rngs::StdRng {
    use rand::{SeedableRng, rngs::StdRng};
    let mixed = (base_seed as u64).wrapping_add((entry_index as u64).wrapping_mul(7919));
    StdRng::seed_from_u64(mixed)
}

/// 格子 → i64 key（x << 32 | y，格子坐标非负）
fn cell_to_key(cell: Vector2i) -> i64 {
    ((cell.x.max(0) as i64) << 32) | (cell.y.max(0) as i64 & 0xFFFF_FFFF)
}

/// 格内确定性散布偏移（格子哈希 → -0.2..0.2 格比例）
fn cell_jitter(cell: Vector2i) -> (f32, f32) {
    let h = (cell.x as u64).wrapping_mul(0x9E3779B97F4A7C15)
        ^ (cell.y as u64).rotate_left(32);
    let fx = ((h & 0xFFFF) as f32 / 65535.0 - 0.5) * 0.4;
    let fy = ((h >> 16 & 0xFFFF) as f32 / 65535.0 - 0.5) * 0.4;
    (fx, fy)
}

/// 菱形多边形顶点（矿石色块造型）
fn diamond_points(s: f32) -> PackedVector2Array {
    let mut pts = PackedVector2Array::new();
    pts.push(Vector2::new(0.0, -s));
    pts.push(Vector2::new(s * 0.8, 0.0));
    pts.push(Vector2::new(0.0, s));
    pts.push(Vector2::new(-s * 0.8, 0.0));
    pts
}
