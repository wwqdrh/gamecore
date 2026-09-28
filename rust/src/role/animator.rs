// GdRoleAnimator - 角色动画/贴图管理器
// 挂在角色（GdRoleMover）的子节点上，自动读取移动器的移动状态与朝向并驱动精灵：
//   - AnimatedSprite2D：按命名约定 "{状态}_{朝向}" 自动组合动画名（如 idle_down / walk_left）
//     找不到具体朝向动画时逐级回退：{state}_{facing} -> {state}（可翻转） -> default
//   - Sprite2D：通过 set_state_texture 注册状态贴图表（状态+朝向 -> 贴图），自动切换与翻转
//   - 未找到精灵节点时自动在父节点下创建 AnimatedSprite2D
//
// 信号：
//   s_anim_changed(name)  当前动画/贴图标识变化

use godot::prelude::*;
use godot::builtin::{GString, StringName, VarDictionary};
use godot::classes::{AnimatedSprite2D, INode, Node, Sprite2D, Texture2D};

use super::movement::GdRoleMover;

/// 精灵类型：未解析
const SPRITE_NONE: i64 = 0;
/// 精灵类型：帧动画 AnimatedSprite2D
const SPRITE_ANIMATED: i64 = 1;
/// 精灵类型：静态贴图 Sprite2D
const SPRITE_SIMPLE: i64 = 2;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdRoleAnimator {
    /// 是否启用自动动画驱动
    #[export]
    enable: bool,

    /// 精灵节点路径（留空则自动在父节点下查找 AnimatedSprite2D / Sprite2D）
    #[export]
    sprite_path: NodePath,

    /// 待机动画基础名（与朝向组合成 idle_down 等）
    #[export]
    idle_anim: GString,

    /// 移动动画基础名（与朝向组合成 walk_left 等）
    #[export]
    walk_anim: GString,

    /// 是否按 "{状态}_{朝向}" 自动组合动画名；关闭后直接使用 idle_anim/walk_anim
    #[export]
    auto_name: bool,

    /// 横向模式下用翻转（flip_h）代替 _left/_right 后缀
    #[export]
    use_flip: bool,

    /// Sprite2D 模式的兜底贴图（未注册状态贴图时使用）
    #[export]
    #[var(get = get_fallback_tex, set = set_fallback_tex)]
    fallback_tex: Option<Gd<Texture2D>>,

    // ---- 运行时状态 ----
    sprite_kind: i64,
    sprite_animated: Option<Gd<AnimatedSprite2D>>,
    sprite_simple: Option<Gd<Sprite2D>>,
    mover: Option<Gd<GdRoleMover>>,
    /// 状态贴图表（"{state}|{facing}" -> Texture2D，facing 为空表示任意朝向）
    tex_table: VarDictionary,
    current_anim: GString,
    current_tex: Option<Gd<Texture2D>>,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdRoleAnimator {
    fn init(base: Base<Node>) -> Self {
        Self {
            enable: true,
            sprite_path: NodePath::default(),
            idle_anim: GString::from("idle"),
            walk_anim: GString::from("walk"),
            auto_name: true,
            use_flip: true,
            fallback_tex: None,
            sprite_kind: SPRITE_NONE,
            sprite_animated: None,
            sprite_simple: None,
            mover: None,
            tex_table: VarDictionary::new(),
            current_anim: GString::new(),
            current_tex: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.resolve_nodes();
    }

    fn process(&mut self, _delta: f64) {
        if !self.enable {
            return;
        }

        // 读取移动器状态
        let (moving, facing) = if let Some(ref mover) = self.mover {
            if !mover.is_instance_valid() {
                (false, String::new())
            } else {
                let m = mover.bind();
                (m.is_moving(), m.get_facing().to_string())
            }
        } else {
            (false, String::new())
        };

        let state = if moving {
            self.walk_anim.to_string()
        } else {
            self.idle_anim.to_string()
        };

        match self.sprite_kind {
            SPRITE_ANIMATED => self.update_animated(&state, &facing),
            SPRITE_SIMPLE => self.update_simple(&state, &facing),
            _ => {}
        }
    }
}

#[godot_api]
impl GdRoleAnimator {
    /// 动画/贴图变化信号
    #[signal]
    fn s_anim_changed(name: GString);

    #[func]
    fn get_fallback_tex(&self) -> Option<Gd<Texture2D>> {
        self.fallback_tex.clone()
    }

    #[func]
    fn set_fallback_tex(&mut self, tex: Option<Gd<Texture2D>>) {
        self.fallback_tex = tex;
    }

    /// 当前动画/贴图标识
    #[func]
    fn get_current_anim(&self) -> GString {
        self.current_anim.clone()
    }

    /// 注册状态贴图（Sprite2D 模式）。facing 传空字符串表示任意朝向
    #[func]
    fn set_state_texture(&mut self, state: GString, facing: GString, tex: Option<Gd<Texture2D>>) {
        let key = GString::from(format!("{}|{}", state, facing).as_str());
        if let Some(t) = tex {
            self.tex_table.set(&key.to_variant(), &t.to_variant());
        } else {
            self.tex_table.erase(&key.to_variant());
        }
    }

    /// 清除状态贴图
    #[func]
    fn clear_state_texture(&mut self, state: GString, facing: GString) {
        let key = GString::from(format!("{}|{}", state, facing).as_str());
        self.tex_table.erase(&key.to_variant());
    }

    /// 手动播放指定动画（脱离自动状态机）
    #[func]
    pub fn play_anim(&mut self, name: GString) {
        if let Some(ref anim) = self.sprite_animated {
            if anim.is_instance_valid() {
                anim.clone()
                    .play_ex()
                    .name(&StringName::from(&name))
                    .done();
                self.current_anim = name.clone();
                let var = self.current_anim.to_variant();
                self.base_mut().emit_signal("s_anim_changed", &[var]);
            }
        }
    }

    /// 重新解析精灵与移动器节点（运行时改结构后调用）
    #[func]
    fn resolve(&mut self) {
        self.sprite_kind = SPRITE_NONE;
        self.sprite_animated = None;
        self.sprite_simple = None;
        self.mover = None;
        self.resolve_nodes();
    }

    // ---- 内部实现 ----

    /// 解析移动器（父节点）与精灵节点，找不到精灵时自动创建
    fn resolve_nodes(&mut self) {
        // 移动器：默认父节点
        if let Some(parent) = self.base().get_parent() {
            self.mover = parent.try_cast::<GdRoleMover>().ok();
        }

        // 显式路径优先
        if !self.sprite_path.is_empty() {
            if let Some(node) = self.base().get_node_or_null(&self.sprite_path) {
                self.attach_sprite(node);
                return;
            }
        }

        // 在父节点子级中查找：AnimatedSprite2D 优先，其次 Sprite2D
        if let Some(parent) = self.base().get_parent() {
            let mut animated: Option<Gd<Node>> = None;
            let mut simple: Option<Gd<Node>> = None;
            for child in parent.get_children().iter_shared() {
                let cls = child.get_class().to_string();
                if cls == "AnimatedSprite2D" && animated.is_none() {
                    animated = Some(child);
                } else if cls == "Sprite2D" && simple.is_none() {
                    simple = Some(child);
                }
            }
            if let Some(n) = animated {
                self.attach_sprite(n);
                return;
            }
            if let Some(n) = simple {
                self.attach_sprite(n);
                return;
            }

            // 自动创建 AnimatedSprite2D（ready 期间父节点正在装配子节点，
            // 直接 add_child 会被拒绝，必须走 deferred）
            let mut spr = AnimatedSprite2D::new_alloc();
            spr.set_name(&StringName::from("AnimatedSprite2D"));
            let mut parent = parent;
            parent.call_deferred(&StringName::from("add_child"), &[spr.to_variant()]);
            self.sprite_animated = Some(spr);
            self.sprite_kind = SPRITE_ANIMATED;
        }
    }

    /// 附加到已有精灵节点
    fn attach_sprite(&mut self, node: Gd<Node>) {
        if let Ok(anim) = node.clone().try_cast::<AnimatedSprite2D>() {
            self.sprite_animated = Some(anim);
            self.sprite_kind = SPRITE_ANIMATED;
        } else if let Ok(sp) = node.try_cast::<Sprite2D>() {
            self.sprite_simple = Some(sp);
            self.sprite_kind = SPRITE_SIMPLE;
        }
    }

    /// 驱动 AnimatedSprite2D：命名约定 + 回退链
    fn update_animated(&mut self, state: &str, facing: &str) {
        let Some(ref anim) = self.sprite_animated else {
            return;
        };
        if !anim.is_instance_valid() {
            return;
        }
        let mut anim = anim.clone();
        let Some(frames) = anim.get_sprite_frames() else {
            return;
        };
        let frames = frames;

        let mut name = String::new();
        let mut flip = false;

        if self.auto_name && !facing.is_empty() {
            let specific = format!("{}_{}", state, facing);
            if frames.has_animation(&StringName::from(specific.as_str())) {
                name = specific;
            } else if self.use_flip && (facing == "left" || facing == "right") {
                // 翻转方案：复用 _right 动画或无后缀动画
                let right = format!("{}_right", state);
                if frames.has_animation(&StringName::from(right.as_str())) {
                    name = right;
                    flip = facing == "left";
                } else if frames.has_animation(&StringName::from(state)) {
                    name = state.to_string();
                    flip = facing == "left";
                }
            } else if frames.has_animation(&StringName::from(state)) {
                name = state.to_string();
            }
        } else {
            name = state.to_string();
        }

        // 兜底 default 动画
        if name.is_empty() && frames.has_animation(&StringName::from("default")) {
            name = "default".to_string();
        }
        if name.is_empty() {
            return;
        }

        anim.set_flip_h(flip);
        let sn = StringName::from(name.as_str());
        if anim.get_animation() != sn || !anim.is_playing() {
            anim.play_ex().name(&sn).done();
            self.current_anim = GString::from(name.as_str());
            let var = self.current_anim.to_variant();
            self.base_mut().emit_signal("s_anim_changed", &[var]);
        }
    }

    /// 驱动 Sprite2D：状态贴图表查找 + 翻转
    fn update_simple(&mut self, state: &str, facing: &str) {
        let key_specific = format!("{}|{}", state, facing);
        let key_any = format!("{}|", state);

        // 查找顺序：(state, facing) -> (state, 任意朝向) -> fallback_tex
        let mut tex: Option<Gd<Texture2D>> = None;
        for key in [key_specific.clone(), key_any] {
            let var = self.tex_table.get_or_nil(&key.to_variant());
            if !var.is_nil() {
                if let Ok(t) = var.try_to::<Gd<Texture2D>>() {
                    tex = Some(t);
                }
                break;
            }
        }
        if tex.is_none() {
            tex = self.fallback_tex.clone();
        }

        let flip = self.use_flip && facing == "left";
        if let Some(ref sp) = self.sprite_simple {
            if sp.is_instance_valid() {
                let mut sp = sp.clone();
                if sp.get_texture() != tex {
                    sp.call("set_texture", &[tex.to_variant()]);
                }
                sp.set_flip_h(flip);
            }
        }

        if self.current_tex != tex {
            self.current_tex = tex;
            self.current_anim = GString::from(key_specific.as_str());
            let var = self.current_anim.to_variant();
            self.base_mut().emit_signal("s_anim_changed", &[var]);
        }
    }
}
