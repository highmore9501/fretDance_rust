// hand_pose_record_pool.rs
use serde_json;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use crate::guitar::guitar_chord::convert_notes_to_chord;
use crate::guitar::guitar_instance::Guitar;
use crate::hand::left_finger::PressState;
use crate::hand::left_hand::{LeftHand, convert_chord_to_finger_positions};
use crate::hand::right_hand::{RightHand, RightHandCombination};
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

/// 无法处理的音符信息结构体
#[derive(Debug, Clone)]
pub struct UnprocessableNoteInfo {
    pub real_tick: f64,
    pub notes: Vec<i32>,
    pub reason: String,
}

/// 手势记录器池，使用优先队列实现
pub struct HandPoseRecordPool {
    /// 优先队列，存储记录器（最大堆，熵值最大的在顶部）
    recorders: BinaryHeap<RecorderRef>,
    /// 池子的最大容量
    capacity: usize,
    /// 之前的手势记录器池
    pre_recorders: Vec<HandRecorder>,
    /// 无法处理的音符组合列表
    unprocessable_notes: Vec<UnprocessableNoteInfo>,
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
            unprocessable_notes: Vec::new(),
        }
    }

    /// 准备记录，将当前池子移动到之前池子，清空当前池子
    pub fn ready_for_record(&mut self) {
        self.pre_recorders = self.recorders.drain().map(|rr| rr.recorder).collect();
        self.recorders.clear();
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
            // 保存无法处理的音符组合
            self.unprocessable_notes.push(UnprocessableNoteInfo {
                real_tick,
                notes: processed_notes.clone(),
                reason: "没有找到合适的按法".to_string(),
            });
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

    /// 获取无法处理的音符组合列表
    pub fn get_unprocessable_notes(&self) -> &Vec<UnprocessableNoteInfo> {
        &self.unprocessable_notes
    }

    /// 获取无法处理的音符组合列表（可变引用）
    pub fn get_unprocessable_notes_mut(&mut self) -> &mut Vec<UnprocessableNoteInfo> {
        &mut self.unprocessable_notes
    }

    /// 清空无法处理的音符组合列表
    pub fn clear_unprocessable_notes(&mut self) {
        self.unprocessable_notes.clear();
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
        let total_steps = notes_map.len();
        let start_time = Instant::now();
        let current_time = Instant::now();
        for i in 0..total_steps {
            let guitar_note = &notes_map[i];
            self.generate_left_hand_recorder(
                guitar_note,
                guitar,
                midi_processor,
                current_recorder_num,
                previous_recorder_num,
            );

            // 每处理10项报告一次进度
            if i % 10 == 0 || i == total_steps - 1 {
                let elapsed = current_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    (i + 1) as f64 / elapsed
                } else {
                    0.0
                };
                callback(&format!(
                    "处理进度: {}/{} ({:.2} step/秒)",
                    i + 1,
                    total_steps,
                    speed
                ));
            }
        }

        let end_time = Instant::now();
        callback(&format!(
            "左手数据处理完成，一共费时：{:} 秒",
            end_time.duration_since(start_time).as_secs_f64()
        ));
    }

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
        let left_hand = match item.get("left_hand").and_then(|v| v.as_array()) {
            Some(hand) => hand,
            None => return,
        };

        let mut touched_strings = Vec::new();

        // 遍历手指信息
        for finger in left_hand {
            if let Some(finger_obj) = finger.as_object() {
                // 先检查手指索引，如果为-1则直接跳过
                let finger_index = finger_obj
                    .get("finger_index")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(-1);
                if finger_index == -1 {
                    continue;
                }

                if let Some(finger_info) = finger_obj.get("finger_info").and_then(|v| v.as_object())
                {
                    // 获取按弦类型
                    let press_value = match finger_info.get("press").and_then(|v| v.as_str()) {
                        Some(value) => value,
                        None => {
                            println!("Missing press value in finger info：{:?}", finger_info);
                            continue;
                        }
                    };

                    let press = PressState::from_str(&press_value).to_i32();

                    // 检查按弦类型条件（在0到5之间，不包括0和5）
                    if press <= 0 || press >= 5 {
                        continue;
                    }

                    // 只有当手指索引和按弦类型都符合条件时才处理弦索引
                    let string_index = finger_info
                        .get("string_index")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    touched_strings.push(string_index as i32);
                }
            }
        }

        // 如果没有被使用的弦，直接返回
        if touched_strings.is_empty() {
            return;
        }

        self.ready_for_record();

        // 去重
        touched_strings.sort_unstable();
        touched_strings.dedup();

        // 从高到低排序
        touched_strings.sort_unstable_by(|a, b| b.cmp(a));

        let all_strings: Vec<i32> = (0..=max_string_index as i32).collect();

        // 直接处理之前的记录器，避免生成所有组合再处理
        let pre_recorders = std::mem::take(&mut self.pre_recorders);

        // 遍历之前的手势记录器
        for prev_recorder in &pre_recorders {
            if let HandRecorder::Right(right_recorder) = prev_recorder {
                if let Some(last_hand) = right_recorder.hand_pose_list.last() {
                    // 为当前情况生成可能的右手组合
                    let possible_combinations = self.generate_right_hand_combinations(
                        touched_strings.clone(),
                        all_strings.clone(),
                    );

                    if possible_combinations.is_empty() {
                        println!(
                            "当前要拨动的弦是{:?}，当前右手状态是{:?}。",
                            touched_strings, last_hand
                        );
                        continue;
                    }

                    // 遍历所有可能的组合
                    for combination in &possible_combinations {
                        let used_fingers = combination.used_fingers.clone();
                        let right_finger_positions = combination.right_finger_positions.clone();

                        // 验证右手姿势
                        let is_valid = last_hand.validate_right_hand(
                            Some(used_fingers.clone()),
                            Some(right_finger_positions.clone()),
                        );

                        if !is_valid {
                            continue;
                        }

                        let is_arpeggio = touched_strings.len() > 4;
                        let right_hand = RightHand::new(
                            used_fingers.clone(),
                            right_finger_positions.clone(),
                            last_hand.used_fingers.clone(),
                            is_arpeggio,
                            last_hand.is_playing_bass,
                        );

                        let entropy = last_hand.calculate_diff(&right_hand);
                        let new_entropy = right_recorder.current_entropy + entropy;

                        // 检查是否应该插入
                        let current_hand_pose_record_is_smallest = self.recorders.len() > 0
                            && new_entropy
                                <= self.recorders.peek().map_or(f64::MAX, |recorder| {
                                    recorder.recorder.current_entropy()
                                });

                        // 如果池子已满且新记录器熵值不小于当前最小熵值，则不插入
                        if self.recorders.len() == self.capacity
                            && !current_hand_pose_record_is_smallest
                        {
                            continue;
                        }

                        // 创建新的记录器
                        let mut new_hand_pose_list = right_recorder.hand_pose_list.clone();
                        new_hand_pose_list.push(right_hand);

                        let mut new_entropies = right_recorder.entropies.clone();
                        new_entropies.push(new_entropy);

                        let mut new_real_ticks = right_recorder.real_ticks.clone();
                        new_real_ticks.push(real_tick);

                        let new_recorder = RightHandRecorder::with_data(
                            new_hand_pose_list,
                            new_entropy,
                            new_entropies,
                            new_real_ticks,
                        );

                        // 执行插入操作
                        self.recorders.push(RecorderRef {
                            recorder: HandRecorder::Right(new_recorder),
                        });

                        // 如果插入后超过了容量，移除熵值最大的记录器
                        if self.recorders.len() > self.capacity {
                            self.recorders.pop();
                        }
                    }
                }
            }
        }

        // 恢复pre_recorders
        self.pre_recorders = pre_recorders;
    }

    /// 高效生成右手组合的方法
    fn generate_right_hand_combinations(
        &self,
        touched_strings: Vec<i32>,
        all_strings: Vec<i32>,
    ) -> Vec<RightHandCombination> {
        let mut combinations = Vec::new();

        // 特殊情况：如果触弦数超过4根，使用琶音方式
        if touched_strings.len() > 4 {
            combinations.push(RightHandCombination {
                used_fingers: Vec::new(),
                right_finger_positions: vec![5, 2, 1, 0],
            });
            return combinations;
        }

        // 生成所有满足条件的组合
        let mut current_placement = vec![0; 4]; // p, i, m, a 四个手指的位置
        let mut used_strings = std::collections::HashSet::new();
        self.generate_combinations_recursive(
            &touched_strings,
            &all_strings,
            &mut current_placement,
            &mut used_strings,
            0,
            &mut combinations,
        );

        combinations
    }

    /// 递归生成所有满足条件的组合
    fn generate_combinations_recursive(
        &self,
        touched_strings: &Vec<i32>,
        all_strings: &Vec<i32>,
        current_placement: &mut Vec<i32>,
        used_strings: &mut std::collections::HashSet<i32>,
        finger_index: usize,
        combinations: &mut Vec<RightHandCombination>,
    ) {
        // 基础情况：所有手指都已经放置
        if finger_index == 4 {
            // 检查是否所有touched_strings都被使用
            let currently_used_strings: std::collections::HashSet<i32> =
                current_placement.iter().cloned().collect();
            if touched_strings
                .iter()
                .all(|s| currently_used_strings.contains(s))
            {
                // 检查手指位置是否满足递减条件
                if (0..3).all(|i| current_placement[i] >= current_placement[i + 1]) {
                    // 生成组合
                    let fingers = vec![
                        "p".to_string(),
                        "i".to_string(),
                        "m".to_string(),
                        "a".to_string(),
                    ];
                    let used_fingers: Vec<String> = fingers
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| touched_strings.contains(&current_placement[*i]))
                        .map(|(_, f)| f.clone())
                        .collect();

                    combinations.push(RightHandCombination {
                        used_fingers,
                        right_finger_positions: current_placement.clone(),
                    });
                }
            }
            return;
        }

        // 对于每个手指，尝试所有可能的弦位置
        for &string in all_strings {
            // 检查该弦是否已经被其他手指使用
            if used_strings.contains(&string) {
                continue;
            }

            current_placement[finger_index] = string;

            // 更新已使用的弦集合
            used_strings.insert(string);

            // 如果不是第一个手指，检查是否满足递减条件
            if finger_index == 0 || current_placement[finger_index - 1] >= string {
                self.generate_combinations_recursive(
                    touched_strings,
                    all_strings,
                    current_placement,
                    used_strings,
                    finger_index + 1,
                    combinations,
                );
            }

            // 回溯：移除当前手指使用的弦
            used_strings.remove(&string);
        }
    } // 更新右手记录器池
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
        let start_time = Instant::now();
        let current_time = Instant::now();
        progress_callback("==============================");
        progress_callback(&format!("开始处理右手数据，共 {} 项", total_steps));

        for i in 0..total_steps {
            let item = &data[i];
            self.generate_right_hand_recorder(item, max_string_index);

            // 每处理10项报告一次进度
            if i % 10 == 0 || i == total_steps - 1 {
                let elapsed = current_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    (i + 1) as f64 / elapsed
                } else {
                    0.0
                };
                progress_callback(&format!(
                    "处理进度: {}/{} ({:.2} step/秒)",
                    i + 1,
                    total_steps,
                    speed
                ));
            }
        }

        let end_time = Instant::now();
        progress_callback(&format!(
            "右手数据处理完成，一共费时：{:} 秒",
            end_time.duration_since(start_time).as_secs_f64()
        ));
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

    /// 获取池子中最差的记录器（熵值最大的）
    pub fn get_worst_recorder(&self) -> &HandRecorder {
        &self
            .recorders
            .iter()
            .max_by(|a, b| {
                a.recorder
                    .current_entropy()
                    .partial_cmp(&b.recorder.current_entropy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("Recorder pool should not be empty")
            .recorder
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

    /// 获取之前记录器中最差的一个（熵值最大的）
    pub fn get_worst_pre_recorder(&self) -> &HandRecorder {
        self.pre_recorders
            .iter()
            .max_by(|a, b| {
                a.current_entropy()
                    .partial_cmp(&b.current_entropy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("Previous recorders should not be empty")
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
