// GdNpcSchedule - NPC 日程表（生活化行为系统核心组件）
//
// 挂在角色（GdRoleMover 子类）的子节点上，按游戏内小时（0.0~24.0 浮点）切换
// 「位置 + 行为」：到点后经地图 find_path 走网格路径前往目标格，到达后按条目
// 重配 GdNpcBrain（站桩/游走）并启动 idle 行为池（随机转身/头顶气泡自言自语）。
//
// 设计要点（详见 example/demo/xiuxian/role/npc/NPC_LIFE.md）：
//   - 时间源经 time_source_path 注入（鸭子调用 get_hour()/get_game_time()），
//     组件不依赖任何 demo 单例，测试可注入假时钟。
//   - 每日随机扰动：扰动量由 hash(npc_id, 天数) 决定，同一天内稳定、跨天不同。
//   - 后台推进 illusion：目标太远或初次落位时直接传送——NPC 随地图加载/释放，
//     玩家切回地图时 NPC 已在对应时段的位置。
//   - 暂停友好：宿主 paused（对话中）日程冻结；对话结束自动补位（含跨段重走）。
//   - 鸭子类型：宿主需 is_paused/is_grid_path_active/set_grid_path/set_facing/
//     show_bubble（demo 侧 XiuNpcBase 即 GdRoleMover）。
//
// 自动加入 "npc_schedule" 分组（GdNpcChat 按此查找同伴），meta npc_id 供匹配。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rand::Rng;

use godot::prelude::*;
use godot::builtin::{GString, PackedStringArray, PackedVector2Array, Vector2, Vector2i};
use godot::classes::{INode, Node, Node2D};

use super::npc_ai::{GdNpcBrain, AI_IDLE, AI_WANDER};

/// 分组名：本组件自动加入，GdNpcChat 按此扫描同伴
pub const NPC_SCHEDULE_GROUP: &str = "npc_schedule";

/// 单条日程（set_entries 由 GDScript 传 JSON 解析结果）
#[derive(Clone)]
struct ScheduleEntry {
    from_hour: f64,
    to_hour: f64,
    cell: Vector2i,
    /// AI_IDLE / AI_WANDER
    behavior: i64,
    /// wander 半径（像素）
    radius: f64,
    /// idle 行为池自言自语台词
    lines: Vec<String>,
    /// 变更通知用标签（空 = 自动 "cell_x,y"）
    label: String,
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdNpcSchedule {
    /// 是否启用（false = 日程冻结，宿主静止；测试/剧情事件可用）
    #[export]
    enable: bool,

    /// 角色标识（闲聊配对用；空 = 宿主节点名）
    #[export]
    npc_id: GString,

    /// 时间源节点（鸭子调用 get_hour() -> float，可选 get_game_time() -> float）
    #[export]
    time_source_path: NodePath,

    /// 每日随机扰动上限（分钟）：hash(npc_id, 天数) → [-jitter, +jitter]
    #[export]
    jitter_minutes: f64,

    /// 目标超过 teleport_cells 格时直接落位（后台推进 illusion）
    #[export]
    teleport_far: bool,

    /// 落位阈值（格数）
    #[export]
    teleport_cells: i64,

    /// idle 行为池触发间隔下限（秒）
    #[export]
    ambient_interval_min: f64,

    /// idle 行为池触发间隔上限（秒）
    #[export]
    ambient_interval_max: f64,

    /// 远距离过渡时出场行走的最长格数（走到视野外/上限即传送）
    #[export]
    exit_walk_cells: i64,

    // ---- 运行时状态 ----
    entries: Vec<ScheduleEntry>,
    /// 当前生效条目索引（-1 = 未应用）
    applied_idx: i64,
    /// 当前目标格（导航用）
    target_cell: Vector2i,
    /// 正在导航中
    navigating: bool,
    /// 已到达当前条目位置
    arrived: bool,
    /// 场景进入后的首次落位是否已完成（首次应用条目直接传送显形，不走过去）
    spawn_placed: bool,
    /// 远距离过渡阶段：0 无 / 1 出场走（走向视野外）/ 2 入场走（从视野外进目标）
    illusion: u8,
    /// 远距离过渡的完整路径（出场段/入场段都从它切片）
    full_path: PackedVector2Array,
    /// 出场段终点在 full_path 中的下标
    exit_idx: usize,
    /// 缓存的 Brain（宿主子节点查找/创建）
    brain: Option<Gd<GdNpcBrain>>,
    /// 时间源节点缓存
    time_source: Option<Gd<Object>>,
    /// idle 行为池倒计时
    ambient_timer: f64,
    /// 每日扰动缓存（天数, 偏移小时）
    day_jitter: (i64, f64),
    /// 缺时间源告警只打一次
    warned_no_time: bool,
    /// tick 累计（0.25s 节流）
    acc: f64,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdNpcSchedule {
    fn init(base: Base<Node>) -> Self {
        Self {
            enable: true,
            npc_id: GString::new(),
            time_source_path: NodePath::default(),
            jitter_minutes: 10.0,
            teleport_far: true,
            teleport_cells: 10,
            ambient_interval_min: 6.0,
            ambient_interval_max: 16.0,
            exit_walk_cells: 12,
            entries: Vec::new(),
            applied_idx: -1,
            target_cell: Vector2i::ZERO,
            navigating: false,
            arrived: false,
            spawn_placed: false,
            illusion: 0,
            full_path: PackedVector2Array::new(),
            exit_idx: 0,
            brain: None,
            time_source: None,
            ambient_timer: 4.0,
            day_jitter: (i64::MIN, 0.0),
            warned_no_time: false,
            acc: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group(NPC_SCHEDULE_GROUP);
        self.refresh_id_meta();
        // 时间源解析（ npc.gd 会把 WeatherManager 路径写进 time_source_path；
        // 解析失败不致命，tick 里按小时 -1 跳过选段并告警一次）
        if !self.time_source_path.is_empty() {
            if let Some(n) = self.base().get_node_or_null(&self.time_source_path) {
                self.time_source = Some(n.upcast());
            }
        }
    }

    fn process(&mut self, delta: f64) {
        if !self.enable {
            return;
        }
        // 0.25s 节流：日程是分钟级逻辑，无需逐帧
        self.acc += delta;
        if self.acc < 0.25 {
            return;
        }
        self.acc = 0.0;
        // 宿主 paused（对话中/闲聊中）日程与 idle 池整体冻结
        if self.host_paused() {
            return;
        }
        self.tick_schedule();
        self.tick_ambient(delta * 4.0); // 补回节流吞掉的时间
    }
}

#[godot_api]
impl GdNpcSchedule {
    #[signal]
    fn s_schedule_changed(label: GString);

    /// 批量设置日程条目（GDScript 传入 JSON 解析结果）。
    /// 每条：{ from: float, to: float, cell: [x, y], behavior: "idle"|"wander",
    ///         radius?: float, lines?: Array[String], label?: String }
    #[func]
    fn set_entries(&mut self, entries: Array<Variant>) {
        self.entries.clear();
        for v in entries.iter_shared() {
            let Ok(dict) = v.try_to::<Dictionary<Variant, Variant>>() else {
                godot_warn!("[NpcSchedule] set_entries: 条目不是 Dictionary，跳过");
                continue;
            };
            let get_f = |k: &str, def: f64| -> f64 {
                dict.get(&Variant::from(k)).and_then(|v| v.try_to::<f64>().ok()).unwrap_or(def)
            };
            let get_s = |k: &str| -> String {
                dict.get(&Variant::from(k))
                    .and_then(|v| v.try_to::<GString>().ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            };
            let cell = match dict.get(&Variant::from("cell")) {
                Some(v) => {
                    if let Ok(vi) = v.clone().try_to::<Vector2i>() {
                        vi
                    } else if let Ok(arr) = v.clone().try_to::<Array<Variant>>() {
                        let x = arr.get(0).and_then(|x| x.try_to::<f64>().ok()).unwrap_or(0.0);
                        let y = arr.get(1).and_then(|y| y.try_to::<f64>().ok()).unwrap_or(0.0);
                        Vector2i::new(x as i32, y as i32)
                    } else {
                        godot_warn!("[NpcSchedule] set_entries: cell 字段格式异常（需 Vector2i 或 [x,y]）");
                        Vector2i::ZERO
                    }
                }
                None => Vector2i::ZERO,
            };
            let behavior = match dict.get(&Variant::from("behavior")) {
                Some(v) => {
                    if let Ok(s) = v.clone().try_to::<GString>() {
                        match s.to_string().as_str() {
                            "wander" => AI_WANDER,
                            _ => AI_IDLE,
                        }
                    } else {
                        v.clone().try_to::<i64>().unwrap_or(AI_IDLE)
                    }
                }
                None => AI_IDLE,
            };
            let mut lines = Vec::new();
            if let Some(v) = dict.get(&Variant::from("lines")) {
                if let Ok(arr) = v.clone().try_to::<Array<Variant>>() {
                    for lv in arr.iter_shared() {
                        if let Ok(s) = lv.try_to::<GString>() {
                            lines.push(s.to_string());
                        }
                    }
                }
            }
            let from = get_f("from", 0.0);
            let to = get_f("to", 24.0);
            let label = {
                let l = get_s("label");
                if l.is_empty() {
                    format!("{}_{}", cell.x, cell.y)
                } else {
                    l
                }
            };
            self.entries.push(ScheduleEntry {
                from_hour: from,
                to_hour: to,
                cell,
                behavior,
                radius: get_f("radius", 64.0),
                lines,
                label,
            });
        }
        // 条目变更后强制重新应用
        self.applied_idx = -1;
    }

    /// 当前是否处于某标签条目（对话 condition_fn 可用："is_at_label:酒馆"）
    #[func]
    fn is_at_label(&self, label: GString) -> bool {
        match self.applied_idx as usize {
            i if i < self.entries.len() => self.entries[i].label == label.to_string(),
            _ => false,
        }
    }

    /// 供 GdNpcChat 查询：当前是否已到岗且不在导航/对话中
    #[func]
    fn is_at_post(&self) -> bool {
        self.arrived && !self.navigating
    }

    /// 诊断：条目数/当前应用索引/最近一次解析的 hour/时间源是否就绪
    #[func]
    fn debug_state(&self) -> Dictionary<Variant, Variant> {
        let mut d = Dictionary::<Variant, Variant>::new();
        d.set(&Variant::from("entries"), &(self.entries.len() as i64).to_variant());
        d.set(&Variant::from("applied_idx"), &self.applied_idx.to_variant());
        d.set(&Variant::from("arrived"), &self.arrived.to_variant());
        d.set(&Variant::from("navigating"), &self.navigating.to_variant());
        d.set(&Variant::from("spawn_placed"), &self.spawn_placed.to_variant());
        d.set(&Variant::from("illusion"), &(self.illusion as i64).to_variant());
        d.set(&Variant::from("time_ok"), &self.time_source.is_some().to_variant());
        d
    }
}

impl GdNpcSchedule {
    // ---- 宿主/地图鸭子访问 ----

    fn host(&self) -> Option<Gd<Node>> {
        self.base().get_parent()
    }

    fn host_n2d(&self) -> Option<Gd<Node2D>> {
        self.host().and_then(|h| h.try_cast::<Node2D>().ok())
    }

    fn host_paused(&self) -> bool {
        self.host().map(|mut h| h.call("is_paused", &[]).to::<bool>()).unwrap_or(false)
    }

    fn host_path_active(&self) -> bool {
        self.host().map(|mut h| h.call("is_grid_path_active", &[]).to::<bool>()).unwrap_or(false)
    }

    /// 地图 = 宿主父节点（demo 中 NPC 作为地图场景子节点实例化）
    fn map(&self) -> Option<Gd<Node>> {
        self.host().and_then(|h| h.get_parent())
    }

    fn refresh_id_meta(&mut self) {
        let id = if self.npc_id.is_empty() {
            self.host().map(|h| h.get_name().to_string()).unwrap_or_default()
        } else {
            self.npc_id.to_string()
        };
        self.base_mut().set_meta(&StringName::from("npc_id"), &GString::from(id.as_str()).to_variant());
    }

    // ---- 时间 ----

    fn current_hour(&mut self) -> f64 {
        // 懒解析：path 可能晚于 ready 才赋值（如 GDScript 先 add_child 再接线）
        if self.time_source.is_none() && !self.time_source_path.is_empty() {
            if let Some(n) = self.base().get_node_or_null(&self.time_source_path) {
                self.time_source = Some(n.upcast());
            }
        }
        let Some(ref src) = self.time_source else {
            if !self.warned_no_time {
                self.warned_no_time = true;
                godot_warn!("[NpcSchedule] 时间源未解析（time_source_path={:?}），日程不生效",
                    self.time_source_path.to_string());
            }
            return -1.0;
        };
        let Ok(mut o) = src.clone().try_cast::<Object>() else { return -1.0 };
        let Ok(hour) = o.call("get_hour", &[]).try_to::<f64>() else { return -1.0 };
        hour + self.day_jitter_hours(&mut o)
    }

    /// 每日扰动：hash(npc_id, 天数) → [-jitter, +jitter] 小时；同一天内稳定
    fn day_jitter_hours(&mut self, time_src: &mut Gd<Object>) -> f64 {
        if self.jitter_minutes <= 0.0 {
            return 0.0;
        }
        let total = match time_src.call("get_game_time", &[]).try_to::<f64>() {
            Ok(t) => t,
            Err(_) => return 0.0,
        };
        let day = (total / 86400.0).floor() as i64;
        if self.day_jitter.0 == day {
            return self.day_jitter.1;
        }
        let id = self.npc_id.to_string();
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        day.hash(&mut hasher);
        let h = hasher.finish();
        // [0,1) → [-1,1] * jitter
        let unit = (h % 2001) as f64 / 1000.0 - 1.0;
        let offset = unit * self.jitter_minutes / 60.0;
        self.day_jitter = (day, offset);
        offset
    }

    // ---- 选段与应用 ----

    fn find_entry(&self, hour: f64) -> i64 {
        // 归一化到 [0,24)：负 jitter 可能把 0 点推成小负值（如 -0.05h），
        // 应回绕到前一天末尾命中跨午夜段（-1 哨兵由调用方守卫，不进这里）
        let hour = ((hour % 24.0) + 24.0) % 24.0;
        for (i, e) in self.entries.iter().enumerate() {
            // from <= h < to；from > to 视为跨午夜段（如 22:00-6:00）
            let inside = if e.from_hour <= e.to_hour {
                hour >= e.from_hour && hour < e.to_hour
            } else {
                hour >= e.from_hour || hour < e.to_hour
            };
            if inside {
                return i as i64;
            }
        }
        -1
    }

    fn tick_schedule(&mut self) {
        let Some(mut host) = self.host() else { return };
        if self.entries.is_empty() {
            return;
        }
        // 对话中（mover paused）日程冻结
        if self.host_paused() {
            return;
        }
        let hour = self.current_hour();
        if hour < 0.0 {
            return; // 时间源缺失（-1 哨兵）
        }
        let idx = self.find_entry(hour);
        if idx < 0 {
            return;
        }
        if idx != self.applied_idx {
            self.applied_idx = idx;
            self.arrived = false;
            self.navigating = false;
            let entry = self.entries[idx as usize].clone();
            self.target_cell = entry.cell;
            self.apply_entry(&mut host, &entry);
            return;
        }
        // 导航完成检测：路径失效但未标记到达 → 按阶段处理
        if self.navigating && !self.host_path_active() {
            self.navigating = false;
            if self.illusion == 1 {
                // 出场走完成：传送到视野外入场点，再从那里走进目标
                let path = self.full_path.clone();
                if path.len() < 2 {
                    self.illusion = 0;
                    return;
                }
                let last = path.len() - 1;
                let enter_i = self.enter_index(&path);
                self.illusion = 2;
                if let Some(p) = path.get(enter_i) {
                    if let Ok(mut n2d) = host.clone().try_cast::<Node2D>() {
                        n2d.set_global_position(p);
                    }
                }
                host.call("clear_grid_path", &[]);
                if enter_i >= last {
                    // 入场点即目标：直接到岗
                    self.illusion = 0;
                    self.mark_arrived(&mut host);
                } else {
                    let rest = self.path_slice(&path, enter_i, last);
                    host.call("set_grid_path", &[rest.to_variant()]);
                    self.navigating = true;
                }
                return;
            }
            self.illusion = 0;
            if let Some(pos) = self.host_pos() {
                let target = self.cell_center(self.target_cell);
                if pos.distance_to(target) <= 24.0 {
                    self.mark_arrived(&mut host);
                } else {
                    // 路径被打断（如对话清空 grid_path）：重新导航
                    let idx = self.applied_idx;
                    if idx >= 0 {
                        let entry = self.entries[idx as usize].clone();
                        self.apply_entry(&mut host, &entry);
                    }
                }
            }
        }
    }

    fn apply_entry(&mut self, host: &mut Gd<Node>, entry: &ScheduleEntry) {
        let Some(mut map) = self.map() else { return };
        let cell = self.snap_walkable(&mut map, entry.cell);
        self.target_cell = cell;
        let cs = self.cell_size(&mut map) as f32;

        // 场景进入（本组件生命周期内首次应用条目）：直接在时段位置显形，
        // 不从实例化坐标走过去——玩家切图时 NPC 应"已经在那儿"
        if !self.spawn_placed {
            self.spawn_placed = true;
            self.teleport_to(host, cell, cs);
            self.mark_arrived(host);
            return;
        }

        // 已在目标格附近：直接到岗
        if let Some(pos) = self.host_pos() {
            let center = Vector2::new((cell.x as f32 + 0.5) * cs, (cell.y as f32 + 0.5) * cs);
            if pos.distance_to(center) <= cs * 0.75 {
                self.mark_arrived(host);
                return;
            }
        }

        let Some(from) = self.host_cell(&mut map) else { return };
        let path_res = map.call(
            "find_path",
            &[from.to_variant(), cell.to_variant()],
        );
        let path: PackedVector2Array = path_res.to::<PackedVector2Array>();

        // 不可达：直接落位（后台推进 illusion）
        if path.is_empty() {
            self.teleport_to(host, cell, cs);
            self.mark_arrived(host);
            return;
        }

        // 挂起 Brain（否则边走边被游走逻辑抢方向盘）
        self.ensure_brain(host).map(|mut b| b.set("enable", &false.to_variant()));

        // 近距离：正常走过去（可见移动）
        let far = self.teleport_far && path.len() as i64 > self.teleport_cells;
        if !far {
            host.call("set_grid_path", &[path.to_variant()]);
            self.navigating = true;
            return;
        }

        // 远距离：出场/入场过渡——先走到视野外，视野外传送，再从视野外走进目标，
        // 避免 NPC 在玩家眼前凭空消失/出现
        let exit_k = self.exit_index(&path);
        self.full_path = path.clone();
        self.exit_idx = exit_k;
        self.illusion = 1;
        let partial = self.path_slice(&path, 0, exit_k);
        host.call("set_grid_path", &[partial.to_variant()]);
        self.navigating = true;
    }

    fn mark_arrived(&mut self, host: &mut Gd<Node>) {
        self.arrived = true;
        self.navigating = false;
        self.illusion = 0;
        self.full_path = PackedVector2Array::new();
        let idx = self.applied_idx;
        if idx < 0 || idx as usize >= self.entries.len() {
            return;
        }
        let entry = self.entries[idx as usize].clone();
        // 重配 Brain：到岗即以此为新家（restart 清游走目标并重设 home）
        if let Some(mut b) = self.ensure_brain(host) {
            b.set("behavior", &entry.behavior.to_variant());
            b.set("wander_radius", &entry.radius.to_variant());
            b.set("enable", &true.to_variant());
            b.call("restart", &[]);
        }
        self.ambient_timer = self.rand_ambient_interval();
        let label = GString::from(entry.label.as_str());
        self.base_mut().emit_signal(
            &StringName::from("s_schedule_changed"),
            &[label.to_variant()],
        );
    }

    fn teleport_to(&mut self, host: &mut Gd<Node>, cell: Vector2i, cs: f32) {
        let center = Vector2::new((cell.x as f32 + 0.5) * cs, (cell.y as f32 + 0.5) * cs);
        if let Ok(mut n2d) = host.clone().try_cast::<Node2D>() {
            n2d.set_global_position(center);
        }
        // 打断中的路径一并清掉
        host.call("clear_grid_path", &[]);
    }

    // ---- 远距离过渡：视野外出场/入场 ----

    /// 当前相机视野对应的世界矩形（headless/无相机时为默认视口矩形）
    fn visible_world_rect(&self) -> Option<Rect2> {
        let host = self.host_n2d()?;
        let vp = host.get_viewport()?;
        let ct = vp.get_canvas_transform();
        let size = vp.get_visible_rect().size;
        if size.x <= 0.0 || size.y <= 0.0 {
            return None;
        }
        // 画布逆变换把视口四角映射回世界坐标，取包围盒
        let inv = ct.affine_inverse();
        let c0 = inv * Vector2::ZERO;
        let c1 = inv * Vector2::new(size.x, 0.0);
        let c2 = inv * Vector2::new(0.0, size.y);
        let c3 = inv * size;
        let min_x = c0.x.min(c1.x).min(c2.x).min(c3.x);
        let min_y = c0.y.min(c1.y).min(c2.y).min(c3.y);
        let max_x = c0.x.max(c1.x).max(c2.x).max(c3.x);
        let max_y = c0.y.max(c1.y).max(c2.y).max(c3.y);
        Some(Rect2::new(
            Vector2::new(min_x, min_y),
            Vector2::new(max_x - min_x, max_y - min_y),
        ))
    }

    /// 视野外判定用矩形：视野向外扩 margin 像素（给"走到视野外"留缓冲）
    fn grown_view_rect(&self, rect: Rect2) -> Rect2 {
        let m = 48.0;
        Rect2::new(
            rect.position - Vector2::new(m, m),
            rect.size + Vector2::new(m * 2.0, m * 2.0),
        )
    }

    /// 出场段终点下标：路径上第一个视野外点（至少走到下标 1），
    /// 上限 exit_walk_cells，且给入场段至少留 1 个点
    fn exit_index(&self, path: &PackedVector2Array) -> usize {
        let cap = (self.exit_walk_cells.max(1) as usize)
            .min(path.len().saturating_sub(2))
            .max(1);
        let Some(rect) = self.visible_world_rect() else {
            return cap;
        };
        let grown = self.grown_view_rect(rect);
        for i in 1..=cap {
            let Some(p) = path.get(i) else { continue };
            if !grown.contains_point(p) {
                return i;
            }
        }
        cap
    }

    /// 入场传送下标：从目标端往回找第一个视野外点；找不到（全程可见，
    /// 如小地图或测试环境）则退化为给入场走留 teleport_cells 格
    fn enter_index(&self, path: &PackedVector2Array) -> usize {
        let last = path.len() - 1;
        let exit_k = self.exit_idx;
        let fallback = if last > exit_k + self.teleport_cells.max(1) as usize {
            last - self.teleport_cells.max(1) as usize
        } else {
            (exit_k + 1).min(last)
        };
        let Some(rect) = self.visible_world_rect() else {
            return fallback;
        };
        let grown = self.grown_view_rect(rect);
        for i in ((exit_k + 1)..last).rev() {
            let Some(p) = path.get(i) else { continue };
            if !grown.contains_point(p) {
                return i;
            }
        }
        fallback
    }

    /// 路径切片 [from, to]（闭区间；out of range 自动钳制）
    fn path_slice(&self, path: &PackedVector2Array, from: usize, to: usize) -> PackedVector2Array {
        let last = path.len().saturating_sub(1);
        let from = from.min(last);
        let to = to.min(last);
        let mut out = PackedVector2Array::new();
        if from > to {
            return out;
        }
        for i in from..=to {
            if let Some(p) = path.get(i) {
                out.push(p);
            }
        }
        out
    }

    // ---- idle 行为池：随机转身 / 自言自语气泡 ----

    fn tick_ambient(&mut self, delta: f64) {
        if !self.arrived || self.navigating {
            return;
        }
        let idx = self.applied_idx;
        if idx < 0 || idx as usize >= self.entries.len() {
            return;
        }
        if self.entries[idx as usize].behavior != AI_IDLE {
            return;
        }
        self.ambient_timer -= delta;
        if self.ambient_timer > 0.0 {
            return;
        }
        self.ambient_timer = self.rand_ambient_interval();
        let Some(mut host) = self.host() else { return };
        let mut rng = rand::thread_rng();
        let lines = &self.entries[idx as usize].lines;
        if !lines.is_empty() && rng.gen_bool(0.65) {
            let line = lines[rng.gen_range(0..lines.len())].clone();
            // 宿主 show_bubble 优先，否则 rust 默认头顶气泡
            crate::dialog::show_bubble_on(&mut host, &line, 3.0);
        } else {
            // 环顾四周
            let dirs = ["left", "right", "up", "down"];
            let dir = dirs[rng.gen_range(0..dirs.len())];
            if host.has_method("set_facing") {
                host.call("set_facing", &[GString::from(dir).to_variant()]);
            }
        }
    }

    fn rand_ambient_interval(&self) -> f64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(self.ambient_interval_min..=self.ambient_interval_max.max(self.ambient_interval_min + 0.1))
    }

    // ---- Brain / 地形工具 ----

    fn ensure_brain(&mut self, host: &mut Gd<Node>) -> Option<Gd<GdNpcBrain>> {
        if let Some(ref b) = self.brain {
            if b.is_instance_valid() {
                return Some(b.clone());
            }
        }
        // 先找宿主已有 Brain（静态 ai_behavior 创建的）
        for i in 0..host.get_child_count() {
            if let Some(c) = host.get_child(i) {
                if let Ok(b) = c.try_cast::<GdNpcBrain>() {
                    self.brain = Some(b);
                    return self.brain.clone();
                }
            }
        }
        // 没有就建一个（日程接管行为）
        let mut b = GdNpcBrain::new_alloc();
        b.set_name(&StringName::from("ScheduleBrain"));
        host.add_child(&b);
        self.brain = Some(b);
        self.brain.clone()
    }

    fn host_pos(&self) -> Option<Vector2> {
        self.host_n2d().map(|mut n| n.get_global_position())
    }

    fn host_cell(&self, map: &mut Gd<Node>) -> Option<Vector2i> {
        let pos = self.host_pos()?;
        let cell = map.call("world_to_cell", &[pos.to_variant()]);
        cell.try_to::<Vector2i>().ok()
    }

    fn cell_size(&self, map: &mut Gd<Node>) -> i64 {
        map.call("get_cell_size_px", &[]).to::<i64>()
    }

    fn cell_center(&self, cell: Vector2i) -> Vector2 {
        // 尺寸未知时用 32 兜底（仅用于距离粗判）
        Vector2::new((cell.x as f32 + 0.5) * 32.0, (cell.y as f32 + 0.5) * 32.0)
    }

    /// BFS 吸附到最近可行走格（站位/目标格落在水/山时），与 npc.gd 落位逻辑一致
    fn snap_walkable(&self, map: &mut Gd<Node>, cell: Vector2i) -> Vector2i {
        let walk = |c: Vector2i, map: &mut Gd<Node>| map.call("is_walkable", &[c.to_variant()]).to::<bool>();
        if walk(cell, map) {
            return cell;
        }
        let w = map.call("get_map_width", &[]).to::<i64>() as i32;
        let h = map.call("get_map_height", &[]).to::<i64>() as i32;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(cell);
        let mut seen = std::collections::HashSet::new();
        seen.insert(cell);
        while let Some(cur) = queue.pop_front() {
            if walk(cur, map) {
                return cur;
            }
            for off in [Vector2i::new(1, 0), Vector2i::new(-1, 0), Vector2i::new(0, 1), Vector2i::new(0, -1)] {
                let nxt = cur + off;
                if nxt.x < 0 || nxt.y < 0 || nxt.x >= w || nxt.y >= h || seen.contains(&nxt) {
                    continue;
                }
                seen.insert(nxt);
                queue.push_back(nxt);
            }
        }
        cell
    }
}
