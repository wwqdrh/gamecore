// GdUIForm 系列表单组件 —— 设置界面常用的表单控件 + Rust 侧预定义处理逻辑的接口组件
//
// 基础表单控件（纯 UI 状态 + 统一信号上报，业务逻辑由使用方接线）：
//   <FormCheck text="显示伤害数字" checked="true" />            多选框（checkbox）
//   <FormRadio options="历练,问道,渡劫" value="问道" />          单选框组（radio group）
//   <FormSelect options="流畅,均衡,高清,影视" value="均衡" />     下拉选项
//   <FormSwitch text="全屏" checked="false" />                  开关（switch）
//   <FormSliderH min_value="0" max_value="1" step="0.01" value="0.8" />  左右滑块
//   <FormSliderV min_value="0" max_value="1" value="0.8" />     上下滑块
//
// 接口组件（bind 属性绑定 manager 接口，Rust 侧预定义读写逻辑，UI 即插即用，
// 经 GdViewSetting（rust/src/manager/setting.rs）统一读写并即时持久化）：
//   <SettingSlider bind="volume:Music" min_value="0" max_value="1" step="0.01" />
//   <SettingSwitch bind="fullscreen" />   bind="vsync" / bind="custom:<key>"
//   <SettingSelect bind="custom:graphics" options="..." value="..." />
//
// bind 协议：
//   volume:<Bus>   → GdViewSetting.set_volume(bus, v)（Master/Music/Audio/Voice）
//   fullscreen     → set_fullscreen(bool)（经根 Window，headless 只持久化）
//   vsync          → set_vsync(bool)（读侧走持久化值，headless 不应用）
//   custom:<key>   → set_value(key, v)（自定义设置，随 user://settings.data 持久化）
//
// 统一信号（GML 内 @s_toggled / @s_value_changed 即可就近绑定，语义同 @pressed）：
//   s_toggled(checked: bool)          —— FormCheck / FormSwitch / SettingSwitch
//   s_value_changed(value: ...)       —— FormRadio/FormSelect/SettingSelect(GString)、
//                                        FormSliderH/V(f64)、SettingSlider(f64)
//
// 程序化 API：行选中互斥 / 初值应用走 apply_checked（FormCheck/FormSwitch）与
// select_value（FormRadio/FormSelect）——#[var] 属性访问器（get_value/set_value/
// set_checked…）只改存储字段，不驱动视图刷新，运行期改状态请用这两个方法。
//
// 编辑器内：tool 组件静态绘制当前状态，所见即所得；
// 运行期：FormRadio 行、绑定初值等在 ready 装配（不随 tscn 打包，无重复子节点）

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Vector2};
use godot::classes::{
    IButton, IVBoxContainer, IOptionButton, IHSlider, IVSlider,
    Button, VBoxContainer, OptionButton, HSlider, VSlider, StyleBoxFlat, StyleBoxEmpty, Font,
    StyleBox, Engine, Control, Node, Object, SceneTree,
};
use godot::global::{Key, HorizontalAlignment};
use godot::obj::WithBaseField;

use crate::manager::setting::GdViewSetting;

// ---------------------------------------------------------------------------
// GdViewSetting 共享桥（单实例复用，避免每个组件重复加载设置文件）
// 实例挂在 SceneTree 根节点 meta 上（引擎自有生命周期，退出时由引擎释放；
// 严禁用 thread_local 缓存 Gd —— 引擎卸载后 TLS drop 会触发 ffi panic + abort）
// ---------------------------------------------------------------------------

const SETTING_META: &str = "__ui_form_setting";

/// 从根节点 meta 取共享实例，缺失时构建并缓存
fn fetch_setting() -> Option<Gd<GdViewSetting>> {
    let main_loop = Engine::singleton().get_main_loop()?;
    let tree: Gd<SceneTree> = main_loop.try_cast().ok()?;
    let mut root = tree.get_root()?;
    let meta_name = StringName::from(SETTING_META);
    if root.has_meta(&meta_name) {
        let v = root.get_meta(&meta_name);
        if let Ok(gd) = v.try_to::<Gd<GdViewSetting>>() {
            if gd.is_instance_valid() {
                return Some(gd);
            }
        }
    }
    let gd = GdViewSetting::build(GString::from("user://settings.data"), GString::from("setting"));
    root.set_meta(&meta_name, &gd.to_variant());
    Some(gd)
}

/// 访问共享 GdViewSetting（首次访问懒构建 user://settings.data / scope=setting）
fn with_setting<R>(f: impl FnOnce(&mut Gd<GdViewSetting>) -> R) -> Option<R> {
    let mut gd = fetch_setting()?;
    Some(f(&mut gd))
}

/// bind 协议解析
enum SettingBind {
    Volume(String),
    Fullscreen,
    Vsync,
    Custom(String),
}

fn parse_bind(bind: &str) -> SettingBind {
    let b = bind.trim();
    if let Some(bus) = b.strip_prefix("volume:") {
        return SettingBind::Volume(bus.trim().to_string());
    }
    match b {
        "fullscreen" => SettingBind::Fullscreen,
        "vsync" => SettingBind::Vsync,
        _ => SettingBind::Custom(b.strip_prefix("custom:").unwrap_or(b).trim().to_string()),
    }
}

/// 读布尔型绑定值（SettingSwitch 初值）
fn read_bind_bool(bind: &str, default_value: bool) -> bool {
    match parse_bind(bind) {
        SettingBind::Fullscreen => {
            with_setting(|s| s.bind().is_fullscreen()).unwrap_or(default_value)
        }
        SettingBind::Vsync => with_setting(|s| {
            s.bind().get_value(GString::from("vsync"), default_value.to_variant())
        })
        .map(|v| v.to::<bool>())
        .unwrap_or(default_value),
        SettingBind::Custom(key) => with_setting(|s| {
            s.bind().get_value(GString::from(key.as_str()), default_value.to_variant())
        })
        .map(|v| v.to::<bool>())
        .unwrap_or(default_value),
        SettingBind::Volume(_) => default_value,
    }
}

/// 写布尔型绑定值
fn apply_bind_bool(bind: &str, value: bool) {
    match parse_bind(bind) {
        SettingBind::Fullscreen => {
            let _ = with_setting(|s| s.bind_mut().set_fullscreen(value));
        }
        SettingBind::Vsync => {
            let _ = with_setting(|s| s.bind_mut().set_vsync(value));
        }
        SettingBind::Custom(key) => {
            let _ = with_setting(|s| {
                s.bind_mut().set_value(GString::from(key.as_str()), value.to_variant())
            });
        }
        SettingBind::Volume(_) => {}
    }
}

/// 读数值型绑定值（SettingSlider 初值）
fn read_bind_f64(bind: &str, default_value: f64) -> f64 {
    match parse_bind(bind) {
        SettingBind::Volume(bus) => with_setting(|s| {
            s.bind().get_volume(GString::from(bus.as_str()))
        })
        .unwrap_or(default_value),
        SettingBind::Custom(key) => with_setting(|s| {
            s.bind().get_value(GString::from(key.as_str()), default_value.to_variant())
        })
        .and_then(|v| v.try_to::<f64>().ok())
        .unwrap_or(default_value),
        _ => default_value,
    }
}

/// 写数值型绑定值
fn apply_bind_f64(bind: &str, value: f64) {
    match parse_bind(bind) {
        SettingBind::Volume(bus) => {
            let _ = with_setting(|s| {
                s.bind_mut().set_volume(GString::from(bus.as_str()), value)
            });
        }
        SettingBind::Custom(key) => {
            let _ = with_setting(|s| {
                s.bind_mut().set_value(GString::from(key.as_str()), value.to_variant())
            });
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// 共享绘制/样式工具（FormCheck / FormSwitch / SettingSwitch 复用）
// ---------------------------------------------------------------------------

/// Button 系自定义绘制组件清空原生主题样式（normal/hover/pressed/focus 全部透明）
fn clear_button_styleboxes(btn: &mut Gd<Button>) {
    for state in ["normal", "hover", "pressed", "hover_pressed", "focus", "disabled"] {
        let sb = StyleBoxEmpty::new_gd();
        btn.add_theme_stylebox_override(&StringName::from(state), &sb);
    }
}

/// 取主题字体与字号（font_size 覆写对自定义 draw 同样生效）
fn theme_font(btn: &Gd<Button>) -> (Option<Gd<Font>>, i32) {
    let font = btn.get_theme_font(&StringName::from("font"));
    let size = btn.get_theme_font_size(&StringName::from("font_size"));
    (font, size.max(12))
}

/// 绘制圆角矩形（StyleBoxFlat 代理，radius 像素圆角）
fn draw_round_rect(
    base: &mut Gd<Button>,
    rect: Rect2,
    fill: Color,
    border: Color,
    border_width: i32,
    radius: i32,
) {
    let mut sb = StyleBoxFlat::new_gd();
    sb.set_bg_color(fill);
    sb.set_corner_radius_all(radius);
    if border_width > 0 {
        sb.set_border_width_all(border_width);
        sb.set_border_color(border);
    }
    let sb_up: Gd<StyleBox> = sb.upcast();
    base.draw_style_box(&sb_up, rect);
}

/// 绘制文本（主题字体，baseline 垂直居中近似）
fn draw_label_text(base: &mut Gd<Button>, text: &str, x: f32, color: Color) {
    if text.is_empty() {
        return;
    }
    let (font, font_size) = theme_font(base);
    let Some(font) = font else { return };
    let size = base.get_size();
    let baseline = size.y / 2.0 + font_size as f32 * 0.36;
    let text_gd = GString::from(text);
    base.draw_string_ex(
        &font,
        Vector2::new(x, baseline),
        &text_gd,
    )
    .font_size(font_size)
    .modulate(color)
    .done();
}

/// 绘制多选框/单选框视觉（box 18x18 垂直居中 + 文本）
/// radio_style=true 画圆环 + 内点；false 画方框 + 对勾
fn draw_check_visual(
    base: &mut Gd<Button>,
    label: &str,
    radio_style: bool,
    checked: bool,
    accent_color: Color,
    border_color: Color,
    label_color: Color,
) {
    let size = base.get_size();
    let box_size = 18.0;
    let box_y = (size.y - box_size) / 2.0;
    let rect = Rect2::new(Vector2::new(2.0, box_y), Vector2::new(box_size, box_size));
    let center = Vector2::new(2.0 + box_size / 2.0, box_y + box_size / 2.0);

    if radio_style {
        let border = if checked { accent_color } else { border_color };
        draw_round_rect(base, rect, Color::from_rgba(0.0, 0.0, 0.0, 0.0), border, 2, 9);
        if checked {
            base.draw_circle(center, 4.5, accent_color);
        }
    } else {
        let fill = if checked { accent_color } else { Color::from_rgba(0.0, 0.0, 0.0, 0.0) };
        let border = if checked { accent_color } else { border_color };
        draw_round_rect(base, rect, fill, border, 2, 4);
        if checked {
            base.draw_line_ex(
                center + Vector2::new(-4.5, 0.0),
                center + Vector2::new(-1.0, 3.5),
                label_color,
            )
            .width(2.0)
            .done();
            base.draw_line_ex(
                center + Vector2::new(-1.0, 3.5),
                center + Vector2::new(4.5, -3.0),
                label_color,
            )
            .width(2.0)
            .done();
        }
    }

    draw_label_text(base, label, 30.0, label_color);
}

/// 绘制 switch 视觉（文本居左 + 轨道靠右 + 圆形滑块，progress 0..1 动画）
fn draw_switch_visual(
    base: &mut Gd<Button>,
    label: &str,
    progress: f32,
    on_color: Color,
    off_color: Color,
    knob_color: Color,
    label_color: Color,
) {
    let size = base.get_size();
    let track_w = 46.0;
    let track_h = 24.0;
    let track_x = (size.x - track_w).max(0.0);
    let track_y = (size.y - track_h) / 2.0;

    // 轨道颜色随进度在 off/on 之间插值
    let t = progress.clamp(0.0, 1.0);
    let track_color = Color::from_rgba(
        off_color.r + (on_color.r - off_color.r) * t,
        off_color.g + (on_color.g - off_color.g) * t,
        off_color.b + (on_color.b - off_color.b) * t,
        1.0,
    );
    let rect = Rect2::new(Vector2::new(track_x, track_y), Vector2::new(track_w, track_h));
    draw_round_rect(base, rect, track_color, Color::from_rgba(0.0, 0.0, 0.0, 0.0), 0, 12);

    // 滑块：3px 内边距，直径 18
    let knob_r = 9.0;
    let knob_x = track_x + 3.0 + t * (track_w - 6.0 - knob_r * 2.0);
    base.draw_circle(Vector2::new(knob_x + knob_r, track_y + track_h / 2.0), knob_r, knob_color);

    draw_label_text(base, label, 2.0, label_color);
}

/// OptionButton 系下拉控件的内建样式（深底 + 描边圆角 + 字色）
fn style_option_button(btn: &mut Gd<OptionButton>, label_color: Color) {
    let sn = StringName::from;
    for state in ["normal", "hover", "pressed"] {
        let mut sb = StyleBoxFlat::new_gd();
        sb.set_bg_color(match state {
            "hover" => Color::from_rgb(0.23, 0.22, 0.26),
            _ => Color::from_rgb(0.18, 0.17, 0.21),
        });
        sb.set_corner_radius_all(6);
        sb.set_border_width_all(1);
        sb.set_border_color(Color::from_rgb(0.45, 0.4, 0.3));
        sb.set_content_margin_all(6.0);
        btn.add_theme_stylebox_override(&sn(state), &sb);
    }
    btn.add_theme_color_override(&sn("font_color"), label_color);
    btn.add_theme_color_override(&sn("font_hover_color"), label_color);
    btn.add_theme_color_override(&sn("font_pressed_color"), label_color);
}

/// 填充下拉选项并按 value 选中，返回选中索引（无匹配且有项时选 0）
fn populate_option_button(ob: &mut Gd<OptionButton>, options: &str, value: &str) -> i64 {
    ob.clear();
    let mut selected: i64 = -1;
    let mut count: i64 = 0;
    for opt in options.split(',') {
        let opt = opt.trim();
        if opt.is_empty() {
            continue;
        }
        ob.add_item(&GString::from(opt));
        if opt == value {
            selected = count;
        }
        count += 1;
    }
    if selected < 0 && count > 0 {
        selected = 0;
    }
    if selected >= 0 {
        ob.select(selected as i32);
    }
    selected
}

// ---------------------------------------------------------------------------
// FormCheck —— 多选框（多个实例组合即多选；radio_style=true 时为圆形单选样式，
// 互斥语义由 FormRadio 组容器负责）
// ---------------------------------------------------------------------------

#[derive(GodotClass)]
#[class(base = Button, tool)]
pub struct GdUIFormCheck {
    base: Base<Button>,

    /// 文本标签（自定义绘制；避免与 Button 原生 text 属性重名）
    #[export]
    label: GString,
    /// 是否选中
    #[export]
    checked: bool,
    /// 单选样式（圆环 + 内点），互斥逻辑由 FormRadio 驱动
    #[export]
    radio_style: bool,
    #[export]
    accent_color: Color,
    #[export]
    border_color: Color,
    #[export]
    label_color: Color,
}

#[godot_api]
impl IButton for GdUIFormCheck {
    fn init(base: Base<Button>) -> Self {
        Self {
            base,
            label: GString::new(),
            checked: false,
            radio_style: false,
            accent_color: Color::from_rgb(0.91, 0.78, 0.47),
            border_color: Color::from_rgb(0.52, 0.47, 0.38),
            label_color: Color::from_rgb(0.16, 0.14, 0.1),
        }
    }

    fn ready(&mut self) {
        let mut btn = self.to_gd().upcast::<Button>();
        btn.set_toggle_mode(true);
        btn.set_pressed_no_signal(self.checked);
        clear_button_styleboxes(&mut btn);
        // 按文本实测宽度设最小尺寸：宽度 0 的控件鼠标命中测试永远打不到（点击无反应），
        // 且自绘内容不受矩形裁剪，会越界画到行右侧之外
        let min_h = 30.0f32;
        let (font, font_size) = theme_font(&btn);
        let label = self.label.to_string();
        let text_w = font
            .map(|f| {
                f.get_string_size_ex(&GString::from(label.as_str()))
                    .alignment(HorizontalAlignment::LEFT)
                    .width(-1.0)
                    .font_size(font_size)
                    .done()
                    .x
            })
            .unwrap_or(0.0);
        let min_w = (30.0 + text_w + 10.0).max(40.0);
        btn.set_custom_minimum_size(Vector2::new(min_w, min_h));
        let cb = Callable::from_object_method(&*self.base(), "_on_toggled");
        self.base_mut()
            .connect(&StringName::from("toggled"), &cb);
    }

    fn draw(&mut self) {
        let label = self.label.to_string();
        let mut btn = self.to_gd().upcast::<Button>();
        draw_check_visual(
            &mut btn,
            &label,
            self.radio_style,
            self.checked,
            self.accent_color,
            self.border_color,
            self.label_color,
        );
    }
}

#[godot_api]
impl GdUIFormCheck {
    #[signal]
    fn s_toggled(checked: bool);

    /// 按钮点击回调（toggle_mode 下由原生 toggled 信号驱动）
    #[func]
    fn _on_toggled(&mut self, pressed: bool) {
        self.apply_checked(pressed);
        self.base_mut()
            .emit_signal(&StringName::from("s_toggled"), &[pressed.to_variant()]);
    }

    /// 程序化设置选中态（发信号用 emit / 直接点击；本方法不发信号）
    #[func]
    fn apply_checked(&mut self, value: bool) {
        self.checked = value;
        self.base_mut().set_pressed_no_signal(value);
        self.base_mut().queue_redraw();
    }

    #[func]
    fn is_checked(&self) -> bool {
        self.checked
    }
}

// ---------------------------------------------------------------------------
// FormRadio —— 单选框组（VBoxContainer，行 = radio_style 的 FormCheck）
// ---------------------------------------------------------------------------

#[derive(GodotClass)]
#[class(base = VBoxContainer, tool)]
pub struct GdUIFormRadio {
    base: Base<VBoxContainer>,

    /// 选项列表（逗号分隔）
    #[export]
    options: GString,
    /// 当前选中值
    #[export]
    value: GString,
    /// 行高（像素）
    #[export]
    item_height: i32,
    #[export]
    font_size: i32,
    #[export]
    accent_color: Color,
    #[export]
    border_color: Color,
    #[export]
    label_color: Color,

    /// 行引用（构建于 ready，不随 tscn 打包）
    rows: Vec<Gd<GdUIFormCheck>>,
}

#[godot_api]
impl IVBoxContainer for GdUIFormRadio {
    fn init(base: Base<VBoxContainer>) -> Self {
        Self {
            base,
            options: GString::new(),
            value: GString::new(),
            item_height: 30,
            font_size: 15,
            accent_color: Color::from_rgb(0.91, 0.78, 0.47),
            border_color: Color::from_rgb(0.52, 0.47, 0.38),
            label_color: Color::from_rgb(0.16, 0.14, 0.1),
            rows: Vec::new(),
        }
    }

    fn ready(&mut self) {
        let options = self.options.to_string();
        let initial_value = self.value.to_string();
        let item_height = self.item_height;
        let font_size = self.font_size;
        let accent = self.accent_color;
        let border = self.border_color;
        let label_color = self.label_color;

        self.rows.clear();
        self.base_mut()
            .add_theme_constant_override(&StringName::from("separation"), 4);

        for opt in options.split(',') {
            let opt = opt.trim();
            if opt.is_empty() {
                continue;
            }
            let mut row = GdUIFormCheck::new_alloc();
            {
                let mut ctrl = row.clone().upcast::<Control>();
                ctrl.set(&StringName::from("label"), &GString::from(opt).to_variant());
                ctrl.set(&StringName::from("radio_style"), &true.to_variant());
                ctrl.set(
                    &StringName::from("checked"),
                    &(opt == initial_value).to_variant(),
                );
                ctrl.set(&StringName::from("accent_color"), &accent.to_variant());
                ctrl.set(&StringName::from("border_color"), &border.to_variant());
                ctrl.set(&StringName::from("label_color"), &label_color.to_variant());
                ctrl.add_theme_font_size_override(&StringName::from("font_size"), font_size);
            }
            self.base_mut()
                .add_child(&row.clone().upcast::<Node>());

            // add_child 触发行 ready（按文本实测出最小宽度）后，只把行高调到 item_height，
            // 保留宽度——覆盖成 (0, h) 会让行失去命中区域
            {
                let mut ctrl = row.clone().upcast::<Control>();
                let mut min_size = ctrl.get_combined_minimum_size();
                min_size.y = item_height as f32;
                ctrl.set_custom_minimum_size(min_size);
            }

            // 行选中 → 组内互斥（from_fn 捕获行引用，运行期转发组容器处理）
            let row_cb: Gd<Object> = row.clone().upcast();
            let group_cb = Callable::from_object_method(&*self.base(), "_on_row_toggled");
            let cb = Callable::from_fn("form_radio_row", move |args: &[&Variant]| {
                let checked = args
                    .first()
                    .and_then(|v| v.try_to::<bool>().ok())
                    .unwrap_or(false);
                group_cb.call(&[row_cb.to_variant(), checked.to_variant()]);
                Variant::nil()
            });
            let mut ctrl_conn = row.clone().upcast::<Control>();
            ctrl_conn.connect(&StringName::from("s_toggled"), &cb);
            self.rows.push(row);
        }
    }
}

#[godot_api]
impl GdUIFormRadio {
    #[signal]
    fn s_value_changed(value: GString);

    /// 行选中转发（deferred 应用：避免回调链上重入绑定发出行）
    #[func]
    fn _on_row_toggled(&mut self, row: Gd<Control>, checked: bool) {
        self.base_mut().call_deferred(
            &StringName::from("_apply_radio"),
            &[row.to_variant(), checked.to_variant()],
        );
    }

    /// 互斥应用（deferred）：选中行清其余、取消选中恢复（单选不可反选）
    #[func]
    fn _apply_radio(&mut self, row: Gd<Control>, checked: bool) {
        let row_id = row.instance_id();
        if checked {
            for r in &self.rows {
                if r.instance_id() != row_id {
                    r.clone().bind_mut().apply_checked(false);
                }
            }
            // 更新值并上报
            if let Ok(r) = row.clone().try_cast::<GdUIFormCheck>() {
                let label = r.bind().label.clone();
                if self.value != label {
                    self.value = label.clone();
                    self.base_mut().emit_signal(
                        &StringName::from("s_value_changed"),
                        &[label.to_variant()],
                    );
                }
            }
        } else {
            // 单选不可反选：恢复选中
            for r in &self.rows {
                if r.instance_id() == row_id {
                    r.clone().bind_mut().apply_checked(true);
                }
            }
        }
    }

    /// 程序化设值并刷新行选中态（不发信号；ready 前调用则作为初值生效）
    #[func]
    fn select_value(&mut self, value: GString) {
        self.value = value.clone();
        let want = value.to_string();
        for r in &self.rows {
            let label = r.bind().label.to_string();
            r.clone().bind_mut().apply_checked(label == want);
        }
    }
}

// ---------------------------------------------------------------------------
// FormSelect —— 下拉选项（OptionButton 子类：options/value 字符串语义）
// ---------------------------------------------------------------------------

#[derive(GodotClass)]
#[class(base = OptionButton, tool)]
pub struct GdUIFormSelect {
    base: Base<OptionButton>,

    /// 选项列表（逗号分隔）
    #[export]
    options: GString,
    /// 当前值（选项文本）
    #[export]
    value: GString,
    #[export]
    label_color: Color,
}

#[godot_api]
impl IOptionButton for GdUIFormSelect {
    fn init(base: Base<OptionButton>) -> Self {
        Self {
            base,
            options: GString::new(),
            value: GString::new(),
            label_color: Color::from_rgb(0.2, 0.18, 0.14),
        }
    }

    fn ready(&mut self) {
        let options = self.options.to_string();
        let value = self.value.to_string();
        {
            let mut ob = self.to_gd().upcast::<OptionButton>();
            style_option_button(&mut ob, self.label_color);
            populate_option_button(&mut ob, &options, &value);
        }
        let selected = self.base().get_selected();
        if selected >= 0 {
            self.value = self.base().get_item_text(selected);
        }
        let cb = Callable::from_object_method(&*self.base(), "_on_item_selected");
        self.base_mut()
            .connect(&StringName::from("item_selected"), &cb);
    }
}

#[godot_api]
impl GdUIFormSelect {
    #[signal]
    fn s_value_changed(value: GString);

    #[func]
    fn _on_item_selected(&mut self, index: i64) {
        let text = self.base().get_item_text(index as i32);
        self.value = text.clone();
        self.base_mut()
            .emit_signal(&StringName::from("s_value_changed"), &[text.to_variant()]);
    }

    /// 程序化设值并刷新下拉选中项（不发信号；ready 前调用则作为初值生效）
    #[func]
    fn select_value(&mut self, value: GString) {
        self.value = value.clone();
        let options = self.options.to_string();
        let selected = {
            let mut ob = self.to_gd().upcast::<OptionButton>();
            populate_option_button(&mut ob, &options, &value.to_string())
        };
        if selected >= 0 {
            self.value = self.base().get_item_text(selected as i32);
        }
    }
}

// ---------------------------------------------------------------------------
// FormSwitch —— 开关（toggle Button + 自绘轨道滑块，带 0.12s 动画）
// ---------------------------------------------------------------------------

const SWITCH_ANIM_SPEED: f32 = 1.0 / 0.12;

#[derive(GodotClass)]
#[class(base = Button, tool)]
pub struct GdUIFormSwitch {
    base: Base<Button>,

    /// 文本标签（自定义绘制）
    #[export]
    label: GString,
    /// 是否开启
    #[export]
    checked: bool,
    #[export]
    on_color: Color,
    #[export]
    off_color: Color,
    #[export]
    knob_color: Color,
    #[export]
    label_color: Color,

    /// 滑块动画进度（0=关 1=开）
    knob_progress: f32,
}

#[godot_api]
impl IButton for GdUIFormSwitch {
    fn init(base: Base<Button>) -> Self {
        Self {
            base,
            label: GString::new(),
            checked: false,
            on_color: Color::from_rgb(0.55, 0.72, 0.42),
            off_color: Color::from_rgb(0.32, 0.3, 0.33),
            knob_color: Color::from_rgb(0.95, 0.92, 0.84),
            label_color: Color::from_rgb(0.16, 0.14, 0.1),
            knob_progress: 0.0,
        }
    }

    fn ready(&mut self) {
        {
            let mut btn = self.to_gd().upcast::<Button>();
            btn.set_toggle_mode(true);
            btn.set_pressed_no_signal(self.checked);
            btn.set_custom_minimum_size(Vector2::new(140.0, 34.0));
            clear_button_styleboxes(&mut btn);
        }
        self.knob_progress = if self.checked { 1.0 } else { 0.0 };
        let cb = Callable::from_object_method(&*self.base(), "_on_toggled");
        self.base_mut()
            .connect(&StringName::from("toggled"), &cb);
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
            return;
        }
        let target = if self.checked { 1.0 } else { 0.0 };
        if (self.knob_progress - target).abs() > 0.001 {
            let step = (delta as f32) * SWITCH_ANIM_SPEED;
            if self.knob_progress < target {
                self.knob_progress = (self.knob_progress + step).min(target);
            } else {
                self.knob_progress = (self.knob_progress - step).max(target);
            }
            self.base_mut().queue_redraw();
        }
    }

    fn draw(&mut self) {
        let label = self.label.to_string();
        let mut btn = self.to_gd().upcast::<Button>();
        draw_switch_visual(
            &mut btn,
            &label,
            self.knob_progress,
            self.on_color,
            self.off_color,
            self.knob_color,
            self.label_color,
        );
    }
}

#[godot_api]
impl GdUIFormSwitch {
    #[signal]
    fn s_toggled(checked: bool);

    #[func]
    fn _on_toggled(&mut self, pressed: bool) {
        self.apply_checked(pressed);
        self.base_mut()
            .emit_signal(&StringName::from("s_toggled"), &[pressed.to_variant()]);
    }

    /// 程序化设置（不发信号）
    #[func]
    fn apply_checked(&mut self, value: bool) {
        self.checked = value;
        self.base_mut().set_pressed_no_signal(value);
        self.base_mut().queue_redraw();
    }

    #[func]
    fn is_checked(&self) -> bool {
        self.checked
    }
}

// ---------------------------------------------------------------------------
// FormSliderH / FormSliderV —— 左右/上下滑块（Range 原生交互，
// 统一 re-emit s_value_changed(f64)；min/max/step/value 属性走通用 Range 解析）
// ---------------------------------------------------------------------------

/// FormSliderH —— 左右滑块：原生 HSlider 交互 + s_value_changed(f64) 统一上报
#[derive(GodotClass)]
#[class(base = HSlider, tool)]
pub struct GdUIFormSliderH {
    base: Base<HSlider>,
}

#[godot_api]
impl IHSlider for GdUIFormSliderH {
    fn init(base: Base<HSlider>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        let cb = Callable::from_object_method(&*self.base(), "_on_value_changed");
        self.base_mut()
            .connect(&StringName::from("value_changed"), &cb);
    }
}

#[godot_api]
impl GdUIFormSliderH {
    #[signal]
    fn s_value_changed(value: f64);

    #[func]
    fn _on_value_changed(&mut self, value: f64) {
        self.base_mut().emit_signal(
            &StringName::from("s_value_changed"),
            &[value.to_variant()],
        );
    }
}

/// FormSliderV —— 上下滑块：原生 VSlider 交互 + s_value_changed(f64) 统一上报
#[derive(GodotClass)]
#[class(base = VSlider, tool)]
pub struct GdUIFormSliderV {
    base: Base<VSlider>,
}

#[godot_api]
impl IVSlider for GdUIFormSliderV {
    fn init(base: Base<VSlider>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        let cb = Callable::from_object_method(&*self.base(), "_on_value_changed");
        self.base_mut()
            .connect(&StringName::from("value_changed"), &cb);
    }
}

#[godot_api]
impl GdUIFormSliderV {
    #[signal]
    fn s_value_changed(value: f64);

    #[func]
    fn _on_value_changed(&mut self, value: f64) {
        self.base_mut().emit_signal(
            &StringName::from("s_value_changed"),
            &[value.to_variant()],
        );
    }
}

// ---------------------------------------------------------------------------
// SettingSlider —— 音量/自定义数值接口组件（bind → GdViewSetting）
// 初值自动从设置读取，拖动即时应用并持久化
// ---------------------------------------------------------------------------

#[derive(GodotClass)]
#[class(base = HSlider, tool)]
pub struct GdUISettingSlider {
    base: Base<HSlider>,

    /// 绑定协议：volume:<Bus> / custom:<key>
    #[export]
    bind: GString,
    /// custom 键缺失时的默认值
    #[export]
    default_value: f64,

    /// 程序化设值期间抑制回写（防初值回环）
    muting: bool,
}

#[godot_api]
impl IHSlider for GdUISettingSlider {
    fn init(base: Base<HSlider>) -> Self {
        Self {
            base,
            bind: GString::new(),
            default_value: 0.0,
            muting: false,
        }
    }

    fn ready(&mut self) {
        let bind = self.bind.to_string();
        let default_value = self.default_value;
        let cur = read_bind_f64(&bind, default_value);
        self.muting = true;
        self.base_mut().set_value(cur);
        self.muting = false;
        let cb = Callable::from_object_method(&*self.base(), "_on_value_changed");
        self.base_mut()
            .connect(&StringName::from("value_changed"), &cb);
    }
}

#[godot_api]
impl GdUISettingSlider {
    #[signal]
    fn s_value_changed(value: f64);

    #[func]
    fn _on_value_changed(&mut self, value: f64) {
        if self.muting {
            return;
        }
        let bind = self.bind.to_string();
        apply_bind_f64(&bind, value);
        self.base_mut().emit_signal(
            &StringName::from("s_value_changed"),
            &[value.to_variant()],
        );
    }
}

// ---------------------------------------------------------------------------
// SettingSwitch —— 布尔接口组件（bind → fullscreen / vsync / custom:<key>）
// ---------------------------------------------------------------------------

#[derive(GodotClass)]
#[class(base = Button, tool)]
pub struct GdUISettingSwitch {
    base: Base<Button>,

    /// 文本标签（自定义绘制）
    #[export]
    label: GString,
    /// 绑定协议：fullscreen / vsync / custom:<key>
    #[export]
    bind: GString,
    /// custom 键缺失时的默认值
    #[export]
    default_value: bool,
    #[export]
    on_color: Color,
    #[export]
    off_color: Color,
    #[export]
    knob_color: Color,
    #[export]
    label_color: Color,

    /// 当前开关状态（GML 声明的 checked 初值在 ready 前生效）
    #[export]
    checked: bool,
}

#[godot_api]
impl IButton for GdUISettingSwitch {
    fn init(base: Base<Button>) -> Self {
        Self {
            base,
            label: GString::new(),
            bind: GString::new(),
            default_value: false,
            on_color: Color::from_rgb(0.55, 0.72, 0.42),
            off_color: Color::from_rgb(0.32, 0.3, 0.33),
            knob_color: Color::from_rgb(0.95, 0.92, 0.84),
            label_color: Color::from_rgb(0.16, 0.14, 0.1),
            checked: false,
        }
    }

    fn ready(&mut self) {
        // 读当前绑定值作为初值（headless 下 fullscreen 等只持久化不应用，读侧仍准确）
        let bind = self.bind.to_string();
        let cur = read_bind_bool(&bind, self.default_value);
        self.checked = cur;
        {
            let mut btn = self.to_gd().upcast::<Button>();
            btn.set_toggle_mode(true);
            btn.set_pressed_no_signal(cur);
            btn.set_custom_minimum_size(Vector2::new(140.0, 34.0));
            clear_button_styleboxes(&mut btn);
        }
        let cb = Callable::from_object_method(&*self.base(), "_on_toggled");
        self.base_mut()
            .connect(&StringName::from("toggled"), &cb);
    }

    fn draw(&mut self) {
        let label = self.label.to_string();
        let mut btn = self.to_gd().upcast::<Button>();
        draw_switch_visual(
            &mut btn,
            &label,
            if self.checked { 1.0 } else { 0.0 },
            self.on_color,
            self.off_color,
            self.knob_color,
            self.label_color,
        );
    }
}

#[godot_api]
impl GdUISettingSwitch {
    #[signal]
    fn s_toggled(checked: bool);

    #[func]
    fn _on_toggled(&mut self, pressed: bool) {
        self.checked = pressed;
        self.base_mut().queue_redraw();
        let bind = self.bind.to_string();
        apply_bind_bool(&bind, pressed);
        self.base_mut()
            .emit_signal(&StringName::from("s_toggled"), &[pressed.to_variant()]);
    }
}

// ---------------------------------------------------------------------------
// SettingSelect —— 自定义字符串设置下拉（bind → custom:<key>，即时持久化）
// ---------------------------------------------------------------------------

#[derive(GodotClass)]
#[class(base = OptionButton, tool)]
pub struct GdUISettingSelect {
    base: Base<OptionButton>,

    /// 选项列表（逗号分隔）
    #[export]
    options: GString,
    /// 当前值（选项文本）
    #[export]
    value: GString,
    /// 绑定协议：custom:<key>
    #[export]
    bind: GString,
    #[export]
    label_color: Color,
}

#[godot_api]
impl IOptionButton for GdUISettingSelect {
    fn init(base: Base<OptionButton>) -> Self {
        Self {
            base,
            options: GString::new(),
            value: GString::new(),
            bind: GString::new(),
            label_color: Color::from_rgb(0.2, 0.18, 0.14),
        }
    }

    fn ready(&mut self) {
        // custom 键缺省时回退 value 属性（GML 声明的默认项）
        let bind = self.bind.to_string();
        let declared = self.value.to_string();
        let stored = match parse_bind(&bind) {
            SettingBind::Custom(key) => with_setting(|s| {
                s.bind().get_value(
                    GString::from(key.as_str()),
                    GString::from(declared.as_str()).to_variant(),
                )
            })
            .and_then(|v| v.try_to::<GString>().ok())
            .map(|g| g.to_string()),
            _ => None,
        }
        .unwrap_or(declared);

        let options = self.options.to_string();
        {
            let mut ob = self.to_gd().upcast::<OptionButton>();
            style_option_button(&mut ob, self.label_color);
            populate_option_button(&mut ob, &options, &stored);
        }
        let selected = self.base().get_selected();
        if selected >= 0 {
            self.value = self.base().get_item_text(selected);
        }
        let cb = Callable::from_object_method(&*self.base(), "_on_item_selected");
        self.base_mut()
            .connect(&StringName::from("item_selected"), &cb);
    }
}

#[godot_api]
impl GdUISettingSelect {
    #[signal]
    fn s_value_changed(value: GString);

    #[func]
    fn _on_item_selected(&mut self, index: i64) {
        let text = self.base().get_item_text(index as i32);
        self.value = text.clone();
        let bind = self.bind.to_string();
        if let SettingBind::Custom(key) = parse_bind(&bind) {
            let _ = with_setting(|s| {
                s.bind_mut()
                    .set_value(GString::from(key.as_str()), text.to_variant())
            });
        }
        self.base_mut()
            .emit_signal(&StringName::from("s_value_changed"), &[text.to_variant()]);
    }
}
