// GdUINavMenu - 导航菜单节点
// 继承 Control，支持多级级联菜单
// GML 标签：<NavMenu direction="left" menu_width="160">
//   <NavItem text="Audio">
//     <NavItem text="Volume">
//       <NavItem text="Master" />
//       <NavItem text="Music" />
//     </NavItem>
//     <NavItem text="Output" />
//   </NavItem>
// </NavMenu>
// NavItem 递归嵌套，有子 NavItem 则点击展开下一级，无子 NavItem 则触发 s_item_clicked 信号

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Side, Variant, Vector2};
use godot::classes::{
    IControl, Control, PanelContainer, VBoxContainer,
    Button, MarginContainer, ColorRect, StyleBoxFlat,
    InputEvent, InputEventMouseButton,
};
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::global::MouseButton;
use godot::obj::{WithBaseField, BaseMut};

/// 菜单方向
const DIR_LEFT: i32 = 0;
const DIR_RIGHT: i32 = 1;

/// 递归菜单项数据
#[derive(Clone)]
struct NavItemData {
    text: String,
    signal_binding: Option<String>,
    children: Vec<NavItemData>,
}

impl NavItemData {
    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

/// 面板信息：每级菜单对应一个面板
struct PanelInfo {
    panel: Gd<PanelContainer>,
    margin: Gd<MarginContainer>,
    vbox: Gd<VBoxContainer>,
    buttons: Vec<Gd<Button>>,
    /// 当前该面板中激活的项索引（-1 表示无）
    active_index: i32,
    /// 动画进度 0.0~1.0
    anim_progress: f64,
    /// 是否正在动画
    animating: bool,
    /// 是否正在打开（false = 正在关闭）
    opening: bool,
}

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdUINavMenu {
    base: Base<Control>,

    #[export]
    direction: i32,
    #[export]
    menu_width: i32,
    #[export]
    sub_menu_width: i32,
    #[export]
    menu_bg_color: Color,
    #[export]
    menu_border_color: Color,
    #[export]
    overlay_color: Color,
    #[export]
    corner_radius: i32,
    #[export]
    animation_duration: f64,
    #[export]
    close_on_overlay: bool,
    #[export]
    item_font_size: i32,
    #[export]
    item_color: Color,
    #[export]
    item_hover_color: Color,
    #[export]
    item_active_color: Color,
    #[export]
    sub_item_font_size: i32,
    #[export]
    sub_item_color: Color,
    #[export]
    sub_item_hover_color: Color,

    // 内部节点引用
    overlay: Option<Gd<ColorRect>>,
    /// 多级面板列表，panels[0] 是一级菜单，panels[1+] 是子级
    panels: Vec<PanelInfo>,

    // 状态
    is_open: bool,
    /// 主菜单动画进度
    main_anim_progress: f64,
    main_animating: bool,
    main_opening: bool,
    ui_built: bool,

    // 菜单数据
    menu_data: Vec<NavItemData>,
}

#[godot_api]
impl IControl for GdUINavMenu {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            direction: DIR_LEFT,
            menu_width: 160,
            sub_menu_width: 180,
            menu_bg_color: Color::from_rgba(0.08, 0.08, 0.14, 0.97),
            menu_border_color: Color::from_rgb(0.35, 0.4, 0.55),
            overlay_color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            corner_radius: 0,
            animation_duration: 0.2,
            close_on_overlay: true,
            item_font_size: 16,
            item_color: Color::from_rgb(0.85, 0.85, 0.9),
            item_hover_color: Color::from_rgb(1.0, 1.0, 1.0),
            item_active_color: Color::from_rgb(0.4, 0.8, 1.0),
            sub_item_font_size: 14,
            sub_item_color: Color::from_rgb(0.75, 0.75, 0.8),
            sub_item_hover_color: Color::from_rgb(1.0, 1.0, 1.0),
            overlay: None,
            panels: Vec::new(),
            is_open: false,
            main_anim_progress: 0.0,
            main_animating: false,
            main_opening: false,
            ui_built: false,
            menu_data: Vec::new(),
        }
    }

    fn ready(&mut self) {
        if !self.ui_built {
            self.parse_children();
            self.build_ui();
        }
        self.is_open = false;
        self.main_animating = false;
        self.main_anim_progress = 0.0;
        self.base_mut().set_visible(false);
    }

    fn process(&mut self, delta: f64) {
        let speed = if self.animation_duration > 0.0 {
            1.0 / self.animation_duration
        } else {
            100.0
        };

        // 主菜单动画
        if self.main_animating {
            if self.main_opening {
                self.main_anim_progress = (self.main_anim_progress + delta * speed).min(1.0);
                if self.main_anim_progress >= 1.0 {
                    self.main_animating = false;
                    self.base_mut().emit_signal(&StringName::from("s_menu_opened"), &[]);
                }
            } else {
                self.main_anim_progress = (self.main_anim_progress - delta * speed).max(0.0);
                if self.main_anim_progress <= 0.0 {
                    self.main_animating = false;
                    self.base_mut().set_visible(false);
                    self.base_mut().emit_signal(&StringName::from("s_menu_closed"), &[]);
                }
            }
            self.apply_main_animation();
        }

        // 各级子面板动画
        // 先收集需要动画的面板信息
        let mut panel_anim_data: Vec<(usize, f32, bool, bool)> = Vec::new(); // (index, progress, opening, animating)
        for (i, panel_info) in self.panels.iter_mut().enumerate() {
            if panel_info.animating {
                if panel_info.opening {
                    panel_info.anim_progress = (panel_info.anim_progress + delta * speed).min(1.0);
                    if panel_info.anim_progress >= 1.0 {
                        panel_info.animating = false;
                    }
                } else {
                    panel_info.anim_progress = (panel_info.anim_progress - delta * speed).max(0.0);
                    if panel_info.anim_progress <= 0.0 {
                        panel_info.animating = false;
                        panel_info.panel.clone().set_visible(false);
                    }
                }
                panel_anim_data.push((i, panel_info.anim_progress as f32, panel_info.opening, panel_info.animating));
            }
        }
        // 应用动画
        for (idx, progress, _opening, _animating) in &panel_anim_data {
            self.apply_panel_animation_for(*idx, *progress);
        }
    }
}

#[godot_api]
impl GdUINavMenu {
    #[signal]
    fn s_menu_opened();

    #[signal]
    fn s_menu_closed();

    /// 叶子项被点击时触发，path 为从根到叶的索引路径（如 "0,1,2"）
    #[signal]
    fn s_item_clicked(path: GString);

    /// 打开菜单
    #[func]
    fn open(&mut self) {
        if self.is_open {
            return;
        }
        self.is_open = true;
        self.main_opening = true;
        self.main_animating = true;
        self.base_mut().set_visible(true);
        if let Some(ref overlay) = self.overlay {
            let mut o = overlay.clone();
            o.set_color(Color::from_rgba(
                self.overlay_color.r,
                self.overlay_color.g,
                self.overlay_color.b,
                0.0,
            ));
        }
    }

    /// 关闭菜单
    #[func]
    fn close(&mut self) {
        if !self.is_open && !self.main_animating {
            return;
        }
        self.is_open = false;
        self.main_opening = false;
        self.main_animating = true;
        // 关闭所有子面板
        self.collapse_all_sub_panels();
    }

    /// 切换菜单
    #[func]
    fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// 菜单是否打开
    #[func]
    fn is_menu_open(&self) -> bool {
        self.is_open
    }

    /// 确保内部 UI 已构建（供 builder 调用）
    #[func]
    fn ensure_ui_built(&mut self) {
        if !self.ui_built {
            self.parse_children();
            self.build_ui();
        }
    }

    /// 重新计算菜单布局（menu_width/sub_menu_width 变化后调用）
    /// 更新所有面板的锚点偏移使其匹配新的宽度
    #[func]
    fn update_layout(&mut self) {
        let menu_width = self.menu_width;
        let sub_menu_width = self.sub_menu_width;
        let direction = self.direction;

        for (level, panel_info) in self.panels.iter().enumerate() {
            let mut panel = panel_info.panel.clone();
            let (left_offset, right_offset) = if level == 0 {
                (0.0, menu_width as f32)
            } else {
                let l = (menu_width + sub_menu_width * (level as i32 - 1)) as f32;
                let r = (menu_width + sub_menu_width * level as i32) as f32;
                (l, r)
            };

            match direction {
                DIR_LEFT => {
                    panel.set_anchor_and_offset(Side::LEFT, 0.0, left_offset);
                    panel.set_anchor_and_offset(Side::RIGHT, 0.0, right_offset);
                }
                DIR_RIGHT => {
                    panel.set_anchor_and_offset(Side::LEFT, 1.0, -right_offset);
                    panel.set_anchor_and_offset(Side::RIGHT, 1.0, -left_offset);
                }
                _ => {}
            }
        }
    }

    /// 处理遮罩点击
    #[func]
    fn _on_overlay_gui_input(&mut self, event: Gd<InputEvent>) {
        if !self.close_on_overlay {
            return;
        }
        if let Ok(mouse_btn) = event.try_cast::<InputEventMouseButton>() {
            if mouse_btn.get_button_index() == MouseButton::LEFT && mouse_btn.is_pressed() {
                self.close();
            }
        }
    }

    /// 处理一级菜单项点击
    #[func]
    fn _on_level0_pressed(&mut self, item_index: i32) {
        self.on_item_pressed(0, item_index);
    }

    /// 处理子级菜单项点击（level 从 1 开始）
    #[func]
    fn _on_sub_level_pressed(&mut self, level: i32, item_index: i32) {
        self.on_item_pressed(level as usize, item_index);
    }
}

impl GdUINavMenu {
    /// 递归解析子节点（NavItem）提取菜单数据
    fn parse_children(&mut self) {
        let children = self.base().get_children();
        let mut items_to_remove: Vec<Gd<Control>> = Vec::new();

        for i in 0..children.len() {
            if let Some(child) = children.get(i) {
                if let Ok(ctrl) = child.clone().try_cast::<Control>() {
                    if ctrl.has_meta(&StringName::from("__nav_item")) {
                        if let Some(item_data) = Self::parse_nav_item(&ctrl) {
                            self.menu_data.push(item_data);
                        }
                        items_to_remove.push(ctrl);
                    }
                }
            }
        }

        // 移除已解析的数据节点
        for mut ctrl in items_to_remove {
            self.base_mut().remove_child(&ctrl);
            ctrl.queue_free();
        }
    }

    /// 递归解析单个 NavItem 节点
    fn parse_nav_item(ctrl: &Gd<Control>) -> Option<NavItemData> {
        let text = if ctrl.has_meta(&StringName::from("__nav_text")) {
            ctrl.get_meta(&StringName::from("__nav_text")).to_string()
        } else {
            ctrl.get_name().to_string()
        };

        let signal_binding = if ctrl.has_meta(&StringName::from("__signal_pressed")) {
            Some(ctrl.get_meta(&StringName::from("__signal_pressed")).to_string())
        } else {
            None
        };

        let mut children: Vec<NavItemData> = Vec::new();
        let sub_children = ctrl.get_children();
        for i in 0..sub_children.len() {
            if let Some(sub_child) = sub_children.get(i) {
                if let Ok(sub_ctrl) = sub_child.clone().try_cast::<Control>() {
                    if sub_ctrl.has_meta(&StringName::from("__nav_item")) {
                        if let Some(child_data) = Self::parse_nav_item(&sub_ctrl) {
                            children.push(child_data);
                        }
                    }
                }
            }
        }

        Some(NavItemData {
            text,
            signal_binding,
            children,
        })
    }

    /// 构建视觉 UI
    fn build_ui(&mut self) {
        if self.ui_built {
            return;
        }
        self.ui_built = true;

        // 先读取属性值，避免借用冲突
        let overlay_color = self.overlay_color;
        let menu_width = self.menu_width;
        let direction = self.direction;
        let menu_bg_color = self.menu_bg_color;
        let menu_border_color = self.menu_border_color;
        let corner_radius = self.corner_radius;
        let item_font_size = self.item_font_size;
        let item_color = self.item_color;
        let item_hover_color = self.item_hover_color;
        let item_active_color = self.item_active_color;
        let menu_data = self.menu_data.clone();

        let mut overlay_node: Option<Gd<ColorRect>> = None;
        let mut first_panel_info: Option<PanelInfo> = None;

        {
            let mut base = self.base_mut();
            base.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);

            // 遮罩层
            let mut overlay = ColorRect::new_alloc();
            overlay.set_name("Overlay");
            overlay.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
            overlay.set_color(overlay_color);
            overlay.set_mouse_filter(MouseFilter::STOP);
            let overlay_cb = Callable::from_object_method(&*base, "_on_overlay_gui_input");
            overlay.connect(&StringName::from("gui_input"), &overlay_cb);
            base.add_child(&overlay);
            overlay_node = Some(overlay);

            // 一级菜单面板
            let (panel, margin, mut vbox, _buttons) = Self::create_panel(
                &mut base,
                "Level0Panel",
                direction,
                0, // level
                menu_width,
                0, // sub_menu_width not used for level 0
                menu_bg_color,
                menu_border_color,
                corner_radius,
            );

            // 构建一级菜单按钮
            let item_hover_bg = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
            let mut btn_nodes: Vec<Gd<Button>> = Vec::new();

            for (i, item) in menu_data.iter().enumerate() {
                let has_children = !item.is_leaf();
                let mut btn = Self::create_menu_button(
                    &item.text,
                    item_font_size,
                    item_color,
                    item_hover_color,
                    item_active_color,
                    item_hover_bg,
                    has_children,
                );

                // 连接信号
                let item_index = i as i32;
                let callable = Callable::from_object_method(&*base, "_on_level0_pressed")
                    .bind(&[Variant::from(item_index)]);
                btn.connect(&StringName::from("pressed"), &callable);

                vbox.add_child(&btn);
                btn_nodes.push(btn);
            }

            first_panel_info = Some(PanelInfo {
                panel,
                margin,
                vbox,
                buttons: btn_nodes,
                active_index: -1,
                anim_progress: 1.0, // 一级面板跟随主动画
                animating: false,
                opening: false,
            });
        }

        self.overlay = overlay_node;
        if let Some(pi) = first_panel_info {
            self.panels.push(pi);
        }
    }

    /// 创建一个面板（PanelContainer + MarginContainer + VBoxContainer）
    fn create_panel(
        base: &mut BaseMut<'_, GdUINavMenu>,
        name: &str,
        direction: i32,
        level: usize,
        menu_width: i32,
        sub_menu_width: i32,
        bg_color: Color,
        border_color: Color,
        corner_radius: i32,
    ) -> (Gd<PanelContainer>, Gd<MarginContainer>, Gd<VBoxContainer>, Vec<Gd<Button>>) {
        let mut panel = PanelContainer::new_alloc();
        panel.set_name(&StringName::from(name));

        // 计算面板位置
        // Level 0: LEFT=0, RIGHT=menu_width
        // Level N (N>=1): LEFT=menu_width+sub_menu_width*(N-1), RIGHT=menu_width+sub_menu_width*N
        let (left_offset, right_offset) = if level == 0 {
            (0.0, menu_width as f32)
        } else {
            let l = (menu_width + sub_menu_width * (level as i32 - 1)) as f32;
            let r = (menu_width + sub_menu_width * level as i32) as f32;
            (l, r)
        };

        match direction {
            DIR_LEFT => {
                panel.set_anchor_and_offset(Side::LEFT, 0.0, left_offset);
                panel.set_anchor_and_offset(Side::RIGHT, 0.0, right_offset);
                panel.set_anchor_and_offset(Side::TOP, 0.0, 0.0);
                panel.set_anchor_and_offset(Side::BOTTOM, 1.0, 0.0);
            }
            DIR_RIGHT => {
                panel.set_anchor_and_offset(Side::LEFT, 1.0, -right_offset);
                panel.set_anchor_and_offset(Side::RIGHT, 1.0, -left_offset);
                panel.set_anchor_and_offset(Side::TOP, 0.0, 0.0);
                panel.set_anchor_and_offset(Side::BOTTOM, 1.0, 0.0);
            }
            _ => {}
        }

        if level > 0 {
            panel.set_visible(false);
        }

        let mut style = StyleBoxFlat::new_gd();
        style.set_bg_color(bg_color);
        style.set_border_color(border_color);
        style.set_border_width_all(1);
        style.set_corner_radius_all(corner_radius);
        panel.add_theme_stylebox_override(&StringName::from("panel"), &style);
        base.add_child(&panel);

        // MarginContainer
        let mut margin = MarginContainer::new_alloc();
        margin.add_theme_constant_override(&StringName::from("margin_top"), 16);
        margin.add_theme_constant_override(&StringName::from("margin_bottom"), 16);
        margin.add_theme_constant_override(&StringName::from("margin_left"), 12);
        margin.add_theme_constant_override(&StringName::from("margin_right"), 12);
        margin.set_v_size_flags(SizeFlags::EXPAND_FILL);
        panel.add_child(&margin);

        // VBoxContainer
        let mut vbox = VBoxContainer::new_alloc();
        let vbox_name = format!("Level{}VBox", level);
        vbox.set_name(&StringName::from(&vbox_name));
        vbox.add_theme_constant_override(&StringName::from("separation"), 4);
        vbox.set_v_size_flags(SizeFlags::SHRINK_CENTER);
        margin.add_child(&vbox);

        (panel, margin, vbox, Vec::new())
    }

    /// 创建菜单按钮
    fn create_menu_button(
        text: &str,
        font_size: i32,
        color: Color,
        hover_color: Color,
        active_color: Color,
        hover_bg: Color,
        has_children: bool,
    ) -> Gd<Button> {
        let display_text = if has_children {
            format!("{}  ▸", text)
        } else {
            text.to_string()
        };

        let mut btn = Button::new_alloc();
        btn.set_text(&GString::from(&display_text));
        btn.add_theme_font_size_override(&StringName::from("font_size"), font_size);
        btn.add_theme_color_override(&StringName::from("font_color"), color);
        btn.add_theme_color_override(&StringName::from("font_hover_color"), hover_color);
        btn.add_theme_color_override(&StringName::from("font_pressed_color"), active_color);
        btn.set_custom_minimum_size(Vector2::new(0.0, 36.0));
        btn.set_h_size_flags(SizeFlags::EXPAND_FILL);

        // 透明背景样式
        let mut normal_style = StyleBoxFlat::new_gd();
        normal_style.set_bg_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        btn.add_theme_stylebox_override(&StringName::from("normal"), &normal_style);

        let mut hover_style = StyleBoxFlat::new_gd();
        hover_style.set_bg_color(hover_bg);
        btn.add_theme_stylebox_override(&StringName::from("hover"), &hover_style);

        let mut pressed_style = StyleBoxFlat::new_gd();
        pressed_style.set_bg_color(Color::from_rgba(0.4, 0.8, 1.0, 0.12));
        btn.add_theme_stylebox_override(&StringName::from("pressed"), &pressed_style);

        btn
    }

    /// 处理菜单项点击
    fn on_item_pressed(&mut self, level: usize, item_index: i32) {
        // 获取点击项对应的数据
        let item_data = match self.get_item_data(level, item_index) {
            Some(data) => data.clone(),
            None => return,
        };

        if level >= self.panels.len() {
            return;
        }

        let was_active = self.panels[level].active_index == item_index;

        // 更新当前级别的激活状态
        self.update_active_style(level, item_index);

        if item_data.is_leaf() {
            // 叶子项：发出信号
            let path = self.build_path(level, item_index);
            self.base_mut().emit_signal(
                &StringName::from("s_item_clicked"),
                &[Variant::from(GString::from(&path))],
            );
        } else if was_active {
            // 再次点击同一项：收起子面板
            self.collapse_from_level(level + 1);
        } else {
            // 展开子面板
            self.expand_sub_panel(level, item_index, &item_data);
        }
    }

    /// 获取指定级别和索引的菜单项数据
    fn get_item_data(&self, level: usize, item_index: i32) -> Option<&NavItemData> {
        if level == 0 {
            self.menu_data.get(item_index as usize)
        } else {
            // 沿 active_path 找到对应的数据
            let mut current: &Vec<NavItemData> = &self.menu_data;
            for l in 0..level {
                if l >= self.panels.len() {
                    return None;
                }
                let active = self.panels[l].active_index;
                if active < 0 {
                    return None;
                }
                if let Some(item) = current.get(active as usize) {
                    current = &item.children;
                } else {
                    return None;
                }
            }
            current.get(item_index as usize)
        }
    }

    /// 构建路径字符串（如 "0,1,2"）
    fn build_path(&self, level: usize, item_index: i32) -> String {
        let mut parts: Vec<String> = Vec::new();
        for l in 0..level {
            if l < self.panels.len() && self.panels[l].active_index >= 0 {
                parts.push(self.panels[l].active_index.to_string());
            }
        }
        parts.push(item_index.to_string());
        parts.join(",")
    }

    /// 更新指定级别面板的激活样式
    fn update_active_style(&mut self, level: usize, new_active: i32) {
        if level >= self.panels.len() {
            return;
        }

        let active_color = if level == 0 {
            self.item_active_color
        } else {
            self.item_active_color
        };
        let normal_color = if level == 0 {
            self.item_color
        } else {
            self.sub_item_color
        };

        let old_active = self.panels[level].active_index;
        self.panels[level].active_index = new_active;

        // 更新旧激活项样式
        if old_active >= 0 && (old_active as usize) < self.panels[level].buttons.len() {
            let mut btn = self.panels[level].buttons[old_active as usize].clone();
            btn.add_theme_color_override(&StringName::from("font_color"), normal_color);
        }

        // 更新新激活项样式
        if new_active >= 0 && (new_active as usize) < self.panels[level].buttons.len() {
            let mut btn = self.panels[level].buttons[new_active as usize].clone();
            btn.add_theme_color_override(&StringName::from("font_color"), active_color);
        }
    }

    /// 展开子面板
    fn expand_sub_panel(&mut self, level: usize, _item_index: i32, item_data: &NavItemData) {
        // 先收起该级别之后的所有面板
        self.collapse_from_level(level + 1);

        // 读取属性值
        let direction = self.direction;
        let menu_width = self.menu_width;
        let sub_menu_width = self.sub_menu_width;
        let menu_bg_color = self.menu_bg_color;
        let menu_border_color = self.menu_border_color;
        let corner_radius = self.corner_radius;
        let sub_item_font_size = self.sub_item_font_size;
        let sub_item_color = self.sub_item_color;
        let sub_item_hover_color = self.sub_item_hover_color;
        let item_active_color = self.item_active_color;
        let sub_item_hover_bg = Color::from_rgba(1.0, 1.0, 1.0, 0.06);

        let next_level = level + 1;

        // 如果已有该级别的面板，复用
        if next_level < self.panels.len() {
            // 先收集旧按钮，稍后释放
            let old_buttons: Vec<Gd<Button>> = self.panels[next_level].buttons.drain(..).collect();
            self.panels[next_level].active_index = -1;

            // 创建新按钮（需要 base 引用来连接信号）
            let mut new_buttons: Vec<Gd<Button>> = Vec::new();
            {
                let base_ref = self.base().clone();
                for (j, child) in item_data.children.iter().enumerate() {
                    let has_children = !child.is_leaf();
                    let mut btn = Self::create_menu_button(
                        &child.text,
                        sub_item_font_size,
                        sub_item_color,
                        sub_item_hover_color,
                        item_active_color,
                        sub_item_hover_bg,
                        has_children,
                    );

                    let level_idx = next_level as i32;
                    let item_idx = j as i32;
                    let callable = Callable::from_object_method(&base_ref, "_on_sub_level_pressed")
                        .bind(&[Variant::from(level_idx), Variant::from(item_idx)]);
                    btn.connect(&StringName::from("pressed"), &callable);

                    if let Some(ref binding) = child.signal_binding {
                        btn.set_meta(
                            &StringName::from("__signal_pressed"),
                            &binding.to_variant(),
                        );
                    }

                    new_buttons.push(btn);
                }
            }

            // 添加新按钮到 vbox
            let mut vbox = self.panels[next_level].vbox.clone();
            for btn in &new_buttons {
                vbox.add_child(btn);
            }
            self.panels[next_level].buttons = new_buttons;

            // 释放旧按钮
            for mut btn in old_buttons {
                vbox.remove_child(&btn);
                btn.queue_free();
            }

            // 显示并开始动画
            self.panels[next_level].panel.clone().set_visible(true);
            self.panels[next_level].anim_progress = 0.0;
            self.panels[next_level].animating = true;
            self.panels[next_level].opening = true;
        } else {
            // 创建新面板
            let mut btn_nodes: Vec<Gd<Button>> = Vec::new();
            let base_ref = self.base().clone();

            let (mut panel, margin, mut vbox, _) = {
                let mut base = self.base_mut();
                let result = Self::create_panel(
                    &mut base,
                    &format!("Level{}Panel", next_level),
                    direction,
                    next_level,
                    menu_width,
                    sub_menu_width,
                    menu_bg_color,
                    menu_border_color,
                    corner_radius,
                );
                result
            };

            // 创建按钮（base_mut 已释放）
            for (j, child) in item_data.children.iter().enumerate() {
                let has_children = !child.is_leaf();
                let mut btn = Self::create_menu_button(
                    &child.text,
                    sub_item_font_size,
                    sub_item_color,
                    sub_item_hover_color,
                    item_active_color,
                    sub_item_hover_bg,
                    has_children,
                );

                let level_idx = next_level as i32;
                let item_idx = j as i32;
                let callable = Callable::from_object_method(&base_ref, "_on_sub_level_pressed")
                    .bind(&[Variant::from(level_idx), Variant::from(item_idx)]);
                btn.connect(&StringName::from("pressed"), &callable);

                if let Some(ref binding) = child.signal_binding {
                    btn.set_meta(
                        &StringName::from("__signal_pressed"),
                        &binding.to_variant(),
                    );
                }

                vbox.add_child(&btn);
                btn_nodes.push(btn);
            }

            // 设置初始位置（完全偏移，动画滑入）
            // 与 apply_panel_animation_for 一致，t=0 时 offset = sub_menu_width
            let sw = sub_menu_width as f32;
            let offset = sw;

            match direction {
                DIR_LEFT => {
                    let base_left = (menu_width + sub_menu_width * (next_level as i32 - 1)) as f32;
                    panel.set_anchor_and_offset(Side::LEFT, 0.0, base_left - offset);
                    panel.set_anchor_and_offset(Side::RIGHT, 0.0, base_left + sw - offset);
                }
                DIR_RIGHT => {
                    let base_right = (menu_width + sub_menu_width * next_level as i32) as f32;
                    panel.set_anchor_and_offset(Side::LEFT, 1.0, -(base_right + sw) + offset);
                    panel.set_anchor_and_offset(Side::RIGHT, 1.0, -base_right + offset);
                }
                _ => {}
            }

            // 设为可见（create_panel 中 level>0 时设了 visible=false）
            panel.set_visible(true);

            self.panels.push(PanelInfo {
                panel,
                margin,
                vbox,
                buttons: btn_nodes,
                active_index: -1,
                anim_progress: 0.0,
                animating: true,
                opening: true,
            });
        }
    }

    /// 从指定级别开始收起所有子面板
    fn collapse_from_level(&mut self, from_level: usize) {
        // 对于 from_level 及之后的面板，启动关闭动画
        for i in from_level..self.panels.len() {
            self.panels[i].active_index = -1;
            if self.panels[i].panel.clone().is_visible() {
                self.panels[i].animating = true;
                self.panels[i].opening = false;
            }
            // 重置按钮样式
            let normal_color = if i == 0 {
                self.item_color
            } else {
                self.sub_item_color
            };
            for btn in &self.panels[i].buttons {
                let mut b = btn.clone();
                b.add_theme_color_override(&StringName::from("font_color"), normal_color);
            }
        }
    }

    /// 关闭所有子面板（关闭菜单时调用）
    /// 从 level 1 开始，level 0 由主动画系统管理
    fn collapse_all_sub_panels(&mut self) {
        self.collapse_from_level(1);
    }

    /// 应用主菜单动画（一级面板 + 遮罩）
    fn apply_main_animation(&mut self) {
        let t = ease_out_cubic(self.main_anim_progress as f32) as f64;

        // 更新遮罩透明度
        if let Some(ref overlay) = self.overlay {
            let mut o = overlay.clone();
            o.set_color(Color::from_rgba(
                self.overlay_color.r,
                self.overlay_color.g,
                self.overlay_color.b,
                self.overlay_color.a * t as f32,
            ));
        }

        // 更新一级面板位置
        if !self.panels.is_empty() {
            let mut p = self.panels[0].panel.clone();
            let w = self.menu_width as f32;
            let offset = w * (1.0 - t as f32);

            match self.direction {
                DIR_LEFT => {
                    p.set_anchor_and_offset(Side::LEFT, 0.0, -offset);
                    p.set_anchor_and_offset(Side::RIGHT, 0.0, w - offset);
                }
                DIR_RIGHT => {
                    p.set_anchor_and_offset(Side::LEFT, 1.0, -w + offset);
                    p.set_anchor_and_offset(Side::RIGHT, 1.0, offset);
                }
                _ => {}
            }
        }
    }

    /// 应用子面板动画（按索引）
    /// Level N (N>=1) 最终位置: LEFT=menu_width+sub_menu_width*(N-1), RIGHT=menu_width+sub_menu_width*N
    /// 动画: 从左侧滑入，offset = sub_menu_width * (1-t)
    fn apply_panel_animation_for(&self, level: usize, progress: f32) {
        if level == 0 || level >= self.panels.len() {
            return;
        }

        let t = ease_out_cubic(progress);
        let mut p = self.panels[level].panel.clone();
        let sw = self.sub_menu_width as f32;
        let offset = sw * (1.0 - t);

        match self.direction {
            DIR_LEFT => {
                let base_left = (self.menu_width + self.sub_menu_width * (level as i32 - 1)) as f32;
                p.set_anchor_and_offset(Side::LEFT, 0.0, base_left - offset);
                p.set_anchor_and_offset(Side::RIGHT, 0.0, base_left + sw - offset);
            }
            DIR_RIGHT => {
                let base_right = (self.menu_width + self.sub_menu_width * level as i32) as f32;
                p.set_anchor_and_offset(Side::LEFT, 1.0, -(base_right + sw) + offset);
                p.set_anchor_and_offset(Side::RIGHT, 1.0, -base_right + offset);
            }
            _ => {}
        }
    }
}

/// 缓动函数：ease-out cubic
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
