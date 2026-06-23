// GdViewCamera - 相机管理器
// 继承 Camera2D，提供相机跟随、震动、缩放、旋转、平移、径向模糊、帧冻结等功能
// 移植自 C++ manager/camera.h/camera.cpp
// 使用 EaseMover 实现平滑动画，使用 Tween 实现特效动画
// 通过 GdInput 管理键盘移动输入，通过内联 Shader 实现径向模糊效果

use godot::prelude::*;
use godot::global::*;
use godot::builtin::{GString, VarArray, Vector2, Vector4, StringName, NodePath};
use godot::classes::{
    ICamera2D, Camera2D, Node2D, CanvasLayer, ColorRect,
    Shader, ShaderMaterial, Engine, Input, InputEvent,
    InputEventMouseMotion, InputEventScreenDrag, ProjectSettings,
    control::LayoutPreset, tween::{TransitionType, EaseType},
};
use godot::global::{
    randf_range, move_toward, lerpf, lerp_angle, deg_to_rad,
    fposmod, clampf, wrapf, Key, MouseButton,
};
use godot::obj::{NewGd, EngineEnum};
use godot::builtin::Side;

use crate::anim::easy_move::EaseMover;
use crate::manager::input::GdInput;

#[derive(GodotClass)]
#[class(base = Camera2D)]
pub struct GdViewCamera {
    /// 相机移动速度
    #[export]
    move_speed: f64,

    /// 设计尺寸
    #[var(pub)]
    design_size: Vector2,

    /// 震动恢复速度
    #[export]
    shake_recover_speed: f64,

    /// 锁定距离
    #[var(pub)]
    lock_distance: f64,

    /// 最小缩放
    #[export]
    zoom_min: f64,

    /// 最大缩放
    #[export]
    zoom_max: f64,

    /// 缩放动作名称
    #[export]
    zoom_action: GString,

    /// 旋转偏移
    #[var(pub)]
    turn_offset: Vector2,

    /// 是否启用键盘移动
    #[export]
    #[var(get = get_enable_keyboard_move, set = set_enable_keyboard_move)]
    enable_keyboard_move: bool,

    /// 是否启用鼠标移动
    #[export]
    enable_mouse_move: bool,

    /// 跟随节点
    #[var(get = get_follow_node, set = set_follow_node)]
    follow_node: Option<Gd<Node2D>>,

    /// 目标位置
    #[var(pub)]
    target_pos: Vector2,

    /// 震动强度
    shake_strength: f32,

    /// 默认边界限制
    default_limit_left: i32,
    default_limit_right: i32,
    default_limit_top: i32,
    default_limit_bottom: i32,

    /// 平移动画
    follow_pan_ease: EaseMover,
    /// 是否正在平移
    is_move_panning: bool,
    /// 是否正在移动
    is_moving: bool,
    /// 跟随位置
    follow_position: Vector2,

    /// 相机状态
    is_shake: bool,
    is_rotating: bool,

    /// 旋转动画
    turn_ease: EaseMover,
    turn_from: f64,
    turn_to: f64,
    turn_sign: f64,

    /// 缩放动画
    zoom_ease: EaseMover,
    is_zoom: bool,
    zoom_from: f64,
    zoom_to: f64,
    zoom_step: i64,
    zoom_steps: f64,

    /// 帧冻结
    is_frame_freeze: bool,

    /// 输入管理器
    input_manager: Option<Gd<GdInput>>,

    /// 拖动位置
    last_drag_position: Vector2,

    /// 径向模糊
    radial_canvas: Option<Gd<CanvasLayer>>,
    radial_blur: Option<Gd<ColorRect>>,
    is_blur: bool,
    blur_scale: f32,

    base: Base<Camera2D>,
}

#[godot_api]
impl ICamera2D for GdViewCamera {
    fn init(base: Base<Camera2D>) -> Self {
        Self {
            move_speed: 4.8,
            design_size: Vector2::new(1152.0, 648.0),
            shake_recover_speed: 16.0,
            lock_distance: 16.0,
            zoom_min: 1.0,
            zoom_max: 2.5,
            zoom_action: GString::new(),
            turn_offset: Vector2::ZERO,
            enable_keyboard_move: false,
            enable_mouse_move: false,
            follow_node: None,
            target_pos: Vector2::ZERO,
            shake_strength: 0.0,
            default_limit_left: 0,
            default_limit_right: 0,
            default_limit_top: 0,
            default_limit_bottom: 0,
            follow_pan_ease: EaseMover::new(2.0, 0.0, Vector2::ZERO, Vector2::ZERO, None),
            is_move_panning: false,
            is_moving: false,
            follow_position: Vector2::ZERO,
            is_shake: false,
            is_rotating: false,
            turn_ease: EaseMover::new(0.5, 0.0, Vector2::ZERO, Vector2::ZERO, None),
            turn_from: 0.0,
            turn_to: 0.0,
            turn_sign: 1.0,
            zoom_ease: EaseMover::new(0.5, 0.0, Vector2::ZERO, Vector2::ZERO, None),
            is_zoom: false,
            zoom_from: 0.0,
            zoom_to: 0.0,
            zoom_step: 0,
            zoom_steps: 2.0,
            is_frame_freeze: false,
            input_manager: None,
            last_drag_position: Vector2::ZERO,
            radial_canvas: None,
            radial_blur: None,
            is_blur: false,
            blur_scale: 1.0,
            base,
        }
    }

    fn ready(&mut self) {
        // 保存默认边界限制
        self.default_limit_left = self.base().get_limit(Side::LEFT);
        self.default_limit_right = self.base().get_limit(Side::RIGHT);
        self.default_limit_top = self.base().get_limit(Side::TOP);
        self.default_limit_bottom = self.base().get_limit(Side::BOTTOM);

        self.target_pos = self.base().get_global_position();

        // 创建径向模糊层
        self.create_radial_layer();

        // 创建输入管理器并注册键盘动作
        let mut input_manager = Gd::<GdInput>::from_init_fn(|base| {
            <GdInput as INode2D>::init(base)
        });
        self.base_mut().add_child(&input_manager);

        // 注册键盘移动动作
        let mut key_left = VarArray::new();
        key_left.push(&(Key::A.ord() as i64).to_variant());
        let mut key_right = VarArray::new();
        key_right.push(&(Key::D.ord() as i64).to_variant());
        let empty_joy = VarArray::new();

        let cb_left = Callable::from_object_method(&*self.base_mut(), "move_camera_left");
        let cb_right = Callable::from_object_method(&*self.base_mut(), "move_camera_right");
        let noop = Callable::invalid();

        input_manager.call(
            &StringName::from("register_virtual_action"),
            &[
                GString::from("camera_move_left").to_variant(),
                key_left.to_variant(),
                empty_joy.to_variant(),
                cb_left.to_variant(),
                noop.to_variant(),
            ],
        );
        input_manager.call(
            &StringName::from("register_virtual_action"),
            &[
                GString::from("camera_move_right").to_variant(),
                key_right.to_variant(),
                empty_joy.to_variant(),
                cb_right.to_variant(),
                noop.to_variant(),
            ],
        );

        self.input_manager = Some(input_manager);

        // 设置设计尺寸和位置
        let screen_size = Self::get_default_screen_size();
        self.design_size = screen_size;
        self.base_mut().set_global_position(screen_size * 0.5);
        self.base_mut().set_offset(Vector2::ZERO);

        // 平滑设置
        self.base_mut().set_limit_smoothing_enabled(true);
        self.base_mut().set_position_smoothing_enabled(true);
        self.base_mut().set_position_smoothing_speed(5.0);
        self.base_mut().set_ignore_rotation(false);
        let zoom_min = self.zoom_min as f32;
        self.base_mut().set_zoom(Vector2::ONE * zoom_min);
        self.adjust_zoom();

        // 连接 viewport size_changed 信号
        if let Some(mut viewport) = self.base().get_viewport() {
            let callable = Callable::from_object_method(&*self.base_mut(), "adjust_zoom");
            let _ = viewport.connect("size_changed", &callable);
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        // 处理缩放动作
        if !self.zoom_action.is_empty() {
            let action_sn = StringName::from(&self.zoom_action);
            if event.is_action_pressed(&action_sn) {
                self.start_zoom_public(self.zoom_step + 1, -1.0, -1.0);
            }
        }

        // 处理鼠标移动
        if self.enable_mouse_move {
            let class_name = event.get_class().to_string();
            if class_name == "InputEventScreenDrag" {
                if let Ok(drag_event) = event.try_cast::<InputEventScreenDrag>() {
                    let drag_delta = drag_event.get_relative();
                    self.adjust_camera_position(drag_delta);
                }
            } else if class_name == "InputEventMouseMotion" {
                let input = Input::singleton();
                if input.is_mouse_button_pressed(MouseButton::LEFT) {
                    if let Ok(motion_event) = event.try_cast::<InputEventMouseMotion>() {
                        let motion_delta = motion_event.get_relative();
                        self.adjust_camera_position(motion_delta);
                    }
                }
            }
        }
    }

    fn process(&mut self, delta: f64) {
        // 处理平移
        if self.is_move_panning {
            let pos = self.follow_pan_ease.move_node(delta, true, true);
            self.base_mut().set_global_position(pos);
            if self.follow_pan_ease.is_complete() {
                self.is_move_panning = false;
                self.base_mut().emit_signal("s_pan_ed", &[]);
            }
        } else {
            // 跟踪目标
            let follow_node = self.follow_node.clone();
            if let Some(follow_node) = follow_node {
                if follow_node.is_instance_valid() {
                    self.follow_position = follow_node.get_global_position();
                    let current_pos = self.base().get_global_position();
                    let rotation = self.base().get_rotation();
                    let target = self.follow_position + self.turn_offset.rotated(rotation);
                    let new_pos = current_pos.lerp(target, (self.move_speed * delta) as f32);
                    self.base_mut().set_global_position(new_pos);
                } else {
                    self.follow_node = None;
                }
            }
        }

        // 处理震动
        if self.is_shake {
            let offset_x = randf_range(-self.shake_strength as f64, self.shake_strength as f64) as f32;
            let offset_y = randf_range(-self.shake_strength as f64, self.shake_strength as f64) as f32;
            self.base_mut().set_offset(Vector2::new(offset_x, offset_y));
            self.shake_strength = move_toward(
                self.shake_strength as f64,
                0.0,
                self.shake_recover_speed * delta,
            ) as f32;
            if self.shake_strength <= 0.001 {
                self.is_shake = false;
            }
        }

        // 处理缩放
        if self.is_zoom {
            let progress = self.zoom_ease.count(delta, true, true);
            let zoom_val = lerpf(self.zoom_from, self.zoom_to, progress) as f32;
            self.base_mut().set_zoom(Vector2::ONE * zoom_val);
            if self.zoom_ease.is_complete() {
                self.is_zoom = false;
            }
        }

        // 处理旋转
        if self.is_rotating {
            if !self.turn_ease.is_complete() {
                let progress = self.turn_ease.count(delta, true, true);
                let rotation = lerp_angle(self.turn_from, self.turn_to, progress);
                self.base_mut().set_rotation(rotation as f32);
                self.base_mut().emit_signal("s_turning", &[rotation.to_variant()]);

                // 径向模糊
                if let Some(ref radial_blur) = self.radial_blur.clone() {
                    let w = absf(wrapf(self.turn_ease.smooth() * 2.0, -1.0, 1.0));
                    let t = lerp_angle(self.turn_from, self.turn_to, 1.0) - self.turn_from;
                    let mat = radial_blur.get_material();
                    if let Some(mat) = mat {
                        if let Ok(shader_mat) = mat.try_cast::<ShaderMaterial>() {
                            let mut shader_mat = shader_mat;
                            shader_mat.set_shader_parameter(
                                &StringName::from("blur_angle"),
                                &(t * self.blur_scale as f64 * delta * w).to_variant(),
                            );
                        }
                    }
                }
            } else {
                self.is_rotating = false;
            }
        }
    }
}

#[godot_api]
impl GdViewCamera {
    /// 缩放信号
    #[signal]
    fn s_zoom();

    /// 移动信号
    #[signal]
    fn s_move();

    /// 旋转信号
    #[signal]
    fn s_turning(angle: f64);

    /// 平移完成信号
    #[signal]
    fn s_pan_ed();

    #[func]
    fn get_enable_keyboard_move(&self) -> bool {
        self.enable_keyboard_move
    }

    #[func]
    fn set_enable_keyboard_move(&mut self, value: bool) {
        self.enable_keyboard_move = value;
        if let Some(ref mut input_manager) = self.input_manager {
            input_manager.call("set_enable", &[value.to_variant()]);
        }
    }

    #[func]
    fn get_follow_node(&self) -> Option<Gd<Node2D>> {
        self.follow_node.clone()
    }

    #[func]
    fn set_follow_node(&mut self, value: Option<Gd<Node2D>>) {
        self.follow_node = value;
    }

    /// 相机震动
    /// amount: 震动强度
    #[func]
    fn shake(&mut self, amount: f64) {
        self.shake_strength = amount as f32;
        self.is_shake = true;
    }

    /// 打击效果
    /// scale: 缩放比例
    /// offset: 偏移量
    #[func]
    fn effect_hit(&mut self, scale: Vector2, offset: Vector2) {
        let ori_offset = self.base().get_offset();
        let ori_zoom = self.base().get_zoom();
        let target_offset = offset + ori_offset;

        let mut tween = self.base_mut().create_tween();
        let camera_obj = self.base_mut().clone().upcast::<godot::classes::Object>();
        tween.tween_property(
            &camera_obj,
            &NodePath::from("offset"),
            &ori_offset.to_variant(),
            0.12,
        ).from(&target_offset.to_variant());

        let mut tween2 = tween.parallel();
        let from_zoom = Vector2::new(ori_zoom.x * scale.x, ori_zoom.y * scale.y);
        tween2.tween_property(
            &camera_obj,
            &NodePath::from("zoom"),
            &ori_zoom.to_variant(),
            0.12,
        ).from(&from_zoom.to_variant());
    }

    /// 帧冻结
    /// scale: 时间缩放
    /// duration: 持续时间
    #[func]
    fn frame_freeze(&mut self, scale: f64, duration: f64) {
        if self.is_frame_freeze {
            return;
        }
        self.is_frame_freeze = true;
        Engine::singleton().set_time_scale(scale);
        let mut tree = self.base().get_tree();
        let mut timer = tree.create_timer(duration * scale);
        let callable = Callable::from_object_method(&*self.base_mut(), "frame_freeze_end");
        let _ = timer.connect("timeout", &callable);
    }

    /// 帧冻结结束
    #[func]
    fn frame_freeze_end(&mut self) {
        Engine::singleton().set_time_scale(1.0);
        self.is_frame_freeze = false;
    }

    /// 视图居中
    #[func]
    fn view_center(&mut self) {
        let screen_size = Self::get_default_screen_size();
        self.base_mut().set_position(screen_size * 0.5);
    }

    /// 更新边界限制
    /// limits: Vector4(left, right, top, bottom)
    #[func]
    fn update_limit(&mut self, limits: Vector4) {
        if limits.x != 0.0 {
            self.base_mut().set_limit(Side::LEFT, limits.x as i32);
        }
        if limits.y != 0.0 {
            self.base_mut().set_limit(Side::RIGHT, limits.y as i32);
        }
        if limits.z != 0.0 {
            self.base_mut().set_limit(Side::TOP, limits.z as i32);
        }
        if limits.w != 0.0 {
            self.base_mut().set_limit(Side::BOTTOM, limits.w as i32);
        }
    }

    /// 禁用边界限制
    #[func]
    fn disable_limit(&mut self) {
        self.base_mut().set_limit(Side::RIGHT, 99999);
        self.base_mut().set_limit(Side::BOTTOM, 99999);
        self.base_mut().set_limit(Side::LEFT, -99999);
        self.base_mut().set_limit(Side::TOP, -99999);
    }

    /// 重置边界限制
    #[func]
    fn reset_limit(&mut self) {
        let right = self.default_limit_right;
        let bottom = self.default_limit_bottom;
        let left = self.default_limit_left;
        let top = self.default_limit_top;
        self.base_mut().set_limit(Side::RIGHT, right);
        self.base_mut().set_limit(Side::BOTTOM, bottom);
        self.base_mut().set_limit(Side::LEFT, left);
        self.base_mut().set_limit(Side::TOP, top);
    }

    /// 平移到指定位置
    /// pos: 目标位置
    #[func]
    fn pan(&mut self, pos: Vector2) {
        self.follow_pan_ease.clock = 0.0;
        self.is_move_panning = true;
        self.follow_pan_ease.from = self.base().get_global_position();
        self.follow_pan_ease.to = pos;
        let distance = self.follow_pan_ease.from.distance_to(self.follow_pan_ease.to);
        let time = lerpf(0.3, 1.0, clampf(distance as f64 / 100.0, 0.0, 20.0) / 20.0);
        self.follow_pan_ease.time = time;
    }

    /// 瞬间移动到指定位置
    /// pos: 目标位置
    /// turn_data: 旋转角度
    #[func]
    fn snap_to(&mut self, pos: Vector2, turn_data: f32) {
        self.base_mut().set_global_position(pos);
        self.target_pos = pos;
        self.turn_from = turn_data as f64;
        self.turn_to = turn_data as f64;
        self.base_mut().set_rotation(turn_data);
        self.turn_ease.clock = 99.0;
        self.base_mut().reset_smoothing();
        self.base_mut().force_update_scroll();
        self.base_mut().force_update_transform();
    }

    /// 瞬间移动到跟随节点
    #[func]
    fn snap_follow(&mut self) {
        let follow_node = self.follow_node.clone();
        if let Some(follow_node) = follow_node {
            if follow_node.is_instance_valid() {
                let pos = follow_node.get_global_position();
                let rotation = follow_node.get_rotation();
                self.snap_to(pos, rotation);
            }
        }
    }

    /// 设置径向模糊
    /// val: 模糊值（>0 启用，0 禁用）
    #[func]
    fn blur(&mut self, val: f32) {
        self.is_blur = val > 0.0;
        if let Some(ref mut radial_canvas) = self.radial_canvas {
            radial_canvas.set_visible(self.is_blur);
        }

        let a = fposmod(val as f64, 8.0) as i64;
        let blur_scales: [f32; 8] = [0.0, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 70.0];
        let bsteps_values: [f32; 8] = [1.0, 3.0, 4.0, 8.0, 8.0, 8.0, 12.0, 20.0];
        self.blur_scale = blur_scales[a as usize];
        let bsteps = bsteps_values[a as usize];

        if let Some(ref radial_blur) = self.radial_blur.clone() {
            let mat = radial_blur.get_material();
            if let Some(mat) = mat {
                if let Ok(shader_mat) = mat.try_cast::<ShaderMaterial>() {
                    let mut shader_mat = shader_mat;
                    shader_mat.set_shader_parameter(
                        &StringName::from("steps"),
                        &bsteps.to_variant(),
                    );
                }
            }
        }
    }

    /// 旋转相机
    /// target: 目标角度（度数，累加）
    #[func]
    fn turn(&mut self, target: f64) {
        self.turn_from = self.base().get_rotation() as f64;
        self.turn_to = deg_to_rad(target);
        if self.turn_from != self.turn_to {
            self.is_rotating = true;
            self.turn_ease.clock = 0.0;
            let end_angle = lerp_angle(self.turn_from, self.turn_to, 1.0);
            if self.turn_from < end_angle {
                self.turn_sign = 1.0;
            } else {
                self.turn_sign = -1.0;
            }
        }
    }

    /// 开始缩放
    /// arg: 缩放步骤（-1 表示下一步）
    /// zmin: 最小缩放（-1 使用默认值）
    /// zmax: 最大缩放（-1 使用默认值）
    #[func]
    fn start_zoom(&mut self, arg: i64, zmin: f32, zmax: f32) {
        let mut arg = arg;
        if arg == -1 {
            arg = self.zoom_step + 1;
        }
        let zmin = if (zmin - (-1.0)).abs() < 0.00001 {
            self.zoom_min as f32
        } else {
            zmin
        };
        let zmax = if (zmax - (-1.0)).abs() < 0.00001 {
            self.zoom_max as f32
        } else {
            zmax
        };
        self.zoom_step = fposmod(arg as f64, self.zoom_steps + 1.0) as i64;
        self.is_zoom = true;
        self.zoom_ease.clock = 0.0;
        self.zoom_from = self.base().get_zoom().x as f64;
        let frac = self.zoom_step as f64 / self.zoom_steps;
        self.zoom_to = lerpf(zmin as f64, zmax as f64, frac);
        self.base_mut().emit_signal("s_zoom", &[]);
    }

    /// 重置缩放
    #[func]
    fn reset_zoom(&mut self) {
        self.is_zoom = false;
        self.zoom_step = 0;
        let zoom_min = self.zoom_min as f32;
        self.base_mut().set_zoom(Vector2::ONE * zoom_min);
        self.zoom_to = self.zoom_min;
    }

    /// 聚焦目标节点
    /// target: 目标节点
    /// duration: 动画持续时间
    #[func]
    fn focus(&mut self, target: Gd<Node2D>, duration: f32) {
        if !target.is_instance_valid() {
            return;
        }

        let camera_global = self.base().get_global_position();
        let target_global = target.get_global_position();
        let new_camera_offset = target_global - camera_global;

        let mut tween = self.base_mut().create_tween();
        let camera_obj = self.base_mut().clone().upcast::<godot::classes::Object>();
        tween.tween_property(
            &camera_obj,
            &NodePath::from("offset"),
            &new_camera_offset.to_variant(),
            duration as f64,
        );
        tween.set_trans(TransitionType::CUBIC);
        tween.set_ease(EaseType::IN_OUT);
    }

    /// 取消聚焦
    #[func]
    fn unfocus(&mut self) {
        self.base_mut().set_offset(Vector2::ZERO);
    }

    /// 跟随目标节点
    /// target: 目标节点
    /// now: 是否立即移动到目标位置
    /// do_enable: 是否启用相机
    #[func]
    fn follow(&mut self, target: Gd<Node2D>, now: bool, do_enable: bool) {
        if !target.is_instance_valid() {
            return;
        }

        if do_enable {
            self.base_mut().set_enabled(do_enable);
        }
        if now {
            self.base_mut().set_position_smoothing_enabled(false);
            let target_pos = target.get_global_position();
            self.base_mut().set_global_position(target_pos);
            self.base_mut().set_position_smoothing_enabled(true);
        }
        self.follow_node = Some(target);
    }

    /// 取消跟随
    /// disable: 是否禁用相机
    #[func]
    fn unfollow(&mut self, disable: bool) {
        if disable {
            self.base_mut().set_enabled(false);
        }
        self.follow_node = None;
    }

    /// 水平波动动画
    /// duration: 动画持续时间
    #[func]
    fn wave_h(&mut self, duration: f32) {
        let ori_pos = self.base().get_offset();
        let top_pos = ori_pos + Vector2::new(0.0, 100.0);
        let bottom_pos = ori_pos - Vector2::new(0.0, 100.0);

        let mut tween = self.base_mut().create_tween();
        let camera_obj = self.base_mut().clone().upcast::<godot::classes::Object>();
        tween.tween_property(&camera_obj, &NodePath::from("offset"), &top_pos.to_variant(), duration as f64);
        tween.tween_property(&camera_obj, &NodePath::from("offset"), &bottom_pos.to_variant(), (duration * 2.0) as f64);
        tween.tween_property(&camera_obj, &NodePath::from("offset"), &ori_pos.to_variant(), duration as f64);
        tween.set_trans(TransitionType::CUBIC);
        tween.set_ease(EaseType::IN_OUT);
    }

    /// 垂直波动动画
    /// duration: 动画持续时间
    #[func]
    fn wave_v(&mut self, duration: f32) {
        let ori_pos = self.base().get_offset();
        let right_pos = ori_pos + Vector2::new(100.0, 0.0);
        let left_pos = ori_pos - Vector2::new(100.0, 0.0);

        let mut tween = self.base_mut().create_tween();
        let camera_obj = self.base_mut().clone().upcast::<godot::classes::Object>();
        tween.tween_property(&camera_obj, &NodePath::from("offset"), &right_pos.to_variant(), duration as f64);
        tween.tween_property(&camera_obj, &NodePath::from("offset"), &left_pos.to_variant(), (duration * 2.0) as f64);
        tween.tween_property(&camera_obj, &NodePath::from("offset"), &ori_pos.to_variant(), duration as f64);
        tween.set_trans(TransitionType::CUBIC);
        tween.set_ease(EaseType::IN_OUT);
    }

    /// 场景切换前清理
    #[func]
    fn clean_before_change_scnene(&mut self) {
        if self.follow_node.is_some() {
            self.unfollow(false);
        }
        self.reset_limit();
    }

    /// 向左移动相机（由输入回调调用）
    #[func]
    fn move_camera_left(&mut self) {
        let cur_pos = self.base().get_position();
        let move_distance = self.move_speed as f32 * 10.0;
        if cur_pos.x >= self.design_size.x / 2.0 - move_distance {
            self.base_mut().set_position(Vector2::new(cur_pos.x - move_distance, cur_pos.y));
        }
    }

    /// 向右移动相机（由输入回调调用）
    #[func]
    fn move_camera_right(&mut self) {
        let diff = self.base().get_limit(Side::RIGHT) as f32 - self.design_size.x;
        let cur_pos = self.base().get_position();
        let move_distance = self.move_speed as f32 * 10.0;
        if cur_pos.x <= self.design_size.x / 2.0 + diff + move_distance {
            self.base_mut().set_position(Vector2::new(cur_pos.x + move_distance, cur_pos.y));
        }
    }

    /// 调整缩放（由 viewport size_changed 信号调用）
    #[func]
    fn adjust_zoom(&mut self) {
        self.reset_limit();
        let viewport_size = self.base().get_viewport_rect().size;
        let zoom_x = viewport_size.x / self.design_size.x;
        let zoom_y = viewport_size.y / self.design_size.y;
        if zoom_x == 0.0 || zoom_y == 0.0 {
            return;
        }

        if zoom_x > zoom_y {
            self.base_mut().set_zoom(Vector2::new(zoom_x, zoom_x));
        } else {
            self.base_mut().set_zoom(Vector2::new(zoom_y, zoom_y));
        }
    }
}

/// 私有方法实现
impl GdViewCamera {
    /// 创建径向模糊层
    fn create_radial_layer(&mut self) {
        let mut radial_blur = ColorRect::new_alloc();
        // 设置全屏
        radial_blur.set_anchors_preset(LayoutPreset::FULL_RECT);

        // 创建 ShaderMaterial
        let mut shader_material = ShaderMaterial::new_gd();
        let mut shader = Shader::new_gd();

        let shader_code = r#"
shader_type canvas_item;
render_mode blend_mix;

uniform sampler2D SCREEN_TEXTURE : hint_screen_texture, filter_linear_mipmap;

uniform float blur_angle : hint_range(-10, 10) = 0.0;
uniform float blur_offset : hint_range(-1.0, 1.0) = 0.0;
uniform float steps : hint_range(1.0, 30.0) = 8.0;
uniform float aspect : hint_range(0.0, 2.0) = 0.5625;

void fragment() {
    vec2 uv = SCREEN_UV;
    uv.y *= aspect;
    vec2 center = vec2(0.5, 0.5 * aspect);
    float angle = atan(uv.y - center.y, uv.x - center.x);
    float dist = distance(uv, center);
    vec3 color = vec3(0.0);
    for (float i = 0.0; i < steps; i += 1.0) {
        float _angle = angle + (blur_angle * blur_offset) + mix(-blur_angle, blur_angle, i / ceil(steps));
        vec2 tuv = vec2(cos(_angle), sin(_angle)) * dist + center;
        tuv.y /= aspect;
        color += texture(SCREEN_TEXTURE, tuv).rgb / ceil(steps);
    }
    
    COLOR.rgb = color;
}
"#;
        shader.set_code(&GString::from(shader_code));
        shader_material.set_shader(&shader);

        // 使用 call 设置 material（ByOption 类型不匹配）
        radial_blur.call("set_material", &[shader_material.to_variant()]);

        let mut radial_canvas = CanvasLayer::new_alloc();
        radial_canvas.add_child(&radial_blur);
        self.base_mut().add_child(&radial_canvas);

        radial_canvas.set_visible(self.is_blur);

        self.radial_blur = Some(radial_blur);
        self.radial_canvas = Some(radial_canvas);
    }

    /// 调整相机位置（拖动）
    fn adjust_camera_position(&mut self, delta: Vector2) {
        let mut new_position = self.base().get_position();
        new_position.x -= delta.x;
        self.base_mut().set_position(new_position);
    }

    /// 获取默认屏幕尺寸
    fn get_default_screen_size() -> Vector2 {
        let project_settings = ProjectSettings::singleton();
        let width = project_settings.get_setting(&GString::from("display/window/size/viewport_width"));
        let height = project_settings.get_setting(&GString::from("display/window/size/viewport_height"));
        Vector2::new(width.to::<i32>() as f32, height.to::<i32>() as f32)
    }

    /// start_zoom 的公开包装方法（供 input 内部调用）
    fn start_zoom_public(&mut self, arg: i64, zmin: f32, zmax: f32) {
        self.start_zoom(arg, zmin, zmax);
    }
}
