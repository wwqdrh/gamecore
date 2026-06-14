// 核心动画系统
// 移植自 C++ juice/juice.h/cpp
// 提供丰富的 UI 动画效果：波浪、旋转、抖动、弹跳、爆炸、收集、入场、呼吸、走路等
// 所有动画基于 Godot Tween 系统，使用 tween_property + 内置缓动类型
// 由于 gdext 0.5 的 tween_method 不支持 Rust 闭包，所有动画均使用 tween_property 实现

use godot::prelude::*;
use godot::builtin::{GString, StringName, NodePath, Color, Vector2, Array};
use godot::classes::{
    Node, Node2D, Control, Tween,
    DisplayServer,
};

use super::easing::Easing;

// ===== 动画辅助函数 =====

/// 检查动画是否正在处理（防重入）
fn check_is_handle(target: &Gd<Node>, name: &str) -> bool {
    let name_sn = StringName::from(name);
    if target.has_meta(&name_sn) {
        let meta = target.get_meta(&name_sn);
        if meta.booleanize() {
            return true;
        }
    }
    false
}

/// 标记动画开始处理
fn mark_handle(target: &mut Gd<Node>, name: &str) {
    target.set_meta(&StringName::from(name), &true.to_variant());
}

/// 动画处理完成，清除标记并调用回调
fn anim_handle_finished(target: &mut Gd<Node>, name: &str, cb: &Callable) {
    target.remove_meta(&StringName::from(name));
    if cb.is_valid() {
        let _ = cb.callv(&Array::<Variant>::new());
    }
}

/// 创建清理 meta 的回调 Callable
fn make_cleanup_callable(target: &Gd<Node>, key: &str) -> Callable {
    let mut t = target.clone();
    let k = key.to_string();
    Callable::from_fn("juice_cleanup", move |_args| {
        if t.is_instance_valid() {
            t.remove_meta(&StringName::from(&k));
        }
    })
}

/// 将 Easing 枚举映射为 Godot 内置的 TransitionType
pub fn easing_to_trans(ease: Easing) -> godot::classes::tween::TransitionType {
    use godot::classes::tween::TransitionType;
    match ease {
        Easing::Linear => TransitionType::LINEAR,
        Easing::SineIn | Easing::SineOut | Easing::SineInOut => TransitionType::SINE,
        Easing::QuadIn | Easing::QuadOut | Easing::QuadInOut => TransitionType::QUAD,
        Easing::CubicIn | Easing::CubicOut | Easing::CubicInOut => TransitionType::CUBIC,
        Easing::QuartIn | Easing::QuartOut | Easing::QuartInOut => TransitionType::QUART,
        Easing::QuintIn | Easing::QuintOut | Easing::QuintInOut => TransitionType::QUINT,
        Easing::ExpoIn | Easing::ExpoOut | Easing::ExpoInOut => TransitionType::EXPO,
        Easing::CircIn | Easing::CircOut | Easing::CircInOut => TransitionType::CIRC,
        Easing::BackIn | Easing::BackOut | Easing::BackInOut => TransitionType::BACK,
        Easing::ElasticIn | Easing::ElasticOut | Easing::ElasticInOut => TransitionType::ELASTIC,
        Easing::BounceIn | Easing::BounceOut | Easing::BounceInOut => TransitionType::BOUNCE,
    }
}

/// 将 Easing 枚举映射为 Godot 内置的 EaseType
pub fn easing_to_ease_type(ease: Easing) -> godot::classes::tween::EaseType {
    use godot::classes::tween::EaseType;
    match ease {
        Easing::SineIn | Easing::QuadIn | Easing::CubicIn | Easing::QuartIn
        | Easing::QuintIn | Easing::ExpoIn | Easing::CircIn | Easing::BackIn
        | Easing::ElasticIn | Easing::BounceIn | Easing::Linear => EaseType::IN,
        Easing::SineOut | Easing::QuadOut | Easing::CubicOut | Easing::QuartOut
        | Easing::QuintOut | Easing::ExpoOut | Easing::CircOut | Easing::BackOut
        | Easing::ElasticOut | Easing::BounceOut => EaseType::OUT,
        Easing::SineInOut | Easing::QuadInOut | Easing::CubicInOut | Easing::QuartInOut
        | Easing::QuintInOut | Easing::ExpoInOut | Easing::CircInOut | Easing::BackInOut
        | Easing::ElasticInOut | Easing::BounceInOut => EaseType::IN_OUT,
    }
}

// ===== 公开动画 API =====

/// 波浪摆动动画（循环，使用 Godot 内置缓动）
/// target: 目标节点
/// wave_offset: 波浪偏移量（默认 Vector2(0, 5)）
/// duration: 单次摆动时长（默认 0.5）
pub fn anim_wave_simple(
    target: &Gd<Node>,
    wave_offset: Vector2,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_wave_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }
    if !target.has_method(&StringName::from("get_position")) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let original_pos: Vector2 = target.call(&StringName::from("get_position"), &[]).to();

    let mut tween = target.create_tween();
    tween.set_loops();
    tween.set_trans(godot::classes::tween::TransitionType::SINE);
    tween.set_ease(godot::classes::tween::EaseType::IN_OUT);

    // 向上摆动
    tween.tween_property(
        &target,
        &NodePath::from("position"),
        &(original_pos + wave_offset).to_variant(),
        duration,
    );
    // 向下摆动
    tween.tween_property(
        &target,
        &NodePath::from("position"),
        &(original_pos - wave_offset).to_variant(),
        duration,
    );

    Some(tween)
}

/// 旋转动画（循环360度，使用 Godot 内置缓动）
pub fn anim_rotate_circle_simple(
    target: &Gd<Node>,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_rotate_circle_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }
    if !target.has_method(&StringName::from("get_rotation_degrees")) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let current_degrees: f32 = target.call(&StringName::from("get_rotation_degrees"), &[]).to();

    let mut tween = target.create_tween();
    tween.set_loops();
    tween.set_trans(godot::classes::tween::TransitionType::LINEAR);

    tween.tween_property(
        &target,
        &NodePath::from("rotation_degrees"),
        &(current_degrees + 360.0).to_variant(),
        duration,
    );

    Some(tween)
}

/// 抖动动画（循环左右旋转，使用 Godot 内置缓动）
pub fn anim_shake_simple(
    target: &Gd<Node>,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_shake_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }
    if !target.has_method(&StringName::from("set_rotation_degrees")) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let mut tween = target.create_tween();
    tween.set_loops();
    tween.set_trans(godot::classes::tween::TransitionType::SINE);
    tween.set_ease(godot::classes::tween::EaseType::IN_OUT);

    // 向左抖
    tween.tween_property(
        &target,
        &NodePath::from("rotation_degrees"),
        &5.0_f32.to_variant(),
        duration,
    );
    // 向右抖
    tween.tween_property(
        &target,
        &NodePath::from("rotation_degrees"),
        &(-5.0_f32).to_variant(),
        duration,
    );

    Some(tween)
}

/// 直线移动动画（使用 Godot 内置缓动）
pub fn anim_move_straight_simple(
    target: &Gd<Node>,
    to_pos: Vector2,
    duration: f64,
    ease_type: godot::classes::tween::TransitionType,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_move_straight_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }
    if !target.has_method(&StringName::from("get_position")) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let mut tween = target.create_tween();
    tween.set_trans(ease_type);
    tween.set_ease(godot::classes::tween::EaseType::IN_OUT);

    tween.tween_property(
        &target,
        &NodePath::from("position"),
        &to_pos.to_variant(),
        duration,
    );

    let cb = make_cleanup_callable(&target, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// 弹跳动画（向目标方向弹出后返回，使用 Godot 内置缓动）
pub fn anim_bounce_simple(
    target: &Gd<Node>,
    dire_pos: Vector2,
    distance: f32,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_bounce_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let original_position: Vector2 = target.call(&StringName::from("get_position"), &[]).to();
    let diff = dire_pos - original_position;
    let direction = if diff.length() > 0.0 { diff.normalized() } else { Vector2::ZERO };
    let target_position = original_position + direction * distance;

    let mut tween = target.create_tween();
    tween.set_trans(godot::classes::tween::TransitionType::BACK);
    tween.set_ease(godot::classes::tween::EaseType::OUT);

    // 向外弹
    tween.tween_property(
        &target,
        &NodePath::from("position"),
        &target_position.to_variant(),
        duration,
    );

    // 返回原位
    tween.tween_property(
        &target,
        &NodePath::from("position"),
        &original_position.to_variant(),
        duration,
    );

    let cb = make_cleanup_callable(&target, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// 缩放动画（使用 Godot 内置缓动）
pub fn do_scale_simple(
    target: &Gd<Node>,
    target_scale: Vector2,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_scale_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }
    if !target.has_method(&StringName::from("get_scale")) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let mut tween = target.create_tween();
    tween.set_trans(godot::classes::tween::TransitionType::BACK);
    tween.set_ease(godot::classes::tween::EaseType::OUT);

    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &target_scale.to_variant(),
        duration,
    );

    let cb = make_cleanup_callable(&target, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// 呼吸缩放动画（循环，使用 Godot 内置缓动）
/// target: 目标节点
/// breath_factor: 呼吸幅度因子（默认 0.05）
/// anim_span: 一次完整呼吸周期时长（默认 1.0）
/// duration: 总时长（<= 0 表示无限循环）
pub fn anim_breath_simple(
    target: &Gd<Node>,
    breath_factor: f32,
    anim_span: f64,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_breath_simple_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let original_scale: Vector2 = target.call(&StringName::from("get_scale"), &[]).to();

    let tall_scale = Vector2::new(
        original_scale.x * (1.0 - breath_factor),
        original_scale.y * (1.0 + breath_factor),
    );
    let wide_scale = Vector2::new(
        original_scale.x * (1.0 + breath_factor),
        original_scale.y * (1.0 - breath_factor),
    );

    let mut tween = target.create_tween();
    tween.set_loops();
    tween.set_trans(godot::classes::tween::TransitionType::SINE);
    tween.set_ease(godot::classes::tween::EaseType::IN_OUT);

    let span = anim_span / 4.0;

    // 原始 -> 细高
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &tall_scale.to_variant(),
        span,
    );
    // 细高 -> 原始
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &original_scale.to_variant(),
        span,
    );
    // 原始 -> 矮胖
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &wide_scale.to_variant(),
        span,
    );
    // 矮胖 -> 原始
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &original_scale.to_variant(),
        span,
    );

    // 如果有固定时长，设置定时器结束
    if duration > 0.0 {
        let mut t2 = target.clone();
        let mut tween_clone = tween.clone();
        let key = anim_key.to_string();
        let stop_cb = Callable::from_fn("juice_breath_stop", move |_args| {
            if t2.is_instance_valid() {
                if tween_clone.is_instance_valid() {
                    tween_clone.kill();
                }
                t2.remove_meta(&StringName::from(&key));
            }
        });
        let mut timer_tween = target.create_tween();
        timer_tween.tween_interval(duration);
        timer_tween.tween_callback(&stop_cb);
    }

    Some(tween)
}

/// 走路摆动动画（循环，使用 Godot 内置缓动）
/// 通过交替旋转和缩放来模拟走路效果
pub fn anim_walk_simple(
    target: &Gd<Node>,
    walk_span: f64,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_walk_simple_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let original_scale: Vector2 = target.call(&StringName::from("get_scale"), &[]).to();
    let max_rotation = 5.0_f32;
    let scale_factor = 0.05_f32;

    let lean_left_scale = Vector2::new(
        original_scale.x * (1.0 + scale_factor),
        original_scale.y * (1.0 - scale_factor),
    );
    let lean_right_scale = Vector2::new(
        original_scale.x * (1.0 - scale_factor),
        original_scale.y * (1.0 + scale_factor),
    );

    let mut tween = target.create_tween();
    tween.set_loops();
    tween.set_trans(godot::classes::tween::TransitionType::SINE);
    tween.set_ease(godot::classes::tween::EaseType::IN_OUT);

    let half_span = walk_span / 2.0;

    // 向左倾斜：旋转 + 缩放
    tween.tween_property(
        &target,
        &NodePath::from("rotation_degrees"),
        &(-max_rotation).to_variant(),
        half_span,
    );
    tween.parallel().tween_property(
        &target,
        &NodePath::from("scale"),
        &lean_left_scale.to_variant(),
        half_span,
    );

    // 回到中间
    tween.tween_property(
        &target,
        &NodePath::from("rotation_degrees"),
        &0.0_f32.to_variant(),
        half_span,
    );
    tween.parallel().tween_property(
        &target,
        &NodePath::from("scale"),
        &original_scale.to_variant(),
        half_span,
    );

    // 向右倾斜：旋转 + 缩放
    tween.tween_property(
        &target,
        &NodePath::from("rotation_degrees"),
        &max_rotation.to_variant(),
        half_span,
    );
    tween.parallel().tween_property(
        &target,
        &NodePath::from("scale"),
        &lean_right_scale.to_variant(),
        half_span,
    );

    // 回到中间
    tween.tween_property(
        &target,
        &NodePath::from("rotation_degrees"),
        &0.0_f32.to_variant(),
        half_span,
    );
    tween.parallel().tween_property(
        &target,
        &NodePath::from("scale"),
        &original_scale.to_variant(),
        half_span,
    );

    // 如果有固定时长，设置定时器结束
    if duration > 0.0 {
        let mut t2 = target.clone();
        let mut tween_clone = tween.clone();
        let key = anim_key.to_string();
        let stop_cb = Callable::from_fn("juice_walk_stop", move |_args| {
            if t2.is_instance_valid() {
                if tween_clone.is_instance_valid() {
                    tween_clone.kill();
                }
                t2.remove_meta(&StringName::from(&key));
            }
        });
        let mut timer_tween = target.create_tween();
        timer_tween.tween_interval(duration);
        timer_tween.tween_callback(&stop_cb);
    }

    Some(tween)
}

/// 入场动画（从界面外移入+透明度变化，按分组依次执行）
/// direction: 0=从下往上, 1=从上往下, 2=从左往右, 3=从右往左
/// container: 容器节点（None 则相对于视口）
/// duration: 总动画时长
/// delay: 组间延迟
/// mode: 0=按 anim_enter_ 前缀分组, 1=按子节点顺序逐个分组
pub fn anim_enter(
    target: &Gd<Node>,
    container: Option<Gd<Control>>,
    direction: i32,
    duration: f64,
    delay: f64,
    mode: i32,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_enter_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    // 收集分组
    let groups = collect_enter_groups(&target, mode);
    if groups.is_empty() {
        target.remove_meta(&StringName::from(anim_key));
        return None;
    }

    let mut group_ids: Vec<i32> = groups.keys().copied().collect();
    group_ids.sort();

    let mut tween = target.create_tween();

    for (group_idx, &group_id) in group_ids.iter().enumerate() {
        let nodes = &groups[&group_id];
        let mut first_in_group = true;

        for node in nodes {
            let mut n = node.clone();
            let target_position: Vector2 = n.call(&StringName::from("get_global_position"), &[]).to();
            let mut start_position = target_position;

            if let Some(ref container) = container {
                let container_rect = container.get_global_rect();
                match direction {
                    0 => start_position.y = container_rect.position.y + container_rect.size.y + 100.0,
                    1 => start_position.y = container_rect.position.y - 100.0,
                    2 => start_position.x = container_rect.position.x - 100.0,
                    3 => start_position.x = container_rect.position.x + container_rect.size.x + 100.0,
                    _ => {}
                }
            } else {
                let screen_size = DisplayServer::singleton().window_get_size();
                match direction {
                    0 => start_position.y = screen_size.y as f32 + 100.0,
                    1 => start_position.y = -100.0,
                    2 => start_position.x = -100.0,
                    3 => start_position.x = screen_size.x as f32 + 100.0,
                    _ => {}
                }
            }

            // 设置起始位置和透明度
            n.call(&StringName::from("set_global_position"), &[start_position.to_variant()]);
            n.call(&StringName::from("set_modulate"), &[Color::from_rgba(1.0, 1.0, 1.0, 0.0).to_variant()]);
            if n.has_method(&StringName::from("show")) {
                n.call(&StringName::from("show"), &[]);
            }

            // 位置动画
            if first_in_group {
                tween.tween_property(
                    &n,
                    &NodePath::from("global_position"),
                    &target_position.to_variant(),
                    duration,
                );
                first_in_group = false;
            } else {
                tween.parallel().tween_property(
                    &n,
                    &NodePath::from("global_position"),
                    &target_position.to_variant(),
                    duration,
                );
            }

            // 透明度动画（与位置动画并行）
            tween.parallel().tween_property(
                &n,
                &NodePath::from("modulate:a"),
                &1.0_f32.to_variant(),
                duration,
            );
        }

        // 组间延迟
        if group_idx < group_ids.len() - 1 {
            tween.tween_interval(delay);
        }
    }

    let cb = make_cleanup_callable(&target, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// 收集入场动画的分组节点
fn collect_enter_groups(target: &Gd<Node>, mode: i32) -> std::collections::HashMap<i32, Vec<Gd<Node>>> {
    let mut groups: std::collections::HashMap<i32, Vec<Gd<Node>>> = std::collections::HashMap::new();

    if mode == 0 {
        // 按 anim_enter_ 前缀分组
        let children = get_all_children(target);
        for child in children {
            let name = child.get_name().to_string();
            if name.starts_with("anim_enter_") {
                let name_without_prefix = &name["anim_enter_".len()..];
                let parts: Vec<&str> = name_without_prefix.split('_').collect();
                if !parts.is_empty() {
                    if let Ok(group_id) = parts[0].parse::<i32>() {
                        groups.entry(group_id).or_default().push(child);
                    }
                }
            }
        }
    } else if mode == 1 {
        // 按直接子节点顺序分组，每个节点一组
        let child_count = target.get_child_count();
        for i in 0..child_count {
            if let Some(child) = target.get_child(i) {
                let group_id = (i + 1) as i32;
                groups.entry(group_id).or_default().push(child);
            }
        }
    }

    groups
}

/// 递归获取所有子节点
fn get_all_children(node: &Gd<Node>) -> Vec<Gd<Node>> {
    let mut result = Vec::new();
    let child_count = node.get_child_count();
    for i in 0..child_count {
        if let Some(child) = node.get_child(i) {
            result.push(child.clone());
            result.extend(get_all_children(&child));
        }
    }
    result
}

/// 爆炸动画（子节点向四周散开并淡出，使用 Godot 内置缓动）
pub fn anim_explosion(
    target: &Gd<Node>,
    max_distance: f32,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_explosion_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    // 收集子节点原始状态
    let child_count = target.get_child_count();
    let mut child_states: Vec<(Gd<Node>, Vector2, Color)> = Vec::new();
    for i in 0..child_count {
        if let Some(mut child) = target.get_child(i) {
            if child.is_class(&GString::from("Node2D")) || child.is_class(&GString::from("Control")) {
                let pos: Vector2 = child.call(&StringName::from("get_position"), &[]).to();
                let modulate: Color = child.call(&StringName::from("get_self_modulate"), &[]).to();
                if child.has_method(&StringName::from("show")) {
                    child.call(&StringName::from("show"), &[]);
                }
                child_states.push((child, pos, modulate));
            }
        }
    }

    let mut tween = target.create_tween();
    tween.set_trans(godot::classes::tween::TransitionType::BACK);
    tween.set_ease(godot::classes::tween::EaseType::IN);

    // 为每个子节点创建散开+淡出动画
    let mut first_child = true;
    for (child, original_position, _original_modulate) in &child_states {
        // 随机方向
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let direction = Vector2::new(angle.cos(), angle.sin());
        let target_position = *original_position + direction * max_distance;

        // 移动动画
        if first_child {
            tween.tween_property(
                child,
                &NodePath::from("position"),
                &target_position.to_variant(),
                duration,
            );
            first_child = false;
        } else {
            tween.parallel().tween_property(
                child,
                &NodePath::from("position"),
                &target_position.to_variant(),
                duration,
            );
        }

        // 淡出动画（与移动并行）
        tween.parallel().tween_property(
            child,
            &NodePath::from("self_modulate:a"),
            &0.0_f32.to_variant(),
            duration,
        );
    }

    // 完成后恢复子节点状态
    let states_for_restore = child_states.clone();
    let restore_cb = Callable::from_fn("juice_explosion_restore", move |_args| {
        for (child, original_position, original_modulate) in &states_for_restore {
            if child.is_instance_valid() {
                let mut c = child.clone();
                if c.has_method(&StringName::from("set_position")) {
                    c.call(&StringName::from("set_position"), &[original_position.to_variant()]);
                }
                if c.has_method(&StringName::from("set_self_modulate")) {
                    c.call(&StringName::from("set_self_modulate"), &[original_modulate.to_variant()]);
                }
                if c.has_method(&StringName::from("hide")) {
                    c.call(&StringName::from("hide"), &[]);
                }
            }
        }
    });
    tween.tween_callback(&restore_cb);

    let cb = make_cleanup_callable(&target, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// 收集动画（子节点从随机位置汇聚到目标点，使用 Godot 内置缓动）
pub fn anim_collect(
    target: &Gd<Node>,
    target_position: Vector2,
    is_global: bool,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_anim_collect_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    // 收集子节点原始状态
    let child_count = target.get_child_count();
    let mut child_states: Vec<(Gd<Node>, Vector2)> = Vec::new();
    for i in 0..child_count {
        if let Some(mut child) = target.get_child(i) {
            if child.is_class(&GString::from("Node2D")) || child.is_class(&GString::from("Control")) {
                let pos = if is_global {
                    child.call(&StringName::from("get_global_position"), &[]).to()
                } else {
                    child.call(&StringName::from("get_position"), &[]).to()
                };
                if child.has_method(&StringName::from("show")) {
                    child.call(&StringName::from("show"), &[]);
                }
                child_states.push((child, pos));
            }
        }
    }

    let mut tween = target.create_tween();
    tween.set_trans(godot::classes::tween::TransitionType::SINE);
    tween.set_ease(godot::classes::tween::EaseType::IN_OUT);

    let mut first_child = true;
    for (child, original_position) in &child_states {
        // 随机偏移起始位置
        let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
        let random_distance = 10.0 + rand::random::<f32>() * 40.0;
        let offset = Vector2::new(random_angle.cos(), random_angle.sin()) * random_distance;
        let start_pos = *original_position + offset;

        let prop_path = if is_global {
            NodePath::from("global_position")
        } else {
            NodePath::from("position")
        };

        // 设置起始位置
        let mut c = child.clone();
        c.call(&StringName::from(&prop_path.to_string()), &[start_pos.to_variant()]);

        // 移动到目标位置
        if first_child {
            tween.tween_property(
                child,
                &prop_path,
                &target_position.to_variant(),
                duration,
            );
            first_child = false;
        } else {
            tween.parallel().tween_property(
                child,
                &prop_path,
                &target_position.to_variant(),
                duration,
            );
        }
    }

    // 完成后恢复子节点状态
    let states_for_restore = child_states.clone();
    let is_global_restore = is_global;
    let restore_cb = Callable::from_fn("juice_collect_restore", move |_args| {
        for (child, original_position) in &states_for_restore {
            if child.is_instance_valid() {
                let mut c = child.clone();
                if c.has_method(&StringName::from("hide")) {
                    c.call(&StringName::from("hide"), &[]);
                }
                if is_global_restore {
                    if c.has_method(&StringName::from("set_global_position")) {
                        c.call(&StringName::from("set_global_position"), &[original_position.to_variant()]);
                    }
                } else {
                    if c.has_method(&StringName::from("set_position")) {
                        c.call(&StringName::from("set_position"), &[original_position.to_variant()]);
                    }
                }
            }
        }
    });
    tween.tween_callback(&restore_cb);

    let cb = make_cleanup_callable(&target, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// 打击标签动画（缩放+颜色闪烁+上浮淡出，使用 Godot 内置缓动）
pub fn hit_label(
    target: &Gd<Node>,
    duration: f64,
) -> Option<Gd<Tween>> {
    let anim_key = "_juice_hit_label_handling";
    if check_is_handle(target, anim_key) {
        return None;
    }
    if !target.has_method(&StringName::from("get_scale")) {
        return None;
    }

    let mut target = target.clone();
    mark_handle(&mut target, anim_key);

    if target.has_method(&StringName::from("show")) {
        target.call(&StringName::from("show"), &[]);
    }

    let initial_scale: Vector2 = target.call(&StringName::from("get_scale"), &[]).to();
    let initial_position: Vector2 = target.call(&StringName::from("get_position"), &[]).to();
    let initial_modulate: Color = target.call(&StringName::from("get_modulate"), &[]).to();

    // 确保开始时完全不透明
    target.call(
        &StringName::from("set_modulate"),
        &[Color::from_rgba(initial_modulate.r, initial_modulate.g, initial_modulate.b, 1.0).to_variant()],
    );

    let large_scale = Vector2::new(1.5, 1.5);
    let small_scale = Vector2::new(0.5, 0.5);

    let mut tween = target.create_tween();

    // 第一阶段：缩放动画（从小变大）
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &large_scale.to_variant(),
        0.2,
    ).from(&small_scale.to_variant())
     .set_trans(godot::classes::tween::TransitionType::BACK)
     .set_ease(godot::classes::tween::EaseType::OUT);

    // 从大变回正常
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &initial_scale.to_variant(),
        0.1,
    ).from(&large_scale.to_variant());

    // 并行：颜色变化
    let mut color_tween = tween.parallel();
    let color_segment = 0.3 / 4.0;
    let aqua = Color::from_rgb(0.0, 1.0, 1.0);
    let gold = Color::from_rgb(1.0, 0.843137, 0.0);
    let crimson = Color::from_rgb(0.863, 0.078, 0.235);
    let initial_opaque = Color::from_rgba(initial_modulate.r, initial_modulate.g, initial_modulate.b, 1.0);

    color_tween.tween_property(&target, &NodePath::from("modulate"), &aqua.to_variant(), color_segment);
    color_tween.tween_property(&target, &NodePath::from("modulate"), &gold.to_variant(), color_segment);
    color_tween.tween_property(&target, &NodePath::from("modulate"), &crimson.to_variant(), color_segment);
    color_tween.tween_property(&target, &NodePath::from("modulate"), &initial_opaque.to_variant(), color_segment);

    // 第二阶段：上浮淡出
    let target_y = initial_position.y - 20.0;
    tween.tween_property(
        &target,
        &NodePath::from("position:y"),
        &target_y.to_variant(),
        0.3,
    );

    // 并行：透明度变为0
    let mut fade_tween = tween.parallel();
    fade_tween.tween_property(&target, &NodePath::from("modulate:a"), &0.0_f32.to_variant(), 0.3);

    let cb = make_cleanup_callable(&target, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

// ===== UI 专用快捷动画 =====

/// 弹窗弹入动画（缩放+淡入，使用 Godot 内置缓动）
pub fn popup_enter(target: &Gd<Control>, duration: f64) -> Option<Gd<Tween>> {
    let node = target.clone().upcast::<Node>();
    let anim_key = "_juice_popup_enter_handling";
    if check_is_handle(&node, anim_key) {
        return None;
    }

    let mut target = target.clone();
    let mut node_mut = node.clone();
    mark_handle(&mut node_mut, anim_key);

    let mut tween = target.create_tween();

    // 设置初始状态
    target.set_scale(Vector2::new(0.8, 0.8));
    target.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));

    // 缩放动画
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &Vector2::new(1.0, 1.0).to_variant(),
        duration,
    ).set_trans(godot::classes::tween::TransitionType::BACK)
     .set_ease(godot::classes::tween::EaseType::OUT);

    // 并行：淡入
    let mut alpha_tween = tween.parallel();
    alpha_tween.tween_property(
        &target,
        &NodePath::from("modulate:a"),
        &1.0_f32.to_variant(),
        duration * 0.6,
    );

    let n = target.clone().upcast::<Node>();
    let cb = make_cleanup_callable(&n, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// 弹窗弹出动画（缩放+淡出，使用 Godot 内置缓动）
pub fn popup_exit(target: &Gd<Control>, duration: f64) -> Option<Gd<Tween>> {
    let node = target.clone().upcast::<Node>();
    let anim_key = "_juice_popup_exit_handling";
    if check_is_handle(&node, anim_key) {
        return None;
    }

    let mut target = target.clone();
    let mut node_mut = node.clone();
    mark_handle(&mut node_mut, anim_key);

    let mut tween = target.create_tween();

    // 缩放动画
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &Vector2::new(0.8, 0.8).to_variant(),
        duration,
    ).set_trans(godot::classes::tween::TransitionType::SINE)
     .set_ease(godot::classes::tween::EaseType::IN);

    // 并行：淡出
    let mut alpha_tween = tween.parallel();
    alpha_tween.tween_property(
        &target,
        &NodePath::from("modulate:a"),
        &0.0_f32.to_variant(),
        duration,
    );

    let n = target.clone().upcast::<Node>();
    let cb = make_cleanup_callable(&n, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// 点击缩放反馈动画（使用 Godot 内置缓动）
pub fn click_feedback(target: &Gd<Control>) -> Option<Gd<Tween>> {
    let node = target.clone().upcast::<Node>();
    let anim_key = "_juice_click_feedback_handling";
    if check_is_handle(&node, anim_key) {
        return None;
    }

    let mut target = target.clone();
    let mut node_mut = node.clone();
    mark_handle(&mut node_mut, anim_key);

    let original_scale = target.get_scale();

    let mut tween = target.create_tween();

    // 缩小
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &(original_scale * 0.9).to_variant(),
        0.05,
    );

    // 弹回
    tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &original_scale.to_variant(),
        0.15,
    ).set_trans(godot::classes::tween::TransitionType::BACK)
     .set_ease(godot::classes::tween::EaseType::OUT);

    let n = target.clone().upcast::<Node>();
    let cb = make_cleanup_callable(&n, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// Tooltip 淡入动画（使用 Godot 内置缓动）
pub fn tooltip_fade_in(target: &Gd<Control>, duration: f64) -> Option<Gd<Tween>> {
    let node = target.clone().upcast::<Node>();
    let anim_key = "_juice_tooltip_fade_in_handling";
    if check_is_handle(&node, anim_key) {
        return None;
    }

    let mut target = target.clone();
    let mut node_mut = node.clone();
    mark_handle(&mut node_mut, anim_key);

    target.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));
    target.set_scale(Vector2::new(0.95, 0.95));

    let mut tween = target.create_tween();

    tween.tween_property(
        &target,
        &NodePath::from("modulate:a"),
        &1.0_f32.to_variant(),
        duration,
    ).set_trans(godot::classes::tween::TransitionType::SINE)
     .set_ease(godot::classes::tween::EaseType::OUT);

    let mut scale_tween = tween.parallel();
    scale_tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &Vector2::new(1.0, 1.0).to_variant(),
        duration,
    ).set_trans(godot::classes::tween::TransitionType::BACK)
     .set_ease(godot::classes::tween::EaseType::OUT);

    let n = target.clone().upcast::<Node>();
    let cb = make_cleanup_callable(&n, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}

/// Tooltip 淡出动画（使用 Godot 内置缓动）
pub fn tooltip_fade_out(target: &Gd<Control>, duration: f64) -> Option<Gd<Tween>> {
    let node = target.clone().upcast::<Node>();
    let anim_key = "_juice_tooltip_fade_out_handling";
    if check_is_handle(&node, anim_key) {
        return None;
    }

    let mut target = target.clone();
    let mut node_mut = node.clone();
    mark_handle(&mut node_mut, anim_key);

    let mut tween = target.create_tween();

    tween.tween_property(
        &target,
        &NodePath::from("modulate:a"),
        &0.0_f32.to_variant(),
        duration,
    ).set_trans(godot::classes::tween::TransitionType::SINE)
     .set_ease(godot::classes::tween::EaseType::IN);

    let mut scale_tween = tween.parallel();
    scale_tween.tween_property(
        &target,
        &NodePath::from("scale"),
        &Vector2::new(0.95, 0.95).to_variant(),
        duration,
    ).set_trans(godot::classes::tween::TransitionType::SINE)
     .set_ease(godot::classes::tween::EaseType::IN);

    let n = target.clone().upcast::<Node>();
    let cb = make_cleanup_callable(&n, anim_key);
    tween.tween_callback(&cb);

    Some(tween)
}
