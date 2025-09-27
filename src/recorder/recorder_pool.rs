// hand_pose_record_pool.rs
use serde_json;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::BufReader;

// 假设已存在的模块和结构体
use crate::guitar::guitar_chord::convert_notes_to_chord;
use crate::guitar::guitar_instance::Guitar;
use crate::hand::left_hand::{LeftHand, convert_chord_to_finger_positions};
use crate::hand::right_hand::RightHand;
use crate::midi::midi_to_note::{MidiProcessor, NoteInfo, TempoChange};
use crate::recorder::left_hand_recorder::LeftHandRecorder;
use crate::recorder::right_hand_recorder::RightHandRecorder;

#[derive(Debug)]
pub enum HandRecorder {
    Left(LeftHandRecorder),
    Right(RightHandRecorder),
}

pub trait HasHandPoseList<T> {
    fn hand_pose_list(&self) -> &Vec<T>;
}

impl HasHandPoseList<LeftHand> for LeftHandRecorder {
    fn hand_pose_list(&self) -> &Vec<LeftHand> {
        &self.hand_pose_list
    }
}

impl HasHandPoseList<RightHand> for RightHandRecorder {
    fn hand_pose_list(&self) -> &Vec<RightHand> {
        &self.hand_pose_list
    }
}

impl HandRecorder {
    /// 获取当前熵值
    pub fn current_entropy(&self) -> f64 {
        match self {
            HandRecorder::Left(recorder) => recorder.current_entropy,
            HandRecorder::Right(recorder) => recorder.current_entropy,
        }
    }

    pub fn entropies(&self) -> &Vec<f64> {
        match self {
            HandRecorder::Left(recorder) => &recorder.entropies,
            HandRecorder::Right(recorder) => &recorder.entropies,
        }
    }

    pub fn real_ticks(&self) -> &Vec<f64> {
        match self {
            HandRecorder::Left(recorder) => &recorder.real_ticks,
            HandRecorder::Right(recorder) => &recorder.real_ticks,
        }
    }

    // 克隆左手列表的方法
    pub fn clone_left_hand_pose_list(&self) -> Vec<LeftHand> {
        match self {
            HandRecorder::Left(recorder) => recorder.hand_pose_list.clone(),
            HandRecorder::Right(_) => Vec::new(),
        }
    }

    // 克隆右手列表的方法
    pub fn clone_right_hand_pose_list(&self) -> Vec<RightHand> {
        match self {
            HandRecorder::Left(_) => Vec::new(),
            HandRecorder::Right(recorder) => recorder.hand_pose_list.clone(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            HandRecorder::Left(recorder) => recorder.hand_pose_list.len(),
            HandRecorder::Right(recorder) => recorder.hand_pose_list.len(),
        }
    }

    // 保存当前记录
    pub fn save(
        &self,
        json_file_path: &str,
        tempo_changes: &Vec<TempoChange>,
        ticks_per_beat: u16,
        fps: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            HandRecorder::Left(recorder) => {
                recorder.save(json_file_path, tempo_changes, ticks_per_beat, fps)
            }
            HandRecorder::Right(recorder) => {
                recorder.save(json_file_path, tempo_changes, ticks_per_beat, fps)
            }
        }
    }

    /// 获取可变引用
    pub fn as_mut(&mut self) -> &mut dyn HasCurrentEntropy {
        match self {
            HandRecorder::Left(recorder) => recorder,
            HandRecorder::Right(recorder) => recorder,
        }
    }

    /// 获取不可变引用
    pub fn as_ref(&self) -> &dyn HasCurrentEntropy {
        match self {
            HandRecorder::Left(recorder) => recorder,
            HandRecorder::Right(recorder) => recorder,
        }
    }
}
/// 用于访问current_entropy属性的trait
pub trait HasCurrentEntropy {
    fn current_entropy(&self) -> f64;
}

impl HasCurrentEntropy for LeftHandRecorder {
    fn current_entropy(&self) -> f64 {
        self.current_entropy
    }
}

impl HasCurrentEntropy for RightHandRecorder {
    fn current_entropy(&self) -> f64 {
        self.current_entropy
    }
}
/// 用于堆操作的包装结构
#[derive(Debug)]
struct RecorderRef {
    recorder: HandRecorder,
}

impl PartialEq for RecorderRef {
    fn eq(&self, other: &Self) -> bool {
        self.recorder.current_entropy() == other.recorder.current_entropy()
    }
}

impl Eq for RecorderRef {}

impl PartialOrd for RecorderRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 为了实现最大堆，我们需要反向比较（熵值大的优先级低）
        other
            .recorder
            .current_entropy()
            .partial_cmp(&self.recorder.current_entropy())
    }
}

impl Ord for RecorderRef {
    fn cmp(&self, other: &Self) -> Ordering {
        // 为了实现最大堆，我们需要反向比较
        other
            .recorder
            .current_entropy()
            .partial_cmp(&self.recorder.current_entropy())
            .unwrap_or(Ordering::Equal)
    }
}

/// 手势记录器池，使用优先队列实现
pub struct HandPoseRecordPool {
    /// 优先队列，存储记录器（最大堆，熵值最大的在顶部）
    recorders: BinaryHeap<RecorderRef>,
    /// 池子的最大容量
    capacity: usize,
    /// 之前的手势记录器池
    pre_recorders: Vec<HandRecorder>,
}

impl HandPoseRecordPool {
    /// 创建新的手势记录器池
    ///
    /// # 参数
    /// * `capacity` - 池子的容量
    pub fn new(capacity: usize) -> Self {
        HandPoseRecordPool {
            recorders: BinaryHeap::with_capacity(capacity),
            capacity,
            pre_recorders: Vec::new(),
        }
    }

    /// 准备记录，将当前池子移动到之前池子，清空当前池子
    pub fn ready_for_record(&mut self) {
        self.pre_recorders = self.recorders.drain().map(|rr| rr.recorder).collect();
        self.recorders.clear();
    }

    /// 添加左手记录器
    pub fn add_left_recorder(&mut self, recorder: LeftHandRecorder) {
        self.add_recorder(HandRecorder::Left(recorder));
    }

    /// 添加右手记录器
    pub fn add_right_recorder(&mut self, recorder: RightHandRecorder) {
        self.add_recorder(HandRecorder::Right(recorder));
    }

    /// 添加新的手势记录器
    fn add_recorder(&mut self, new_recorder: HandRecorder) {
        let entropy = new_recorder.current_entropy();

        // 如果池子未满，直接添加
        if self.recorders.len() < self.capacity {
            self.recorders.push(RecorderRef {
                recorder: new_recorder,
            });
        } else {
            // 如果池子已满，检查新记录器是否应该替换现有记录器
            if let Some(max_entropy_recorder) = self.recorders.peek() {
                // 如果新记录器的熵值小于当前最大熵值，则替换
                if entropy < max_entropy_recorder.recorder.current_entropy() {
                    // 移除熵值最大的记录器
                    self.recorders.pop();
                    // 添加新的记录器
                    self.recorders.push(RecorderRef {
                        recorder: new_recorder,
                    });
                }
            }
        }
    }
    /// 插入新的手势记录器
    pub fn insert_new_hand_pose_recorder(
        &mut self,
        new_recorder: HandRecorder,
        index: Option<usize>,
    ) {
        // 如果索引为None，表示不插入
        let index = match index {
            Some(idx) => idx,
            None => return,
        };

        // 插入新的元素
        let mut recorders_vec: Vec<RecorderRef> = self.recorders.drain().collect();
        recorders_vec.insert(
            index,
            RecorderRef {
                recorder: new_recorder,
            },
        );

        // 如果插入后的大小超过了 capacity，移除最后一个元素
        if recorders_vec.len() > self.capacity {
            recorders_vec.pop();
        }

        // 重新填充堆
        for recorder in recorders_vec {
            self.recorders.push(recorder);
        }
    }

    /// 生成左手记录器
    pub fn generate_left_hand_recorder(
        &mut self,
        guitar_note: &NoteInfo,
        guitar: &Guitar,
        midi_processor: &MidiProcessor,
        current_recorder_num: &mut usize,
        previous_recorder_num: &mut usize,
    ) {
        // 获取音符信息
        let notes = &guitar_note.notes;
        let real_tick = guitar_note.real_tick;

        // 如果没有音符，直接返回
        if notes.is_empty() {
            return;
        }

        // 定义吉他的最低音和最高音
        let min_note = guitar.guitar_strings.last().unwrap().get_base_note();
        let max_note = guitar.guitar_strings.first().unwrap().get_base_note() + 22;

        // 处理音符，确保它们在吉他的音域范围内
        let processed_notes: Vec<i32> = midi_processor.processed_notes(notes, min_note, max_note);

        // 如果处理后没有有效音符，返回
        if processed_notes.is_empty() {
            return;
        }

        // 计算所有可能的和弦，包含音符在吉他上的位置信息
        let chords = convert_notes_to_chord(&processed_notes, guitar);

        // 准备记录，将当前池子移动到之前池子，清空当前池子
        self.ready_for_record();

        // 计算所有可能的按法，包含手指在吉他上的位置信息
        let mut finger_positions_list = Vec::new();
        for chord in chords {
            let possible_finger_positions = convert_chord_to_finger_positions(&chord.positions);
            if !possible_finger_positions.is_empty() {
                finger_positions_list.extend(possible_finger_positions);
            }
        }

        // 如果没有找到合适的按法，记录日志
        if finger_positions_list.is_empty() {
            println!(
                "当前时间是{}，当前notes是{:?},没有找到合适的按法。",
                real_tick, processed_notes
            );
        }

        // 遍历之前的手势记录器和按法列表，生成新的记录器
        for prev_recorder in &self.pre_recorders {
            // 只处理左手记录器
            if let HandRecorder::Left(left_recorder) = prev_recorder {
                // 获取当前手型，如果为空则跳过
                let old_hand = match left_recorder.current_hand_pose() {
                    Some(hand) => hand,
                    None => continue, // 跳过空的手型
                };

                // 遍历按法列表，根据按法生成新的左手对象
                for finger_positions in &finger_positions_list {
                    if let Some((new_fingers, entropy, use_barre)) =
                        old_hand.generate_next_hands(guitar, &finger_positions.positions)
                    {
                        let new_entropy = left_recorder.current_entropy + entropy;
                        let current_size = self.recorders.len();

                        // 检查是否应该插入
                        let current_hand_pose_record_is_largest = current_size > 0
                            && new_entropy
                                >= self
                                    .recorders
                                    .peek()
                                    .map_or(0.0, |recorder| recorder.recorder.current_entropy());

                        // 如果池子已满且新记录器熵值不小于当前最大熵值，则不插入
                        if current_size == self.capacity && current_hand_pose_record_is_largest {
                            continue;
                        }

                        // 创建新的记录器
                        let mut new_hand_pose_list = left_recorder.hand_pose_list.clone();
                        let new_hand = LeftHand::new(
                            new_fingers,
                            use_barre,
                            old_hand.get_max_finger_distance(),
                        );
                        new_hand_pose_list.push(new_hand);

                        let mut new_entropys = left_recorder.entropies.clone();
                        new_entropys.push(new_entropy);

                        let mut new_real_ticks = left_recorder.real_ticks.clone();
                        new_real_ticks.push(real_tick);

                        let new_recorder = LeftHandRecorder::with_data(
                            new_hand_pose_list,
                            new_entropy,
                            new_entropys,
                            new_real_ticks,
                        );

                        // 执行插入操作
                        self.recorders.push(RecorderRef {
                            recorder: HandRecorder::Left(new_recorder),
                        });

                        // 如果插入后超过了容量，移除熵值最大的记录器
                        if self.recorders.len() > self.capacity {
                            self.recorders.pop();
                        }
                    }
                }
            }
        }

        *previous_recorder_num = *current_recorder_num;
        *current_recorder_num = self.len();

        // 如果无法生成正常的按法，就取当前最佳的recoder，然后为它再添加一个最后的按法，清空其它的recorder
        if *current_recorder_num == 0 && !self.pre_recorders.is_empty() {
            println!(
                "前一次记录是{}个，当前生成记录数量为0",
                *previous_recorder_num
            );
            let best_recorder: &HandRecorder = self.get_best_pre_recorder();

            // 克隆最佳记录器的所有数据
            let mut new_hand_pose_list = best_recorder.clone_left_hand_pose_list();
            let new_entropy = best_recorder.current_entropy();
            let mut new_entropys = best_recorder.entropies().clone();
            let mut new_real_ticks = best_recorder.real_ticks().clone();

            // 添加重复的最后一个手型
            if let Some(last_hand) = new_hand_pose_list.last() {
                new_hand_pose_list.push(last_hand.clone());
                new_entropys.push(new_entropy);
                new_real_ticks.push(real_tick);

                let new_recorder = LeftHandRecorder::with_data(
                    new_hand_pose_list,
                    new_entropy,
                    new_entropys,
                    new_real_ticks,
                );

                self.insert_new_hand_pose_recorder(HandRecorder::Left(new_recorder), Some(0));

                println!("重复最后一个手型，清空其它的recorder");
            }
        }
    }

    /// 更新记录器池
    pub fn update_left_handrecorder_pool<F>(
        &mut self,
        guitar: &Guitar,
        notes_map: &Vec<NoteInfo>,
        midi_processor: &MidiProcessor,
        current_recorder_num: &mut usize,
        previous_recorder_num: &mut usize,
        callback: F,
    ) where
        F: Fn(&str),
    {
        let total = notes_map.len();
        for (index, guitar_note) in notes_map.iter().enumerate() {
            self.generate_left_hand_recorder(
                guitar_note,
                guitar,
                midi_processor,
                current_recorder_num,
                previous_recorder_num,
            );

            if index % 10 == 0 {
                callback(&format!("左手生成进度：{}/{}", index, total));
            }
        }
    }

    /// 生成右手记录器
    pub fn generate_right_hand_recorder(
        &mut self,
        item: &serde_json::Value,
        max_string_index: usize,
    ) {
        // 获取real_tick
        let real_tick = match item.get("real_tick").and_then(|v| v.as_f64()) {
            Some(tick) => tick,
            None => return,
        };

        // 获取leftHand数组
        let left_hand = match item.get("leftHand").and_then(|v| v.as_array()) {
            Some(hand) => hand,
            None => return,
        };

        let mut touched_strings = Vec::new();
        let mut lower_strings = Vec::new();

        // 遍历手指信息
        for finger in left_hand {
            if let Some(finger_obj) = finger.as_object() {
                let finger_index = finger_obj
                    .get("fingerIndex")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(-1);
                if let Some(finger_info) = finger_obj.get("fingerInfo").and_then(|v| v.as_object())
                {
                    let press = finger_info
                        .get("press")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let string_index = finger_info
                        .get("stringIndex")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    // 如果是无效手指索引或者按压力度在0到5之间（不包括0和5）
                    if finger_index == -1 || (press > 0.0 && press < 5.0) {
                        touched_strings.push(string_index as i32);
                        if string_index > 2 {
                            lower_strings.push(string_index as i32);
                        }
                    }
                }
            }
        }

        // 如果没有触摸的弦，直接返回
        if touched_strings.is_empty() {
            return;
        }

        self.ready_for_record();

        // 去重
        touched_strings.sort_unstable();
        touched_strings.dedup();

        // 从高到低排序
        touched_strings.sort_unstable_by(|a, b| b.cmp(a));

        // 判断是否允许p指弹两根弦
        let allow_double_p = max_string_index > 3 && lower_strings.len() > 1;
        let all_fingers = if allow_double_p {
            vec![
                "p".to_string(),
                "p".to_string(),
                "i".to_string(),
                "m".to_string(),
                "a".to_string(),
            ]
        } else {
            vec![
                "p".to_string(),
                "i".to_string(),
                "m".to_string(),
                "a".to_string(),
            ]
        };

        let all_strings: Vec<i32> = (0..=max_string_index as i32).collect();

        // 生成可能的右手组合
        let possible_combinations = crate::hand::right_hand::generate_possible_right_hands(
            touched_strings.clone(),
            all_fingers,
            all_strings,
        );

        if possible_combinations.is_empty() {
            println!(
                "当前要拨动的弦是{:?}，没有找到合适的右手拨法。",
                touched_strings
            );
            return;
        }

        // 收集所有需要添加的新记录器，避免在遍历过程中修改self.recorders
        let mut new_recorders = Vec::new();

        // 遍历所有可能的组合和预记录器
        for combination in &possible_combinations {
            for pre_recorder in &self.pre_recorders {
                if let HandRecorder::Right(right_recorder) = pre_recorder {
                    if let Some(last_hand) = right_recorder.hand_pose_list.last() {
                        let used_fingers = match combination.get("usedFingers") {
                            Some(fingers) => match fingers.as_array() {
                                Some(arr) => arr
                                    .iter()
                                    .map(|v| v.as_str().unwrap_or("").to_string())
                                    .collect(),
                                None => Vec::new(),
                            },
                            None => Vec::new(),
                        };

                        let right_finger_positions = match combination.get("rightFingerPositions") {
                            Some(positions) => match positions.as_array() {
                                Some(arr) => {
                                    arr.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()
                                }
                                None => vec![5, 4, 3, 2],
                            },
                            None => vec![5, 4, 3, 2],
                        };

                        // 验证右手姿势
                        let is_valid = last_hand.validate_right_hand(
                            Some(used_fingers.clone()),
                            Some(right_finger_positions.clone()),
                        );

                        if !is_valid {
                            continue;
                        }

                        let is_arpeggio = used_fingers.is_empty();
                        let is_playing_bass = false; // 根据上下文，这里可能需要更准确的判断
                        let right_hand = RightHand::new(
                            used_fingers.clone(),
                            right_finger_positions.clone(),
                            last_hand.used_fingers.clone(),
                            is_arpeggio,
                            is_playing_bass,
                        );

                        let entropy = last_hand.calculate_diff(&right_hand);
                        let new_entropy = right_recorder.current_entropy + entropy;

                        // 创建新的记录器
                        let mut new_recorder = RightHandRecorder::new();
                        new_recorder.hand_pose_list = right_recorder.hand_pose_list.clone();
                        new_recorder.hand_pose_list.push(right_hand);
                        new_recorder.current_entropy = new_entropy;
                        new_recorder.entropies = right_recorder.entropies.clone();
                        new_recorder.entropies.push(new_entropy);
                        new_recorder.real_ticks = right_recorder.real_ticks.clone();
                        new_recorder.real_ticks.push(real_tick);

                        new_recorders.push(new_recorder);
                    }
                }
            }
        }

        // 统一添加所有新记录器
        for new_recorder in new_recorders {
            self.add_right_recorder(new_recorder);
        }
    }
    // 更新右手记录器池
    pub fn update_right_hand_recorder_pool<F>(
        &mut self,
        left_hand_recorder_file: &str,
        max_string_index: usize,
        progress_callback: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(&str),
    {
        // 读取左手记录文件
        let file = File::open(left_hand_recorder_file)?;
        let reader = BufReader::new(file);
        let data: Vec<serde_json::Value> = serde_json::from_reader(reader)?;

        let total_steps = data.len();
        progress_callback(&format!("开始处理右手数据，共 {} 项", total_steps));

        // 遍历数据处理（将来可以在这里添加 eGui 进度条更新逻辑）
        for i in 0..total_steps {
            let item = &data[i];
            self.generate_right_hand_recorder(item, max_string_index);

            // 每处理10项报告一次进度
            if i % 10 == 0 || i == total_steps - 1 {
                progress_callback(&format!("处理进度: {}/{}", i + 1, total_steps));
            }
        }

        progress_callback("右手数据处理完成");
        Ok(())
    }

    /// 获取当前池子的长度
    pub fn len(&self) -> usize {
        self.recorders.len()
    }

    /// 检查当前池子是否为空
    pub fn is_empty(&self) -> bool {
        self.recorders.is_empty()
    }

    /// 获取当前池子的容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 获取当前池子中的最大熵值
    pub fn max_entropy(&self) -> Option<f64> {
        self.recorders
            .peek()
            .map(|rr| rr.recorder.current_entropy())
    }

    /// 获取当前池子中的最小熵值
    pub fn min_entropy(&self) -> Option<f64> {
        self.recorders
            .iter()
            .map(|rr| rr.recorder.current_entropy())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
    }

    /// 获取池子中所有记录器的熵值
    pub fn get_all_entropies(&self) -> Vec<f64> {
        self.recorders
            .iter()
            .map(|rr| rr.recorder.current_entropy())
            .collect()
    }

    /// 获取所有记录器（消耗所有权）
    pub fn into_recorders(self) -> Vec<HandRecorder> {
        self.recorders.into_iter().map(|rr| rr.recorder).collect()
    }

    pub fn get_best_recorder(&self) -> &HandRecorder {
        &self
            .recorders
            .iter()
            .min_by(|a, b| {
                a.recorder
                    .current_entropy()
                    .partial_cmp(&b.recorder.current_entropy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("Recorder pool should not be empty")
            .recorder
    }

    /// 获取之前记录器中最好的一个（熵值最小的）
    pub fn get_best_pre_recorder(&self) -> &HandRecorder {
        self.pre_recorders
            .iter()
            .min_by(|a, b| {
                a.current_entropy()
                    .partial_cmp(&b.current_entropy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("Previous recorders should not be empty")
    }
}

// 为HandPoseRecordPool实现默认构造函数
impl Default for HandPoseRecordPool {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hand_pose_record_pool_creation() {
        let pool = HandPoseRecordPool::new(5);
        assert_eq!(pool.capacity(), 5);
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }
}
