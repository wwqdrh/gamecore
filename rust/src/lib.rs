#![allow(clippy::needless_return)]
#![allow(clippy::useless_conversion)]
#![allow(unused_doc_comments)]
#![allow(private_bounds)]

#![cfg_attr(docsrs, feature(doc_cfg))]

use godot::builtin::{Callable, Variant};
use godot::prelude::*;
use godot::init::InitStage;

mod runtime;
mod state;
mod rogue;
mod console;
mod dialog;
mod ui;
mod anim;
mod manager;

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
            console::register_gdconsole_singleton();
        }
    }

    fn on_stage_deinit(stage: InitStage) {
        if stage == InitStage::Scene {
            console::unregister_gdconsole_singleton();
            state::gdcore::unregister_gdcore_singleton();
        }
    }
}

pub mod prelude {
	pub use super::runtime::coroutine::{
		SpireCoroutine,
		SIGNAL_FINISHED,
		IsRunning,
		IsFinished,
		IsPaused,
		PollMode,
	};

	pub use super::runtime::yielding::{
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

	pub use super::runtime::start_coroutine::StartCoroutine;
	pub use super::runtime::builder::CoroutineBuilder;
	pub use super::runtime::start_async_task::StartAsyncTask;
}
