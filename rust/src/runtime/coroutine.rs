// SpireCoroutine - 基于 Future 的协程管理器
// 替代原来的 Coroutine trait，使用标准 Future + async/await

use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task::{Context, Poll};

use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::OnFinishCall;
use crate::runtime::yielding;

/// 负责管理协程的 Godot 类
///
/// 不应手动构建，请使用：
/// - [crate::prelude::CoroutineBuilder]
/// - [node.start_coroutine](crate::prelude::StartCoroutine::start_coroutine)
/// - [node.start_async_task](crate::prelude::StartAsyncTask::start_async_task)
#[derive(GodotClass)]
#[class(no_init, base = Node)]
pub struct SpireCoroutine {
    #[doc(hidden)]
    pub base: Base<Node>,
    #[doc(hidden)]
    pub future: Pin<Box<dyn Future<Output = Variant>>>,
    #[doc(hidden)]
    pub poll_mode: PollMode,
    #[doc(hidden)]
    pub paused: bool,
    #[doc(hidden)]
    pub calls_on_finish: Vec<OnFinishCall>,
}

/// 定义协程在 process 帧还是 physics 帧轮询
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PollMode {
    Process,
    Physics,
}

#[godot_api]
impl INode for SpireCoroutine {
    fn process(&mut self, delta: f64) {
        if !self.paused && self.poll_mode == PollMode::Process {
            self.tick(delta);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if !self.paused && self.poll_mode == PollMode::Physics {
            self.tick(delta);
        }
    }
}

/// finished 信号名称
///
/// 可手动连接此信号以获取协程完成时的结果。
pub const SIGNAL_FINISHED: &str = "finished";

#[godot_api]
impl SpireCoroutine {
    #[signal]
    pub fn finished(result: Variant);

    // ============================================================
    // 静态工厂方法 - GDScript 侧创建协程
    // ============================================================

    /// 等待指定帧数后完成
    ///
    /// [codeblock]
    /// var coro = SpireCoroutine.wait_frames(self, 60)
    /// coro.finished.connect(func(r): print("60帧后完成"))
    /// [/codeblock]
    #[func]
    pub fn wait_frames(node: Gd<Node>, count: i64) -> Gd<SpireCoroutine> {
        let future = async move {
            yielding::frames(count).await;
            Variant::nil()
        };
        Self::spawn_with(node, Box::pin(future))
    }

    /// 等待指定秒数后完成
    ///
    /// [codeblock]
    /// var coro = SpireCoroutine.wait_seconds(self, 3.0)
    /// coro.finished.connect(func(r): print("3秒后完成"))
    /// [/codeblock]
    #[func]
    pub fn wait_seconds(node: Gd<Node>, secs: f64) -> Gd<SpireCoroutine> {
        let future = async move {
            yielding::seconds(secs).await;
            Variant::nil()
        };
        Self::spawn_with(node, Box::pin(future))
    }

    /// 等待信号发射后完成
    ///
    /// [codeblock]
    /// var sig = Signal(self, "tree_entered")
    /// var coro = SpireCoroutine.wait_signal(self, sig)
    /// coro.finished.connect(func(r): print("信号已发射"))
    /// [/codeblock]
    #[func]
    pub fn wait_signal(node: Gd<Node>, signal: Signal) -> Gd<SpireCoroutine> {
        let future = async move {
            yielding::wait_for_signal_untyped(signal).await;
            Variant::nil()
        };
        Self::spawn_with(node, Box::pin(future))
    }

    /// 从 GDScript Callable 创建协程
    ///
    /// callable 每帧被调用一次，参数为 delta_time (float)。
    /// 返回 null 时继续等待，返回非 null 值时协程完成。
    ///
    /// [codeblock]
    /// var elapsed = 0.0
    /// var coro = SpireCoroutine.run(self, _my_step)
    /// coro.finished.connect(func(r): print("结果: ", r))
    ///
    /// func _my_step(delta: float):
    ///     elapsed += delta
    ///     if elapsed >= 3.0:
    ///         return elapsed  # 非 null = 完成
    ///     return null         # null = 继续等待
    /// [/codeblock]
    #[func]
    pub fn run(node: Gd<Node>, callable: Callable) -> Gd<SpireCoroutine> {
        let future = yielding::from_callable(callable);
        Self::spawn_with(node, Box::pin(future))
    }

    // ============================================================
    // 实例方法
    // ============================================================

    #[func]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 当以下条件同时满足时返回 `true`：
    /// - 协程未暂停
    /// - 协程未完成
    #[func]
    pub fn is_running(&self) -> bool {
        !self.paused && !self.base().is_queued_for_deletion()
    }

    #[func]
    pub fn is_finished(&self) -> bool {
        self.base().is_queued_for_deletion()
    }

    /// 恢复协程
    ///
    /// 恢复一个已经在运行的协程不会有任何效果。
    #[func]
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// 暂停协程，确保在恢复之前不会执行任何指令
    ///
    /// 暂停一个已经暂停的协程不会有任何效果。
    #[func]
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// 强制协程立即完成
    ///
    /// 不会触发 `finished` 信号，结果直接返回。
    ///
    /// 注意：一次性运行协程中的所有指令可能导致意外结果。
    #[func]
    pub fn force_run_to_completion(&mut self) -> Variant {
        let mut iters_remaining = 4096;

        loop {
            match self.poll_future() {
                PollResult::Pending => {
                    iters_remaining -= 1;
                    if iters_remaining > 0 {
                        continue;
                    } else {
                        godot_error!("The coroutine exceeded the maximum number of iterations(4096). \n\
                                      This is likely a infinite loop, force stopping the coroutine.");
                        return Variant::nil();
                    }
                }
                PollResult::Complete(result) => {
                    self.de_spawn();
                    return result;
                }
                PollResult::Panicked => {
                    return Variant::nil();
                }
            }
        }
    }

    /// 销毁协程
    ///
    /// 不会触发 `finished` 信号。
    #[func]
    pub fn kill(&mut self) {
        self.de_spawn();
    }

    /// 销毁协程
    ///
    /// 以 `result` 为参数触发 `finished` 信号。
    #[func]
    pub fn finish_with(&mut self, result: Variant) {
        for call in self.calls_on_finish.drain(..) {
            match call {
                OnFinishCall::Closure(closure) => {
                    closure(result.clone());
                }
                OnFinishCall::Callable(callable) => {
                    if callable.is_valid() {
                        callable.callv(&VarArray::from(&[result.clone()]));
                    }
                }
            }
        }

        self.base_mut().emit_signal(SIGNAL_FINISHED, &[result]);
        self.de_spawn();
    }

    fn de_spawn(&mut self) {
        let mut base = self.base_mut();

        if let Some(mut parent) = base.get_parent() {
            parent.remove_child(&*base)
        }

        base.queue_free();
    }

    /// 创建 SpireCoroutine 并挂载到指定节点
    fn spawn_with(owner: Gd<Node>, future: Pin<Box<dyn Future<Output = Variant>>>) -> Gd<SpireCoroutine> {
        let mut coroutine = Gd::from_init_fn(|base| {
            SpireCoroutine {
                base,
                future,
                poll_mode: PollMode::Process,
                paused: false,
                calls_on_finish: Vec::new(),
            }
        });

        coroutine.set_process_priority(256);
        coroutine.set_physics_process_priority(256);

        let mut owner = owner;
        owner.call_deferred("add_child", &[coroutine.to_variant()]);

        coroutine
    }

    fn tick(&mut self, delta_time: f64) {
        // 设置当前帧的 delta_time，供 Future 的 poll 使用
        yielding::set_current_delta(delta_time);

        match self.poll_future() {
            PollResult::Pending => {}
            PollResult::Complete(result) => {
                self.finish_with(result);
            }
            PollResult::Panicked => {}
        }
    }

    fn poll_future(&mut self) -> PollResult {
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let waker = yielding::noop_waker();
            let mut cx = Context::from_waker(&waker);
            self.future.as_mut().poll(&mut cx)
        }));

        match result {
            Ok(Poll::Ready(value)) => PollResult::Complete(value),
            Ok(Poll::Pending) => PollResult::Pending,
            Err(err) => {
                // 协程 Future panic，替换为空 Future 并销毁
                self.future = Box::pin(async { Variant::nil() });

                let reason: &dyn std::fmt::Debug =
                    if let Some(str) = err.downcast_ref::<&str>() {
                        str
                    } else if let Some(string) = err.downcast_ref::<String>() {
                        string
                    } else {
                        &err
                    };

                self.kill();

                godot_error!("Coroutine's future panicked, the SpireCoroutine will now self-destruct.\n\
                              Panic Reason: \"{reason:?}\"");
                PollResult::Panicked
            }
        }
    }
}

enum PollResult {
    Pending,
    Complete(Variant),
    Panicked,
}

pub trait IsRunning {
    /// 参见 [SpireCoroutine::is_running]
    fn is_running(&self) -> bool;
}

impl IsRunning for Gd<SpireCoroutine> {
    fn is_running(&self) -> bool {
        self.is_instance_valid() && self.bind().is_running()
    }
}

pub trait IsFinished {
    /// 参见 [SpireCoroutine::is_finished]
    fn is_finished(&self) -> bool;
}

impl IsFinished for Gd<SpireCoroutine> {
    fn is_finished(&self) -> bool {
        !self.is_instance_valid() || self.bind().is_finished()
    }
}

pub trait IsPaused {
    /// 参见 [SpireCoroutine::is_paused]
    fn is_paused(&self) -> bool;
}

impl IsPaused for Gd<SpireCoroutine> {
    fn is_paused(&self) -> bool {
        self.is_instance_valid() && self.bind().is_paused()
    }
}
