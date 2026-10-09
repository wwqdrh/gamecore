// GdDisplayFit - 自适应分辨率（content scale）配置器
//
// 解决「窗口缩放/全屏更高分辨率时 UI 不跟随放大」：以配置的设计基准分辨率
// （如 1920x1080）为逻辑坐标系，引擎把整个画布（GML UI + 字体 + 游戏世界）
// 等比缩放到真实窗口大小——UI 布局代码零改动即获得自适应能力。
//
// 引擎机制：根 Window 的 content_scale 系列属性
//   · content_scale_mode = CANVAS_ITEMS：按画布项缩放（矢量/字体清晰）
//   · content_scale_aspect = KEEP：非基准宽高比时黑边补齐，不变形不裁切
//   · content_scale_size = 设计基准分辨率（逻辑坐标上限）
//
// 配置驱动（game_config.json 的 display 块，缺省 = 不启用，框架零影响）：
//   "display": {
//     "enabled": true,
//     "base_width": 1920, "base_height": 1080,
//     "mode": "canvas_items",     // canvas_items / viewport / disabled
//     "aspect": "keep"            // keep / expand / keep_width / keep_height
//   }
//
// 调用时机：
//   · GdSceneRoot::ready 自动调用（管理器场景流：title→main 全覆盖）
//   · 直开 GML 组合根等无 GdSceneRoot 的启动路径：GDScript 调
//     GdDisplayFit.apply_display_fit(false)
//
// headless 守卫：-s 无头测试/CI 下窗口 64x64 且合成输入事件坐标不经过
// content scale 变换，启用会破坏输入路由测试——自动跳过（force=true 可强制，
// 仅供建议性测试断言属性生效用）。
// 幂等：重复调用重复写同值，无副作用。

use godot::prelude::*;
use godot::builtin::{GString, Variant, VariantType};
use godot::classes::{DisplayServer, Engine, IRefCounted, SceneTree, Window};
use godot::classes::window::{ContentScaleAspect, ContentScaleMode};

use super::config_manager::ensure_config_manager;

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdDisplayFit {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GdDisplayFit {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GdDisplayFit {
    /// 应用自适应分辨率配置（读 game_config.json 的 display 块）。
    /// headless 环境默认跳过；force=true 强制应用（测试断言用）。
    /// 返回是否实际应用。
    #[func]
    pub fn apply_display_fit(force: bool) -> bool {
        Self::apply_from_config(force)
    }
}

impl GdDisplayFit {
    /// 从 GdConfigManager 读取 display 配置并应用到根 Window
    pub fn apply_from_config(force: bool) -> bool {
        // headless 守卫：-s 无头模式跳过（force 可越过，供测试）
        if !force && DisplayServer::singleton().get_name() == GString::from("headless") {
            return false;
        }

        // 配置缺失/未启用 = 框架零影响（保持引擎默认行为）
        let Some(mut config) = ensure_config_manager() else {
            return false;
        };
        let display_var = config.call(
            &StringName::from("get_config"),
            &[GString::from("display").to_variant(), Variant::nil()],
        );
        if display_var.get_type() != VariantType::DICTIONARY {
            return false;
        }
        let display = display_var.to::<VarDictionary>();
        if !bool::from_variant(&display.get_or_nil(&"enabled".to_variant())) {
            return false;
        }

        // 设计基准分辨率（缺省 1920x1080，与 project.godot 窗口基准一致）
        let base_w = int_config(&display, "base_width", 1920);
        let base_h = int_config(&display, "base_height", 1080);
        let mode = str_variant(&display, "mode", "canvas_items");
        let aspect = str_variant(&display, "aspect", "keep");

        // 根 Window：Engine 主循环 -> SceneTree -> root
        let Some(main_loop) = Engine::singleton().get_main_loop() else {
            return false;
        };
        let Ok(tree) = main_loop.try_cast::<SceneTree>() else {
            return false;
        };
        let Some(mut root) = tree.get_root() else {
            return false;
        };

        root.set_content_scale_size(Vector2i::new(base_w as i32, base_h as i32));
        root.set_content_scale_mode(parse_mode(&mode));
        root.set_content_scale_aspect(parse_aspect(&aspect));

        godot_print!(
            "[GdDisplayFit] 自适应分辨率已启用: base={}x{} mode={} aspect={}",
            base_w,
            base_h,
            mode,
            aspect
        );
        true
    }
}

/// 读取字符串配置项（缺省回退）
fn str_variant(dict: &VarDictionary, key: &str, default: &str) -> String {
    let v = dict.get_or_nil(&key.to_variant());
    if v.get_type() == VariantType::STRING {
        v.to::<GString>().to_string()
    } else {
        default.to_string()
    }
}

/// 读取整数配置项（缺省回退）
fn int_config(dict: &VarDictionary, key: &str, default: i64) -> i64 {
    let v = dict.get_or_nil(&key.to_variant());
    if v.get_type() == VariantType::INT {
        v.to::<i64>()
    } else {
        default
    }
}

/// mode 字符串 -> ContentScaleMode（未知值回退 canvas_items）
fn parse_mode(mode: &str) -> ContentScaleMode {
    match mode {
        "disabled" => ContentScaleMode::DISABLED,
        "viewport" => ContentScaleMode::VIEWPORT,
        _ => ContentScaleMode::CANVAS_ITEMS,
    }
}

/// aspect 字符串 -> ContentScaleAspect（未知值回退 keep）
fn parse_aspect(aspect: &str) -> ContentScaleAspect {
    match aspect {
        "expand" => ContentScaleAspect::EXPAND,
        "keep_width" => ContentScaleAspect::KEEP_WIDTH,
        "keep_height" => ContentScaleAspect::KEEP_HEIGHT,
        _ => ContentScaleAspect::KEEP,
    }
}
