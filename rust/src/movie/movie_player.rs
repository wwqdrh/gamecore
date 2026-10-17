// GdMoviePlayer - 剧情演出播放器（dialog 功能扩展）
//
// 在对话基础上扩展演出编排：场景加载、人物出场/退场/移动、相机运镜、
// 对话框/气泡双形态台词。剧本是行式 DSL（.movie 文本，语法见 script.rs），
// 节点引用声明在 game_config.json 的 `movie` 段：
//
//   "movie": {
//     "actors": {
//       "xiao_master": {
//         "scene": "res://example/demo/xiuxian/role/npc/xiao_master/xiao_master.tscn",
//         "display_name": "萧老爷"
//       }
//     }
//   }
//
// 依赖注入（场景装配 export）：
//   · dialog_box_path —— 对话宿主（可选）：实现了 show_movie_line 契约则
//     台词走宿主自定义 UI；否则用 rust 默认对话框（dialog/movie_ui.rs，
//     屏幕下方黑框，点击/E 推进）。has_flag/set_flag 契约仍走该宿主
//     （demo dialog_box.gd 委托对话状态层）。
//   · player_path —— 玩家节点（演出期 set_paused(true) 锁输入；camera
//     follow player 的 target）
// 气泡：show_bubble_on 统一入口——宿主有 show_bubble 用宿主，否则自动挂
// rust 默认头顶气泡（GdMovieBubble）。
// MapManager / ViewCamera 经 GdSceneRoot 组件表查找（框架标准入口）。
//
// 执行模型：play() 启动协程逐指令执行；say/move/wait/camera pan 等指令
// await 完成（点击推进/走完/定时）再执行下一条；jump/if 驱动分支。
// scene 指令：同图跳过重开；跨图切换前拍快照（原地图+玩家格），end/播完/
// stop 时自动回迁——演出是"暂借舞台"，不把世界永久留在剧本最后一张图
// （演出后要真换图由调用方监听 s_movie_finished 自行 open_map）。
// 信号：s_movie_started / s_movie_finished。

use std::collections::{HashMap, HashSet};

use godot::builtin::{GString, PackedVector2Array, Variant, Vector2, Vector2i};
use godot::classes::{Engine, INode, Node, PackedScene, Time};
use godot::prelude::*;

use crate::dialog::{show_bubble_on, GdMovieDialogBox};
use crate::manager::config_manager::ensure_config_manager;
use crate::movie::script::{parse, ActorAction, CamAction, Direction, MovieCmd, MovieScript};
use crate::runtime::coroutine::SpireCoroutine;
use crate::runtime::start_coroutine::StartCoroutine;
use crate::runtime::yielding::{frames, seconds};

/// config `movie.actors` 条目
#[derive(Debug, Clone, Default)]
struct ActorDef {
    scene_path: String,
    display_name: String,
}

    /// 协程单步执行结果：同步动作已完成，这里只描述后续等待
    enum WaitStep {
        None,
        /// 定时等待
        Time(f64),
        /// 等 N 帧
        Frames(i64),
        /// 等台词确认（target 的 movie_line_done 置位；dur 兜底超时）
        LineDone { target: Gd<Node>, dur_max: f64 },
        /// 等 actor 走完网格路径（is_grid_path_active 变 false；timeout 兜底）
        GridDone { actor: Gd<Node>, timeout: f64 },
        /// 走完路径后退场（等 GridDone 随后释放角色——走出画面的 exit）
        ExitAfterWalk { actor: Gd<Node>, id: String, timeout: f64 },
        /// 跳转（pc 重设）
        Jump(usize),
        /// 剧本 end 指令：跳出主循环走收尾（快照回迁 + finish）
        Done,
    }

const GRID_TIMEOUT_SECS: f64 = 20.0;
const MOVIE_FLAG_DONE: &str = "movie_line_done";

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdMoviePlayer {
    base: Base<Node>,

    /// 剧本文件路径（res://.../xxx.movie），设置时加载解析
    #[var]
    script_path: GString,
    /// 台词宿主节点路径（对话框；鸭子契约见文件头）
    #[var]
    dialog_box_path: NodePath,
    /// 玩家节点路径（演出锁输入 + camera follow player 目标）
    #[var]
    player_path: NodePath,
    /// 演出结束是否释放本播放器实例化出的角色（false = 保留在场，供后续
    /// 演出/交互复用；此时重复 enter 同 id 会先收掉旧实例）
    #[var]
    auto_free_actors: bool,

    /// 是否正在演出
    playing: bool,
    /// 剧本缓存
    script: Option<MovieScript>,
    /// config movie.actors
    actor_defs: HashMap<String, ActorDef>,
    /// 在场角色 (id -> 节点)
    actors: HashMap<String, Gd<Node>>,
    /// 本播放器实例化的角色 id（auto_free_actors 时 exit/结束释放）
    owned_actors: HashSet<String>,
    /// 运行中的演出协程
    coroutine: Option<Gd<SpireCoroutine>>,
    /// 台词宿主缓存
    dialog_box: Option<Gd<Node>>,
    /// 玩家缓存
    player: Option<Gd<Node>>,
    /// rust 默认对话框（宿主无 show_movie_line 契约时懒创建，见 dialog/movie_ui.rs）
    default_box: Option<Gd<Node>>,
    /// scene 跨图快照 (原地图 alias, 玩家所在格)：首次真正切图前记录，
    /// 演出结束回迁（世界状态不被剧情永久改写；同图演出不产生快照）
    scene_snapshot: Option<(GString, Vector2i)>,
    /// 回迁落位（restore 后等地图装配再校正玩家位置）
    pending_restore_cell: Option<Vector2i>,
    /// 演出期隐藏的常驻 NPC（结束恢复可见；dedup：地图内嵌 NPC 与剧本
    /// actor 同屏重复）
    suppressed_npcs: Vec<Gd<Node>>,
    /// 待执行的 NPC 抑制（play 开局 / scene 指令开图·同图跳过后置位，
    /// 协程下一指令前消费——等装配帧完成再收，新图 NPC 才已就位）
    pending_suppress: bool,
}

#[godot_api]
impl INode for GdMoviePlayer {
    fn init(base: Base<Node>) -> Self {
        GdMoviePlayer {
            base,
            script_path: GString::new(),
            dialog_box_path: NodePath::default(),
            player_path: NodePath::default(),
            auto_free_actors: true,
            playing: false,
            script: None,
            actor_defs: HashMap::new(),
            actors: HashMap::new(),
            owned_actors: HashSet::new(),
            coroutine: None,
            dialog_box: None,
            player: None,
            default_box: None,
            scene_snapshot: None,
            pending_restore_cell: None,
            suppressed_npcs: Vec::new(),
            pending_suppress: false,
        }
    }

    fn ready(&mut self) {
        self.resolve_refs();
        if !self.actor_defs.is_empty() {
            return; // 已从 config 加载过
        }
        self.load_actor_defs();
        if !self.script_path.is_empty() {
            self.load_script(&self.script_path.clone());
        }
    }

    fn exit_tree(&mut self) {
        // 场景销毁：停演出 + 清理实例化角色（协程 kill 不触发 finished）
        self.stop();
        self.despawn_all_actors();
    }
}

#[godot_api]
impl GdMoviePlayer {
    #[signal]
    fn s_movie_started();

    #[signal]
    fn s_movie_finished();

    // === 公开方法 ===

    /// 运行时配置（幂等）：装配路径 / 剧本 / 角色表一并就位。
    /// tscn 装配走 #[var] 自动 setter（ready 时统一 resolve），代码动态
    /// 构造（测试/热接）走本方法
    #[func]
    pub fn configure(&mut self, script_path: GString, dialog_box_path: NodePath, player_path: NodePath) {
        self.script_path = script_path.clone();
        self.dialog_box_path = dialog_box_path;
        self.player_path = player_path;
        self.resolve_refs();
        self.load_actor_defs();
        if !script_path.is_empty() {
            self.load_script(&script_path);
        }
    }

    /// 是否正在演出
    #[func]
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// 从头播放（需先 set_script_path 或 ready 时已配置）
    #[func]
    pub fn play(&mut self) {
        self.play_from(GString::new());
    }

    /// 从指定标签播放；label 空则从头
    #[func]
    pub fn play_from(&mut self, label: GString) {
        if self.playing {
            godot_warn!("GdMoviePlayer: already playing, ignore play");
            return;
        }
        let script = match &self.script {
            Some(s) => s.clone(),
            None => {
                godot_warn!("GdMoviePlayer: no script loaded (script_path='{}')", self.script_path);
                return;
            }
        };

        let start_pc = if label.is_empty() {
            0
        } else {
            match script.labels.get(&label.to_string()) {
                Some(&pc) => pc,
                None => {
                    godot_warn!("GdMoviePlayer: label '{}' not found", label);
                    return;
                }
            }
        };

        self.playing = true;
        // 演出开局先收一遍当前地图常驻 NPC（无 scene 指令的剧本也要 dedup；
        // 有 scene 指令时开图/同图分支会再置位，下一指令前统一消费）
        self.pending_suppress = true;
        self.lock_player(true);
        self.base_mut().emit_signal("s_movie_started", &[]);

        let this = self.to_gd();
        let cmds = script.cmds;
        let co = this.clone().start_coroutine(Self::run(this, cmds, start_pc));
        self.coroutine = Some(co);
    }

    /// 停止演出（释放实例化角色、解锁玩家、回迁 scene 快照；不发
    /// s_movie_finished——与自然播完区分，调用方按需自行处理收尾）
    #[func]
    pub fn stop(&mut self) {
        if let Some(mut co) = self.coroutine.take() {
            if co.is_instance_valid() {
                co.call("kill", &[]);
            }
        }
        if self.playing {
            self.playing = false;
            // 快照回迁（同步：open_map 内 s_map_changed 级联玩家 _apply_map）
            self.restore_npcs();
            self.restore_snapshot();
            self.settle_restore_pos();
            self.lock_player(false);
            self.close_dialog_box();
            self.despawn_all_actors();
        }
    }

    /// 查询在场角色节点（跨组件取演出角色用，如测试断言）
    #[func]
    pub fn get_actor(&self, id: GString) -> Variant {
        self.actors
            .get(&id.to_string())
            .map(|n| n.to_variant())
            .unwrap_or_else(Variant::nil)
    }

    // === 协程主体 ===

    /// 演出协程：逐指令执行；同步指令在 bind_mut 内完成，等待类在协程
    /// await（不跨 await 持 bind_mut 借用）
    async fn run(mut this: Gd<GdMoviePlayer>, cmds: Vec<MovieCmd>, mut pc: usize) -> GString {
        let total = cmds.len();
        while pc < total {
            // 1. 同步执行本条指令，得到等待动作
            let wait = {
                let mut me = this.bind_mut();
                if !me.playing {
                    // stop() 已杀协程，这里是防御：不再执行
                    return GString::from("aborted");
                }
                // scene 开图/同图跳过后置位的 NPC 抑制：下一指令前消费
                // （跨图时已等过 Frames(2) 装配帧，新图 NPC 已就位）
                if me.pending_suppress {
                    me.pending_suppress = false;
                    me.suppress_map_npcs();
                }
                match &cmds[pc] {
                    MovieCmd::Jump { label } => WaitStep::Jump(me.resolve_label(label)),
                    MovieCmd::JumpIf { flag, label } => {
                        if me.has_flag(flag) {
                            WaitStep::Jump(me.resolve_label(label))
                        } else {
                            WaitStep::None
                        }
                    }
                    MovieCmd::End => WaitStep::Done,
                    cmd => me.exec_cmd(cmd),
                }
            };

            // 2. 处理跳转 / 等待
            match wait {
                WaitStep::Done => break,
                WaitStep::Jump(new_pc) => {
                    pc = new_pc;
                    continue;
                }
                WaitStep::None => {}
                WaitStep::Time(secs) => seconds(secs).await,
                WaitStep::Frames(n) => frames(n).await,
                WaitStep::LineDone { target, dur_max } => {
                    Self::wait_line_done(&target, dur_max).await;
                }
                WaitStep::GridDone { actor, timeout } => {
                    Self::wait_grid_done(actor, timeout).await;
                }
                WaitStep::ExitAfterWalk { actor, id, timeout } => {
                    Self::wait_grid_done(actor, timeout).await;
                    // 走出画面后再收角色
                    let mut me = this.bind_mut();
                    me.actor_exit(&id);
                }
            }
            pc += 1;
        }

        // 收尾（end 指令或播完共用）：先回迁 scene 快照（跨图演出恢复
        // 原地图与玩家位置），等地图装配稳定后校正落位，再走 finish
        let restored = { this.bind_mut().restore_snapshot() };
        if restored {
            frames(2).await; // 等地图装配 + 玩家 _apply_map（s_map_changed 同步级联）
            this.bind_mut().settle_restore_pos();
        }
        {
            let mut me = this.bind_mut();
            me.finish();
        }
        GString::from(if restored { "ended_restored" } else { "completed" })
    }

    /// 等待台词推进：轮询 target 的 movie_line_done（玩家点击置位），
    /// dur_max 兜底超时（0 表示不限时）
    async fn wait_line_done(control: &Gd<Node>, dur_max: f64) {
        let start_ms = now_ms();
        loop {
            frames(1).await;
            let done: bool = control.get(MOVIE_FLAG_DONE).to();
            if done {
                return;
            }
            if dur_max > 0.0 && (now_ms() - start_ms) as f64 / 1000.0 >= dur_max {
                return;
            }
        }
    }

    /// 等待 actor 走完网格路径（is_grid_path_active 变 false），超时兜底
    async fn wait_grid_done(mut actor: Gd<Node>, timeout: f64) {
        let start_ms = now_ms();
        loop {
            frames(1).await;
            if !actor.is_instance_valid() {
                return;
            }
            let active: bool = actor.call("is_grid_path_active", &[]).to();
            if !active {
                return;
            }
            if (now_ms() - start_ms) as f64 / 1000.0 >= timeout {
                godot_warn!("GdMoviePlayer: move timeout ({}s), force continue", timeout);
                if actor.is_instance_valid() {
                    actor.call("clear_grid_path", &[]);
                }
                return;
            }
        }
    }

    // === 指令执行（同步部分，bind_mut 内调用）===

    fn exec_cmd(&mut self, cmd: &MovieCmd) -> WaitStep {
        match cmd {
            MovieCmd::Scene { alias, spawn } => self.exec_scene(alias, *spawn),
            MovieCmd::PlayerPos { cell } => self.exec_player_pos(*cell),
            MovieCmd::Actor { id, action } => self.exec_actor(id, action),
            MovieCmd::Move { id, cell } => self.exec_move(id, *cell),
            MovieCmd::Camera(cam) => self.exec_camera(cam),
            MovieCmd::Say { id, text, dur } => self.exec_say(id, text, *dur),
            MovieCmd::Bubble { id, secs, text } => self.exec_bubble(id, *secs, text),
            MovieCmd::Wait { secs } => WaitStep::Time(*secs),
            MovieCmd::SetFlag { name } => {
                self.set_flag(name);
                WaitStep::None
            }
            // Jump/JumpIf/End 在 run() 分支处理，正常不会到这里
            _ => WaitStep::None,
        }
    }    /// 场景加载：经组件表取 MapManager.open_map（别名未注册/失败只告警继续）。
    /// · 同图演出跳过重开（避免地图重载/NPC 重建的整场洗牌）；
    /// · 跨图切换前拍快照（原地图 + 玩家所在格），演出结束自动回迁——
    ///   演出是"暂借舞台"，不把世界永久留在剧本最后一张图上。
    ///   剧本若要演出后真的换图（如剧情位移），由调用方监听 s_movie_finished
    ///   后自行 open_map（显式优于隐式）。
    fn exec_scene(&mut self, alias: &str, spawn: Option<Vector2i>) -> WaitStep {
        let Some(mut mgr) = self.find_map_manager() else {
            godot_warn!("GdMoviePlayer: scene '{}' skipped, MapManager not found", alias);
            return WaitStep::None;
        };
        let alias_g = GString::from(alias);
        let cur: GString = mgr.call("get_current_alias", &[]).to();
        if !cur.is_empty() && cur == alias_g {
            // 同图：跳过重复开关（保留现场，无需快照回迁）；常驻 NPC 待抑制
            // （否则内嵌 NPC 与剧本 actor 同屏重复）
            self.pending_suppress = true;
            return WaitStep::Frames(1);
        }
        if self.scene_snapshot.is_none() {
            let cell = self.snapshot_player_cell(&mut mgr);
            self.scene_snapshot = Some((cur.clone(), cell));
        }
        let spawn = spawn.unwrap_or(Vector2i::ZERO);
        let ok: bool = mgr
            .call("open_map", &[alias_g.to_variant(), spawn.to_variant()])
            .to();
        if !ok {
            godot_warn!("GdMoviePlayer: open_map('{}') failed", alias);
        }
        // 地图装配（含 NPC _ready 级联）需要帧末落地；NPC 抑制待装配后执行
        self.pending_suppress = true;
        WaitStep::Frames(2)
    }

    /// 快照玩家当前所在格（取不到玩家/地图时退回 MapManager 记录的出生格）
    fn snapshot_player_cell(&mut self, mgr: &mut Gd<Node>) -> Vector2i {
        if let Some(mut player) = self.resolve_target("player") {
            if let Some(mut map) = self.current_map() {
                let cs = self.map_cell_size(&mut map);
                return self.node_cell(&mut player, cs);
            }
        }
        mgr.call("get_spawn_cell", &[]).to()
    }

    /// 回迁 scene 快照：切回演出前的地图（同步，s_map_changed 级联玩家
    /// _apply_map）；返回是否发生回迁
    fn restore_snapshot(&mut self) -> bool {
        let Some((alias, cell)) = self.scene_snapshot.take() else {
            return false;
        };
        self.pending_restore_cell = Some(cell);
        let Some(mut mgr) = self.find_map_manager() else {
            godot_warn!("GdMoviePlayer: snapshot restore skipped, MapManager not found");
            return true;
        };
        let ok: bool = mgr
            .call("open_map", &[alias.to_variant(), cell.to_variant()])
            .to();
        if !ok {
            godot_warn!("GdMoviePlayer: snapshot restore open_map('{}') failed", alias);
        }
        true
    }

    /// 回迁落位校正：把玩家放回演出前所在格（open_map 后玩家自身按存档
    /// 恢复逻辑落位，可能与演出前实际站位有出入，这里显式归位）
    fn settle_restore_pos(&mut self) {
        let Some(cell) = self.pending_restore_cell.take() else { return };
        let Some(mut player) = self.resolve_target("player") else { return };
        let Some(mut map) = self.current_map() else { return };
        let cs = self.map_cell_size(&mut map);
        // 停掉可能的移动队列再落位
        if player.has_method("stop") {
            player.call("stop", &[]);
        }
        node_set_pos(&mut player, cell_center(cell, cs));
    }

    /// 玩家瞬移落格：显式声明演出站位（玩家位置随档持久化，scene spawn
    /// 出生点可能被存档恢复位置覆盖；对话前用它保证双方站位靠近）
    fn exec_player_pos(&mut self, cell: Vector2i) -> WaitStep {
        let Some(mut player) = self.resolve_target("player") else {
            godot_warn!("GdMoviePlayer: player (x,y) skipped, no player node");
            return WaitStep::None;
        };
        let Some(mut map) = self.current_map() else { return WaitStep::None };
        let cs = self.map_cell_size(&mut map);
        // 停掉进行中的移动再落位，防路径队列把位置又拉回去
        if player.has_method("stop") {
            player.call("stop", &[]);
        }
        node_set_pos(&mut player, cell_center(cell, cs));
        WaitStep::Frames(1)
    }

    fn exec_actor(&mut self, id: &str, action: &ActorAction) -> WaitStep {
        match action {
            ActorAction::Enter { cell, from } => self.exec_enter(id, *cell, *from),
            ActorAction::Exit { to, dir } => self.exec_exit(id, *to, *dir),
            ActorAction::Teleport { cell } => {
                self.actor_teleport(id, *cell);
                WaitStep::None
            }
        }
    }

    /// 角色出场：按 config 实例化挂到当前地图；同 id 旧实例先收掉。
    /// from 有值时落在 from 格并走直线网格路径到 cell（屏幕外走入进场），
    /// 返回在场节点（实例化失败返回 None）
    fn exec_enter(&mut self, id: &str, cell: Option<Vector2i>, from: Option<Vector2i>) -> WaitStep {
        let Some(mut actor) = self.actor_enter(id, from.or(cell)) else {
            return WaitStep::Frames(1);
        };
        // 走入进场：起点(from) → 落点位(cell) 不同时走过去再继续
        if let (Some(f), Some(c)) = (from, cell) {
            if f != c {
                if let Some(mut map) = self.current_map() {
                    let cs = self.map_cell_size(&mut map);
                    let mut node = actor.clone();
                    let path = self.build_walk_path(&mut node, c, cs);
                    if !path.is_empty() {
                        actor.call("set_grid_path", &[path.to_variant()]);
                        return WaitStep::GridDone { actor, timeout: GRID_TIMEOUT_SECS };
                    }
                }
            }
        }
        WaitStep::Frames(1)
    }

    /// 角色退场：to 指定格 / dir 方向词时先走出画面再消失；缺省原地消失。
    /// 实例化的角色 free；外部注入的角色仅从注册表摘除不动原节点
    fn exec_exit(
        &mut self,
        id: &str,
        to: Option<Vector2i>,
        dir: Option<Direction>,
    ) -> WaitStep {
        let staged = self
            .actors
            .get(id)
            .cloned()
            .filter(|n| n.is_instance_valid());
        let Some(mut actor) = staged else {
            // 不在场：按原语义摘表 + 收尾
            self.actor_exit(id);
            return WaitStep::Frames(1);
        };
        // 退场目标格：显式 to 优先，其次方向词（当前位置沿方向走出 N 格）
        let target = match (to, dir) {
            (Some(t), _) => Some(t),
            (None, Some(d)) => self.current_map().map(|mut map| {
                let cs = self.map_cell_size(&mut map);
                self.node_cell(&mut actor, cs) + d.offset()
            }),
            (None, None) => None,
        };
        if let Some(cell) = target {
            if let Some(mut map) = self.current_map() {
                let cs = self.map_cell_size(&mut map);
                let path = self.build_walk_path(&mut actor, cell, cs);
                if !path.is_empty() {
                    actor.call("set_grid_path", &[path.to_variant()]);
                    return WaitStep::ExitAfterWalk {
                        actor,
                        id: id.to_string(),
                        timeout: GRID_TIMEOUT_SECS,
                    };
                }
            }
        }
        self.actor_exit(id);
        WaitStep::Frames(1)
    }

    /// 角色退场：实例化的角色 free；外部注入的角色仅从注册表摘除不动原节点
    fn actor_exit(&mut self, id: &str) {
        if let Some(mut node) = self.actors.remove(id) {
            if self.owned_actors.contains(id) {
                if node.is_instance_valid() {
                    node.call_deferred("queue_free", &[]);
                }
                self.owned_actors.remove(id);
            }
        }
    }

    /// 实例化角色并落位到指定格（None = (0,0) 由宿主 npc_cell 自吸附），
    /// 成功返回在场节点
    fn actor_enter(&mut self, id: &str, cell: Option<Vector2i>) -> Option<Gd<Node>> {
        self.actor_exit(id); // 重复 enter 自愈
        let def = self.actor_defs.get(id).cloned()?;
        if def.scene_path.is_empty() {
            godot_warn!("GdMoviePlayer: actor '{}' has no scene in config", id);
            return None;
        }
        let scene_res = godot::classes::ResourceLoader::singleton().load(def.scene_path.as_str());
        let Some(scene_res) = scene_res else {
            godot_warn!("GdMoviePlayer: actor '{}' scene load failed: {}", id, def.scene_path);
            return None;
        };
        let Ok(packed) = scene_res.try_cast::<PackedScene>() else {
            godot_warn!("GdMoviePlayer: actor '{}' scene is not PackedScene", id);
            return None;
        };
        let Some(mut map) = self.current_map() else {
            godot_warn!("GdMoviePlayer: actor '{}' enter skipped, no current map", id);
            return None;
        };
        let Some(node) = packed.instantiate() else {
            godot_warn!("GdMoviePlayer: actor '{}' instantiate failed", id);
            return None;
        };
        let mut node = node;
        node.set_name(id);
        // 落位：优先经 demo 宿主的 npc_cell（ready 自吸附最近可行走格）
        if let Some(cell) = cell {
            node.set("npc_cell", &cell.to_variant());
        }
        map.add_child(&node.clone());
        // 演出模式契约（鸭子调用）：屏蔽宿主自治系统——日程「场景进入直接
        // 落位」/Brain 漫游会在演出数秒后把角色瞬移走，站位吸附也一并禁用
        if node.has_method("set_movie_actor") {
            node.call("set_movie_actor", &[true.to_variant()]);
        }
        if let Some(cell) = cell {
            let cs = self.map_cell_size(&mut map);
            node_set_pos(&mut node, cell_center(cell, cs));
        }
        if !def.display_name.is_empty() {
            node.set("display_name", &def.display_name.as_str().to_variant());
        }
        self.actors.insert(id.to_string(), node.clone());
        self.owned_actors.insert(id.to_string());
        Some(node)
    }

    fn actor_teleport(&mut self, id: &str, cell: Vector2i) {
        let Some(node) = self.actors.get(id).cloned() else {
            godot_warn!("GdMoviePlayer: teleport '{}', actor not on stage", id);
            return;
        };
        if !node.is_instance_valid() {
            return;
        }
        let Some(mut map) = self.current_map() else { return };
        let cs = self.map_cell_size(&mut map);
        let mut n2d = node;
        node_set_pos(&mut n2d, cell_center(cell, cs));
    }

    /// 网格移动：地图有 find_path（GdQuickMap）则寻路；否则（GdDigLayer 等
    /// 协议无寻路能力）直线逐格路径；均不可行时直接落格（保持演出连续）
    fn exec_move(&mut self, id: &str, cell: Vector2i) -> WaitStep {
        let Some(mut actor) = self.actors.get(id).cloned() else {
            godot_warn!("GdMoviePlayer: move '{}', actor not on stage", id);
            return WaitStep::None;
        };
        if !actor.is_instance_valid() {
            return WaitStep::None;
        }
        let Some(mut map) = self.current_map() else { return WaitStep::None };
        let cs = self.map_cell_size(&mut map);
        let path = if map.has_method("find_path") {
            let from = self.node_cell(&mut actor, cs);
            let path_res: Variant =
                map.call("find_path", &[from.to_variant(), cell.to_variant()]);
            let p = path_res.to::<PackedVector2Array>();
            if p.is_empty() { None } else { Some(p) }
        } else {
            Some(self.build_walk_path(&mut actor, cell, cs))
        };
        match path {
            Some(p) if !p.is_empty() => {
                actor.call("set_grid_path", &[p.to_variant()]);
                WaitStep::GridDone { actor, timeout: GRID_TIMEOUT_SECS }
            }
            _ => {
                // 不可达：直接落格（保持演出连续）
                let mut n2d = actor;
                node_set_pos(&mut n2d, cell_center(cell, cs));
                WaitStep::None
            }
        }
    }

    /// 直线逐格路径（先横后纵）：不依赖地图寻路协议，任意地图可用。
    /// 用于 GdDigLayer 等无 find_path 地图的 move / 进出场走位
    fn build_walk_path(&self, actor: &mut Gd<Node>, cell: Vector2i, cs: f32) -> PackedVector2Array {
        let from = self.node_cell(actor, cs);
        let mut cells: Vec<Vector2i> = Vec::new();
        let dx = (cell.x - from.x).signum();
        let mut x = from.x;
        while x != cell.x {
            x += dx;
            cells.push(Vector2i::new(x, from.y));
        }
        let dy = (cell.y - from.y).signum();
        let mut y = from.y;
        while y != cell.y {
            y += dy;
            cells.push(Vector2i::new(x, y));
        }
        cells
            .into_iter()
            .map(|c| cell_center(c, cs))
            .collect::<PackedVector2Array>()
    }

    fn exec_camera(&mut self, cam: &CamAction) -> WaitStep {
        let Some(mut camera) = self.find_camera() else {
            godot_warn!("GdMoviePlayer: camera skipped, ViewCamera not found");
            return WaitStep::None;
        };
        match cam {
            CamAction::Follow { target } => {
                if let Some(node) = self.resolve_target(target) {
                    camera.call("follow", &[node.to_variant(), true.to_variant(), false.to_variant()]);
                }
                WaitStep::None
            }
            CamAction::Unfollow => {
                camera.call("unfollow", &[false.to_variant()]);
                WaitStep::None
            }
            CamAction::Pan { pos, dur } => {
                camera.call("pan", &[pos.to_variant()]);
                WaitStep::Time(*dur)
            }
            CamAction::Focus { target, dur } => {
                if let Some(node) = self.resolve_target(target) {
                    camera.call("focus", &[node.to_variant(), (*dur as f32).to_variant()]);
                    WaitStep::Time(*dur)
                } else {
                    WaitStep::None
                }
            }
            CamAction::Unfocus => {
                camera.call("unfocus", &[]);
                WaitStep::None
            }
            CamAction::Shake { amount } => {
                camera.call("shake", &[amount.to_variant()]);
                WaitStep::Time(0.4)
            }
            CamAction::Snap { pos } => {
                camera.call("snap_to", &[pos.to_variant(), 0.0f32.to_variant()]);
                WaitStep::None
            }
            CamAction::Zoom { step } => {
                camera.call("start_zoom", &[(*step).to_variant(), (-1.0f32).to_variant(), (-1.0f32).to_variant()]);
                WaitStep::None
            }
            CamAction::ResetZoom => {
                camera.call("reset_zoom", &[]);
                WaitStep::None
            }
            CamAction::Blur { val } => {
                camera.call("blur", &[(*val).to_variant()]);
                WaitStep::None
            }
        }
    }

    /// 对话框台词：台词 UI 宿主（show_movie_line 契约）优先，无宿主则用
    /// rust 默认对话框（dialog/movie_ui.rs GdMovieDialogBox，懒创建）。
    /// dur 有值定时继续，否则等玩家点击（movie_line_done 置位）
    fn exec_say(&mut self, id: &str, text: &str, dur: Option<f64>) -> WaitStep {
        let Some(mut target) = self.resolve_say_target() else {
            godot_warn!("GdMoviePlayer: say skipped, no dialog host (text: {})", text);
            return WaitStep::Time(1.0);
        };
        let mut line = VarDictionary::new();
        line.set("name", id);
        if let Some(def) = self.actor_defs.get(id) {
            if !def.display_name.is_empty() {
                line.set("display_name", def.display_name.as_str());
            }
        }
        line.set("text", text);
        target.call("show_movie_line", &[line.to_variant()]);
        match dur {
            Some(d) => WaitStep::Time(d),
            None => WaitStep::LineDone { target, dur_max: 0.0 },
        }
    }

    /// 台词 UI 目标解析：配置宿主实现了 show_movie_line 则用宿主
    /// （demo 自定义样式），否则懒创建 rust 默认对话框
    fn resolve_say_target(&mut self) -> Option<Gd<Node>> {
        if let Some(host) = self.dialog_box.clone() {
            if host.is_instance_valid() && host.has_method("show_movie_line") {
                return Some(host);
            }
        }
        // 默认对话框：失效重建（跨场景/重播自愈）
        if self
            .default_box
            .as_ref()
            .map_or(true, |n| !n.is_instance_valid())
        {
            let mut bx = GdMovieDialogBox::new_alloc();
            bx.set_name("__MovieDialogBox");
            self.base_mut().add_child(&bx.clone());
            self.default_box = Some(bx.clone().upcast::<Node>());
        }
        self.default_box.clone()
    }

    /// 气泡台词：统一走 show_bubble_on（宿主 show_bubble 契约优先，
    /// 否则挂 rust 默认头顶气泡），等秒数
    fn exec_bubble(&mut self, id: &str, secs: f64, text: &str) -> WaitStep {
        match self.actors.get(id).cloned() {
            Some(mut actor) if actor.is_instance_valid() => {
                show_bubble_on(&mut actor, text, secs);
            }
            _ => {
                godot_warn!("GdMoviePlayer: bubble '{}', actor not on stage", id);
            }
        }
        WaitStep::Time(secs)
    }

    // === 收尾 ===

    /// 演出自然结束：解锁玩家、关对话框、按配置清角色、发信号
    fn finish(&mut self) {
        if !self.playing {
            return;
        }
        self.playing = false;
        self.coroutine = None;
        self.lock_player(false);
        self.close_dialog_box();
        self.restore_npcs();
        if self.auto_free_actors {
            self.despawn_all_actors();
        } else {
            // 保留在场：解除演出契约，恢复日程/对话/寒暄自治
            // （否则宿主自治系统被 set_movie_actor(true) 永久禁用）
            self.resume_actors();
        }
        self.base_mut().emit_signal("s_movie_finished", &[]);
    }

    /// 保留在场的角色解除演出模式（set_movie_actor(false)），宿主自治恢复
    fn resume_actors(&mut self) {
        for node in self.actors.values_mut() {
            if node.is_instance_valid() && node.has_method("set_movie_actor") {
                node.call("set_movie_actor", &[false.to_variant()]);
            }
        }
    }

    fn despawn_all_actors(&mut self) {
        for (_, mut node) in self.actors.drain() {
            if node.is_instance_valid() {
                node.call_deferred("queue_free", &[]);
            }
        }
        self.owned_actors.clear();
    }

    // === 常驻 NPC 抑制（演出 dedup） ===

    /// 演出期隐藏当前地图上的常驻 NPC：地图场景内嵌的 NPC 与剧本 actor
    /// （同一角色再实例化一份）会同屏重复——演出期间把常驻 NPC 藏起来，
    /// 结束/stop 时 restore_npcs 恢复。
    /// 识别契约：节点带 `npc_id` 属性（NPC 基类鸭子识别，框架不感知
    /// demo 类名）；演出 actor 本尊及其子树豁免（防同图重开二段抑制
    /// 把已出场的 actor 也藏掉）。
    fn suppress_map_npcs(&mut self) {
        let Some(map) = self.current_map() else { return };
        let before = self.suppressed_npcs.len();
        let mut stack: Vec<Gd<Node>> = vec![map];
        while let Some(node) = stack.pop() {
            for child in node.get_children().iter_shared() {
                let mut c = child;
                stack.push(c.clone());
                // hide 守卫：npc_id 属性不止 NPC 本尊持有（子组件
                // GdNpcSchedule 同样镜像了 npc_id，纯逻辑 Node 无 hide）
                if c.get("npc_id").get_type() == VariantType::NIL
                    || !c.has_method("hide")
                    || self.is_actor_tree(&c)
                    || self
                        .suppressed_npcs
                        .iter()
                        .any(|n| n.instance_id() == c.instance_id())
                {
                    continue;
                }
                c.call("hide", &[]);
                self.suppressed_npcs.push(c);
            }
        }
        let hidden = self.suppressed_npcs.len() - before;
        if hidden > 0 {
            godot_print!("[GdMoviePlayer] 演出期隐藏常驻 NPC x{}", hidden);
        }
    }

    /// 恢复演出期隐藏的常驻 NPC（结束/stop/exit_tree；实例已随旧地图
    /// 释放的条目自然跳过——回迁重载的地图上 NPC 本来就是新实例）
    fn restore_npcs(&mut self) {
        self.pending_suppress = false;
        for mut npc in self.suppressed_npcs.drain(..) {
            if npc.is_instance_valid() {
                npc.call("show", &[]);
            }
        }
    }

    /// 节点是否属于本演出的 actor（本尊或其子树）
    fn is_actor_tree(&self, node: &Gd<Node>) -> bool {
        self.actors.values().any(|a| {
            a.is_instance_valid()
                && (a.instance_id() == node.instance_id() || a.is_ancestor_of(node))
        })
    }

    fn close_dialog_box(&mut self) {
        // 台词 UI 宿主 + rust 默认对话框都收（哪个在场关哪个）
        if let Some(mut control) = self.dialog_box.clone() {
            if control.is_instance_valid() && control.has_method("close_movie_box") {
                control.call("close_movie_box", &[]);
            }
        }
        if let Some(mut bx) = self.default_box.clone() {
            if bx.is_instance_valid() {
                bx.call("close_movie_box", &[]);
            }
        }
    }

    fn lock_player(&mut self, lock: bool) {
        let Some(mut player) = self.player.clone() else { return };
        if player.is_instance_valid() && player.has_method("set_paused") {
            player.call("set_paused", &[lock.to_variant()]);
        }
    }

    // === 查找/工具 ===

    fn resolve_label(&self, label: &str) -> usize {
        match self.script.as_ref().and_then(|s| s.labels.get(label)) {
            Some(&pc) => pc,
            None => {
                godot_warn!("GdMoviePlayer: jump label '{}' not found, treat as end", label);
                usize::MAX
            }
        }
    }

    fn has_flag(&mut self, flag: &str) -> bool {
        let Some(mut control) = self.dialog_box.clone() else { return false };
        if !control.is_instance_valid() || !control.has_method("has_flag") {
            return false;
        }
        control.call("has_flag", &[flag.to_variant()]).to()
    }

    fn set_flag(&mut self, flag: &str) {
        let Some(mut control) = self.dialog_box.clone() else { return };
        if control.is_instance_valid() && control.has_method("set_flag") {
            control.call("set_flag", &[flag.to_variant()]);
        } else {
            godot_warn!("GdMoviePlayer: set_flag '{}' skipped, no control", flag);
        }
    }

    /// camera/actor 的目标解析：`player` → player 节点；其余 → 在场 actor
    fn resolve_target(&self, target: &str) -> Option<Gd<Node>> {
        if target == "player" {
            return self
                .player
                .clone()
                .filter(|p| p.is_instance_valid());
        }
        self.actors
            .get(target)
            .cloned()
            .filter(|n| n.is_instance_valid())
    }

    /// MapManager：组件表标准入口（demo 范式；未注册返回 None）
    fn find_map_manager(&mut self) -> Option<Gd<Node>> {
        let mut scene_root = self.find_scene_root()?;
        let mgr = scene_root.call("get_component", &["MapManager".to_variant()]);
        if mgr.get_type() != VariantType::OBJECT {
            return None;
        }
        mgr.try_to::<Gd<Node>>().ok()
    }

    /// ViewCamera：组件表（GdSceneRoot ready 时自动注册）
    fn find_camera(&mut self) -> Option<Gd<Node>> {
        let mut scene_root = self.find_scene_root()?;
        let cam = scene_root.call("get_component", &["ViewCamera".to_variant()]);
        if cam.get_type() != VariantType::OBJECT {
            return None;
        }
        cam.try_to::<Gd<Node>>().ok()
    }

    /// 场景根：GDCORE 全局节点表（manager_id 默认 "default"）
    fn find_scene_root(&mut self) -> Option<Gd<Node>> {
        let mut gdcore = Engine::singleton().get_singleton("GDCORE")?;
        let root_var = gdcore.call(
            &StringName::from("get_global_node"),
            &[GString::from("default").to_variant()],
        );
        if root_var.get_type() != VariantType::OBJECT {
            return None;
        }
        root_var.try_to::<Gd<Node>>().ok()
    }

    fn current_map(&mut self) -> Option<Gd<Node>> {
        let mut mgr = self.find_map_manager()?;
        let map: Variant = mgr.call("get_current_map", &[]);
        if map.get_type() != VariantType::OBJECT {
            return None;
        }
        map.try_to::<Gd<Node>>().ok()
    }

    fn map_cell_size(&self, map: &mut Gd<Node>) -> f32 {
        if map.has_method("get_cell_size_px") {
            let cs: i64 = map.call("get_cell_size_px", &[]).to();
            return cs as f32;
        }
        32.0
    }

    /// 节点所在格（世界坐标 → 格坐标）
    fn node_cell(&self, node: &mut Gd<Node>, cs: f32) -> Vector2i {
        let pos: Vector2 = node.call("get_global_position", &[]).to();
        Vector2i::new((pos.x / cs) as i32, (pos.y / cs) as i32)
    }

    fn resolve_refs(&mut self) {
        self.dialog_box = self
            .try_node(&self.dialog_box_path)
            .filter(|n| n.is_instance_valid());
        self.player = self.try_node(&self.player_path).filter(|n| n.is_instance_valid());
    }

    fn try_node(&self, path: &NodePath) -> Option<Gd<Node>> {
        if path.is_empty() {
            return None;
        }
        self.base().get_node_or_null(path)
    }

    /// config `movie.actors`：{id: {scene: tscn路径, display_name: 展示名}}
    fn load_actor_defs(&mut self) {
        let Some(mut config) = ensure_config_manager() else {
            godot_warn!("GdMoviePlayer: GdConfigManager unavailable, actors empty");
            return;
        };
        let movie: VarDictionary = config
            .call(
                "get_config",
                &["movie".to_variant(), VarDictionary::new().to_variant()],
            )
            .to();
        if movie.is_empty() {
            return;
        }
        let actors: VarDictionary = movie
            .get("actors")
            .and_then(|v| v.try_to::<VarDictionary>().ok())
            .unwrap_or_default();
        let keys = actors.keys_array();
        for i in 0..keys.len() {
            let key = keys.at(i);
            let id: GString = key.to();
            let entry: VarDictionary = actors
                .get(&key)
                .and_then(|v| v.try_to::<VarDictionary>().ok())
                .unwrap_or_default();
            let scene_path: GString = entry
                .get("scene")
                .and_then(|v| v.try_to::<GString>().ok())
                .unwrap_or_default();
            let display_name: GString = entry
                .get("display_name")
                .and_then(|v| v.try_to::<GString>().ok())
                .unwrap_or_default();
            self.actor_defs.insert(
                id.to_string(),
                ActorDef {
                    scene_path: scene_path.to_string(),
                    display_name: display_name.to_string(),
                },
            );
        }
    }

    fn load_script(&mut self, path: &GString) {
        let file = godot::classes::FileAccess::open(
            &path.to_string(),
            godot::classes::file_access::ModeFlags::READ,
        );
        let Some(file) = file else {
            godot_warn!("GdMoviePlayer: script '{}' not found", path);
            return;
        };
        let text = file.get_as_text();
        match parse(&text.to_string()) {
            Ok(script) => self.script = Some(script),
            Err(e) => godot_warn!("GdMoviePlayer: parse '{}' failed: {}", path, e),
        }
    }
}

fn now_ms() -> i64 {
    Time::singleton().get_ticks_msec() as i64
}

/// 格心世界坐标
fn cell_center(cell: Vector2i, cs: f32) -> Vector2 {
    Vector2::new((cell.x as f32 + 0.5) * cs, (cell.y as f32 + 0.5) * cs)
}

/// 鸭子设位置（node 可能是任意 Node2D 宿主，不经静态类型）
fn node_set_pos(node: &mut Gd<Node>, pos: Vector2) {
    node.call("set_position", &[pos.to_variant()]);
}
