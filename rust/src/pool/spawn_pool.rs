// GdSpawnPool - 对象池单例
// 注册为 Engine singleton "GDSPAWNPOOL"
//
// 原型高频生成物体（子弹、敌人、飘字、粒子）时反复 instantiate/free
// 有性能抖动，本池以"回收复用"替代"销毁重建"：
//   - register(alias, path, prewarm)：预加载场景并预热 N 个实例
//   - spawn(alias, parent, pos)：取闲置实例或实例化新节点
//   - despawn(node)：从树上摘下收回池中（不 free，下次直接复用）
//
// GDScript 用法：
//   var pool = Engine.get_singleton("GDSPAWNPOOL")
//   pool.register("bullet", "res://fx/bullet.tscn", 20)
//   var b = pool.spawn("bullet", null, Vector2(100, 100))
//   pool.despawn(b)

use std::collections::HashMap;

use godot::prelude::*;
use godot::builtin::{GString, StringName, Variant, Vector2};
use godot::classes::{Engine, IObject, Node, Node2D, Object, PackedScene, ResourceLoader, SceneTree};

/// 单个池条目
struct PoolEntry {
    scene: Gd<PackedScene>,
    /// 场景资源路径（归池匹配用）
    scene_path: String,
    /// 闲置节点（未挂树）
    idle: Vec<Gd<Node>>,
    /// 累计实例化数量（观测用）
    created: i64,
}

#[derive(GodotClass)]
#[class(base = Object)]
pub struct GdSpawnPool {
    pools: HashMap<String, PoolEntry>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GdSpawnPool {
    fn init(base: Base<Object>) -> Self {
        Self {
            pools: HashMap::new(),
            base,
        }
    }
}

#[godot_api]
impl GdSpawnPool {
    /// 注册池：加载场景并预热 prewarm 个闲置实例，返回是否成功
    #[func]
    pub fn register(&mut self, alias: GString, path: GString, prewarm: i64) -> bool {
        if alias.is_empty() || path.is_empty() {
            godot_error!("[GdSpawnPool] register: alias/path 不能为空");
            return false;
        }
        let Some(scene) = Self::load_scene(&path) else {
            godot_error!("[GdSpawnPool] register: 场景加载失败 {} ({})", path, alias);
            return false;
        };
        let (idle, created) = Self::prewarm(&scene, prewarm);
        self.pools.insert(
            alias.to_string(),
            PoolEntry {
                scene,
                scene_path: path.to_string(),
                idle,
                created,
            },
        );
        true
    }

    /// 用已有 PackedScene 注册池（无需文件路径）
    #[func]
    pub fn register_scene(&mut self, alias: GString, scene: Gd<PackedScene>, prewarm: i64) -> bool {
        if alias.is_empty() {
            godot_error!("[GdSpawnPool] register_scene: alias 不能为空");
            return false;
        }
        // register_scene 无路径来源：延迟到首次 spawn 时从实例节点捕获
        let scene_path = String::new();
        let (idle, created) = Self::prewarm(&scene, prewarm);
        self.pools.insert(
            alias.to_string(),
            PoolEntry {
                scene,
                scene_path,
                idle,
                created,
            },
        );
        true
    }

    /// 生成节点：优先复用闲置实例；parent 为空时挂到当前场景；
    /// pos 仅对 Node2D 生效。返回 None 表示别名未注册。
    #[func]
    pub fn spawn(
        &mut self,
        alias: GString,
        parent: Option<Gd<Node>>,
        pos: Variant,
    ) -> Option<Gd<Node>> {
        let key = alias.to_string();
        let Some(entry) = self.pools.get_mut(&key) else {
            godot_error!("[GdSpawnPool] spawn: 未注册的池别名 \"{}\"", alias);
            return None;
        };
        // 取闲置或实例化
        let mut node = match entry.idle.pop() {
            Some(n) if n.is_instance_valid() => n,
            _ => {
                entry.created += 1;
                match entry.scene.instantiate() {
                    Some(n) => n,
                    None => {
                        godot_error!("[GdSpawnPool] spawn: 场景实例化失败（{}）", alias);
                        return None;
                    }
                }
            }
        };
        // register_scene 场景：从实例节点捕获资源路径，供 despawn 归池匹配
        if entry.scene_path.is_empty() {
            let path = node.get_scene_file_path().to_string();
            if !path.is_empty() {
                entry.scene_path = path;
            }
        }
        // register_scene 场景：从实例节点捕获资源路径，供 despawn 归池匹配
        if entry.scene_path.is_empty() {
            let path = node.get_scene_file_path().to_string();
            if !path.is_empty() {
                entry.scene_path = path;
            }
        }
        // 挂树
        let mut parent = match parent {
            Some(p) if p.is_instance_valid() => p,
            _ => match Self::current_scene() {
                Some(cs) => cs,
                None => {
                    godot_error!("[GdSpawnPool] spawn: 无可用父节点（当前场景为空）");
                    entry.idle.push(node);
                    return None;
                }
            },
        };
        parent.add_child(&node);
        // 位置（Node2D 才有）
        if let Ok(v2) = pos.try_to::<Vector2>() {
            if let Ok(mut n2d) = node.clone().try_cast::<Node2D>() {
                n2d.set_position(v2);
            }
        }
        node.call("set_visible", &[true.to_variant()]);
        Some(node)
    }

    /// 回收节点：从树上摘下（不 free），归还闲置队列；
    /// 未通过本池 spawn 的节点无法归还（找不到来源池时返回 false）
    #[func]
    pub fn despawn(&mut self, mut node: Gd<Node>) -> bool {
        if !node.is_instance_valid() {
            return false;
        }
        // 找到节点所属池（按场景资源匹配）
        let scene_path = node.get_scene_file_path();
        if scene_path.is_empty() {
            godot_error!("[GdSpawnPool] despawn: 节点无 scene_file_path，无法归池");
            return false;
        }
        let node_path = scene_path.to_string();
        let alias = self
            .pools
            .iter()
            .find(|(_, e)| !e.scene_path.is_empty() && e.scene_path == node_path)
            .map(|(k, _)| k.clone());
        let Some(alias) = alias else {
            godot_error!(
                "[GdSpawnPool] despawn: 场景 {} 未注册任何池",
                scene_path
            );
            return false;
        };
        // 摘下（reparent 到空：先 remove_child）
        if let Some(mut p) = node.get_parent() {
            p.remove_child(&node);
        }
        node.call("set_visible", &[false.to_variant()]);
        self.pools
            .get_mut(&alias)
            .expect("alias 来自池遍历")
            .idle
            .push(node);
        true
    }

    /// 清空某个池的闲置实例（不传别名 = 清空所有池）
    #[func]
    pub fn clear(&mut self, alias: GString) {
        if alias.is_empty() {
            self.pools.clear();
        } else {
            self.pools.remove(&alias.to_string());
        }
    }

    /// 某池闲置节点数
    #[func]
    pub fn idle_count(&self, alias: GString) -> i64 {
        self.pools
            .get(&alias.to_string())
            .map_or(0, |e| e.idle.len() as i64)
    }

    /// 某池累计实例化数量（观测池是否有效减少了实例化）
    #[func]
    pub fn created_count(&self, alias: GString) -> i64 {
        self.pools
            .get(&alias.to_string())
            .map_or(0, |e| e.created)
    }

    /// 已注册的池别名列表
    #[func]
    pub fn aliases(&self) -> PackedStringArray {
        self.pools
            .keys()
            .map(|k| GString::from(k.as_str()))
            .collect()
    }

    // ---- 内部实现 ----

    /// 预热实例化（失败项跳过并告警）
    fn prewarm(scene: &Gd<PackedScene>, count: i64) -> (Vec<Gd<Node>>, i64) {
        let mut idle = Vec::new();
        for _ in 0..count.max(0) {
            if let Some(n) = scene.instantiate() {
                idle.push(n);
            }
        }
        let created = idle.len() as i64;
        (idle, created)
    }

    fn load_scene(path: &GString) -> Option<Gd<PackedScene>> {
        let res = ResourceLoader::singleton().load_ex(path).done()?;
        res.try_cast::<PackedScene>().ok()
    }

    fn current_scene() -> Option<Gd<Node>> {
        let main_loop = Engine::singleton().get_main_loop()?;
        let tree: Gd<SceneTree> = main_loop.try_cast().ok()?;
        tree.get_current_scene()
    }
}

/// 注册为 Engine singleton "GDSPAWNPOOL"（Object 手动内存，进程内永不回收）
pub fn register_gdspawnpool_singleton() {
    let instance = Gd::<GdSpawnPool>::from_init_fn(|base| GdSpawnPool::init(base));
    let name = StringName::from("GDSPAWNPOOL");
    Engine::singleton().register_singleton(&name, &instance);
    std::mem::forget(instance);
}

pub fn unregister_gdspawnpool_singleton() {
    let name = StringName::from("GDSPAWNPOOL");
    Engine::singleton().unregister_singleton(&name);
}
