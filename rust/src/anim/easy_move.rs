// 平滑移动工具
// 移植自 C++ juice/easy_move.h/cpp
// 基于时钟的平滑插值移动，支持前进/后退/平滑步进

use godot::prelude::*;
use godot::builtin::{Vector2, StringName};
use godot::classes::Control;
use godot::obj::NewGd;

/// 平滑移动器：基于时钟的 Vector2 插值移动
/// 通过 count/move 方法驱动，自动应用位置到目标 Control 节点
pub struct EaseMover {
    pub show: bool,
    pub clock: f64,
    pub time: f64,
    pub last: f64,
    pub from: Vector2,
    pub to: Vector2,
    pub current: Vector2,
    pub node: Option<Gd<Control>>,
}

impl EaseMover {
    pub fn new(
        time: f64,
        clock: f64,
        from: Vector2,
        to: Vector2,
        node: Option<Gd<Control>>,
    ) -> Self {
        let clock = if clock < 0.0 { time } else { clock };
        Self {
            show: true,
            clock,
            time,
            last: -1.0,
            from,
            to,
            current: from,
            node,
        }
    }

    /// 推进时钟并返回缓动进度
    /// delta: 帧间隔时间
    /// forward: true=前进，false=后退
    /// is_smooth: true=使用 smoothstep，false=线性
    pub fn count(&mut self, delta: f64, forward: bool, is_smooth: bool) -> f64 {
        self.last = self.clock;
        if forward {
            self.clock = (self.clock + delta).min(self.time);
        } else {
            self.clock = (self.clock - delta).max(0.0);
        }
        if is_smooth {
            self.smooth()
        } else {
            self.frac()
        }
    }

    /// 推进时钟并移动节点位置
    pub fn move_node(&mut self, delta: f64, forward: bool, is_smooth: bool) -> Vector2 {
        let progress = self.count(delta, forward, is_smooth);
        self.current = self.from.lerp(self.to, progress as f32);

        if let Some(ref node) = self.node {
            if node.is_instance_valid() {
                let mut n = node.clone();
                n.set_position(self.current);
            }
        }

        self.current
    }

    /// 从 from 到 to 的插值，arg < 0 时使用当前进度
    pub fn from_lerp_to(&self, arg: f64) -> Vector2 {
        let progress = if arg < 0.0 {
            self.frac() as f32
        } else {
            arg as f32
        };
        self.from.lerp(self.to, progress)
    }

    /// 线性进度 clock / time
    pub fn frac(&self) -> f64 {
        if self.time == 0.0 {
            1.0
        } else {
            self.clock / self.time
        }
    }

    /// smoothstep 缓动进度
    pub fn smooth(&self) -> f64 {
        let t = self.frac();
        // smoothstep(0, 1, t) = 3t^2 - 2t^3
        t * t * (3.0 - 2.0 * t)
    }

    /// 动画是否完成
    pub fn is_complete(&self) -> bool {
        (self.clock - self.time).abs() < f64::EPSILON
    }

    /// 动画是否还在进行中
    pub fn is_less(&self) -> bool {
        self.clock < self.time
    }

    /// 时钟是否没有变化（动画卡住）
    pub fn is_last(&self) -> bool {
        (self.clock - self.last).abs() < f64::EPSILON
    }
}
