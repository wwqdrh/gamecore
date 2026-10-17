# Lua 调试控制台

对应 Rust 源码：`rust/src/console/gdconsole.rs`
可运行示例：`example/console/console_example.gd`

## 面板挂载

方式一：**场景节点**（推荐）——面板脚本带 `class_name GdConsolePanel`，
在场景中创建 CanvasLayer 节点直接挂 `addons/gamecore/ui/console_panel.gd`，
Inspector 可配置：

- `toggle_key`（Key，默认 `KEY_QUOTELEFT` 反引号键）：开合快捷键
- `console_layer`（默认 100）：面板所在 CanvasLayer 层级
  （调试面板建议高于演出段 100，如 200）
- `console_height_ratio` / `max_log_lines` / `font_size`

面板公开 API：`open_console()` / `close_console()` / `toggle_console()` /
`is_open()`；控制台打开时 ESC 也可关闭。

```gdscript
# index.tscn 范本（example/demo/xiuxian/index.tscn）
[node name="ConsolePanel" type="CanvasLayer" parent="."]
console_layer = 200
script = ExtResource(".../console_panel.gd")
```

方式二：运行时动态挂载：

```gdscript
var panel = load("res://addons/gamecore/ui/console_panel.gd").new()
add_child(panel)
```

## GDScript 侧推送输出

`GdConsole.console_print(text)`：把一行文本立即经 `console_output` 信号
显示到面板（不进 Lua 缓冲区、不被下一次 execute 清空；同时留痕 stdout）。
适合命令执行进度播报（范本 demo `index.gd` 连播命令）。

**红线：控制台命令回调内禁直调 GdConsole 任何方法**——命令由
`execute()` **同步回调**（execute 持有 `&mut` 借用全程，且 lua Mutex
非重入），直调 `console_print` 借用重入 panic（"Gd<T>::bind_mut()
failed, already bound"），直调 `execute/eval` 死锁。必须
`console.call_deferred("console_print", text)`，待 execute 返回后再发
（与「watch 回调内禁同步调 GDSTATE」同类借用重入问题）。

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

## 连播剧情命令范本（demo index.gd）

`example/demo/xiuxian/index.gd` 绑定顶层 `index.tscn` 根
（extends GdSceneRoot），注册 `play_all_movies()` / `stop_all_movies()`：
扫描 `movie/` 下全部 .movie（叙事排序、排除 test_smoke），逐段串行播放
（上段 `s_movie_finished` → deferred 接下一段），不写 XiuMovieState 播放
记录（纯预览不影响存档）。

**红线：根脚本 extends GdSceneRoot 时严禁定义 `_ready` / `_process`** ——
GDScript 虚函数会整体遮蔽 Rust 侧同名虚函数（GdSceneRoot.ready 负责
SceneLayer/ViewCamera/组件表/entry_scene 自动进场景，process 驱动转场
状态机），被遮蔽即框架瘫痪。初始化钩子用 `_enter_tree` / `_exit_tree`
（Rust 未实现，不冲突）；`_exit_tree` 里注销命令防悬垂 Callable。
