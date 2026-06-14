// StartCoroutine trait - 为 Godot 节点提供启动协程的方法
// 基于 Future 的实现，替代原来的 Coroutine trait

use std::future::Future;

use godot::meta::conv::ObjectToOwned;
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::runtime::builder::CoroutineBuilder;
use crate::runtime::coroutine::SpireCoroutine;

pub trait StartCoroutine {
    /// 使用默认设置启动新协程
    ///
    /// # Example
    ///
    /// ```no_run
    /// use godot::prelude::*;
    /// use gamekit_core::prelude::*;
    ///
    /// fn showcase_start_coroutine(node: Gd<Node>) {
    ///     node.start_coroutine(async {
    ///         frames(5).await;
    ///         // 5 帧后继续执行
    ///     });
    /// }
    /// ```
    ///
    /// # 关于 Panic
    /// 如果 Future panic，SpireCoroutine 会自动销毁
    fn start_coroutine<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> Gd<SpireCoroutine>
    where
        R: 'static + ToGodot,
    {
        self.coroutine(f).spawn()
    }

    /// 创建新的协程构建器
    ///
    /// 协程在调用 [CoroutineBuilder::spawn] 之前不会实际启动。
    ///
    /// # Example
    ///
    /// ```no_run
    /// use godot::classes::node::ProcessMode;
    /// use godot::prelude::*;
    /// use gamekit_core::prelude::*;
    ///
    /// fn showcase_coroutine(node: Gd<Node>) {
    ///     node.coroutine(async {
    ///         seconds(2.0).await;
    ///         // 2 秒后继续执行
    ///     })
    ///     .auto_start(false)
    ///     .process_mode(ProcessMode::WHEN_PAUSED)
    ///     .spawn();
    /// }
    /// ```
    ///
    /// # 关于 Panic
    /// 如果 Future panic，SpireCoroutine 会自动销毁
    fn coroutine<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    where
        R: 'static + ToGodot;
}

impl<TSelf> StartCoroutine for Gd<TSelf>
where
    TSelf: GodotClass + Inherits<Node>,
{
    fn coroutine<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    where
        R: 'static + ToGodot,
    {
        CoroutineBuilder::new_coroutine(self.clone().upcast(), f)
    }
}

impl<T> StartCoroutine for &T
where
    T: WithBaseField + Inherits<Node>,
{
    fn coroutine<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    where
        R: 'static + ToGodot,
    {
        let base = self.object_to_owned();
        CoroutineBuilder::new_coroutine(base.upcast(), f)
    }
}

impl<T> StartCoroutine for &mut T
where
    T: WithBaseField + Inherits<Node>,
{
    fn coroutine<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    where
        R: 'static + ToGodot,
    {
        let base = self.object_to_owned();
        CoroutineBuilder::new_coroutine(base.upcast(), f)
    }
}
