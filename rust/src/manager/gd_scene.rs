// GdScene - 场景页面节点
// 继承 Control，提供状态管理、生命周期回调
// 生命周期：on_enter(data) → on_ready() → on_exit()
// 当没有 GdSceneRoot 时自动创建默认实例避免报错

use godot::prelude::*;
use godot::builtin::{GString, VarDictionary, VarArray, Variant, StringName};
use godot::classes::{Control, IControl, Node};

use super::gd_scene_root::GdSceneRoot;

#[derive(GodotClass)]
#[class(base = Control)]
pub struct GdScene {
    /// 场景别名标识
    pub(crate) scene_id: GString,

    /// 当前状态名
    #[export]
    pub(crate) current_state: GString,

    /// 场景初始化数据
    #[export]
    pub(crate) init_data: VarDictionary,

    /// 指向 GdSceneRoot 的引用
    manager: Option<Gd<GdSceneRoot>>,

    /// 状态栈
    state_stack: VarArray,

    /// 状态初始化数据映射
    state_init_data: VarDictionary,

    base: Base<Control>,
}

#[godot_api]
impl IControl for GdScene {
    fn init(base: Base<Control>) -> Self {
        Self {
            scene_id: GString::new(),
            current_state: GString::new(),
            init_data: VarDictionary::new(),
            manager: None,
            state_stack: VarArray::new(),
            state_init_data: VarDictionary::new(),
            base,
        }
    }

    fn ready(&mut self) {
        // 设置鼠标过滤器为穿透
        self.base_mut().set_mouse_filter(godot::classes::control::MouseFilter::PASS);

        // 检查是否有 GdSceneRoot 管理此场景
        let is_managed = self.base().has_meta("__managed");

        if !is_managed {
            // 独立模式：没有 GdSceneRoot 管理
            // 查找或创建默认 GdSceneRoot
            self.ensure_manager();

            // 独立场景直接调用 on_enter 和 on_ready
            self.call_virtual("on_enter", &[self.init_data.to_variant()]);
            if !self.current_state.is_empty() {
                let state = self.current_state.clone();
                let data = VarDictionary::new();
                self.change_state(state, data);
            }
            self.call_virtual("on_ready", &[]);
        } else {
            // 受 GdSceneRoot 管理
            // on_enter 在此调用，on_ready 由 GdSceneRoot 在转场完成后调用
            self.call_virtual("on_enter", &[self.init_data.to_variant()]);
            if !self.current_state.is_empty() {
                let state = self.current_state.clone();
                let data = VarDictionary::new();
                self.change_state(state, data);
            }
        }
    }
}

#[godot_api]
impl GdScene {
    /// 状态变更信号
    #[signal]
    fn s_state_changed(state: GString, data: VarDictionary);

    /// 切换状态，将新状态压入栈
    #[func]
    fn change_state(&mut self, name: GString, data: VarDictionary) {
        self.state_stack.push(&name.to_variant());
        self.state_init_data.set(&name.to_variant(), &data.to_variant());
        self.current_state = name.clone();

        // 发射信号
        self.base_mut().emit_signal(
            "s_state_changed",
            &[name.to_variant(), data.to_variant()],
        );

        // 调用 GDScript 的 on_state_change 虚函数
        self.call_virtual("on_state_change", &[name.to_variant(), data.to_variant()]);
    }

    /// 回退到上一个状态
    #[func]
    fn back_state(&mut self) {
        if self.state_stack.len() <= 1 {
            godot_warn!("GdScene: state stack has only one item, can't back");
            return;
        }
        // 弹出当前状态
        self.state_stack.pop();
        // 弹出目标状态（会被 change_state 重新压入）
        if let Some(name_var) = self.state_stack.pop() {
            let name = name_var.to::<GString>();
            let data_var = self.state_init_data.get_or_nil(&name_var);
            let data = if data_var.is_nil() {
                VarDictionary::new()
            } else {
                data_var.to::<VarDictionary>()
            };
            self.change_state(name, data);
        }
    }

    /// 由 GdSceneRoot 在转场完成后调用，触发 on_ready
    #[func]
    fn root_call_ready(&mut self) {
        self.call_virtual("on_ready", &[]);
    }

    /// 获取管理器引用
    #[func]
    fn get_manager(&self) -> Variant {
        match &self.manager {
            Some(m) => m.to_variant(),
            None => Variant::nil(),
        }
    }

    /// 获取状态栈
    #[func]
    fn get_state_stack(&self) -> VarArray {
        self.state_stack.clone()
    }

    /// 获取状态初始化数据
    #[func]
    fn get_state_init_data(&self) -> VarDictionary {
        self.state_init_data.clone()
    }

    /// 切换场景（委托给 GdSceneRoot）
    /// alias: 目标场景别名
    /// data: 传递给新场景的初始化数据
    #[func]
    fn change_scene(&mut self, alias: GString, data: VarDictionary) -> bool {
        if let Some(ref manager) = self.manager {
            let mut mgr = manager.clone();
            let result = mgr.call(
                &StringName::from("change_scene"),
                &[alias.to_variant(), data.to_variant(), GString::new().to_variant(), true.to_variant(), false.to_variant()],
            );
            return result.to::<bool>();
        }
        godot_warn!("GdScene: no manager, can't change_scene");
        false
    }

    /// 切换场景（带初始状态）
    #[func]
    fn change_scene_with_state(&mut self, alias: GString, data: VarDictionary, init_state: GString) -> bool {
        if let Some(ref manager) = self.manager {
            let mut mgr = manager.clone();
            let result = mgr.call(
                &StringName::from("change_scene"),
                &[alias.to_variant(), data.to_variant(), init_state.to_variant(), true.to_variant(), false.to_variant()],
            );
            return result.to::<bool>();
        }
        godot_warn!("GdScene: no manager, can't change_scene");
        false
    }

    /// 切换场景（无转场动画）
    #[func]
    fn change_scene_no_anim(&mut self, alias: GString, data: VarDictionary) -> bool {
        if let Some(ref manager) = self.manager {
            let mut mgr = manager.clone();
            let result = mgr.call(
                &StringName::from("change_scene"),
                &[alias.to_variant(), data.to_variant(), GString::new().to_variant(), false.to_variant(), false.to_variant()],
            );
            return result.to::<bool>();
        }
        godot_warn!("GdScene: no manager, can't change_scene");
        false
    }

    /// 回退到上一个场景（委托给 GdSceneRoot）
    #[func]
    fn back_scene(&mut self) -> bool {
        if let Some(ref manager) = self.manager {
            let mut mgr = manager.clone();
            let result = mgr.call(&StringName::from("back_scene"), &[]);
            return result.to::<bool>();
        }
        godot_warn!("GdScene: no manager, can't back_scene");
        false
    }

    /// 重启当前场景（委托给 GdSceneRoot）
    #[func]
    fn restart_scene(&mut self, ext_data: VarDictionary) -> bool {
        if let Some(ref manager) = self.manager {
            let mut mgr = manager.clone();
            let result = mgr.call(
                &StringName::from("restart_scene"),
                &[true.to_variant(), ext_data.to_variant()],
            );
            return result.to::<bool>();
        }
        godot_warn!("GdScene: no manager, can't restart_scene");
        false
    }
}

impl GdScene {
    /// 调用 GDScript 虚函数（如果 GDScript 有定义则调用，否则跳过）
    fn call_virtual(&self, method: &str, args: &[Variant]) {
        let mut gd = self.base().clone();
        let method_name = StringName::from(method);
        if gd.has_method(&method_name) {
            gd.call(&method_name, args);
        }
    }

    /// 确保有 GdSceneRoot 管理器，没有则自动创建默认实例
    fn ensure_manager(&mut self) {
        // 搜索场景树中是否已有 GdSceneRoot
        if let Some(tree) = self.base().get_tree_or_null() {
            if let Some(root) = tree.get_root() {
                let root_node = root.upcast::<Node>();
                if let Some(manager) = Self::find_scene_root(&root_node) {
                    self.manager = Some(manager);
                    return;
                }
            }
        }

        // 没有找到 GdSceneRoot，创建默认实例
        godot_print!("GdScene: No GdSceneRoot found, creating default instance");

        let mut default_root = Gd::<GdSceneRoot>::from_init_fn(|base| GdSceneRoot::init(base));

        // 延迟添加到场景树（避免在 _ready 中 add_child 报错）
        if let Some(tree) = self.base().get_tree_or_null() {
            if let Some(mut root) = tree.get_root() {
                root.call_deferred(
                    &StringName::from("add_child"),
                    &[default_root.to_variant()],
                );
            }
        }

        // 将当前场景注册为 GdSceneRoot 的当前场景
        let scene_gd = self.base().clone().upcast::<Node>();
        default_root.bind_mut().set_current_scene_direct(scene_gd);

        self.manager = Some(default_root);
    }

    /// 递归搜索场景树中的 GdSceneRoot 节点
    fn find_scene_root(node: &Gd<Node>) -> Option<Gd<GdSceneRoot>> {
        // 检查当前节点是否是 GdSceneRoot
        if node.get_class() == GString::from("GdSceneRoot") {
            let instance_id = node.instance_id();
            if let Ok(root) = Gd::<GdSceneRoot>::try_from_instance_id(instance_id) {
                return Some(root);
            }
        }

        // 递归检查子节点
        let children = node.get_children();
        for child in children.iter_shared() {
            if let Some(root) = Self::find_scene_root(&child) {
                return Some(root);
            }
        }

        None
    }

    /// 设置管理器（由 GdSceneRoot 调用）
    pub(crate) fn set_manager_internal(&mut self, manager: Gd<GdSceneRoot>) {
        self.manager = Some(manager);
    }
}
