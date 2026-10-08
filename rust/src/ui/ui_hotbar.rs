// GdUIHotbar - 快捷装备栏 / 道具栏（数字键绑定 + 选中高亮框）
//
// GML 用法：
//   <Hotbar name="EquipSlots" slot_count="4" slot_size="46,46" key_bind="true"
//           selected_index="0" separation="6" @s_selected="_on_selected" />
//
// 能力：
//   - 槽位：slot_count 个 Panel 格子（背景/描边/圆角自绘，可放图标文字），
//     每格左上角显示键位角标（1..N），中央显示图标文字（set_slot_glyph 设置）
//   - 键盘选择：key_bind 开启时按数字键 1..N 选中对应槽位（unhandled_key_input，
//     不抢 UI 焦点；编辑器内自动忽略）
//   - 鼠标选择：点击槽位选中（gui_input 左键）
//   - 高亮框：选中槽位描边加粗换高亮色（StyleBoxFlat 覆盖，边框 3px）
//   - 信号：s_selected(index) —— 选中变化时发出（-1 = 清除选中）；
//     GML 内 @s_selected="方法名" 即可就近绑定
//
// 程序化 API：
//   select_slot(index)      选中（会发信号；-1 清除）
//   get_selected()          当前选中索引
//   set_slot_glyph(i, text) 设置槽位图标文字（emoji/字符，正式素材就位后可换 TextureRect）
//
// 装备语义（demo 范本 example/demo/xiuxian/scenes/main/ui/mainhud/mainhud_equipbar.gd）：
//   控制器把「选中槽位 → 物品 id」写入 GdState 状态总线，消费方（player.gd）
//   watch 同名键自行开/关对应能力（如枪支 → 开启射击），UI 与角色零耦合。

use godot::prelude::*;
use godot::builtin::{Color, GString, Side, StringName, Vector2};
use godot::classes::{
    Engine, HBoxContainer, IHBoxContainer, InputEvent, InputEventKey, Label, Panel, StyleBoxFlat,
};
use godot::classes::control::{LayoutPreset, MouseFilter};
use godot::global::{HorizontalAlignment, Key, MouseButton, VerticalAlignment};

#[derive(GodotClass)]
#[class(base = HBoxContainer, tool)]
pub struct GdUIHotbar {
    /// 槽位数量（1~9，对应数字键 1..N）
    #[export]
    slot_count: i32,
    /// 单个槽位尺寸（像素）
    #[export]
    slot_size: Vector2,
    /// 槽位间距（像素）
    #[export]
    separation: i32,
    /// 是否启用数字键 1..N 选中槽位
    #[export]
    key_bind: bool,
    /// 初始选中槽位（-1 = 无；ready 时应用，不发信号）
    #[export]
    selected_index: i32,
    /// 槽位背景色
    #[export]
    slot_bg: Color,
    /// 槽位描边色
    #[export]
    slot_border: Color,
    /// 选中高亮框颜色
    #[export]
    highlight_color: Color,
    /// 键位角标/图标文字颜色
    #[export]
    text_color: Color,

    // ---- 运行时状态 ----
    glyphs: Vec<GString>,
    slots: Vec<Gd<Panel>>,

    base: Base<HBoxContainer>,
}

#[godot_api]
impl IHBoxContainer for GdUIHotbar {
    fn init(base: Base<HBoxContainer>) -> Self {
        Self {
            slot_count: 4,
            slot_size: Vector2::new(46.0, 46.0),
            separation: 8,
            key_bind: true,
            selected_index: 0,
            slot_bg: Color::from_rgba(0.09, 0.15, 0.12, 0.92),
            slot_border: Color::from_rgba(0.42, 0.38, 0.28, 1.0),
            highlight_color: Color::from_rgba(0.95, 0.79, 0.35, 1.0),
            text_color: Color::from_rgba(0.95, 0.92, 0.82, 1.0),
            glyphs: Vec::new(),
            slots: Vec::new(),
            base,
        }
    }

    fn ready(&mut self) {
        let count = self.slot_count.clamp(1, 9) as usize;
        let separation = self.separation;
        self.base_mut().add_theme_constant_override(
            &StringName::from("separation"),
            separation,
        );

        // 槽位构建：Panel + 键位角标 + 图标文字（ready 装配，不随 tscn 打包）
        self.slots.clear();
        self.glyphs = vec![GString::new(); count];
        for i in 0..count {
            let panel = self.build_slot(i as i32);
            self.base_mut()
                .add_child(&panel.clone().upcast::<Node>());
            self.slots.push(panel);
        }

        // 初始选中：只刷高亮不发信号（消费者 ready 后经 get_selected 自行读取初值）
        let sel = if self.selected_index >= count as i32 {
            count as i32 - 1
        } else {
            self.selected_index
        };
        self.selected_index = sel;
        if sel >= 0 {
            self.apply_highlight(sel);
        }
    }

    /// 数字键 1..N 选中槽位（走 unhandled 阶段：UI 有焦点输入时优先，不抢事件）
    fn unhandled_key_input(&mut self, event: Gd<InputEvent>) {
        if !self.key_bind || Engine::singleton().is_editor_hint() {
            return;
        }
        let Ok(key_ev) = event.try_cast::<InputEventKey>() else {
            return;
        };
        if !key_ev.is_pressed() || key_ev.is_echo() {
            return;
        }
        let idx = keycode_to_slot(key_ev.get_keycode());
        if let Some(idx) = idx {
            self.select_slot(idx);
        }
    }
}

#[godot_api]
impl GdUIHotbar {
    /// 选中变化时发出（参数：槽位索引，-1 = 清除选中）
    #[signal]
    fn s_selected(index: i32);

    /// 选中槽位（会发出 s_selected；index = -1 清除选中，越界忽略）
    #[func]
    pub fn select_slot(&mut self, index: i32) {
        if index < -1 || index as usize >= self.slots.len() || index == self.selected_index {
            return;
        }
        self.selected_index = index;
        self.apply_highlight(index);
        self.base_mut()
            .emit_signal("s_selected", &[index.to_variant()]);
    }

    /// 当前选中槽位索引（-1 = 无）
    #[func]
    pub fn get_selected(&self) -> i32 {
        self.selected_index
    }

    /// 设置槽位图标文字（emoji/字符；正式素材就位后可扩展 TextureRect 版本）
    #[func]
    pub fn set_slot_glyph(&mut self, index: i32, glyph: GString) {
        if index < 0 || index as usize >= self.slots.len() {
            return;
        }
        self.glyphs[index as usize] = glyph.clone();
        if let Some(node) = self.slots[index as usize]
            .get_node_or_null(&NodePath::from("Glyph"))
        {
            if let Ok(mut lb) = node.try_cast::<Label>() {
                lb.set_text(&glyph);
            }
        }
    }

    /// 槽位点击（gui_input 转发，槽位索引由构建期闭包补绑）
    #[func]
    fn _on_slot_gui_input(&mut self, event: Gd<InputEvent>, index: i32) {
        if index < 0 || index as usize >= self.slots.len() {
            return;
        }
        if let Ok(mb) = event.clone().try_cast::<godot::classes::InputEventMouseButton>() {
            if mb.is_pressed() && mb.get_button_index() == MouseButton::LEFT {
                self.select_slot(index);
            }
        }
    }

    // ---- 内部实现 ----

    /// 构建单个槽位：Panel（背景/描边/圆角）+ 键位角标（左上）+ 图标文字（居中）
    fn build_slot(&self, index: i32) -> Gd<Panel> {
        let mut panel = Panel::new_alloc();
        panel.set_custom_minimum_size(self.slot_size);
        panel.add_theme_stylebox_override(
            &StringName::from("panel"),
            &self.make_stylebox(false),
        );

        // 键位角标：左上角小号数字（1..N）
        let mut key_lb = Label::new_alloc();
        key_lb.set_name("KeyNum");
        key_lb.set_text(&GString::from(format!("{}", index + 1).as_str()));
        key_lb.add_theme_font_size_override(&StringName::from("font_size"), 10);
        key_lb.add_theme_color_override(&StringName::from("font_color"), self.text_color);
        key_lb.set_position(Vector2::new(4.0, 1.0));
        key_lb.set_mouse_filter(MouseFilter::IGNORE);
        panel.add_child(&key_lb.clone().upcast::<Node>());

        // 图标文字：居中大号（emoji 占位，正式素材就位后替换）
        let mut glyph_lb = Label::new_alloc();
        glyph_lb.set_name("Glyph");
        glyph_lb.set_horizontal_alignment(HorizontalAlignment::CENTER);
        glyph_lb.set_vertical_alignment(VerticalAlignment::CENTER);
        glyph_lb.add_theme_font_size_override(&StringName::from("font_size"), 20);
        glyph_lb.add_theme_color_override(&StringName::from("font_color"), self.text_color);
        glyph_lb.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
        glyph_lb.set_mouse_filter(MouseFilter::IGNORE);
        panel.add_child(&glyph_lb.clone().upcast::<Node>());

        // 点击选中：gui_input 经闭包转发（补绑槽位索引）
        let forward = Callable::from_object_method(&*self.base(), "_on_slot_gui_input");
        let idx = index;
        let cb = Callable::from_fn("hotbar_slot_click", move |args: &[&Variant]| {
            let ev = args.first().copied().unwrap_or(&Variant::nil()).clone();
            forward.call(&[ev, idx.to_variant()]);
            Variant::nil()
        });
        panel.connect(&StringName::from("gui_input"), &cb);
        panel
    }

    /// 刷新全部槽位高亮（选中槽位描边 3px 高亮色，其余恢复普通描边）
    fn apply_highlight(&mut self, selected: i32) {
        for (i, slot) in self.slots.iter().enumerate() {
            let mut panel = slot.clone();
            panel.add_theme_stylebox_override(
                &StringName::from("panel"),
                &self.make_stylebox(i as i32 == selected),
            );
        }
    }

    /// 槽位样式：普通 = 1px 描边；选中 = 3px 高亮描边
    fn make_stylebox(&self, selected: bool) -> Gd<StyleBoxFlat> {
        let mut sb = StyleBoxFlat::new_gd();
        sb.set_bg_color(self.slot_bg);
        sb.set_border_color(if selected {
            self.highlight_color
        } else {
            self.slot_border
        });
        let width = if selected { 3 } else { 1 };
        for side in [Side::LEFT, Side::TOP, Side::RIGHT, Side::BOTTOM] {
            sb.set_border_width(side, width);
        }
        sb.set_corner_radius_all(8);
        sb
    }
}

/// 数字键 → 槽位索引（KEY_1 → 0 ... KEY_9 → 8），非数字键返回 None
fn keycode_to_slot(key: Key) -> Option<i32> {
    match key {
        Key::KEY_1 => Some(0),
        Key::KEY_2 => Some(1),
        Key::KEY_3 => Some(2),
        Key::KEY_4 => Some(3),
        Key::KEY_5 => Some(4),
        Key::KEY_6 => Some(5),
        Key::KEY_7 => Some(6),
        Key::KEY_8 => Some(7),
        Key::KEY_9 => Some(8),
        _ => None,
    }
}
