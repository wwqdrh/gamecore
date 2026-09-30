// 环境模块
// shaders:  内置天气 shader 源码注册表（字符串动态创建 Shader）
// GdWeather: 天气管理节点（白天/夜晚/雨/雪/风），屏幕后期 + 刮风树叶
//
// 天气 shader 走"全屏 ColorRect 覆盖层 + hint_screen_texture 后期处理"路线：
//   采样已渲染画面 → 调色 / 加粒子 → mix(intensity) 输出，
//   无需任何贴图资源；GdWeather.set_weather 按名字查表动态创建 Shader。

pub mod shaders;
pub mod weather;

pub use weather::GdWeather;
