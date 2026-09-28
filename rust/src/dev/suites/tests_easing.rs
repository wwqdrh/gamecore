// suite: easing - 缓动函数纯数学测试
// 覆盖: 31 种缓动的边界值（x=0 -> 0, x=1 -> 1）、代表性取值、值域范围

use crate::anim::easing::Easing;
use crate::dev::framework::TestContext;
use crate::dev::framework;

const EPS: f64 = 1e-3;

const ALL: [Easing; 31] = [
	Easing::SineIn, Easing::SineOut, Easing::SineInOut,
	Easing::QuadIn, Easing::QuadOut, Easing::QuadInOut,
	Easing::CubicIn, Easing::CubicOut, Easing::CubicInOut,
	Easing::QuartIn, Easing::QuartOut, Easing::QuartInOut,
	Easing::QuintIn, Easing::QuintOut, Easing::QuintInOut,
	Easing::ExpoIn, Easing::ExpoOut, Easing::ExpoInOut,
	Easing::CircIn, Easing::CircOut, Easing::CircInOut,
	Easing::BackIn, Easing::BackOut, Easing::BackInOut,
	Easing::ElasticIn, Easing::ElasticOut, Easing::ElasticInOut,
	Easing::BounceIn, Easing::BounceOut, Easing::BounceInOut,
	Easing::Linear,
];

fn endpoints(ctx: &mut TestContext) {
	for easing in ALL {
		let name = format!("{easing:?}");
		let at_zero = easing.get_progress(0.0) as f64;
		let at_one = easing.get_progress(1.0) as f64;

		ctx.assert_near(at_zero, 0.0, EPS, format!("{name} 在 x=0 处应为 0"));
		ctx.assert_near(at_one, 1.0, EPS, format!("{name} 在 x=1 处应为 1"));
	}
}

fn value_range(ctx: &mut TestContext) {
	for easing in ALL {
		let name = format!("{easing:?}");
		let mut x = 0.0f32;
		while x <= 1.0 {
			let p = easing.get_progress(x) as f64;
			// Back/Elastic 允许过冲/回弹
			let overshot = matches!(
				easing,
				Easing::BackIn | Easing::BackOut | Easing::BackInOut
					| Easing::ElasticIn | Easing::ElasticOut | Easing::ElasticInOut
			);
			if overshot {
				ctx.assert_true(p > -3.0 && p < 3.0, format!("{name} 在 x={x} 处应在合理范围内"));
			} else {
				ctx.assert_true(
					p >= -EPS && p <= 1.0 + EPS,
					format!("{name} 在 x={x} 处应处于 [0,1] (got {p})"),
				);
			}
			x += 0.05;
		}
	}
}

fn known_values(ctx: &mut TestContext) {
	// 线性恒等
	ctx.assert_near(Easing::Linear.get_progress(0.4) as f64, 0.4, EPS, "Linear 应为恒等映射");
	// 二次缓入: x^2
	ctx.assert_near(Easing::QuadIn.get_progress(0.5) as f64, 0.25, EPS, "QuadIn(0.5) 应为 0.25");
	// 二次缓出: 1-(1-x)^2
	ctx.assert_near(Easing::QuadOut.get_progress(0.5) as f64, 0.75, EPS, "QuadOut(0.5) 应为 0.75");
	// 三次缓入: x^3
	ctx.assert_near(Easing::CubicIn.get_progress(0.5) as f64, 0.125, EPS, "CubicIn(0.5) 应为 0.125");
	// 正弦缓出
	ctx.assert_near(
		Easing::SineOut.get_progress(0.5) as f64,
		(0.5f64 * std::f64::consts::FRAC_PI_2).sin(),
		EPS,
		"SineOut(0.5) 应为 sin(pi/4)",
	);
	// 枚举值映射（与 C++ GdJuice::EASING 对齐）
	ctx.assert_eq(Easing::Linear as i32, 30, "Linear 枚举值应为 30");
	ctx.assert_eq(Easing::ElasticOut as i32, 25, "ElasticOut 枚举值应为 25");
}

pub fn register() {
	framework::register("easing", "endpoints", endpoints);
	framework::register("easing", "value_range", value_range);
	framework::register("easing", "known_values", known_values);
}
