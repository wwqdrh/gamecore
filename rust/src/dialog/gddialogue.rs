// GdDialogue - 对话控制节点
//
// 继承 Node，管理 Timeline 和对话推进
// 参考 C++ 版 dialogue.cpp 实现，暴露给 GDScript 的接口与 C++ 版一致
//
// 属性：dialogue_control_path, timeline, click_next, skip, skip_can_next, skip_time, handle_fn
// 方法：next, exec_response, is_registered_role, register_role_node,
//       register_role_meta, get_role_display_name, get_role_portrait,
//       get_role_pos, initial, goto_stage, all_stages, has_next, stage_index
// 信号：s_finished

use godot::prelude::*;
use godot::builtin::{VarDictionary, Array, Variant, PackedStringArray, VarArray};
use godot::classes::{INode, Node};
use godot::obj::WithBaseField;
use gamedialog::{Timeline, SceneManager};
use std::collections::HashMap;

/// 角色元数据（Speaker 注册，UI 展示用；display_name 为空时 UI 回退角色名）
struct RoleMeta {
    display_name: String,
    portrait: String,
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdDialogue {
    base: Base<Node>,

    /// 对话框控制节点的路径
    dialogue_control_path: NodePath,
    /// Timeline 文件路径，设置时自动加载
    timeline_path: GString,
    /// 是否支持点击下一条
    click_next: bool,
    /// 是否跳过
    skip: bool,
    /// 跳过是否可继续下一条
    skip_can_next: bool,
    /// 跳过间隔时间
    skip_time: f64,
    /// 处理对话行的回调函数名
    handle_fn: GString,

    // 内部状态
    /// Timeline 实例
    timeline: Option<Timeline>,
    /// 场景管理器
    scene_manager: SceneManager,
    /// 角色注册表 (role -> NodePath)
    role_target: HashMap<String, NodePath>,
    /// 角色元数据 (role -> 展示名/立绘)，来自 GdRoleSpeaker，随行下发
    role_meta: HashMap<String, RoleMeta>,
    /// 是否在等待 response 选择
    in_response: bool,
    /// 对话是否正在播放中（第一条 next 后为 true，timeline 播完为 false）
    playing: bool,
    /// 对话控制节点的缓存引用
    dialogue_control: Option<Gd<Node>>,
    /// precheck 安装时 control 尚未就绪的标记（next() 时重试安装）
    precheck_stale: bool,
}

#[godot_api]
impl INode for GdDialogue {
    fn init(base: Base<Node>) -> Self {
        GdDialogue {
            base,
            dialogue_control_path: NodePath::default(),
            timeline_path: GString::new(),
            click_next: false,
            skip: false,
            skip_can_next: false,
            skip_time: 0.2,
            handle_fn: GString::from("handle_line"),
            timeline: None,
            scene_manager: SceneManager::new(),
            role_target: HashMap::new(),
            role_meta: HashMap::new(),
            in_response: false,
            playing: false,
            dialogue_control: None,
            precheck_stale: false,
        }
    }

    fn ready(&mut self) {
        self.resolve_dialogue_control();
        if !self.timeline_path.is_empty() {
            self.load_timeline_from_path(&self.timeline_path.clone());
        }
    }
}

#[godot_api]
impl GdDialogue {
    #[signal]
    fn s_finished();

    // === 导出属性 ===

    #[func]
    fn set_dialogue_control_path(&mut self, path: NodePath) {
        self.dialogue_control_path = path;
        self.resolve_dialogue_control();
    }

    #[func]
    fn get_dialogue_control_path(&self) -> NodePath {
        self.dialogue_control_path.clone()
    }

    #[func]
    pub fn set_timeline_path(&mut self, path: GString) {
        self.timeline_path = path.clone();
        if !path.is_empty() {
            self.load_timeline_from_path(&path);
        }
    }

    #[func]
    fn get_timeline_path(&self) -> GString {
        self.timeline_path.clone()
    }

    #[func]
    fn set_click_next(&mut self, val: bool) {
        self.click_next = val;
    }

    #[func]
    fn get_click_next(&self) -> bool {
        self.click_next
    }

    #[func]
    fn set_skip(&mut self, val: bool) {
        self.skip = val;
        if val {
            self.skip_can_next = true;
        }
    }

    #[func]
    fn get_skip(&self) -> bool {
        self.skip
    }

    #[func]
    fn set_skip_can_next(&mut self, val: bool) {
        self.skip_can_next = val;
    }

    #[func]
    fn get_skip_can_next(&self) -> bool {
        self.skip_can_next
    }

    #[func]
    fn set_skip_time(&mut self, val: f64) {
        self.skip_time = val;
    }

    #[func]
    fn get_skip_time(&self) -> f64 {
        self.skip_time
    }

    #[func]
    fn set_handle_fn(&mut self, val: GString) {
        self.handle_fn = val;
    }

    #[func]
    fn get_handle_fn(&self) -> GString {
        self.handle_fn.clone()
    }

    // === 公开方法 ===

    /// 检查角色是否已注册且节点有效
    #[func]
    pub fn is_registered_role(&self, role: GString) -> bool {
        let role_str = role.to_string();
        if let Some(path) = self.role_target.get(&role_str) {
            if self.try_get_node(path).is_some() {
                return true;
            }
        }
        false
    }

    /// 注册角色对应的节点
    #[func]
    pub fn register_role_node(&mut self, role: GString, target: Gd<Node>) {
        let role_str = role.to_string();
        let path = target.get_path();
        self.role_target.insert(role_str, path);
    }

    /// 注册角色元数据（展示名/立绘；触发器启动对话时从 GdRoleSpeaker 自动注册）
    #[func]
    pub fn register_role_meta(&mut self, role: GString, display_name: GString, portrait: GString) {
        self.role_meta.insert(role.to_string(), RoleMeta {
            display_name: display_name.to_string(),
            portrait: portrait.to_string(),
        });
    }

    /// 查询角色展示名（未注册或为空返回 ""）
    #[func]
    pub fn get_role_display_name(&self, role: GString) -> GString {
        self.role_meta.get(&role.to_string())
            .map(|m| GString::from(m.display_name.as_str()))
            .unwrap_or_default()
    }

    /// 查询角色立绘贴图路径（未注册或未配置返回 ""）
    #[func]
    pub fn get_role_portrait(&self, role: GString) -> GString {
        self.role_meta.get(&role.to_string())
            .map(|m| GString::from(m.portrait.as_str()))
            .unwrap_or_default()
    }

    /// 获取角色节点的全局位置
    #[func]
    pub fn get_role_pos(&self, role: GString) -> Variant {
        let role_str = role.to_string();
        if let Some(path) = self.role_target.get(&role_str) {
            if let Some(mut node) = self.try_get_node(path) {
                return node.call("get_global_position", &[]);
            }
        }
        Variant::nil()
    }

    /// 推进对话到下一条，可选跳转到指定 label
    #[func]
    pub fn next(&mut self, label: GString) {
        if self.in_response {
            return;
        }

        let label_str = label.to_string();
        if !label_str.is_empty() {
            self.goto_stage(label);
        }

        let mut control = match self.get_dialogue_control() {
            Some(c) => c,
            None => return,
        };

        let timeline = match &mut self.timeline {
            Some(t) => t,
            None => return,
        };

        if !timeline.has_next() {
            self.playing = false;
            self.base_mut().emit_signal("s_finished", &[]);
            return;
        }

        let word = match timeline.next() {
            Some(w) => w,
            None => return,
        };
        self.playing = true;

        let mut dia_line = VarDictionary::new();
        let speaker_name = word.get_name();
        dia_line.set("name", speaker_name.clone());
        dia_line.set("text", word.get_text());
        dia_line.set("stage", word.get_stage());
        // 角色元数据随行下发（展示名/立绘；未配置的键不注入，UI 回退角色名/隐藏立绘）
        if let Some(meta) = self.role_meta.get(&speaker_name.to_string()) {
            if !meta.display_name.is_empty() {
                dia_line.set("display_name", meta.display_name.as_str());
            }
            if !meta.portrait.is_empty() {
                dia_line.set("portrait", meta.portrait.as_str());
            }
        }

        // 处理 functions
        for expr in word.get_functions() {
            let parts: Vec<&str> = expr.splitn(2, ':').collect();
            if parts.is_empty() {
                continue;
            }
            if parts.len() == 1 {
                control.call(parts[0], &[]);
            } else if parts.len() == 2 {
                let args: VarArray = parts[1].split(',')
                    .map(|s| s.to_variant())
                    .collect();
                control.callv(parts[0], &args);
            }
        }

        // 处理 response
        let mut resp = Array::<VarDictionary>::new();
        for (text, target) in word.get_responses() {
            let mut respitem = VarDictionary::new();
            respitem.set("text", text.as_str());
            respitem.set("fn", target.as_str());
            respitem.set("stage", word.get_stage());
            resp.push(&respitem);
        }
        if resp.len() > 0 {
            self.in_response = true;
        }
        dia_line.set("response", &resp);

        let handle_fn_name = self.handle_fn.to_string();
        let has_method: bool = control.call("has_method", &[handle_fn_name.to_variant()]).to();
        if has_method {
            control.call(&handle_fn_name, &[dia_line.to_variant()]);
        }
    }

    /// 强制停止播放（触发器 cancel_dialog / 外部中止对话时用）：
    /// 复位播放与选项等待状态，timeline 跳到结尾（stage 进度清干净）
    #[func]
    pub fn stop(&mut self) {
        self.playing = false;
        self.in_response = false;
        if let Some(ref mut timeline) = self.timeline {
            timeline.goto_end();
        }
    }

    /// 执行选择分支的响应动作
    #[func]
    pub fn exec_response(&mut self, data: VarDictionary, role: GString) {
        let expr: GString = match data.get("fn") {
            Some(v) => v.to::<GString>(),
            None => return,
        };
        if expr.is_empty() {
            return;
        }

        let current_stage: GString = match data.get("stage") {
            Some(v) => v.to::<GString>(),
            None => GString::new(),
        };

        self.in_response = false;

        let expr_str = expr.to_string();
        let exprparts: Vec<&str> = expr_str.split(';').collect();

        // 收集需要执行的操作，避免 timeline 借用与 self 的其他方法冲突
        enum PendingOp {
            GotoStage(GString),
            Continue,
            End,
            CallControl { fn_name: String, args_str: Option<String> },
        }
        let mut pending_ops: Vec<PendingOp> = Vec::new();

        for item in exprparts {
            if item.is_empty() {
                continue;
            }

            if item.starts_with("goto") {
                let parts: Vec<&str> = item.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let target = parts[1];
                    if target.starts_with('#') {
                        let sub_label = target.trim_start_matches('#');
                        let full_label = format!("{}/{}", current_stage, sub_label);
                        pending_ops.push(PendingOp::GotoStage(GString::from(&full_label)));
                    } else {
                        pending_ops.push(PendingOp::GotoStage(GString::from(target)));
                    }
                    pending_ops.push(PendingOp::Continue);
                }
            } else if item == "continue" {
                pending_ops.push(PendingOp::Continue);
            } else if item == "end" {
                pending_ops.push(PendingOp::End);
                pending_ops.push(PendingOp::Continue);
            } else {
                let parts: Vec<&str> = item.splitn(2, ':').collect();
                let fn_name = parts[0].to_string();
                let args_str = if parts.len() == 2 { Some(parts[1].to_string()) } else { None };
                pending_ops.push(PendingOp::CallControl { fn_name, args_str });
            }
        }

        // 先执行 timeline 的 goto_end（如果需要）
        let needs_goto_end = pending_ops.iter().any(|op| matches!(op, PendingOp::End));
        if needs_goto_end {
            if let Some(ref mut timeline) = self.timeline {
                timeline.goto_end();
            }
        }

        // 执行收集到的操作
        let mut control = self.get_dialogue_control();
        for op in pending_ops {
            match op {
                PendingOp::GotoStage(label) => {
                    self.goto_stage(label);
                }
                PendingOp::Continue => {
                    self.next(GString::new());
                }
                PendingOp::End => {
                    // 已在上面处理
                }
                PendingOp::CallControl { fn_name, args_str } => {
                    let mut handled = false;
                    if let Some(ref mut control_node) = control {
                        let has_method: bool = control_node.call("has_method", &[fn_name.to_variant()]).to();
                        if has_method {
                            if let Some(ref args) = args_str {
                                let var_args: VarArray = args.split(',')
                                    .map(|s| s.to_variant())
                                    .collect();
                                control_node.callv(&fn_name, &var_args);
                            } else {
                                control_node.call(&fn_name, &[]);
                            }
                            handled = true;
                        }
                    }
                    // control 上没有该方法，尝试 role 节点
                    if !handled && self.is_registered_role(role.clone()) {
                        let role_str = role.to_string();
                        if let Some(path) = self.role_target.get(&role_str) {
                            if let Some(mut node) = self.try_get_node(path) {
                                let has_fn: bool = node.call("has_method", &[fn_name.to_variant()]).to();
                                if has_fn {
                                    if let Some(ref args) = args_str {
                                        let var_args: VarArray = args.split(',')
                                            .map(|s| s.to_variant())
                                            .collect();
                                        node.callv(&fn_name, &var_args);
                                    } else {
                                        node.call(&fn_name, &[]);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// 用文本数据初始化 Timeline
    #[func]
    pub fn initial(&mut self, data: GString) {
        let data_str = data.to_string();
        let mut timeline = Timeline::new(&data_str);
        self.timeline = Some(timeline);
        self.install_precheck();
    }

    /// 从 entry_stage 启动对话：flag 条件满足则直达，否则回退到第一个
    /// 满足条件的 stage（对话进度记录：已看过的段落经 flag 门控自动跳过）；
    /// entry 为空或无任何 stage 满足时从头开始/直接结束
    #[func]
    pub fn start_from(&mut self, entry: GString) {
        if self.in_response {
            return;
        }
        if self.precheck_stale {
            self.install_precheck();
        }
        let entry_ok = !entry.is_empty()
            && self
                .timeline
                .as_ref()
                .is_some_and(|t| t.check_stage_flag(&entry.to_string()));
        if entry_ok {
            self.goto_stage(entry);
        } else if let Some(ref mut timeline) = self.timeline {
            // 回到开头：next() 内部的 get_first_flag 会选出第一个满足
            // flag 条件的 stage；一个都不满足则直接到结尾
            timeline.goto_begin();
        }
        self.next(GString::new());
    }

    /// 跳转到指定 stage
    #[func]
    pub fn goto_stage(&mut self, label: GString) {
        if let Some(ref mut timeline) = self.timeline {
            timeline.goto_stage(&label.to_string());
        }
    }

    /// 返回所有 stage 名称列表
    #[func]
    pub fn all_stages(&mut self) -> PackedStringArray {
        if let Some(ref timeline) = self.timeline {
            let stages = timeline.all_stages();
            return stages.iter()
                .map(|s| GString::from(s.as_str()))
                .collect();
        }
        PackedStringArray::new()
    }

    /// 是否还有下一条对话
    #[func]
    pub fn has_next(&self) -> bool {
        self.timeline.as_ref().map_or(false, |t| t.has_next())
    }

    /// 对话是否正在播放中
    #[func]
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// 在 dialogue_control 节点上调用方法（无 control 或方法缺失时返回 nil）
    /// 用于触发器条件回调（如 DialogBox.has_flag）等跨节点查询
    #[func]
    pub fn call_control(&mut self, fn_name: GString, args: VarArray) -> Variant {
        let Some(ref mut control) = self.dialogue_control else {
            return Variant::nil();
        };
        if !control.is_instance_valid() {
            return Variant::nil();
        }
        let has: bool = control.call("has_method", &[fn_name.to_variant()]).to();
        if has {
            control.callv(fn_name.to_string().as_str(), &args)
        } else {
            Variant::nil()
        }
    }

    /// 获取 stage 的索引
    #[func]
    pub fn stage_index(&self, label: GString) -> i32 {
        self.timeline.as_ref().map_or(-1, |t| t.stage_index(&label.to_string()))
    }
}

// === 内部方法 ===
impl GdDialogue {
    /// 解析 dialogue_control 节点引用
    fn resolve_dialogue_control(&mut self) {
        if self.dialogue_control_path.is_empty() {
            self.dialogue_control = None;
            return;
        }
        self.dialogue_control = self.try_get_node(&self.dialogue_control_path);
    }

    /// 获取对话控制节点
    fn get_dialogue_control(&self) -> Option<Gd<Node>> {
        self.dialogue_control.clone()
    }

    /// 通过路径获取节点（安全方式）
    fn try_get_node(&self, path: &NodePath) -> Option<Gd<Node>> {
        let base = self.base();
        base.get_node_or_null(path)
    }

    /// 安装 Timeline 的 precheck 回调（stage flag 门控）
    ///
    /// precheck 在 timeline 内部被同步调用（next/goto_stage），绝不能再
    /// 借用 self（GdCell 单借用重入会 panic），所以闭包只捕获 control 的
    /// Gd<Node> 克隆，表达式直接对 control 求值。
    ///
    /// stage flag 语法（[stage@expr1;expr2]，全部通过才可进入该 stage）：
    ///   `flagname`          → control.has_flag("flagname")
    ///   `!flagname`         → 取反
    ///   `method:arg1,arg2`  → control.callv(method, args)，falsy 不过
    fn install_precheck(&mut self) {
        let control = self.get_dialogue_control();
        if let Some(ref mut timeline) = self.timeline {
            match control {
                Some(c) => {
                    self.precheck_stale = false;
                    timeline.set_precheck(Box::new(move |token: &str| {
                        Self::eval_flag_expr(&c, token)
                    }));
                }
                None => {
                    // control 未注册：先放行，next() 时发现 stale 会重装
                    self.precheck_stale = true;
                    timeline.set_precheck(Box::new(|_expr| true));
                }
            }
        }
    }

    /// 对单个 flag 表达式求值（不借用 self，可安全在 precheck 闭包内调用）
    fn eval_flag_expr(control: &Gd<Node>, token: &str) -> bool {
        let token = token.trim();
        let (negate, expr) = match token.strip_prefix('!') {
            Some(rest) => (true, rest.trim()),
            None => (false, token),
        };
        if expr.is_empty() {
            return !negate;
        }
        let mut ctl = control.clone();
        let res: Variant = if let Some((m, args)) = expr.split_once(':') {
            let var_args: VarArray = args.split(',').map(|s| s.to_variant()).collect();
            ctl.callv(m, &var_args)
        } else {
            // 无冒号的裸标记：按 has_flag 处理
            ctl.call("has_flag", &[expr.to_variant()])
        };
        let truthy: bool = res.try_to::<bool>().unwrap_or_else(|_| {
            // 非布尔返回值：非空字符串视为真，其余视为假
            res.try_to::<GString>().map_or(false, |s| !s.is_empty())
        });
        truthy != negate
    }

    /// 从文件路径加载 Timeline 数据
    fn load_timeline_from_path(&mut self, path: &GString) {
        let file = godot::classes::FileAccess::open(
            &path.to_string(),
            godot::classes::file_access::ModeFlags::READ,
        );
        if let Some(file) = file {
            let data = file.get_as_text();
            self.initial(data);
        }
    }

    /// 执行 stage 的前置条件检查
    #[allow(dead_code)]
    fn stage_precheck(&self, exprs_str: &str) -> Variant {
        let mut control = match self.get_dialogue_control() {
            Some(c) => c,
            None => return Variant::from(false),
        };

        if exprs_str.is_empty() {
            return Variant::from(true);
        }

        let expr_list: Vec<&str> = exprs_str.split(';').collect();
        for expr in expr_list {
            let parts: Vec<&str> = expr.splitn(2, ':').collect();
            if parts.is_empty() {
                continue;
            }

            let res = if parts.len() == 1 {
                control.call(parts[0], &[])
            } else if parts.len() == 2 {
                let args: VarArray = parts[1].split(',')
                    .map(|s| s.to_variant())
                    .collect();
                control.callv(parts[0], &args)
            } else {
                continue;
            };

            // 检查返回值
            if res.get_type() == godot::builtin::VariantType::STRING {
                let res_str: GString = res.to::<GString>();
                if res_str.to_string().starts_with("error:") {
                    return Variant::from(false);
                }
            } else if res.get_type() == godot::builtin::VariantType::BOOL {
                let val: bool = res.to::<bool>();
                if !val {
                    return Variant::from(false);
                }
            }
        }
        Variant::from(true)
    }
}
