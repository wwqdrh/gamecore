// GdConsole - 全局控制台单例
// 基于 mlua 的 Lua 控制台，支持 GDScript 注册命令和执行 Lua 脚本
// 内置函数：fps(), memory(), cpu_info(), help(), gc_info()
// 注册为 Engine singleton "GdConsole"

use std::collections::HashMap;
use std::sync::LazyLock;

use mlua::{IntoLua, Lua, MultiValue, Value};
use parking_lot::Mutex;

use godot::builtin::{GString, StringName, Variant};
use godot::classes::{Engine, IRefCounted, Os, Performance};
use godot::prelude::*;

/// Send-safe Callable 包装
/// 安全性：所有访问均在 Godot 主线程上进行
struct SendCallable(Callable);
unsafe impl Send for SendCallable {}
unsafe impl Sync for SendCallable {}

/// 注册命令条目
struct CommandEntry {
    callable: SendCallable,
    description: String,
}

/// 全局命令注册表
static COMMAND_REGISTRY: LazyLock<Mutex<HashMap<String, CommandEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 全局输出缓冲区
static CONSOLE_OUTPUT: LazyLock<Mutex<Vec<String>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdConsole {
    lua: Mutex<Lua>,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GdConsole {
    fn init(base: Base<RefCounted>) -> Self {
        let lua = Lua::new();

        let console = Self {
            lua: Mutex::new(lua),
            base,
        };

        console.register_builtins();
        console
    }
}

#[godot_api]
impl GdConsole {
    /// 控制台输出信号，每次执行后触发
    #[signal]
    fn console_output(text: GString);

    /// 执行 Lua 代码，返回输出内容（print 输出 + 返回值或错误信息）
    #[func]
    fn execute(&mut self, code: GString) -> GString {
        CONSOLE_OUTPUT.lock().clear();

        let code_str = code.to_string();

        let (exec_result, return_value) = {
            let lua = self.lua.lock();
            // 先尝试 eval 捕获返回值
            let eval_result: mlua::Result<Value> = lua.load(&code_str).eval();
            match eval_result {
                Ok(value) => (Ok(()), Some(value)),
                Err(_) => {
                    // eval 失败（可能是语句而非表达式），回退到 exec
                    let exec_result = lua.load(&code_str).exec();
                    (exec_result, None)
                }
            }
        };

        let output = CONSOLE_OUTPUT.lock().join("\n");

        let result_str = match exec_result {
            Ok(()) => {
                let mut parts = String::new();
                if !output.is_empty() {
                    parts.push_str(&output);
                }
                // 如果有返回值且非 nil，追加到输出
                if let Some(ref value) = return_value {
                    let val_str = lua_value_to_string(value);
                    if val_str != "nil" {
                        if !parts.is_empty() {
                            parts.push('\n');
                        }
                        parts.push_str(&val_str);
                    }
                }
                GString::from(&parts)
            }
            Err(e) => {
                let error_msg = e.to_string();
                godot_error!("GdConsole: {}", error_msg);
                if output.is_empty() {
                    GString::from(&error_msg)
                } else {
                    GString::from(format!("{}\n{}", output, error_msg).as_str())
                }
            }
        };

        self.base_mut()
            .emit_signal("console_output", &[result_str.clone().to_variant()]);

        result_str
    }

    /// 执行 Lua 表达式并返回结果
    #[func]
    fn eval(&self, code: GString) -> Variant {
        CONSOLE_OUTPUT.lock().clear();

        let lua = self.lua.lock();
        match lua.load(&code.to_string()).eval::<Value>() {
            Ok(value) => lua_value_to_variant(&value),
            Err(e) => {
                godot_error!("GdConsole eval: {}", e);
                Variant::nil()
            }
        }
    }

    /// 注册 GDScript 命令，可在 Lua 中直接按名称调用
    #[func]
    fn register_command(&self, name: GString, callable: Callable, description: GString) {
        let name_str = name.to_string();
        let desc_str = description.to_string();

        COMMAND_REGISTRY.lock().insert(
            name_str.clone(),
            CommandEntry {
                callable: SendCallable(callable),
                description: desc_str,
            },
        );

        let lua = self.lua.lock();
        let cmd_name = name_str.clone();
        if let Ok(func) = lua.create_function(move |lua, args: MultiValue| {
            invoke_registered_command(&cmd_name, lua, args)
        }) {
            let globals = lua.globals();
            let _ = globals.set(name_str.as_str(), func);
        }
    }

    /// 注销已注册的命令
    #[func]
    fn unregister_command(&self, name: GString) {
        let name_str = name.to_string();
        COMMAND_REGISTRY.lock().remove(&name_str);

        let lua = self.lua.lock();
        let globals = lua.globals();
        let _ = globals.set(name.to_string().as_str(), Value::Nil);
    }

    /// 列出所有已注册命令及其描述
    #[func]
    fn list_commands(&self) -> PackedStringArray {
        let registry = COMMAND_REGISTRY.lock();
        let mut arr = PackedStringArray::new();
        for (name, entry) in registry.iter() {
            arr.push(&GString::from(
                format!("{} - {}", name, entry.description).as_str(),
            ));
        }
        arr
    }
}

/// 调用已注册的命令
fn invoke_registered_command(name: &str, lua: &Lua, args: MultiValue) -> mlua::Result<Value> {
    let registry = COMMAND_REGISTRY.lock();
    if let Some(entry) = registry.get(name) {
        let variant_args: Vec<Variant> = args.iter().map(lua_value_to_variant).collect();
        let mut arg_array = Array::<Variant>::new();
        for arg in &variant_args {
            arg_array.push(arg);
        }

        let result = entry.callable.0.callv(&arg_array);
        variant_to_lua_value(lua, &result)
    } else {
        Err(mlua::Error::external(format!(
            "Command '{}' not found",
            name
        )))
    }
}

impl GdConsole {
    /// 注册内置 Lua 函数
    fn register_builtins(&self) {
        let lua = self.lua.lock();

        // fps() - 获取当前帧率
        let fps_fn = lua
            .create_function(|_, ()| -> mlua::Result<f64> {
                Ok(Engine::singleton().get_frames_per_second() as f64)
            })
            .unwrap();

        // memory() - 获取内存信息
        let memory_fn = lua
            .create_function(|lua, ()| {
                let perf = Performance::singleton();
                let table = lua.create_table()?;
                table.set(
                    "static",
                    perf.get_monitor(godot::classes::performance::Monitor::MEMORY_STATIC),
                )?;
                table.set(
                    "message_buffer_max",
                    perf.get_monitor(
                        godot::classes::performance::Monitor::MEMORY_MESSAGE_BUFFER_MAX,
                    ),
                )?;
                Ok(table)
            })
            .unwrap();

        // gc_info() - 获取 Godot 对象信息
        let gc_fn = lua
            .create_function(|lua, ()| {
                let perf = Performance::singleton();
                let table = lua.create_table()?;
                table.set(
                    "object_count",
                    perf.get_monitor(godot::classes::performance::Monitor::OBJECT_COUNT),
                )?;
                table.set(
                    "resource_count",
                    perf.get_monitor(
                        godot::classes::performance::Monitor::OBJECT_RESOURCE_COUNT,
                    ),
                )?;
                table.set(
                    "node_count",
                    perf.get_monitor(godot::classes::performance::Monitor::OBJECT_NODE_COUNT),
                )?;
                Ok(table)
            })
            .unwrap();

        // cpu_info() - 获取 CPU 信息
        let cpu_fn = lua
            .create_function(|lua, ()| {
                let os = Os::singleton();
                let table = lua.create_table()?;
                table.set("processor_count", os.get_processor_count() as f64)?;
                Ok(table)
            })
            .unwrap();

        // help() - 列出所有可用命令
        let help_fn = lua
            .create_function(|_, ()| -> mlua::Result<()> {
                let registry = COMMAND_REGISTRY.lock();
                let mut result = String::from("Built-in commands:\n");
                result.push_str("  fps()        - Get current FPS\n");
                result.push_str("  memory()     - Get memory info\n");
                result.push_str("  gc_info()    - Get Godot object info\n");
                result.push_str("  cpu_info()   - Get CPU info\n");
                result.push_str("  help()       - Show this help\n\n");
                if !registry.is_empty() {
                    result.push_str("Registered commands:\n");
                    for (name, entry) in registry.iter() {
                        result.push_str(&format!("  {} - {}\n", name, entry.description));
                    }
                }
                CONSOLE_OUTPUT.lock().push(result);
                Ok(())
            })
            .unwrap();

        // print() - 重定向输出到控制台缓冲区
        let print_fn = lua
            .create_function(|_, args: MultiValue| {
                let mut output = String::new();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        output.push('\t');
                    }
                    output.push_str(&lua_value_to_string(&arg));
                }
                CONSOLE_OUTPUT.lock().push(output.clone());
                godot_print!("[GdConsole] {}", output);
                Ok(())
            })
            .unwrap();

        let globals = lua.globals();
        let _ = globals.set("fps", fps_fn);
        let _ = globals.set("memory", memory_fn);
        let _ = globals.set("gc_info", gc_fn);
        let _ = globals.set("cpu_info", cpu_fn);
        let _ = globals.set("help", help_fn);
        let _ = globals.set("print", print_fn);
    }
}

/// Lua Value 转字符串
fn lua_value_to_string(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string_lossy().to_string(),
        Value::Table(t) => {
            let mut result = String::from("{");
            let mut first = true;
            for pair in t.clone().pairs::<Value, Value>() {
                if let Ok((key, val)) = pair {
                    if !first {
                        result.push_str(", ");
                    }
                    first = false;
                    result.push_str(&lua_value_to_string(&key));
                    result.push('=');
                    result.push_str(&lua_value_to_string(&val));
                }
            }
            result.push('}');
            result
        }
        _ => format!("{:?}", value),
    }
}

/// Lua Value 转 Godot Variant
fn lua_value_to_variant(value: &Value) -> Variant {
    match value {
        Value::Nil => Variant::nil(),
        Value::Boolean(b) => b.to_variant(),
        Value::Integer(i) => i.to_variant(),
        Value::Number(n) => n.to_variant(),
        Value::String(s) => GString::from(&*s.to_string_lossy()).to_variant(),
        Value::Table(t) => {
            let mut dict = Dictionary::<GString, Variant>::new();
            for pair in t.clone().pairs::<Value, Value>() {
                if let Ok((key, val)) = pair {
                    let key_str = match &key {
                        Value::String(s) => GString::from(&*s.to_string_lossy()),
                        Value::Integer(i) => GString::from(i.to_string().as_str()),
                        _ => continue,
                    };
                    dict.set(&key_str, &lua_value_to_variant(&val));
                }
            }
            dict.to_variant()
        }
        _ => Variant::nil(),
    }
}

/// Godot Variant 转 Lua Value
fn variant_to_lua_value(lua: &Lua, variant: &Variant) -> mlua::Result<Value> {
    match variant.get_type() {
        VariantType::NIL => Ok(Value::Nil),
        VariantType::BOOL => {
            let b: bool = variant.to();
            b.into_lua(lua)
        }
        VariantType::INT => {
            let i: i64 = variant.to();
            i.into_lua(lua)
        }
        VariantType::FLOAT => {
            let f: f64 = variant.to();
            f.into_lua(lua)
        }
        VariantType::STRING => {
            let s: GString = variant.to();
            s.to_string().into_lua(lua)
        }
        _ => Ok(Value::Nil),
    }
}

/// 注册 GdConsole 全局单例
pub fn register_gdconsole_singleton() {
    let instance = Gd::<GdConsole>::from_init_fn(|base| GdConsole::init(base));
    let name = StringName::from("GdConsole");
    Engine::singleton().register_singleton(&name, &instance);
    std::mem::forget(instance);
}

/// 注销 GdConsole 全局单例
pub fn unregister_gdconsole_singleton() {
    let name = StringName::from("GdConsole");
    Engine::singleton().unregister_singleton(&name);
}
