// left_hand_recorder.rs
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::vec::Vec;

// 假设已存在的模块和结构体
use crate::hand::left_hand::LeftHand;
use crate::midi::midi_to_note::{MidiProcessor, TempoChange};

#[derive(Debug, Serialize, Deserialize)]
pub struct FingerInfo {
    pub string_index: i32,
    pub fret: i32,
    pub press: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordedFinger {
    pub finger_index: i32,
    pub finger_info: FingerInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordedLeftHand {
    pub real_tick: f64,
    pub frame: f64,
    pub left_hand: Vec<RecordedFinger>,
    pub use_barre: bool,
    pub hand_position: i32,
}
#[derive(Debug, Clone)]
pub struct LeftHandRecorder {
    pub hand_pose_list: Vec<LeftHand>,
    pub current_entropy: f64,
    pub entropies: Vec<f64>,
    pub real_ticks: Vec<f64>,
}

impl LeftHandRecorder {
    /// 创建新的左手记录器
    pub fn new() -> Self {
        LeftHandRecorder {
            hand_pose_list: Vec::new(),
            current_entropy: 0.0,
            entropies: Vec::new(),
            real_ticks: Vec::new(),
        }
    }

    /// 从现有数据创建新的左手记录器
    pub fn with_data(
        hand_pose_list: Vec<LeftHand>,
        current_entropy: f64,
        entropies: Vec<f64>,
        real_ticks: Vec<f64>,
    ) -> Self {
        LeftHandRecorder {
            hand_pose_list,
            current_entropy,
            entropies,
            real_ticks,
        }
    }

    /// 添加手部姿态记录
    pub fn add_hand_pose(&mut self, hand_pose: LeftHand, entropy: f64, real_tick: f64) {
        self.hand_pose_list.push(hand_pose);
        self.current_entropy += entropy;
        self.entropies.push(self.current_entropy);
        self.real_ticks.push(real_tick);
    }

    /// 获取当前手部姿态
    pub fn current_hand_pose(&self) -> Option<&LeftHand> {
        self.hand_pose_list.last()
    }

    /// 输出当前手部姿态
    pub fn output_current(&self, show_open_finger: bool) {
        if !self.hand_pose_list.is_empty() {
            println!("Entropy: {}", self.current_entropy);
            println!("real_tick: {}", self.real_ticks.last().unwrap_or(&0.0));

            if let Some(last_pose) = self.hand_pose_list.last() {
                // 注意：这里需要LeftHand实现output方法
                last_pose.output(show_open_finger);
            }
        }
    }

    /// 输出所有手部姿态
    pub fn output(&self, show_open_finger: bool) {
        for i in 1..self.hand_pose_list.len() {
            println!("Entropy: {}", self.entropies[i]);
            println!("real_tick: {}", self.real_ticks[i]);

            // 注意：这里需要LeftHand实现output方法
            self.hand_pose_list[i].output(show_open_finger);
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

            let left_hand = &self.hand_pose_list[i];
            let mut hand_info = Vec::new();

            for finger in &left_hand.fingers {
                let finger_info = FingerInfo {
                    string_index: finger.string_index,
                    fret: finger.fret,
                    press: format!("{:?}", finger.press),
                };

                hand_info.push(RecordedFinger {
                    finger_index: finger.finger_index,
                    finger_info,
                });
            }

            hands_dict.push(RecordedLeftHand {
                real_tick,
                frame,
                left_hand: hand_info,
                use_barre: left_hand.use_barre,
                hand_position: left_hand.hand_position,
            });
        }

        // 统计去重前的数量
        let original_count = hands_dict.len();

        // 根据frame去重，保留每个frame第一次出现的记录
        let mut unique_hands_dict = Vec::new();
        let mut seen_frames = std::collections::HashSet::new();

        for item in hands_dict {
            let frame_key = (item.frame * 1000.0) as i64; // 保留3位小数精度
            if !seen_frames.contains(&frame_key) {
                seen_frames.insert(frame_key);
                unique_hands_dict.push(item);
            }
        }

        // 统计去重后的数量和去重数量
        let unique_count = unique_hands_dict.len();
        let duplicates_removed = original_count - unique_count;

        // 根据frame排序
        unique_hands_dict.sort_by_key(|x| (x.frame * 1000.0) as i64);

        // 输出去重统计信息
        println!(
            "去重统计: 原始记录 {} 条，去重后 {} 条，删除重复记录 {} 条",
            original_count, unique_count, duplicates_removed
        );

        // 写入文件
        let file = File::create(json_file_path)?;
        serde_json::to_writer_pretty(file, &unique_hands_dict)?;

        Ok(())
    }
}

// 为LeftHandRecorder实现默认构造函数
impl Default for LeftHandRecorder {
    fn default() -> Self {
        Self::new()
    }
}
