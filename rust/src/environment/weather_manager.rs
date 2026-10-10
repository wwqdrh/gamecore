// GdWeatherManager - 天气/昼夜管理器（纯逻辑，驱动 GdWeather 视觉节点）
//
// 职责：持有游戏内时间流（total_time），按昼夜周期计算四相位
//       （黎明 dawn / 白天 day / 黄昏 dusk / 黑夜 night，相位间自然过渡），
//       按概率周期性判定降雨（rain/storm）与晨雾（fog），
//       把最终天气名下发给 GdWeather（双通道视觉：tint 相位调色板缓动——
//       亮度连续无跳变；fx 效果 density 门控——雨滴数量真实增减）。
//       本节点不画任何东西——视觉全在 GdWeather（shader 后期 + 树叶）。
//
// 时间流（一次传入，内部自算——业务无需每帧喂数据）：
//   · 初始化：ready 时若 auto_start_from_ticks（默认 true），优先取
//     GDCORE.get_play_time()（当前存档跨开关累计的真实游玩秒数），
//     单例不可用时回退引擎 Time.get_ticks_msec()；外部也可显式调
//     set_total_time(seconds) 传入（优先级最高）。
//   · 流转：process 中 total_time += delta * time_scale（游戏秒/现实秒）。
//   · 相位：game_hour = total_time / day_length * 24 % 24，
//     [day_start-twilight, day_start) 黎明、[day_start, night_start-twilight)
//     白天、[night_start-twilight, night_start) 黄昏、其余黑夜。
//   · 降雨：每 check_interval（游戏秒）roll randf() < rain_chance，命中后
//     再按 storm_chance 决定 rain/storm，持续 randf_range(0.4, 1.0) *
//     rain_max_duration 后回到当前相位基准（过渡由 GdWeather 交叉渐变承担）。
//   · 晨雾：黎明相位内每判定周期 roll fog_chance，命中起雾到黎明结束。
//
// 天气优先级：降雨(rain/storm) > 晨雾(fog) > 相位基准(dawn/day/dusk/night)
//
// 配置（编辑器 Inspector 可调，#[var] 导出）：
//   time_scale            游戏时间流速（游戏秒/现实秒，默认 600）
//   day_length            一昼夜的游戏秒数（默认 86400）
//   start_time            初始游戏时刻偏移（游戏秒，默认 21600 = 早 6:00）
//   day_start_hour        白天开始小时（默认 6.0）
//   night_start_hour      黑夜开始小时（默认 18.0）
//   twilight_hours        黎明/黄昏时长（小时，默认 1.5）
//   rain_chance           下雨概率 0..1（每次判定，默认 0.25）
//   storm_chance          降雨中雷暴占比 0..1（默认 0.3）
//   fog_chance            黎明遇雾概率 0..1（默认 0.25）
//   check_interval        天气判定间隔（游戏秒，默认 3600）
//   rain_max_duration     单次降雨最长持续（游戏秒，默认 7200）
//   auto_start_from_ticks ready 自动取 GDCORE 存档时长作初值（默认 true）
//   weather_path          GdWeather 视觉节点路径（默认 "Weather"，支持兄弟回退）
//
// 信号：s_phase_changed(is_day) / s_phase_name_changed(name) / s_weather_changed(name)

use godot::prelude::*;
use godot::builtin::{GString, NodePath};
use godot::classes::{Engine, INode, Node, Time};
use godot::global::{randf, randf_range};

use super::shaders;
use super::weather::GdWeather;
use crate::state::gdcore::GDCore;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdWeatherManager {
    /// 游戏时间流速：游戏秒 / 现实秒（600 = 1 现实秒过 10 游戏分钟）
    #[var(pub)]
    time_scale: f64,
    /// 一昼夜的游戏秒数
    #[var(pub)]
    day_length: f64,
    /// 初始游戏时刻偏移（游戏秒；21600 = 早 6:00）
    #[var(pub)]
    start_time: f64,
    /// 白天开始小时
    #[var(pub)]
    day_start_hour: f64,
    /// 黑夜开始小时
    #[var(pub)]
    night_start_hour: f64,
    /// 黎明/黄昏时长（小时；黎明 = day_start-twilight .. day_start）
    #[var(pub)]
    twilight_hours: f64,
    /// 下雨概率 0..1（每次判定）
    #[var(pub)]
    rain_chance: f64,
    /// 降雨中雷暴占比 0..1
    #[var(pub)]
    storm_chance: f64,
    /// 黎明遇雾概率 0..1
    #[var(pub)]
    fog_chance: f64,
    /// 天气判定间隔（游戏秒）
    #[var(pub)]
    check_interval: f64,
    /// 单次降雨最长持续（游戏秒，实际随机 0.4..1.0 倍）
    #[var(pub)]
    rain_max_duration: f64,
    /// ready 时自动取 GDCORE 当前存档累计时长作初值（回退引擎进程时长）
    #[var(pub)]
    auto_start_from_ticks: bool,
    /// GdWeather 视觉节点路径（空则跳过视觉下发，纯逻辑运行）
    #[var(pub)]
    weather_path: NodePath,
    /// 效果天气（rain/storm/snow/fog/wind）的密度终值倍率 0..1
    #[var(pub)]
    weather_intensity: f64,

    // ---- 运行时状态 ----
    /// 游戏内总时长（游戏秒；初始值一次传入，此后内部累计）
    total_time: f64,
    /// 是否已接收过初值（set_total_time / 自动取任一）
    initialized: bool,
    /// 当前下发的天气名（去重，避免同天气反复触发渐变）
    current_weather: GString,
    /// 当前相位名（dawn/day/dusk/night）
    phase_name: GString,
    /// 当前白天/黑夜（黎明/黄昏按昼夜语义归属，语义与旧版一致）
    is_day: bool,
    /// 天气判定倒计时（游戏秒）
    check_timer: f64,
    /// 降雨剩余时长（游戏秒；> 0 = 降雨中）
    rain_left: f64,
    /// 晨雾剩余时长（游戏秒；> 0 = 起雾中）
    fog_left: f64,
    /// 视觉节点缓存
    weather_node: Option<Gd<GdWeather>>,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdWeatherManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            time_scale: 600.0,
            day_length: 86400.0,
            start_time: 21600.0,
            day_start_hour: 6.0,
            night_start_hour: 18.0,
            twilight_hours: 1.5,
            rain_chance: 0.25,
            storm_chance: 0.3,
            fog_chance: 0.25,
            check_interval: 3600.0,
            rain_max_duration: 7200.0,
            auto_start_from_ticks: true,
            weather_path: NodePath::from("Weather"),
            weather_intensity: 1.0,
            total_time: 0.0,
            initialized: false,
            current_weather: GString::new(),
            phase_name: GString::new(),
            is_day: true,
            check_timer: 0.0,
            rain_left: 0.0,
            fog_left: 0.0,
            weather_node: None,
            base,
        }
    }

    fn ready(&mut self) {
        // 初始游戏总时长：优先 GDCORE 当前存档累计真实时长（跨开关应用累积），
        // 回退引擎进程运行时长；一次取用，之后内部流转
        if !self.initialized && self.auto_start_from_ticks {
            self.total_time = fetch_play_time_secs().unwrap_or_else(|| {
                Time::singleton().get_ticks_msec() as f64 / 1000.0
            });
            self.initialized = true;
        }
        // 相位计算基于 (total_time + start_time)，这里把 start_time 并入基准
        // 保证 set_total_time 传"累计时长"时也能从 start_time 时刻起算
        self.total_time += self.start_time;
        self.start_time = 0.0;

        self.resolve_weather_node();
        // 立即按当前时刻定相位与天气（ready 初始不发切换信号；
        // current_weather 留空交给 push_weather 设置，避免去重短路视觉下发）
        self.is_day = self.compute_is_day();
        self.phase_name = GString::from(self.phase_name().as_str());
        let base_weather = self.phase_name();
        self.push_weather(&base_weather);
        self.check_timer = self.check_interval.max(1.0);
    }

    fn process(&mut self, delta: f64) {
        // 时间流转：全部在管理器内部累计（外部只在初始化时传一次总时长）
        let game_dt = delta * self.time_scale;
        self.total_time += game_dt;

        // 相位切换检测（黎明/白天/黄昏/黑夜）
        let phase = self.phase_name();
        if phase != self.phase_name.to_string() {
            self.phase_name = GString::from(phase.as_str());
            let arg = self.phase_name.clone().to_variant();
            self.base_mut().emit_signal("s_phase_name_changed", &[arg]);
            let day_now = self.compute_is_day();
            if day_now != self.is_day {
                self.is_day = day_now;
                let arg = day_now.to_variant();
                self.base_mut().emit_signal("s_phase_changed", &[arg]);
            }
            // 黄昏/黎明结束：晨雾随之消散（雾只在黎明存在）
            if phase != "dawn" && self.fog_left > 0.0 {
                self.fog_left = 0.0;
            }
            // 非降雨/雾状态跟随相位换基准天气
            if self.rain_left <= 0.0 && self.fog_left <= 0.0 {
                self.push_weather(&phase);
            }
        }

        // 降雨倒计时：到点回落（优先级让给雾，再让给相位基准）
        if self.rain_left > 0.0 {
            self.rain_left -= game_dt;
            if self.rain_left <= 0.0 {
                self.rain_left = 0.0;
                if self.fog_left > 0.0 {
                    self.push_weather("fog");
                } else {
                    let name = self.phase_name();
                    self.push_weather(&name);
                }
            }
            return;
        }

        // 晨雾倒计时：到点回到相位基准
        if self.fog_left > 0.0 {
            self.fog_left -= game_dt;
            if self.fog_left <= 0.0 {
                self.fog_left = 0.0;
                let name = self.phase_name();
                self.push_weather(&name);
            }
            return;
        }

        // 天气判定：每 check_interval（游戏秒）roll 一次（降雨 > 晨雾）
        self.check_timer -= game_dt;
        if self.check_timer > 0.0 {
            return;
        }
        self.check_timer = self.check_interval.max(1.0);
        if self.rain_chance > 0.0 && randf() < self.rain_chance {
            self.rain_left = randf_range(0.4, 1.0) * self.rain_max_duration.max(1.0);
            if self.storm_chance > 0.0 && randf() < self.storm_chance {
                self.push_weather("storm");
            } else {
                self.push_weather("rain");
            }
        } else if self.phase_name.to_string() == "dawn"
            && self.fog_chance > 0.0
            && randf() < self.fog_chance
        {
            // 晨雾：从现在起到黎明结束
            self.fog_left = self.seconds_until_day_start();
            self.push_weather("fog");
        }
    }
}

#[godot_api]
impl GdWeatherManager {
    /// 昼夜切换信号（参数：是否白天；黎明/黄昏边界触发）
    #[signal]
    fn s_phase_changed(is_day: bool);

    /// 相位切换信号（参数：相位名 dawn/day/dusk/night）
    #[signal]
    fn s_phase_name_changed(name: GString);

    /// 天气切换信号（参数：天气名，含基准相位与 rain/storm/fog）
    #[signal]
    fn s_weather_changed(name: GString);

    /// 传入游戏运行总时长（游戏秒）作为初始时间基准——只需在场景初始化时
    /// 调一次（如读存档的累计时长）；之后时间流转在管理器内部自行计算。
    /// 传 0 = 从 start_time 配置的时刻开始。会清空进行中的降雨/晨雾
    /// （瞬态天气与旧时间绑定，换时间基准后无意义）。
    #[func]
    pub fn set_total_time(&mut self, seconds: f64) {
        self.total_time = seconds.max(0.0) + self.start_time;
        self.start_time = 0.0;
        self.initialized = true;
        // 瞬态天气清零；强制重发基准（视觉可能还停在降雨/雾，去重会漏发）
        self.rain_left = 0.0;
        self.fog_left = 0.0;
        self.current_weather = GString::new();
        let day_now = self.compute_is_day();
        if day_now != self.is_day {
            self.is_day = day_now;
            let arg = day_now.to_variant();
            self.base_mut().emit_signal("s_phase_changed", &[arg]);
        }
        let phase = self.phase_name();
        if phase != self.phase_name.to_string() {
            self.phase_name = GString::from(phase.as_str());
            let arg = self.phase_name.clone().to_variant();
            self.base_mut().emit_signal("s_phase_name_changed", &[arg]);
        }
        self.push_weather(&phase);
    }

    /// 游戏内总时长（游戏秒，含 start_time 偏移）
    #[func]
    pub fn get_game_time(&self) -> f64 {
        self.total_time
    }

    /// 当前游戏内小时数 0..24（浮点）
    #[func]
    pub fn get_hour(&self) -> f64 {
        self.hour_of_day()
    }

    /// 当前是否白天（语义与昼夜信号一致：day_start .. night_start）
    #[func]
    pub fn is_daytime(&self) -> bool {
        self.is_day
    }

    /// 当前相位名（dawn/day/dusk/night）
    #[func]
    pub fn get_phase_name(&self) -> GString {
        self.phase_name.clone()
    }

    /// 当前下发天气名（相位基准或 rain/storm/fog）
    #[func]
    pub fn get_current_weather(&self) -> GString {
        self.current_weather.clone()
    }

    // ---------- 内部实现 ----------

    /// 当前游戏内小时（0..24）：一昼夜 day_length 秒映射 24 小时
    fn hour_of_day(&self) -> f64 {
        let cycle = self.day_length.max(1.0);
        let phase = (self.total_time % cycle) / cycle;
        phase * 24.0
    }

    /// 昼夜判定：day_start .. night_start 白天
    fn compute_is_day(&self) -> bool {
        let h = self.hour_of_day();
        h >= self.day_start_hour && h < self.night_start_hour
    }

    /// 当前相位名：黎明/白天/黄昏/黑夜
    fn phase_name(&self) -> String {
        let h = self.hour_of_day();
        let twilight = self.twilight_hours.max(0.0);
        if h >= self.day_start_hour && h < self.night_start_hour {
            // 白天侧：黄昏 = night_start 前 twilight 小时
            if h >= self.night_start_hour - twilight {
                "dusk".to_string()
            } else {
                "day".to_string()
            }
        } else {
            // 黑夜侧：黎明 = day_start 前 twilight 小时（跨午夜回绕）
            if twilight > 0.0 && self.hours_until_day_start() <= twilight {
                "dawn".to_string()
            } else {
                "night".to_string()
            }
        }
    }

    /// 距白天开始（day_start_hour）的游戏小时数（0..24，含跨午夜回绕）
    fn hours_until_day_start(&self) -> f64 {
        let h = self.hour_of_day();
        let diff = self.day_start_hour - h;
        if diff <= 0.0 {
            diff + 24.0
        } else {
            diff
        }
    }

    /// 距白天开始的剩余游戏秒数（晨雾持续到黎明结束）
    fn seconds_until_day_start(&self) -> f64 {
        self.hours_until_day_start() / 24.0 * self.day_length.max(1.0)
    }

    /// 解析视觉节点：weather_path 指向的 GdWeather。解析顺序：显式路径
    /// （子节点/绝对路径）→ 未命中且为相对名时按兄弟节点回退（管理器与
    /// 视觉节点平级是 demo 常见摆法，同 GdHurtbox 找 Health 的宽容策略）。
    /// 全部未命中置 None（纯逻辑运行，不发视觉）。
    fn resolve_weather_node(&mut self) {
        self.weather_node = None;
        if self.weather_path.is_empty() {
            return;
        }
        let path = self.weather_path.clone();
        let node = self
            .base()
            .get_node_or_null(&path)
            // 兄弟回退："Weather" → "../Weather"
            .or_else(|| {
                let as_str = path.to_string();
                if as_str.is_empty() || as_str.contains('/') || as_str.contains(':') {
                    None
                } else {
                    let sibling = NodePath::from(format!("../{}", as_str).as_str());
                    self.base().get_node_or_null(&sibling)
                }
            });
        if let Some(node) = node {
            if let Ok(w) = node.try_cast::<GdWeather>() {
                self.weather_node = Some(w);
            }
        }
    }

    /// 下发天气（去重 + 信号 + 视觉节点）。
    /// 双通道路由：fx 效果名（rain/storm/snow/fog/wind）→ set_effect
    /// （density 门控渐变，雨滴数量真实增减）；tint 相位名（dawn/day/
    /// dusk/night）→ set_tint（调色板缓动，亮度连续）+ 清效果通道。
    fn push_weather(&mut self, name: &str) {
        if self.current_weather.to_string() == name {
            return;
        }
        let gs = GString::from(name);
        self.current_weather = gs.clone();
        let arg = gs.clone().to_variant();
        self.base_mut().emit_signal("s_weather_changed", &[arg]);
        if let Some(mut w) = self.weather_node.take() {
            if w.is_instance_valid() {
                if shaders::is_fx_weather(name) {
                    w.bind_mut().set_intensity(self.weather_intensity);
                    w.bind_mut().set_effect(gs);
                } else {
                    w.bind_mut().set_tint(gs);
                    w.bind_mut().set_effect(GString::from("clear"));
                }
                self.weather_node = Some(w);
            }
        }
    }
}

/// 取 GDCORE 当前存档的累计游玩秒数（单例不可用返回 None）
fn fetch_play_time_secs() -> Option<f64> {
    let singleton = Engine::singleton().get_singleton(&StringName::from("GDCORE"))?;
    let core = singleton.try_cast::<GDCore>().ok()?;
    Some(core.bind().get_play_time())
}
