// UI 列表辅助工具
// 翻译自 C++ gmlc/ui_list_helper.h/cpp
// 包含 GdListHelper（列表初始化/更新/节点值设置/信号绑定）
// 以及 GdSlotHighlight（方形/圆形高亮效果）、GdSlotFill（方形/圆形填充效果）

use godot::prelude::*;
use godot::builtin::{GString, StringName, Color, Variant, Array, Dictionary, NodePath};
use godot::classes::{
    Control, ColorRect, Shader, ShaderMaterial, ResourceLoader,
};
use godot::classes::control::{LayoutPreset, MouseFilter};
use godot::obj::NewGd;

/// 方形高亮 Shader 代码
const SQUARE_OUTLINE_SHADER: &str = r#"
shader_type canvas_item;
render_mode unshaded;

uniform float border_width : hint_range(0, 0.5) = 0.02;
uniform vec4 border_color : source_color = vec4(1.0, 1.0, 0.0, 1.0);
uniform vec4 fill_color : source_color = vec4(0.0, 0.0, 0.0, 0.0);

void fragment() {
    vec2 uv = UV;
    float left = border_width;
    float right = 1.0 - border_width;
    float top = border_width;
    float bottom = 1.0 - border_width;
    if (uv.x < left || uv.x > right || uv.y < top || uv.y > bottom) {
        COLOR = border_color;
    } else {
        COLOR = fill_color;
    }
}
"#;

/// 圆形高亮 Shader 代码
const CIRCLE_OUTLINE_SHADER: &str = r#"
shader_type canvas_item;
render_mode unshaded;

uniform float border_width : hint_range(0, 0.5) = 0.02;
uniform vec4 border_color : source_color = vec4(1.0, 1.0, 0.0, 1.0);
uniform vec4 fill_color : source_color = vec4(0.0, 0.0, 0.0, 0.0);

void fragment() {
    vec2 uv = UV;
    vec2 center = vec2(0.5, 0.5);
    float distance = length(uv - center);
    if (distance > 0.5 - border_width && distance <= 0.5) {
        COLOR = border_color;
    } else if (distance <= 0.5 - border_width) {
        COLOR = fill_color;
    } else {
        COLOR = vec4(0.0);
    }
}
"#;

/// 方形填充 Shader 代码
const SQUARE_INTERIOR_SHADER: &str = r#"
shader_type canvas_item;
render_mode unshaded;

uniform float padding : hint_range(0, 0.5) = 0.02;
uniform vec4 interior_color : source_color = vec4(1.0, 1.0, 0.0, 1.0);

void fragment() {
    vec2 uv = UV;
    float left = padding;
    float right = 1.0 - padding;
    float top = padding;
    float bottom = 1.0 - padding;
    if (uv.x >= left && uv.x <= right && uv.y >= top && uv.y <= bottom) {
        COLOR = interior_color;
    } else {
        COLOR = vec4(0.0);
    }
}
"#;

/// 圆形填充 Shader 代码
const CIRCLE_INTERIOR_SHADER: &str = r#"
shader_type canvas_item;
render_mode unshaded;

uniform float padding : hint_range(0, 0.5) = 0.02;
uniform vec4 interior_color : source_color = vec4(1.0, 1.0, 0.0, 1.0);

void fragment() {
    vec2 uv = UV;
    vec2 center = vec2(0.5, 0.5);
    float distance = length(uv - center);
    float interior_radius = 0.5 - padding;
    if (distance <= interior_radius) {
        COLOR = interior_color;
    } else {
        COLOR = vec4(0.0);
    }
}
"#;

// ===== GdSlotHighlight =====

/// 创建方形高亮节点
pub fn create_square_highlight_node(border_width: f32, border_color: Color) -> Gd<Control> {
    let mut outline = ColorRect::new_alloc();
    let mut shader_material = ShaderMaterial::new_gd();
    let mut shader = Shader::new_gd();

    shader.set_code(&GString::from(SQUARE_OUTLINE_SHADER));
    shader_material.set_shader(&shader);
    shader_material.set_shader_parameter(&StringName::from("border_width"), &border_width.to_variant());
    shader_material.set_shader_parameter(&StringName::from("border_color"), &border_color.to_variant());

    outline.set_material(&shader_material);
    outline.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
    outline.set_mouse_filter(MouseFilter::IGNORE);

    let mut highlight_node = Control::new_alloc();
    highlight_node.set_mouse_filter(MouseFilter::IGNORE);
    highlight_node.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
    highlight_node.add_child(&outline);

    highlight_node
}

/// 创建圆形高亮节点
pub fn create_circle_highlight_node(border_width: f32, border_color: Color) -> Gd<Control> {
    let mut outline = ColorRect::new_alloc();
    let mut shader_material = ShaderMaterial::new_gd();
    let mut shader = Shader::new_gd();

    shader.set_code(&GString::from(CIRCLE_OUTLINE_SHADER));
    shader_material.set_shader(&shader);
    shader_material.set_shader_parameter(&StringName::from("border_width"), &border_width.to_variant());
    shader_material.set_shader_parameter(&StringName::from("border_color"), &border_color.to_variant());

    outline.set_material(&shader_material);
    outline.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
    outline.set_mouse_filter(MouseFilter::IGNORE);

    let mut highlight_node = Control::new_alloc();
    highlight_node.set_mouse_filter(MouseFilter::IGNORE);
    highlight_node.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
    highlight_node.add_child(&outline);

    highlight_node
}

// ===== GdSlotFill =====

/// 创建方形填充节点
pub fn create_square_fill_node(interior_color: Color) -> Gd<Control> {
    let mut outline = ColorRect::new_alloc();
    let mut shader_material = ShaderMaterial::new_gd();
    let mut shader = Shader::new_gd();

    shader.set_code(&GString::from(SQUARE_INTERIOR_SHADER));
    shader_material.set_shader(&shader);
    shader_material.set_shader_parameter(&StringName::from("interior_color"), &interior_color.to_variant());

    outline.set_material(&shader_material);
    outline.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
    outline.set_mouse_filter(MouseFilter::IGNORE);

    let mut fill_node = Control::new_alloc();
    fill_node.set_mouse_filter(MouseFilter::IGNORE);
    fill_node.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
    fill_node.add_child(&outline);

    fill_node
}

/// 创建圆形填充节点
pub fn create_circle_fill_node(interior_color: Color) -> Gd<Control> {
    let mut outline = ColorRect::new_alloc();
    let mut shader_material = ShaderMaterial::new_gd();
    let mut shader = Shader::new_gd();

    shader.set_code(&GString::from(CIRCLE_INTERIOR_SHADER));
    shader_material.set_shader(&shader);
    shader_material.set_shader_parameter(&StringName::from("interior_color"), &interior_color.to_variant());

    outline.set_material(&shader_material);
    outline.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
    outline.set_mouse_filter(MouseFilter::IGNORE);

    let mut fill_node = Control::new_alloc();
    fill_node.set_mouse_filter(MouseFilter::IGNORE);
    fill_node.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
    fill_node.add_child(&outline);

    fill_node
}

// ===== GdListHelper =====

/// 列表初始化：根据 count 复制/删除 slot 子节点
pub fn list_initial(target: &mut Gd<Control>, slot: &Gd<Control>, count: i32) {
    let mut slot = slot.clone();
    slot.set_visible(false);

    if count <= 0 {
        return;
    }

    let current_count = target.get_child_count();
    if current_count < count {
        // 需要添加节点
        for _ in 0..(count - current_count) {
            let mut cc = slot.duplicate_node(); // 复制节点
            copy_signal_meta(&slot, &mut cc);
            cc.set_owner(Gd::null_arg());
            target.add_child(&cc);
            cc.set_visible(true);
        }
    } else if current_count > count {
        // 需要移除多余节点
        for i in (count..current_count).rev() {
            if let Some(child) = target.get_child(i) {
                let mut c = child;
                c.set_owner(Gd::null_arg());
                target.remove_child(&c);
                c.queue_free();
            }
        }
    }
}

/// 更新数据别名：将 @idx:key 格式的 key 替换为 slots 中的路径
pub fn update_data_alias(data: &Array<Variant>, slots: &Array<Variant>) -> Array<Variant> {
    let mut new_res = Array::new();
    for i in 0..data.len() {
        if let Some(item) = data.get(i) {
            let mut args: Dictionary<Variant, Variant> = Dictionary::new();
            let ori: Dictionary<Variant, Variant> = item.try_to::<Dictionary<Variant, Variant>>().unwrap_or_default();
            let ori_keys = ori.keys_array();

            for ki in 0..ori_keys.len() {
                if let Some(ori_key_var) = ori_keys.get(ki) {
                    let ori_key = ori_key_var.to_string();
                    if ori_key.starts_with('@') {
                        // @idx:key 格式
                        let rest = &ori_key[1..];
                        let parts: Vec<&str> = rest.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            if let Ok(idx) = parts[0].parse::<i32>() {
                                if idx >= 0 && (idx as usize) < slots.len() {
                                    if let Some(np_var) = slots.get(idx as usize) {
                                        let mut np = np_var.to_string();
                                        if np.contains('/') {
                                            np = np[np.find('/').unwrap() + 1..].to_string();
                                        } else {
                                            np = ".".to_string();
                                        }
                                        let new_key = format!("{}:{}", np, parts[1]);
                                        args.set(&Variant::from(new_key.as_str()), &ori.get(&ori_key_var).unwrap_or(Variant::nil()));
                                    }
                                } else {
                                    args.set(&Variant::from(ori_key.as_str()), &ori.get(&ori_key_var).unwrap_or(Variant::nil()));
                                }
                            }
                        }
                    } else {
                        args.set(&Variant::from(ori_key.as_str()), &ori.get(&ori_key_var).unwrap_or(Variant::nil()));
                    }
                }
            }
            new_res.push(&args);
        }
    }
    new_res
}

/// 把模板树中声明的 __signal_* 元数据（@pressed / on_pressed）同步到 duplicate 出的条目树
/// （Node.duplicate 不复制 meta；条目重建后信号绑定声明需随条目存活，
/// 由 bind_events -> auto_bind_item_signals 据此重连）
fn copy_signal_meta(src: &Gd<Control>, dst: &mut Gd<Control>) {
    for (signal_name, method_name) in crate::ui::gdui_builder::get_signal_meta_list(src) {
        dst.set_meta(
            &StringName::from(format!("__signal_{}", signal_name).as_str()),
            &method_name.to_variant(),
        );
    }
    let src_children = src.get_children();
    for i in 0..src_children.len() {
        if let Some(sc) = src_children.get(i) {
            if let Ok(src_ctrl) = sc.clone().try_cast::<Control>() {
                if let Some(dc) = dst.get_child(i as i32) {
                    if let Ok(mut dst_ctrl) = dc.try_cast::<Control>() {
                        copy_signal_meta(&src_ctrl, &mut dst_ctrl);
                    }
                }
            }
        }
    }
}

/// 更新容器：根据 data 数组动态创建/删除/更新子节点
pub fn update_container(target: &mut Gd<Control>, slot: &Gd<Control>, count: i32, data: &Array<Variant>) {
    let slot = slot.clone();
    // 列表过滤（GML filter_key + filter_value_var 声明 → __filter_* meta）：
    // 只保留 item[filter_key] == filter_value 的条目；未声明过滤或过滤值为空串时显示全部。
    // 在建条目前统一过滤，bean: 响应式绑定推来的全量数据 / 直接 update() 的外部数据
    // 都在此生效，列表声明 filter 后即成为"只显示匹配分类"的视图
    let data = apply_list_filter(target, data);
    let data_size = data.len() as i32;
    let _target_name = target.get_name().to_string();
    // slot 模板始终在 index 0，可见子节点从 index 1 开始
    let visible_count = target.get_child_count() - 1;
    // //godot_print!("[ListHelper] update_container: node='{}', count={}, data_size={}, child_count={}, visible_count={}", target_name, count, data_size, target.get_child_count(), visible_count);

    // 动态调整可见子节点数量（不含 slot 模板）
    if count > 0 {
        if visible_count > data_size && count > data_size {
            // 清理多余可见节点（从末尾移除，跳过 index 0 的 slot）
            for i in ((data_size + 1)..=(visible_count.min(count))).rev() {
                if let Some(child) = target.get_child(i) {
                    let mut c = child;
                    c.set_owner(Gd::null_arg());
                    target.remove_child(&c);
                    c.queue_free();
                }
            }
        } else if count < data_size && visible_count < data_size {
            // 创建不足的可见节点
            for _ in visible_count..data_size {
                let mut cc = slot.duplicate_node();
                copy_signal_meta(&slot, &mut cc);
                cc.set_owner(Gd::null_arg());
                cc.set_custom_minimum_size(slot.get_custom_minimum_size());
                target.add_child(&cc);
                cc.set_visible(true);
            }
        }
    } else {
        // count <= 0 时，按 data 大小动态调整
        for _ in visible_count..data_size {
            let mut cc = slot.duplicate_node();
            copy_signal_meta(&slot, &mut cc);
            cc.set_owner(Gd::null_arg());
            cc.set_custom_minimum_size(slot.get_custom_minimum_size());
            target.add_child(&cc);
            cc.set_visible(true);
        }
        for i in ((data_size + 1)..=(visible_count)).rev() {
            if let Some(child) = target.get_child(i) {
                let mut c = child;
                c.set_owner(Gd::null_arg());
                target.remove_child(&c);
                c.queue_free();
            }
        }
    }

    // 更新可见子节点数据（跳过 index 0 的 slot 模板）
    let children = target.get_children();
    for i in 0..data.len() {
        if let Some(data_item) = data.get(i) {
            if data_item.get_type() == godot::builtin::VariantType::DICTIONARY {
                // data[i] 映射到 children[i+1]（跳过 slot 模板）
                if let Some(child_var) = children.get(i + 1) {
                    if let Ok(mut c) = child_var.clone().try_cast::<Control>() {
                        let spec: Dictionary<Variant, Variant> = data_item.try_to::<Dictionary<Variant, Variant>>().unwrap_or_default();
                        let keys = spec.keys_array();

                        // 分离简单 key 和路径 key
                        let mut simple_keys: Vec<(String, Variant)> = Vec::new();
                        let mut path_keys: Vec<(String, Variant)> = Vec::new();
                        for ki in 0..keys.len() {
                            if let Some(key_var) = keys.get(ki) {
                                let key = key_var.to_string();
                                let val = spec.get(&key_var).unwrap_or(Variant::nil());
                                if key.contains(':') || key.contains('/') {
                                    path_keys.push((key, val));
                                } else {
                                    simple_keys.push((key, val));
                                }
                            }
                        }

                        // 先处理路径 key（兼容旧格式）
                        for (key, val) in &path_keys {
                            update_node_value(&mut c, key, val);
                        }

                        // 处理简单 key：通过模板绑定解析
                        let mut used_keys: Vec<String> = Vec::new();

                        // 数据契约注入：条目脚本 @export 声明的同名变量直接写入脚本实例
                        // （@export 即契约：只有声明了的变量才会被注入，
                        //   编辑条目 GML 时对照 <ui script> 脚本的 @export 即知会传入哪些数据）
                        if !simple_keys.is_empty() {
                            let export_vars = collect_export_var_names(&c);
                            if !export_vars.is_empty() {
                                for (key, val) in &simple_keys {
                                    if export_vars.contains(key) {
                                        c.set(&StringName::from(key.as_str()), val);
                                    }
                                }
                            }
                        }

                        resolve_template_bindings_recursive(&mut c, &simple_keys, &mut used_keys);

                        // 未被模板绑定使用的简单 key，存储为 meta
                        for (key, val) in &simple_keys {
                            if !used_keys.contains(key) {
                                c.set_meta(&StringName::from(key.as_str()), val);
                            }
                        }

                        // 存储完整数据字典为 __item_data meta，供 Tooltip 的 update_data 使用
                        c.set_meta(&StringName::from("__item_data"), &spec.to_variant());
                    }
                }
            }
        }
    }

    // 重置 data.size 到 visible_count 之间的数据为默认值
    let default_value = get_default_exported_variables(&slot);
    let new_visible_count = (target.get_child_count() - 1) as usize;
    if new_visible_count > data.len() {
        for i in data.len()..new_visible_count {
            // children[i+1] 跳过 slot 模板
            if let Some(child_var) = children.get(i + 1) {
                if let Ok(mut c) = child_var.clone().try_cast::<Control>() {
                    let keys = default_value.keys_array();
                    for ki in 0..keys.len() {
                        if let Some(key_var) = keys.get(ki) {
                            let key = key_var.to_string();
                            let val = default_value.get(&key_var).unwrap_or(Variant::nil());
                            update_node_value(&mut c, &key, &val);
                        }
                    }
                }
            }
        }
    }
}

/// 应用列表过滤：__filter_key（数据字段名）+ __filter_value（匹配值）meta 由
/// GdUiBuilder 从 GML filter_key / filter_value_var 属性注入。
/// 返回过滤后的数组；未声明过滤、缺任一 meta 或过滤值为空串时原样返回。
fn apply_list_filter(target: &Gd<Control>, data: &Array<Variant>) -> Array<Variant> {
    let key_sn = StringName::from("__filter_key");
    let value_sn = StringName::from("__filter_value");
    if !target.has_meta(&key_sn) || !target.has_meta(&value_sn) {
        return data.clone();
    }
    let key = target.get_meta(&key_sn).to_string();
    let filter_value = target.get_meta(&value_sn);
    // 空串过滤值视为"未指定分类"：不过滤（列表组件独立打开的合理默认）
    if filter_value.get_type() == VariantType::STRING && filter_value.to_string().is_empty() {
        return data.clone();
    }
    let mut filtered = Array::new();
    for i in 0..data.len() {
        if let Some(item) = data.get(i) {
            if item.get_type() != godot::builtin::VariantType::DICTIONARY {
                continue;
            }
            let dict: Dictionary<Variant, Variant> = item
                .clone()
                .try_to::<Dictionary<Variant, Variant>>()
                .unwrap_or_default();
            if let Some(v) = dict.get(&Variant::from(key.as_str())) {
                if v == filter_value {
                    filtered.push(&item);
                }
            }
        }
    }
    filtered
}

/// 更新单个子节点的字典数据
/// child_index 是可见子节点的索引（0-based），内部 +1 跳过 slot 模板
pub fn update_child_dict(target: &mut Gd<Control>, child_index: i32, data: &Dictionary<Variant, Variant>) {
    // +1 跳过 index 0 的 slot 模板
    let actual_index = child_index + 1;
    if actual_index < 1 || actual_index >= target.get_child_count() {
        return;
    }
    if let Some(child) = target.get_child(actual_index) {
        if let Ok(mut c) = child.try_cast::<Control>() {
            let keys = data.keys_array();
            for ki in 0..keys.len() {
                if let Some(key_var) = keys.get(ki) {
                    let key = key_var.to_string();
                    let val = data.get(&key_var).unwrap_or(Variant::nil());
                    update_node_value(&mut c, &key, &val);
                }
            }
        }
    }
}

/// 更新单个子节点的单个属性
/// child_index 是可见子节点的索引（0-based），内部 +1 跳过 slot 模板
pub fn update_child(target: &mut Gd<Control>, child_index: i32, key: &str, value: &Variant) {
    // +1 跳过 index 0 的 slot 模板
    let actual_index = child_index + 1;
    if actual_index < 1 || actual_index >= target.get_child_count() {
        return;
    }
    if let Some(child) = target.get_child(actual_index) {
        if let Ok(mut c) = child.try_cast::<Control>() {
            update_node_value(&mut c, key, value);
        }
    }
}

/// 更新节点值：支持 node_path:attr 格式
/// 格式说明：
///   "attr"           -> 设置当前节点的属性
///   "path:attr"      -> 设置子节点 path 的属性
///   "meta:key"       -> 设置节点的 meta 数据
///   "slot:fill"      -> 设置填充效果
///   "@method"        -> 调用方法而非设置属性
pub fn update_node_value(container: &mut Gd<Control>, node_spec: &str, value: &Variant) {
    let parts: Vec<&str> = node_spec.splitn(2, ':').collect();
    let (path_part, attr_part) = if parts.len() == 1 {
        (".", parts[0])
    } else if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        return;
    };

    if path_part == "meta" {
        container.set_meta(&StringName::from(attr_part), value);
    } else if path_part == "slot" {
        if attr_part == "fill" && value.get_type() == godot::builtin::VariantType::DICTIONARY {
            let dict: Dictionary<Variant, Variant> = value.try_to::<Dictionary<Variant, Variant>>().unwrap_or_default();
            if let Some(color_var) = dict.get(&"color".to_variant()) {
                let fill_color: Color = color_var.try_to::<Color>().unwrap_or(Color::WHITE);
                if let Some(mode_var) = dict.get(&"mode".to_variant()) {
                    let fill_mode: i32 = mode_var.try_to::<i32>().unwrap_or(0);
                    update_slot_fill(container, fill_color, fill_mode);
                }
            }
        }
    } else {
        // 尝试获取子节点
        let node_path = NodePath::from(path_part);
        if let Some(node) = container.get_node_or_null(&node_path) {
            if let Ok(mut l) = node.try_cast::<Control>() {
                let mut attr_name = attr_part.to_string();
                let mut is_method = false;
                let mut val = value.clone();

                if attr_name.starts_with('@') {
                    is_method = true;
                    attr_name = attr_name[1..].to_string();
                }

                // 处理特殊属性
                if attr_name == "texture" || attr_name == "texture_normal" {
                    if value.get_type() == godot::builtin::VariantType::STRING {
                        let v = value.to_string();
                        if v.is_empty() {
                            val = Variant::nil();
                        } else {
                            let path = GString::from(&v);
                            if let Some(res) = ResourceLoader::singleton().load(&path) {
                                val = res.to_variant();
                            }
                        }
                    }
                }

                if is_method {
                    l.call(&StringName::from(attr_name.as_str()), &[val]);
                } else {
                    l.set(&StringName::from(attr_name.as_str()), &val);
                }
            }
        }
    }
}

/// 批量绑定信号到所有子节点
pub fn allbind_signal(container: &mut Gd<Control>, path: &str, sig: &str, cb: &Callable) {
    let child_count = container.get_child_count();
    for i in 0..child_count {
        if let Some(child) = container.get_child(i) {
            if let Ok(c) = child.clone().try_cast::<Control>() {
                let node_path = NodePath::from(path);
                if let Some(target) = c.get_node_or_null(&node_path) {
                    if let Ok(mut target_ctrl) = target.try_cast::<Control>() {
                        let bound_cb = cb.bind(&[target_ctrl.to_variant()]);
                        if !target_ctrl.is_connected(&StringName::from(sig), &bound_cb) {
                            target_ctrl.connect(&StringName::from(sig), &bound_cb);
                        }
                    }
                }
            }
        }
    }
}

/// 自动连接条目内声明的信号绑定（@pressed / on_pressed 属性 → __signal_xxx meta）。
/// 信号目标从列表节点的 __signal_target meta 读取（connect_signals 递归时记录到列表控件上）。
/// 条目由 slot.duplicate() 创建：指向外部脚本的连接不会随 duplicate 复制，
/// 因此 update() 重建条目后需重新连接——各列表 bind_events 在每次更新后调用本函数。
pub fn auto_bind_item_signals(list: &Gd<Control>) {
    if !list.has_meta(&StringName::from("__gml_signal_target")) {
        return;
    }
    let target_var = list.get_meta(&StringName::from("__gml_signal_target"));
    let Ok(target) = target_var.try_to::<Gd<Object>>() else {
        return;
    };
    let children = list.get_children();
    // 从 index 1 开始，跳过 index 0 的 slot 模板
    for i in 1..children.len() {
        if let Some(child) = children.get(i) {
            if let Ok(mut item) = child.clone().try_cast::<Control>() {
                connect_signal_meta_recursive(&mut item, &target);
            }
        }
    }
}

/// 就近解析信号绑定目标：从 node 沿父链向上（含自身）找最近一个
/// "挂了脚本且实现了该方法"的节点——条目根由 <ui script> 挂载脚本后，
/// 条目内 @pressed 声明优先绑定条目自身脚本而非外部场景；
/// 找不到时回退 fallback（场景连接目标）
fn resolve_target_for_method(
    node: &Gd<Control>,
    method: &StringName,
    fallback: &Gd<Object>,
) -> Gd<Object> {
    let mut cur = Some(node.clone().upcast::<godot::classes::Node>());
    while let Some(n) = cur {
        let parent = n.get_parent();
        if n.get_script().is_some() && n.has_method(method) {
            return n.upcast::<godot::classes::Object>();
        }
        cur = parent;
    }
    fallback.clone()
}

/// 连接单个节点上声明的全部 __signal_xxx 信号绑定（@pressed / on_pressed）
/// 目标方法要求的参数多于信号提供的参数时，补绑发出节点——
/// 条目回调可写成 _on_item(btn) 直接拿到按钮引用（与 allbind_signal 语义一致）
pub fn connect_node_signal_meta(node: &mut Gd<Control>, fallback: &Gd<Object>) {
    for (signal_name, method_name) in crate::ui::gdui_builder::get_signal_meta_list(node) {
        let method_sn = StringName::from(method_name.as_str());
        // 就近解析绑定目标：优先条目自身挂载的脚本（<ui script>），回退场景目标
        let target = resolve_target_for_method(node, &method_sn, fallback);
        if !target.has_method(&method_sn) {
            godot_warn!(
                "[GdUiBuilder] 信号绑定方法不存在: {}()（信号 {}，可检查 GML 中 @/on_ 声明与目标脚本）",
                method_name, signal_name
            );
            continue;
        }
        let sig_sn = StringName::from(signal_name.as_str());
        let callable = Callable::from_object_method(&target, &method_sn);
        let extra = required_arg_count(&target, &method_sn)
            .map(|req| req.saturating_sub(signal_arg_count(node, signal_name.as_str())))
            .unwrap_or(0);
        let bound = if extra > 0 {
            let mut bind_args: Vec<Variant> = Vec::new();
            bind_args.push(node.clone().to_variant());
            for _ in 1..extra {
                bind_args.push(Variant::nil());
            }
            callable.bind(&bind_args)
        } else {
            callable
        };
        if !node.is_connected(&sig_sn, &bound) {
            node.connect(&sig_sn, &bound);
        }
    }
}

/// 递归连接节点树中带 __signal_xxx 元数据的节点到目标脚本
fn connect_signal_meta_recursive(node: &mut Gd<Control>, target: &Gd<Object>) {
    connect_node_signal_meta(node, target);

    let children = node.get_children();
    for i in 0..children.len() {
        if let Some(child) = children.get(i) {
            if let Ok(mut c) = child.clone().try_cast::<Control>() {
                connect_signal_meta_recursive(&mut c, target);
            }
        }
    }
}

/// 目标方法的必填参数个数（总参数 - 默认参数）；查不到方法信息时返回 None
fn required_arg_count(target: &Gd<Object>, method: &StringName) -> Option<usize> {
    let list = target.get_method_list();
    for i in 0..list.len() {
        let Some(m) = list.get(i) else { continue; };
        let name = m.get_or_nil(&"name".to_variant()).to_string();
        if name == method.to_string() {
            // 方法列表中的 args/default_args 为 typed Array，gdext 的 try_to 转换会失败，
            // 改用 Variant.call("size") 取长度
            let size_of = |v: Variant| -> usize {
                if v.get_type() == VariantType::ARRAY {
                    v.call(&StringName::from("size"), &[]).to::<i64>().max(0) as usize
                } else {
                    0
                }
            };
            let args_len = size_of(m.get_or_nil(&"args".to_variant()));
            let defaults_len = size_of(m.get_or_nil(&"default_args".to_variant()));
            return Some(args_len.saturating_sub(defaults_len));
        }
    }
    None
}

/// 信号的参数个数；信号不存在时返回 0
fn signal_arg_count(node: &Gd<Control>, signal: &str) -> usize {
    let list = node.get_signal_list();
    for i in 0..list.len() {
        let Some(s) = list.get(i) else { continue; };
        let name = s.get_or_nil(&"name".to_variant()).to_string();
        if name == signal {
            let args_v = s.get_or_nil(&"args".to_variant());
            if args_v.get_type() == VariantType::ARRAY {
                return args_v.call(&StringName::from("size"), &[]).to::<i64>().max(0) as usize;
            }
            return 0;
        }
    }
    0
}

/// 更新槽位填充效果
pub fn update_slot_fill(target: &mut Gd<Control>, fill_color: Color, mode: i32) {
    let internal_children = target.get_child_count(); // 不含 internal
    if internal_children == 0 {
        return;
    }

    // 检查是否已有填充节点
    let has_fill = if let Some(first_child) = target.get_child(0) {
        first_child.get_meta(&StringName::from("list_slot_fill")).booleanize()
    } else {
        false
    };

    if !has_fill {
        let fill_node = match mode {
            1 => create_square_fill_node(fill_color),
            2 => create_circle_fill_node(fill_color),
            _ => return,
        };
        let mut fill_node = fill_node;
        fill_node.set_meta(&StringName::from("list_slot_fill"), &true.to_variant());
        target.add_child(&fill_node);
        target.move_child(&fill_node, 0);
    } else {
        // 更新已有填充节点的颜色
        if let Some(first_child) = target.get_child(0) {
            if let Ok(fill_ctrl) = first_child.try_cast::<Control>() {
                if fill_ctrl.get_child_count() > 0 {
                    if let Some(color_rect_child) = fill_ctrl.get_child(0) {
                        if let Ok(col) = color_rect_child.try_cast::<ColorRect>() {
                            if let Some(mat) = col.get_material() {
                                if let Ok(mut shader_mat) = mat.try_cast::<ShaderMaterial>() {
                                    shader_mat.set_shader_parameter(
                                        &StringName::from("interior_color"),
                                        &fill_color.to_variant(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 收集节点脚本声明的 @export 变量名（数据注入契约）。
/// 判定依据（Godot 4.6 实测）：@export var 的 usage =
/// SCRIPT_VARIABLE(4096) | STORAGE(4) | EDITOR(2)；
/// 普通脚本变量只有 SCRIPT_VARIABLE(4096)——用 EDITOR 位区分"声明为可注入"。
/// 内建属性（size/texture 等）无 SCRIPT_VARIABLE 位，不会被误收集。
pub fn collect_export_var_names(node: &Gd<Control>) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let properties = node.get_property_list();
    for i in 0..properties.len() {
        if let Some(prop) = properties.get(i) {
            let usage = prop
                .get_or_nil(&"usage".to_variant())
                .try_to::<i64>()
                .unwrap_or(0);
            if (usage & 2) != 0 && (usage & 4096) != 0 {
                let name = prop.get_or_nil(&"name".to_variant()).to_string();
                names.push(name);
            }
        }
    }
    names
}

/// 获取节点的导出变量（以 ui_ 开头的属性）
pub fn get_default_exported_variables(target_node: &Gd<Control>) -> Dictionary<Variant, Variant> {
    let mut result = Dictionary::new();
    let properties = target_node.get_property_list();

    for i in 0..properties.len() {
        if let Some(prop) = properties.get(i) {
            if let Some(usage_var) = prop.get(&"usage".to_variant()) {
                let usage: i32 = usage_var.try_to::<i32>().unwrap_or(0);
                // Godot 4.6 实测：PROPERTY_USAGE_SCRIPT_VARIABLE = 4096，
                // PROPERTY_USAGE_EDITOR = 2（@export var usage = 4102 = 4096|4|2；
                // 旧常量 4/32 在 4.6 下永远匹配不到 @export 变量，过滤实际失效）
                if (usage & 4096) != 0 && (usage & 2) != 0 {
                    if let Some(name_var) = prop.get(&"name".to_variant()) {
                        let name = name_var.to_string();
                        if name.starts_with("ui_") {
                            let value = target_node.get(&StringName::from(name.as_str()));
                            result.set(&Variant::from(name.as_str()), &value);
                        }
                    }
                }
            }
        }
    }

    result
}

/// 递归解析模板绑定
/// 遍历节点及其子节点，查找 __tpl_keys 和 __tpl_attr 元数据，
/// 将数据字典中对应的值设置到节点的属性上
fn resolve_template_bindings_recursive(
    node: &mut Gd<Control>,
    simple_keys: &[(String, Variant)],
    used_keys: &mut Vec<String>,
) {
    let node_name = node.get_name().to_string();
    // 检查当前节点的模板绑定
    if node.has_meta(&StringName::from("__tpl_keys")) {
        let tpl_keys_var = node.get_meta(&StringName::from("__tpl_keys"));
        if tpl_keys_var.get_type() == godot::builtin::VariantType::STRING {
            let keys_str = tpl_keys_var.to_string();
            // //godot_print!("[ListHelper] resolve_template: node='{}' has __tpl_keys='{}', simple_keys={:?}", node_name, keys_str, simple_keys.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>());
            for attr_name in keys_str.split(',') {
                let attr_name = attr_name.trim();
                if attr_name.is_empty() {
                    continue;
                }
                let tpl_meta_key = format!("__tpl_{}", attr_name);
                if !node.has_meta(&StringName::from(tpl_meta_key.as_str())) {
                    godot_warn!("[ListHelper] resolve_template: node='{}' missing meta '{}'", node_name, tpl_meta_key);
                    continue;
                }
                let data_key_var = node.get_meta(&StringName::from(tpl_meta_key.as_str()));
                if data_key_var.get_type() == godot::builtin::VariantType::STRING {
                    let data_key = data_key_var.to_string();
                    // 在 simple_keys 中查找对应的值
                    for (key, val) in simple_keys {
                        if key == &data_key {
                            //godot_print!("[ListHelper] resolve_template: node='{}' set {} = {}", node_name, attr_name, val);
                            node.set(&StringName::from(attr_name), val);
                            if !used_keys.contains(&data_key) {
                                used_keys.push(data_key.clone());
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    // 递归处理子节点
    let child_count = node.get_child_count();
    for i in 0..child_count {
        if let Some(child) = node.get_child(i) {
            if let Ok(mut child_ctrl) = child.try_cast::<Control>() {
                resolve_template_bindings_recursive(&mut child_ctrl, simple_keys, used_keys);
            }
        }
    }
}
