#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(unboxed_closures)]

#![allow(clippy::needless_return)]
#![allow(clippy::useless_conversion)]
#![allow(unused_doc_comments)]
#![allow(private_bounds)]

#![cfg_attr(docsrs, feature(doc_cfg))]

use godot::builtin::{Callable, Variant};
use godot::prelude::*;
use godot::init::InitStage;

mod coroutine;
mod yielding;
mod builder;
mod start_coroutine;
mod start_async_task;
mod state;
mod rogue;

#[doc(hidden)]
pub enum OnFinishCall {
	Closure(Box<dyn FnOnce(Variant)>),
	Callable(Callable),
}

struct GameKitCore;

#[gdextension]
unsafe impl ExtensionLibrary for GameKitCore {
    fn on_stage_init(stage: InitStage) {
        if stage == InitStage::Scene {
            state::gdcore::register_gdcore_singleton();
        }
    }

    fn on_stage_deinit(stage: InitStage) {
        if stage == InitStage::Scene {
            state::gdcore::unregister_gdcore_singleton();
        }
    }
}

pub mod prelude {
	pub use crate::coroutine::{
		SpireCoroutine,
		SIGNAL_FINISHED,
		IsRunning,
		IsFinished,
		IsPaused,
		PollMode,
	};

	pub use crate::yielding::{
		seconds,
		frames,
		wait_while,
		wait_until,
        wait_for_signal,
        wait_for_signal_untyped,
		KeepWaiting,
		WaitUntilFinished,
		SpireYield as Yield,
        shortcuts,
	};

	pub use crate::start_coroutine::StartCoroutine;
	pub use crate::builder::CoroutineBuilder;
	pub use crate::start_async_task::StartAsyncTask;
}
