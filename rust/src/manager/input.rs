// GdInput - 输入管理器
// 继承 Node2D，提供虚拟动作注册、键盘/手柄组合按键检测、鼠标点击/滑动检测、轴映射功能
// 移植自 C++ manager/input.h/input.cpp
// 通过 _process 轮询检测组合按键状态，通过 _input 处理鼠标事件

use std::collections::HashMap;
use std::sync::OnceLock;

use parking_lot::Mutex;

use godot::prelude::*;
use godot::builtin::{GString, Variant, VarArray, VarDictionary, Vector2, StringName};
use godot::classes::{
    INode2D, Node2D, Node, Input, InputMap, InputEvent, InputEventKey,
    InputEventJoypadButton, InputEventMouseButton, InputEventMouseMotion,
    Area2D, CollisionShape2D, Shape2D,
};
use godot::global::{Key, MouseButton, JoyButton};
use godot::obj::NewGd;

/// 全局 GdInput 实例注册表（id -> instance_id）
static INPUT_INSTANCES: OnceLock<Mutex<HashMap<String, InstanceId>>> = OnceLock::new();

fn instances() -> &'static Mutex<HashMap<String, InstanceId>> {
    INPUT_INSTANCES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct GdInput {
    /// 是否启用输入检测
    #[var(get = get_enable, set = set_enable)]
    enable: bool,

    /// 输入节点 ID（用于跨实例查找）
    #[var(pub)]
    id: GString,

    /// 滑动检测死区（像素）
    #[var(pub)]
    swipe_deadzone: i64,

    /// 是否监听鼠标进入事件
    #[var(get = get_watch_mouse_enter, set = set_watch_mouse_enter)]
    watch_mouse_enter: bool,

    /// 是否监听鼠标点击事件
    #[var(get = get_watch_mouse_click, set = set_watch_mouse_click)]
    watch_mouse_click: bool,

    /// 鼠标检测区域形状
    #[var(get = get_watch_mouse_area, set = set_watch_mouse_area)]
    watch_mouse_area: Option<Gd<Shape2D>>,

    /// 已注册的动作名列表
    action_names: VarArray,
    /// 虚拟动作映射 (action_name -> Array<InputEvent>)
    virtual_actions: VarDictionary,
    /// 轴映射 (axis_name -> Dictionary{positive_x, negative_x, positive_y, negative_y})
    axis_mappings: VarDictionary,
    /// 动作状态 (action_name -> bool)
    action_states: VarDictionary,
    /// 按下回调映射 (action_name -> Callable)
    fn_mappings_press: VarDictionary,
    /// 松开回调映射 (action_name -> Callable)
    fn_mappings_release: VarDictionary,
    /// 长按状态 (action_name -> bool)
    hold_states: VarDictionary,
    /// 长按计时器 (action_name -> f64)
    hold_timers: VarDictionary,
    /// 长按间隔时间（秒）
    hold_interval: f64,

    /// 左键按下回调列表
    fn_lmouse_click: VarArray,
    /// 左键释放回调列表
    fn_lmouse_release: VarArray,
    /// 左键滑动回调列表
    fn_lmouse_swipe: VarArray,

    /// 左键按下起始位置
    lmouse_press_pos: Vector2,
    /// 左键是否按下
    lmouse_pressed: bool,
    /// 是否已检测到滑动（防止重复触发）
    swipe_detected: bool,

    /// 当前连接的手柄 ID 列表
    joypadids: Array<i64>,

    /// 是否挂在 Node2D 上（否则挂在 Control 上）
    is_mout_node2d: bool,
    /// 原始缩放
    ori_scale: Vector2,

    /// 鼠标检测区域
    area_: Option<Gd<Area2D>>,
    /// 碰撞形状
    coll: Option<Gd<CollisionShape2D>>,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GdInput {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            enable: true,
            id: GString::new(),
            swipe_deadzone: 50,
            watch_mouse_enter: false,
            watch_mouse_click: false,
            watch_mouse_area: None,
            action_names: VarArray::new(),
            virtual_actions: VarDictionary::new(),
            axis_mappings: VarDictionary::new(),
            action_states: VarDictionary::new(),
            fn_mappings_press: VarDictionary::new(),
            fn_mappings_release: VarDictionary::new(),
            hold_states: VarDictionary::new(),
            hold_timers: VarDictionary::new(),
            hold_interval: 0.1,
            fn_lmouse_click: VarArray::new(),
            fn_lmouse_release: VarArray::new(),
            fn_lmouse_swipe: VarArray::new(),
            lmouse_press_pos: Vector2::ZERO,
            lmouse_pressed: false,
            swipe_detected: false,
            joypadids: Array::new(),
            is_mout_node2d: false,
            ori_scale: Vector2::new(1.0, 1.0),
            area_: None,
            coll: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_process_mode(godot::classes::node::ProcessMode::ALWAYS);

        // 检查父节点类型
        if let Some(parent) = self.base().get_parent() {
            if parent.get_class() == GString::from("Node2D") {
                self.is_mout_node2d = true;
            }
        }

        if self.is_mout_node2d {
            self.init_mouse_watch_node2d();
        } else {
            self.init_mouse_watch_control();
        }

        // 注册到全局实例表
        let id_str = self.id.to_string();
        if !id_str.is_empty() {
            instances().lock().insert(id_str, self.base().instance_id());
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if !self.enable || !self.base().is_visible_in_tree() {
            return;
        }

        // 处理鼠标按键事件
        if let Ok(mouse_btn) = event.clone().try_cast::<InputEventMouseButton>() {
            if mouse_btn.get_button_index() == MouseButton::LEFT {
                if mouse_btn.is_pressed() {
                    self.lmouse_press_pos = mouse_btn.get_position();
                    self.lmouse_pressed = true;
                    self.swipe_detected = false;

                    for i in 0..self.fn_lmouse_click.len() {
                        let fn_var = self.fn_lmouse_click.at(i);
                        let callable = fn_var.to::<Callable>();
                        if callable.is_valid() {
                            let _ = callable.call(&[]);
                        }
                    }
                } else if mouse_btn.is_released() {
                    self.lmouse_pressed = false;
                    for i in 0..self.fn_lmouse_release.len() {
                        let fn_var = self.fn_lmouse_release.at(i);
                        let callable = fn_var.to::<Callable>();
                        if callable.is_valid() {
                            let _ = callable.call(&[]);
                        }
                    }
                }
            }
        }

        // 处理鼠标移动事件（滑动检测）
        if let Ok(mouse_motion) = event.try_cast::<InputEventMouseMotion>() {
            if self.lmouse_pressed && !self.swipe_detected {
                let current_pos = mouse_motion.get_position();
                let swipe_vector = current_pos - self.lmouse_press_pos;
                let distance = swipe_vector.length();

                if distance >= self.swipe_deadzone as f32 {
                    self.detect_swipe_direction(swipe_vector);
                    self.swipe_detected = true;
                }
            }
        }
    }

    fn process(&mut self, _delta: f64) {
        if !self.enable || !self.base().is_visible_in_tree() {
            return;
        }

        let input = Input::singleton();
        self.joypadids = input.get_connected_joypads();

        // 检查所有注册的动作组合键
        for i in 0..self.action_names.len() {
            let action_var = self.action_names.at(i);
            let action = action_var.to::<GString>();
            let action_str = action.to_string();

            let key_var = action.to_variant();
            let events_var = self.virtual_actions.get_or_nil(&key_var);
            if events_var.is_nil() {
                continue;
            }
            let events = events_var.to::<VarArray>();

            let mut all_keys_pressed = true;
            let mut goto_next = false;

            for j in 0..events.len() {
                let event_var = events.at(j);
                if self.joypadids.is_empty() && action_str.starts_with("key_") {
                    if let Ok(e) = event_var.try_to::<Gd<InputEventKey>>() {
                        if !input.is_key_pressed(e.get_keycode()) {
                            all_keys_pressed = false;
                            break;
                        }
                    } else {
                        goto_next = true;
                        break;
                    }
                } else if !self.joypadids.is_empty() && action_str.starts_with("joy_") {
                    if let Ok(e) = event_var.try_to::<Gd<InputEventJoypadButton>>() {
                        let button = e.get_button_index();
                        let device = e.get_device();
                        if !input.is_joy_button_pressed(device, button) {
                            all_keys_pressed = false;
                            break;
                        }
                    } else {
                        goto_next = true;
                        break;
                    }
                } else {
                    goto_next = true;
                    break;
                }
            }

            if goto_next {
                continue;
            }

            let state_var = self.action_states.get_or_nil(&key_var);
            let was_pressed = !state_var.is_nil() && state_var.to::<bool>();

            if all_keys_pressed && !was_pressed {
                self.action_states.set(&key_var, &true.to_variant());
                let fn_var = self.fn_mappings_press.get_or_nil(&key_var);
                let callable = fn_var.to::<Callable>();
                if callable.is_valid() {
                    let _ = callable.call(&[]);
                } else {
                    self.remove_virtual_input(action);
                }
                self.hold_states.set(&key_var, &true.to_variant());
                self.hold_timers.set(&key_var, &0.0.to_variant());
            } else if all_keys_pressed && was_pressed {
                let hold_state_var = self.hold_states.get_or_nil(&key_var);
                if !hold_state_var.is_nil() && hold_state_var.to::<bool>() {
                    let timer_var = self.hold_timers.get_or_nil(&key_var);
                    let mut timer = if timer_var.is_nil() { 0.0 } else { timer_var.to::<f64>() };
                    timer += _delta;
                    if timer >= self.hold_interval {
                        let fn_var = self.fn_mappings_press.get_or_nil(&key_var);
                        let callable = fn_var.to::<Callable>();
                        if callable.is_valid() {
                            let _ = callable.call(&[]);
                        }
                        timer = 0.0;
                    }
                    self.hold_timers.set(&key_var, &timer.to_variant());
                }
            } else if was_pressed && !all_keys_pressed {
                let fn_var = self.fn_mappings_release.get_or_nil(&key_var);
                let callable = fn_var.to::<Callable>();
                if callable.is_valid() {
                    let _ = callable.call(&[]);
                }
                self.action_states.set(&key_var, &false.to_variant());
                self.hold_states.set(&key_var, &false.to_variant());
            }
        }

        self.process_axis_input();
    }

    fn exit_tree(&mut self) {
        self.clear_virtual_inputs();
    }
}

#[godot_api]
impl GdInput {
    /// 轴值变化信号
    #[signal]
    fn axis_changed(axis_name: GString, value: Vector2);

    /// 鼠标进入信号
    #[signal]
    fn s_mouse_enter();

    /// 鼠标退出信号
    #[signal]
    fn s_mouse_exit();

    /// 鼠标点击信号
    #[signal]
    fn s_mouse_click();

    #[func]
    fn get_enable(&self) -> bool {
        self.enable
    }

    #[func]
    fn set_enable(&mut self, value: bool) {
        self.enable = value;
    }

    #[func]
    fn get_watch_mouse_enter(&self) -> bool {
        self.watch_mouse_enter
    }

    #[func]
    fn set_watch_mouse_enter(&mut self, value: bool) {
        self.watch_mouse_enter = value;
        if !self.base().is_node_ready() {
            return;
        }
        self.update_mouse_enter_watch();
    }

    #[func]
    fn get_watch_mouse_click(&self) -> bool {
        self.watch_mouse_click
    }

    #[func]
    fn set_watch_mouse_click(&mut self, value: bool) {
        self.watch_mouse_click = value;
        if !self.base().is_node_ready() {
            return;
        }
        self.update_mouse_click_watch();
    }

    #[func]
    fn get_watch_mouse_area(&self) -> Option<Gd<Shape2D>> {
        self.watch_mouse_area.clone()
    }

    #[func]
    fn set_watch_mouse_area(&mut self, value: Option<Gd<Shape2D>>) {
        self.watch_mouse_area = value.clone();
        if let Some(ref coll) = self.coll {
            let mut coll = coll.clone();
            coll.call("set_shape", &[value.to_variant()]);
        }
    }

    /// 注册轴映射
    #[func]
    fn register_axis(
        &mut self,
        axis_name: GString,
        positive_x: GString,
        negative_x: GString,
        positive_y: GString,
        negative_y: GString,
    ) {
        let mut mapping = VarDictionary::new();
        mapping.set(&"positive_x".to_variant(), &positive_x.to_variant());
        mapping.set(&"negative_x".to_variant(), &negative_x.to_variant());
        mapping.set(&"positive_y".to_variant(), &positive_y.to_variant());
        mapping.set(&"negative_y".to_variant(), &negative_y.to_variant());
        self.axis_mappings.set(&axis_name.to_variant(), &mapping.to_variant());
    }

    /// 仅注册按键码到 InputMap（不绑定回调）
    #[func]
    fn register_only_code(&mut self, action_name: GString, key_combination: VarArray) {
        let mut input_map = InputMap::singleton();
        if !input_map.has_action(&StringName::from(&action_name)) {
            input_map.add_action(&StringName::from(&action_name));
        }
        for i in 0..key_combination.len() {
            let keycode_var = key_combination.at(i);
            let keycode = keycode_var.to::<i64>();
            let mut event = InputEventKey::new_gd();
            event.set_keycode(Key::from_ord(keycode as i32));
            event.set_physical_keycode(Key::from_ord(keycode as i32));
            input_map.action_add_event(&StringName::from(&action_name), &event);
        }
    }

    /// 移除 InputMap 中的动作
    #[func]
    fn remove_only_code(&mut self, action_name: GString) {
        let mut input_map = InputMap::singleton();
        if input_map.has_action(&StringName::from(&action_name)) {
            input_map.erase_action(&StringName::from(&action_name));
        }
    }

    /// 注册虚拟动作（组合按键 + 回调）
    /// 长按期间会每隔 hold_interval 秒重复调用 on_press 回调
    #[func]
    fn register_virtual_action(
        &mut self,
        action_name: GString,
        key_combination: VarArray,
        joy_combination: VarArray,
        on_press: Callable,
        on_release: Callable,
    ) {
        let mut input_map = InputMap::singleton();

        // 注册键盘动作 key_{action_name}
        let key_action = GString::from(&format!("key_{}", action_name));
        self.register_virtual_action_(&key_action);
        let mut keycodes = VarArray::new();
        for i in 0..key_combination.len() {
            let keycode = key_combination.at(i).to::<i64>();
            let mut event = InputEventKey::new_gd();
            event.set_keycode(Key::from_ord(keycode as i32));
            event.set_physical_keycode(Key::from_ord(keycode as i32));
            input_map.action_add_event(&StringName::from(&key_action), &event);
            keycodes.push(&event.to_variant());
        }
        if keycodes.len() > 0 {
            let key_var = key_action.to_variant();
            self.virtual_actions.set(&key_var, &keycodes.to_variant());
            self.action_states.set(&key_var, &false.to_variant());
            self.fn_mappings_press.set(&key_var, &on_press.to_variant());
            self.fn_mappings_release.set(&key_var, &on_release.to_variant());
            self.hold_states.set(&key_var, &false.to_variant());
            self.hold_timers.set(&key_var, &0.0.to_variant());
        }

        // 注册手柄动作 joy_{action_name}
        let joy_action = GString::from(&format!("joy_{}", action_name));
        self.register_virtual_action_(&joy_action);
        let mut joykeycodes = VarArray::new();
        for i in 0..joy_combination.len() {
            let button = joy_combination.at(i).to::<i64>();
            let mut event = InputEventJoypadButton::new_gd();
            event.set_button_index(JoyButton::from_ord(button as i32));
            event.set_device(0);
            input_map.action_add_event(&StringName::from(&joy_action), &event);
            joykeycodes.push(&event.to_variant());
        }
        if joykeycodes.len() > 0 {
            let joy_var = joy_action.to_variant();
            self.virtual_actions.set(&joy_var, &joykeycodes.to_variant());
            self.action_states.set(&joy_var, &false.to_variant());
            self.fn_mappings_press.set(&joy_var, &on_press.to_variant());
            self.fn_mappings_release.set(&joy_var, &on_release.to_variant());
            self.hold_states.set(&joy_var, &false.to_variant());
            self.hold_timers.set(&joy_var, &0.0.to_variant());
        }
    }

    /// 注册左键点击回调
    #[func]
    fn register_lmouse_click(&mut self, on_press: Callable, on_release: Callable) {
        if on_press.is_valid() {
            self.fn_lmouse_click.push(&on_press.to_variant());
        }
        if on_release.is_valid() {
            self.fn_lmouse_release.push(&on_release.to_variant());
        }
    }

    /// 注册左键滑动回调
    #[func]
    fn register_lmouse_swipe(&mut self, on_swipe: Callable) {
        if on_swipe.is_valid() {
            self.fn_lmouse_swipe.push(&on_swipe.to_variant());
        }
    }

    /// 模拟按键按下
    #[func]
    fn key_press(&mut self, key_code: i64, release_time: f64) {
        if !self.enable || !self.base().is_visible_in_tree() {
            return;
        }
        let mut event = InputEventKey::new_gd();
        let key = Key::from_ord(key_code as i32);
        event.set_keycode(key);
        event.set_physical_keycode(key);
        event.set_echo(false);
        event.set_pressed(true);
        Input::singleton().parse_input_event(&event);

        if release_time > 0.0 {
            if let Some(mut tree) = self.base().get_tree_or_null() {
                let timer = tree.create_timer(release_time);
                let mut timer = timer;
                let callable = Callable::from_object_method(&*self.base_mut(), "key_release")
                    .bind(&[key_code.to_variant()]);
                let _ = timer.connect("timeout", &callable);
            }
        }
    }

    /// 模拟按键释放
    #[func]
    fn key_release(&mut self, key_code: i64) {
        if !self.enable || !self.base().is_visible_in_tree() {
            return;
        }
        let mut event = InputEventKey::new_gd();
        let key = Key::from_ord(key_code as i32);
        event.set_keycode(key);
        event.set_physical_keycode(key);
        event.set_echo(false);
        event.set_pressed(false);
        Input::singleton().parse_input_event(&event);
    }

    /// 触发动作
    #[func]
    fn emit_action(&mut self, action_name: GString, input_id: GString, release_time: f64) {
        if !input_id.is_empty() {
            let id_str = input_id.to_string();
            let target_id = instances().lock().get(&id_str).copied();
            if let Some(tid) = target_id {
                if let Ok(mut target) = Gd::<GdInput>::try_from_instance_id(tid) {
                    if target.is_instance_valid() {
                        target.bind_mut().emit_action(action_name, GString::new(), release_time);
                    }
                }
            }
        } else {
            if !self.enable || !self.base().is_visible_in_tree() {
                return;
            }
            let key_action = GString::from(&format!("key_{}", action_name));
            let key_var = key_action.to_variant();
            let fn_var = self.fn_mappings_press.get_or_nil(&key_var);
            let callable = fn_var.to::<Callable>();
            if callable.is_valid() {
                let events_var = self.virtual_actions.get_or_nil(&key_var);
                if !events_var.is_nil() {
                    let events = events_var.to::<VarArray>();
                    for i in 0..events.len() {
                        let event_var = events.at(i);
                        if let Ok(mut event) = event_var.try_to::<Gd<InputEventKey>>() {
                            let keycode = event.get_keycode();
                            event.set_physical_keycode(keycode);
                            event.set_echo(false);
                            event.set_pressed(true);
                            Input::singleton().parse_input_event(&event);
                        }
                    }
                    if release_time > 0.0 {
                        if let Some(mut tree) = self.base().get_tree_or_null() {
                            let timer = tree.create_timer(0.2);
                            let mut timer = timer;
                            let callable = Callable::from_object_method(&*self.base_mut(), "emit_action_release")
                                .bind(&[action_name.to_variant(), input_id.to_variant()]);
                            let _ = timer.connect("timeout", &callable);
                        }
                    }
                }
            } else {
                self.remove_virtual_input(action_name);
            }
        }
    }

    /// 触发动作释放
    #[func]
    fn emit_action_release(&mut self, action_name: GString, input_id: GString) {
        if !input_id.is_empty() {
            let id_str = input_id.to_string();
            let target_id = instances().lock().get(&id_str).copied();
            if let Some(tid) = target_id {
                if let Ok(mut target) = Gd::<GdInput>::try_from_instance_id(tid) {
                    if target.is_instance_valid() {
                        target.bind_mut().emit_action_release(action_name, GString::new());
                    }
                }
            }
        } else {
            if !self.enable || !self.base().is_visible_in_tree() {
                return;
            }
            let key_action = GString::from(&format!("key_{}", action_name));
            let key_var = key_action.to_variant();
            let fn_var = self.fn_mappings_press.get_or_nil(&key_var);
            let callable = fn_var.to::<Callable>();
            if callable.is_valid() {
                let events_var = self.virtual_actions.get_or_nil(&key_var);
                if !events_var.is_nil() {
                    let events = events_var.to::<VarArray>();
                    for i in 0..events.len() {
                        let event_var = events.at(i);
                        if let Ok(mut event) = event_var.try_to::<Gd<InputEventKey>>() {
                            let keycode = event.get_keycode();
                            event.set_physical_keycode(keycode);
                            event.set_echo(false);
                            event.set_pressed(false);
                            Input::singleton().parse_input_event(&event);
                        }
                    }
                }
            } else {
                self.remove_virtual_input(action_name);
            }
        }
    }

    /// 移除虚拟输入
    #[func]
    fn remove_virtual_input(&mut self, action_name: GString) {
        let mut input_map = InputMap::singleton();
        // 从 action_names 中移除
        for i in 0..self.action_names.len() {
            let name = self.action_names.at(i).to::<GString>();
            if name == action_name {
                self.action_names.remove(i);
                break;
            }
        }
        if input_map.has_action(&StringName::from(&action_name)) {
            input_map.erase_action(&StringName::from(&action_name));
        }
        let key = action_name.to_variant();
        self.virtual_actions.erase(&key);
        self.action_states.erase(&key);
        self.fn_mappings_press.erase(&key);
        self.fn_mappings_release.erase(&key);
        self.hold_states.erase(&key);
        self.hold_timers.erase(&key);
    }

    /// 清除所有虚拟输入
    #[func]
    fn clear_virtual_inputs(&mut self) {
        let mut input_map = InputMap::singleton();
        let keys = self.virtual_actions.keys_array();
        for i in 0..keys.len() {
            let action = keys.at(i).to::<GString>();
            if input_map.has_action(&StringName::from(&action)) {
                input_map.erase_action(&StringName::from(&action));
            }
        }
        self.action_names.clear();
        self.virtual_actions.clear();
        self.axis_mappings.clear();
        self.action_states.clear();
        self.hold_states.clear();
        self.hold_timers.clear();
        self.fn_lmouse_click.clear();
        self.fn_lmouse_release.clear();
    }

    /// 检查动作是否按下
    #[func]
    fn is_action_pressed(&self, action_name: GString) -> bool {
        let state_var = self.action_states.get_or_nil(&action_name.to_variant());
        !state_var.is_nil() && state_var.to::<bool>()
    }

    /// 检查动作是否刚按下
    #[func]
    fn is_action_just_pressed(&self, action_name: GString) -> bool {
        Input::singleton().is_action_just_pressed(&StringName::from(&action_name))
    }

    /// 检查动作是否刚释放
    #[func]
    fn is_action_just_released(&self, action_name: GString) -> bool {
        Input::singleton().is_action_just_released(&StringName::from(&action_name))
    }

    /// 获取轴值
    #[func]
    fn get_axis_value(&self, axis_name: GString) -> Vector2 {
        let mapping_var = self.axis_mappings.get_or_nil(&axis_name.to_variant());
        if mapping_var.is_nil() {
            return Vector2::ZERO;
        }
        let mapping = mapping_var.to::<VarDictionary>();
        let mut result = Vector2::ZERO;
        let input = Input::singleton();

        let px = mapping.get_or_nil(&"positive_x".to_variant());
        if !px.is_nil() && input.is_action_pressed(&StringName::from(&px.to::<GString>())) {
            result.x += 1.0;
        }
        let nx = mapping.get_or_nil(&"negative_x".to_variant());
        if !nx.is_nil() && input.is_action_pressed(&StringName::from(&nx.to::<GString>())) {
            result.x -= 1.0;
        }
        let py = mapping.get_or_nil(&"positive_y".to_variant());
        if !py.is_nil() && input.is_action_pressed(&StringName::from(&py.to::<GString>())) {
            result.y += 1.0;
        }
        let ny = mapping.get_or_nil(&"negative_y".to_variant());
        if !ny.is_nil() && input.is_action_pressed(&StringName::from(&ny.to::<GString>())) {
            result.y -= 1.0;
        }
        result
    }

    /// 鼠标进入回调（Node2D/Control 通用）
    #[func]
    fn on_mouse_entered(&mut self) {
        if !self.enable || !self.base().is_visible_in_tree() {
            return;
        }
        self.base_mut().emit_signal("s_mouse_enter", &[]);
    }

    /// 鼠标退出回调
    #[func]
    fn on_mouse_exited(&mut self) {
        if !self.enable || !self.base().is_visible_in_tree() {
            return;
        }
        self.base_mut().emit_signal("s_mouse_exit", &[]);
    }

    /// 鼠标输入事件回调（Node2D Area2D 模式）
    #[func]
    fn on_mouse_input_event(&mut self, _viewport: Gd<Node>, event: Gd<InputEvent>, _shape_idx: i64) {
        if !self.enable || !self.base().is_visible_in_tree() {
            return;
        }
        if let Ok(mouse_btn) = event.try_cast::<InputEventMouseButton>() {
            if mouse_btn.get_button_index() == MouseButton::LEFT
                && mouse_btn.is_pressed()
            {
                self.base_mut().emit_signal("s_mouse_click", &[]);
            }
        }
    }

    /// 鼠标 GUI 输入回调（Control 模式）
    #[func]
    fn on_mouse_gui_input(&mut self, event: Gd<InputEvent>) {
        if !self.enable || !self.base().is_visible_in_tree() {
            return;
        }
        if let Ok(mouse_btn) = event.try_cast::<InputEventMouseButton>() {
            if mouse_btn.get_button_index() == MouseButton::LEFT
                && mouse_btn.is_pressed()
            {
                self.base_mut().emit_signal("s_mouse_click", &[]);
            }
        }
    }
}

impl GdInput {
    /// 内部注册动作到 InputMap（清除已有事件）
    fn register_virtual_action_(&mut self, action_name: &GString) {
        let mut input_map = InputMap::singleton();
        if !input_map.has_action(&StringName::from(action_name)) {
            input_map.add_action(&StringName::from(action_name));
            self.action_names.push(&action_name.to_variant());
        } else {
            // 清除已有的输入事件
            let existing = input_map.action_get_events(&StringName::from(action_name));
            for i in 0..existing.len() {
                let ev = existing.at(i);
                input_map.action_erase_event(&StringName::from(action_name), &ev);
            }
        }
    }

    /// Node2D 模式初始化鼠标监听
    fn init_mouse_watch_node2d(&mut self) {
        if !self.is_mout_node2d {
            return;
        }

        // 创建 CollisionShape2D 和 Area2D
        let mut coll = CollisionShape2D::new_alloc();
        if let Some(ref shape) = self.watch_mouse_area {
            coll.call("set_shape", &[shape.to_variant()]);
        }

        let mut area = Area2D::new_alloc();
        area.add_child(&coll);
        self.base_mut().add_child(&area);

        self.coll = Some(coll);
        self.area_ = Some(area);

        // 触发信号连接
        self.update_mouse_enter_watch();
        self.update_mouse_click_watch();
    }

    /// Control 模式初始化鼠标监听
    fn init_mouse_watch_control(&mut self) {
        if self.is_mout_node2d {
            return;
        }
        self.update_mouse_enter_watch();
        self.update_mouse_click_watch();
    }

    /// 更新鼠标进入监听
    fn update_mouse_enter_watch(&mut self) {
        let base = self.base().clone();

        if self.is_mout_node2d {
            if let Some(ref area) = self.area_ {
                let mut area = area.clone();
                let enter_cb = Callable::from_object_method(&base, "on_mouse_entered");
                let exit_cb = Callable::from_object_method(&base, "on_mouse_exited");

                if self.watch_mouse_enter {
                    if !area.is_connected("mouse_entered", &enter_cb) {
                        let _ = area.connect("mouse_entered", &enter_cb);
                    }
                    if !area.is_connected("mouse_exited", &exit_cb) {
                        let _ = area.connect("mouse_exited", &exit_cb);
                    }
                } else {
                    if area.is_connected("mouse_entered", &enter_cb) {
                        area.disconnect("mouse_entered", &enter_cb);
                    }
                    if area.is_connected("mouse_exited", &exit_cb) {
                        area.disconnect("mouse_exited", &exit_cb);
                    }
                }
            }
        } else {
            // Control 模式：连接到父节点
            if let Some(parent) = self.base().get_parent() {
                if parent.get_class() != GString::from("Control") {
                    return;
                }
                let mut target = parent;
                let enter_cb = Callable::from_object_method(&base, "on_mouse_entered");
                let exit_cb = Callable::from_object_method(&base, "on_mouse_exited");

                if self.watch_mouse_enter {
                    if !target.is_connected("mouse_entered", &enter_cb) {
                        let _ = target.connect("mouse_entered", &enter_cb);
                    }
                    if !target.is_connected("mouse_exited", &exit_cb) {
                        let _ = target.connect("mouse_exited", &exit_cb);
                    }
                } else {
                    if target.is_connected("mouse_entered", &enter_cb) {
                        target.disconnect("mouse_entered", &enter_cb);
                    }
                    if target.is_connected("mouse_exited", &exit_cb) {
                        target.disconnect("mouse_exited", &exit_cb);
                    }
                }
            }
        }
    }

    /// 更新鼠标点击监听
    fn update_mouse_click_watch(&mut self) {
        let base = self.base().clone();

        if self.is_mout_node2d {
            if let Some(ref area) = self.area_ {
                let mut area = area.clone();
                let cb = Callable::from_object_method(&base, "on_mouse_input_event");

                if self.watch_mouse_click {
                    if !area.is_connected("input_event", &cb) {
                        let _ = area.connect("input_event", &cb);
                    }
                } else {
                    if area.is_connected("input_event", &cb) {
                        area.disconnect("input_event", &cb);
                    }
                }
            }
        } else {
            if let Some(parent) = self.base().get_parent() {
                if parent.get_class() != GString::from("Control") {
                    return;
                }
                let mut target = parent;
                let cb = Callable::from_object_method(&base, "on_mouse_gui_input");

                if self.watch_mouse_click {
                    if !target.is_connected("gui_input", &cb) {
                        let _ = target.connect("gui_input", &cb);
                    }
                } else {
                    if target.is_connected("gui_input", &cb) {
                        target.disconnect("gui_input", &cb);
                    }
                }
            }
        }
    }

    /// 处理轴输入
    fn process_axis_input(&mut self) {
        let axis_names = self.axis_mappings.keys_array();
        for i in 0..axis_names.len() {
            let axis_name = axis_names.at(i).to::<GString>();
            let mapping_var = self.axis_mappings.get_or_nil(&axis_name.to_variant());
            if mapping_var.is_nil() {
                continue;
            }
            let mapping = mapping_var.to::<VarDictionary>();
            let mut axis_value = Vector2::ZERO;
            let input = Input::singleton();

            let px = mapping.get_or_nil(&"positive_x".to_variant());
            if !px.is_nil() && input.is_action_pressed(&StringName::from(&px.to::<GString>())) {
                axis_value.x += 1.0;
            }
            let nx = mapping.get_or_nil(&"negative_x".to_variant());
            if !nx.is_nil() && input.is_action_pressed(&StringName::from(&nx.to::<GString>())) {
                axis_value.x -= 1.0;
            }
            let py = mapping.get_or_nil(&"positive_y".to_variant());
            if !py.is_nil() && input.is_action_pressed(&StringName::from(&py.to::<GString>())) {
                axis_value.y += 1.0;
            }
            let ny = mapping.get_or_nil(&"negative_y".to_variant());
            if !ny.is_nil() && input.is_action_pressed(&StringName::from(&ny.to::<GString>())) {
                axis_value.y -= 1.0;
            }

            self.base_mut().emit_signal(
                "axis_changed",
                &[axis_name.to_variant(), axis_value.to_variant()],
            );
        }
    }

    /// 检测滑动方向
    fn detect_swipe_direction(&mut self, swipe_vector: Vector2) {
        let abs_x = swipe_vector.x.abs();
        let abs_y = swipe_vector.y.abs();

        let direction = if abs_x > abs_y {
            if swipe_vector.x > 0.0 {
                GString::from("right")
            } else {
                GString::from("left")
            }
        } else {
            if swipe_vector.y > 0.0 {
                GString::from("down")
            } else {
                GString::from("up")
            }
        };

        for i in 0..self.fn_lmouse_swipe.len() {
            let fn_var = self.fn_lmouse_swipe.at(i);
            let callable = fn_var.to::<Callable>();
            if callable.is_valid() {
                let _ = callable.call(&[direction.to_variant()]);
            }
        }
    }
}
