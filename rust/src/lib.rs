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
mod map;
mod role;
mod dev;
mod drawer;
mod environment;
mod event;
mod pool;
mod components;
mod debug;

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
            state::state_store::register_gdstate_singleton();
            console::register_gdconsole_singleton();
            event::event_bus::register_gdeventbus_singleton();
            pool::spawn_pool::register_gdspawnpool_singleton();
            debug::debug_draw::register_gddebugdraw_singleton();
            // .gml 不再注册 ResourceFormatLoader（那会劫持 .gml 使其无法在
            // 编辑器中按纯文本打开/编辑）。工作流改为：.gml 是纯文本源码，
            // 编辑器插件扫描后调用 GdUiBuilder.build_scene_file 自动生成
            // 同名 .gml.tscn（真实场景文件），gml 修改后 tscn 自动再生成。
            // .gjson 相反：是加密产物（密文无文本编辑意义），必须注册
            // ResourceFormatLoader，编辑器双击以 GdJson 资源打开而非文本
            state::gjson_loader::register_gjson_loader();
        }
    }

    // gml 自举钩子：Scene stage init 时 main loop 尚未创建，延迟到首帧连接
    // node_added 监听（幂等，连接成功后立即返回）。
    // 同时在进程首帧一次性恢复持久化设置（音频/全屏/垂直同步）——不能放
    // Scene stage init（main loop 未创建，根窗口不可用），也不能依赖设置 UI
    // 懒构建（标题场景无设置 UI，全屏要进主场景才生效）。
    fn on_main_loop_frame() {
        manager::setting::restore_persisted_once();
        state::gdcore::connect_gml_auto_connect_hook();
    }

    fn on_stage_deinit(stage: InitStage) {
        if stage == InitStage::Scene {
            state::gjson_loader::unregister_gjson_loader();
            debug::debug_draw::unregister_gddebugdraw_singleton();
            pool::spawn_pool::unregister_gdspawnpool_singleton();
            event::event_bus::unregister_gdeventbus_singleton();
            console::unregister_gdconsole_singleton();
            state::state_store::unregister_gdstate_singleton();
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
