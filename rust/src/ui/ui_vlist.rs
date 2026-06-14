// GdUIVList - 垂直列表节点
// 翻译自 C++ gmlc/ui_list_v.h/cpp
// 继承 VBoxContainer，支持 slot 模板复制、点击高亮、填充效果、鼠标进入/离开事件、随机高度
// GML 标签：<UIVList count="3" highlight_mode="2" fill_mode="1" enable_random_pos="true">

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Variant, Array, Dictionary, Vector2, Rect2, NodePath};
use godot::classes::{
    IVBoxContainer, VBoxContainer, Control, InputEvent, InputEventMouseButton,
    Tween,
};
use godot::global::MouseButton;
use godot::obj::WithBaseField;

use super::ui_list_helper;

#[derive(GodotClass)]
#[class(base = VBoxContainer)]
pub struct GdUIVList {
    base: Base<VBoxContainer>,

    inited: bool,
    prev_mouse_enter_idx: i32,
    highlight_node: Option<Gd<Control>>,
    is_dragging: bool,
    drag_start_pos: Vector2,
    pressed_item_idx: i32,

    #[export]
    slot_path: GString,
    #[export]
    count: i32,
    #[export]
    highlight_mode: i32,
    #[export]
    highlight_color: Color,
    #[export]
    fill_mode: i32,
    #[export]
    fill_color: Color,
    #[export]
    enable_random_pos: bool,
    #[export]
    random_rotate: f32,
    #[export]
    update_slot: GString,
    #[export]
    slots: Array<Variant>,
}

#[godot_api]
impl IVBoxContainer for GdUIVList {
    fn init(base: Base<VBoxContainer>) -> Self {
        Self {
            base,
            inited: false,
            prev_mouse_enter_idx: -1,
            highlight_node: None,
            is_dragging: false,
            drag_start_pos: Vector2::ZERO,
            pressed_item_idx: -1,
            slot_path: GString::new(),
            count: -1,
            highlight_mode: 0,
            highlight_color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            fill_mode: 0,
            fill_color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            enable_random_pos: false,
            random_rotate: 0.0,
            update_slot: GString::from("update_slot"),
            slots: Array::new(),
        }
    }

    fn ready(&mut self) {
        self.inited = false;
        self.initial();
    }
}

#[godot_api]
impl GdUIVList {
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

        // 随机高度
        if self.enable_random_pos {
            let children = self.base().get_children();
            for i in 0..children.len() {
                if let Some(child_var) = children.get(i) {
                    if let Ok(mut n) = child_var.clone().try_cast::<Control>() {
                        if !n.get_meta(&StringName::from("vlist_height_random")).booleanize() {
                            let current_size = n.get_custom_minimum_size();
                            if self.random_rotate > 0.0 && n.get_child_count() > 0 {
                                if let Some(first_child) = n.get_child(0) {
                                    if let Ok(mut nn) = first_child.try_cast::<Control>() {
                                        let size = nn.get_size();
                                        nn.set_pivot_offset(size / 2.0);
                                        let angle = (rand::random::<f32>() * self.random_rotate)
                                            .to_radians()
                                            .to_degrees();
                                        nn.set_rotation(angle);
                                    }
                                }
                            }
                            let extra_height = 30.0 + rand::random::<f32>() * 50.0;
                            n.set_custom_minimum_size(Vector2::new(
                                current_size.x,
                                current_size.y + extra_height,
                            ));
                            n.set_meta(
                                &StringName::from("vlist_height_random"),
                                &true.to_variant(),
                            );
                        }
                    }
                }
            }
        }
    }

    /// 更新所有子项
    #[func]
    fn update_all(&mut self, data: Dictionary<Variant, Variant>) {
        let child_count = self.base().get_child_count();
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

    /// 设置高度倍数
    #[func]
    fn set_height_times(&mut self, idx: i32, times: i32) {
        if let Some(slot) = self.get_slot() {
            let ori_size = slot.get_size();
            if let Some(node) = self.get_at(idx) {
                let mut node = node;
                node.set_custom_minimum_size(Vector2::new(ori_size.x, ori_size.y * times as f32));
            }
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
                        // 高亮效果
                        if self.highlight_mode > 0 {
                            if let Some(ref highlight) = self.highlight_node {
                                if highlight.is_inside_tree() {
                                    if let Some(mut parent) = highlight.get_parent() {
                                        parent.remove_child(highlight);
                                    }
                                }
                            }
                            if self.highlight_node.is_none() {
                                self.highlight_node = Some(match self.highlight_mode {
                                    1 => ui_list_helper::create_square_highlight_node(
                                        0.05,
                                        self.highlight_color,
                                    ),
                                    2 => ui_list_helper::create_circle_highlight_node(
                                        0.05,
                                        self.highlight_color,
                                    ),
                                    _ => return,
                                });
                            }
                            if let Some(ref highlight) = self.highlight_node {
                                if let Some(mut click_item) = self.get_at(item_index) {
                                    click_item.add_child(highlight);
                                }
                            }
                        }

                        // 发射点击信号
                        if let Some(click_item) = self.get_at(item_index) {
                            // 点击缩放反馈动画
                            self.play_click_feedback(&click_item);
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

    /// 内部：处理鼠标进入事件
    #[func]
    fn _on_item_mouse_enter_internal(&mut self, item_index: i32) {
        if let Some(click_item) = self.get_at(item_index) {
            let local_mouse_pos = click_item.get_local_mouse_position();
            let item_rect = Rect2::new(Vector2::ZERO, click_item.get_size());
            if item_rect.contains_point(local_mouse_pos) {
                if self.prev_mouse_enter_idx != item_index {
                    self.prev_mouse_enter_idx = item_index;
                    self.base_mut().emit_signal(
                        &StringName::from("s_mouse_enter_item"),
                        &[click_item.to_variant()],
                    );
                }
            }
        }
    }

    /// 内部：处理鼠标离开事件
    #[func]
    fn _on_item_mouse_exit_internal(&mut self, item_index: i32) {
        if let Some(click_item) = self.get_at(item_index) {
            let local_mouse_pos = click_item.get_local_mouse_position();
            let item_rect = Rect2::new(Vector2::ZERO, click_item.get_size());
            if !item_rect.contains_point(local_mouse_pos) {
                self.prev_mouse_enter_idx = -1;
                self.base_mut().emit_signal(
                    &StringName::from("s_mouse_exit_item"),
                    &[click_item.to_variant()],
                );
            }
        }
    }
}

impl GdUIVList {
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

    /// 为子节点绑定事件（跳过 index 0 的 slot 模板）
    fn bind_events(&mut self) {
        let children = self.base().get_children();
        let fill_mode = self.fill_mode;
        let fill_color = self.fill_color;

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

                    // 绑定鼠标进入/离开事件 - 使用 bind 传入 item_index
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

                    // 填充效果
                    if fill_mode > 0 {
                        ui_list_helper::update_slot_fill(&mut n, fill_color, fill_mode);
                    }
                }
            }
        }
    }

    /// 点击缩放反馈动画
    fn play_click_feedback(&self, item: &Gd<Control>) {
        let original_scale = item.get_scale();
        let mut item = item.clone();
        let mut tween = item.create_tween();
        // 缩小
        tween.tween_property(
            &item,
            &NodePath::from("scale"),
            &(original_scale * 0.9).to_variant(),
            0.05,
        );
        // 弹回
        tween.tween_property(
            &item,
            &NodePath::from("scale"),
            &original_scale.to_variant(),
            0.15,
        ).set_trans(godot::classes::tween::TransitionType::BACK)
         .set_ease(godot::classes::tween::EaseType::OUT);
    }
}
