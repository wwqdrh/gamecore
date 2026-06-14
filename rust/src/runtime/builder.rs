// CoroutineBuilder - 协程构建器
// 基于 Future 的构建模式，替代原来的 Coroutine trait

use std::future::Future;
use std::pin::Pin;

use godot::classes::node::ProcessMode;
use godot::prelude::*;
use godot::task::TaskHandle;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::OnFinishCall;
use crate::runtime::coroutine::{PollMode, SpireCoroutine};

/// 协程构建器，用于自定义协程行为
#[must_use]
pub struct CoroutineBuilder<R: 'static + ToGodot = ()> {
    #[doc(hidden)]
    pub f: Pin<Box<dyn Future<Output = Variant>>>,
    #[doc(hidden)]
    pub owner: Gd<Node>,
    /// 决定协程在 [_process](INode::process) 还是 [_physics_process](INode::physics_process) 中轮询
    #[doc(hidden)]
    pub poll_mode: PollMode,
    /// 协程运行的 Godot [ProcessMode]
    #[doc(hidden)]
    pub process_mode: ProcessMode,
    /// 协程是否自动开始
    #[doc(hidden)]
    pub auto_start: bool,
    /// 协程完成时调用的回调列表
    #[doc(hidden)]
    pub calls_on_finish: Vec<OnFinishCall>,
    /// 协程返回值的类型提示
    #[doc(hidden)]
    pub type_hint: std::marker::PhantomData<R>,
}

impl<R> CoroutineBuilder<R>
where
    R: 'static + ToGodot,
{
    /// 创建新的协程构建器
    #[doc(hidden)]
    pub fn new_coroutine(
        owner: Gd<Node>,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R> {
        // 将用户 Future 包装为 Output = Variant 的统一 Future
        let wrapped = async move {
            let result = f.await;
            result.to_variant()
        };

        Self {
            f: Box::pin(wrapped),
            owner,
            poll_mode: PollMode::Process,
            process_mode: ProcessMode::INHERIT,
            auto_start: true,
            calls_on_finish: Vec::new(),
            type_hint: std::marker::PhantomData,
        }
    }

    /// 创建新的异步任务构建器
    ///
    /// 在后台线程运行 [Future]，主线程轮询等待结果。
    #[doc(hidden)]
    pub fn new_async_task(
        owner: Gd<Node>,
        f: impl Future<Output = R> + 'static,
    ) -> CoroutineBuilder<R>
    {
        let result_handle = Arc::new(Mutex::new(Variant::nil()));

        let task: TaskHandle = godot::task::spawn({
            let result_handle = Arc::clone(&result_handle);
            async move {
                let result = f.await;
                let mut lock = result_handle.lock();
                *lock = result.to_variant();
            }
        });

        let routine = async move {
            while task.is_pending() {
                crate::runtime::yielding::frames(1).await;
            }
            result_handle.lock().clone()
        };

        CoroutineBuilder {
            f: Box::pin(routine),
            owner,
            poll_mode: PollMode::Process,
            process_mode: ProcessMode::INHERIT,
            auto_start: true,
            calls_on_finish: Vec::new(),
            type_hint: std::marker::PhantomData,
        }
    }

    /// 协程是否在 spawn 后自动开始
    ///
    /// 如果为 false，需要在 spawn 后手动调用 [SpireCoroutine::resume]
    pub fn auto_start(self, auto_start: bool) -> Self {
        Self {
            auto_start,
            ..self
        }
    }

    /// 协程运行的 Godot [ProcessMode]
    pub fn process_mode(self, process_mode: ProcessMode) -> Self {
        Self {
            process_mode,
            ..self
        }
    }

    /// 决定协程在 [_process](INode::process) 还是 [_physics_process](INode::physics_process) 中轮询
    pub fn poll_mode(self, poll_mode: PollMode) -> Self {
        Self {
            poll_mode,
            ..self
        }
    }

    /// 添加协程完成时调用的闭包
    ///
    /// 协程的返回值(`T`)会传递给 `f`。
    ///
    /// [finished](super::coroutine::SIGNAL_FINISHED) 仅在协程正常完成时触发。
    ///
    /// 以下情况被视为"异常"结束：
    /// - 协程的父节点被删除（freed）
    /// - 协程的 Future panic
    /// - 协程以 [force_run_to_completion](SpireCoroutine::force_run_to_completion) 结束
    /// - 协程以 [kill](SpireCoroutine::kill) 结束
    pub fn on_finish(
        self,
        f: impl 'static + FnOnce(R),
    ) -> Self
    where
        R: FromGodot,
    {
        let wrapper =
            move |var: Variant| {
                match var.try_to::<R>() {
                    Ok(r) => { f(r); }
                    Err(err) => {
                        godot_error!("{err}");
                    }
                }
            };

        let mut calls_on_finish = self.calls_on_finish;
        calls_on_finish.push(OnFinishCall::Closure(Box::new(wrapper)));

        Self {
            calls_on_finish,
            ..self
        }
    }

    /// 参见 [on_finish](CoroutineBuilder::on_finish)
    ///
    /// 此变体接受 [Callable] 而非闭包。
    pub fn on_finish_callable(
        self,
        callable: Callable,
    ) -> Self
    where
        R: FromGodot,
    {
        let mut calls_on_finish = self.calls_on_finish;
        calls_on_finish.push(OnFinishCall::Callable(callable));

        Self {
            calls_on_finish,
            ..self
        }
    }

    /// 完成构建，生成协程执行器
    ///
    /// 执行器类型为 [SpireCoroutine]，会作为 `owner` 的子节点添加。
    pub fn spawn(self) -> Gd<SpireCoroutine> {
        let mut coroutine =
            Gd::from_init_fn(|base| {
                SpireCoroutine {
                    base,
                    future: self.f,
                    poll_mode: self.poll_mode,
                    paused: !self.auto_start,
                    calls_on_finish: self.calls_on_finish,
                }
            });

        coroutine.set_process_priority(256);
        coroutine.set_physics_process_priority(256);

        coroutine.set_process_mode(self.process_mode);

        let mut owner = self.owner;
        // 以防从非主线程调用
        owner.call_deferred("add_child", &[coroutine.to_variant()]);

        coroutine
    }
}
