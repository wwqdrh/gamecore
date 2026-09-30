// 全局事件 / 消息总线模块
// GdEventBus：单例发布/订阅，跨场景解耦广播

pub mod event_bus;

pub use event_bus::*;
