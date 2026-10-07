// GdUIModal - 模态弹窗节点
// 全屏遮罩 + 内容区，遮罩淡入 + 内容缩放动画（Drawer 的居中弹窗版）
// GML 标签：<Modal name="StoreModal" ui_id="StoreModal" anchor="full"
//            close_on_overlay="true" content_margin="56">
//   <Gml src="..." />  ← 内容子节点进入 ContentArea（四周留白 content_margin 像素）
//   content_margin: 内容区四周留白（像素，默认 0）——弹窗主体由内容自绘
//                   （如 window-bg 面板），本组件只负责遮罩与容器
//   close_on_overlay: 点击遮罩关闭，默认 true
//   内容关闭联动：内容根声明 s_close_requested 信号（面板关闭按钮 emit）时
//                自动 close——面板独立 F6 仍走自身 hide()，组合进 Modal 时弹窗整体关闭
//   与 GdUIManager 集成：ui_id="XxxModal" 声明后，任意组件 @pressed="show:XxxModal"
//   编辑器内：静态预览展开布局（遮罩透明），所见即所得；
//   运行期：初始隐藏，由 open()/内部动作 show:/toggle: 驱动

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color};
use godot::classes::{
    IControl, Control, ColorRect, MarginContainer, InputEvent, InputEventMouseButton, Engine,
};
use godot::classes::control::{LayoutPreset, MouseFilter};
use godot::global::MouseButton;
use godot::obj::WithBaseField;

use crate::anim::easing::ease_out_cubic;

/// 内容缩放动画：从 94% 弹到 100%
const SCALE_FROM: f32 = 0.94;

#[derive(GodotClass)]
#[class(base = Control, tool)]
pub struct GdUIModal {
    base: Base<Control>,

    #[export]
    overlay_color: Color,
    /// 内容区四周留白（像素），0 = 内容铺满
    #[export]
    content_margin: i32,
    #[export]
    animation_duration: f64,
    #[export]
    close_on_overlay: bool,

    // 内部节点引用
    overlay: Option<Gd<ColorRect>>,
    content_area: Option<Gd<MarginContainer>>,
    is_open: bool,
    ui_built: bool,
    anim_progress: f64,
    animating: bool,
    opening: bool,
}

#[godot_api]
impl IControl for GdUIModal {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            overlay_color: Color::from_rgba(0.0, 0.0, 0.0, 0.55),
            content_margin: 0,
            animation_duration: 0.22,
            close_on_overlay: true,
            overlay: None,
            content_area: None,
            is_open: false,
            ui_built: false,
            anim_progress: 0.0,
            animating: false,
            opening: false,
        }
    }

    fn ready(&mut self) {
        if !self.ui_built {
            self.build_ui();
        }
        // 编辑器：静态预览"展开到位"布局（遮罩透明），所见即所得
        if Engine::singleton().is_editor_hint() {
            if let Some(ref overlay) = self.overlay {
                let mut o = overlay.clone();
                o.set_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
            }
            return;
        }
        // 运行期：直接隐藏（open() 时从遮罩全透明 + 内容 94% 缩放进入）
        self.is_open = false;
        self.animating = false;
        self.anim_progress = 0.0;
        self.base_mut().set_visible(false);
    }

    fn process(&mut self, delta: f64) {
        if !self.animating {
            return;
        }
        let speed = if self.animation_duration > 0.0 {
            1.0 / self.animation_duration
        } else {
            100.0
        };

        if self.opening {
            self.anim_progress += delta * speed;
            if self.anim_progress >= 1.0 {
                self.anim_progress = 1.0;
                self.animating = false;
            }
        } else {
            self.anim_progress -= delta * speed;
            if self.anim_progress <= 0.0 {
                self.anim_progress = 0.0;
                self.animating = false;
                // 动画结束，隐藏整个 Modal
                self.base_mut().set_visible(false);
            }
        }

        self.apply_animation();
    }
}

#[godot_api]
impl GdUIModal {
    #[signal]
    fn s_modal_opened();

    #[signal]
    fn s_modal_closed();

    /// 打开弹窗
    #[func]
    fn open(&mut self) {
        if self.is_open {
            return;
        }
        self.is_open = true;
        self.opening = true;
        self.animating = true;
        self.base_mut().set_visible(true);
        // 内容可能在上次关闭时被内容自身 hide()（面板关闭按钮 hide + emit
        // s_close_requested 的标准协议）——打开时统一恢复内容根可见，
        // 否则二次打开只剩遮罩没有窗口
        if let Some(ref ca) = self.content_area {
            let kids = ca.clone().get_children();
            for i in 0..kids.len() {
                if let Some(mut k) = kids.get(i) {
                    if let Ok(mut c) = k.try_cast::<Control>() {
                        c.set_visible(true);
                    }
                }
            }
        }
        // 遮罩从全透明渐入，内容从 94% 缩放弹出
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

    /// 关闭弹窗
    #[func]
    fn close(&mut self) {
        if !self.is_open && !self.animating {
            return;
        }
        self.is_open = false;
        self.opening = false;
        self.animating = true;
        self.base_mut().emit_signal(&StringName::from("s_modal_closed"), &[]);
    }

    /// 切换弹窗开关
    #[func]
    fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// 弹窗是否打开
    #[func]
    fn is_modal_open(&self) -> bool {
        self.is_open
    }

    /// 添加子节点到内容区域（供 builder 调用）；
    /// 内容根声明 s_close_requested 信号时自动连接 → close（内容关闭联动）
    #[func]
    fn add_content_child(&mut self, mut child: Gd<godot::classes::Node>) {
        if let Some(ref mut ca) = self.content_area {
            ca.add_child(&child);
            child.set_owner(&ca.clone().upcast::<godot::classes::Node>());
        } else {
            godot_error!("[UIModal] add_content_child: content_area is None!");
            return;
        }
        self.connect_content_close(&child);
    }

    /// 处理遮罩点击关闭
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

    /// 确保内部 UI 已构建（供 builder 在添加子节点前调用）
    #[func]
    fn ensure_ui_built(&mut self) {
        if !self.ui_built {
            self.build_ui();
        }
    }
}

impl GdUIModal {
    /// 重连内容关闭联动：内容根（或其任意后代）声明 s_close_requested 信号
    /// （面板关闭按钮 emit）→ 弹窗整体 close。
    /// 构建期 CallableCustom 连接不随 tscn 序列化，tscn 实例化（复用路径）必须重连；
    /// is_connected 保证幂等
    fn connect_content_close(&mut self, content: &Gd<godot::classes::Node>) {
        let base = self.base();
        let cb = Callable::from_object_method(&*base, "close");
        let sig = StringName::from("s_close_requested");
        let mut stack: Vec<Gd<godot::classes::Node>> = vec![content.clone()];
        while let Some(mut n) = stack.pop() {
            if n.has_signal(&sig) && !n.is_connected(&sig, &cb) {
                n.connect(&sig, &cb);
            }
            let kids = n.get_children();
            for i in 0..kids.len() {
                if let Some(k) = kids.get(i) {
                    stack.push(k);
                }
            }
        }
    }

    /// 重连内部回调：遮罩 gui_input → 点击关闭；内容关闭联动
    /// （构建期连接不随 tscn 序列化，复用路径必须重连；运行期 parse 幂等）
    fn reconnect_internal_callbacks(&mut self) {
        if let Some(ref overlay) = self.overlay {
            let base = self.base();
            let cb = Callable::from_object_method(&*base, "_on_overlay_gui_input");
            let sig = StringName::from("gui_input");
            let mut o = overlay.clone();
            if !o.is_connected(&sig, &cb) {
                o.connect(&sig, &cb);
            }
        }
        if let Some(ref ca) = self.content_area {
            let kids = ca.clone().get_children();
            for i in 0..kids.len() {
                if let Some(k) = kids.get(i) {
                    self.connect_content_close(&k);
                }
            }
        }
    }

    fn build_ui(&mut self) {
        if self.ui_built {
            return;
        }

        // 从生成 tscn 实例化时，构建期 ensure_ui_built 的产物（Overlay/ContentArea）
        // 已随场景恢复——重新抓取内部节点引用即可，避免二次创建导致重复遮罩/容器
        {
            let mut base = self.base_mut();
            let has_existing = base.get_node_or_null(&NodePath::from("Overlay")).is_some()
                && base.get_node_or_null(&NodePath::from("ContentArea")).is_some();
            if has_existing {
                drop(base);
                let mut overlay_ref: Option<Gd<ColorRect>> = None;
                if let Some(ov) = self.base().get_node_or_null(&NodePath::from("Overlay")) {
                    if let Ok(ov) = ov.try_cast::<ColorRect>() {
                        overlay_ref = Some(ov);
                    }
                }
                let mut content_ref: Option<Gd<MarginContainer>> = None;
                if let Some(ca) = self.base().get_node_or_null(&NodePath::from("ContentArea")) {
                    if let Ok(ca) = ca.try_cast::<MarginContainer>() {
                        content_ref = Some(ca);
                    }
                }
                self.overlay = overlay_ref;
                self.content_area = content_ref;
                self.ui_built = true;
                // 构建期的 CallableCustom 回调不随 tscn 序列化（遮罩点击/内容关闭联动），
                // tscn 实例化走本复用路径，必须重连
                self.reconnect_internal_callbacks();
                return;
            }
        }
        self.ui_built = true;

        let overlay_color = self.overlay_color;
        let content_margin = self.content_margin;

        let mut overlay_node: Option<Gd<ColorRect>> = None;
        let mut content_node: Option<Gd<MarginContainer>> = None;

        {
            let mut base = self.base_mut();
            base.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);

            // 遮罩层（点击关闭）
            let mut overlay = ColorRect::new_alloc();
            overlay.set_name("Overlay");
            overlay.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
            overlay.set_color(overlay_color);
            overlay.set_mouse_filter(MouseFilter::STOP);
            let overlay_input_cb = Callable::from_object_method(
                &*base,
                "_on_overlay_gui_input",
            );
            overlay.connect(&StringName::from("gui_input"), &overlay_input_cb);
            base.add_child(&overlay);
            overlay_node = Some(overlay);

            // 内容区：四周留白 content_margin，鼠标穿透（留白区域点击落到遮罩 → 关闭）
            let mut content = MarginContainer::new_alloc();
            content.set_name("ContentArea");
            content.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
            content.set_mouse_filter(MouseFilter::IGNORE);
            content.add_theme_constant_override(&StringName::from("margin_left"), content_margin);
            content.add_theme_constant_override(&StringName::from("margin_right"), content_margin);
            content.add_theme_constant_override(&StringName::from("margin_top"), content_margin);
            content.add_theme_constant_override(&StringName::from("margin_bottom"), content_margin);
            base.add_child(&content);
            content_node = Some(content);
        }

        self.overlay = overlay_node;
        self.content_area = content_node;
        self.reconnect_internal_callbacks();
    }

    /// 应用动画插值（进度 self.anim_progress：0=完全隐藏，1=完全展开）
    fn apply_animation(&mut self) {
        let eased = ease_out_cubic(self.anim_progress as f32);

        // 遮罩透明度渐入渐出
        if let Some(ref overlay) = self.overlay {
            let mut o = overlay.clone();
            o.set_color(Color::from_rgba(
                self.overlay_color.r,
                self.overlay_color.g,
                self.overlay_color.b,
                self.overlay_color.a * eased,
            ));
        }

        // 内容缩放（中心为基准弹出/收回）
        if let Some(ref ca) = self.content_area {
            let mut c = ca.clone();
            let size = c.get_size();
            c.set_pivot_offset(size / 2.0);
            let s = SCALE_FROM + (1.0 - SCALE_FROM) * eased;
            c.set_scale(Vector2::new(s, s));
        }

        // 打开完成时发送信号
        if self.anim_progress >= 1.0 && self.opening {
            self.base_mut().emit_signal(&StringName::from("s_modal_opened"), &[]);
        }
    }
}
