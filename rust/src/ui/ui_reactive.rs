// GML 响应式条件显隐 —— <script> 变量驱动的 show_if / hide_if
//
// GML 声明（属性存节点 meta，随 tscn 序列化保留，重开场景仍有效）：
//   <Button show_if="has_save" />   变量真值时显示，否则隐藏
//   <Button hide_if="has_save" />   变量真值时隐藏（反向）
//   表达式支持前缀 ! 取反：show_if="!empty"
//
// 运行期响应（GDScript 静态调用，变量变更即时重算本 gml 树全部条件）：
//   GdUIReactive.set_var(self, "has_save", true)   更新变量并重算
//   GdUIReactive.get_var(self, "has_save")         读变量（不存在返回 null）
//   GdUIReactive.apply_conditions(self)            手动重算（直接改过变量表后用）
//
// 真值规则：null/变量不存在=false（show_if 隐藏、hide_if 显示）；bool 取原值；
//   数字≠0 为真；字符串非空且非 "false"/"0" 为真；数组/字典非空为真；对象非空为真。
// 作用域：以就近 __gml_root 为界——嵌套 <Gml> 子文件各自独立变量表，互不串扰，
//   重算外层树时跳过内层子作用域（内层条件由其自身构建期/自身 set_var 驱动）。
// 变量存储：__script_vars meta 字典（根 + 顶层子节点各一份，与 builder
//   apply_data_bindings 同语义）；set_var 同步更新所有副本后全树重算。

use godot::classes::Control;
use godot::prelude::*;

use crate::ui::parser::DataValue;
use crate::ui::ui_manager::nearest_gml_root;

/// 条件表达式 meta 键
const SHOW_IF_META: &str = "__show_if";
const HIDE_IF_META: &str = "__hide_if";

/// 条件表达式求值：`var` / `!var`。resolve 返回 None = 变量不存在，按 false。
pub(crate) fn eval_condition(expr: &str, resolve: &dyn Fn(&str) -> Option<bool>) -> bool {
    let expr = expr.trim();
    if let Some(rest) = expr.strip_prefix('!') {
        return !resolve(rest.trim()).unwrap_or(false);
    }
    resolve(expr).unwrap_or(false)
}

/// 构建期真值：DataValue（<script> 块 / data-xxx 注入变量）
pub(crate) fn data_value_truthy(v: &DataValue) -> bool {
    match v {
        DataValue::Null => false,
        DataValue::Bool(b) => *b,
        DataValue::Num(n) => *n != 0.0,
        DataValue::Str(s) => !s.is_empty() && s != "false" && s != "0",
        DataValue::Array(a) => !a.is_empty(),
        DataValue::Dict(d) => !d.is_empty(),
    }
}

/// 运行期真值：Variant（__script_vars 字典取出的值）
fn variant_truthy(v: &Variant) -> bool {
    match v.get_type() {
        VariantType::NIL => false,
        VariantType::BOOL => v.try_to::<bool>().unwrap_or(false),
        VariantType::INT => v.try_to::<i64>().unwrap_or(0) != 0,
        VariantType::FLOAT => v.try_to::<f64>().unwrap_or(0.0) != 0.0,
        VariantType::STRING | VariantType::STRING_NAME => {
            let s = v.to_string();
            !s.is_empty() && s != "false" && s != "0"
        }
        VariantType::ARRAY => !v.try_to::<Array<Variant>>().unwrap_or_default().is_empty(),
        VariantType::DICTIONARY => !v
            .try_to::<Dictionary<Variant, Variant>>()
            .unwrap_or_default()
            .is_empty(),
        _ => true,
    }
}

/// 从 root 树重算全部条件显隐（node 本身属于本作用域；嵌套 __gml_root 子树跳过）。
/// visible 直接 set_visible 覆盖——容器内隐藏节点自动退出布局，符合直觉。
pub(crate) fn apply_conditions(
    node: &Gd<Node>,
    resolve: &dyn Fn(&str) -> Option<bool>,
) {
    for key in [SHOW_IF_META, HIDE_IF_META] {
        let sn = StringName::from(key);
        if !node.has_meta(&sn) {
            continue;
        }
        let expr = node.get_meta(&sn).to_string();
        let truth = eval_condition(&expr, resolve);
        let visible = if key == SHOW_IF_META { truth } else { !truth };
        if let Ok(mut c) = node.clone().try_cast::<Control>() {
            c.set_visible(visible);
        }
    }
    for i in 0..node.get_child_count() {
        if let Some(child) = node.get_child(i) {
            // 嵌套 gml 子树是独立作用域（自带 __gml_root 与独立变量表），跳过
            if child.has_meta(&StringName::from("__gml_root")) {
                continue;
            }
            apply_conditions(&child, resolve);
        }
    }
}

/// 构建期初始求值（builder 在 apply_data_bindings 末尾调用）：
/// 变量来自 <script> 块与 data-xxx 注入；缺失按 false。
pub(crate) fn apply_conditions_build(root: &Gd<Control>, vars: &std::collections::HashMap<String, DataValue>) {
    let resolve = |name: &str| -> Option<bool> { vars.get(name).map(data_value_truthy) };
    let node: Gd<Node> = root.clone().upcast();
    apply_conditions(&node, &resolve);
}

/// 从作用域根读 __script_vars 字典（无则返回空字典）
fn load_vars(root: &Gd<Node>) -> Dictionary<Variant, Variant> {
    let key = StringName::from("__script_vars");
    if root.has_meta(&key) {
        root.get_meta(&key)
            .try_to::<Dictionary<Variant, Variant>>()
            .unwrap_or_default()
    } else {
        Dictionary::new()
    }
}

/// 变量字典写回根 + 全部顶层子节点（与 builder apply_data_bindings 同语义：
/// 顶层子节点可能被剥壳嫁接/复制，各持一份副本）
fn store_vars(root: &Gd<Node>, dict: &Dictionary<Variant, Variant>) {
    let key = StringName::from("__script_vars");
    let variant = dict.to_variant();
    let mut root_writable = root.clone();
    root_writable.set_meta(&key, &variant);
    let children = root.get_children();
    for i in 0..children.len() {
        if let Some(child) = children.get(i) {
            if let Ok(mut c) = child.try_cast::<Control>() {
                c.set_meta(&key, &variant);
            }
        }
    }
}

/// GML 响应式变量与条件显隐入口（GDScript 静态调用）
#[derive(GodotClass)]
#[class(base = Object, init)]
pub struct GdUIReactive {
    base: Base<Object>,
}

#[godot_api]
impl GdUIReactive {
    /// 设置变量并即时重算 from 所属 gml 树的条件显隐。
    /// from：目标树内任意节点（通常传 self）；变量表在就近 __gml_root 上。
    /// 用法：`GdUIReactive.set_var(self, "slot1_has_save", true)`
    #[func]
    pub fn set_var(from: Gd<Object>, name: GString, value: Variant) {
        let Ok(node) = from.try_cast::<Node>() else {
            godot_error!("[GdUIReactive] set_var: from 不是 Node 类型");
            return;
        };
        let Some(root) = nearest_gml_root(&node) else {
            godot_warn!(
                "[GdUIReactive] set_var: 就近未找到 __gml_root（节点 {}）",
                node.get_name()
            );
            return;
        };
        let mut dict = load_vars(&root);
        dict.set(&Variant::from(name), &value);
        store_vars(&root, &dict);
        let resolve = |var: &str| -> Option<bool> {
            dict.get(&Variant::from(var)).map(|v| variant_truthy(&v))
        };
        apply_conditions(&root, &resolve);
    }

    /// 读取变量（不存在返回 null）
    #[func]
    pub fn get_var(from: Gd<Object>, name: GString) -> Variant {
        let Ok(node) = from.try_cast::<Node>() else {
            return Variant::nil();
        };
        let Some(root) = nearest_gml_root(&node) else {
            return Variant::nil();
        };
        load_vars(&root).get(&Variant::from(name)).unwrap_or_default()
    }

    /// 手动重算 from 所属 gml 树的条件显隐（变量表被直接修改后使用）
    #[func]
    pub fn apply_conditions(from: Gd<Object>) {
        let Ok(node) = from.try_cast::<Node>() else {
            return;
        };
        let Some(root) = nearest_gml_root(&node) else {
            return;
        };
        let dict = load_vars(&root);
        let resolve = |var: &str| -> Option<bool> {
            dict.get(&Variant::from(var)).map(|v| variant_truthy(&v))
        };
        apply_conditions(&root, &resolve);
    }
}
