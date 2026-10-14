// GdUITree —— 树形 UI 组件（GML 标签 <Tree> + <TreeNode>）
//
// 通用树形结构：横向（right/left）或纵向（down/up）四向生长，父节点到子节点
// 画肘形连线，节点可展开/收起、选中、锁定、携带任意状态。适用：地图路径、
// 技能点升级树、剧情分支等。
//
// GML 用法（构建期声明静态结构，同 NavItem 的占位符契约）：
//   <Tree direction="up" node_width="120" node_height="48" level_gap="44">
//     <TreeNode id="root" text="道基">
//       <TreeNode id="a" text="功法" desc="提升攻击" />
//       <TreeNode id="b" text="体修" desc="提升生命" locked="true" />
//     </TreeNode>
//   </Tree>
//
// 运行期 API：
//   dump_state() -> String          状态导出（JSON 文本）
//   restore_state(json) -> bool     从 JSON 恢复状态（按 id 匹配现有结构）
//   build_from_json(json) -> bool   从 JSON 动态建树（替换现有结构）
//   select_node(id) / get_selected_id() / toggle_expand(id) / expand_all() / collapse_all()
//   set_node_locked(id, b) / is_node_locked(id) / set_node_state(id, dict) / get_node_state(id)
//   信号：s_node_clicked(id) / s_node_selected(id) / s_node_expanded(id, expanded)
//
// 状态 JSON 格式：
//   {"direction":"up","selected":"root","nodes":{"<id>":{"expanded":true,
//     "locked":false,"tone":"normal","state":{...任意脚本数据...}}}}
// 结构（nodes 树形数组）由 gml 标记或 build_from_json 提供，dump/restore 只管状态；
// build_from_json 可同时携带结构 {"nodes":[{id,text,desc,children:[...],...}]}。
//
// tone 视觉语义（demo 层映射：normal=可学 done=已习得 locked=未解锁 active=高亮）。

use std::collections::HashMap;

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Vector2, Dictionary, PackedStringArray};

/// 本组件统一使用无类型字典
type VarDict = Dictionary<Variant, Variant>;
use godot::classes::{IControl, Control, Button, StyleBoxFlat, Json, ScrollContainer};
use godot::obj::{WithBaseField, NewGd};

/// 展开指示箭头尺寸
const ARROW_SIZE: f32 = 18.0;

#[derive(Clone)]
struct TreeNodeData {
    id: String,
    text: String,
    desc: String,
    locked: bool,
    expanded: bool,
    /// 视觉基调：normal / active / done / locked
    tone: String,
    /// 脚本自定义状态（随 dump/restore 序列化）
    state: VarDict,
    /// 父节点下标（-1 = 根）
    parent: i32,
    /// 子节点下标列表
    children: Vec<usize>,
    /// 行按钮（含展开箭头子按钮）
    row: Option<Gd<Button>>,
    /// 布局位置（左上角，容器本地坐标）
    pos: Vector2,
}

impl TreeNodeData {
    fn new(id: String) -> Self {
        Self {
            id,
            text: String::new(),
            desc: String::new(),
            locked: false,
            expanded: true,
            tone: "normal".to_string(),
            state: VarDict::new(),
            parent: -1,
            children: Vec::new(),
            row: None,
            pos: Vector2::ZERO,
        }
    }
    fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdUITree {
    base: Base<Control>,

    // ---- 布局导出 ----
    /// 生长方向：down / up / left / right
    #[export]
    direction: GString,
    #[export]
    node_width: f32,
    #[export]
    node_height: f32,
    /// 层间（父子）间距
    #[export]
    level_gap: f32,
    /// 同层（兄弟）间距
    #[export]
    sibling_gap: f32,
    #[export]
    font_size: i32,

    // ---- 视觉导出 ----
    #[export]
    line_color: Color,
    #[export]
    line_width: f32,
    #[export]
    node_bg: Color,
    #[export]
    node_border: Color,
    #[export]
    text_color: Color,
    /// tone=done（完成/已习得）
    #[export]
    bg_done: Color,
    #[export]
    border_done: Color,
    #[export]
    text_done: Color,
    /// tone=locked（未解锁）
    #[export]
    bg_locked: Color,
    #[export]
    border_locked: Color,
    #[export]
    text_locked: Color,
    /// tone=active 强调色
    #[export]
    accent_color: Color,
    /// 选中节点边框色
    #[export]
    select_color: Color,

    // ---- 运行时模型 ----
    nodes: Vec<TreeNodeData>,
    id_index: HashMap<String, usize>,
    selected: Option<String>,
    ui_built: bool,
    /// 视口 size_changed 是否已连接（每实例一次）
    vp_wired: bool,
    /// 父级 ScrollContainer resized 是否已连接（面板/模态内可用区域跟随）
    scroll_wired: bool,
    /// rebuild 中转：行 __tree_parent id 暂存（第二遍父子链接用）
    pending_parent: Vec<String>,
}

#[godot_api]
impl IControl for GdUITree {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            direction: "down".into(),
            node_width: 120.0,
            node_height: 48.0,
            level_gap: 44.0,
            sibling_gap: 14.0,
            font_size: 13,
            line_color: Color::from_rgb(0.35, 0.4, 0.5),
            line_width: 2.0,
            node_bg: Color::from_rgb(0.16, 0.18, 0.23),
            node_border: Color::from_rgb(0.29, 0.33, 0.41),
            text_color: Color::from_rgb(0.91, 0.91, 0.92),
            bg_done: Color::from_rgb(0.14, 0.25, 0.18),
            border_done: Color::from_rgb(0.3, 0.69, 0.43),
            text_done: Color::from_rgb(0.79, 0.94, 0.83),
            bg_locked: Color::from_rgb(0.14, 0.15, 0.17),
            border_locked: Color::from_rgb(0.23, 0.24, 0.27),
            text_locked: Color::from_rgb(0.42, 0.44, 0.48),
            accent_color: Color::from_rgb(0.31, 0.64, 1.0),
            select_color: Color::from_rgb(1.0, 0.84, 0.42),
            nodes: Vec::new(),
            id_index: HashMap::new(),
            selected: None,
            ui_built: false,
            vp_wired: false,
            scroll_wired: false,
            pending_parent: Vec::new(),
        }
    }

    fn ready(&mut self) {
        if !self.ui_built {
            // 构建期 initial() 已建行（tscn 序列化）→ 运行期从行 meta 重建模型；
            // 否则尝试解析占位子节点（parse_string 动态构建等路径）
            if !self.rebuild_from_rows() {
                if self.parse_placeholders() {
                    self.build_rows();
                }
            }
            self.ui_built = true;
        }
        // 视口尺寸变化 → 重排（独立场景兜底）
        if !self.vp_wired {
            if let Some(vp) = self.base().get_viewport() {
                let mut vp = vp;
                let cb = Callable::from_object_method(&*self.base(), "relayout");
                vp.connect(&StringName::from("size_changed"), &cb);
                self.vp_wired = true;
            }
        }
        // 父级 ScrollContainer 尺寸变化 → 重排（面板/模态内真实可用区域）
        if !self.scroll_wired {
            if let Some(parent) = self.base().get_parent() {
                if let Ok(mut scroll) = parent.try_cast::<ScrollContainer>() {
                    let cb = Callable::from_object_method(&*self.base(), "relayout");
                    scroll.connect(&StringName::from("resized"), &cb);
                    self.scroll_wired = true;
                }
            }
        }
        self.layout_all();
    }

    fn draw(&mut self) {
        self.draw_connectors();
    }

    fn exit_tree(&mut self) {
        // 断开视口信号：节点释放后悬垂 callable 会在视口 size_changed 时崩溃
        if self.vp_wired {
            if let Some(mut vp) = self.base().get_viewport() {
                let cb = Callable::from_object_method(&*self.base(), "relayout");
                vp.disconnect(&StringName::from("size_changed"), &cb);
            }
            self.vp_wired = false;
        }
        // 同理断开父级 ScrollContainer 的 resized（悬垂 callable 红线）
        if self.scroll_wired {
            if let Some(parent) = self.base().get_parent() {
                if let Ok(mut scroll) = parent.try_cast::<ScrollContainer>() {
                    let cb = Callable::from_object_method(&*self.base(), "relayout");
                    scroll.disconnect(&StringName::from("resized"), &cb);
                }
            }
            self.scroll_wired = false;
        }
    }
}

#[godot_api]
impl GdUITree {
    #[signal]
    fn s_node_clicked(id: GString);

    #[signal]
    fn s_node_selected(id: GString);

    #[signal]
    fn s_node_expanded(id: GString, expanded: bool);

    // ------------------------------------------------------------------
    // 构建期入口（builder 在子节点装配完成后调用，同列表 initial() 契约）
    // ------------------------------------------------------------------

    /// 解析 <TreeNode> 占位子节点 → 数据模型 → 建行 → 布局。
    /// 构建期调用（行节点设 owner 随 tscn 序列化）；运行期动态路径由 ready 处理。
    #[func]
    pub fn initial(&mut self) {
        if self.ui_built {
            return;
        }
        self.ui_built = true;
        self.parse_placeholders();
        self.build_rows();
        self.layout_all();
    }

    /// 递归解析占位子节点（带 __tree_node meta 的 Control）。返回是否找到。
    fn parse_placeholders(&mut self) -> bool {
        let children = self.base().get_children();
        let mut found = false;
        let mut to_remove: Vec<Gd<Node>> = Vec::new();
        for i in 0..children.len() {
            if let Some(child) = children.get(i) {
                let Ok(ctrl) = child.clone().try_cast::<Control>() else {
                    continue;
                };
                if !ctrl.has_meta(&StringName::from("__tree_node")) {
                    continue;
                }
                found = true;
                self.parse_placeholder_rec(&ctrl, -1);
                to_remove.push(child.clone());
            }
        }
        for mut n in to_remove {
            self.base_mut().remove_child(&n);
            n.queue_free();
        }
        found
    }

    /// 递归解析单个占位节点（含其后代）
    fn parse_placeholder_rec(&mut self, ctrl: &Gd<Control>, parent: i32) -> usize {
        let mut data = TreeNodeData::new(String::new());
        if ctrl.has_meta(&StringName::from("__tree_id")) {
            data.id = ctrl.get_meta(&StringName::from("__tree_id")).to_string();
        }
        if ctrl.has_meta(&StringName::from("__tree_text")) {
            data.text = ctrl.get_meta(&StringName::from("__tree_text")).to_string();
        }
        if ctrl.has_meta(&StringName::from("__tree_desc")) {
            data.desc = ctrl.get_meta(&StringName::from("__tree_desc")).to_string();
        }
        if ctrl.has_meta(&StringName::from("__tree_locked")) {
            data.locked = ctrl.get_meta(&StringName::from("__tree_locked")).to::<bool>();
        }
        if ctrl.has_meta(&StringName::from("__tree_expanded")) {
            data.expanded = ctrl.get_meta(&StringName::from("__tree_expanded")).to::<bool>();
        }
        if ctrl.has_meta(&StringName::from("__tree_tone")) {
            data.tone = ctrl.get_meta(&StringName::from("__tree_tone")).to_string();
        }
        if data.id.is_empty() {
            data.id = format!("n{}", self.nodes.len());
        }
        data.parent = parent;
        let idx = self.push_node(data);

        // 后代
        let sub = ctrl.get_children();
        for i in 0..sub.len() {
            if let Some(child) = sub.get(i) {
                if let Ok(sub_ctrl) = child.clone().try_cast::<Control>() {
                    if sub_ctrl.has_meta(&StringName::from("__tree_node")) {
                        let child_idx = self.parse_placeholder_rec(&sub_ctrl, idx as i32);
                        self.nodes[idx].children.push(child_idx);
                    }
                }
            }
        }
        idx
    }

    /// 登记节点（id 去重：冲突自动追加后缀）
    fn push_node(&mut self, mut data: TreeNodeData) -> usize {
        if self.id_index.contains_key(&data.id) {
            let mut n = 1;
            loop {
                let cand = format!("{}_{}", data.id, n);
                if !self.id_index.contains_key(&cand) {
                    godot_warn!("[GdUITree] 节点 id '{}' 重复，已改用 '{}'", data.id, cand);
                    data.id = cand;
                    break;
                }
                n += 1;
            }
        }
        let idx = self.nodes.len();
        self.id_index.insert(data.id.clone(), idx);
        self.nodes.push(data);
        idx
    }

    /// 为模型中每个节点创建行按钮（构建期：owner=自身随 tscn 序列化）
    fn build_rows(&mut self) {
        let count = self.nodes.len();
        for idx in 0..count {
            self.create_row(idx, true);
        }
    }

    /// 创建节点行（Button + 展开箭头子按钮）
    fn create_row(&mut self, idx: usize, set_owner: bool) {
        let (text, desc, w, h, font_size) = {
            let n = &self.nodes[idx];
            (
                n.text.clone(),
                n.desc.clone(),
                self.node_width,
                self.node_height,
                self.font_size,
            )
        };
        let mut row = Button::new_alloc();
        row.set_text(&GString::from(text.as_str()));
        row.set_custom_minimum_size(Vector2::new(w, h));
        row.set_size(Vector2::new(w, h));
        row.add_theme_font_size_override(&StringName::from("font_size"), font_size);
        row.set_clip_text(true);
        // 元信息 meta（随 tscn 序列化，运行期 rebuild_from_rows 的数据源）
        let n = &self.nodes[idx];
        row.set_meta(&StringName::from("__tree_row"), &true.to_variant());
        row.set_meta(&StringName::from("__tree_id"), &GString::from(n.id.as_str()).to_variant());
        row.set_meta(&StringName::from("__tree_text"), &GString::from(n.text.as_str()).to_variant());
        row.set_meta(&StringName::from("__tree_desc"), &GString::from(n.desc.as_str()).to_variant());
        row.set_meta(&StringName::from("__tree_locked"), &n.locked.to_variant());
        row.set_meta(&StringName::from("__tree_expanded"), &n.expanded.to_variant());
        row.set_meta(&StringName::from("__tree_tone"), &GString::from(n.tone.as_str()).to_variant());
        row.set_meta(&StringName::from("__tree_state"), &n.state.to_variant());
        let parent_id = if n.parent >= 0 {
            self.nodes[n.parent as usize].id.clone()
        } else {
            String::new()
        };
        row.set_meta(&StringName::from("__tree_parent"), &GString::from(parent_id.as_str()).to_variant());
        // desc → 悬停 tips（挂树自举 wire_gml_tips 自动接线；动态行错过扫描期则无提示）
        if !desc.is_empty() {
            row.set_meta(&StringName::from("__tips"), &GString::from(desc.as_str()).to_variant());
            row.set_meta(&StringName::from("__tips_title"), &GString::from(text.as_str()).to_variant());
        }

        // 展开箭头（有子节点才显示）
        let has_children = self.nodes[idx].has_children();
        let mut arrow = Button::new_alloc();
        arrow.set_name(&StringName::from("__TreeArrow"));
        arrow.set_text(&GString::from(if has_children && self.nodes[idx].expanded { "▾" } else { "▸" }));
        arrow.set_custom_minimum_size(Vector2::new(ARROW_SIZE, ARROW_SIZE));
        arrow.set_size(Vector2::new(ARROW_SIZE, ARROW_SIZE));
        arrow.add_theme_font_size_override(&StringName::from("font_size"), 11);
        arrow.set_focus_mode(godot::classes::control::FocusMode::NONE);
        let mut transparent = StyleBoxFlat::new_gd();
        transparent.set_bg_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        for state in ["normal", "hover", "pressed", "disabled"] {
            arrow.add_theme_stylebox_override(&StringName::from(state), &transparent.clone());
        }
        row.add_child(&arrow);

        // 挂到树根
        let mut row_node: Gd<Control> = row.clone().upcast();
        self.base_mut().add_child(&row_node);
        if set_owner {
            row_node.set_owner(&*self.base());
            arrow.set_owner(&row_node);
        }

        self.nodes[idx].row = Some(row);
        self.wire_row(idx);
        self.apply_row_style(idx);
    }

    /// 连接行/箭头信号（幂等：已有连接则跳过）
    fn wire_row(&mut self, idx: usize) {
        let Some(ref row) = self.nodes[idx].row else { return };
        if row
            .get_signal_connection_list(&StringName::from("pressed"))
            .is_empty()
        {
            let cb = Callable::from_object_method(&*self.base(), "_on_row_pressed")
                .bind(&[(idx as i64).to_variant()]);
            row.clone().connect(&StringName::from("pressed"), &cb);
        }
        // 箭头
        let arrow = row.get_node_or_null(&NodePath::from("__TreeArrow"));
        if let Some(a) = arrow {
            if let Ok(mut ab) = a.try_cast::<Button>() {
                if ab
                    .get_signal_connection_list(&StringName::from("pressed"))
                    .is_empty()
                {
                    let cb = Callable::from_object_method(&*self.base(), "_on_arrow_pressed")
                        .bind(&[(idx as i64).to_variant()]);
                    ab.connect(&StringName::from("pressed"), &cb);
                }
            }
        }
    }

    /// 运行期从已序列化的行节点重建模型（tscn 直开路径）。返回是否找到行。
    fn rebuild_from_rows(&mut self) -> bool {
        let children = self.base().get_children();
        let mut rows: Vec<Gd<Button>> = Vec::new();
        for i in 0..children.len() {
            if let Some(child) = children.get(i) {
                if let Ok(ctrl) = child.clone().try_cast::<Control>() {
                    if ctrl.has_meta(&StringName::from("__tree_row")) {
                        if let Ok(btn) = ctrl.try_cast::<Button>() {
                            rows.push(btn);
                        }
                    }
                }
            }
        }
        if rows.is_empty() {
            return false;
        }

        // 第一遍：建节点（id → 下标）
        let mut id_map: HashMap<String, usize> = HashMap::new();
        for row in &rows {
            let mut data = TreeNodeData::new(String::new());
            data.id = if row.has_meta(&StringName::from("__tree_id")) {
                row.get_meta(&StringName::from("__tree_id")).to_string()
            } else {
                format!("n{}", self.nodes.len())
            };
            data.text = if row.has_meta(&StringName::from("__tree_text")) {
                row.get_meta(&StringName::from("__tree_text")).to_string()
            } else {
                row.get_text().to_string()
            };
            data.desc = if row.has_meta(&StringName::from("__tree_desc")) {
                row.get_meta(&StringName::from("__tree_desc")).to_string()
            } else {
                String::new()
            };
            data.locked = row
                .get_meta(&StringName::from("__tree_locked"))
                .to::<bool>();
            data.expanded = row
                .get_meta(&StringName::from("__tree_expanded"))
                .to::<bool>();
            data.tone = row.get_meta(&StringName::from("__tree_tone")).to_string();
            data.state = row
                .get_meta(&StringName::from("__tree_state"))
                .to::<VarDict>();
            let pid = row.get_meta(&StringName::from("__tree_parent")).to_string();
            data.row = Some(row.clone());
            // parent 链接推迟到第二遍（存 pid 到临时字段：借用 id_map 前先取出）
            let idx = self.nodes.len();
            id_map.insert(data.id.clone(), idx);
            self.nodes.push(data);
            self.pending_parent.push(pid);
        }
        // 第二遍：父子链接
        for i in 0..self.nodes.len() {
            let pid = self.pending_parent.get(i).cloned().unwrap_or_default();
            self.pending_parent[i] = String::new();
            if pid.is_empty() {
                self.nodes[i].parent = -1;
            } else if let Some(&pi) = id_map.get(&pid) {
                self.nodes[i].parent = pi as i32;
                self.nodes[pi].children.push(i);
            } else {
                self.nodes[i].parent = -1;
            }
        }
        self.pending_parent.clear();
        self.id_index = id_map;

        // 重连信号（tscn 序列化不保留 callable）
        for idx in 0..self.nodes.len() {
            self.wire_row(idx);
            self.apply_row_style(idx);
        }
        true
    }

    // ------------------------------------------------------------------
    // 布局与绘制
    // ------------------------------------------------------------------

    fn is_vertical(&self) -> bool {
        matches!(self.direction.to_string().as_str(), "down" | "up" | "")
    }

    /// 节点是否可见（所有祖先均展开）
    fn node_visible(&self, idx: usize) -> bool {
        let mut cur = self.nodes[idx].parent;
        while cur >= 0 {
            let p = cur as usize;
            if !self.nodes[p].expanded {
                return false;
            }
            cur = self.nodes[p].parent;
        }
        true
    }

    /// 子树占位尺寸（横轴）：vertical=宽度 / horizontal=高度
    fn measure(&self, idx: usize) -> f32 {
        let n = &self.nodes[idx];
        let own = if self.is_vertical() {
            self.node_width
        } else {
            self.node_height
        };
        if !n.expanded || n.children.is_empty() {
            return own;
        }
        let sum: f32 = n.children.iter().map(|&c| self.measure(c)).sum();
        let gaps = self.sibling_gap * (n.children.len().saturating_sub(1)) as f32;
        own.max(sum + gaps)
    }

    /// 全量重排：计算每行位置 + 容器内容尺寸 + 重绘
    #[func]
    pub fn relayout(&mut self) {
        self.layout_all();
    }

    fn layout_all(&mut self) {
        if self.nodes.is_empty() {
            return;
        }
        // 内容总尺寸
        let roots: Vec<usize> = (0..self.nodes.len())
            .filter(|&i| self.nodes[i].parent < 0)
            .collect();
        let vertical = self.is_vertical();
        let pitch = if vertical {
            self.node_height + self.level_gap
        } else {
            self.node_width + self.level_gap
        };
        // 最大深度（可见性无关，折叠子树不占深度——以展开路径计）
        let max_depth = roots
            .iter()
            .map(|&r| self.subtree_max_depth(r, 0))
            .max()
            .unwrap_or(0);
        let cross_total: f32 = roots.iter().map(|&r| self.measure(r)).sum();
        let gaps = self.sibling_gap * roots.len().saturating_sub(1) as f32;
        let cross_total = cross_total + gaps;
        let (total_w, total_h) = if vertical {
            (
                cross_total,
                max_depth as f32 * pitch + self.node_height,
            )
        } else {
            (
                max_depth as f32 * pitch + self.node_width,
                cross_total,
            )
        };

        let mut cursor = 0.0;
        for &r in &roots {
            let used = if vertical {
                self.place_v(r, cursor, 0, total_h)
            } else {
                self.place_h(r, cursor, 0, total_w)
            };
            cursor += used + self.sibling_gap;
        }

        // 行可见性：折叠子树整体隐藏（祖先链任一收起即不可见）
        let vis_flags: Vec<bool> = (0..self.nodes.len()).map(|i| self.node_visible(i)).collect();
        for (idx, vis) in vis_flags.iter().enumerate() {
            if let Some(ref mut row) = self.nodes[idx].row {
                if row.is_instance_valid() {
                    row.set_visible(*vis);
                }
            }
        }

        // 容器尺寸：用父级真实可用区域（available_size），内容宽不足时水平居中；
        // 生长方向锚定 —— direction=up 内容贴底向上长、direction=left 内容贴右
        // 向左长（可用空间富余时控件撑满滚动区，行整体偏移到对应边）。
        let avail = self.available_size();
        let dir = self.direction.to_string();
        let (mut outer_w, mut outer_h) = (total_w, total_h);
        let (mut x_off, mut y_off) = (0.0_f32, 0.0_f32);
        if vertical {
            outer_w = total_w.max(avail.x);
            x_off = (outer_w - total_w) / 2.0;
            if dir == "up" && total_h < avail.y {
                outer_h = avail.y;
                y_off = outer_h - total_h;
            }
        } else {
            outer_h = total_h.max(avail.y);
            y_off = (outer_h - total_h) / 2.0;
            if dir == "left" && total_w < avail.x {
                outer_w = avail.x;
                x_off = outer_w - total_w;
            }
        }
        if x_off > 0.5 || y_off > 0.5 {
            for n in &mut self.nodes {
                n.pos.x += x_off;
                n.pos.y += y_off;
                if let Some(ref mut row) = n.row {
                    if row.is_instance_valid() {
                        let mut p = row.get_position();
                        p.x += x_off;
                        p.y += y_off;
                        row.set_position(p);
                    }
                }
            }
        }
        let mut base = self.base_mut();
        base.set_custom_minimum_size(Vector2::new(outer_w, outer_h));
        base.set_size(Vector2::new(outer_w, outer_h));
        base.queue_redraw();
    }

    fn subtree_max_depth(&self, idx: usize, depth: i32) -> i32 {
        let n = &self.nodes[idx];
        if !n.expanded || n.children.is_empty() {
            return depth;
        }
        n.children
            .iter()
            .map(|&c| self.subtree_max_depth(c, depth + 1))
            .max()
            .unwrap_or(depth)
    }

    /// 布局可用区域：父级为 ScrollContainer 时取其真实矩形（面板/模态内布局），
    /// 否则退回视口（独立整页场景）。严禁直接用视口宽当内容宽——在模态里
    /// 会把最小宽度撑到超过父容器，把整条容器链顶穿（顶栏溢出 bug 根因）。
    fn available_size(&self) -> Vector2 {
        let base = self.base();
        if !base.is_inside_tree() {
            return Vector2::ZERO;
        }
        if let Some(parent) = base.get_parent() {
            if let Ok(scroll) = parent.try_cast::<ScrollContainer>() {
                return scroll.get_size();
            }
        }
        base.get_viewport_rect().size
    }

    /// 纵向布局（down：自上而下 / up：自下而上）。返回子树占用宽度。
    fn place_v(&mut self, idx: usize, x_cursor: f32, depth: i32, total_h: f32) -> f32 {
        let w = self.measure(idx);
        let pitch = self.node_height + self.level_gap;
        let nx = x_cursor + w / 2.0 - self.node_width / 2.0;
        let ny = match self.direction.to_string().as_str() {
            "up" => total_h - depth as f32 * pitch - self.node_height,
            _ => depth as f32 * pitch,
        };
        self.nodes[idx].pos = Vector2::new(nx, ny);
        if let Some(ref mut row) = self.nodes[idx].row {
            if row.is_instance_valid() {
                row.set_position(Vector2::new(nx, ny));
            }
        }
        let n = &self.nodes[idx];
        if n.expanded && !n.children.is_empty() {
            let mut child_cursor = x_cursor;
            let children = n.children.clone();
            for c in children {
                let used = self.place_v(c, child_cursor, depth + 1, total_h);
                child_cursor += used + self.sibling_gap;
            }
        }
        w
    }

    /// 横向布局（right：自左向右 / left：自右向左）。返回子树占用高度。
    fn place_h(&mut self, idx: usize, y_cursor: f32, depth: i32, total_w: f32) -> f32 {
        let h = self.measure(idx);
        let pitch = self.node_width + self.level_gap;
        let nx = match self.direction.to_string().as_str() {
            "left" => total_w - depth as f32 * pitch - self.node_width,
            _ => depth as f32 * pitch,
        };
        let ny = y_cursor + h / 2.0 - self.node_height / 2.0;
        self.nodes[idx].pos = Vector2::new(nx, ny);
        if let Some(ref mut row) = self.nodes[idx].row {
            if row.is_instance_valid() {
                row.set_position(Vector2::new(nx, ny));
            }
        }
        let n = &self.nodes[idx];
        if n.expanded && !n.children.is_empty() {
            let mut child_cursor = y_cursor;
            let children = n.children.clone();
            for c in children {
                let used = self.place_h(c, child_cursor, depth + 1, total_w);
                child_cursor += used + self.sibling_gap;
            }
        }
        h
    }

    /// 父子连线（肘形三段）
    fn draw_connectors(&mut self) {
        let color = self.line_color;
        let width = self.line_width;
        let (nw, nh) = (self.node_width, self.node_height);
        for idx in 0..self.nodes.len() {
            let p = self.nodes[idx].parent;
            if p < 0 || !self.node_visible(idx) {
                continue;
            }
            let (pp, cp) = (self.nodes[p as usize].pos, self.nodes[idx].pos);
            let dir = self.direction.to_string();
            match dir.as_str() {
                "up" => {
                    let a = Vector2::new(pp.x + nw / 2.0, pp.y);
                    let d = Vector2::new(cp.x + nw / 2.0, cp.y + nh);
                    let mid_y = (a.y + d.y) / 2.0;
                    let b = Vector2::new(a.x, mid_y);
                    let c = Vector2::new(d.x, mid_y);
                    self.base_mut().draw_line_ex(a, b, color).width(width).done();
                    self.base_mut().draw_line_ex(b, c, color).width(width).done();
                    self.base_mut().draw_line_ex(c, d, color).width(width).done();
                }
                "right" => {
                    let a = Vector2::new(pp.x + nw, pp.y + nh / 2.0);
                    let d = Vector2::new(cp.x, cp.y + nh / 2.0);
                    let mid_x = (a.x + d.x) / 2.0;
                    let b = Vector2::new(mid_x, a.y);
                    let c = Vector2::new(mid_x, d.y);
                    self.base_mut().draw_line_ex(a, b, color).width(width).done();
                    self.base_mut().draw_line_ex(b, c, color).width(width).done();
                    self.base_mut().draw_line_ex(c, d, color).width(width).done();
                }
                "left" => {
                    let a = Vector2::new(pp.x, pp.y + nh / 2.0);
                    let d = Vector2::new(cp.x + nw, cp.y + nh / 2.0);
                    let mid_x = (a.x + d.x) / 2.0;
                    let b = Vector2::new(mid_x, a.y);
                    let c = Vector2::new(mid_x, d.y);
                    self.base_mut().draw_line_ex(a, b, color).width(width).done();
                    self.base_mut().draw_line_ex(b, c, color).width(width).done();
                    self.base_mut().draw_line_ex(c, d, color).width(width).done();
                }
                _ => {
                    // down
                    let a = Vector2::new(pp.x + nw / 2.0, pp.y + nh);
                    let d = Vector2::new(cp.x + nw / 2.0, cp.y);
                    let mid_y = (a.y + d.y) / 2.0;
                    let b = Vector2::new(a.x, mid_y);
                    let c = Vector2::new(d.x, mid_y);
                    self.base_mut().draw_line_ex(a, b, color).width(width).done();
                    self.base_mut().draw_line_ex(b, c, color).width(width).done();
                    self.base_mut().draw_line_ex(c, d, color).width(width).done();
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // 视觉样式
    // ------------------------------------------------------------------

    /// 按 tone + locked + selected 刷行样式与箭头
    fn apply_row_style(&mut self, idx: usize) {
        let (bg, border, text_color) = {
            let n = &self.nodes[idx];
            match n.tone.as_str() {
                "done" => (self.bg_done, self.border_done, self.text_done),
                "locked" => (self.bg_locked, self.border_locked, self.text_locked),
                "active" => (self.node_bg, self.accent_color, self.text_color),
                _ => (self.node_bg, self.node_border, self.text_color),
            }
        };
        let selected = self.selected.as_deref() == Some(self.nodes[idx].id.as_str());
        let border = if selected { self.select_color } else { border };
        let border_w = if selected { 3.0 } else { 1.5 };
        let has_children = self.nodes[idx].has_children();
        let expanded = self.nodes[idx].expanded;
        let (nw, nh) = (self.node_width, self.node_height);

        let Some(ref mut row) = self.nodes[idx].row else { return };
        if !row.is_instance_valid() {
            return;
        }
        let mut sb = StyleBoxFlat::new_gd();
        sb.set_bg_color(bg);
        sb.set_border_color(border);
        sb.set_border_width_all(border_w as i32);
        sb.set_corner_radius_all(8);
        row.add_theme_stylebox_override(&StringName::from("normal"), &sb.clone());
        let mut hover = StyleBoxFlat::new_gd();
        hover.set_bg_color(Color::from_rgba(
            (bg.r + 0.08).min(1.0),
            (bg.g + 0.08).min(1.0),
            (bg.b + 0.08).min(1.0),
            bg.a,
        ));
        hover.set_border_color(border);
        hover.set_border_width_all(border_w as i32);
        hover.set_corner_radius_all(8);
        row.add_theme_stylebox_override(&StringName::from("hover"), &hover.clone());
        row.add_theme_stylebox_override(&StringName::from("pressed"), &sb);
        row.add_theme_color_override(&StringName::from("font_color"), text_color);
        row.add_theme_color_override(&StringName::from("font_hover_color"), text_color);
        row.add_theme_color_override(&StringName::from("font_pressed_color"), text_color);

        // 箭头
        if let Some(a) = row.get_node_or_null(&NodePath::from("__TreeArrow")) {
            if let Ok(mut ab) = a.try_cast::<Button>() {
                ab.set_visible(has_children);
                ab.set_text(&GString::from(if expanded { "▾" } else { "▸" }));
                ab.set_position(Vector2::new(
                    nw - ARROW_SIZE - 4.0,
                    (nh - ARROW_SIZE) / 2.0,
                ));
            }
        }
    }

    fn refresh_all_styles(&mut self) {
        for idx in 0..self.nodes.len() {
            self.apply_row_style(idx);
        }
    }

    // ------------------------------------------------------------------
    // 回调
    // ------------------------------------------------------------------

    /// 行点击：选中 + 发信号（锁定节点照常发信号，由脚本层决定是否可操作）
    #[func]
    fn _on_row_pressed(&mut self, idx: i64) {
        let idx = idx as usize;
        if idx >= self.nodes.len() {
            return;
        }
        let id = GString::from(self.nodes[idx].id.as_str());
        self.set_selected_internal(idx);
        self.base_mut()
            .emit_signal(&StringName::from("s_node_clicked"), &[id.to_variant()]);
    }

    /// 箭头点击：展开/收起
    #[func]
    fn _on_arrow_pressed(&mut self, idx: i64) {
        let idx = idx as usize;
        if idx >= self.nodes.len() {
            return;
        }
        let id = GString::from(self.nodes[idx].id.as_str());
        let expanded = !self.nodes[idx].expanded;
        self.nodes[idx].expanded = expanded;
        self.sync_row_meta(idx);
        self.apply_row_style(idx);
        self.layout_all();
        self.base_mut().emit_signal(
            &StringName::from("s_node_expanded"),
            &[id.to_variant(), expanded.to_variant()],
        );
    }

    /// 行 meta 与模型同步（dump 前调用保证一致；行 meta 是 tscn 持久化源）
    fn sync_row_meta(&mut self, idx: usize) {
        let Some(row) = self.nodes[idx].row.clone() else { return };
        if !row.is_instance_valid() {
            return;
        }
        let mut row = row;
        let n = &self.nodes[idx];
        row.set_meta(&StringName::from("__tree_locked"), &n.locked.to_variant());
        row.set_meta(&StringName::from("__tree_expanded"), &n.expanded.to_variant());
        row.set_meta(&StringName::from("__tree_tone"), &GString::from(n.tone.as_str()).to_variant());
        row.set_meta(&StringName::from("__tree_state"), &n.state.to_variant());
    }

    fn set_selected_internal(&mut self, idx: usize) {
        let new_id = self.nodes[idx].id.clone();
        if self.selected.as_deref() == Some(new_id.as_str()) {
            return;
        }
        let old_idx = self
            .selected
            .as_deref()
            .and_then(|id| self.id_index.get(id).copied());
        self.selected = Some(new_id);
        if let Some(old) = old_idx {
            self.apply_row_style(old);
        }
        self.apply_row_style(idx);
    }

    // ------------------------------------------------------------------
    // 公开 API
    // ------------------------------------------------------------------

    /// 选中节点（emit=true 时发 s_node_selected）
    #[func]
    pub fn select_node(&mut self, id: GString, emit: bool) -> bool {
        let Some(&idx) = self.id_index.get(id.to_string().as_str()) else {
            return false;
        };
        self.set_selected_internal(idx);
        if emit {
            let gid = GString::from(self.nodes[idx].id.as_str());
            self.base_mut()
                .emit_signal(&StringName::from("s_node_selected"), &[gid.to_variant()]);
        }
        true
    }

    /// 当前选中节点 id（未选中返回空串）
    #[func]
    pub fn get_selected_id(&self) -> GString {
        match &self.selected {
            Some(id) => GString::from(id.as_str()),
            None => GString::new(),
        }
    }

    /// 展开/收起指定节点
    #[func]
    pub fn toggle_expand(&mut self, id: GString) -> bool {
        let Some(&idx) = self.id_index.get(id.to_string().as_str()) else {
            return false;
        };
        let expanded = !self.nodes[idx].expanded;
        self.nodes[idx].expanded = expanded;
        self.sync_row_meta(idx);
        self.apply_row_style(idx);
        self.layout_all();
        let gid = GString::from(self.nodes[idx].id.as_str());
        self.base_mut().emit_signal(
            &StringName::from("s_node_expanded"),
            &[gid.to_variant(), expanded.to_variant()],
        );
        true
    }

    #[func]
    pub fn expand_all(&mut self) {
        for idx in 0..self.nodes.len() {
            if self.nodes[idx].has_children() && !self.nodes[idx].expanded {
                self.nodes[idx].expanded = true;
                self.sync_row_meta(idx);
                self.apply_row_style(idx);
            }
        }
        self.layout_all();
    }

    #[func]
    pub fn collapse_all(&mut self) {
        for idx in 0..self.nodes.len() {
            if self.nodes[idx].has_children() && self.nodes[idx].expanded {
                self.nodes[idx].expanded = false;
                self.sync_row_meta(idx);
                self.apply_row_style(idx);
            }
        }
        self.layout_all();
    }

    /// 节点锁定（视觉置灰；点击仍发信号，由脚本层校验）
    #[func]
    pub fn set_node_locked(&mut self, id: GString, locked: bool) -> bool {
        let Some(&idx) = self.id_index.get(id.to_string().as_str()) else {
            return false;
        };
        self.nodes[idx].locked = locked;
        self.sync_row_meta(idx);
        self.apply_row_style(idx);
        true
    }

    #[func]
    pub fn is_node_locked(&self, id: GString) -> bool {
        match self.id_index.get(id.to_string().as_str()) {
            Some(&idx) => self.nodes[idx].locked,
            None => false,
        }
    }

    /// 设置节点脚本状态；state 含 "tone" 键时同步视觉基调
    #[func]
    pub fn set_node_state(&mut self, id: GString, state: VarDict) -> bool {
        let Some(&idx) = self.id_index.get(id.to_string().as_str()) else {
            return false;
        };
        self.nodes[idx].state = state.clone();
        if let Some(tone) = state.get(&Variant::from("tone")) {
            let t = tone.try_to::<GString>().map(|g| g.to_string()).unwrap_or_default();
            if matches!(t.as_str(), "normal" | "active" | "done" | "locked") {
                self.nodes[idx].tone = t;
            }
        }
        self.sync_row_meta(idx);
        self.apply_row_style(idx);
        true
    }

    #[func]
    pub fn get_node_state(&self, id: GString) -> VarDict {
        match self.id_index.get(id.to_string().as_str()) {
            Some(&idx) => self.nodes[idx].state.clone(),
            None => VarDict::new(),
        }
    }

    /// 节点直接子节点 id 列表（构建顺序）
    #[func]
    pub fn get_node_children(&self, id: GString) -> PackedStringArray {
        let mut ids: Vec<String> = Vec::new();
        if let Some(&idx) = self.id_index.get(id.to_string().as_str()) {
            for &c in &self.nodes[idx].children {
                ids.push(self.nodes[c].id.clone());
            }
        }
        PackedStringArray::from_iter(ids.into_iter().map(|s| GString::from(s.as_str())))
    }

    /// 节点父节点 id（根返回空串）
    #[func]
    pub fn get_node_parent(&self, id: GString) -> GString {
        match self.id_index.get(id.to_string().as_str()) {
            Some(&idx) if self.nodes[idx].parent >= 0 => {
                GString::from(self.nodes[self.nodes[idx].parent as usize].id.as_str())
            }
            _ => GString::new(),
        }
    }

    /// 全部节点 id（构建顺序）
    #[func]
    pub fn get_node_ids(&self) -> PackedStringArray {
        PackedStringArray::from_iter(
            self.nodes.iter().map(|n| GString::from(n.id.as_str())),
        )
    }

    /// 读取节点文本
    #[func]
    pub fn get_node_text(&self, id: GString) -> GString {
        match self.id_index.get(id.to_string().as_str()) {
            Some(&idx) => GString::from(self.nodes[idx].text.as_str()),
            None => GString::new(),
        }
    }

    /// 设置节点文本
    #[func]
    pub fn set_node_text(&mut self, id: GString, text: GString) -> bool {
        let Some(&idx) = self.id_index.get(id.to_string().as_str()) else {
            return false;
        };
        self.nodes[idx].text = text.to_string();
        self.sync_row_meta(idx);
        if let Some(ref mut row) = self.nodes[idx].row {
            if row.is_instance_valid() {
                row.set_text(&text);
            }
        }
        true
    }

    // ------------------------------------------------------------------
    // JSON 状态导出 / 恢复
    // ------------------------------------------------------------------

    /// 导出状态为 JSON 文本：
    /// {"direction":"up","selected":"root","nodes":{"<id>":{expanded,locked,tone,state}}}
    #[func]
    pub fn dump_state(&self) -> GString {
        let mut nodes_d = VarDict::new();
        for n in &self.nodes {
            let mut d = VarDict::new();
            d.set(&Variant::from("expanded"), &n.expanded.to_variant());
            d.set(&Variant::from("locked"), &n.locked.to_variant());
            d.set(&Variant::from("tone"), &GString::from(n.tone.as_str()).to_variant());
            d.set(&Variant::from("state"), &n.state.to_variant());
            nodes_d.set(
                &Variant::from(GString::from(n.id.as_str())),
                &d.to_variant(),
            );
        }
        let mut root = VarDict::new();
        root.set(&Variant::from("direction"), &self.direction.to_variant());
        root.set(
            &Variant::from("selected"),
            &GString::from(self.selected.as_deref().unwrap_or("")).to_variant(),
        );
        root.set(&Variant::from("nodes"), &nodes_d.to_variant());
        Json::stringify(&root.to_variant())
    }

    /// 从 JSON 恢复状态（按 id 匹配现有结构；缺失 id 忽略）。
    /// 可选携带 "direction" 切换布局方向。返回是否成功解析。
    #[func]
    pub fn restore_state(&mut self, json: GString) -> bool {
        let parsed = Json::parse_string(&json);
        let Some(dict) = parsed.try_to::<VarDict>().ok() else {
            godot_warn!("[GdUITree] restore_state: JSON 不是对象");
            return false;
        };
        if let Some(dir) = dict.get(&Variant::from("direction")) {
            let d = dir.try_to::<GString>().map(|g| g.to_string()).unwrap_or_default();
            if matches!(d.as_str(), "down" | "up" | "left" | "right") && !d.is_empty() {
                self.direction = GString::from(d.as_str());
            }
        }
        if let Some(sel) = dict.get(&Variant::from("selected")) {
            let s = sel.try_to::<GString>().map(|g| g.to_string()).unwrap_or_default();
            if !s.is_empty() {
                if let Some(&idx) = self.id_index.get(s.as_str()) {
                    self.set_selected_internal(idx);
                }
            }
        }
        let Some(nodes_d) = dict.get(&Variant::from("nodes")).and_then(|v| v.try_to::<VarDict>().ok()) else {
            return true;
        };
        for (key, val) in nodes_d.iter_shared() {
            let id = key.to_string();
            let Some(&idx) = self.id_index.get(id.as_str()) else {
                continue;
            };
            let Some(d) = val.try_to::<VarDict>().ok() else {
                continue;
            };
            if let Some(v) = d.get(&Variant::from("expanded")) {
                self.nodes[idx].expanded = v.try_to::<bool>().unwrap_or(true);
            }
            if let Some(v) = d.get(&Variant::from("locked")) {
                self.nodes[idx].locked = v.try_to::<bool>().unwrap_or(false);
            }
            if let Some(v) = d.get(&Variant::from("tone")) {
                let t = v.try_to::<GString>().map(|g| g.to_string()).unwrap_or_default();
                if matches!(t.as_str(), "normal" | "active" | "done" | "locked") {
                    self.nodes[idx].tone = t;
                }
            }
            if let Some(v) = d.get(&Variant::from("state")) {
                if let Ok(sd) = v.try_to::<VarDict>() {
                    self.nodes[idx].state = sd;
                }
            }
            self.sync_row_meta(idx);
            self.apply_row_style(idx);
        }
        self.layout_all();
        true
    }

    /// 从 JSON 动态建树（替换现有结构与行节点）：
    /// {"direction":"up","nodes":[{"id","text","desc","locked","expanded","tone","state","children":[...递归...]}]}
    /// 返回是否成功解析并重建。
    #[func]
    pub fn build_from_json(&mut self, json: GString) -> bool {
        let parsed = Json::parse_string(&json);
        let Some(dict) = parsed.try_to::<VarDict>().ok() else {
            godot_warn!("[GdUITree] build_from_json: JSON 不是对象");
            return false;
        };
        if let Some(dir) = dict.get(&Variant::from("direction")) {
            let d = dir.try_to::<GString>().map(|g| g.to_string()).unwrap_or_default();
            if matches!(d.as_str(), "down" | "up" | "left" | "right") && !d.is_empty() {
                self.direction = GString::from(d.as_str());
            }
        }
        let Some(items) = dict.get(&Variant::from("nodes")).and_then(|v| v.try_to::<Array<Variant>>().ok()) else {
            godot_warn!("[GdUITree] build_from_json: 缺少 nodes 数组");
            return false;
        };
        // 释放旧行 + 清空模型
        self.clear_rows();
        // 递归建模型
        let count = items.len();
        for i in 0..count {
            if let Some(item) = items.get(i) {
                self.parse_json_rec(&item, -1);
            }
        }
        self.build_rows();
        self.layout_all();
        self.ui_built = true;
        true
    }

    fn parse_json_rec(&mut self, item: &Variant, parent: i32) -> usize {
        let mut data = TreeNodeData::new(String::new());
        if let Ok(d) = item.clone().try_to::<VarDict>() {
            let gets = |k: &str| -> String {
                d.get(&Variant::from(k))
                    .and_then(|v| v.try_to::<GString>().ok())
                    .map(|g| g.to_string())
                    .unwrap_or_default()
            };
            data.id = gets("id");
            data.text = gets("text");
            data.desc = gets("desc");
            if let Some(v) = d.get(&Variant::from("locked")) {
                data.locked = v.try_to::<bool>().unwrap_or(false);
            }
            if let Some(v) = d.get(&Variant::from("expanded")) {
                data.expanded = v.try_to::<bool>().unwrap_or(true);
            }
            let t = gets("tone");
            if matches!(t.as_str(), "normal" | "active" | "done" | "locked") {
                data.tone = t;
            }
            if let Some(v) = d.get(&Variant::from("state")) {
                if let Ok(sd) = v.try_to::<VarDict>() {
                    data.state = sd;
                }
            }
        }
        if data.id.is_empty() {
            data.id = format!("n{}", self.nodes.len());
        }
        data.parent = parent;
        let idx = self.push_node(data);
        if let Ok(d) = item.clone().try_to::<VarDict>() {
            if let Some(children) = d.get(&Variant::from("children")).and_then(|v| v.try_to::<Array<Variant>>().ok()) {
                for i in 0..children.len() {
                    if let Some(child) = children.get(i) {
                        let c = self.parse_json_rec(&child, idx as i32);
                        self.nodes[idx].children.push(c);
                    }
                }
            }
        }
        idx
    }

    /// 释放全部行节点并清空模型（build_from_json 重建前调用）
    fn clear_rows(&mut self) {
        for n in &self.nodes {
            if let Some(ref row) = n.row {
                if row.is_instance_valid() {
                    let mut r = row.clone();
                    r.queue_free();
                }
            }
        }
        self.nodes.clear();
        self.id_index.clear();
        self.selected = None;
    }
}
