// 缓动函数库
// 移植自 C++ juice/easing.h，提供 31 种标准缓动函数
// 所有函数输入 x ∈ [0, 1]，输出 ∈ [0, 1]

use std::f32::consts::{PI, TAU};

/// 缓动类型枚举，对应 C++ GdJuice::EASING
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    SineIn = 0,
    SineOut = 1,
    SineInOut = 2,
    QuadIn = 3,
    QuadOut = 4,
    QuadInOut = 5,
    CubicIn = 6,
    CubicOut = 7,
    CubicInOut = 8,
    QuartIn = 9,
    QuartOut = 10,
    QuartInOut = 11,
    QuintIn = 12,
    QuintOut = 13,
    QuintInOut = 14,
    ExpoIn = 15,
    ExpoOut = 16,
    ExpoInOut = 17,
    CircIn = 18,
    CircOut = 19,
    CircInOut = 20,
    BackIn = 21,
    BackOut = 22,
    BackInOut = 23,
    ElasticIn = 24,
    ElasticOut = 25,
    ElasticInOut = 26,
    BounceIn = 27,
    BounceOut = 28,
    BounceInOut = 29,
    Linear = 30,
}

impl Easing {
    /// 根据枚举值获取缓动进度
    pub fn get_progress(self, x: f32) -> f32 {
        match self {
            Self::SineIn => ease_in_sine(x),
            Self::SineOut => ease_out_sine(x),
            Self::SineInOut => ease_in_out_sine(x),
            Self::QuadIn => ease_in_quad(x),
            Self::QuadOut => ease_out_quad(x),
            Self::QuadInOut => ease_in_out_quad(x),
            Self::CubicIn => ease_in_cubic(x),
            Self::CubicOut => ease_out_cubic(x),
            Self::CubicInOut => ease_in_out_cubic(x),
            Self::QuartIn => ease_in_quart(x),
            Self::QuartOut => ease_out_quart(x),
            Self::QuartInOut => ease_in_out_quart(x),
            Self::QuintIn => ease_in_quint(x),
            Self::QuintOut => ease_out_quint(x),
            Self::QuintInOut => ease_in_out_quint(x),
            Self::ExpoIn => ease_in_expo(x),
            Self::ExpoOut => ease_out_expo(x),
            Self::ExpoInOut => ease_in_out_expo(x),
            Self::CircIn => ease_in_circ(x),
            Self::CircOut => ease_out_circ(x),
            Self::CircInOut => ease_in_out_circ(x),
            Self::BackIn => ease_in_back(x),
            Self::BackOut => ease_out_back(x),
            Self::BackInOut => ease_in_out_back(x),
            Self::ElasticIn => ease_in_elastic(x),
            Self::ElasticOut => ease_out_elastic(x),
            Self::ElasticInOut => ease_in_out_elastic(x),
            Self::BounceIn => ease_in_bounce(x),
            Self::BounceOut => ease_out_bounce(x),
            Self::BounceInOut => ease_in_out_bounce(x),
            Self::Linear => ease_linear(x),
        }
    }

    /// 从整数值解析 Easing 枚举
    pub fn from_int(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::SineIn),
            1 => Some(Self::SineOut),
            2 => Some(Self::SineInOut),
            3 => Some(Self::QuadIn),
            4 => Some(Self::QuadOut),
            5 => Some(Self::QuadInOut),
            6 => Some(Self::CubicIn),
            7 => Some(Self::CubicOut),
            8 => Some(Self::CubicInOut),
            9 => Some(Self::QuartIn),
            10 => Some(Self::QuartOut),
            11 => Some(Self::QuartInOut),
            12 => Some(Self::QuintIn),
            13 => Some(Self::QuintOut),
            14 => Some(Self::QuintInOut),
            15 => Some(Self::ExpoIn),
            16 => Some(Self::ExpoOut),
            17 => Some(Self::ExpoInOut),
            18 => Some(Self::CircIn),
            19 => Some(Self::CircOut),
            20 => Some(Self::CircInOut),
            21 => Some(Self::BackIn),
            22 => Some(Self::BackOut),
            23 => Some(Self::BackInOut),
            24 => Some(Self::ElasticIn),
            25 => Some(Self::ElasticOut),
            26 => Some(Self::ElasticInOut),
            27 => Some(Self::BounceIn),
            28 => Some(Self::BounceOut),
            29 => Some(Self::BounceInOut),
            30 => Some(Self::Linear),
            _ => None,
        }
    }
}

// ===== Sine =====

pub fn ease_linear(x: f32) -> f32 {
    x
}

pub fn ease_in_sine(x: f32) -> f32 {
    1.0 - ((x * PI) / 2.0).cos()
}

pub fn ease_out_sine(x: f32) -> f32 {
    1.0 - ((x * PI) / 2.0).sin()
}

pub fn ease_in_out_sine(x: f32) -> f32 {
    -(x * PI).cos() + 1.0 / 2.0
}

// ===== Quad =====

pub fn ease_in_quad(x: f32) -> f32 {
    x * x
}

pub fn ease_out_quad(x: f32) -> f32 {
    1.0 - (1.0 - x) * (1.0 - x)
}

pub fn ease_in_out_quad(x: f32) -> f32 {
    if x < 0.5 {
        2.0 * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
    }
}

// ===== Cubic =====

pub fn ease_in_cubic(x: f32) -> f32 {
    x * x * x
}

pub fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

pub fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
    }
}

// ===== Quart =====

pub fn ease_in_quart(x: f32) -> f32 {
    x * x * x * x
}

pub fn ease_out_quart(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(4)
}

pub fn ease_in_out_quart(x: f32) -> f32 {
    if x < 0.5 {
        8.0 * x * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powi(4) / 2.0
    }
}

// ===== Quint =====

pub fn ease_in_quint(x: f32) -> f32 {
    x * x * x * x * x
}

pub fn ease_out_quint(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(5)
}

pub fn ease_in_out_quint(x: f32) -> f32 {
    if x < 0.5 {
        16.0 * x * x * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powi(5) / 2.0
    }
}

// ===== Expo =====

pub fn ease_in_expo(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    } else {
        2.0_f32.powf(10.0 * x - 10.0)
    }
}

pub fn ease_out_expo(x: f32) -> f32 {
    if x == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * x)
    }
}

pub fn ease_in_out_expo(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    } else if x == 1.0 {
        1.0
    } else if x < 0.5 {
        2.0_f32.powf(20.0 * x - 10.0) / 2.0
    } else {
        (2.0 - 2.0_f32.powf(-20.0 * x + 10.0)) / 2.0
    }
}

// ===== Circ =====

pub fn ease_in_circ(x: f32) -> f32 {
    1.0 - (1.0 - x * x).sqrt()
}

pub fn ease_out_circ(x: f32) -> f32 {
    (1.0 - (x - 1.0).powi(2)).sqrt()
}

pub fn ease_in_out_circ(x: f32) -> f32 {
    if x < 0.5 {
        (1.0 - (2.0 * x).powi(2)).sqrt() / -2.0 + 0.5
    } else {
        ((1.0 - (-2.0 * x + 2.0).powi(2)).sqrt() + 1.0) / 2.0
    }
}

// ===== Back =====

pub fn ease_in_back(x: f32) -> f32 {
    let c1 = 1.70158_f32;
    let c3 = c1 + 1.0;
    c3 * x * x * x - c1 * x * x
}

pub fn ease_out_back(x: f32) -> f32 {
    let c1 = 1.70158_f32;
    let c3 = c1 + 1.0;
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

pub fn ease_in_out_back(x: f32) -> f32 {
    let c1 = 1.70158_f32;
    let c2 = c1 * 1.525;
    if x < 0.5 {
        (2.0 * x).powi(2) * ((c2 + 1.0) * 2.0 * x - c2) / 2.0
    } else {
        ((2.0 * x - 2.0).powi(2) * ((c2 + 1.0) * (x * 2.0 - 2.0) + c2) + 2.0) / 2.0
    }
}

// ===== Elastic =====

pub fn ease_in_elastic(x: f32) -> f32 {
    let c4 = TAU / 3.0;
    if x == 0.0 {
        0.0
    } else if x == 1.0 {
        1.0
    } else {
        -2.0_f32.powf(10.0 * x - 10.0) * ((x * 10.0 - 10.75) * c4).sin()
    }
}

pub fn ease_out_elastic(x: f32) -> f32 {
    let c4 = TAU / 3.0;
    if x == 0.0 {
        0.0
    } else if x == 1.0 {
        1.0
    } else {
        2.0_f32.powf(-10.0 * x) * ((x * 10.0 - 0.75) * c4).sin() + 1.0
    }
}

pub fn ease_in_out_elastic(x: f32) -> f32 {
    let c5 = TAU / 4.5;
    if x == 0.0 {
        0.0
    } else if x == 1.0 {
        1.0
    } else if x < 0.5 {
        -(2.0_f32.powf(20.0 * x - 10.0) * ((20.0 * x - 11.125) * c5).sin()) / 2.0
    } else {
        2.0_f32.powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * c5).sin() / 2.0 + 1.0
    }
}

// ===== Bounce =====

pub fn ease_in_bounce(x: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - x)
}

pub fn ease_out_bounce(x: f32) -> f32 {
    let n1 = 7.5625_f32;
    let d1 = 2.75_f32;

    if x < 1.0 / d1 {
        n1 * x * x
    } else if x < 2.0 / d1 {
        let x = x - 1.5 / d1;
        n1 * x * x + 0.75
    } else if x < 2.5 / d1 {
        let x = x - 2.25 / d1;
        n1 * x * x + 0.9375
    } else {
        let x = x - 2.625 / d1;
        n1 * x * x + 0.984375
    }
}

pub fn ease_in_out_bounce(x: f32) -> f32 {
    if x < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * x)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * x - 1.0)) / 2.0
    }
}
