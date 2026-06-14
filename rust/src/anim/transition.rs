// 过渡动画
// 移植自 C++ juice/transition.h
// 提供淡入淡出过渡效果

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, NodePath};
use godot::classes::{Node, Tween};

use super::easing::Easing;
use super::juice::easing_to_trans;
use super::juice::easing_to_ease_type;

/// 淡入淡出过渡效果
pub struct TransitionFade {
    pub duration: f64,
    pub fade_type: FadeType,
    pub easing: Easing,
}

/// 淡入淡出类型
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FadeType {
    FadeIn,
    FadeOut,
}

impl TransitionFade {
    pub fn new(duration: f64, fade_type: FadeType, easing: Easing) -> Self {
        Self {
            duration,
            fade_type,
            easing,
        }
    }

    /// 在目标节点上启动淡入淡出动画
    pub fn start(&self, target: &Gd<Node>) -> Option<Gd<Tween>> {
        let mut target = target.clone();

        let (value_start, value_end) = match self.fade_type {
            FadeType::FadeIn => (0.0_f32, 1.0_f32),
            FadeType::FadeOut => (1.0_f32, 0.0_f32),
        };

        // 显示目标节点
        if target.has_method(&StringName::from("show")) {
            target.call(&StringName::from("show"), &[]);
        }

        let mut tween = target.create_tween();
        tween.set_trans(easing_to_trans(self.easing));
        tween.set_ease(easing_to_ease_type(self.easing));

        // 设置起始透明度
        let start_color = Color::from_rgba(1.0, 1.0, 1.0, value_start);
        let end_color = Color::from_rgba(1.0, 1.0, 1.0, value_end);

        // 使用 tween_property 动画 modulate 属性
        tween.tween_property(
            &target,
            &NodePath::from("modulate"),
            &end_color.to_variant(),
            self.duration,
        ).from(&start_color.to_variant());

        Some(tween)
    }
}
