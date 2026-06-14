// 协程等待机制 - 基于 async/await 的 Future 实现
// 替代原来的 SpireYield 枚举 + yield 语法，使用标准 Future trait

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

use godot::{obj::WithSignals, prelude::*, signal::TypedSignal};

use crate::prelude::*;

// ============================================================
// Delta Time 上下文 - 通过 Thread Local 传递当前帧的 delta
// ============================================================

thread_local! {
    static CURRENT_DELTA: std::cell::RefCell<f64> = std::cell::RefCell::new(0.0);
}

/// 设置当前帧的 delta_time（由 SpireCoroutine 在 poll 时调用）
pub(crate) fn set_current_delta(delta: f64) {
    CURRENT_DELTA.with(|d| *d.borrow_mut() = delta);
}

/// 获取当前帧的 delta_time
pub(crate) fn get_current_delta() -> f64 {
    CURRENT_DELTA.with(|d| *d.borrow())
}

// ============================================================
// No-op Waker - 帧驱动轮询不需要唤醒通知
// ============================================================

/// 创建一个 no-op Waker
///
/// 帧驱动轮询不需要唤醒通知，Waker 的所有操作都是空操作。
pub(crate) fn noop_waker() -> Waker {
    use std::task::{RawWaker, RawWakerVTable};

    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |s| RawWaker::new(s, &VTABLE), // clone
        |_| {},                         // wake
        |_| {},                         // wake_by_ref
        |_| {},                         // drop
    );
    // SAFETY: vtable 指向静态常量，data 为空指针，所有操作都是 no-op
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

// ============================================================
// FramesFuture - 等待指定帧数
// ============================================================

/// 等待指定帧数的 Future
pub struct FramesFuture {
    frames_remaining: i64,
}

impl Future for FramesFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// 等待指定帧数
///
/// 一帧等于一次 [_process](INode::process) 或 [_physics_process](INode::physics_process) 调用，
/// 取决于协程的 [PollMode](super::coroutine::PollMode)。
///
/// # Example
///
/// ```no_run
/// use godot::prelude::*;
/// use gamekit_core::prelude::*;
///
/// fn showcase_frames(node: Gd<Node>) {
///     node.start_coroutine(async {
///         frames(5).await;
///         // 5 帧后继续执行
///     });
/// }
/// ```
pub fn frames(count: i64) -> FramesFuture {
    FramesFuture { frames_remaining: count }
}

// ============================================================
// SecondsFuture - 等待指定秒数
// ============================================================

/// 等待指定秒数的 Future
pub struct SecondsFuture {
    seconds_remaining: f64,
}

impl Future for SecondsFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let delta = get_current_delta();
        if self.seconds_remaining > delta {
            self.seconds_remaining -= delta;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// 等待指定秒数
///
/// 时间计数受 [Engine::time_scale](Engine::get_time_scale) 影响，
/// 也取决于协程的 [PollMode](super::coroutine::PollMode)。
/// 协程未被处理时时间不会流逝。
///
/// # Example
///
/// ```no_run
/// use godot::prelude::*;
/// use gamekit_core::prelude::*;
///
/// fn showcase_seconds(node: Gd<Node>) {
///     node.start_coroutine(async {
///         seconds(7.5).await;
///         // 7.5 秒后继续执行
///     });
/// }
/// ```
pub fn seconds(secs: f64) -> SecondsFuture {
    SecondsFuture { seconds_remaining: secs }
}

// ============================================================
// WaitWhileFuture - 条件为 true 时持续等待
// ============================================================

/// 条件为 true 时持续等待的 Future
pub struct WaitWhileFuture {
    condition: Box<dyn FnMut() -> bool>,
}

impl Future for WaitWhileFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if (self.condition)() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// 协程暂停执行，直到 `f` 返回 false
///
/// `f` 在每次协程被轮询时调用。
///
/// # Example
///
/// ```no_run
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use godot::prelude::*;
/// use gamekit_core::prelude::*;
///
/// fn showcase_wait_while(node: Gd<Node>, flag: AtomicBool) {
///     node.start_coroutine(async {
///         wait_while(move || flag.load(Ordering::Relaxed)).await;
///         // flag 变为 false 后继续执行
///     });
/// }
/// ```
pub fn wait_while(f: impl FnMut() -> bool + 'static) -> WaitWhileFuture {
    WaitWhileFuture { condition: Box::new(f) }
}

// ============================================================
// WaitUntilFuture - 条件为 true 时恢复
// ============================================================

/// 条件为 true 时恢复执行的 Future
pub struct WaitUntilFuture {
    condition: Box<dyn FnMut() -> bool>,
}

impl Future for WaitUntilFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if (self.condition)() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// 协程暂停执行，直到 `f` 返回 true
///
/// `f` 在每次协程被轮询时调用。
///
/// # Example
///
/// ```no_run
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use godot::prelude::*;
/// use gamekit_core::prelude::*;
///
/// fn showcase_wait_until(node: Gd<Node>, flag: AtomicBool) {
///     node.start_coroutine(async {
///         wait_until(move || flag.load(Ordering::Relaxed)).await;
///         // flag 变为 true 后继续执行
///     });
/// }
/// ```
pub fn wait_until(f: impl FnMut() -> bool + 'static) -> WaitUntilFuture {
    WaitUntilFuture { condition: Box::new(f) }
}

// ============================================================
// SignalFuture - 等待 Godot 信号
// ============================================================

/// 等待 Godot 信号发射的 Future
pub struct SignalFuture {
    signal: Signal,
    emission_tracker: Option<Arc<AtomicBool>>,
}

impl Future for SignalFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 先初始化 tracker（如果尚未初始化）
        if self.emission_tracker.is_none() {
            let tracker = Arc::new(AtomicBool::new(false));
            self.signal.connect(&Callable::from_sync_fn("coroutines_signal_emission_tracker", {
                let tracker = tracker.clone();
                move |_| tracker.store(true, Ordering::Relaxed)
            }));
            self.emission_tracker = Some(tracker);
        }

        if self.emission_tracker.as_ref().unwrap().load(Ordering::Relaxed) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// 协程暂停执行，直到给定的 [Signal] 被发射（非类型化版本）
///
/// 参见 [wait_for_signal] 的类型化版本。
///
/// # Example
///
/// ```no_run
/// use godot::prelude::*;
/// use gamekit_core::prelude::*;
///
/// fn showcase_wait_for_signal_untyped(node: Gd<Node>) {
///     let signal = Signal::from_object_signal(&node, "child_entered_tree");
///     node.start_coroutine(async move {
///         wait_for_signal_untyped(signal).await;
///         // 信号发射后继续执行
///     });
/// }
/// ```
pub fn wait_for_signal_untyped(signal: Signal) -> SignalFuture {
    SignalFuture {
        signal,
        emission_tracker: None,
    }
}

/// 协程暂停执行，直到给定的类型化信号被发射
///
/// 接受 [TypedSignal] 的引用，可通过 Godot 类的 `gd.signals().signal_name()` 获取。
///
/// 参见 [wait_for_signal_untyped] 的非类型化版本。
///
/// # Example
///
/// ```no_run
/// use godot::prelude::*;
/// use gamekit_core::prelude::*;
///
/// fn showcase_wait_for_signal(node: Gd<Node>) {
///     let node_cp = node.clone();
///     node.start_coroutine(async move {
///         wait_for_signal(&node_cp.signals().child_entered_tree()).await;
///         // 信号发射后继续执行
///     });
/// }
/// ```
pub fn wait_for_signal<C, PS>(signal: &TypedSignal<C, PS>) -> SignalFuture
where
    C: WithSignals,
    PS: godot::meta::conv::ParamTuple,
{
    let untyped = signal.to_untyped();
    wait_for_signal_untyped(untyped)
}

// ============================================================
// CoroutineFuture - 等待另一个协程完成
// ============================================================

/// 等待另一个协程完成的 Future
pub struct CoroutineFuture {
    coroutine: Gd<SpireCoroutine>,
}

impl Future for CoroutineFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 协程完成时会自动销毁（de_spawn），实例失效即表示完成
        if self.coroutine.is_instance_valid() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub trait WaitUntilFinished {
    /// 返回一个 Future，等待该协程完成后 resolve。
    fn wait_until_finished(&self) -> CoroutineFuture;
}

impl WaitUntilFinished for Gd<SpireCoroutine> {
    fn wait_until_finished(&self) -> CoroutineFuture {
        CoroutineFuture { coroutine: self.clone() }
    }
}

impl WaitUntilFinished for SpireCoroutine {
    fn wait_until_finished(&self) -> CoroutineFuture {
        self.to_gd().wait_until_finished()
    }
}

// ============================================================
// KeepWaiting trait - 保留向后兼容
// ============================================================

/// 判断是否继续等待的 trait
pub trait KeepWaiting {
    /// 返回 true 表示继续等待，false 表示可以恢复
    fn keep_waiting(&mut self, delta_time: f64) -> bool;
}

impl<T: FnMut() -> bool> KeepWaiting for T {
    fn keep_waiting(&mut self, _delta_time: f64) -> bool { self() }
}

impl KeepWaiting for Gd<SpireCoroutine> {
    fn keep_waiting(&mut self, _delta_time: f64) -> bool {
        self.is_instance_valid()
    }
}

// ============================================================
// SpireYield 枚举 - 保留用于 Dyn 变体的兼容
// ============================================================

/// 协程等待模式枚举（保留用于 Dyn 变体兼容）
pub enum SpireYield {
    Dyn(Box<dyn KeepWaiting>),
}

// ============================================================
// CallableFuture - 包装 GDScript Callable 为 Future
// ============================================================

/// 包装 GDScript Callable 的 Future
///
/// 每帧调用一次 callable，传入 delta_time 作为参数。
/// - 返回 `null` → 继续等待 (Pending)
/// - 返回非 null 值 → 协程完成，该值作为结果 (Ready)
pub struct CallableFuture {
    callable: Callable,
}

impl Future for CallableFuture {
    type Output = Variant;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Variant> {
        let delta = get_current_delta();
        let args = [delta.to_variant()];
        let result = self.callable.callv(&VarArray::from(&args));
        if result.is_nil() {
            Poll::Pending
        } else {
            Poll::Ready(result)
        }
    }
}

/// 从 GDScript Callable 创建 Future
///
/// callable 每帧被调用一次，参数为 delta_time (float)。
/// 返回 null 时继续等待，返回非 null 值时协程完成。
pub fn from_callable(callable: Callable) -> CallableFuture {
    CallableFuture { callable }
}

// ============================================================
// 快捷方式
// ============================================================

/// 快捷方式模块
pub mod shortcuts {
    pub use super::wait_for_signal_untyped as signal_untyped;
    pub use super::wait_for_signal as signal;
    pub use super::wait_until as until;
    pub use super::wait_while as whilst;

    /// 等待下一帧
    pub fn next_frame() -> super::FramesFuture { super::frames(1) }
}
