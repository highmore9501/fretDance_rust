// right_hand_recorder.rs
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::vec::Vec;

// 假设已存在的模块和结构体
use crate::hand::right_hand::RightHand;
use crate::midi::midi_to_note::{MidiProcessor, TempoChange};

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordedRightHandInfo {
    pub used_fingers: Vec<String>,
    pub right_finger_positions: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordedRightHand {
    pub real_tick: f64,
    pub frame: f64,
    pub right_hand: RecordedRightHandInfo,
}
#[derive(Debug, Clone)]
pub struct RightHandRecorder {
    pub hand_pose_list: Vec<RightHand>,
    pub current_entropy: f64,
    pub entropies: Vec<f64>,
    pub real_ticks: Vec<f64>,
}

impl RightHandRecorder {
    /// 创建新的右手记录器
    pub fn new() -> Self {
        RightHandRecorder {
            hand_pose_list: Vec::new(),
            current_entropy: 0.0,
            entropies: Vec::new(),
            real_ticks: Vec::new(),
        }
    }

    /// 添加手部姿态记录
    pub fn add_hand_pose(&mut self, hand_pose: RightHand, entropy: f64, real_tick: f64) {
        self.hand_pose_list.push(hand_pose);
        self.current_entropy += entropy;
        self.entropies.push(self.current_entropy);
        self.real_ticks.push(real_tick);
    }

    /// 获取当前手部姿态
    pub fn current_hand_pose(&self) -> Option<&RightHand> {
        self.hand_pose_list.last()
    }

    /// 输出所有手部姿态
    pub fn output(&self) {
        println!("Entropy: {}", self.current_entropy);
        for i in 1..self.hand_pose_list.len() {
            println!("Entropy: {}", self.entropies[i]);
            println!("real_tick: {}", self.real_ticks[i]);
            self.hand_pose_list[i].output();
        }
    }

    /// 保存记录到JSON文件
    pub fn save(
        &self,
        json_file_path: &str,
        tempo_changes: &Vec<TempoChange>, // (time, tempo) tuples
        ticks_per_beat: u16,
        fps: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut hands_dict = Vec::new();
        let midi_processor = MidiProcessor::new();

        for i in 1..self.hand_pose_list.len() {
            let real_tick = self.real_ticks[i];
            let frame =
                midi_processor.calculate_frame(tempo_changes, ticks_per_beat, fps, real_tick);

            let right_hand = &self.hand_pose_list[i];

            let hand_info = RecordedRightHandInfo {
                used_fingers: right_hand.used_fingers.clone(),
                right_finger_positions: right_hand.right_finger_positions.clone(),
            };

            hands_dict.push(RecordedRightHand {
                real_tick,
                frame,
                right_hand: hand_info,
            });
        }

        // 写入文件
        let file = File::create(json_file_path)?;
        serde_json::to_writer_pretty(file, &hands_dict)?;

        Ok(())
    }
}

// 为RightHandRecorder实现默认构造函数
impl Default for RightHandRecorder {
    fn default() -> Self {
        Self::new()
    }
}
