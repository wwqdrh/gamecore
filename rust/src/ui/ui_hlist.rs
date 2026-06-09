// GdUIHList - 水平列表节点
// 翻译自 C++ gmlc/ui_list.h/cpp
// 继承 HBoxContainer，支持 slot 模板复制、点击高亮、填充效果
// GML 标签：<UIHList count="5" highlight_mode="1" highlight_color="#ffff00">

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Variant, Array, Dictionary, Vector2, NodePath};
use godot::classes::{
    IHBoxContainer, HBoxContainer, Control, InputEvent, InputEventMouseButton,
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
}

#[godot_api]
impl IHBoxContainer for GdUIHList {
    fn init(base: Base<HBoxContainer>) -> Self {
        Self {
            base,
            inited: false,
            highlight_node: None,
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
        if id < 0 || id >= self.base().get_child_count() {
            return None;
        }
        if let Some(child) = self.base().get_child(id) {
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

    /// 为子节点绑定事件
    fn bind_events(&mut self) {
        let children = self.base().get_children();
        let fill_mode = self.fill_mode;
        let fill_color = self.fill_color;

        for i in 0..children.len() {
            if let Some(child_var) = children.get(i) {
                if let Ok(mut n) = child_var.clone().try_cast::<Control>() {
                    // 绑定点击事件 - 使用 bind 传入 item_index
                    let signal_name = StringName::from("gui_input");
                    let callable = Callable::from_object_method(
                        &*self.base_mut(),
                        "_on_item_gui_input_internal",
                    ).bind(&[Variant::from(i as i64)]);

                    if !n.is_connected(&signal_name, &callable) {
                        n.connect(&signal_name, &callable);
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
            self.base_mut().emit_signal(
                &StringName::from("s_click_item"),
                &[click_item.to_variant()],
            );
        }
    }
}
