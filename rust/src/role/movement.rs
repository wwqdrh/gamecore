// GdRoleMover - 游戏角色移动控制器
// 继承 CharacterBody2D，统一管理角色移动：
//   - 控制方式可配置：键盘（InputMap 动作 / WASD 方向键兜底）、鼠标（点击/按住移动）、AI（由 GdNpcBrain 驱动）
//   - 移动方向可配置：四向（俯视）/ 横向（平台）
//   - 重力可配置：gravity_enabled / gravity / max_fall_speed，横向模式下输入只控制水平轴
//   - 移动速度可配置：speed + acceleration / friction 平滑加减速（0 = 瞬时响应）
// 配套：GdRoleAnimator 负责动画驱动，GdNpcBrain 负责 NPC 行为
//
// 信号：
//   s_moving_changed(moving)  开始/停止移动
//   s_facing_changed(facing)  朝向变化 left/right/up/down
//   s_landed()                落地
//   s_jumped()                起跳

use godot::prelude::*;
use godot::builtin::{GString, StringName, Vector2};
use godot::classes::{
    CharacterBody2D, ICharacterBody2D, Input, InputEvent, InputEventMouseButton, InputMap,
};
use godot::global::{Key, MouseButton};

/// 控制方式：无控制（外部/脚本驱动）
pub const CONTROL_NONE: i64 = 0;
/// 控制方式：键盘（InputMap 动作优先，缺失时回退 WASD/方向键）
pub const CONTROL_KEYBOARD: i64 = 1;
/// 控制方式：鼠标（左键点击移动，mouse_follow 开启时按住持续跟随）
pub const CONTROL_MOUSE: i64 = 2;
/// 控制方式：AI（由 GdNpcBrain 通过 set_ai_direction / set_ai_target_position 驱动）
pub const CONTROL_AI: i64 = 3;

/// 移动模式：四向（俯视，上下左右）
pub const MODE_FOUR_WAY: i64 = 0;
/// 移动模式：横向（平台，仅左右，配合重力做平台跳跃）
pub const MODE_HORIZONTAL: i64 = 1;

#[derive(GodotClass)]
#[class(base = CharacterBody2D)]
pub struct GdRoleMover {
    /// 是否启用移动控制
    #[export]
    enable: bool,

    /// 剧情锁：对话/演出期间暂停一切移动（不影响 enable 的配置语义）
    paused: bool,

    /// 控制方式 CONTROL_NONE / CONTROL_KEYBOARD / CONTROL_MOUSE / CONTROL_AI
    #[export]
    control_mode: i64,

    /// 移动模式 MODE_FOUR_WAY / MODE_HORIZONTAL
    #[export]
    move_mode: i64,

    /// 移动速度（像素/秒）
    #[export]
    speed: f64,

    /// 加速度（0 = 瞬时达到目标速度）
    #[export]
    acceleration: f64,

    /// 摩擦/减速度（0 = 瞬时停止）
    #[export]
    friction: f64,

    /// 是否启用重力（启用后输入只控制水平轴，垂直交给重力/跳跃）
    #[export]
    gravity_enabled: bool,

    /// 重力加速度（像素/秒^2）
    #[export]
    gravity: f64,

    /// 最大下落速度
    #[export]
    max_fall_speed: f64,

    /// 起跳速度（负值向上；0 = 禁用跳跃；仅 gravity_enabled 时生效）
    #[export]
    jump_velocity: f64,

    /// 跳跃动作名（InputMap 缺失时回退空格键）
    #[export]
    jump_action: GString,

    /// 左移动作名（InputMap 缺失时回退 A/←）
    #[export]
    action_left: GString,

    /// 右移动作名（InputMap 缺失时回退 D/→）
    #[export]
    action_right: GString,

    /// 上移动作名（四向模式，InputMap 缺失时回退 W/↑）
    #[export]
    action_up: GString,

    /// 下移动作名（四向模式，InputMap 缺失时回退 S/↓）
    #[export]
    action_down: GString,

    /// 鼠标按住期间持续跟随鼠标位置
    #[export]
    mouse_follow: bool,

    /// 鼠标/AI 到达判定距离（像素）
    #[export]
    arrival_distance: f64,

    // ---- 运行时状态 ----
    /// 当前朝向 left/right/up/down
    facing: GString,
    /// 是否正在移动
    moving: bool,
    /// 上一帧是否在地面（用于落地信号）
    was_on_floor: bool,
    /// 鼠标左键是否按住
    mouse_held: bool,
    /// 上一帧跳跃键是否按住（边沿检测）
    jump_held_prev: bool,
    /// 鼠标移动目标点
    move_target: Vector2,
    /// 是否存在鼠标移动目标
    has_move_target: bool,
    /// AI 直接方向驱动
    ai_dir: Vector2,
    /// AI 目标点驱动
    ai_target: Vector2,
    /// AI 是否使用目标点驱动
    ai_use_target: bool,
    /// AI 到达停止距离
    ai_stop_distance: f64,

    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for GdRoleMover {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            enable: true,
            paused: false,
            control_mode: CONTROL_KEYBOARD,
            move_mode: MODE_FOUR_WAY,
            speed: 150.0,
            acceleration: 0.0,
            friction: 0.0,
            gravity_enabled: false,
            gravity: 980.0,
            max_fall_speed: 2000.0,
            jump_velocity: -350.0,
            jump_action: GString::from("jump"),
            action_left: GString::from("move_left"),
            action_right: GString::from("move_right"),
            action_up: GString::from("move_up"),
            action_down: GString::from("move_down"),
            mouse_follow: false,
            arrival_distance: 6.0,
            facing: GString::from("right"),
            moving: false,
            was_on_floor: false,
            mouse_held: false,
            jump_held_prev: false,
            move_target: Vector2::ZERO,
            has_move_target: false,
            ai_dir: Vector2::ZERO,
            ai_target: Vector2::ZERO,
            ai_use_target: false,
            ai_stop_distance: 6.0,
            base,
        }
    }

    /// 鼠标控制：左键按下设定移动目标点
    fn input(&mut self, event: Gd<InputEvent>) {
        if !self.enable || self.control_mode != CONTROL_MOUSE {
            return;
        }
        if let Ok(btn) = event.try_cast::<InputEventMouseButton>() {
            if btn.get_button_index() == MouseButton::LEFT {
                if btn.is_pressed() {
                    self.mouse_held = true;
                    self.move_target = self.base().get_global_mouse_position();
                    self.has_move_target = true;
                } else {
                    self.mouse_held = false;
                }
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if !self.enable || self.paused {
            return;
        }
        self.tick(delta);
    }
}

#[godot_api]
impl GdRoleMover {
    /// 开始/停止移动信号
    #[signal]
    fn s_moving_changed(moving: bool);

    /// 朝向变化信号（left/right/up/down）
    #[signal]
    fn s_facing_changed(facing: GString);

    /// 落地信号
    #[signal]
    fn s_landed();

    /// 起跳信号
    #[signal]
    fn s_jumped();

    /// 当前朝向
    #[func]
    pub fn get_facing(&self) -> GString {
        self.facing.clone()
    }

    /// 强制设置朝向（不影响移动）
    #[func]
    pub fn set_facing(&mut self, facing: GString) {
        if self.facing != facing {
            self.facing = facing;
            let var = self.facing.to_variant();
            self.base_mut().emit_signal("s_facing_changed", &[var]);
        }
    }

    /// 是否正在移动
    #[func]
    pub fn is_moving(&self) -> bool {
        self.moving
    }

    /// 停止一切移动指令（清除鼠标目标与 AI 驱动）
    #[func]
    pub fn stop(&mut self) {
        self.has_move_target = false;
        self.mouse_held = false;
        self.ai_dir = Vector2::ZERO;
        self.ai_use_target = false;
    }

    /// 脚本便捷接口：移动到世界坐标点（切换为鼠标控制方式）
    #[func]
    pub fn move_to_point(&mut self, target: Vector2) {
        self.control_mode = CONTROL_MOUSE;
        self.move_target = target;
        self.has_move_target = true;
    }

    /// AI 接口：直接方向驱动（归一化前向量即可，内部按速度应用）
    #[func]
    pub fn set_ai_direction(&mut self, dir: Vector2) {
        self.control_mode = CONTROL_AI;
        self.ai_use_target = false;
        self.ai_dir = dir;
    }

    /// AI 接口：目标点驱动，距离小于 stop_distance 时停下
    #[func]
    pub fn set_ai_target_position(&mut self, target: Vector2, stop_distance: f64) {
        self.control_mode = CONTROL_AI;
        self.ai_use_target = true;
        self.ai_target = target;
        self.ai_stop_distance = stop_distance;
    }

    /// 当前速度向量
    #[func]
    pub fn get_velocity(&self) -> Vector2 {
        self.base().get_velocity()
    }

    /// 暂停/恢复移动（对话等剧情演出期间锁定，停止并清零速度）
    #[func]
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if paused {
            self.stop();
            self.base_mut().set_velocity(Vector2::ZERO);
        }
    }

    /// 是否被剧情锁暂停
    #[func]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    // ---- 内部实现 ----

    /// 物理帧驱动
    fn tick(&mut self, delta: f64) {
        let mut input_vec = self.read_input();

        // 横向模式：输入只保留水平分量
        if self.move_mode == MODE_HORIZONTAL {
            input_vec.y = 0.0;
        }

        let mut velocity = self.base().get_velocity();

        if self.gravity_enabled {
            // 平台模式：水平轴由输入平滑驱动，垂直轴交给重力/跳跃
            let target_x = input_vec.x * self.speed as f32;
            velocity.x = self.approach_axis(
                velocity.x,
                target_x,
                input_vec.x != 0.0,
                delta,
            );

            // 跳跃（边沿检测：仅按下瞬间触发）
            if self.jump_velocity != 0.0 && self.base().is_on_floor() {
                let held = self.jump_held();
                if held && !self.jump_held_prev {
                    velocity.y = self.jump_velocity as f32;
                    self.base_mut().emit_signal("s_jumped", &[]);
                }
                self.jump_held_prev = held;
            }

            // 重力
            if !self.base().is_on_floor() {
                velocity.y = (velocity.y + (self.gravity * delta) as f32)
                    .min(self.max_fall_speed as f32);
            }
        } else {
            // 俯视模式：输入直接驱动双轴（斜向归一化）
            let mut dir = input_vec;
            if dir.length() > 1.0 {
                dir = dir.normalized();
            }
            let target_vel = dir * self.speed as f32;
            let accelerating = dir.length_squared() > 0.0001;
            let rate = if accelerating {
                self.acceleration
            } else {
                self.friction
            };
            if rate <= 0.0 {
                velocity = target_vel;
            } else {
                velocity = velocity.move_toward(target_vel, (rate * delta) as f32);
            }
        }

        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        // 落地检测
        let on_floor = self.base().is_on_floor();
        if on_floor && !self.was_on_floor {
            self.base_mut().emit_signal("s_landed", &[]);
        }
        self.was_on_floor = on_floor;

        // 朝向与移动状态更新
        self.update_state(input_vec);
    }

    /// 按控制方式读取输入向量
    fn read_input(&mut self) -> Vector2 {
        match self.control_mode {
            CONTROL_KEYBOARD => self.read_keyboard(),
            CONTROL_MOUSE => {
                // 按住持续跟随
                if self.mouse_follow && self.mouse_held {
                    self.move_target = self.base().get_global_mouse_position();
                    self.has_move_target = true;
                }
                if !self.has_move_target {
                    return Vector2::ZERO;
                }
                let mut target = self.move_target;
                if self.move_mode == MODE_HORIZONTAL {
                    // 横向模式锁定垂直坐标
                    target.y = self.base().get_global_position().y;
                }
                let diff = target - self.base().get_global_position();
                if diff.length() <= self.arrival_distance as f32 {
                    return Vector2::ZERO;
                }
                diff.normalized()
            }
            CONTROL_AI => self.read_ai(),
            _ => Vector2::ZERO,
        }
    }

    /// 键盘输入：InputMap 动作优先，缺失时回退物理按键
    fn read_keyboard(&self) -> Vector2 {
        let mut v = Vector2::ZERO;
        if self.pressed(&self.action_left, &[Key::A, Key::LEFT]) {
            v.x -= 1.0;
        }
        if self.pressed(&self.action_right, &[Key::D, Key::RIGHT]) {
            v.x += 1.0;
        }
        if self.move_mode == MODE_FOUR_WAY {
            if self.pressed(&self.action_up, &[Key::W, Key::UP]) {
                v.y -= 1.0;
            }
            if self.pressed(&self.action_down, &[Key::S, Key::DOWN]) {
                v.y += 1.0;
            }
        }
        v
    }

    /// AI 输入：目标点驱动或方向驱动
    fn read_ai(&mut self) -> Vector2 {
        if self.ai_use_target {
            let diff = self.ai_target - self.base().get_global_position();
            let stop = self.ai_stop_distance.max(self.arrival_distance);
            if diff.length() <= stop as f32 {
                return Vector2::ZERO;
            }
            return diff.normalized();
        }
        self.ai_dir
    }

    /// 检测动作是否按下，动作未注册时回退到物理按键组
    fn pressed(&self, action: &GString, fallback_keys: &[Key]) -> bool {
        let input = Input::singleton();
        let sn = StringName::from(action);
        if InputMap::singleton().has_action(&sn) {
            input.is_action_pressed(&sn)
        } else {
            fallback_keys
                .iter()
                .any(|k| input.is_physical_key_pressed(*k))
        }
    }

    /// 跳跃键检测：动作优先，回退空格键
    fn jump_held(&self) -> bool {
        let input = Input::singleton();
        let sn = StringName::from(&self.jump_action);
        if InputMap::singleton().has_action(&sn) {
            input.is_action_pressed(&sn)
        } else {
            input.is_physical_key_pressed(Key::SPACE)
        }
    }

    /// 单轴平滑逼近（加减速）
    fn approach_axis(&self, current: f32, target: f32, moving: bool, delta: f64) -> f32 {
        let rate = if moving {
            self.acceleration
        } else {
            self.friction
        };
        if rate <= 0.0 {
            return target;
        }
        let step = (rate * delta) as f32;
        let diff = target - current;
        if diff.abs() <= step {
            target
        } else {
            current + diff.signum() * step
        }
    }

    /// 更新移动状态与朝向，变化时发信号
    fn update_state(&mut self, input_vec: Vector2) {
        let is_moving = input_vec.length_squared() > 0.0001
            || self.base().get_velocity().length_squared() > 4.0;
        if is_moving != self.moving {
            self.moving = is_moving;
            self.base_mut()
                .emit_signal("s_moving_changed", &[is_moving.to_variant()]);
        }

        if input_vec.length_squared() > 0.0001 {
            let new_facing = if self.move_mode == MODE_HORIZONTAL {
                if input_vec.x > 0.0 {
                    "right"
                } else {
                    "left"
                }
            } else if input_vec.x.abs() >= input_vec.y.abs() {
                if input_vec.x > 0.0 {
                    "right"
                } else {
                    "left"
                }
            } else if input_vec.y > 0.0 {
                "down"
            } else {
                "up"
            };
            let gs = GString::from(new_facing);
            if self.facing != gs {
                self.facing = gs;
                let var = self.facing.to_variant();
                self.base_mut().emit_signal("s_facing_changed", &[var]);
            }
        }
    }
}
