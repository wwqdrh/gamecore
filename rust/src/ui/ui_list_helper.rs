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

/// 更新容器：根据 data 数组动态创建/删除/更新子节点
pub fn update_container(target: &mut Gd<Control>, slot: &Gd<Control>, count: i32, data: &Array<Variant>) {
    let slot = slot.clone();
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

/// 获取节点的导出变量（以 ui_ 开头的属性）
pub fn get_default_exported_variables(target_node: &Gd<Control>) -> Dictionary<Variant, Variant> {
    let mut result = Dictionary::new();
    let properties = target_node.get_property_list();

    for i in 0..properties.len() {
        if let Some(prop) = properties.get(i) {
            if let Some(usage_var) = prop.get(&"usage".to_variant()) {
                let usage: i32 = usage_var.try_to::<i32>().unwrap_or(0);
                // PROPERTY_USAGE_SCRIPT_VARIABLE = 1 << 2 = 4
                // PROPERTY_USAGE_EDITOR = 1 << 5 = 32
                if (usage & 4) != 0 && (usage & 32) != 0 {
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
