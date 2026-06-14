// StartAsyncTask trait - 为 Godot 节点提供启动异步任务的方法
// 基于 Future 的实现

use std::future::Future;

use godot::meta::conv::ObjectToOwned;
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::runtime::builder::CoroutineBuilder;
use crate::runtime::coroutine::SpireCoroutine;

pub trait StartAsyncTask {
    /// 使用默认设置启动异步任务
    ///
    /// # Example
    ///
    /// ```no_run
    /// use godot::prelude::*;
    /// use gamekit_core::prelude::*;
    ///
    /// fn showcase_start_async_task(node: Gd<Node>) {
    ///     node.start_async_task(async {
    ///         // 在后台线程执行耗时操作
    ///         42
    ///     });
    /// }
    /// ```
    fn start_async_task<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> Gd<SpireCoroutine>
    where
        R: 'static + ToGodot,
    {
        self.async_task(f).spawn()
    }

    /// 创建新的异步任务构建器
    ///
    /// 任务在调用 [CoroutineBuilder::spawn] 之前不会实际启动。
    ///
    /// # Example
    ///
    /// ```no_run
    /// use godot::prelude::*;
    /// use gamekit_core::prelude::*;
    ///
    /// fn showcase_async_task(node: Gd<Node>) {
    ///     node.async_task(async {
    ///         // 在后台线程执行耗时操作
    ///         "result"
    ///     })
    ///     .on_finish(|result| {
    ///         godot_print!("Task finished with: {result}");
    ///     })
    ///     .spawn();
    /// }
    /// ```
    fn async_task<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    where
        R: 'static + ToGodot;
}

impl<TSelf> StartAsyncTask for Gd<TSelf>
where
    TSelf: GodotClass + Inherits<Node>,
{
    fn async_task<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    where
        R: 'static + ToGodot,
    {
        CoroutineBuilder::new_async_task(self.clone().upcast(), f)
    }
}

impl<T> StartAsyncTask for &T
where
    T: WithBaseField + Inherits<Node>,
{
    fn async_task<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    where
        R: 'static + ToGodot,
    {
        let base = self.object_to_owned();
        CoroutineBuilder::new_async_task(base.upcast(), f)
    }
}

impl<T> StartAsyncTask for &mut T
where
    T: WithBaseField + Inherits<Node>,
{
    fn async_task<R>(
        &self,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    where
        R: 'static + ToGodot,
    {
        let base = self.object_to_owned();
        CoroutineBuilder::new_async_task(base.upcast(), f)
    }
}
