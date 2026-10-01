# Lua 调试控制台

对应 Rust 源码：`rust/src/console/gdconsole.rs`
可运行示例：`example/console/console_example.gd`

## 面板挂载

```gdscript
# 运行时把控制台面板挂到当前场景（按 ` 键打开/关闭）
var panel = load("res://addons/gamecore/ui/console_panel.gd").new()
add_child(panel)
```

## 注册命令（单例 "GdConsole"，注意大小写）

```gdscript
var console = Engine.get_singleton("GdConsole")

# register_command(命令名, 可调用, 帮助文本)
# 命令函数参数即 Lua 调用参数；返回值会打印到控制台
console.register_command("heal", _cmd_heal, "Heal player by amount (e.g. heal(50))")
console.register_command("gold", _cmd_gold, "Add gold (e.g. gold(100))")

func _cmd_heal(amount: int) -> String:
    GDCORE.get_global_node("player").health.heal(float(amount))
    return "healed %d" % amount
```

控制台内可执行任意 Lua 脚本，内置函数：`fps()`、`memory()`、`cpu_info()`、`gc_info()`、`help()`。

调试用途：运行期改数据、调状态、查性能，避免频繁改代码重启。
