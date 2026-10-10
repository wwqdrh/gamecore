# 天气环境与画布绘图

对应 Rust 源码：`rust/src/environment/`（shaders / weather）、`rust/src/drawer/`（geometry / colors / canvas_drawer）
可运行示例：`example/environment/environment_demo.gd`

## GdWeather — 天气视觉节点（Node2D，双通道架构）

屏幕后期处理方案：全屏 ColorRect 覆盖层 + `hint_screen_texture` 采样已渲染画面，
shader 以字符串形式编译在 Rust 二进制内（**无需任何 .gdshader / 贴图资源**）。

**双通道架构（过渡连续性的关键）**：

- **色调通道（tint）**：单一覆盖层 + 通用 tint shader，承载 dawn/day/dusk/night
  四相位。相位切换**不换 shader、不换层**，只是调色板目标值（乘色/附加光/暗角）
  变化 → 线性进度 + smoothstep 逐帧缓动（`tint_fade_time`，默认 4s）——
  亮度/色温全程连续，天亮天黑无跳变；中途切换相位从当前显示值出发，始终平滑。
- **效果通道（fx）**：双层覆盖层交叉渐变，承载 rain/storm/snow/fog/wind。
  shader 内部按 **density 哈希门控**（雨柱/雪花逐个"点亮/熄灭"）——
  density 由 fade × intensity 驱动 → **雨滴数量由少到多/由多到少真实渐变**
  （起雨/停雨/雾浓雾淡都是数量渐变，而非整屏透明度缩放）。
- 通道独立：夜晚下雨 = night 色调 + rain 效果叠加（各管各的）。

```gdscript
var weather = GdWeather.new()
weather.leaf_count = 50            # 刮风模式树叶数量
weather.fade_in_time = 2.0         # 效果渐入时长秒（默认 2.0）
weather.fade_out_time = 3.0        # 效果渐出时长秒（默认 3.0）
weather.tint_fade_time = 4.0       # 色调缓动时长秒（默认 4.0）
add_child(weather)                 # 建议挂场景根或 CanvasLayer

weather.set_weather("day")         # 兼容入口：按名字自动路由 tint/effect 通道
weather.set_tint("night")          # 色调通道（day/night/dawn/dusk，其他名 = 清除）
weather.set_effect("rain")         # 效果通道（rain/storm/snow/fog/wind，其他 = 清除）
weather.intensity = 0.5            # 效果密度终值倍率 0..1（乘在 density 上）

weather.get_weather()              # 当前目标天气名
weather.get_tint_strength()        # 色调强度 0..1（缓动值）
weather.get_fx_density()           # 效果密度 0..1（fade × intensity）
weather.get_visible_overlay_count() # 可见覆盖层数（tint + fx）
weather.get_overlay()              # fx 活动层覆盖层 ColorRect（特殊调整用）
```

内置天气效果：

| 名称 | 通道 | 效果 |
|---|---|---|
| `day` | tint | 暖阳色调 |
| `night` | tint | 冷蓝暗色 + 暗角 |
| `dawn` | tint | 晨光金调（黎明） |
| `dusk` | tint | 橙红暮色 + 压暗（黄昏） |
| `fog` | fx | 去饱和 + 双层流动噪声雾团（浓度随 density 渐变） |
| `rain` | fx | 密度门控双层视差雨条（雨滴数量随 density 增减） |
| `storm` | fx | 暗青压暗 + 密雨 + 闪电（雨势足够大才触发闪电） |
| `snow` | fx | 密度门控网格哈希雪花 |
| `wind` | fx | 值噪声云影 + 叶片飘动（树叶透明度跟随密度） |
| `clear` | - | 清除（fx density 渐出至 0 / tint 强度渐出至 0） |

性能要点：shader 对象按名字缓存复用（切换不再重建）；density / tint uniforms
去重（值稳定后不再每帧写）；树叶绘制先收集后绘制（避免整表克隆）。

**覆盖层锚定（2026-10-10 修复）**：覆盖层与树叶活动范围每帧对齐「可视世界
矩形」（视口矩形经画布变换逆推，含相机位置/缩放/旋转）并外扩半屏——与相机
彻底解耦，换地图（zoom/边界变化 + limit smoothing 滑行越界）也不会露出无天气
区域。红线：**严禁把覆盖层固定在世界原点按窗口像素尺寸摆放**——只有地图恰好
在原点附近且不大于视口时才碰巧盖住屏幕，换图后必现"天气被重置"亮带。配套
措施：GdWeather ready 设 `process_priority = 4096`（晚于相机/玩家，消除同帧
时序滞后）；fx/tint shader 全部基于 SCREEN_UV 屏幕坐标采样，覆盖层外扩不
影响视觉。回归：test/check_weather_map_switch.gd。

## GdWeatherManager — 天气/昼夜管理器（Node，纯逻辑）

自动昼夜循环 + 概率天气，驱动 GdWeather 视觉节点。持有游戏内时间流：
**初始化时给一次总时长，之后内部自行累计**——ready 时若
`auto_start_from_ticks`（默认 true）优先取 `GDCORE.get_play_time()`
（当前存档跨开关累计的真实游玩秒数）作初值，单例不可用回退引擎进程时长
（`Time.get_ticks_msec()`）；外部也可调 `set_total_time(seconds)` 传入
（优先级最高，会清空进行中的降雨/雾瞬态）。process 中按
`total_time += delta * time_scale` 内部流转，业务无需每帧喂数据。

```gdscript
var mgr = GdWeatherManager.new()
# 编辑器 Inspector 可调的配置项：
mgr.time_scale = 600.0          # 时间流速：游戏秒/现实秒（600 = 1 现实秒 10 游戏分钟）
mgr.day_length = 86400.0        # 一昼夜的游戏秒数
mgr.start_time = 21600.0        # 初始时刻偏移（游戏秒；21600 = 早 6:00）
mgr.day_start_hour = 6.0        # 白天开始小时
mgr.night_start_hour = 18.0     # 黑夜开始小时
mgr.twilight_hours = 1.5        # 黎明/黄昏时长（小时）
mgr.rain_chance = 0.35          # 下雨概率 0..1（每次判定）
mgr.storm_chance = 0.3          # 降雨中雷暴占比 0..1
mgr.fog_chance = 0.25           # 黎明遇雾概率 0..1（雾持续到黎明结束）
mgr.check_interval = 3600.0     # 天气判定间隔（游戏秒）
mgr.rain_max_duration = 7200.0  # 单次降雨最长持续（实际随机 0.4..1 倍）
mgr.weather_path = "Weather"    # GdWeather 视觉节点路径（子节点，兄弟名自动回退 "../"）
mgr.auto_start_from_ticks = true
add_child(mgr)

var w = GdWeather.new(); w.name = "Weather"
add_child(w)                    # 管理器自动 set_weather("dawn"/"day"/"dusk"/"night"/...)

mgr.set_total_time(save.play_seconds)  # 读档时传入累计时长（只需一次）
mgr.get_game_time()             # 游戏内总时长（游戏秒）
mgr.get_hour()                  # 当前游戏内小时 0..24
mgr.is_daytime()                # day_start .. night_start 白天语义
mgr.get_phase_name()            # 当前相位 dawn/day/dusk/night
mgr.get_current_weather()       # 当前下发天气名

mgr.s_phase_changed.connect(func(is_day): ...)          # 昼夜语义切换
mgr.s_phase_name_changed.connect(func(name): ...)       # 相位切换（黎明→白天等）
mgr.s_weather_changed.connect(func(name): ...)          # 天气切换（含降雨起止/雾/雷暴）
```

相位规则：`hour = total_time / day_length * 24`；
黎明 = day_start 前 twilight 小时（跨午夜回绕）、白天、黄昏 = night_start 前
twilight 小时、其余黑夜。天气优先级：**降雨(rain/storm) > 晨雾(fog) >
相位基准(dawn/day/dusk/night)**。视觉过渡由 GdWeather 双通道承担：
相位基准走 tint 调色板缓动（亮度连续），rain/storm/fog 走 fx density 门控
（雨滴数量真实增减）。管理器另可配 `weather_intensity`（效果密度倍率 0..1）。
`weather_path` 未命中时纯逻辑运行（信号照发，不下发视觉）。
节点摆放：视觉节点需位于地图/玩家之后、UI CanvasLayer 之前（覆盖层在默认画布按树序绘制）。

验收：`test/check_weather_manager.gd`（时间流/四相位与信号/降雨雷暴/晨雾/
视觉下发/demo 主场景集成）+ `test/check_weather_fade.gd`（交叉渐变/天气全集）
+ `test/check_playtime.gd`（GDCORE 存档时长）；demo 范本
`example/demo/xiuxian/scenes/main/index.tscn`（WeatherManager + Weather 平级挂主场景根）。

## GdCanvasDrawer — 队列式画布绘制（Node2D）

用代码画装饰性图形（原型期占位美术），叶子轮廓为二次贝塞尔生成：

```gdscript
var d = GdCanvasDrawer.new()
add_child(d)

d.add_rect(pos, size, color)              # 矩形
d.add_circle(pos, radius, color)          # 圆
d.add_line(from, to, color, width)        # 线
d.add_polygon(points, color)              # 多边形
d.add_leaf(pos, rotation, scale, color)   # 树叶

d.shape_count()                           # 当前图形数
d.clear_shapes()                          # 清空
```

## 颜色工具

```gdscript
# GdColorUtil 静态方法（秋季叶色调色板、hex 解析、插值）
var c = GdColorUtil.from_hex("c96f2e")
```

纯几何函数（`geometry.rs`：树叶/圆轮廓、点集旋转平移）与调色板仅 Rust 侧内部使用，
GDScript 侧通过 GdCanvasDrawer / GdWeather 间接消费。
