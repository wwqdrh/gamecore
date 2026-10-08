# 场景管理、设置、相机、音频、输入

对应 Rust 源码：`rust/src/manager/`（gd_scene_root / gd_scene / setting / camera / audio / input / config_manager）

## GdSceneRoot — 场景管理器

场景树根节点，管理页面栈、转场动画、全局暂停。业务场景挂在它下面。

```gdscript
# 场景切换（正规路径，带页面栈与转场）
var root := get_parent()  # 业务节点位于 GdSceneRoot 之下时
root.change_scene("combat", {"from": "role"}, "", true, false)
# 签名: change_scene(alias: String, data: Dictionary, init_state: String, with_anim: bool, force: bool) -> bool

root.register_scene("combat", preload("res://scenes/combat.tscn"))  # 注册场景别名
root.register_scenes({"combat": scene_a, "role": scene_b})          # 批量注册

root.back_scene()                          # 返回上一页 -> bool
root.restart_scene(true, {})               # 重开当前页 (with_anim, ext_data)
root.get_current_scene()                   # 当前页面节点
root.get_scene_stack()                     # 页面栈

# 全局暂停（不要直接改 get_tree().paused）
root.set_game_paused(not root.is_game_paused())
```

`GdScene`（页面节点，继承 Control）：提供状态管理与生命周期回调，业务页面根节点用它。

## GdSceneRoot — 组件注册表（跨组件查找规范）

场景内跨组件协作**一律走 GdSceneRoot 组件注册表**，禁止 `get_parent` 链遍历、
`find_child("Xxx")` 魔法查找——节点层级是装配细节，不应成为业务耦合点。

```gdscript
# 拿场景根：GDCORE 全局节点表（GdSceneRoot 以 manager_id 注册，默认 "default"）
var scene_root: Node = Engine.get_singleton("GDCORE").get_global_node("default")

# 按组件名查询（未注册返回 null）
var map_mgr  = scene_root.get_component("MapManager")
var camera   = scene_root.get_component("ViewCamera")

# 自定义组件：装配/业务侧显式注册（重名覆盖并告警）
scene_root.register_component("Inventory", $Inventory)

# 组件释放时注销（防死引用）
scene_root.unregister_component("Inventory")
scene_root.get_component_names()   # 已注册组件名列表（调试用）
```

- **自动注册**：`ViewCamera`（GdSceneRoot ready 挂相机时）、`MapManager`
  （GdMapManager process 前几帧自动向场景根注册，兼容 GdScene 延迟创建
  默认管理器的时序；重试约 5 秒后放弃并告警）
- 场景根 `manager_id` 非 "default" 时，GdMapManager 无法自动找到场景根，
  需装配侧显式 `register_component("MapManager", $MapManager)`
- 查询时机注意：组件注册晚于子节点 ready（MapManager 靠 process 重试、
  相机靠 SceneRoot ready）——业务接线重试统一放 **_process 驱动**。
  **红线：禁止 call_deferred 自重试**——deferred 队列 flush 到空才结束，
  自重试让队列永不为空，主循环卡死在 flush 内、process 永不执行（死锁）。
  demo 范本见 `example/demo/xiuxian/role/player/player.gd`

## GdViewSetting — 游戏设置管理器

统一封装音频/窗口/自定义设置的读取、应用与持久化（依赖 GdCoreData 存档）。

```gdscript
var setting = GdViewSetting.build("user://settings.data", "setting")  # build(path, scope)
# build 后自动从存档恢复；主动重载用 load_and_apply()

# 音频
setting.set_volume("Master", 0.8)
var vol = setting.get_volume("Master")

# 窗口
setting.set_fullscreen(true)
setting.is_fullscreen()
setting.set_vsync(true)

# 自定义键值（任意业务设置）
setting.set_value("player_name", "旅人")
var name = setting.get_value("player_name", "默认名")

# 变更订阅（设置面板联动）
setting.watch("difficulty", func(_path): _apply_difficulty())
```

## GdViewCamera — 相机管理器

```gdscript
var cam = GdViewCamera.new()
add_child(cam)
cam.make_current()

# 跟随
cam.follow(player, true, true)     # (target, 立即, 启用)

# 边界限制（left, right, top, bottom）
cam.update_limit(Vector4(0, MAP_W * CELL, 0, MAP_H * CELL))
cam.disable_limit(); cam.reset_limit()

# 缩放（档位式）
cam.zoom_min = 1.5; cam.zoom_max = 2.5
cam.start_zoom(2, -1.0, -1.0)      # (档位, zmin, zmax)，-1 用默认
cam.reset_zoom()

# 表现效果
cam.shake(6.0)                     # 屏幕震动
cam.effect_hit(0.9, Vector2(2, 2)) # 受击顿挫
cam.frame_freeze(0.3, 0.1)         # 顿帧
cam.pan(target_pos)                # 平移
cam.snap_to(pos, 0.5)              # 快速吸附
cam.focus(target, 0.3)             # 聚焦；cam.unfocus() 解除
cam.turn(other_target)             # 转向另一目标
cam.blur(0.4)                      # 模糊；wave_h(0.2)/wave_v(0.2) 波动
cam.unfollow(false)                # 解除跟随
```

## GdViewAudio — 音频管理器

```gdscript
var audio = GdViewAudio.new()
add_child(audio)

# 预加载（alias, path, audio_type）
audio.preload_audio("bgm_town", "res://assets/audio/town.ogg", "bgm")
audio.preload_audio("sfx_hit", "res://assets/audio/hit.wav", "sfx")

audio.play_bgm("bgm_town", 1.0, true)   # (alias, fade秒, loop)
audio.play_bgm_random(1.0)              # 随机播放已预载 bgm
audio.play_audio("sfx_hit", 0.0)        # -> AudioStreamPlayer2D?（可定位音源）
audio.play_sfx("res://assets/audio/hit.wav", -6.0)  # 路径直放 -> bool
audio.play_voice("res://assets/audio/voice1.ogg")
audio.stop_bgm(); audio.stop_all()
```

## GdInput — 输入管理器

虚拟动作注册 + 组合按键 + 鼠标/滑动检测 + 轴映射，不依赖 InputMap 配置。

```gdscript
var input = GdInput.new()
add_child(input)

input.register_axis("move", ...)                 # 注册轴
input.register_virtual_action("interact", ...)   # 注册虚拟动作
input.register_lmouse_click(_on_press, _on_release)  # 鼠标点击回调

input.is_action_pressed("interact")
input.is_action_just_pressed("interact")
input.is_action_just_released("interact")
var dir: Vector2 = input.get_axis_value("move")
input.set_enable(false)                          # 临时禁用（对话/暂停时）
```

## GdConfigManager — 通用配置管理器

按需读取 `res://game_config.json`，适合策划配置表。

```gdscript
var config = GdConfigManager.new()   # 首次访问时读取 game_config.json
```
