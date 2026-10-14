// GdNpcChat - NPC 相遇闲聊（世界不围着玩家转）
//
// 挂在角色（GdRoleMover 子类）的子节点上，与另一 NPC（同图、均在 npc_schedule
// 分组）相遇时有概率触发一段预制气泡对聊：双方暂停、逐句头顶气泡、播完恢复
// 并进入冷却。玩家靠近即"偷听"，不参与照样发生。
//
// 鸭子类型：宿主需 set_paused/show_bubble/hide_bubble/is_paused/
// is_grid_path_active（demo 侧 XiuNpcBase 即 GdRoleMover）。同伴按
// npc_schedule 分组成员 meta 的 npc_id 匹配。
//
// 防重入：触发方把双方 set_paused(true)，对方组件扫描时看到 paused 直接跳过——
// 同一对 NPC 即使互相都配了 chat 也只会触发一场。

use rand::Rng;

use godot::prelude::*;
use godot::builtin::{GString, PackedStringArray, Vector2};
use godot::classes::{INode, Node, Node2D};

use super::npc_schedule::NPC_SCHEDULE_GROUP;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdNpcChat {
    /// 是否启用
    #[export]
    enable: bool,

    /// 闲聊对象 npc_id（对方 GdNpcSchedule.npc_id，默认取其 role_name）
    #[export]
    partner_id: GString,

    /// 我方台词（逐句）
    #[export]
    my_lines: PackedStringArray,

    /// 对方台词（逐句，与 my_lines 按索引配对）
    #[export]
    partner_lines: PackedStringArray,

    /// 触发距离（像素）
    #[export]
    chat_radius: f64,

    /// 每次判定的触发概率
    #[export]
    chance: f64,

    /// 冷却（秒，触发后附加 0~30s 随机）
    #[export]
    cooldown: f64,

    /// 每句气泡持续时长（秒）
    #[export]
    line_secs: f64,

    /// 进场初始冷却（秒）：错开多 NPC 触发时机，避免开局齐聊
    #[export]
    initial_cooldown: f64,

    // ---- 运行时状态 ----
    cooldown_left: f64,
    chatting: bool,
    line_idx: i64,
    line_timer: f64,
    partner: Option<Gd<Node>>,
    acc: f64,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdNpcChat {
    fn init(base: Base<Node>) -> Self {
        Self {
            enable: true,
            partner_id: GString::new(),
            my_lines: PackedStringArray::new(),
            partner_lines: PackedStringArray::new(),
            chat_radius: 160.0,
            chance: 0.5,
            cooldown: 120.0,
            line_secs: 2.5,
            initial_cooldown: 10.0,
            cooldown_left: 20.0,
            chatting: false,
            line_idx: 0,
            line_timer: 0.0,
            partner: None,
            acc: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        // 进场初始冷却：错开多 NPC 首次闲聊时机
        self.cooldown_left = self.initial_cooldown;
    }

    fn process(&mut self, delta: f64) {
        if !self.enable {
            return;
        }        // 0.5s 节流
        self.acc += delta;
        if self.acc < 0.5 {
            self.tick_chat(delta);
            return;
        }
        self.acc = 0.0;
        self.try_start_chat();
        self.tick_chat(delta);
    }
}

#[godot_api]
impl GdNpcChat {
    /// 是否正在闲聊（对话触发器 condition_fn 可用来避开闲聊中的 NPC）
    #[func]
    fn is_chatting(&self) -> bool {
        self.chatting
    }

    /// 剩余冷却秒数（0 = 可触发）
    #[func]
    fn get_cooldown_left(&self) -> f64 {
        self.cooldown_left
    }
}

impl GdNpcChat {
    fn host(&self) -> Option<Gd<Node>> {
        self.base().get_parent()
    }

    fn host_n2d(&self) -> Option<Gd<Node2D>> {
        self.host().and_then(|h| h.try_cast::<Node2D>().ok())
    }

    // ---- 触发扫描 ----

    fn try_start_chat(&mut self) {
        if self.chatting {
            return;
        }
        if self.cooldown_left > 0.0 {
            self.cooldown_left -= 0.5;
            return;
        }
        if self.my_lines.is_empty() || self.partner_id.is_empty() {
            return;
        }
        let Some(mut host) = self.host() else { return };
        // 自方条件：不在对话/导航中
        if host.call("is_paused", &[]).to::<bool>()
            || host.call("is_grid_path_active", &[]).to::<bool>()
        {
            return;
        }
        let Some(mut partner) = self.find_partner(&host) else { return };
        // 对方条件：不在对话/导航中（paused 同时天然防双触发）
        if partner.call("is_paused", &[]).to::<bool>()
            || partner.call("is_grid_path_active", &[]).to::<bool>()
        {
            return;
        }
        // 距离判定
        let (Some(mut a), Some(mut b)) = (
            self.host_n2d(),
            partner.clone().try_cast::<Node2D>().ok(),
        ) else {
            return;
        };
        if a.get_global_position().distance_to(b.get_global_position()) > self.chat_radius as f32 {
            return;
        }
        // 概率触发
        let mut rng = rand::thread_rng();
        if !rng.gen_bool(self.chance.clamp(0.0, 1.0)) {
            return;
        }
        // 开聊：双方暂停 + 第一句（鸭子调用守卫：宿主缺方法时不 panic）
        self.partner = Some(partner.clone());
        self.chatting = true;
        self.line_idx = 0;
        self.line_timer = self.line_secs;
        if host.has_method("set_paused") {
            host.call("set_paused", &[true.to_variant()]);
        }
        if partner.has_method("set_paused") {
            partner.call("set_paused", &[true.to_variant()]);
        }
        self.show_line_pair();
    }

    /// 在 npc_schedule 分组里找 npc_id 匹配的同伴（返回同伴的宿主）
    fn find_partner(&self, host: &Gd<Node>) -> Option<Gd<Node>> {
        let tree = self.base().get_tree();
        let members = tree.get_nodes_in_group(NPC_SCHEDULE_GROUP);
        let key = self.partner_id.to_string();
        // 分组成员即各 NPC 的 GdNpcSchedule 组件，其父节点才是宿主
        for node in members.iter_shared() {
            if node.instance_id() == host.instance_id() {
                continue; // 自己的 schedule 组件
            }
            let Some(partner_host) = node.get_parent() else { continue };
            if partner_host.instance_id() == host.instance_id() {
                continue;
            }
            // 匹配 meta npc_id（GdNpcSchedule.ready 时写入）
            let id = node.get_meta(&StringName::from("npc_id")).to::<GString>();
            if id.to_string() == key {
                return Some(partner_host);
            }
        }
        None
    }

    // ---- 聊天推进 ----

    fn tick_chat(&mut self, delta: f64) {
        if !self.chatting {
            return;
        }
        // 同伴中途消失（切图/释放）：安全终止
        let partner_ok = self
            .partner
            .as_ref()
            .is_some_and(|p| p.is_instance_valid());
        if !partner_ok {
            self.end_chat();
            return;
        }
        self.line_timer -= delta;
        if self.line_timer > 0.0 {
            return;
        }
        self.line_idx += 1;
        if self.line_idx as usize >= self.my_lines.len() {
            self.end_chat();
            return;
        }
        self.show_line_pair();
    }

    fn show_line_pair(&mut self) {
        let i = self.line_idx as usize;
        let mut rng = rand::thread_rng();
        let secs = self.line_secs + rng.gen_range(0.0..0.6);
        if let Some(mut host) = self.host() {
            if i < self.my_lines.len() && host.has_method("show_bubble") {
                let line = self.my_lines.get(i).unwrap_or_default();
                host.call("show_bubble", &[line.to_variant(), (secs).to_variant()]);
            }
        }
        if let Some(ref partner) = self.partner {
            let mut p = partner.clone();
            if i < self.partner_lines.len() && p.has_method("show_bubble") {
                let line = self.partner_lines.get(i).unwrap_or_default();
                p.call("show_bubble", &[line.to_variant(), (secs).to_variant()]);
            }
        }
        self.line_timer = secs;
    }

    fn end_chat(&mut self) {
        if let Some(mut host) = self.host() {
            if host.has_method("hide_bubble") {
                host.call("hide_bubble", &[]);
            }
            if host.has_method("set_paused") {
                host.call("set_paused", &[false.to_variant()]);
            }
        }
        if let Some(ref partner) = self.partner {
            if partner.is_instance_valid() {
                let mut p = partner.clone();
                if p.has_method("hide_bubble") {
                    p.call("hide_bubble", &[]);
                }
                if p.has_method("set_paused") {
                    p.call("set_paused", &[false.to_variant()]);
                }
            }
        }
        self.partner = None;
        self.chatting = false;
        self.line_idx = 0;
        let mut rng = rand::thread_rng();
        self.cooldown_left = self.cooldown + rng.gen_range(0.0..30.0);
    }
}
