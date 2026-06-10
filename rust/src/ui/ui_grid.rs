// GdUIGrid - 网格列表节点
// 翻译自 C++ gmlc/ui_list_grid.h/cpp
// 继承 GridContainer，支持 slot 模板复制、点击高亮、填充效果、鼠标进入/离开事件
// 移动端触摸长按支持、patch_item 单项更新
// GML 标签：<UIGrid count="6" columns="3" highlight_mode="1">

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Variant, Array, Dictionary, Vector2, Rect2, NodePath};
use godot::classes::{
    IGridContainer, GridContainer, Control, InputEvent, InputEventMouseButton,
    InputEventScreenTouch, Os,
};
use godot::global::MouseButton;
use godot::obj::WithBaseField;

use super::ui_list_helper;

#[derive(GodotClass)]
#[class(base = GridContainer)]
pub struct GdUIGrid {
    base: Base<GridContainer>,

    inited: bool,
    prev_mouse_enter_idx: i32,
    highlight_node: Option<Gd<Control>>,
    is_touching: bool,
    touch_time: f64,
    last_mouse_enter_time: f64,
    is_dragging: bool,
    drag_start_pos: Vector2,
    pressed_item_idx: i32,
    tooltip_node: Option<Gd<Control>>,

    #[export]
    slot_path: GString,
    #[export]
    count: i32,
    #[export]
    slots: Array<Variant>,
    #[export]
    highlight_mode: i32,
    #[export]
    highlight_color: Color,
    #[export]
    fill_mode: i32,
    #[export]
    fill_color: Color,
    #[export]
    tooltip: GString,
}

#[godot_api]
impl IGridContainer for GdUIGrid {
    fn init(base: Base<GridContainer>) -> Self {
        Self {
            base,
            inited: false,
            prev_mouse_enter_idx: -1,
            highlight_node: None,
            is_touching: false,
            touch_time: 0.0,
            last_mouse_enter_time: 0.0,
            is_dragging: false,
            drag_start_pos: Vector2::ZERO,
            pressed_item_idx: -1,
            tooltip_node: None,
            slot_path: GString::new(),
            count: -1,
            slots: Array::new(),
            highlight_mode: 0,
            highlight_color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            fill_mode: 0,
            fill_color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            tooltip: GString::new(),
        }
    }

    fn ready(&mut self) {
        self.inited = false;
        self.initial();
    }

    fn process(&mut self, delta: f64) {
        // 移动端长按检测
        let os = Os::singleton();
        if os.has_feature(&GString::from("mobile")) {
            if self.is_touching {
                self.touch_time += delta;
                if self.touch_time >= 0.5 {
                    if self.prev_mouse_enter_idx >= 0 {
                        if let Some(d) = self.get_at(self.prev_mouse_enter_idx) {
                            self.base_mut().emit_signal(
                                &StringName::from("s_mouse_enter_item"),
                                &[d.to_variant()],
                            );
                        }
                    }
                    self.is_touching = false;
                }
            }
        }

        // 定期检查鼠标是否移出范围（修复有时无法触发 mouse_exited 的问题）
        let current_time = godot::classes::Time::singleton()
            .get_ticks_msec() as f64
            / 1000.0;
        if self.prev_mouse_enter_idx != -1 && current_time - self.last_mouse_enter_time > 1.0 {
            let child_count = self.base().get_child_count();
            if self.prev_mouse_enter_idx < child_count {
                if let Some(node) = self.get_at(self.prev_mouse_enter_idx) {
                    self.on_item_mouse_exit(&node);
                }
            }
        }
    }
}

#[godot_api]
impl GdUIGrid {
    #[signal]
    fn s_click_item(node: Gd<Control>);
    #[signal]
    fn s_mouse_enter_item(node: Gd<Control>);
    #[signal]
    fn s_mouse_exit_item(node: Gd<Control>);

    /// 初始化列表
    #[func]
    fn initial(&mut self) {
        let slot = self.get_slot();
        if slot.is_none() {
            return;
        }
        let slot = slot.unwrap();

        if !self.inited {
            self.inited = true;
            // 如果 slot 已有父节点，先移除
            if slot.get_parent().is_some() {
                if let Some(mut parent) = slot.get_parent() {
                    parent.remove_child(&slot);
                }
            }
            self.base_mut().add_child(&slot);
            self.base_mut().move_child(&slot, 0);
        }

        let mut target = self.base_mut().clone().upcast::<Control>();
        ui_list_helper::list_initial(&mut target, &slot, self.count);
        self.bind_events();
    }

    /// 更新列表数据
    #[func]
    fn update(&mut self, data: Array<Variant>, force: bool) {
        let slot = self.get_slot();
        if slot.is_none() {
            return;
        }
        let slot = slot.unwrap();

        if force {
            self.count = data.len() as i32;
        }

        let aliased_data = ui_list_helper::update_data_alias(&data, &self.slots);
        let mut target = self.base_mut().clone().upcast::<Control>();
        ui_list_helper::update_container(&mut target, &slot, self.count, &aliased_data);
        self.bind_events();
    }

    /// 更新单个子项
    #[func]
    fn patch_item(&mut self, idx: i32, data: Dictionary<Variant, Variant>) {
        let mut target = self.base_mut().clone().upcast::<Control>();
        ui_list_helper::update_child_dict(&mut target, idx, &data);
    }

    /// 更新所有子项
    #[func]
    fn update_all(&mut self, data: Dictionary<Variant, Variant>) {
        let child_count = self.base().get_child_count() - 1; // 减去 slot
        let mut args = Array::new();
        for _ in 0..child_count {
            args.push(&data);
        }
        self.update(args, false);
    }

    /// 获取指定索引的子节点
    #[func]
    fn get_at(&self, id: i32) -> Option<Gd<Control>> {
        // +1 跳过 index 0 的 slot 模板
        let actual_index = id + 1;
        if actual_index < 1 || actual_index >= self.base().get_child_count() {
            return None;
        }
        if let Some(child) = self.base().get_child(actual_index) {
            child.clone().try_cast::<Control>().ok()
        } else {
            None
        }
    }

    /// 获取 meta 值
    #[func]
    fn get_meta_value(&self, idx: i32, key: GString, default: Variant) -> Variant {
        if let Some(d) = self.get_at(idx) {
            d.get_meta(&StringName::from(key.to_string().as_str()))
        } else {
            default
        }
    }

    /// 批量绑定信号
    #[func]
    fn allbind_signal(&mut self, path: GString, sig: GString, cb: Callable) {
        let mut target = self.base_mut().clone().upcast::<Control>();
        ui_list_helper::allbind_signal(
            &mut target,
            &path.to_string(),
            &sig.to_string(),
            &cb,
        );
    }

    /// 内部：处理子节点点击事件
    #[func]
    fn _on_item_gui_input_internal(&mut self, event: Gd<InputEvent>, item_index: i32) {
        let os = Os::singleton();
        if os.has_feature(&GString::from("mobile")) {
            // 移动端触摸处理
            if let Ok(touch_event) = event.try_cast::<InputEventScreenTouch>() {
                if touch_event.is_pressed() {
                    self.is_touching = true;
                    self.prev_mouse_enter_idx = item_index;
                    self.is_dragging = false;
                    self.drag_start_pos = touch_event.get_position();
                    self.pressed_item_idx = item_index;
                } else if touch_event.is_released() && self.pressed_item_idx == item_index {
                    if self.touch_time < 0.5 {
                        // 短按 = 点击
                        let drag_end_pos = touch_event.get_position();
                        let drag_distance = self.drag_start_pos.distance_to(drag_end_pos);
                        const CLICK_THRESHOLD: f32 = 5.0;

                        if !self.is_dragging && drag_distance < CLICK_THRESHOLD {
                            self.apply_highlight(item_index);
                            if let Some(click_item) = self.get_at(item_index) {
                                self.base_mut().emit_signal(
                                    &StringName::from("s_click_item"),
                                    &[click_item.to_variant()],
                                );
                            }
                        }
                    } else {
                        // 长按后松手 = mouse_exit
                        if let Some(click_item) = self.get_at(item_index) {
                            self.base_mut().emit_signal(
                                &StringName::from("s_mouse_exit_item"),
                                &[click_item.to_variant()],
                            );
                        }
                    }
                    self.is_touching = false;
                    self.touch_time = 0.0;
                    self.prev_mouse_enter_idx = -1;
                    self.pressed_item_idx = -1;
                }
            }
        } else {
            // PC 端鼠标处理
            if let Ok(mouse_btn) = event.try_cast::<InputEventMouseButton>() {
                if mouse_btn.get_button_index() == MouseButton::LEFT {
                    if mouse_btn.is_pressed() {
                        self.is_dragging = false;
                        self.drag_start_pos = mouse_btn.get_position();
                        self.pressed_item_idx = item_index;
                    } else if mouse_btn.is_released() && self.pressed_item_idx == item_index {
                        let drag_end_pos = mouse_btn.get_position();
                        let drag_distance = self.drag_start_pos.distance_to(drag_end_pos);
                        const CLICK_THRESHOLD: f32 = 5.0;

                        if !self.is_dragging && drag_distance < CLICK_THRESHOLD {
                            self.apply_highlight(item_index);
                            if let Some(click_item) = self.get_at(item_index) {
                                self.base_mut().emit_signal(
                                    &StringName::from("s_click_item"),
                                    &[click_item.to_variant()],
                                );
                            }
                        }
                        self.pressed_item_idx = -1;
                        self.is_dragging = false;
                    }
                }
            }
        }
    }

    /// 内部：处理鼠标进入事件
    #[func]
    fn _on_item_mouse_enter_internal(&mut self, item_index: i32) {
        if let Some(click_item) = self.get_at(item_index) {
            self.on_item_mouse_enter(&click_item);
        }
    }

    /// 内部：处理鼠标离开事件
    #[func]
    fn _on_item_mouse_exit_internal(&mut self, item_index: i32) {
        if let Some(click_item) = self.get_at(item_index) {
            self.on_item_mouse_exit(&click_item);
        }
    }
}

impl GdUIGrid {
    fn get_slot(&self) -> Option<Gd<Control>> {
        if self.slot_path.is_empty() {
            if self.base().get_child_count() > 0 {
                if let Some(child) = self.base().get_child(0) {
                    return child.clone().try_cast::<Control>().ok();
                }
            }
            return None;
        }
        let node_path = NodePath::from(&self.slot_path.to_string());
        self.base()
            .get_node_or_null(&node_path)
            .and_then(|n| n.try_cast::<Control>().ok())
    }

    fn bind_events(&mut self) {
        let children = self.base().get_children();
        let fill_mode = self.fill_mode;
        let fill_color = self.fill_color;
        let os = Os::singleton();
        let is_pc = os.has_feature(&GString::from("pc"));

        // 从 index 1 开始，跳过 slot 模板
        for i in 1..children.len() {
            if let Some(child_var) = children.get(i) {
                if let Ok(mut n) = child_var.clone().try_cast::<Control>() {
                    // item_index 为可见子节点索引（0-based）
                    let item_index = (i - 1) as i64;

                    // 绑定点击事件 - 使用 bind 传入 item_index
                    let gui_signal = StringName::from("gui_input");
                    let callable = Callable::from_object_method(
                        &*self.base_mut(),
                        "_on_item_gui_input_internal",
                    ).bind(&[Variant::from(item_index)]);
                    if !n.is_connected(&gui_signal, &callable) {
                        n.connect(&gui_signal, &callable);
                    }

                    // PC 端绑定鼠标进入/离开事件
                    if is_pc {
                        let enter_signal = StringName::from("mouse_entered");
                        let enter_cb = Callable::from_object_method(
                            &*self.base_mut(),
                            "_on_item_mouse_enter_internal",
                        ).bind(&[Variant::from(item_index)]);
                        if !n.is_connected(&enter_signal, &enter_cb) {
                            n.connect(&enter_signal, &enter_cb);
                        }

                        let exit_signal = StringName::from("mouse_exited");
                        let exit_cb = Callable::from_object_method(
                            &*self.base_mut(),
                            "_on_item_mouse_exit_internal",
                        ).bind(&[Variant::from(item_index)]);
                        if !n.is_connected(&exit_signal, &exit_cb) {
                            n.connect(&exit_signal, &exit_cb);
                        }
                    }

                    // 填充效果
                    if fill_mode > 0 {
                        ui_list_helper::update_slot_fill(&mut n, fill_color, fill_mode);
                    }
                }
            }
        }
    }

    fn on_item_mouse_enter(&mut self, click_item: &Gd<Control>) {
        let local_mouse_pos = click_item.get_local_mouse_position();
        let item_rect = Rect2::new(Vector2::ZERO, click_item.get_size());
        if item_rect.contains_point(local_mouse_pos) {
            let idx = click_item.get_index() as i32;
            if self.prev_mouse_enter_idx != idx {
                self.prev_mouse_enter_idx = idx;
                self.last_mouse_enter_time = godot::classes::Time::singleton()
                    .get_ticks_msec() as f64
                    / 1000.0;

                // 自动 Tooltip 绑定
                self.show_tooltip_for_item(click_item);

                self.base_mut().emit_signal(
                    &StringName::from("s_mouse_enter_item"),
                    &[click_item.to_variant()],
                );
            }
        }
    }

    fn on_item_mouse_exit(&mut self, click_item: &Gd<Control>) {
        let local_mouse_pos = click_item.get_local_mouse_position();
        let item_rect = Rect2::new(Vector2::ZERO, click_item.get_size());
        if !item_rect.contains_point(local_mouse_pos) {
            self.prev_mouse_enter_idx = -1;

            // 自动隐藏 Tooltip
            self.hide_tooltip();

            self.base_mut().emit_signal(
                &StringName::from("s_mouse_exit_item"),
                &[click_item.to_variant()],
            );
        }
    }

    /// 查找 Tooltip 节点（向上遍历节点树搜索）
    fn find_tooltip_node(&mut self) -> Option<Gd<Control>> {
        if self.tooltip.is_empty() {
            return None;
        }
        let tooltip_gstr = GString::from(&self.tooltip.to_string());
        // 从直接父节点开始，逐级向上搜索
        let mut current = self.base().get_parent();
        while let Some(parent) = current {
            if let Some(node) = parent.find_child(&tooltip_gstr) {
                return node.try_cast::<Control>().ok();
            }
            current = parent.get_parent();
        }
        None
    }

    /// 为指定项显示 Tooltip
    fn show_tooltip_for_item(&mut self, item: &Gd<Control>) {
        if self.tooltip.is_empty() {
            return;
        }
        if self.tooltip_node.is_none() {
            self.tooltip_node = self.find_tooltip_node();
        }
        if let Some(ref mut tooltip_ctrl) = self.tooltip_node {
            // 从 item 的 meta 中读取 name 和 desc（安全获取，避免 nil 传入导致 panic）
            let name_key = StringName::from("name");
            let desc_key = StringName::from("desc");
            if !item.has_meta(&name_key) {
                return;
            }
            let name_val = item.get_meta(&name_key);
            let name_str = name_val.to_string();
            if name_str.is_empty() {
                return;
            }
            let desc_val = if item.has_meta(&desc_key) {
                item.get_meta(&desc_key).to_string()
            } else {
                String::new()
            };
            tooltip_ctrl.call(&StringName::from("set_tooltip_title"), &[GString::from(&name_str).to_variant()]);
            tooltip_ctrl.call(&StringName::from("set_tooltip_content"), &[GString::from(&desc_val).to_variant()]);
            tooltip_ctrl.call(&StringName::from("show_tooltip"), &[]);
        }
    }

    /// 隐藏 Tooltip
    fn hide_tooltip(&mut self) {
        if let Some(ref mut tooltip_ctrl) = self.tooltip_node {
            tooltip_ctrl.call(&StringName::from("hide_tooltip"), &[]);
        }
    }

    fn apply_highlight(&mut self, item_index: i32) {
        if self.highlight_mode <= 0 {
            return;
        }

        if let Some(ref highlight) = self.highlight_node {
            if highlight.is_inside_tree() {
                if let Some(mut parent) = highlight.get_parent() {
                    parent.remove_child(highlight);
                }
            }
        }

        if self.highlight_node.is_none() {
            self.highlight_node = Some(match self.highlight_mode {
                1 => ui_list_helper::create_square_highlight_node(0.05, self.highlight_color),
                2 => ui_list_helper::create_circle_highlight_node(0.05, self.highlight_color),
                _ => return,
            });
        }

        if let Some(ref highlight) = self.highlight_node {
            if let Some(mut click_item) = self.get_at(item_index) {
                click_item.add_child(highlight);
            }
        }
    }
}
