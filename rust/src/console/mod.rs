// 后台控制台模块
// 基于 mlua 的 Lua 控制台，支持 GDScript 注册命令和执行 Lua 脚本
// 内置函数：fps(), memory(), cpu_info(), help(), gc_info()

mod gdconsole;

pub use gdconsole::{register_gdconsole_singleton, unregister_gdconsole_singleton};
