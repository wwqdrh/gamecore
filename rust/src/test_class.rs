// TestClass - 协程功能测试类，暴露给 Godot 用于 GDScript 调用测试

use std::time::Duration;

use async_compat::Compat;
use crate::prelude::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base = Node)]
struct TestClass {
    base: Base<Node>,
}

#[godot_api]
impl TestClass {
    #[func]
    fn test_routine(&mut self) -> Gd<SpireCoroutine> {
        self.start_async_task(Compat::new(async {
            godot_print!("Using compat layer!");
            smol::Timer::after(Duration::from_secs(2)).await;
            godot_print!("Awaited 2 seconds, returning 5");
            5_i32
        }))
    }

    #[func]
    fn test_from_other_node(node: Gd<Node>) -> Gd<SpireCoroutine> {
        node.start_async_task(Compat::new(async {
            godot_print!("Async task from other node!");
            smol::Timer::after(Duration::from_secs(2)).await;
            godot_print!("Awaited 2 seconds, returning `finished task`");
            "finished task"
        }))
    }
}
