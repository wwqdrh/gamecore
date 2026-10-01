# 天气环境与画布绘图

对应 Rust 源码：`rust/src/environment/`（shaders / weather）、`rust/src/drawer/`（geometry / colors / canvas_drawer）
可运行示例：`example/environment/environment_demo.gd`

## GdWeather — 天气管理节点（Node2D）

屏幕后期处理方案：全屏 ColorRect 覆盖层 + `hint_screen_texture` 采样已渲染画面，
shader 以字符串形式编译在 Rust 二进制内（**无需任何 .gdshader / 贴图资源**）。

```gdscript
var weather = GdWeather.new()
weather.leaf_count = 50            # 刮风模式树叶数量
add_child(weather)                 # 建议挂场景根或 CanvasLayer

weather.set_weather("day")         # "day"/"night"/"rain"/"snow"/"wind"/"clear"
weather.intensity = 0.5            # 强度 0..1（渐入由内部 fade 处理）

weather.get_weather()              # 当前天气名
weather.get_overlay()              # 覆盖层 ColorRect（特殊调整用）
```

内置天气效果：

| 名称 | 效果 |
|---|---|
| `day` | 暖阳色调 |
| `night` | 冷蓝暗色 + 暗角 |
| `rain` | 双层视差雨条（向下落） |
| `snow` | 网格哈希雪花（向下飘落） |
| `wind` | 值噪声云影 + 叶片飘动（复用 drawer 叶形与秋季调色板） |
| `clear` | 清除天气 |

切换即动态创建对应 shader；`set_weather` 换名即可，无需预加载资源。

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
