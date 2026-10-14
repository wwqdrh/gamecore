// GdUITreeMenu —— 树形菜单组件（GML 标签 <TreeMenu> + <MenuNode>）
//
// 树级菜单栏：多层级折叠/展开、逐级自动缩进、点击选中高亮。手册目录、
// 任务分类、设置分组等侧边栏场景。与 <Tree>（图形化四向布局）不同，
// 本组件是纵向文件树式菜单：整行可点、行内按深度缩进。
//
// GML 用法（构建期声明静态结构，占位符契约同 TreeNode/NavItem）：
//   <TreeMenu indent_width="18" row_height="34" font_size="14"
//             item_bg="#00000000" item_text="#4a3f2a"
//             selected_bg="#e8dfc0" selected_text="#2e2415">
//     <MenuNode id="basics" text="基础知识">
//       <MenuNode id="ability" text="能力体系" />
//     </MenuNode>
//   </TreeMenu>
//
// 交互约定：
//   · 点击行 → 选中 + 发 s_menu_selected(id)；有子菜单时同时切换折叠/展开
//   · 点击行首箭头 → 仅折叠/展开
//   · 默认全部展开（MenuNode expanded="false" 可初始收起）
//
// 运行期 API：
//   select_item(id, emit) / get_selected_id() / toggle_expand(id)
//   expand_all() / collapse_all() / set_item_text(id, text) / is_item_expanded(id)
//   信号：s_menu_selected(id)

use std::collections::HashMap;

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Vector2};
use godot::classes::{IControl, Control, Button, Label, StyleBoxFlat, ScrollContainer};

/// 行首箭头槽宽（叶子也保留空位，跨层级文字对齐）
const ARROW_SLOT: f32 = 20.0;

/// 本组件统一使用无类型字典
type VarDict = Dictionary<Variant, Variant>;

#[derive(Clone)]
struct MenuNodeData {
    id: String,
    text: String,
    expanded: bool,
    /// 父节点下标（-1 = 根级）
    parent: i32,
    children: Vec<usize>,
    /// 行按钮（含 __MenuArrow / __MenuText 子节点）
    row: Option<Gd<Button>>,
    depth: i32,
}

impl MenuNodeData {
    fn new() -> Self {
        Self {
            id: String::new(),
            text: String::new(),
            expanded: true,
            parent: -1,
            children: Vec::new(),
            row: None,
            depth: 0,
        }
    }

    fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

#[derive(GodotClass)]
#[class(base=Control)]
pub struct GdUITreeMenu {
    base: Base<Control>,

    // ---- 导出属性（gml 可配）----
    /// 每级缩进宽度
    #[export]
    indent_width: f32,
    /// 行高
    #[export]
    row_height: f32,
    /// 行间距
    #[export]
    row_gap: f32,
    /// 行字号
    #[export]
    font_size: i32,
    /// 常规行背景（透明=无底色）
    #[export]
    item_bg: Color,
    /// 常规行文字色
    #[export]
    item_text: Color,
    /// 悬停行背景
    #[export]
    item_hover_bg: Color,
    /// 选中行背景
    #[export]
    selected_bg: Color,
    /// 选中行文字色
    #[export]
    selected_text: Color,

    // ---- 运行时模型 ----
    nodes: Vec<MenuNodeData>,
    id_index: HashMap<String, usize>,
    selected: Option<String>,
    ui_built: bool,
    /// 父级 ScrollContainer resized 是否已连接
    scroll_wired: bool,
}

#[godot_api]
impl IControl for GdUITreeMenu {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            indent_width: 18.0,
            row_height: 34.0,
            row_gap: 2.0,
            font_size: 14,
            item_bg: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            item_text: Color::from_rgb(0.29, 0.25, 0.16),
            item_hover_bg: Color::from_rgba(0.55, 0.47, 0.3, 0.18),
            selected_bg: Color::from_rgb(0.91, 0.87, 0.75),
            selected_text: Color::from_rgb(0.18, 0.14, 0.08),
            nodes: Vec::new(),
            id_index: HashMap::new(),
            selected: None,
            ui_built: false,
            scroll_wired: false,
        }
    }

    fn ready(&mut self) {
        if !self.ui_built {
            // tscn 序列化行 → 重建模型；否则解析占位子节点（动态构建路径）
            if !self.rebuild_from_rows() {
                if self.parse_placeholders() {
                    self.build_rows();
                }
            }
            self.ui_built = true;
        }
        // 父级 ScrollContainer 尺寸变化 → 重排（行宽跟随可用区域）
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

    fn exit_tree(&mut self) {
        // 悬垂 callable 红线：节点释放前必须断开外部信号
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
impl GdUITreeMenu {
    #[signal]
    fn s_menu_selected(id: GString);

    // ------------------------------------------------------------------
    // 构建期入口（同 Tree 的 initial() 契约）
    // ------------------------------------------------------------------

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

    /// 递归解析带 __menu_node meta 的占位子节点。返回是否找到。
    fn parse_placeholders(&mut self) -> bool {
        let children = self.base().get_children();
        let mut found = false;
        let mut to_remove: Vec<Gd<Node>> = Vec::new();
        for i in 0..children.len() {
            if let Some(child) = children.get(i) {
                let Ok(ctrl) = child.clone().try_cast::<Control>() else {
                    continue;
                };
                if !ctrl.has_meta(&StringName::from("__menu_node")) {
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

    fn parse_placeholder_rec(&mut self, ctrl: &Gd<Control>, parent: i32) -> usize {
        let mut data = MenuNodeData::new();
        if ctrl.has_meta(&StringName::from("__menu_id")) {
            data.id = ctrl.get_meta(&StringName::from("__menu_id")).to_string();
        }
        if ctrl.has_meta(&StringName::from("__menu_text")) {
            data.text = ctrl
                .get_meta(&StringName::from("__menu_text"))
                .to_string();
        }
        // 菜单默认展开（与 Tree 不同：目录树展开是常态）
        data.expanded = if ctrl.has_meta(&StringName::from("__menu_expanded")) {
            ctrl.get_meta(&StringName::from("__menu_expanded")).to::<bool>()
        } else {
            true
        };
        if data.id.is_empty() {
            data.id = format!("m{}", self.nodes.len());
        }
        data.parent = parent;
        data.depth = if parent >= 0 {
            self.nodes[parent as usize].depth + 1
        } else {
            0
        };
        let idx = self.push_node(data);

        let sub = ctrl.get_children();
        for i in 0..sub.len() {
            if let Some(child) = sub.get(i) {
                if let Ok(sub_ctrl) = child.clone().try_cast::<Control>() {
                    if sub_ctrl.has_meta(&StringName::from("__menu_node")) {
                        let child_idx = self.parse_placeholder_rec(&sub_ctrl, idx as i32);
                        self.nodes[idx].children.push(child_idx);
                    }
                }
            }
        }
        idx
    }

    fn push_node(&mut self, mut data: MenuNodeData) -> usize {
        if self.id_index.contains_key(&data.id) {
            let mut n = 1;
            loop {
                let cand = format!("{}_{}", data.id, n);
                if !self.id_index.contains_key(&cand) {
                    godot_warn!(
                        "[GdUITreeMenu] 菜单项 id '{}' 重复，已改用 '{}'",
                        data.id,
                        cand
                    );
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

    fn build_rows(&mut self) {
        let count = self.nodes.len();
        for idx in 0..count {
            self.create_row(idx, true);
        }
    }

    /// 创建菜单行：整行 Button（无文字）+ 行内箭头按钮 + 文字标签（按深度缩进）
    fn create_row(&mut self, idx: usize, set_owner: bool) {
        let (text, id, font_size, row_h) = {
            let n = &self.nodes[idx];
            (
                n.text.clone(),
                n.id.clone(),
                self.font_size,
                self.row_height,
            )
        };
        let mut row = Button::new_alloc();
        row.set_custom_minimum_size(Vector2::new(0.0, row_h));
        row.set_size(Vector2::new(0.0, row_h));
        row.add_theme_font_size_override(&StringName::from("font_size"), font_size);
        row.set_focus_mode(godot::classes::control::FocusMode::NONE);

        // meta 契约（随 tscn 序列化，运行期 rebuild_from_rows 数据源）
        row.set_meta(&StringName::from("__menu_row"), &true.to_variant());
        row.set_meta(&StringName::from("__menu_id"), &GString::from(id.as_str()).to_variant());
        row.set_meta(&StringName::from("__menu_text"), &GString::from(text.as_str()).to_variant());
        let expanded = self.nodes[idx].expanded;
        row.set_meta(&StringName::from("__menu_expanded"), &expanded.to_variant());
        let parent_id = if self.nodes[idx].parent >= 0 {
            self.nodes[self.nodes[idx].parent as usize].id.clone()
        } else {
            String::new()
        };
        row.set_meta(
            &StringName::from("__menu_parent"),
            &GString::from(parent_id.as_str()).to_variant(),
        );

        // 行首箭头（有子菜单才可见；叶子保留空槽位对齐文字）
        let has_children = self.nodes[idx].has_children();
        let mut arrow = Button::new_alloc();
        arrow.set_name(&StringName::from("__MenuArrow"));
        arrow.set_text(&GString::from(if expanded { "▾" } else { "▸" }));
        arrow.set_visible(has_children);
        arrow.add_theme_font_size_override(&StringName::from("font_size"), font_size - 1);
        arrow.set_focus_mode(godot::classes::control::FocusMode::NONE);
        let mut transparent = StyleBoxFlat::new_gd();
        transparent.set_bg_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        for state in ["normal", "hover", "pressed", "disabled"] {
            arrow.add_theme_stylebox_override(&StringName::from(state), &transparent.clone());
        }
        arrow.add_theme_color_override(
            &StringName::from("font_color"),
            self.item_text,
        );
        row.add_child(&arrow);

        // 行文字标签
        let mut lbl = Label::new_alloc();
        lbl.set_name(&StringName::from("__MenuText"));
        lbl.set_text(&GString::from(text.as_str()));
        lbl.add_theme_font_size_override(&StringName::from("font_size"), font_size);
        lbl.set_vertical_alignment(godot::global::VerticalAlignment::CENTER);
        lbl.set_mouse_filter(godot::classes::control::MouseFilter::IGNORE);
        row.add_child(&lbl);

        // 挂到组件根
        let mut row_node: Gd<Control> = row.clone().upcast();
        self.base_mut().add_child(&row_node);
        if set_owner {
            row_node.set_owner(&*self.base());
            arrow.set_owner(&row_node);
            lbl.set_owner(&row_node);
        }

        self.nodes[idx].row = Some(row);
        self.wire_row(idx);
        self.apply_row_style(idx);
    }

    /// 连接行/箭头信号（幂等）
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
        let arrow = row.get_node_or_null(&NodePath::from("__MenuArrow"));
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

    /// 运行期从 tscn 序列化行重建模型（两遍法，同 Tree）
    fn rebuild_from_rows(&mut self) -> bool {
        let children = self.base().get_children();
        let mut rows: Vec<Gd<Button>> = Vec::new();
        for i in 0..children.len() {
            if let Some(child) = children.get(i) {
                if let Ok(ctrl) = child.clone().try_cast::<Control>() {
                    if ctrl.has_meta(&StringName::from("__menu_row")) {
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

        let mut id_map: HashMap<String, usize> = HashMap::new();
        let mut pending_parent: Vec<String> = Vec::new();
        for row in &rows {
            let mut data = MenuNodeData::new();
            data.id = if row.has_meta(&StringName::from("__menu_id")) {
                row.get_meta(&StringName::from("__menu_id")).to_string()
            } else {
                format!("m{}", self.nodes.len())
            };
            data.text = if row.has_meta(&StringName::from("__menu_text")) {
                row.get_meta(&StringName::from("__menu_text")).to_string()
            } else {
                String::new()
            };
            data.expanded = if row.has_meta(&StringName::from("__menu_expanded")) {
                row.get_meta(&StringName::from("__menu_expanded")).to::<bool>()
            } else {
                true
            };
            let pid = if row.has_meta(&StringName::from("__menu_parent")) {
                row.get_meta(&StringName::from("__menu_parent")).to_string()
            } else {
                String::new()
            };
            data.row = Some(row.clone());
            let idx = self.nodes.len();
            id_map.insert(data.id.clone(), idx);
            self.nodes.push(data);
            pending_parent.push(pid);
        }
        // depth + 父子链接
        for i in 0..self.nodes.len() {
            let pid = pending_parent[i].clone();
            if pid.is_empty() {
                self.nodes[i].parent = -1;
                self.nodes[i].depth = 0;
            } else if let Some(&pi) = id_map.get(&pid) {
                self.nodes[i].parent = pi as i32;
                self.nodes[i].depth = self.nodes[pi].depth + 1;
                self.nodes[pi].children.push(i);
            } else {
                self.nodes[i].parent = -1;
                self.nodes[i].depth = 0;
            }
        }
        self.id_index = id_map;

        for idx in 0..self.nodes.len() {
            self.wire_row(idx);
            self.apply_row_style(idx);
        }
        true
    }

    // ------------------------------------------------------------------
    // 布局与样式
    // ------------------------------------------------------------------

    /// 祖先链全部展开才可见
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

    fn available_width(&self) -> f32 {
        let base = self.base();
        if !base.is_inside_tree() {
            return 0.0;
        }
        if let Some(parent) = base.get_parent() {
            if let Ok(scroll) = parent.try_cast::<ScrollContainer>() {
                return scroll.get_size().x;
            }
        }
        base.get_viewport_rect().size.x
    }

    /// 全量重排：可见行依序垂直排布，行内按深度缩进
    #[func]
    pub fn relayout(&mut self) {
        self.layout_all();
    }

    fn layout_all(&mut self) {
        if self.nodes.is_empty() {
            return;
        }
        // 先收集可见序（前序遍历）
        let mut order: Vec<usize> = Vec::new();
        let roots: Vec<usize> = (0..self.nodes.len())
            .filter(|&i| self.nodes[i].parent < 0)
            .collect();
        fn walk(nodes: &[MenuNodeData], idx: usize, out: &mut Vec<usize>) {
            out.push(idx);
            if nodes[idx].expanded {
                for &c in &nodes[idx].children {
                    walk(nodes, c, out);
                }
            }
        }
        for &r in &roots {
            walk(&self.nodes, r, &mut order);
        }

        let avail_w = self.available_width();
        let outer_w = if avail_w > 1.0 { avail_w } else { 240.0 };
        let pitch = self.row_height + self.row_gap;
        let total_h = order.len() as f32 * pitch - self.row_gap;

        // 行位置（行本身占满宽，缩进发生在行内文字/箭头上）
        let mut y = 0.0_f32;
        let mut vis: Vec<bool> = vec![false; self.nodes.len()];
        for &idx in &order {
            vis[idx] = true;
            self.nodes[idx].row.as_ref().map(|row| {
                let mut row = row.clone();
                row.set_position(Vector2::new(0.0, y));
                row.set_size(Vector2::new(outer_w, self.row_height));
            });
            // 行内元素
            let (depth, has_children, expanded, text, indent, row_h, font_size) = {
                let n = &self.nodes[idx];
                (
                    n.depth,
                    n.has_children(),
                    n.expanded,
                    n.text.clone(),
                    self.indent_width,
                    self.row_height,
                    self.font_size,
                )
            };
            let text_x = depth as f32 * indent + ARROW_SLOT + 2.0;
            if let Some(row) = self.nodes[idx].row.clone() {
                if row.is_instance_valid() {
                    if let Some(a) = row.get_node_or_null(&NodePath::from("__MenuArrow")) {
                        if let Ok(mut ab) = a.try_cast::<Button>() {
                            ab.set_visible(has_children);
                            ab.set_text(&GString::from(if expanded { "▾" } else { "▸" }));
                            ab.set_position(Vector2::new(
                                depth as f32 * indent,
                                (row_h - 18.0) / 2.0,
                            ));
                            ab.set_size(Vector2::new(ARROW_SLOT, row_h));
                        }
                    }
                    if let Some(l) = row.get_node_or_null(&NodePath::from("__MenuText")) {
                        if let Ok(mut lbl) = l.try_cast::<Label>() {
                            lbl.set_text(&GString::from(text.as_str()));
                            lbl.set_position(Vector2::new(text_x, 0.0));
                            lbl.set_size(Vector2::new(
                                (outer_w - text_x - 8.0).max(40.0),
                                row_h,
                            ));
                            lbl.add_theme_font_size_override(
                                &StringName::from("font_size"),
                                font_size,
                            );
                        }
                    }
                }
            }
            y += pitch;
        }

        // 折叠子树隐藏
        for (idx, v) in vis.iter().enumerate() {
            if let Some(mut row) = self.nodes[idx].row.clone() {
                if row.is_instance_valid() {
                    row.set_visible(*v);
                }
            }
        }

        let mut base = self.base_mut();
        base.set_custom_minimum_size(Vector2::new(outer_w, total_h.max(0.0)));
        base.set_size(Vector2::new(outer_w, total_h.max(0.0)));
    }

    /// 行样式：常规 / 悬停 / 选中三态
    fn apply_row_style(&mut self, idx: usize) {
        let selected = self.selected.as_deref() == Some(self.nodes[idx].id.as_str());
        let (bg, txt) = if selected {
            (self.selected_bg, self.selected_text)
        } else {
            (self.item_bg, self.item_text)
        };
        let hover = if selected {
            self.selected_bg
        } else {
            self.item_hover_bg
        };

        let Some(row) = self.nodes[idx].row.clone() else { return };
        if !row.is_instance_valid() {
            return;
        }
        let mut row = row;
        let mut sb = StyleBoxFlat::new_gd();
        sb.set_bg_color(bg);
        sb.set_corner_radius_all(6);
        row.add_theme_stylebox_override(&StringName::from("normal"), &sb.clone());
        let mut hb = StyleBoxFlat::new_gd();
        hb.set_bg_color(hover);
        hb.set_corner_radius_all(6);
        row.add_theme_stylebox_override(&StringName::from("hover"), &hb.clone());
        row.add_theme_stylebox_override(&StringName::from("pressed"), &sb);
        row.add_theme_color_override(&StringName::from("font_color"), txt);
        row.add_theme_color_override(&StringName::from("font_hover_color"), txt);
        row.add_theme_color_override(&StringName::from("font_pressed_color"), txt);
        // 文字颜色
        if let Some(l) = row.get_node_or_null(&NodePath::from("__MenuText")) {
            if let Ok(mut lbl) = l.try_cast::<Label>() {
                lbl.add_theme_color_override(&StringName::from("font_color"), txt);
            }
        }
        if let Some(a) = row.get_node_or_null(&NodePath::from("__MenuArrow")) {
            if let Ok(mut ab) = a.try_cast::<Button>() {
                ab.add_theme_color_override(&StringName::from("font_color"), txt);
                ab.add_theme_color_override(&StringName::from("font_hover_color"), txt);
                ab.add_theme_color_override(&StringName::from("font_pressed_color"), txt);
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

    /// 行点击：选中 + 发信号；有子菜单时同时折叠/展开
    #[func]
    fn _on_row_pressed(&mut self, idx: i64) {
        let idx = idx as usize;
        if idx >= self.nodes.len() {
            return;
        }
        self.select_item(GString::from(self.nodes[idx].id.as_str()), true);
        if self.nodes[idx].has_children() {
            self.toggle_expand(GString::from(self.nodes[idx].id.as_str()));
        }
    }

    /// 箭头点击：仅折叠/展开
    #[func]
    fn _on_arrow_pressed(&mut self, idx: i64) {
        let idx = idx as usize;
        if idx >= self.nodes.len() {
            return;
        }
        self.toggle_expand(GString::from(self.nodes[idx].id.as_str()));
    }

    // ------------------------------------------------------------------
    // 公开 API
    // ------------------------------------------------------------------

    /// 选中菜单项（emit=true 发 s_menu_selected）
    #[func]
    pub fn select_item(&mut self, id: GString, emit: bool) -> bool {
        let Some(&idx) = self.id_index.get(id.to_string().as_str()) else {
            return false;
        };
        let new_id = self.nodes[idx].id.clone();
        let changed = self.selected.as_deref() != Some(new_id.as_str());
        self.selected = Some(new_id);
        if changed {
            self.refresh_all_styles();
        }
        if emit {
            let gid = GString::from(self.nodes[idx].id.as_str());
            self.base_mut()
                .emit_signal(&StringName::from("s_menu_selected"), &[gid.to_variant()]);
        }
        true
    }

    #[func]
    pub fn get_selected_id(&self) -> GString {
        match &self.selected {
            Some(id) => GString::from(id.as_str()),
            None => GString::new(),
        }
    }

    #[func]
    pub fn is_item_expanded(&self, id: GString) -> bool {
        match self.id_index.get(id.to_string().as_str()) {
            Some(&idx) => self.nodes[idx].expanded,
            None => false,
        }
    }

    /// 折叠/展开指定项
    #[func]
    pub fn toggle_expand(&mut self, id: GString) -> bool {
        let Some(&idx) = self.id_index.get(id.to_string().as_str()) else {
            return false;
        };
        self.nodes[idx].expanded = !self.nodes[idx].expanded;
        self.sync_row_meta(idx);
        self.apply_row_style(idx);
        self.layout_all();
        true
    }

    #[func]
    pub fn expand_all(&mut self) {
        for idx in 0..self.nodes.len() {
            if self.nodes[idx].has_children() && !self.nodes[idx].expanded {
                self.nodes[idx].expanded = true;
                self.sync_row_meta(idx);
            }
        }
        self.refresh_all_styles();
        self.layout_all();
    }

    #[func]
    pub fn collapse_all(&mut self) {
        for idx in 0..self.nodes.len() {
            if self.nodes[idx].has_children() && self.nodes[idx].expanded {
                self.nodes[idx].expanded = false;
                self.sync_row_meta(idx);
            }
        }
        self.refresh_all_styles();
        self.layout_all();
    }

    /// 改菜单项文字（改名避开 item_text 字段的自动 setter set_item_text）
    #[func]
    pub fn rename_item_text(&mut self, id: GString, text: GString) -> bool {
        let Some(&idx) = self.id_index.get(id.to_string().as_str()) else {
            return false;
        };
        self.nodes[idx].text = text.to_string();
        self.sync_row_meta(idx);
        self.layout_all();
        true
    }

    /// 菜单项数量（测试用）
    #[func]
    pub fn get_item_count(&self) -> i32 {
        self.nodes.len() as i32
    }

    /// 行 meta 与模型同步
    fn sync_row_meta(&mut self, idx: usize) {
        let Some(row) = self.nodes[idx].row.clone() else { return };
        if !row.is_instance_valid() {
            return;
        }
        let mut row = row;
        let n = &self.nodes[idx];
        row.set_meta(
            &StringName::from("__menu_expanded"),
            &n.expanded.to_variant(),
        );
        row.set_meta(
            &StringName::from("__menu_text"),
            &GString::from(n.text.as_str()).to_variant(),
        );
    }

    /// 选中状态内部快照（测试用）
    #[func]
    pub fn dump_state(&self) -> VarDict {
        let mut d = VarDict::new();
        d.set(&Variant::from("selected"), &Variant::from(match &self.selected {
            Some(id) => GString::from(id.as_str()),
            None => GString::new(),
        }));
        let mut expanded = VarDict::new();
        for n in &self.nodes {
            expanded.set(
                &Variant::from(GString::from(n.id.as_str())),
                &Variant::from(n.expanded),
            );
        }
        d.set(&Variant::from("expanded"), &Variant::from(expanded));
        d
    }
}
