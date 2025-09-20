// src/guitar/left_finger.rs

use crate::guitar::guitar_instance::Guitar;
use crate::guitar::guitar_string::GuitarString;
use crate::utils::util_methods::lerp_by_fret_scalar;
use std::f64;

// 按弦状态枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PressState {
    Open = 0,
    Pressed = 1,
    Barre = 2,
    PartialBarre2Strings = 3,
    PartialBarre3Strings = 4,
    Keep = 5,
}

impl PressState {
    pub fn from_str(state: &str) -> Self {
        match state {
            "Open" => PressState::Open,
            "Pressed" => PressState::Pressed,
            "Barre" => PressState::Barre,
            "Partial_barre_2_strings" => PressState::PartialBarre2Strings,
            "Partial_barre_3_strings" => PressState::PartialBarre3Strings,
            "Keep" => PressState::Keep,
            _ => PressState::Open, // 默认值
        }
    }

    // 添加这个方法来将 PressState 转换为字符串
    pub fn to_str(&self) -> &'static str {
        match self {
            PressState::Open => "Open",
            PressState::Pressed => "Pressed",
            PressState::Barre => "Barre",
            PressState::PartialBarre2Strings => "Partial_barre_2_strings",
            PressState::PartialBarre3Strings => "Partial_barre_3_strings",
            PressState::Keep => "Keep",
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

// 手指枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Finger {
    Empty = -1,
    Thumb = 0,
    Index = 1,
    Middle = 2,
    Ring = 3,
    Pinky = 4,
}

impl Finger {
    pub fn from_index(index: i32) -> Self {
        match index {
            -1 => Finger::Empty,
            0 => Finger::Thumb,
            1 => Finger::Index,
            2 => Finger::Middle,
            3 => Finger::Ring,
            4 => Finger::Pinky,
            _ => Finger::Empty, // 默认值
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }

    pub fn name(&self) -> &'static str {
        match self {
            Finger::Empty => "Empty",
            Finger::Thumb => "Thumb",
            Finger::Index => "Index",
            Finger::Middle => "Middle",
            Finger::Ring => "Ring",
            Finger::Pinky => "Pinky",
        }
    }
}
#[derive(Debug, Clone)]
pub struct LeftFinger {
    pub finger_index: i32,
    pub finger_name: String,
    pub string_index: i32,
    pub fret: i32,
    pub press: PressState,
}

impl LeftFinger {
    /// 创建新的 LeftFinger 实例
    ///
    /// # 参数
    /// * `finger_index` - 手指索引
    /// * `guitar_string` - 手指所在的吉他弦
    /// * `fret` - 品数，默认为 1
    /// * `press` - 按弦状态，默认为 "Open"
    pub fn new(finger_index: i32, guitar_string: &GuitarString, fret: i32, press: &str) -> Self {
        let finger = Finger::from_index(finger_index as i32);

        LeftFinger {
            finger_index,
            finger_name: finger.name().to_string(),
            string_index: guitar_string.get_string_index(),
            fret,
            press: PressState::from_str(press),
        }
    }

    /// 获取手指名称
    pub fn get_finger_name(&self) -> &str {
        &self.finger_name
    }

    /// 获取手指索引
    pub fn get_finger_index(&self) -> i32 {
        self.finger_index
    }

    /// 输出手指信息
    pub fn output(&self) {
        let press_state_name = match self.press {
            PressState::Open => "Open",
            PressState::Pressed => "Pressed",
            PressState::Barre => "Barre",
            PressState::PartialBarre2Strings => "Partial_barre_2_strings",
            PressState::PartialBarre3Strings => "Partial_barre_3_strings",
            PressState::Keep => "Keep",
        };

        println!(
            "{} | {} string | {} fret | {}",
            self.finger_index, self.string_index, self.fret, press_state_name
        );
    }

    /// 计算到目标手指的距离
    pub fn distance_to(&self, guitar: &Guitar, target_finger: &LeftFinger) -> f64 {
        let finger_string_distance = if self.string_index != target_finger.string_index {
            (self.string_index as f64 - target_finger.string_index as f64).abs()
                * guitar.get_string_distance()
        } else {
            0.0
        };

        let finger_fret_distance = if self.fret != target_finger.fret {
            self.fret_distance_to(guitar, target_finger)
        } else {
            0.0
        };

        (finger_string_distance.powi(2) + finger_fret_distance.powi(2)).sqrt()
    }

    /// 计算品数距离
    pub fn fret_distance_to(&self, guitar: &Guitar, target_finger: &LeftFinger) -> f64 {
        if self.fret == target_finger.fret {
            0.0
        } else {
            let start_fret_position = lerp_by_fret_scalar(0.0, 0.5, self.fret as f64);
            let end_fret_position = lerp_by_fret_scalar(0.0, 0.5, target_finger.fret as f64);
            guitar.get_full_string() * (end_fret_position - start_fret_position).abs()
        }
    }

    // Getter 方法
    pub fn string_index(&self) -> i32 {
        self.string_index
    }

    pub fn fret(&self) -> i32 {
        self.fret
    }

    pub fn press(&self) -> PressState {
        self.press
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FingerPosition {
    pub string_index: i32,
    pub fret: i32,
    pub finger: i32, // 1-4, -1和0表示不按弦
}
