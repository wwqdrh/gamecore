// GdSceneRoot - 场景管理器
// 继承 Node，负责场景注册、切换、转场动画
// 转场动画使用 ColorRect 遮罩 + Tween 淡入淡出
// 通过 _process 驱动转场状态机

use godot::prelude::*;
use godot::builtin::{GString, VarDictionary, VarArray, Variant, StringName, Color, NodePath};
use godot::classes::{
    Node, INode, Control, CanvasLayer, ColorRect, PackedScene, Engine, Tween,
};
use godot::classes::control::LayoutPreset;

use super::gd_scene::GdScene;

/// 转场动画步骤
#[derive(Default, PartialEq, Clone, Copy)]
enum TransitionStep {
    #[default]
    Idle,
    FadingOut,
    FadingIn,
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdSceneRoot {
    /// 已注册的场景映射 (alias -> PackedScene)
    scenes_map: VarDictionary,

    /// 当前活跃场景
    current_scene: Option<Gd<GdScene>>,

    /// 场景容器节点
    scene_layer: Option<Gd<Control>>,

    /// 转场遮罩
    scene_mask: Option<Gd<ColorRect>>,

    /// 遮罩所在 CanvasLayer
    canvas_layer: Option<Gd<CanvasLayer>>,

    /// 是否正在切换场景（防重入）
    is_changing: bool,

    /// 转场动画步骤
    transition_step: TransitionStep,

    /// 等待加载的新场景
    pending_scene: Option<Gd<GdScene>>,

    /// 旧场景引用（用于清理）
    old_scene: Option<Gd<GdScene>>,

    /// 转场动画 Tween
    transition_tween: Option<Gd<Tween>>,

    /// 场景切换历史栈（存储场景别名）
    scene_change_stack: VarArray,

    /// 场景初始化数据映射 (alias -> Dictionary)
    scene_init_data_map: VarDictionary,

    /// 转场动画时长（秒）
    #[var]
    trans_duration: f64,

    /// 管理器 ID（注册到 GDCore）
    #[var]
    manager_id: GString,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdSceneRoot {
    fn init(base: Base<Node>) -> Self {
        Self {
            scenes_map: VarDictionary::new(),
            current_scene: None,
            scene_layer: None,
            scene_mask: None,
            canvas_layer: None,
            is_changing: false,
            transition_step: TransitionStep::Idle,
            pending_scene: None,
            old_scene: None,
            transition_tween: None,
            scene_change_stack: VarArray::new(),
            scene_init_data_map: VarDictionary::new(),
            trans_duration: 0.5,
            manager_id: GString::from("default"),
            base,
        }
    }

    fn ready(&mut self) {
        // 设置为始终处理，确保暂停场景树时 _process 仍能驱动转场状态机
        self.base_mut().set_process_mode(godot::classes::node::ProcessMode::ALWAYS);

        // 创建场景容器
        let mut scene_layer = Control::new_alloc();
        scene_layer.set_name("SceneLayer");
        scene_layer.set_mouse_filter(godot::classes::control::MouseFilter::IGNORE);
        scene_layer.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
        self.base_mut().add_child(&scene_layer);
        self.scene_layer = Some(scene_layer);

        // 创建转场遮罩
        let mut canvas_layer = CanvasLayer::new_alloc();
        canvas_layer.set_layer(9);
        canvas_layer.set_process_mode(godot::classes::node::ProcessMode::ALWAYS);
        self.base_mut().add_child(&canvas_layer);
        self.canvas_layer = Some(canvas_layer);

        let mut scene_mask = ColorRect::new_alloc();
        scene_mask.set_name("SceneMask");
        scene_mask.set_mouse_filter(godot::classes::control::MouseFilter::IGNORE);
        scene_mask.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
        scene_mask.set_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        scene_mask.set_process_mode(godot::classes::node::ProcessMode::ALWAYS);
        scene_mask.hide();

        if let Some(ref canvas) = self.canvas_layer {
            let mut canvas = canvas.clone();
            canvas.add_child(&scene_mask);
        }
        self.scene_mask = Some(scene_mask);

        // 注册到 GDCore
        self.register_to_gdcore();

        // 从 GdConfigManager 加载场景配置
        self.load_scenes_from_config();

        // 检查是否有 SCENES 属性自动注册
        let base = self.base().clone();
        let scenes_var = base.get("SCENES");
        if scenes_var.get_type() == godot::builtin::VariantType::DICTIONARY {
            let scenes = scenes_var.to::<VarDictionary>();
            self.register_scenes(scenes);
        }
    }

    fn process(&mut self, _delta: f64) {
        if self.transition_step == TransitionStep::Idle {
            return;
        }

        // 检查 tween 是否完成
        let tween_finished = match &self.transition_tween {
            Some(tween) => !tween.is_valid() || !tween.is_running(),
            None => true,
        };

        if !tween_finished {
            return;
        }

        match self.transition_step {
            TransitionStep::FadingOut => {
                // 淡出完成（遮罩已变为不透明），执行场景替换
                self.base_mut().emit_signal("s_trans_closed", &[]);

                // 执行场景替换
                self.do_swap_scene();

                // 开始淡入
                self.start_fade_in();
                self.transition_step = TransitionStep::FadingIn;
            }
            TransitionStep::FadingIn => {
                // 淡入完成（遮罩已变为透明），清理
                self.finish_transition();
                self.transition_step = TransitionStep::Idle;
            }
            TransitionStep::Idle => {}
        }
    }
}

#[godot_api]
impl GdSceneRoot {
    /// 转场关闭动画完成信号
    #[signal]
    fn s_trans_closed();

    /// 转场打开动画完成信号
    #[signal]
    fn s_trans_opened();

    /// 注册单个场景
    #[func]
    fn register_scene(&mut self, alias: GString, scene: Gd<PackedScene>) {
        let key = alias.to_variant();
        if self.scenes_map.contains_key(&key) {
            godot_warn!("GdSceneRoot: scene alias '{}' already exists", alias);
            return;
        }
        self.scenes_map.set(&key, &scene.to_variant());
    }

    /// 批量注册场景
    #[func]
    fn register_scenes(&mut self, data: VarDictionary) {
        let keys = data.keys_array();
        for i in 0..keys.len() {
            let key = keys.at(i);
            let alias = key.to::<GString>();

            if self.scenes_map.contains_key(&key) {
                godot_warn!("GdSceneRoot: scene alias '{}' already exists", alias);
                continue;
            }

            let value = data.get_or_nil(&key);
            if value.get_type() == godot::builtin::VariantType::OBJECT {
                self.scenes_map.set(&key, &value);
            }
        }
    }

    /// 切换场景
    /// alias: 场景别名
    /// data: 传递给新场景的初始化数据
    /// init_state: 初始状态名（可选）
    /// with_anim: 是否播放转场动画（默认 true）
    /// force: 是否强制切换，忽略防重入（默认 false）
    #[func]
    fn change_scene(
        &mut self,
        alias: GString,
        data: VarDictionary,
        init_state: GString,
        with_anim: bool,
        force: bool,
    ) -> bool {
        if !force && self.is_changing {
            return false;
        }
        self.is_changing = true;

        // 强制取消暂停
        if let Some(mut tree) = self.base().get_tree_or_null() {
            tree.set_pause(false);
        }

        // 实例化新场景
        let mut new_scene = match self.initial_scene_check(&alias, &data) {
            Some(scene) => scene,
            None => {
                godot_warn!("GdSceneRoot: change_scene failed, alias: {}", alias);
                self.is_changing = false;
                return false;
            }
        };

        // 设置初始状态
        if !init_state.is_empty() {
            new_scene.bind_mut().current_state = init_state;
        }

        // 保存旧场景引用
        self.old_scene = self.current_scene.take();
        self.pending_scene = Some(new_scene);

        if with_anim {
            // 带转场动画
            self.start_fade_out();
        } else {
            // 无动画，直接替换
            self.do_swap_scene();
            self.finish_transition();
        }

        true
    }

    /// 回退到上一个场景
    #[func]
    fn back_scene(&mut self) -> bool {
        if self.scene_change_stack.len() <= 1 {
            godot_warn!("GdSceneRoot: scene stack has only one item, can't back");
            return false;
        }
        // 弹出当前场景名
        self.scene_change_stack.pop();
        // 弹出目标场景名
        if let Some(name_var) = self.scene_change_stack.pop() {
            let scene_name = name_var.to::<GString>();
            let data_var = self.scene_init_data_map.get_or_nil(&name_var);
            let data = if data_var.is_nil() {
                VarDictionary::new()
            } else {
                data_var.to::<VarDictionary>()
            };
            return self.change_scene(scene_name, data, GString::new(), true, false);
        }
        false
    }

    /// 重启当前场景
    #[func]
    fn restart_scene(&mut self, with_anim: bool, ext_data: VarDictionary) -> bool {
        if self.scene_change_stack.is_empty() {
            godot_warn!("GdSceneRoot: no scene, can't restart");
            return false;
        }

        // 获取当前场景名（不弹出）
        let last_idx = self.scene_change_stack.len() - 1;
        let name_var = self.scene_change_stack.at(last_idx);
        let scene_name = name_var.to::<GString>();

        // 弹出当前场景名（change_scene 会重新压入）
        self.scene_change_stack.pop();

        // 合并初始化数据
        let data_var = self.scene_init_data_map.get_or_nil(&name_var);
        let mut data = if data_var.is_nil() {
            VarDictionary::new()
        } else {
            data_var.to::<VarDictionary>()
        };
        // 合并额外数据
        let ext_keys = ext_data.keys_array();
        for i in 0..ext_keys.len() {
            let k = ext_keys.at(i);
            data.set(&k, &ext_data.get_or_nil(&k));
        }

        self.change_scene(scene_name, data, GString::new(), with_anim, false)
    }

    /// 获取当前场景
    #[func]
    fn get_current_scene(&self) -> Variant {
        match &self.current_scene {
            Some(scene) => scene.to_variant(),
            None => Variant::nil(),
        }
    }

    /// 获取已注册的场景别名列表
    #[func]
    fn get_registered_scenes(&self) -> VarArray {
        self.scenes_map.keys_array()
    }

    /// 获取场景切换历史栈
    #[func]
    fn get_scene_stack(&self) -> VarArray {
        self.scene_change_stack.clone()
    }

    /// 是否正在切换场景
    #[func]
    fn is_changing_scene(&self) -> bool {
        self.is_changing
    }
}

impl GdSceneRoot {
    /// 实例化场景并设置基本属性
    fn initial_scene_check(&mut self, alias: &GString, params: &VarDictionary) -> Option<Gd<GdScene>> {
        let key = alias.to_variant();
        if !self.scenes_map.contains_key(&key) {
            godot_warn!("GdSceneRoot: scene '{}' not registered", alias);
            return None;
        }

        let scene_var = self.scenes_map.get_or_nil(&key);
        if scene_var.get_type() != godot::builtin::VariantType::OBJECT {
            godot_warn!("GdSceneRoot: scene '{}' is not a valid PackedScene", alias);
            return None;
        }

        let packed_scene = match scene_var.try_to::<Gd<PackedScene>>() {
            Ok(s) => s,
            Err(_) => {
                godot_warn!("GdSceneRoot: scene '{}' is not a PackedScene", alias);
                return None;
            }
        };

        // 实例化场景
        let instance = match packed_scene.instantiate() {
            Some(inst) => inst,
            None => {
                godot_warn!("GdSceneRoot: scene '{}' instantiate failed", alias);
                return None;
            }
        };

        let mut gd_scene: Gd<GdScene> = match instance.try_cast() {
            Ok(s) => s,
            Err(_) => {
                godot_warn!(
                    "GdSceneRoot: scene '{}' root node is not a GdScene",
                    alias
                );
                return None;
            }
        };

        // 设置管理器引用（通过 instance_id 转换 Gd<Node> -> Gd<GdSceneRoot>）
        let manager_id = self.base().instance_id();
        if let Ok(manager) = Gd::<GdSceneRoot>::try_from_instance_id(manager_id) {
            gd_scene.bind_mut().set_manager_internal(manager);
        }

        // 设置场景属性（直接字段赋值，pub(crate) 可访问）
        gd_scene.bind_mut().scene_id = alias.clone();
        gd_scene.bind_mut().init_data = params.clone();

        // 标记为受管理
        gd_scene.set_meta("__managed", &true.to_variant());

        // 调用 on_preload（如果 GDScript 有定义）
        if gd_scene.has_method("on_preload") {
            let result = gd_scene.call("on_preload", &[]);
            if let Ok(check) = result.try_to::<bool>() {
                if !check {
                    godot_warn!("GdSceneRoot: scene '{}' on_preload returned false", alias);
                    return None;
                }
            }
        }

        // 压入场景切换栈
        self.scene_change_stack.push(&alias.to_variant());
        self.scene_init_data_map.set(&alias.to_variant(), &params.to_variant());

        Some(gd_scene)
    }

    /// 开始淡出动画（遮罩从透明变为不透明）
    fn start_fade_out(&mut self) {
        if let Some(ref mask) = self.scene_mask {
            let mut mask = mask.clone();
            mask.show();
            mask.set_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));

            let mut base_gd = self.base().clone();
            let mut tween = base_gd.create_tween();
            tween.set_pause_mode(godot::classes::tween::TweenPauseMode::PROCESS);

            tween.tween_property(
                &mask,
                &NodePath::from("color"),
                &Color::from_rgba(0.0, 0.0, 0.0, 1.0).to_variant(),
                self.trans_duration,
            );

            self.transition_tween = Some(tween);
        }
        self.transition_step = TransitionStep::FadingOut;

        // 暂停游戏树
        if let Some(mut tree) = self.base().get_tree_or_null() {
            tree.set_pause(true);
        }
    }

    /// 开始淡入动画（遮罩从不透明变为透明）
    fn start_fade_in(&mut self) {
        if let Some(ref mask) = self.scene_mask {
            let mut mask = mask.clone();

            let mut base_gd = self.base().clone();
            let mut tween = base_gd.create_tween();
            tween.set_pause_mode(godot::classes::tween::TweenPauseMode::PROCESS);

            tween.tween_property(
                &mask,
                &NodePath::from("color"),
                &Color::from_rgba(0.0, 0.0, 0.0, 0.0).to_variant(),
                self.trans_duration,
            );

            self.transition_tween = Some(tween);
        }

        // 恢复游戏树
        if let Some(mut tree) = self.base().get_tree_or_null() {
            tree.set_pause(false);
        }

        self.base_mut().emit_signal("s_trans_opened", &[]);
    }

    /// 执行场景替换
    fn do_swap_scene(&mut self) {
        // 对旧场景调用 on_exit 并移除
        if let Some(ref old_scene) = self.old_scene {
            let mut old = old_scene.clone();
            if old.has_method("on_exit") {
                old.call("on_exit", &[]);
            }

            // 从父节点移除
            if let Some(mut parent) = old.get_parent() {
                parent.remove_child(&old);
            }
            old.queue_free();
        }
        self.old_scene = None;

        // 将新场景添加到 scene_layer
        if let Some(ref new_scene) = self.pending_scene {
            if let Some(ref scene_layer) = self.scene_layer {
                let mut layer = scene_layer.clone();
                layer.add_child(new_scene);
            }
            self.current_scene = self.pending_scene.take();
        }
    }

    /// 完成转场，清理遮罩和状态
    fn finish_transition(&mut self) {
        // 隐藏遮罩
        if let Some(ref mask) = self.scene_mask {
            let mut mask = mask.clone();
            mask.hide();
        }

        // 调用新场景的 on_ready
        if let Some(ref scene) = self.current_scene {
            let mut scene = scene.clone();
            scene.call("root_call_ready", &[]);
        }

        // 清理状态
        self.transition_tween = None;
        self.is_changing = false;
    }

    /// 直接设置当前场景（不触发转场逻辑，用于 GdScene 自动创建管理器时）
    pub(crate) fn set_current_scene_direct(&mut self, scene: Gd<Node>) {
        // 尝试转换为 GdScene
        if scene.get_class() == GString::from("GdScene") {
            let instance_id = scene.instance_id();
            if let Ok(gd_scene) = Gd::<GdScene>::try_from_instance_id(instance_id) {
                self.current_scene = Some(gd_scene);
            }
        }
    }

    /// 注册到 GDCore 全局单例
    fn register_to_gdcore(&self) {
        let mut engine = Engine::singleton();
        if let Some(gdcore) = engine.get_singleton("GDCORE") {
            let method = StringName::from("add_global_node");
            let mut gdcore = gdcore;
            if gdcore.has_method(&method) {
                let base = self.base().clone();
                gdcore.call(&method, &[self.manager_id.to_variant(), base.to_variant()]);
            }
        }
    }

    /// 从 GdConfigManager 加载场景配置
    fn load_scenes_from_config(&mut self) {
        // 确保 GdConfigManager 已创建（此时场景树已就绪，安全调用 ResourceLoader）
        let mut config_manager = match super::config_manager::ensure_config_manager() {
            Some(cm) => cm,
            None => return,
        };

        // 获取所有场景别名
        let aliases_result = config_manager.call(&StringName::from("get_scene_aliases"), &[]);
        if aliases_result.get_type() != godot::builtin::VariantType::ARRAY {
            return;
        }

        let aliases = aliases_result.to::<VarArray>();
        for i in 0..aliases.len() {
            let alias_var = aliases.at(i);
            let alias = alias_var.to::<GString>();

            // 获取 PackedScene
            let scene_result = config_manager.call(
                &StringName::from("get_scene"),
                &[alias_var.clone()],
            );
            if scene_result.get_type() != godot::builtin::VariantType::OBJECT {
                continue;
            }

            if let Ok(packed_scene) = scene_result.try_to::<Gd<PackedScene>>() {
                self.register_scene(alias, packed_scene);
            }
        }
    }
}
