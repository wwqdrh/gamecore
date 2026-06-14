// GdUIHList - 水平列表节点
// 翻译自 C++ gmlc/ui_list.h/cpp
// 继承 HBoxContainer，支持 slot 模板复制、点击高亮、填充效果、鼠标进入/离开事件
// GML 标签：<UIHList count="5" highlight_mode="1" highlight_color="#ffff00">

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Variant, Array, Dictionary, Vector2, Rect2, NodePath};
use godot::classes::{
    IHBoxContainer, HBoxContainer, Control, InputEvent, InputEventMouseButton,
    Tween,
};
use godot::global::MouseButton;
use godot::obj::WithBaseField;

use super::ui_list_helper;

#[derive(GodotClass)]
#[class(base = HBoxContainer)]
pub struct GdUIHList {
    base: Base<HBoxContainer>,

    inited: bool,
    highlight_node: Option<Gd<Control>>,
    prev_mouse_enter_idx: i32,
    tooltip_node: Option<Gd<Control>>,

    #[export]
    slot_path: GString,
    #[export]
    count: i32,
    #[export]
    highlight_mode: i32, // 0=无, 1=方形, 2=圆形
    #[export]
    highlight_color: Color,
    #[export]
    fill_mode: i32, // 0=无, 1=方形, 2=圆形
    #[export]
    fill_color: Color,
    #[export]
    update_slot: GString,
    #[export]
    slots: Array<Variant>,
    #[export]
    space_left: f32,
    #[export]
    space_right: f32,
    #[export]
    tooltip: GString,
}

#[godot_api]
impl IHBoxContainer for GdUIHList {
    fn init(base: Base<HBoxContainer>) -> Self {
        Self {
            base,
            inited: false,
            highlight_node: None,
            prev_mouse_enter_idx: -1,
            tooltip_node: None,
            slot_path: GString::new(),
            count: -1,
            highlight_mode: 0,
            highlight_color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            fill_mode: 0,
            fill_color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            update_slot: GString::from("update_slot"),
            slots: Array::new(),
            space_left: 0.0,
            space_right: 0.0,
            tooltip: GString::new(),
        }
    }

    fn ready(&mut self) {
        self.inited = false;
        self.initial();
    }
}

#[godot_api]
impl GdUIHList {
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
            // 移到最前面（INTERNAL_MODE_FRONT 的等效）
            self.base_mut().move_child(&slot, 0);
        }

        let mut target = self.base_mut().clone().upcast::<Control>();
        ui_list_helper::list_initial(&mut target, &slot, self.count);

        // 绑定鼠标事件
        self.bind_events();

        // 设置左右间距
        if self.space_left > 0.0 {
            // 通过 meta 标记的间距节点实现（简化处理）
        }
        if self.space_right > 0.0 {
            // 同上
        }
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

        // 为节点绑定点击事件和填充效果
        self.bind_events();
    }

    /// 更新所有子项为相同数据
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

    /// 获取指定索引子节点的 meta 值
    #[func]
    fn get_meta_value(&self, idx: i32, key: GString, default: Variant) -> Variant {
        if let Some(d) = self.get_at(idx) {
            d.get_meta(&StringName::from(key.to_string().as_str()))
        } else {
            default
        }
    }

    /// 设置指定索引子节点的宽度倍数
    #[func]
    fn set_width_times(&mut self, idx: i32, times: i32) {
        if let Some(slot) = self.get_slot() {
            let ori_size = slot.get_size();
            if let Some(node) = self.get_at(idx) {
                let mut node = node;
                node.set_custom_minimum_size(Vector2::new(ori_size.x * times as f32, ori_size.y));
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
        // 检查是否为鼠标左键点击
        if let Ok(mouse_btn) = event.try_cast::<InputEventMouseButton>() {
            if mouse_btn.get_button_index() == MouseButton::LEFT && mouse_btn.is_pressed() {
                // 左键按下
                self.handle_click(item_index);
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

impl GdUIHList {
    /// 获取 slot 模板节点
    fn get_slot(&self) -> Option<Gd<Control>> {
        if self.slot_path.is_empty() {
            // 默认使用第一个子节点作为 slot
            if self.base().get_child_count() > 0 {
                if let Some(child) = self.base().get_child(0) {
                    return child.clone().try_cast::<Control>().ok();
                }
            }
            return None;
        }
        let node_path = NodePath::from(&self.slot_path.to_string());
        self.base().get_node_or_null(&node_path)
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
                    let signal_name = StringName::from("gui_input");
                    let callable = Callable::from_object_method(
                        &*self.base_mut(),
                        "_on_item_gui_input_internal",
                    ).bind(&[Variant::from(item_index)]);

                    if !n.is_connected(&signal_name, &callable) {
                        n.connect(&signal_name, &callable);
                    }

                    // 绑定鼠标进入事件
                    let enter_signal = StringName::from("mouse_entered");
                    let enter_cb = Callable::from_object_method(
                        &*self.base_mut(),
                        "_on_item_mouse_enter_internal",
                    ).bind(&[Variant::from(item_index)]);
                    if !n.is_connected(&enter_signal, &enter_cb) {
                        n.connect(&enter_signal, &enter_cb);
                    }

                    // 绑定鼠标离开事件
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

    /// 处理点击逻辑
    fn handle_click(&mut self, item_index: i32) {
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

    /// 处理鼠标进入
    fn on_item_mouse_enter(&mut self, click_item: &Gd<Control>) {
        let local_mouse_pos = click_item.get_local_mouse_position();
        let item_rect = Rect2::new(Vector2::ZERO, click_item.get_size());
        if item_rect.contains_point(local_mouse_pos) {
            let idx = click_item.get_index() as i32;
            if self.prev_mouse_enter_idx != idx {
                self.prev_mouse_enter_idx = idx;

                // 自动 Tooltip 绑定
                self.show_tooltip_for_item(click_item);

                self.base_mut().emit_signal(
                    &StringName::from("s_mouse_enter_item"),
                    &[click_item.to_variant()],
                );
            }
        }
    }

    /// 处理鼠标离开
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
            if let Some(node) = parent.find_child_ex(&tooltip_gstr).recursive(true).owned(false).done() {
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
        // 延迟查找 Tooltip 节点
        if self.tooltip_node.is_none() {
            self.tooltip_node = self.find_tooltip_node();
        }
        if let Some(ref mut tooltip_ctrl) = self.tooltip_node {
            // 从 item 的 __item_data meta 中读取完整数据字典
            let item_data_key = StringName::from("__item_data");
            //godot_print!("[UIHList] show_tooltip_for_item: has_item_data={}", item.has_meta(&item_data_key));
            if item.has_meta(&item_data_key) {
                let item_data = item.get_meta(&item_data_key);
                //godot_print!("[UIHList] show_tooltip_for_item: item_data type={:?}", item_data.get_type());
                if item_data.get_type() == godot::builtin::VariantType::DICTIONARY {
                    if let Ok(dict) = item_data.try_to::<Dictionary<Variant, Variant>>() {
                        //godot_print!("[UIHList] show_tooltip_for_item: dict keys={:?}", (0..dict.keys_array().len()).filter_map(|i| dict.keys_array().get(i).map(|v| v.to_string())).collect::<Vec<_>>());
                        // 调用 update_data 解析自定义子节点的 {{key}} 模板绑定
                        tooltip_ctrl.call(&StringName::from("update_data"), &[dict.to_variant()]);

                        // 兼容内置 title/content label：从字典中读取 name/desc
                        let name_val = dict.get(&"name".to_variant()).unwrap_or(Variant::nil());
                        let name_str = name_val.to_string();
                        if name_str.is_empty() {
                            return;
                        }
                        let desc_val = dict.get(&"desc".to_variant()).unwrap_or(Variant::nil());
                        let desc_str = desc_val.to_string();
                        tooltip_ctrl.call(&StringName::from("set_tooltip_title"), &[GString::from(&name_str).to_variant()]);
                        tooltip_ctrl.call(&StringName::from("set_tooltip_content"), &[GString::from(&desc_str).to_variant()]);
                        tooltip_ctrl.call(&StringName::from("show_tooltip"), &[]);
                        return;
                    }
                }
            }

            // 降级：从 item 的 meta 中读取 name 和 desc（兼容旧数据格式）
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
