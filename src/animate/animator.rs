use rand::Rng;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use crate::hand::left_finger::PressState;
use crate::midi::midi_to_note::PitchWheelInfo;
use crate::utils::util_methods::{
    Quaternion, Vector3, add_vectors, get_string_touch_position, lerp_by_fret_quaternion,
    lerp_by_fret_vector3, lerp_by_weight_vector3, scale_vector, slerp, subtract_vectors,
    vector_norm,
};

/// 左手手指索引字典常量
const LEFT_FINGER_INDEX_DICT: [(i32, &str); 4] = [
    (1, "I_L"), // 食指
    (2, "M_L"), // 中指
    (3, "R_L"), // 无名指
    (4, "P_L"), // 小指
];

/// 右手手指索引字典常量
const RIGHT_FINGER_INDEX_DICT: [(&str, usize); 4] = [
    ("p", 0), // 拇指
    ("i", 1), // 食指
    ("m", 2), // 中指
    ("a", 3), // 无名指
];

/// 动画生成器，用于根据指法数据生成左手和右手动画
pub struct Animator {
    /// 人物信息 (动态JSON数据)
    avatar_info: Value,
    /// 人物信息文件路径 (JSON格式)
    avatar_file: String,
    /// 左手指法数据文件路径 (JSON格式)
    left_hand_recorder_file: String,
    /// 输出动画文件路径 (JSON格式)
    animation_file: String,
    /// 动画帧率
    fps: f64,
    /// 乐器最大弦索引
    max_string_index: f64,
}

impl Animator {
    /// 创建一个新的Animator实例
    ///
    /// # 参数
    /// * `avatar` - 人物信息JSON文件路径
    /// * `left_hand_recorder_file` - 左手指法数据JSON文件路径
    /// * `animation_file` - 输出动画JSON文件路径
    /// * `fps` - 动画帧率
    /// * `max_string_index` - 乐器最大弦索引
    pub fn new(
        avatar_file: String,
        left_hand_recorder_file: String,
        animation_file: String,
        fps: f64,
        max_string_index: f64,
    ) -> Result<Self, Box<dyn Error>> {
        // 读取avatar JSON文件
        let avatar_path = format!("asset/controller_infos/{}", avatar_file);
        println!(
            "Reading avatar file: {} from avatar_file: {}",
            avatar_path, avatar_file
        );
        let file = File::open(avatar_path)?;
        let avatar_info: Value = serde_json::from_reader(file)?;

        Ok(Animator {
            avatar_info,
            avatar_file,
            left_hand_recorder_file,
            animation_file,
            fps,
            max_string_index,
        })
    }

    /// 获取avatar文件路径
    pub fn avatar_file(&self) -> &str {
        &self.avatar_file
    }
    /// 获取avatar数据中的特定字段
    pub fn get_avatar_field(&self, field_name: &str) -> Option<&Value> {
        self.avatar_info.get(field_name)
    }

    /// 获取嵌套的avatar数据字段
    pub fn get_avatar_nested_field(&self, path: &[&str]) -> Option<&Value> {
        let mut current = &self.avatar_info;
        for key in path {
            current = current.get(key)?;
        }
        Some(current)
    }

    /// 在生成动画数据时插入pitchwheel信息
    ///
    /// # 参数
    /// * `hand_dicts` - 已解析的手部数据
    /// * `pitch_wheel_map` - pitchwheel信息映射
    ///
    /// # 返回
    /// * 包含pitchwheel信息的扩展手部数据
    fn add_pitchwheel_info(
        &self,
        hand_dicts: Vec<Map<String, Value>>,
        pitch_wheel_map: &Vec<PitchWheelInfo>,
    ) -> Result<Vec<Map<String, Value>>, Box<dyn Error>> {
        let mut new_data = Vec::new();

        for i in 0..hand_dicts.len() {
            let mut new_item = hand_dicts[i].clone();
            // 默认pitchwheel值为0
            new_item.insert(
                "pitchwheel".to_string(),
                Value::Number(serde_json::Number::from(0)),
            );
            new_data.push(new_item.clone());

            if i != hand_dicts.len() - 1 {
                let recorder_tick = hand_dicts[i]
                    .get("real_tick")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing real_tick in item")?;

                let next_tick = hand_dicts[i + 1]
                    .get("real_tick")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing real_tick in next item")?;

                for pitch_wheel_item in pitch_wheel_map {
                    let tick = pitch_wheel_item.real_tick;
                    if recorder_tick <= tick && tick <= next_tick {
                        let mut insert_item = new_item.clone();
                        insert_item.insert(
                            "real_tick".to_string(),
                            Value::Number(serde_json::Number::from_f64(tick).unwrap()),
                        );
                        insert_item.insert(
                            "frame".to_string(),
                            Value::Number(
                                serde_json::Number::from_f64(pitch_wheel_item.frame).unwrap(),
                            ),
                        );
                        insert_item.insert(
                            "pitchwheel".to_string(),
                            Value::Number(serde_json::Number::from(pitch_wheel_item.pitchwheel)),
                        );
                        new_data.push(insert_item);
                    }
                }
            }
        }

        Ok(new_data)
    }

    /// 将左手数据转换为动画
    ///
    /// # 参数
    /// * `avatar` - 人物模型路径
    /// * `recorder_file` - 记录文件路径
    /// * `animation_file` - 动画输出文件路径
    /// * `fps` - 帧率
    /// * `max_string_index` - 最大弦索引
    /// * `is_electric` - 是否为电琴
    pub fn left_hand_2_animation(&self, disable_barre: bool) -> Result<(), Box<dyn Error>> {
        // 这是人物按下弦需要的时间，还是挺快的
        let press_duration = self.fps / 16.0;
        // 这个就是两个不同姿势之间切换时需要的帧数
        let elapsed_frame = press_duration * 3.0;
        // 这是手指从按弦变成松开需要的帧数
        let finger_return_to_rest_frame = press_duration * 1.2;

        // 获取手指位置数据
        let left_finger_positions = self
            .get_avatar_field("LEFT_FINGER_POSITIONS")
            .ok_or("Missing LEFT_FINGER_POSITIONS in avatar data")?;

        let finger_position_p0: Vec<f64> = left_finger_positions
            .get("P0")
            .ok_or("Missing P0 in LEFT_FINGER_POSITIONS")?
            .as_array()
            .ok_or("P0 is not an array")?
            .iter()
            .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
            .collect::<Result<Vec<f64>, _>>()?;

        let finger_position_p1: Vec<f64> = left_finger_positions
            .get("P1")
            .ok_or("Missing P1 in LEFT_FINGER_POSITIONS")?
            .as_array()
            .ok_or("P1 is not an array")?
            .iter()
            .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
            .collect::<Result<Vec<f64>, _>>()?;

        let finger_position_p2: Vec<f64> = left_finger_positions
            .get("P2")
            .ok_or("Missing P2 in LEFT_FINGER_POSITIONS")?
            .as_array()
            .ok_or("P2 is not an array")?
            .iter()
            .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
            .collect::<Result<Vec<f64>, _>>()?;

        // 计算p0和p1的距离
        let p0_p1_diff: Vec<f64> = finger_position_p0
            .iter()
            .zip(finger_position_p1.iter())
            .map(|(a, b)| a - b)
            .collect();

        let p0_p1_distance = p0_p1_diff.iter().map(|x| x * x).sum::<f64>().sqrt();

        // 按弦接下的距离直接取p0和p1距离的1/5
        let press_distance = p0_p1_distance / 5.0;

        // 计算法向量
        let p0_p1: Vec<f64> = finger_position_p0
            .iter()
            .zip(finger_position_p1.iter())
            .map(|(a, b)| a - b)
            .collect();

        let p2_p1: Vec<f64> = finger_position_p2
            .iter()
            .zip(finger_position_p1.iter())
            .map(|(a, b)| a - b)
            .collect();

        // 计算叉积
        let normal = vec![
            p0_p1[1] * p2_p1[2] - p0_p1[2] * p2_p1[1],
            p0_p1[2] * p2_p1[0] - p0_p1[0] * p2_p1[2],
            p0_p1[0] * p2_p1[1] - p0_p1[1] * p2_p1[0],
        ];

        let normal_length = normal.iter().map(|x| x * x).sum::<f64>().sqrt();

        let normal: Vec<f64> = normal.iter().map(|x| x / normal_length).collect();

        // 读取记录文件
        let file = File::open(self.left_hand_recorder_file.clone())?;
        let reader = BufReader::new(file);
        let hand_dicts: Vec<Map<String, Value>> = serde_json::from_reader(reader)?;

        // 如果有pitchwheel信息，则添加到数据中
        let hand_dicts = if let Some(pitch_wheel_map) = self.get_avatar_field("PITCH_WHEEL_MAP") {
            if let Ok(pitch_wheel_vec) =
                serde_json::from_value::<Vec<PitchWheelInfo>>(pitch_wheel_map.clone())
            {
                self.add_pitchwheel_info(hand_dicts, &pitch_wheel_vec)?
            } else {
                hand_dicts
            }
        } else {
            hand_dicts
        };

        let mut data_for_animation = Vec::new();
        let mut init_state = None;

        for i in 0..hand_dicts.len() {
            let item = &hand_dicts[i];
            let frame = item
                .get("frame")
                .and_then(|v| v.as_f64())
                .ok_or("Missing frame in item")?;

            let pitchwheel = item.get("pitchwheel").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            // 计算当前帧的动画信息（beat状态）
            let current_finger_infos =
                self.animated_left_hand(item, &normal, pitchwheel, press_distance, disable_barre)?;

            // 获取需要抬指的手指索引集合
            let mut finger_index_set_need_to_change = std::collections::HashSet::new();

            // 初始化下一帧信息
            let (next_frame, next_finger_infos, next_pitchwheel) = if i != hand_dicts.len() - 1 {
                let next_item = &hand_dicts[i + 1];
                let next_frame = next_item
                    .get("frame")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing frame in next item")?;

                let next_pitchwheel = next_item
                    .get("pitchwheel")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                let next_finger_infos = self.animated_left_hand(
                    next_item,
                    &normal,
                    next_pitchwheel,
                    press_distance,
                    disable_barre,
                )?;

                // 对比当前手势和下一个手势，找出来姿势切换时需要抬指的手指
                let current_hand = item
                    .get("left_hand")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing leftHand in item")?;

                let next_hand = next_item
                    .get("left_hand")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing leftHand in next item")?;

                let mut current_finger_dict = HashMap::new();
                for finger in current_hand {
                    let finger_obj = finger.as_object().ok_or("Finger is not an object")?;
                    let finger_index = finger_obj
                        .get("finger_index")
                        .and_then(|v| v.as_i64())
                        .ok_or("Missing fingerIndex")?;

                    let finger_info = finger_obj.get("finger_info").ok_or("Missing fingerInfo")?;

                    current_finger_dict.insert(finger_index, finger_info);
                }

                let mut next_finger_dict = HashMap::new();
                for finger in next_hand {
                    let finger_obj = finger.as_object().ok_or("Finger is not an object")?;
                    let finger_index = finger_obj
                        .get("finger_index")
                        .and_then(|v| v.as_i64())
                        .ok_or("Missing fingerIndex")?;

                    let finger_info = finger_obj.get("finger_info").ok_or("Missing fingerInfo")?;

                    next_finger_dict.insert(finger_index, finger_info);
                }

                for finger_index in 1..5 {
                    if let (Some(current_finger), Some(next_finger)) = (
                        current_finger_dict.get(&(finger_index as i64)),
                        next_finger_dict.get(&(finger_index as i64)),
                    ) {
                        let current_finger_obj = current_finger
                            .as_object()
                            .ok_or("Current finger info is not an object")?;
                        let next_finger_obj = next_finger
                            .as_object()
                            .ok_or("Next finger info is not an object")?;

                        let current_press = current_finger_obj
                            .get("press")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);

                        let current_string_index = current_finger_obj
                            .get("stringIndex")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);

                        let next_string_index = next_finger_obj
                            .get("stringIndex")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);

                        // 如果一个手指当前是按弦状态，而下一个状态换弦了，就需要有抬指的动作（换品可以不抬指直接滑过去）
                        if current_press != 0 && current_string_index != next_string_index {
                            finger_index_set_need_to_change.insert(finger_index);
                        }
                    }
                }

                (Some(next_frame), Some(next_finger_infos), next_pitchwheel)
            } else {
                (None, None, 0)
            };

            // 第一帧需要添加初始状态
            if i == 0 {
                // 创建初始状态（所有手指处于休息状态）
                init_state = Some(self.create_init_state(
                    item,
                    &normal,
                    pitchwheel,
                    press_distance,
                    disable_barre,
                )?);

                data_for_animation.push(serde_json::json!({
                    "frame": 0,
                    "fingerInfos": &init_state,
                    "pitchwheel": 0
                }));
            }

            // 添加当前帧（beat状态）
            data_for_animation.push(serde_json::json!({
                "frame": frame,
                "fingerInfos": current_finger_infos,
                "pitchwheel": pitchwheel
            }));

            // 插入中间帧
            let frames_to_insert = self.interpolate_left_hand_frames(
                frame,
                next_frame,
                &current_finger_infos,
                next_finger_infos.as_ref(),
                &finger_index_set_need_to_change,
                &normal,
                press_distance,
                press_duration,
                elapsed_frame,
                finger_return_to_rest_frame,
                i == 0,
                init_state.as_ref(),
                pitchwheel,
                next_pitchwheel,
            )?;

            // 将插值帧添加到动画数据中
            for frame_data in frames_to_insert {
                data_for_animation.push(Value::Object(frame_data));
            }
        }

        // 写入动画文件
        // 验证 animation_file 路径是否有效
        if self.animation_file.is_empty() {
            return Err(format!("无效的动画文件路径: {:?}", self.animation_file).into());
        }

        let file = File::create(self.animation_file.clone())?;
        serde_json::to_writer_pretty(file, &data_for_animation)?;

        Ok(())
    }

    /// 将左手转换为电子右手
    ///
    /// # 参数
    /// * `left_hand_recorder_file` - 左手记录文件路径
    /// * `right_hand_recorder_file` - 右手记录文件路径
    pub fn left_hand_2_electronic_right_hand(
        &self,
        left_hand_recorder_file: &str,
        right_hand_recorder_file: &str,
    ) -> Result<Vec<Value>, Box<dyn Error>> {
        // 读取左手记录文件
        let file = File::open(left_hand_recorder_file)?;
        let reader = BufReader::new(file);
        let data: Vec<Value> = serde_json::from_reader(reader)?;
        let total_steps = data.len();

        let mut result = Vec::new();

        // 模拟进度条处理过程
        for i in 0..total_steps {
            let item = &data[i];
            let pitchwheel = item.get("pitchwheel").and_then(|v| v.as_i64()).unwrap_or(0) as i64;

            // 如果有pitchwheel值且不为0，则跳过该帧
            if pitchwheel != 0 {
                continue;
            }

            let left_hand = item
                .get("left_hand")
                .and_then(|v| v.as_array())
                .ok_or("Missing or invalid leftHand in item")?;
            let frame = item
                .get("frame")
                .and_then(|v| v.as_f64())
                .ok_or("Missing or invalid frame in item")?;

            let mut strings = Vec::new();

            // 遍历左手手指信息
            for finger in left_hand {
                let finger_obj = finger.as_object().ok_or("Finger is not an object")?;
                let finger_index = finger_obj
                    .get("finger_index")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(-1);
                let finger_info = finger_obj
                    .get("finger_info")
                    .and_then(|v| v.as_object())
                    .ok_or("Missing or invalid fingerInfo")?;

                let press = finger_info
                    .get("press")
                    .and_then(|v| v.as_str())
                    .map(|s| PressState::from_str(s).to_i32() as f64)
                    .unwrap_or(0.0);
                let string_index = finger_info
                    .get("string_index")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(-1);

                // 如果是无效手指索引或者按压力度在0到5之间（不包括0和5），则添加弦索引
                if finger_index == -1 || (press > 0.0 && press < 5.0) {
                    strings.push(string_index);
                }
            }

            // 去除重复的弦索引
            let strings_set: std::collections::HashSet<i64> = strings.iter().cloned().collect();
            if strings.len() > strings_set.len() {
                strings = strings_set.into_iter().collect();
            }

            // 添加到结果中
            result.push(serde_json::json!({
                "frame": frame,
                "strings": strings,
            }));
        }

        // 写入右手记录文件
        let file = File::create(right_hand_recorder_file)?;
        serde_json::to_writer_pretty(file, &result)?;

        Ok(result)
    }

    /// 将电子右手数据转换为动画
    ///
    /// # 参数
    /// * `recorder_file` - 记录文件路径
    /// * `animation_file` - 动画输出文件路径
    pub fn electronic_right_hand_2_animation(
        &self,
        right_hand_data: &Vec<Value>,
        animation_file: &str,
    ) -> Result<(), Box<dyn Error>> {
        let pick_position = 5.5;
        let mut data_for_animation = Vec::new();
        // 这里是计算拨弦需要保持的时间
        let elapsed_frame = self.fps / 15.0;

        let mut pick_position = pick_position; // 需要可变的pick_position

        for i in 0..right_hand_data.len() {
            let data = &right_hand_data[i];
            let frame = data
                .get("frame")
                .and_then(|v| v.as_f64())
                .ok_or("Missing or invalid frame in data")?;

            let strings_array = data
                .get("strings")
                .and_then(|v| v.as_array())
                .ok_or("Missing or invalid strings in data")?;

            let strings: Vec<i64> = strings_array
                .iter()
                .map(|v| v.as_i64().unwrap_or(0))
                .collect();

            let is_arpeggio = strings.len() > 3;
            let min_string = *strings.iter().min().unwrap_or(&0);
            let max_string = *strings.iter().max().unwrap_or(&0);
            let time_multiplier = if strings.len() > 2 { 2 } else { 1 };
            let played_frame = frame + elapsed_frame * time_multiplier as f64;

            let mut played_finished_frame = None;
            if i < right_hand_data.len() - 1 {
                let next_frame = right_hand_data[i + 1]
                    .get("frame")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing or invalid frame in next data")?;

                if next_frame > played_frame + elapsed_frame {
                    played_finished_frame = Some(played_frame + elapsed_frame);
                }
            }

            // 如果pick当前的位置是在最低弦下面，那么以最低弦为演奏弦并且上扫弦
            // 如果pick当前的位置是在最高弦上面，那么以最高弦为演奏弦并且下扫弦
            let pick_on_low_position = pick_position < min_string as f64;
            let start_string = if pick_on_low_position {
                min_string
            } else {
                max_string
            };
            let end_string = if pick_on_low_position {
                max_string
            } else {
                min_string
            };
            let should_start_at_lower_position = pick_on_low_position;
            let should_end_at_lower_position = !pick_on_low_position;

            let ready = self.calculate_right_pick(
                start_string as i32,
                is_arpeggio,
                should_start_at_lower_position,
            )?;

            let played = self.calculate_right_pick(
                end_string as i32,
                is_arpeggio,
                should_end_at_lower_position,
            )?;

            if is_arpeggio && should_end_at_lower_position {
                pick_position = -0.5;
            } else {
                pick_position = if should_end_at_lower_position {
                    end_string as f64 + 0.5
                } else {
                    end_string as f64 - 0.5
                };
            }

            // pick拨弦分为三个阶段，准备拨弦，拨弦，拨弦后维持动作。它没有再返回准备状态的必要。
            data_for_animation.push(serde_json::json!({
                "frame": frame,
                "fingerInfos": ready
            }));

            data_for_animation.push(serde_json::json!({
                "frame": played_frame,
                "fingerInfos": played
            }));

            if let Some(finished_frame) = played_finished_frame {
                data_for_animation.push(serde_json::json!({
                    "frame": finished_frame,
                    "fingerInfos": played
                }));
            }
        }

        // 写入动画文件
        let file = File::create(animation_file)?;
        serde_json::to_writer_pretty(file, &data_for_animation)?;

        Ok(())
    }

    /// 将右手数据转换为动画
    ///
    /// # 参数
    /// * `avatar` - 人物模型路径
    /// * `recorder_file` - 记录文件路径
    /// * `animation_file` - 动画输出文件路径
    /// * `fps` - 帧率
    /// * `max_string_index` - 最大弦索引
    pub fn right_hand_2_animation(
        &self,
        recorder_file: &str,
        animation_file: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut data_for_animation = Vec::new();
        // 这里是计算按弦需要保持的时间
        let elapsed_frame = self.fps / 15.0;

        // 读取记录文件
        let file = File::open(recorder_file)?;
        let reader = BufReader::new(file);
        let hand_dicts: Vec<Value> = serde_json::from_reader(reader)?;
        let hand_count = hand_dicts.len();

        for i in 0..hand_count {
            let data = &hand_dicts[i];
            let frame = data
                .get("frame")
                .and_then(|v| v.as_f64())
                .ok_or("Missing frame in data")?;

            let right_hand = data.get("right_hand").ok_or("Missing rightHand in data")?;
            let used_fingers = right_hand
                .get("used_fingers")
                .ok_or("Missing usedFingers in rightHand")?;
            let right_finger_positions = right_hand
                .get("right_finger_positions")
                .ok_or("Missing rightFingerPositions in rightHand")?;

            // 这个usedFingers为空，表示是扫弦，所以播放时间要长一些
            let time_multiplier = if used_fingers.as_array().map_or(true, |arr| arr.is_empty()) {
                2.0
            } else {
                1.0
            };
            let played_frame = frame + elapsed_frame * time_multiplier;

            let played_finished_frame = if i != hand_count - 1 {
                let next_frame = hand_dicts[i + 1]
                    .get("frame")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing frame in next data")?;
                if next_frame > played_frame + 2.0 * elapsed_frame {
                    // 这个2 * elapsed_frame 相当于留给手掌移动到下一个位置的时间
                    Some(next_frame - 2.0 * elapsed_frame)
                } else {
                    None
                }
            } else {
                None
            };

            let ready = self.calculate_right_hand_fingers(
                right_finger_positions,
                used_fingers,
                false, // isAfterPlayed = false
            )?;

            let played = self.calculate_right_hand_fingers(
                right_finger_positions,
                used_fingers,
                true, // isAfterPlayed = true
            )?;

            // 右手拨弦分为四个阶段，准备拨弦，拨弦，拨弦后维持动作，返回准备状态。
            // 如果与下一个音符之间的间隔足够长，就需要把这些动作都记录下来

            // 触弦帧
            data_for_animation.push(serde_json::json!({
                "frame": frame,
                "fingerInfos": ready,
            }));

            data_for_animation.push(serde_json::json!({
                "frame": played_frame,
                "fingerInfos": played,
            }));

            // 拨弦后慢慢弹回来
            if let Some(finished_frame) = played_finished_frame {
                data_for_animation.push(serde_json::json!({
                    "frame": finished_frame,
                    "fingerInfos": ready,
                }));
            }
        }

        // 写入动画文件
        let file = File::create(animation_file)?;
        serde_json::to_writer_pretty(file, &data_for_animation)?;

        Ok(())
    }

    /// 计算右手手指位置
    /// 计算右手手指位置
    pub fn calculate_right_hand_fingers(
        &self,
        right_finger_positions: &Value,
        used_fingers: &Value,
        is_after_played: bool,
    ) -> Result<HashMap<String, Vec<f64>>, Box<dyn Error>> {
        // 解析参数
        let right_finger_positions_vec: Result<Vec<i32>, _> = right_finger_positions
            .as_array()
            .ok_or("rightFingerPositions is not an array")?
            .iter()
            .map(|v| {
                v.as_i64()
                    .ok_or("Position value is not a number")
                    .map(|i| i as i32)
            })
            .collect();

        let right_finger_positions_vec = right_finger_positions_vec?;

        let used_fingers_vec: Result<Vec<String>, _> = used_fingers
            .as_array()
            .ok_or("usedFingers is not an array")?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or("Finger value is not a string")
                    .map(|s| s.to_string())
            })
            .collect();

        let used_fingers_vec = used_fingers_vec?;

        let is_arpeggio = used_fingers_vec.is_empty();
        let mut hand_position = 0.0;

        if !is_arpeggio {
            let mut offsets = 0.0;
            for used_finger in &used_fingers_vec {
                // 查找手指索引
                let finger_index = RIGHT_FINGER_INDEX_DICT
                    .iter()
                    .find(|&&(finger, _)| finger == used_finger)
                    .map(|&(_, index)| index)
                    .ok_or(format!("Unknown finger: {}", used_finger))?;

                let current_finger_position = right_finger_positions_vec[finger_index];
                let multiplier = if used_finger == "p" { 3.0 } else { 1.0 };
                offsets += current_finger_position as f64 * multiplier;
            }

            let total_finger_weights = if used_fingers_vec.contains(&"p".to_string()) {
                used_fingers_vec.len() + 3
            } else {
                used_fingers_vec.len()
            };

            let average_offset = offsets / total_finger_weights as f64;

            /*
            这一段解释一下：
            我只在blender里调整了h0和h3的两个位置，然后通过这两个位置的average_offset来计算其它手指的位置。
            所谓 average_offset， 是计量了所有需要弹奏的手指的位置的平均值，然后通过这个平均值来决定手掌的位置，最后面再决定那些没有参与演奏的手指的位置。
            在计算 average_offset时，p指是特殊处理了的，它的权重是3，因为p指是决定手掌位置最重要的手指。

            h0和h3两个极端手型，这两个组合包含了各个手指实际上能触碰的最高和最低弦：
            h3是在{"p": 2, "i": 0, "m": 0, "a": 0}的位置上，它的average_offset是1, 同时h_position=3
            h0是在{"p": 5, "i": 4, "m": 3, "a": 2}的位置上，它的average_offset是6，同时h_position=0

            假定h_position都是线性分布，满足：
            h_position = m * average_offset + b

            根据上面这两个值，我们可以计算出来线性插值的斜率是：
            m = -0.6

            最终线性插值的计算公式是：
            h_position = -0.6 * average_offset + 3.6

            而这个时间手掌的实际位置是：
            H_R = h0 + h_position * (h3 - h0) / 3
            */
            hand_position = -0.6 * average_offset + 3.6;
        }

        // 调用new_finger_position_method方法计算结果
        let result = self.get_fingers_position(
            &right_finger_positions_vec,
            is_arpeggio,
            is_after_played,
            hand_position,
            &used_fingers_vec,
        )?;

        Ok(result)
    }

    /// 获取嵌套的avatar数据字段为f64向量
    fn get_avatar_nested_field_as_f64_vector(
        &self,
        path: &[&str],
    ) -> Result<Vec<f64>, Box<dyn Error>> {
        let field = self
            .get_avatar_nested_field(path)
            .ok_or(format!("Missing field {:?}", path))?;

        let array = field
            .as_array()
            .ok_or(format!("Field {:?} is not an array", path))?;

        array
            .iter()
            .map(|v| {
                v.as_f64()
                    .ok_or(format!("Field {:?} contains non-number values", path).into())
            })
            .collect::<Result<Vec<f64>, Box<dyn Error>>>()
    }

    /// 新的计算手指位置方法
    fn get_fingers_position(
        &self,
        right_finger_positions: &Vec<i32>,
        is_arpeggio: bool,
        is_after_played: bool,
        hand_position: f64,
        used_right_fingers: &Vec<String>,
    ) -> Result<HashMap<String, Vec<f64>>, Box<dyn Error>> {
        let mut result = HashMap::new();

        // 定义手指运动的距离，是P0和P1之间的距离的8分之1，相当于0.725的弦距
        let p0 = self.get_avatar_nested_field_as_f64_vector(&["LEFT_FINGER_POSITIONS", "P0"])?;
        let p1 = self.get_avatar_nested_field_as_f64_vector(&["LEFT_FINGER_POSITIONS", "P1"])?;
        let p0_p1_diff = subtract_vectors(&p0, &p1);
        let finger_move_distance_while_play =
            0.725 * vector_norm(&p0_p1_diff) / (self.max_string_index + 1.0);

        // 大拇指的移动方向以及其它手指的移动方向，一开始设置为None
        let mut t_move: Option<Vec<f64>> = None;
        let mut f_move: Option<Vec<f64>> = None;

        let (h_r, h_rotation_r, hp_r, tp_r, mut t_r, mut i_r, mut m_r, mut r_r, p_r);

        if is_arpeggio {
            if is_after_played {
                h_r = self.get_avatar_nested_field_as_f64_vector(&[
                    "RIGHT_HAND_POSITIONS",
                    "Normal_Pend_H_R",
                ])?;
                h_rotation_r = self.get_avatar_nested_field_as_f64_vector(&[
                    "ROTATIONS",
                    "H_rotation_R",
                    "Normal",
                    "Pend",
                ])?;
                hp_r = self.get_avatar_nested_field_as_f64_vector(&[
                    "RIGHT_HAND_POSITIONS",
                    "Normal_Pend_HP_R",
                ])?;
                tp_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "tpend"])?;
                t_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "pend"])?;
                i_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "iend"])?;
                m_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "mend"])?;
                r_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "aend"])?;
                p_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "chend"])?;
            } else {
                h_rotation_r = self.get_avatar_nested_field_as_f64_vector(&[
                    "ROTATIONS",
                    "H_rotation_R",
                    "Normal",
                    "P0",
                ])?;
                h_r = self.get_avatar_nested_field_as_f64_vector(&[
                    "RIGHT_HAND_POSITIONS",
                    "Normal_P0_H_R",
                ])?;
                hp_r = self.get_avatar_nested_field_as_f64_vector(&[
                    "RIGHT_HAND_POSITIONS",
                    "Normal_P0_HP_R",
                ])?;
                tp_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "tp0"])?;
                t_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "p0"])?;
                i_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "i0"])?;
                m_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "m0"])?;
                r_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "a0"])?;
                p_r =
                    self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "ch0"])?;
            }
        } else {
            let h0 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_POSITIONS",
                "Normal_P0_H_R",
            ])?;
            let h3 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_POSITIONS",
                "Normal_P3_H_R",
            ])?;

            // H_R = h0 + hand_position * (h3 - h0) / 3
            let h3_h0_diff = subtract_vectors(&h3, &h0);
            let scaled_diff = scale_vector(&h3_h0_diff, hand_position / 3.0);
            h_r = add_vectors(&h0, &scaled_diff);

            let h_rotation_0 = self.get_avatar_nested_field_as_f64_vector(&[
                "ROTATIONS",
                "H_rotation_R",
                "Normal",
                "P0",
            ])?;
            let h_rotation_3 = self.get_avatar_nested_field_as_f64_vector(&[
                "ROTATIONS",
                "H_rotation_R",
                "Normal",
                "P3",
            ])?;

            h_rotation_r = if h_rotation_0.len() == 3 {
                let h_rot_3_0_diff = subtract_vectors(&h_rotation_3, &h_rotation_0);
                let scaled_diff = scale_vector(&h_rot_3_0_diff, hand_position / 3.0);
                add_vectors(&h_rotation_0, &scaled_diff)
            } else if h_rotation_0.len() == 4 {
                let hand_weight = hand_position / 3.0;
                let h_rot_0_quat = Quaternion::from_vector64(h_rotation_0);
                let h_rot_3_quat = Quaternion::from_vector64(h_rotation_3);
                let h_rotation_quat = slerp(&h_rot_0_quat, &h_rot_3_quat, hand_weight);
                h_rotation_quat.to_vector64()
            } else {
                return Err("Unknown hand rotation type".into());
            };

            let hp_0 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_POSITIONS",
                "Normal_P0_HP_R",
            ])?;
            let hp_3 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_POSITIONS",
                "Normal_P3_HP_R",
            ])?;
            let hp_3_0_diff = subtract_vectors(&hp_3, &hp_0);
            let scaled_diff = scale_vector(&hp_3_0_diff, hand_position / 3.0);
            hp_r = add_vectors(&hp_0, &scaled_diff);

            let tp_0 =
                self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "tp0"])?;
            let tp_3 =
                self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "tp3"])?;
            let tp_3_0_diff = subtract_vectors(&tp_3, &tp_0);
            let scaled_diff = scale_vector(&tp_3_0_diff, hand_position / 3.0);
            tp_r = add_vectors(&tp_0, &scaled_diff);

            let n_quat_0 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_LINES",
                "right_hand_normal_p0",
                "vector",
            ])?;
            let n_quat_3 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_LINES",
                "right_hand_normal_p3",
                "vector",
            ])?;
            let n_quat_3_0_diff = subtract_vectors(&n_quat_3, &n_quat_0);
            let scaled_diff = scale_vector(&n_quat_3_0_diff, hand_position / 3.0);
            let n_quat = add_vectors(&n_quat_0, &scaled_diff);

            let p2 =
                self.get_avatar_nested_field_as_f64_vector(&["LEFT_FINGER_POSITIONS", "P2"])?;
            let p3 =
                self.get_avatar_nested_field_as_f64_vector(&["LEFT_FINGER_POSITIONS", "P3"])?;

            let t0 = self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "p0"])?;
            let t3 = self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "p3"])?;
            let t_3_0_diff = subtract_vectors(&t3, &t0);
            let scaled_diff = scale_vector(&t_3_0_diff, hand_position / 3.0);
            let t_rest_position = add_vectors(&t0, &scaled_diff);

            let thumb_direction_0 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_LINES",
                "right_thumb_direct_p0",
                "vector",
            ])?;
            let thumb_direction_3 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_LINES",
                "right_thumb_direct_p3",
                "vector",
            ])?;
            let thumb_direction_3_0_diff = subtract_vectors(&thumb_direction_3, &thumb_direction_0);
            let scaled_diff = scale_vector(&thumb_direction_3_0_diff, hand_position / 3.0);
            let thumb_direction = add_vectors(&thumb_direction_0, &scaled_diff);

            let finger_direct_0 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_LINES",
                "right_finger_direct_p0",
                "vector",
            ])?;
            let finger_direct_3 = self.get_avatar_nested_field_as_f64_vector(&[
                "RIGHT_HAND_LINES",
                "right_finger_direct_p3",
                "vector",
            ])?;
            let finger_direct_3_0_diff = subtract_vectors(&finger_direct_3, &finger_direct_0);
            let scaled_diff = scale_vector(&finger_direct_3_0_diff, hand_position / 3.0);
            let finger_direct = add_vectors(&finger_direct_0, &scaled_diff);

            let i0 = self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "i0"])?;
            let i3 = self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "i3"])?;
            let i_3_0_diff = subtract_vectors(&i3, &i0);
            let scaled_diff = scale_vector(&i_3_0_diff, hand_position / 3.0);
            let i_rest_position = add_vectors(&i0, &scaled_diff);

            let m0 = self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "m0"])?;
            let m3 = self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "m3"])?;
            let m_3_0_diff = subtract_vectors(&m3, &m0);
            let scaled_diff = scale_vector(&m_3_0_diff, hand_position / 3.0);
            let m_rest_position = add_vectors(&m0, &scaled_diff);

            let a0 = self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "a0"])?;
            let a3 = self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "a3"])?;
            let a_3_0_diff = subtract_vectors(&a3, &a0);
            let scaled_diff = scale_vector(&a_3_0_diff, hand_position / 3.0);
            let a_rest_position = add_vectors(&a0, &scaled_diff);

            let ch0 =
                self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "ch0"])?;
            let ch3 =
                self.get_avatar_nested_field_as_f64_vector(&["RIGHT_HAND_POSITIONS", "ch3"])?;
            let ch_3_0_diff = subtract_vectors(&ch3, &ch0);
            let scaled_diff = scale_vector(&ch_3_0_diff, hand_position / 3.0);
            let ch_rest_position = add_vectors(&ch0, &scaled_diff);

            if used_right_fingers.contains(&"p".to_string()) {
                let p_current_string_index = right_finger_positions[0];
                // t_target就是大拇指运动时的目标位置，和其它手指都是向掌心运动不同，大拇指是往弦上运动的
                let t_target = add_vectors(&t_rest_position, &thumb_direction);
                let t_touch_position = get_string_touch_position(
                    &t_rest_position,
                    &t_target,
                    &n_quat,
                    &p0,
                    &p1,
                    &p2,
                    &p3,
                    p_current_string_index,
                    self.max_string_index,
                )?;

                // 读取大拇指拨弦的方向
                let thumb_direction_norm = vector_norm(&thumb_direction);
                t_move = Some(scale_vector(&thumb_direction, 1.0 / thumb_direction_norm));

                if is_after_played {
                    if let Some(ref t_move_vec) = t_move {
                        let scaled_move = scale_vector(t_move_vec, finger_move_distance_while_play);
                        t_r = add_vectors(&t_touch_position, &scaled_move);
                    } else {
                        t_r = t_touch_position;
                    }
                } else {
                    if let Some(ref t_move_vec) = t_move {
                        let scaled_move = scale_vector(t_move_vec, finger_move_distance_while_play);
                        t_r = subtract_vectors(&t_touch_position, &scaled_move);
                    } else {
                        t_r = t_touch_position;
                    }
                }
            } else {
                t_r = t_rest_position;
            }

            // 处理i, m, a三个手指的计算,有一个小[r](file://g:\fretDance_rust\target\debug\build\serde-d197b0748e5ae023\stderr)其实就是a指的英文ring
            let finger_configs = vec![
                ("i", 1, i_rest_position),
                ("m", 2, m_rest_position),
                ("r", 3, a_rest_position),
            ];

            let mut finger_results = HashMap::new();

            // 处理ima三个手指的拨弦，它们的逻辑是相似的
            for (finger_char, finger_idx, rest_pos) in finger_configs {
                // 小拇指的英文缩写和吉他用语里的左手小拇指不一样，导致这个判断的写法会啰嗦一点
                if used_right_fingers.contains(&finger_char.to_string())
                    || (finger_char == "r" && used_right_fingers.contains(&"a".to_string()))
                {
                    // 计算手指的拨弦方向，这里要注意因为在blender中方向线与手指运动的方式是相反的，所以这里要加负号
                    let finger_direct_norm = vector_norm(&finger_direct);
                    f_move = Some(scale_vector(&finger_direct, -1.0 / finger_direct_norm));
                    let current_string_index = right_finger_positions[finger_idx];
                    let target = add_vectors(&rest_pos, &finger_direct);
                    let touch_position = get_string_touch_position(
                        &rest_pos,
                        &target,
                        &n_quat,
                        &p0,
                        &p1,
                        &p2,
                        &p3,
                        current_string_index,
                        self.max_string_index,
                    )?;

                    let finger_key = if finger_char == "r" {
                        "R".to_string()
                    } else {
                        finger_char.to_uppercase().to_string()
                    };

                    if is_after_played {
                        if let Some(ref f_move_vec) = f_move {
                            let scaled_move =
                                scale_vector(f_move_vec, finger_move_distance_while_play);
                            finger_results.insert(
                                format!("{}_R", finger_key),
                                add_vectors(&touch_position, &scaled_move),
                            );
                        } else {
                            finger_results.insert(format!("{}_R", finger_key), touch_position);
                        }
                    } else {
                        if let Some(ref f_move_vec) = f_move {
                            let scaled_move =
                                scale_vector(f_move_vec, finger_move_distance_while_play);
                            finger_results.insert(
                                format!("{}_R", finger_key),
                                subtract_vectors(&touch_position, &scaled_move),
                            );
                        } else {
                            finger_results.insert(format!("{}_R", finger_key), touch_position);
                        }
                    }
                } else {
                    let finger_key = if finger_char == "r" {
                        "R".to_string()
                    } else {
                        finger_char.to_uppercase().to_string()
                    };
                    finger_results.insert(format!("{}_R", finger_key), rest_pos);
                }
            }

            // 然后分别赋值
            i_r = finger_results.remove("I_R").unwrap_or_else(|| vec![0.0; 3]);
            m_r = finger_results.remove("M_R").unwrap_or_else(|| vec![0.0; 3]);
            r_r = finger_results.remove("R_R").unwrap_or_else(|| vec![0.0; 3]);

            // ch指是不参与演奏的，所以直接使用休息位置
            p_r = ch_rest_position;
        }

        // 给拨弦后的手掌添加一点移动
        let h_move = if f_move.is_some() {
            f_move
        } else {
            t_move.clone()
        };

        let mut h_r = h_r;
        if let Some(h_move_vec) = h_move {
            if is_after_played {
                let scaled_move = scale_vector(&h_move_vec, finger_move_distance_while_play * 0.25);
                h_r = subtract_vectors(&h_r, &scaled_move);

                // 应用后座力到未参与演奏的手指
                if !used_right_fingers.contains(&"p".to_string()) {
                    let scaled_move =
                        scale_vector(&h_move_vec, finger_move_distance_while_play * 0.25);
                    let t_r_moved = subtract_vectors(&t_r, &scaled_move);
                    t_r = t_r_moved;
                }

                // 处理i, m, a三个手指的后座力效果
                let finger_configs =
                    vec![("i", 1, &mut i_r), ("m", 2, &mut m_r), ("r", 3, &mut r_r)];

                for (finger_char, _finger_idx, finger_pos) in finger_configs {
                    // 检查手指是否未参与演奏
                    let is_used = used_right_fingers.contains(&finger_char.to_string())
                        || (finger_char == "r" && used_right_fingers.contains(&"a".to_string()));

                    if !is_used {
                        let scaled_move =
                            scale_vector(&h_move_vec, finger_move_distance_while_play * 0.25);
                        let moved_pos = subtract_vectors(finger_pos, &scaled_move);
                        *finger_pos = moved_pos;
                    }
                }
            }
        }

        result.insert("H_R".to_string(), h_r);
        result.insert("H_rotation_R".to_string(), h_rotation_r);
        result.insert("HP_R".to_string(), hp_r);
        result.insert("TP_R".to_string(), tp_r);
        result.insert("T_R".to_string(), t_r);
        result.insert("I_R".to_string(), i_r);
        result.insert("M_R".to_string(), m_r);
        result.insert("R_R".to_string(), r_r);
        result.insert("P_R".to_string(), p_r);

        Ok(result)
    }
    /// 生成吉他的弦动画
    ///
    /// # 参数
    /// * `recorder_file` - 记录文件路径
    /// * `string_recorder_file` - 弦记录文件路径
    pub fn animated_guitar_string(
        &self,
        recorder_file: &str,
        string_recorder_file: &str,
    ) -> Result<(), Box<dyn Error>> {
        let elapsed_frame = self.fps / 8.0;

        // 读取记录文件
        let file = File::open(recorder_file)?;
        let reader = BufReader::new(file);
        let hand_dicts: Vec<serde_json::Value> = serde_json::from_reader(reader)?;

        let mut data_for_animation = Vec::new();

        for i in 0..hand_dicts.len() {
            let item = &hand_dicts[i];
            let frame = item["frame"].as_f64().unwrap_or(0.0);
            let left_hand = &item["left_hand"];

            let string_last_frame = if i != hand_dicts.len() - 1 {
                let next_frame = hand_dicts[i + 1]["frame"].as_f64().unwrap_or(0.0);
                if next_frame < frame + elapsed_frame {
                    next_frame
                } else {
                    frame + elapsed_frame
                }
            } else {
                frame + elapsed_frame
            };

            if let Some(fingers) = left_hand.as_array() {
                for finger_data in fingers {
                    let finger_index = finger_data["finger_index"].as_i64().unwrap_or(-1);

                    let finger_info = &finger_data["finger_info"];
                    let press_value = match finger_info.get("press").and_then(|v| v.as_str()) {
                        Some(value) => value,
                        None => {
                            println!("Missing press value in finger info：{:?}", finger_info);
                            continue;
                        }
                    };

                    let press = PressState::from_str(press_value).to_i32();

                    if (press == 0 || press == 5) && finger_index != -1 {
                        continue;
                    }

                    let string_index = finger_info["string_index"].as_i64().unwrap_or(0);
                    let fret = if finger_index == -1 {
                        0
                    } else {
                        finger_info["fret"].as_i64().unwrap_or(0)
                    };

                    let ready = serde_json::json!({
                        "frame": frame - 1.0,
                        "stringIndex": string_index,
                        "fret": fret,
                        "influence": 0.0
                    });

                    let start = serde_json::json!({
                        "frame": frame,
                        "stringIndex": string_index,
                        "fret": fret,
                        "influence": 0.5
                    });

                    let end = serde_json::json!({
                        "frame": string_last_frame,
                        "stringIndex": string_index,
                        "fret": fret,
                        "influence": 0.0
                    });

                    data_for_animation.push(ready);
                    data_for_animation.push(start);
                    data_for_animation.push(end);

                    if string_last_frame - frame > 2.0 {
                        let middle = serde_json::json!({
                            "frame": (string_last_frame + frame) / 2.0,
                            "stringIndex": string_index,
                            "fret": fret,
                            "influence": 1.0
                        });
                        data_for_animation.push(middle);
                    }
                }
            }
        }

        // 过滤掉frame值小于0的元素
        data_for_animation.retain(|item| {
            if let Some(frame) = item.get("frame").and_then(|v| v.as_f64()) {
                frame >= 0.0
            } else {
                false
            }
        });

        // 按frame值进行排序，由低到高
        data_for_animation.sort_by(|a, b| {
            let frame_a = a.get("frame").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let frame_b = b.get("frame").and_then(|v| v.as_f64()).unwrap_or(0.0);
            frame_a
                .partial_cmp(&frame_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 写入弦记录文件
        let file = File::create(string_recorder_file)?;
        serde_json::to_writer_pretty(file, &data_for_animation)?;

        Ok(())
    }
    pub fn animated_left_hand(
        &self,
        item: &Map<String, Value>,
        normal: &[f64],
        pitchwheel: i32,
        rest_finger_distance: f64,
        disable_barre: bool,
    ) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        let left_hand = item
            .get("left_hand")
            .and_then(|v| v.as_array())
            .ok_or("Missing leftHand in item")?;

        let hand_fret = item
            .get("hand_position")
            .and_then(|v| v.as_f64())
            .ok_or("Missing hand_position in item")?;

        let use_barre = item
            .get("use_barre")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            && !disable_barre;

        let mut finger_infos = Map::new();
        let mut barre_finger_string_index = 0.0;
        let mut finger_string_numbers = HashMap::new();

        // 初始化手指弦号统计
        for i in 1..=4 {
            finger_string_numbers.insert(i, 0.0);
        }

        // 开始计算手指信息
        for finger_data in left_hand {
            let finger_obj = finger_data
                .as_object()
                .ok_or("Finger data is not an object")?;

            let finger_index = finger_obj
                .get("finger_index")
                .and_then(|v| v.as_i64())
                .ok_or("Missing fingerIndex")? as i32;

            let finger_info = finger_obj
                .get("finger_info")
                .and_then(|v| v.as_object())
                .ok_or("Missing fingerInfo")?;

            let mut string_index = finger_info
                .get("string_index")
                .and_then(|v| v.as_f64())
                .ok_or("Missing stringIndex")?;

            let fret = finger_info
                .get("fret")
                .and_then(|v| v.as_f64())
                .ok_or("Missing fret")?;

            let press_value = finger_info
                .get("press")
                .and_then(|v| v.as_str())
                .ok_or("Missing press")?;

            let press = PressState::from_str(press_value).to_i32();

            // skip open string. 空弦音跳过
            if finger_index == -1 {
                continue;
            }

            // 不按弦的手指会稍微移动，以避免和按弦的手指挤在一起
            if press == 0 {
                // PRESSSTATE['Open']
                if string_index > 2.0 {
                    string_index -= 0.5;
                } else {
                    string_index += 0.5;
                }
            }

            // 按弦的手指考虑是否有pitchWheel，以进行对应的移动
            if press == 1 && pitchwheel != 0 {
                // PRESSSTATE['Pressed']
                let pitch_move = pitchwheel as f64 / 8192.0;
                if string_index > 2.0 {
                    string_index -= pitch_move;
                } else {
                    string_index += pitch_move;
                }
            }

            let (finger_position, position_value_name) = {
                // 手指的横按与非横按使用两套不同的计算方式
                if use_barre && finger_index == 1 {
                    let pos = self.twice_lerp_barre_fingers(fret, string_index)?;
                    barre_finger_string_index = string_index;
                    (pos, "I_L".to_string())
                } else {
                    finger_string_numbers.insert(finger_index, string_index);
                    let mut pos = self.twice_lerp_fingers(fret, string_index)?;

                    // 如果手指没有按下，那么手指位置会稍微上移
                    if press == 0 {
                        // Open state
                        // 小拇指就是抬得高一些
                        if finger_index == 4 {
                            for i in 0..3 {
                                pos[i] -= 2.0 * normal[i] * rest_finger_distance;
                            }
                        } else {
                            for i in 0..3 {
                                pos[i] -= normal[i] * rest_finger_distance;
                            }
                        }
                    }

                    let name = match finger_index {
                        1 => "I_L".to_string(),
                        2 => "M_L".to_string(),
                        3 => "R_L".to_string(),
                        4 => "P_L".to_string(),
                        _ => "None".to_string(),
                    };

                    (pos, name)
                }
            };

            finger_infos.insert(
                position_value_name,
                serde_json::Value::Array(
                    finger_position
                        .into_iter()
                        .map(|x| {
                            serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap())
                        })
                        .collect(),
                ),
            );
        }

        // 判断当前应该使用哪种手型来计算
        let index_finger_string_number = *finger_string_numbers.get(&1).unwrap_or(&0.0);
        let pinky_finger_string_number = *finger_string_numbers.get(&4).unwrap_or(&0.0);
        let hand_state = (pinky_finger_string_number - index_finger_string_number) as i32;

        // 计算手位置
        let hand_position = if use_barre {
            self.twice_lerp_barre_hand_vector3(hand_state, "position", hand_fret)?
        } else {
            self.twice_lerp_vector3(hand_state, "H_L", "position", hand_fret)?
        };

        let hand_position_vec = hand_position.to_vector64();

        finger_infos.insert(
            "H_L".to_string(),
            serde_json::Value::Array(
                hand_position_vec
                    .into_iter()
                    .map(|x| serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap()))
                    .collect(),
            ),
        );

        // 计算手臂IK，手旋转，大拇指位置，IK
        let (hand_ik_pivot_position, hand_rotation_l, thumb_position, thumb_ik_pivot_position) =
            if use_barre {
                (
                    self.twice_lerp_barre_vector3("HP_L", "position", hand_fret)?,
                    self.twice_lerp_barre_quaternion(
                        "rotation",
                        hand_fret,
                        barre_finger_string_index,
                    )?,
                    self.twice_lerp_barre_vector3("T_L", "position", hand_fret)?,
                    self.twice_lerp_barre_vector3("TP_L", "position", hand_fret)?,
                )
            } else {
                (
                    self.twice_lerp_vector3(hand_state, "HP_L", "position", hand_fret)?,
                    self.twice_lerp_quaternion(hand_state, hand_fret, index_finger_string_number)?,
                    self.twice_lerp_vector3(hand_state, "T_L", "position", hand_fret)?,
                    self.twice_lerp_vector3(hand_state, "TP_L", "position", hand_fret)?,
                )
            };

        let hand_ik_pivot_position_vec = hand_ik_pivot_position.to_vector64();

        finger_infos.insert(
            "HP_L".to_string(),
            serde_json::Value::Array(
                hand_ik_pivot_position_vec
                    .into_iter()
                    .map(|x| serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap()))
                    .collect(),
            ),
        );

        let hand_rotation_l_vec = hand_rotation_l.to_vector64();

        finger_infos.insert(
            "H_rotation_L".to_string(),
            serde_json::Value::Array(
                hand_rotation_l_vec
                    .into_iter()
                    .map(|x| serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap()))
                    .collect(),
            ),
        );

        let thumb_position_vec = thumb_position.to_vector64();

        finger_infos.insert(
            "T_L".to_string(),
            serde_json::Value::Array(
                thumb_position_vec
                    .into_iter()
                    .map(|x| serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap()))
                    .collect(),
            ),
        );

        let thumb_ik_pivot_position_vec = thumb_ik_pivot_position.to_vector64();

        finger_infos.insert(
            "TP_L".to_string(),
            serde_json::Value::Array(
                thumb_ik_pivot_position_vec
                    .into_iter()
                    .map(|x| serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap()))
                    .collect(),
            ),
        );

        Ok(finger_infos)
    }

    pub fn interpolate_left_hand_frames(
        &self,
        current_frame: f64,
        next_frame: Option<f64>,
        current_beat_state: &Map<String, Value>,
        next_ready_state: Option<&Map<String, Value>>,
        finger_index_set_need_to_change: &std::collections::HashSet<i32>,
        normal: &[f64],
        press_distance: f64,
        press_duration: f64,
        action_duration: f64,
        rest_duration: f64,
        is_first_action: bool,
        init_state: Option<&Map<String, Value>>,
        pitchwheel: i32,
        next_pitchwheel: i32,
    ) -> Result<Vec<Map<String, Value>>, Box<dyn std::error::Error>> {
        let mut frames_to_insert = Vec::new();

        // 处理第一个动作的特殊情况
        if is_first_action {
            if let Some(init_state) = init_state {
                // 在当前帧前action_duration的位置插入当前动作的预备状态
                let ready_time = current_frame - action_duration - press_duration;
                if ready_time >= 0.0 {
                    let mut ready_frame = Map::new();
                    ready_frame.insert(
                        "frame".to_string(),
                        Value::Number(serde_json::Number::from_f64(ready_time).unwrap()),
                    );
                    ready_frame
                        .insert("fingerInfos".to_string(), Value::Object(init_state.clone()));
                    ready_frame.insert("pitchwheel".to_string(), Value::Number(pitchwheel.into()));
                    frames_to_insert.push(ready_frame);
                }
            }
        }

        // 如果没有下一个动作帧，仅插入当前动作的rest状态，以示保持
        if next_frame.is_none() {
            let rest_time = current_frame + rest_duration;
            // 创建rest状态（抬指状态）
            let rest_state = self.create_rest_state(
                current_beat_state,
                press_distance,
                finger_index_set_need_to_change,
                normal,
            )?;

            let mut rest_frame_data = Map::new();
            rest_frame_data.insert(
                "frame".to_string(),
                Value::Number(serde_json::Number::from_f64(rest_time).unwrap()),
            );
            rest_frame_data.insert("fingerInfos".to_string(), Value::Object(rest_state.clone()));
            rest_frame_data.insert("pitchwheel".to_string(), Value::Number(pitchwheel.into()));
            frames_to_insert.push(rest_frame_data);

            return Ok(frames_to_insert);
        }

        // 获取当前和下一个动作帧的时间戳
        let current_time = current_frame;
        let next_time = next_frame.unwrap();
        let t = next_time - current_time; // 可用时间

        // 情况1: 时间足够插入所有状态（保持结束、回弹结束、预备状态）
        if t >= rest_duration + action_duration + press_duration {
            // 创建rest状态（抬指状态）
            let rest_state = self.create_rest_state(
                current_beat_state,
                press_distance,
                finger_index_set_need_to_change,
                normal,
            )?;

            let rest_start_time = next_time - press_duration - action_duration - rest_duration;
            let rest_end_time = next_time - press_duration - action_duration;
            let ready_time = next_time - press_duration;

            // 插入beat状态持续结束时的帧
            let mut rest_start_frame = Map::new();
            rest_start_frame.insert(
                "frame".to_string(),
                Value::Number(serde_json::Number::from_f64(rest_start_time).unwrap()),
            );
            rest_start_frame.insert(
                "fingerInfos".to_string(),
                Value::Object(current_beat_state.clone()),
            );
            rest_start_frame.insert("pitchwheel".to_string(), Value::Number(pitchwheel.into()));
            frames_to_insert.push(rest_start_frame);

            // 插入进入到rest状态的帧，注意rest不用保持，也就是达到rest状态以后就可以开始移动了
            let mut rest_end_frame = Map::new();
            rest_end_frame.insert(
                "frame".to_string(),
                Value::Number(serde_json::Number::from_f64(rest_end_time).unwrap()),
            );
            rest_end_frame.insert("fingerInfos".to_string(), Value::Object(rest_state));
            rest_end_frame.insert("pitchwheel".to_string(), Value::Number(pitchwheel.into()));
            frames_to_insert.push(rest_end_frame);

            // 插入下一个动作的ready帧
            if let Some(next_ready_state) = next_ready_state {
                let mut ready_frame = Map::new();
                ready_frame.insert(
                    "frame".to_string(),
                    Value::Number(serde_json::Number::from_f64(ready_time).unwrap()),
                );
                ready_frame.insert(
                    "fingerInfos".to_string(),
                    Value::Object(next_ready_state.clone()),
                );
                ready_frame.insert(
                    "pitchwheel".to_string(),
                    Value::Number(next_pitchwheel.into()),
                );
                frames_to_insert.push(ready_frame);
            }
        }
        // 情况2: 时间不够插入所有状态，但足够插入预备状态和移动过程，所以可以去掉保持beat状态的帧，也就是按完立马开始抬指
        else if t >= action_duration + press_duration {
            let ready_time = next_time - press_duration;
            let rest_end_time = next_time - press_duration - action_duration;

            // 创建rest状态（抬指状态）
            let rest_state = self.create_rest_state(
                current_beat_state,
                press_distance,
                finger_index_set_need_to_change,
                normal,
            )?;

            let mut rest_end_frame = Map::new();
            rest_end_frame.insert(
                "frame".to_string(),
                Value::Number(serde_json::Number::from_f64(rest_end_time).unwrap()),
            );
            rest_end_frame.insert("fingerInfos".to_string(), Value::Object(rest_state));
            rest_end_frame.insert("pitchwheel".to_string(), Value::Number(pitchwheel.into()));
            frames_to_insert.push(rest_end_frame);

            if let Some(next_ready_state) = next_ready_state {
                let mut ready_frame = Map::new();
                ready_frame.insert(
                    "frame".to_string(),
                    Value::Number(serde_json::Number::from_f64(ready_time).unwrap()),
                );
                ready_frame.insert(
                    "fingerInfos".to_string(),
                    Value::Object(next_ready_state.clone()),
                );
                ready_frame.insert(
                    "pitchwheel".to_string(),
                    Value::Number(next_pitchwheel.into()),
                );
                frames_to_insert.push(ready_frame);
            }
        } else if t >= press_duration {
            // 时间只够插入预备状态，所以还是要填入预备状态的
            let ready_time = next_time - press_duration;

            if let Some(next_ready_state) = next_ready_state {
                let mut ready_frame = Map::new();
                ready_frame.insert(
                    "frame".to_string(),
                    Value::Number(serde_json::Number::from_f64(ready_time).unwrap()),
                );
                ready_frame.insert(
                    "fingerInfos".to_string(),
                    Value::Object(next_ready_state.clone()),
                );
                ready_frame.insert(
                    "pitchwheel".to_string(),
                    Value::Number(next_pitchwheel.into()),
                );
                frames_to_insert.push(ready_frame);
            }
        }
        // 情况3: 时间连插入预备状态都不够，只保留当前动作帧
        else {
        }

        Ok(frames_to_insert)
    }

    pub fn twice_lerp_vector3(
        &self,
        hand_state: i32,
        value: &str,
        value_type: &str,
        fret: f64,
    ) -> Result<Vector3, Box<dyn std::error::Error>> {
        let (p0_array, p1_array, p2_array, p3_array) = if value_type == "position" {
            // 处理位置数据
            let data_dict = self
                .get_avatar_nested_field(&["NORMAL_LEFT_HAND_POSITIONS"])
                .ok_or("Missing NORMAL_LEFT_HAND_POSITIONS in avatar data")?;

            let p0_array = data_dict
                .get("P0")
                .and_then(|v| v.get(value))
                .and_then(|v| v.as_array())
                .ok_or(format!(
                    "Missing P0[{}] in NORMAL_LEFT_HAND_POSITIONS",
                    value
                ))?;

            let p1_array = data_dict
                .get("P1")
                .and_then(|v| v.get(value))
                .and_then(|v| v.as_array())
                .ok_or(format!(
                    "Missing P1[{}] in NORMAL_LEFT_HAND_POSITIONS",
                    value
                ))?;

            let p2_array = data_dict
                .get("P2")
                .and_then(|v| v.get(value))
                .and_then(|v| v.as_array())
                .ok_or(format!(
                    "Missing P2[{}] in NORMAL_LEFT_HAND_POSITIONS",
                    value
                ))?;

            let p3_array = data_dict
                .get("P3")
                .and_then(|v| v.get(value))
                .and_then(|v| v.as_array())
                .ok_or(format!(
                    "Missing P3[{}] in NORMAL_LEFT_HAND_POSITIONS",
                    value
                ))?;

            (p0_array, p1_array, p2_array, p3_array)
        } else {
            // 处理旋转数据
            let data_dict = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Normal"])
                .ok_or("Missing ROTATIONS/H_rotation_L/Normal in avatar data")?;

            let p0_array = data_dict
                .get("P0")
                .and_then(|v| v.as_array())
                .ok_or("Missing P0 in ROTATIONS/H_rotation_L/Normal")?;

            let p1_array = data_dict
                .get("P1")
                .and_then(|v| v.as_array())
                .ok_or("Missing P1 in ROTATIONS/H_rotation_L/Normal")?;

            let p2_array = data_dict
                .get("P2")
                .and_then(|v| v.as_array())
                .ok_or("Missing P2 in ROTATIONS/H_rotation_L/Normal")?;

            let p3_array = data_dict
                .get("P3")
                .and_then(|v| v.as_array())
                .ok_or("Missing P3 in ROTATIONS/H_rotation_L/Normal")?;
            (p0_array, p1_array, p2_array, p3_array)
        };

        let p0_vec: Result<Vec<f64>, _> = p0_array
            .iter()
            .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
            .collect();

        let p1_vec: Result<Vec<f64>, _> = p1_array
            .iter()
            .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
            .collect();

        let p2_vec: Result<Vec<f64>, _> = p2_array
            .iter()
            .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
            .collect();

        let p3_vec: Result<Vec<f64>, _> = p3_array
            .iter()
            .map(|v| v.as_f64().ok_or("P3 values are not numbers"))
            .collect();

        let p0_v = p0_vec?;
        let p1_v = p1_vec?;
        let p2_v = p2_vec?;
        let p3_v = p3_vec?;

        if p0_v.len() != 3 || p1_v.len() != 3 || p2_v.len() != 3 || p3_v.len() != 3 {
            return Err("Vector3 data must have exactly 3 components".into());
        }

        let p0_vector = Vector3::from_vector64(p0_v);
        let p1_vector = Vector3::from_vector64(p1_v);
        let p2_vector = Vector3::from_vector64(p2_v);
        let p3_vector = Vector3::from_vector64(p3_v);

        let p_normal_fret_02 = lerp_by_fret_vector3(fret, &p0_vector, &p2_vector);
        let p_normal_fret_13 = lerp_by_fret_vector3(fret, &p1_vector, &p3_vector);

        let hand_state_weight = hand_state as f64 / self.max_string_index;

        if hand_state == 0 {
            Ok(lerp_by_fret_vector3(
                fret,
                &p_normal_fret_02,
                &p_normal_fret_13,
            ))
        } else if hand_state > 0 {
            let (out_p0_array, out_p2_array) = if value_type == "position" {
                let outer_data_dict = self
                    .get_avatar_nested_field(&["OUTER_LEFT_HAND_POSITIONS"])
                    .ok_or("Missing OUTER_LEFT_HAND_POSITIONS in avatar data")?;

                let out_p0_array = outer_data_dict
                    .get("P0")
                    .and_then(|v| v.get(value))
                    .and_then(|v| v.as_array())
                    .ok_or(format!(
                        "Missing P0[{}] in OUTER_LEFT_HAND_POSITIONS",
                        value
                    ))?;

                let out_p2_array = outer_data_dict
                    .get("P2")
                    .and_then(|v| v.get(value))
                    .and_then(|v| v.as_array())
                    .ok_or(format!(
                        "Missing P2[{}] in OUTER_LEFT_HAND_POSITIONS",
                        value
                    ))?;

                (out_p0_array, out_p2_array)
            } else {
                let outer_data_dict = self
                    .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Outer"])
                    .ok_or("Missing ROTATIONS/H_rotation_L/Outer in avatar data")?;

                let out_p0_array = outer_data_dict
                    .get("P0")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing P0 in ROTATIONS/H_rotation_L/Outer")?;

                let out_p2_array = outer_data_dict
                    .get("P2")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing P2 in ROTATIONS/H_rotation_L/Outer")?;

                (out_p0_array, out_p2_array)
            };

            let out_p0_vec: Result<Vec<f64>, _> = out_p0_array
                .iter()
                .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
                .collect();

            let out_p2_vec: Result<Vec<f64>, _> = out_p2_array
                .iter()
                .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
                .collect();

            let out_p0_v = out_p0_vec?;
            let out_p2_v = out_p2_vec?;

            if out_p0_v.len() != 3 || out_p2_v.len() != 3 {
                return Err("Vector3 data must have exactly 3 components".into());
            }

            let out_p0_vector = Vector3::from_vector64(out_p0_v);
            let out_p2_vector = Vector3::from_vector64(out_p2_v);

            // 这个变量的后缀0，表示的是在0-2两个位置中进行品格插值以后，得到的结果
            let p_outer = lerp_by_fret_vector3(fret, &out_p0_vector, &out_p2_vector);

            let p_normal = lerp_by_fret_vector3(fret, &p_normal_fret_02, &p_normal_fret_13);
            Ok(lerp_by_weight_vector3(
                &p_normal,
                &p_outer,
                hand_state_weight,
            ))
        } else {
            let (inner_p1_array, inner_p3_array) = if value_type == "position" {
                let inner_data_dict = self
                    .get_avatar_nested_field(&["INNER_LEFT_HAND_POSITIONS"])
                    .ok_or("Missing INNER_LEFT_HAND_POSITIONS in avatar data")?;

                let inner_p1_array = inner_data_dict
                    .get("P1")
                    .and_then(|v| v.get(value))
                    .and_then(|v| v.as_array())
                    .ok_or(format!(
                        "Missing P1[{}] in INNER_LEFT_HAND_POSITIONS",
                        value
                    ))?;

                let inner_p3_array = inner_data_dict
                    .get("P3")
                    .and_then(|v| v.get(value))
                    .and_then(|v| v.as_array())
                    .ok_or(format!(
                        "Missing P3[{}] in INNER_LEFT_HAND_POSITIONS",
                        value
                    ))?;

                (inner_p1_array, inner_p3_array)
            } else {
                let inner_data_dict = self
                    .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Inner"])
                    .ok_or("Missing ROTATIONS/H_rotation_L/Inner in avatar data")?;

                let inner_p1_array = inner_data_dict
                    .get("P1")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing P1 in ROTATIONS/H_rotation_L/Inner")?;

                let inner_p3_array = inner_data_dict
                    .get("P3")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing P3 in ROTATIONS/H_rotation_L/Inner")?;

                (inner_p1_array, inner_p3_array)
            };

            let inner_p1_vec: Result<Vec<f64>, _> = inner_p1_array
                .iter()
                .map(|v| v.as_f64().ok_or("inner_p1 values are not numbers"))
                .collect();

            let inner_p3_vec: Result<Vec<f64>, _> = inner_p3_array
                .iter()
                .map(|v| v.as_f64().ok_or("inner_p3 values are not numbers"))
                .collect();

            let inner_p1_v = inner_p1_vec?;
            let inner_p3_v = inner_p3_vec?;

            if inner_p1_v.len() != 3 || inner_p3_v.len() != 3 {
                return Err("Vector3 data must have exactly 3 components".into());
            }

            let inner_p1_vector = Vector3::from_vector64(inner_p1_v);
            let inner_p3_vector = Vector3::from_vector64(inner_p3_v);

            // 这个后缀1，是在1-3位置之间进行品格插值以后得到的结果
            let p_inner = lerp_by_fret_vector3(fret, &inner_p1_vector, &inner_p3_vector);

            let p_normal = lerp_by_fret_vector3(fret, &p_normal_fret_02, &p_normal_fret_13);
            Ok(lerp_by_weight_vector3(
                &p_normal,
                &p_inner,
                -hand_state_weight,
            ))
        }
    }

    pub fn twice_lerp_barre_hand_vector3(
        &self,
        hand_state: i32,
        value_type: &str,
        fret: f64,
    ) -> Result<Vector3, Box<dyn std::error::Error>> {
        // 这个值其实相当于食指的索引除以最大弦的索引，它与hand_state一起，
        // 可以表达出当前手型的食指和小拇指的弦索引，并且可以在三种不同手型中进行插值计算

        let hand_state_weight = hand_state as f64 / self.max_string_index;

        let (p0_array, p1_array, p2_array, p3_array) = if value_type == "position" {
            let p0_array = self
                .get_avatar_nested_field(&["BARRE_LEFT_HAND_POSITIONS", "P0", "H_L"])
                .ok_or("Missing Barre_P0_H_L in avatar data")?;

            let p1_array = self
                .get_avatar_nested_field(&["BARRE_LEFT_HAND_POSITIONS", "P1", "H_L"])
                .ok_or("Missing Barre_P1_H_L in avatar data")?;

            let p2_array = self
                .get_avatar_nested_field(&["BARRE_LEFT_HAND_POSITIONS", "P2", "H_L"])
                .ok_or("Missing Barre_P2_H_L in avatar data")?;

            let p3_array = self
                .get_avatar_nested_field(&["BARRE_LEFT_HAND_POSITIONS", "P3", "H_L"])
                .ok_or("Missing Barre_P3_H_L in avatar data")?;

            (p0_array, p1_array, p2_array, p3_array)
        } else {
            let p0_array = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Normal", "P0"])
                .ok_or("Missing ROTATIONS/H_rotation_L/Normal/P0 in avatar data")?;

            let p1_array = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Normal", "P1"])
                .ok_or("Missing ROTATIONS/H_rotation_L/Normal/P1 in avatar data")?;

            let p2_array = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Normal", "P2"])
                .ok_or("Missing ROTATIONS/H_rotation_L/Normal/P2 in avatar data")?;

            let p3_array = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Normal", "P3"])
                .ok_or("Missing ROTATIONS/H_rotation_L/Normal/P3 in avatar data")?;

            (p0_array, p1_array, p2_array, p3_array)
        };

        let p0_array = p0_array.as_array().ok_or("P0 is not an array")?;
        let p1_array = p1_array.as_array().ok_or("P1 is not an array")?;
        let p2_array = p2_array.as_array().ok_or("P2 is not an array")?;
        let p3_array = p3_array.as_array().ok_or("P3 is not an array")?;

        let p0_vec: Result<Vec<f64>, _> = p0_array
            .iter()
            .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
            .collect();

        let p1_vec: Result<Vec<f64>, _> = p1_array
            .iter()
            .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
            .collect();

        let p2_vec: Result<Vec<f64>, _> = p2_array
            .iter()
            .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
            .collect();

        let p3_vec: Result<Vec<f64>, _> = p3_array
            .iter()
            .map(|v| v.as_f64().ok_or("P3 values are not numbers"))
            .collect();

        let p0_v = p0_vec?;
        let p1_v = p1_vec?;
        let p2_v = p2_vec?;
        let p3_v = p3_vec?;

        if p0_v.len() != 3 || p1_v.len() != 3 || p2_v.len() != 3 || p3_v.len() != 3 {
            return Err("Vector3 data must have exactly 3 components".into());
        }

        let p0_vector = Vector3::from_vector64(p0_v);
        let p1_vector = Vector3::from_vector64(p1_v);
        let p2_vector = Vector3::from_vector64(p2_v);
        let p3_vector = Vector3::from_vector64(p3_v);

        let p_normal_fret_02 = lerp_by_fret_vector3(fret, &p0_vector, &p2_vector);
        let p_normal_fret_13 = lerp_by_fret_vector3(fret, &p1_vector, &p3_vector);

        if hand_state == 0 {
            Ok(lerp_by_fret_vector3(
                fret,
                &p_normal_fret_02,
                &p_normal_fret_13,
            ))
        } else if hand_state > 0 {
            let (out_p0_array, out_p2_array) = if value_type == "position" {
                let out_p0_array = self
                    .get_avatar_nested_field(&["OUTER_LEFT_HAND_POSITIONS", "P0", "H_L"])
                    .ok_or("Missing OUTER_LEFT_HAND_POSITIONS/P0 in avatar data")?;

                let out_p2_array = self
                    .get_avatar_nested_field(&["OUTER_LEFT_HAND_POSITIONS", "P2", "H_L"])
                    .ok_or("Missing OUTER_LEFT_HAND_POSITIONS/P2 in avatar data")?;
                (out_p0_array, out_p2_array)
            } else {
                let out_p0_array = self
                    .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Outer", "P0"])
                    .ok_or("Missing ROTATIONS/H_rotation_L/Outer/P0 in avatar data")?;

                let out_p2_array = self
                    .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Outer", "P2"])
                    .ok_or("Missing ROTATIONS/H_rotation_L/Outer/P2 in avatar data")?;

                (out_p0_array, out_p2_array)
            };

            let out_p0_array = out_p0_array.as_array().ok_or("Outer P0 is not an array")?;
            let out_p2_array = out_p2_array.as_array().ok_or("Outer P2 is not an array")?;

            let out_p0_vec: Result<Vec<f64>, _> = out_p0_array
                .iter()
                .map(|v| v.as_f64().ok_or("out_p0 values are not numbers"))
                .collect();

            let out_p2_vec: Result<Vec<f64>, _> = out_p2_array
                .iter()
                .map(|v| v.as_f64().ok_or("out_p2 values are not numbers"))
                .collect();

            let out_p0_v = out_p0_vec?;
            let out_p2_v = out_p2_vec?;

            if out_p0_v.len() != 3 || out_p2_v.len() != 3 {
                return Err("Vector3 data must have exactly 3 components".into());
            }

            let out_p0_vector = Vector3::from_vector64(out_p0_v);
            let out_p2_vector = Vector3::from_vector64(out_p2_v);

            // 这个变量的后缀0，表示的是在0-2两个位置中进行品格插值以后，得到的结果
            let p_outer = lerp_by_fret_vector3(fret, &out_p0_vector, &out_p2_vector);

            let p_normal = lerp_by_fret_vector3(fret, &p_normal_fret_02, &p_normal_fret_13);
            Ok(lerp_by_weight_vector3(
                &p_normal,
                &p_outer,
                hand_state_weight,
            ))
        } else {
            let (inner_p1_array, inner_p3_array) = if value_type == "position" {
                let inner_p1_array = self
                    .get_avatar_nested_field(&["INNER_LEFT_HAND_POSITIONS", "P1", "H_L"])
                    .ok_or("Missing INNER_LEFT_HAND_POSITIONS/P1 in avatar data")?;

                let inner_p3_array = self
                    .get_avatar_nested_field(&["INNER_LEFT_HAND_POSITIONS", "P3", "H_L"])
                    .ok_or("Missing INNER_LEFT_HAND_POSITIONS/P3 in avatar data")?;

                (inner_p1_array, inner_p3_array)
            } else {
                let inner_p1_array = self
                    .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Inner", "P1"])
                    .ok_or("Missing ROTATIONS/H_rotation_L/Inner/P1 in avatar data")?;

                let inner_p3_array = self
                    .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Inner", "P3"])
                    .ok_or("Missing ROTATIONS/H_rotation_L/Inner/P3 in avatar data")?;

                (inner_p1_array, inner_p3_array)
            };

            let inner_p1_array = inner_p1_array
                .as_array()
                .ok_or("Inner P1 is not an array")?;
            let inner_p3_array = inner_p3_array
                .as_array()
                .ok_or("Inner P3 is not an array")?;

            let inner_p1_vec: Result<Vec<f64>, _> = inner_p1_array
                .iter()
                .map(|v| v.as_f64().ok_or("inner_p1 values are not numbers"))
                .collect();

            let inner_p3_vec: Result<Vec<f64>, _> = inner_p3_array
                .iter()
                .map(|v| v.as_f64().ok_or("inner_p3 values are not numbers"))
                .collect();

            let inner_p1_v = inner_p1_vec?;
            let inner_p3_v = inner_p3_vec?;

            if inner_p1_v.len() != 3 || inner_p3_v.len() != 3 {
                return Err("Vector3 data must have exactly 3 components".into());
            }

            let inner_p1_vector = Vector3::from_vector64(inner_p1_v);
            let inner_p3_vector = Vector3::from_vector64(inner_p3_v);

            // 这个后缀1，是在1-3位置之间进行品格插值以后得到的结果
            let p_inner = lerp_by_fret_vector3(fret, &inner_p1_vector, &inner_p3_vector);

            let p_normal = lerp_by_fret_vector3(fret, &p_normal_fret_02, &p_normal_fret_13);
            Ok(lerp_by_weight_vector3(
                &p_normal,
                &p_inner,
                -hand_state_weight,
            ))
        }
    }

    pub fn twice_lerp_quaternion(
        &self,
        hand_state: i32,
        fret: f64,
        string_index: f64,
    ) -> Result<Quaternion, Box<dyn std::error::Error>> {
        // 这个值其实相当于食指的索引除以最大弦的索引，它与hand_state一起，
        // 可以表达出当前手型的食指和小拇指的弦索引，并且可以在三种不同手型中进行插值计算
        let string_weight = string_index / self.max_string_index;
        let hand_weight = (hand_state as f64).abs() / self.max_string_index;

        let (p0, p1, p2, p3) = {
            // 处理旋转数据
            let data_dict = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Normal"])
                .ok_or("Missing ROTATIONS/H_rotation_L/Normal in avatar data")?;

            let p0_array = data_dict
                .get("P0")
                .and_then(|v| v.as_array())
                .ok_or("Missing P0 in ROTATIONS/H_rotation_L/Normal")?;

            let p1_array = data_dict
                .get("P1")
                .and_then(|v| v.as_array())
                .ok_or("Missing P1 in ROTATIONS/H_rotation_L/Normal")?;

            let p2_array = data_dict
                .get("P2")
                .and_then(|v| v.as_array())
                .ok_or("Missing P2 in ROTATIONS/H_rotation_L/Normal")?;

            let p3_array = data_dict
                .get("P3")
                .and_then(|v| v.as_array())
                .ok_or("Missing P3 in ROTATIONS/H_rotation_L/Normal")?;

            let p0_vec: Result<Vec<f64>, _> = p0_array
                .iter()
                .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
                .collect();

            let p1_vec: Result<Vec<f64>, _> = p1_array
                .iter()
                .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
                .collect();

            let p2_vec: Result<Vec<f64>, _> = p2_array
                .iter()
                .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
                .collect();

            let p3_vec: Result<Vec<f64>, _> = p3_array
                .iter()
                .map(|v| v.as_f64().ok_or("P3 values are not numbers"))
                .collect();

            let p0_v = p0_vec?;
            let p1_v = p1_vec?;
            let p2_v = p2_vec?;
            let p3_v = p3_vec?;

            if p0_v.len() != 4 || p1_v.len() != 4 || p2_v.len() != 4 || p3_v.len() != 4 {
                return Err("Quaternion data must have exactly 4 components".into());
            }

            let p0_quat = Quaternion::from_vector64(p0_v);
            let p1_quat = Quaternion::from_vector64(p1_v);
            let p2_quat = Quaternion::from_vector64(p2_v);
            let p3_quat = Quaternion::from_vector64(p3_v);

            (p0_quat, p1_quat, p2_quat, p3_quat)
        };

        let p_normal_fret_02 = lerp_by_fret_quaternion(fret, &p0, &p2);
        let p_normal_fret_13 = lerp_by_fret_quaternion(fret, &p1, &p3);

        if hand_state == 0 {
            let result = slerp(&p_normal_fret_02, &p_normal_fret_13, string_weight);
            Ok(result)
        } else if hand_state > 0 {
            let (out_p0, out_p2) = {
                let outer_data_dict = self
                    .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Outer"])
                    .ok_or("Missing ROTATIONS/H_rotation_L/Outer in avatar data")?;

                let out_p0_array = outer_data_dict
                    .get("P0")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing P0 in ROTATIONS/H_rotation_L/Outer")?;

                let out_p2_array = outer_data_dict
                    .get("P2")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing P2 in ROTATIONS/H_rotation_L/Outer")?;

                let out_p0_vec: Result<Vec<f64>, _> = out_p0_array
                    .iter()
                    .map(|v| v.as_f64().ok_or("out_p0 values are not numbers"))
                    .collect();

                let out_p2_vec: Result<Vec<f64>, _> = out_p2_array
                    .iter()
                    .map(|v| v.as_f64().ok_or("out_p2 values are not numbers"))
                    .collect();

                let out_p0_v = out_p0_vec?;
                let out_p2_v = out_p2_vec?;

                if out_p0_v.len() != 4 || out_p2_v.len() != 4 {
                    return Err("Quaternion data must have exactly 4 components".into());
                }

                let out_p0_quat =
                    Quaternion::new(out_p0_v[0], out_p0_v[1], out_p0_v[2], out_p0_v[3]);
                let out_p2_quat =
                    Quaternion::new(out_p2_v[0], out_p2_v[1], out_p2_v[2], out_p2_v[3]);

                (out_p0_quat, out_p2_quat)
            };

            // 这个变量的后缀0，表示的是在0-2两个位置中进行品格插值以后，得到的结果
            let p_outer = lerp_by_fret_quaternion(fret, &out_p0, &out_p2);

            let p_normal = slerp(&p_normal_fret_02, &p_normal_fret_13, string_weight);
            let result = slerp(&p_normal, &p_outer, hand_weight);
            Ok(result)
        } else {
            let (inner_p1, inner_p3) = {
                let inner_data_dict = self
                    .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Inner"])
                    .ok_or("Missing ROTATIONS/H_rotation_L/Inner in avatar data")?;

                let inner_p1_array = inner_data_dict
                    .get("P1")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing P1 in ROTATIONS/H_rotation_L/Inner")?;

                let inner_p3_array = inner_data_dict
                    .get("P3")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing P3 in ROTATIONS/H_rotation_L/Inner")?;

                let inner_p1_vec: Result<Vec<f64>, _> = inner_p1_array
                    .iter()
                    .map(|v| v.as_f64().ok_or("inner_p1 values are not numbers"))
                    .collect();

                let inner_p3_vec: Result<Vec<f64>, _> = inner_p3_array
                    .iter()
                    .map(|v| v.as_f64().ok_or("inner_p3 values are not numbers"))
                    .collect();

                let inner_p1_v = inner_p1_vec?;
                let inner_p3_v = inner_p3_vec?;

                if inner_p1_v.len() != 4 || inner_p3_v.len() != 4 {
                    return Err("Quaternion data must have exactly 4 components".into());
                }

                let inner_p1_quat =
                    Quaternion::new(inner_p1_v[0], inner_p1_v[1], inner_p1_v[2], inner_p1_v[3]);
                let inner_p3_quat =
                    Quaternion::new(inner_p3_v[0], inner_p3_v[1], inner_p3_v[2], inner_p3_v[3]);

                (inner_p1_quat, inner_p3_quat)
            };

            // 这个后缀1，是在1-3位置之间进行品格插值以后得到的结果
            let p_inner = lerp_by_fret_quaternion(fret, &inner_p1, &inner_p3);

            let p_normal = slerp(&p_normal_fret_02, &p_normal_fret_13, string_weight);
            let result = slerp(&p_normal, &p_inner, hand_weight);
            Ok(result)
        }
    }

    pub fn twice_lerp_fingers(
        &self,
        fret: f64,
        string_index: f64,
    ) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        // 获取LEFT_FINGER_POSITIONS数据
        let left_finger_positions = self
            .get_avatar_field("LEFT_FINGER_POSITIONS")
            .ok_or("Missing LEFT_FINGER_POSITIONS in avatar data")?;

        // 提取P0, P1, P2, P3数组
        let p0_array = left_finger_positions
            .get("P0")
            .and_then(|v| v.as_array())
            .ok_or("Missing P0 in LEFT_FINGER_POSITIONS")?;

        let p1_array = left_finger_positions
            .get("P1")
            .and_then(|v| v.as_array())
            .ok_or("Missing P1 in LEFT_FINGER_POSITIONS")?;

        let p2_array = left_finger_positions
            .get("P2")
            .and_then(|v| v.as_array())
            .ok_or("Missing P2 in LEFT_FINGER_POSITIONS")?;

        let p3_array = left_finger_positions
            .get("P3")
            .and_then(|v| v.as_array())
            .ok_or("Missing P3 in LEFT_FINGER_POSITIONS")?;

        // 转换为f64向量
        let p0: Result<Vec<f64>, _> = p0_array
            .iter()
            .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
            .collect();
        let p0 = p0?;

        let p1: Result<Vec<f64>, _> = p1_array
            .iter()
            .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
            .collect();
        let p1 = p1?;

        let p2: Result<Vec<f64>, _> = p2_array
            .iter()
            .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
            .collect();
        let p2 = p2?;

        let p3: Result<Vec<f64>, _> = p3_array
            .iter()
            .map(|v| v.as_f64().ok_or("P3 values are not numbers"))
            .collect();
        let p3 = p3?;

        // 执行品格插值
        let p0_vec = Vector3::from_vector64(p0);
        let p1_vec = Vector3::from_vector64(p1);
        let p2_vec = Vector3::from_vector64(p2);
        let p3_vec = Vector3::from_vector64(p3);
        let p_fret_0 = lerp_by_fret_vector3(fret, &p0_vec, &p2_vec);
        let p_fret_1 = lerp_by_fret_vector3(fret, &p1_vec, &p3_vec);

        // 执行弦索引插值
        let string_weight = string_index / self.max_string_index;
        let p_fret_0 = p_fret_0.to_vector64();
        let p_fret_1 = p_fret_1.to_vector64();
        let p_final: Vec<f64> = p_fret_0
            .iter()
            .zip(p_fret_1.iter())
            .map(|(&a, &b)| a + (b - a) * string_weight)
            .collect();

        Ok(p_final)
    }

    pub fn twice_lerp_barre_fingers(
        &self,
        fret: f64,
        finger_string_index: f64,
    ) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        // 获取BARRE_LEFT_HAND_POSITIONS数据
        let barre_left_hand_positions = self
            .get_avatar_field("BARRE_LEFT_HAND_POSITIONS")
            .ok_or("Missing BARRE_LEFT_HAND_POSITIONS in avatar data")?;

        // 提取各个位置的I_L数据
        let barre_p0_array = barre_left_hand_positions
            .get("P0")
            .and_then(|v| v.get("I_L"))
            .and_then(|v| v.as_array())
            .ok_or("Missing P0/I_L in BARRE_LEFT_HAND_POSITIONS")?;

        let barre_p1_array = barre_left_hand_positions
            .get("P1")
            .and_then(|v| v.get("I_L"))
            .and_then(|v| v.as_array())
            .ok_or("Missing P1/I_L in BARRE_LEFT_HAND_POSITIONS")?;

        let barre_p2_array = barre_left_hand_positions
            .get("P2")
            .and_then(|v| v.get("I_L"))
            .and_then(|v| v.as_array())
            .ok_or("Missing P2/I_L in BARRE_LEFT_HAND_POSITIONS")?;

        let barre_p3_array = barre_left_hand_positions
            .get("P3")
            .and_then(|v| v.get("I_L"))
            .and_then(|v| v.as_array())
            .ok_or("Missing P3/I_L in BARRE_LEFT_HAND_POSITIONS")?;

        // 转换为f64向量
        let barre_p0: Result<Vec<f64>, _> = barre_p0_array
            .iter()
            .map(|v| v.as_f64().ok_or("barre_p0 values are not numbers"))
            .collect();
        let barre_p0 = barre_p0?;

        let barre_p1: Result<Vec<f64>, _> = barre_p1_array
            .iter()
            .map(|v| v.as_f64().ok_or("barre_p1 values are not numbers"))
            .collect();
        let barre_p1 = barre_p1?;

        let barre_p2: Result<Vec<f64>, _> = barre_p2_array
            .iter()
            .map(|v| v.as_f64().ok_or("barre_p2 values are not numbers"))
            .collect();
        let barre_p2 = barre_p2?;

        let barre_p3: Result<Vec<f64>, _> = barre_p3_array
            .iter()
            .map(|v| v.as_f64().ok_or("barre_p3 values are not numbers"))
            .collect();
        let barre_p3 = barre_p3?;

        // 执行品格插值
        let barre_p0_vec = Vector3::from_vector64(barre_p0);
        let barre_p1_vec = Vector3::from_vector64(barre_p1);
        let barre_p2_vec = Vector3::from_vector64(barre_p2);
        let barre_p3_vec = Vector3::from_vector64(barre_p3);
        let p_fret_0 = lerp_by_fret_vector3(fret, &barre_p0_vec, &barre_p2_vec);
        let p_fret_1 = lerp_by_fret_vector3(fret, &barre_p1_vec, &barre_p3_vec);

        // 使用clamp后的值进行计算
        let string_weight = (finger_string_index - 2.0) / (self.max_string_index - 2.0);
        let p_fret_0 = p_fret_0.to_vector64();
        let p_fret_1 = p_fret_1.to_vector64();
        let p_final: Vec<f64> = p_fret_0
            .iter()
            .zip(p_fret_1.iter())
            .map(|(&a, &b)| a + (b - a) * string_weight)
            .collect();

        Ok(p_final)
    }

    pub fn twice_lerp_barre_vector3(
        &self,
        value: &str,
        value_type: &str,
        fret: f64,
    ) -> Result<Vector3, Box<dyn std::error::Error>> {
        let (p0, p1, p2, p3) = if value_type == "position" {
            let data_dict = self
                .get_avatar_field("BARRE_LEFT_HAND_POSITIONS")
                .ok_or("Missing BARRE_LEFT_HAND_POSITIONS in avatar data")?;

            let p0_array = data_dict
                .get("P0")
                .and_then(|v| v.get(value))
                .and_then(|v| v.as_array())
                .ok_or(format!(
                    "Missing P0[{}] in BARRE_LEFT_HAND_POSITIONS",
                    value
                ))?;

            let p1_array = data_dict
                .get("P1")
                .and_then(|v| v.get(value))
                .and_then(|v| v.as_array())
                .ok_or(format!(
                    "Missing P1[{}] in BARRE_LEFT_HAND_POSITIONS",
                    value
                ))?;

            let p2_array = data_dict
                .get("P2")
                .and_then(|v| v.get(value))
                .and_then(|v| v.as_array())
                .ok_or(format!(
                    "Missing P2[{}] in BARRE_LEFT_HAND_POSITIONS",
                    value
                ))?;

            let p3_array = data_dict
                .get("P3")
                .and_then(|v| v.get(value))
                .and_then(|v| v.as_array())
                .ok_or(format!(
                    "Missing P3[{}] in BARRE_LEFT_HAND_POSITIONS",
                    value
                ))?;

            // 检查数据有效性
            if p0_array.is_empty()
                || p1_array.is_empty()
                || p2_array.is_empty()
                || p3_array.is_empty()
            {
                return Err(format!("Invalid position data: {}", value).into());
            }

            let p0_vec: Result<Vec<f64>, _> = p0_array
                .iter()
                .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
                .collect();

            let p1_vec: Result<Vec<f64>, _> = p1_array
                .iter()
                .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
                .collect();

            let p2_vec: Result<Vec<f64>, _> = p2_array
                .iter()
                .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
                .collect();

            let p3_vec: Result<Vec<f64>, _> = p3_array
                .iter()
                .map(|v| v.as_f64().ok_or("P3 values are not numbers"))
                .collect();

            let p0_v = p0_vec?;
            let p1_v = p1_vec?;
            let p2_v = p2_vec?;
            let p3_v = p3_vec?;

            if p0_v.len() != 3 || p1_v.len() != 3 || p2_v.len() != 3 || p3_v.len() != 3 {
                return Err("Vector3 data must have exactly 3 components".into());
            }

            let p0_vector = Vector3::new(p0_v[0], p0_v[1], p0_v[2]);
            let p1_vector = Vector3::new(p1_v[0], p1_v[1], p1_v[2]);
            let p2_vector = Vector3::new(p2_v[0], p2_v[1], p2_v[2]);
            let p3_vector = Vector3::new(p3_v[0], p3_v[1], p3_v[2]);

            (p0_vector, p1_vector, p2_vector, p3_vector)
        } else if value_type == "rotation" {
            let data_dict = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Barre"])
                .ok_or("Missing ROTATIONS/H_rotation_L/Barre in avatar data")?;

            let p0_array = data_dict
                .get("P0")
                .and_then(|v| v.as_array())
                .ok_or("Missing P0 in ROTATIONS/H_rotation_L/Barre")?;

            let p1_array = data_dict
                .get("P1")
                .and_then(|v| v.as_array())
                .ok_or("Missing P1 in ROTATIONS/H_rotation_L/Barre")?;

            let p2_array = data_dict
                .get("P2")
                .and_then(|v| v.as_array())
                .ok_or("Missing P2 in ROTATIONS/H_rotation_L/Barre")?;

            let p3_array = data_dict
                .get("P3")
                .and_then(|v| v.as_array())
                .ok_or("Missing P3 in ROTATIONS/H_rotation_L/Barre")?;

            // 检查是否为四元数（长度为4）或欧拉角（长度为3）
            if p0_array.len() != 3
                || p1_array.len() != 3
                || p2_array.len() != 3
                || p3_array.len() != 3
            {
                return Err(
                    "Barre rotation data must be in Vector3 format (length 3) for this method"
                        .into(),
                );
            }

            let p0_vec: Result<Vec<f64>, _> = p0_array
                .iter()
                .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
                .collect();

            let p1_vec: Result<Vec<f64>, _> = p1_array
                .iter()
                .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
                .collect();

            let p2_vec: Result<Vec<f64>, _> = p2_array
                .iter()
                .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
                .collect();

            let p3_vec: Result<Vec<f64>, _> = p3_array
                .iter()
                .map(|v| v.as_f64().ok_or("P3 values are not numbers"))
                .collect();

            let p0_v = p0_vec?;
            let p1_v = p1_vec?;
            let p2_v = p2_vec?;
            let p3_v = p3_vec?;

            let p0_vector = Vector3::new(p0_v[0], p0_v[1], p0_v[2]);
            let p1_vector = Vector3::new(p1_v[0], p1_v[1], p1_v[2]);
            let p2_vector = Vector3::new(p2_v[0], p2_v[1], p2_v[2]);
            let p3_vector = Vector3::new(p3_v[0], p3_v[1], p3_v[2]);

            (p0_vector, p1_vector, p2_vector, p3_vector)
        } else {
            return Err("Invalid value type".into());
        };

        let p_fret_02 = lerp_by_fret_vector3(fret, &p0, &p2);
        let p_fret_13 = lerp_by_fret_vector3(fret, &p1, &p3);

        let p_normal = lerp_by_fret_vector3(fret, &p_fret_02, &p_fret_13);
        Ok(lerp_by_fret_vector3(fret, &p_normal, &p_normal)) // 这里使用相同参数，因为不需要额外插值
    }

    pub fn twice_lerp_barre_quaternion(
        &self,
        value_type: &str,
        fret: f64,
        string_index: f64,
    ) -> Result<Quaternion, Box<dyn std::error::Error>> {
        let string_weight = (string_index - 2.0) / (self.max_string_index - 2.0);

        let (p0, p1, p2, p3) = if value_type == "position" {
            let data_dict = self
                .get_avatar_field("BARRE_LEFT_HAND_POSITIONS")
                .ok_or("Missing BARRE_LEFT_HAND_POSITIONS in avatar data")?;

            let p0_array = data_dict
                .get("P0")
                .and_then(|v| v.get("H_rotation_L"))
                .and_then(|v| v.as_array())
                .ok_or("Missing P0[H_rotation_L] in BARRE_LEFT_HAND_POSITIONS")?;

            let p1_array = data_dict
                .get("P1")
                .and_then(|v| v.get("H_rotation_L"))
                .and_then(|v| v.as_array())
                .ok_or("Missing P1[H_rotation_L] in BARRE_LEFT_HAND_POSITIONS")?;

            let p2_array = data_dict
                .get("P2")
                .and_then(|v| v.get("H_rotation_L"))
                .and_then(|v| v.as_array())
                .ok_or("Missing P2[H_rotation_L] in BARRE_LEFT_HAND_POSITIONS")?;

            let p3_array = data_dict
                .get("P3")
                .and_then(|v| v.get("H_rotation_L"))
                .and_then(|v| v.as_array())
                .ok_or("Missing P3[H_rotation_L] in BARRE_LEFT_HAND_POSITIONS")?;

            let p0_vec: Result<Vec<f64>, _> = p0_array
                .iter()
                .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
                .collect();

            let p1_vec: Result<Vec<f64>, _> = p1_array
                .iter()
                .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
                .collect();

            let p2_vec: Result<Vec<f64>, _> = p2_array
                .iter()
                .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
                .collect();

            let p3_vec: Result<Vec<f64>, _> = p3_array
                .iter()
                .map(|v| v.as_f64().ok_or("P3 values are not numbers"))
                .collect();

            let p0_v = p0_vec?;
            let p1_v = p1_vec?;
            let p2_v = p2_vec?;
            let p3_v = p3_vec?;

            if p0_v.len() != 4 || p1_v.len() != 4 || p2_v.len() != 4 || p3_v.len() != 4 {
                return Err("Quaternion data must have exactly 4 components".into());
            }

            let p0_quat = Quaternion::new(p0_v[0], p0_v[1], p0_v[2], p0_v[3]);
            let p1_quat = Quaternion::new(p1_v[0], p1_v[1], p1_v[2], p1_v[3]);
            let p2_quat = Quaternion::new(p2_v[0], p2_v[1], p2_v[2], p2_v[3]);
            let p3_quat = Quaternion::new(p3_v[0], p3_v[1], p3_v[2], p3_v[3]);

            (p0_quat, p1_quat, p2_quat, p3_quat)
        } else if value_type == "rotation" {
            let data_dict = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_L", "Barre"])
                .ok_or("Missing ROTATIONS/H_rotation_L/Barre in avatar data")?;

            let p0_array = data_dict
                .get("P0")
                .and_then(|v| v.as_array())
                .ok_or("Missing P0 in ROTATIONS/H_rotation_L/Barre")?;

            let p1_array = data_dict
                .get("P1")
                .and_then(|v| v.as_array())
                .ok_or("Missing P1 in ROTATIONS/H_rotation_L/Barre")?;

            let p2_array = data_dict
                .get("P2")
                .and_then(|v| v.as_array())
                .ok_or("Missing P2 in ROTATIONS/H_rotation_L/Barre")?;

            let p3_array = data_dict
                .get("P3")
                .and_then(|v| v.as_array())
                .ok_or("Missing P3 in ROTATIONS/H_rotation_L/Barre")?;

            // 检查是否为四元数（长度为4）
            if p0_array.len() != 4
                || p1_array.len() != 4
                || p2_array.len() != 4
                || p3_array.len() != 4
            {
                return Err(
                    "Barre rotation data must be in quaternion format (length 4) for this method"
                        .into(),
                );
            }

            let p0_vec: Result<Vec<f64>, _> = p0_array
                .iter()
                .map(|v| v.as_f64().ok_or("P0 values are not numbers"))
                .collect();

            let p1_vec: Result<Vec<f64>, _> = p1_array
                .iter()
                .map(|v| v.as_f64().ok_or("P1 values are not numbers"))
                .collect();

            let p2_vec: Result<Vec<f64>, _> = p2_array
                .iter()
                .map(|v| v.as_f64().ok_or("P2 values are not numbers"))
                .collect();

            let p3_vec: Result<Vec<f64>, _> = p3_array
                .iter()
                .map(|v| v.as_f64().ok_or("P3 values are not numbers"))
                .collect();

            let p0_v = p0_vec?;
            let p1_v = p1_vec?;
            let p2_v = p2_vec?;
            let p3_v = p3_vec?;

            let p0_quat = Quaternion::new(p0_v[0], p0_v[1], p0_v[2], p0_v[3]);
            let p1_quat = Quaternion::new(p1_v[0], p1_v[1], p1_v[2], p1_v[3]);
            let p2_quat = Quaternion::new(p2_v[0], p2_v[1], p2_v[2], p2_v[3]);
            let p3_quat = Quaternion::new(p3_v[0], p3_v[1], p3_v[2], p3_v[3]);

            (p0_quat, p1_quat, p2_quat, p3_quat)
        } else {
            return Err("Invalid quaternion value type".into());
        };

        let p_fret_02 = lerp_by_fret_quaternion(fret, &p0, &p2);
        let p_fret_13 = lerp_by_fret_quaternion(fret, &p1, &p3);

        let result = slerp(&p_fret_02, &p_fret_13, string_weight);
        Ok(result)
    }

    pub fn create_rest_state(
        &self,
        beat_state: &Map<String, Value>,
        press_distance: f64,
        finger_index_set_need_to_change: &std::collections::HashSet<i32>,
        normal: &[f64],
    ) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        /* 创建手指抬高的休息状态 */
        // 复制当前状态
        let mut rest_state = beat_state.clone();

        // 抬高需要改变的手指
        for &rest_finger_index in finger_index_set_need_to_change {
            if let Some(controller_name) = LEFT_FINGER_INDEX_DICT.iter().find_map(|&(idx, name)| {
                if idx == rest_finger_index {
                    Some(name)
                } else {
                    None
                }
            }) {
                if let Some(position_value) = rest_state.get(controller_name) {
                    let position_array = position_value
                        .as_array()
                        .ok_or(format!("{} is not an array", controller_name))?;

                    let current_position: Result<Vec<f64>, _> = position_array
                        .iter()
                        .map(|v| v.as_f64().ok_or("Position component is not a number"))
                        .collect();

                    let current_position = current_position?;

                    // 小拇指休息时比其它手指抬得要高一点
                    let new_position: Vec<Value> = if rest_finger_index == 4 {
                        current_position
                            .iter()
                            .enumerate()
                            .map(|(i, &pos)| {
                                let new_pos = pos - 2.0 * normal[i] * press_distance;
                                Value::Number(serde_json::Number::from_f64(new_pos).unwrap())
                            })
                            .collect()
                    } else {
                        current_position
                            .iter()
                            .enumerate()
                            .map(|(i, &pos)| {
                                let new_pos = pos - normal[i] * press_distance;
                                Value::Number(serde_json::Number::from_f64(new_pos).unwrap())
                            })
                            .collect()
                    };

                    rest_state.insert(controller_name.to_string(), Value::Array(new_position));
                }
            }
        }

        // rest状态的手随机动一点
        if let Some(h_l_value) = rest_state.get("H_L") {
            let h_l_array = h_l_value.as_array().ok_or("H_L value is not an array")?;

            let current_h_position: Result<Vec<f64>, _> = h_l_array
                .iter()
                .map(|v| v.as_f64().ok_or("H_L component is not a number"))
                .collect();

            let mut current_h_position = current_h_position?;

            // 生成随机向量
            let mut rng = rand::thread_rng();
            let mut random_vector: [f64; 3] = [
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
            ];

            // 归一化随机向量
            let norm = (random_vector[0] * random_vector[0]
                + random_vector[1] * random_vector[1]
                + random_vector[2] * random_vector[2])
                .sqrt();

            for i in 0..3 {
                random_vector[i] /= norm;
                current_h_position[i] += random_vector[i] * press_distance * 0.5;
            }

            let h_l_position: Vec<Value> = current_h_position
                .iter()
                .map(|&pos| serde_json::Value::Number(serde_json::Number::from_f64(pos).unwrap()))
                .collect();

            rest_state.insert("H_L".to_string(), Value::Array(h_l_position));
        }

        Ok(rest_state)
    }

    pub fn create_init_state(
        &self,
        item: &Map<String, Value>,
        normal: &[f64],
        pitchwheel: i32,
        press_distance: f64,
        disable_barre: bool,
    ) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        // 复制当前状态作为基础
        let mut init_finger_infos =
            self.animated_left_hand(item, normal, pitchwheel, press_distance, disable_barre)?;

        for (finger_index, controller_name) in &LEFT_FINGER_INDEX_DICT {
            if let Some(position_value) = init_finger_infos.get(*controller_name) {
                let position_array = position_value
                    .as_array()
                    .ok_or("Position value is not an array")?;

                let current_position: Result<Vec<f64>, _> = position_array
                    .iter()
                    .map(|v| v.as_f64().ok_or("Position component is not a number"))
                    .collect();

                let current_position = current_position?;

                // 手指休息时比按弦时抬高一些
                let new_position: Vec<Value> = if *finger_index == 4 {
                    // 小拇指抬得更高
                    current_position
                        .iter()
                        .enumerate()
                        .map(|(i, &pos)| {
                            let new_pos = pos - 2.0 * normal[i] * press_distance;
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(new_pos).unwrap(),
                            )
                        })
                        .collect()
                } else {
                    current_position
                        .iter()
                        .enumerate()
                        .map(|(i, &pos)| {
                            let new_pos = pos - normal[i] * press_distance;
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(new_pos).unwrap(),
                            )
                        })
                        .collect()
                };

                init_finger_infos.insert(controller_name.to_string(), Value::Array(new_position));
            }
        }

        // 休息状态的手可以往后一点点位置
        if let Some(h_l_value) = init_finger_infos.get("H_L") {
            let h_l_array = h_l_value.as_array().ok_or("H_L value is not an array")?;

            let current_h_position: Result<Vec<f64>, _> = h_l_array
                .iter()
                .map(|v| v.as_f64().ok_or("H_L component is not a number"))
                .collect();

            let mut current_h_position = current_h_position?;

            // 生成随机向量
            let mut rng = rand::thread_rng();
            let mut random_vector: [f64; 3] = [
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
            ];

            // 归一化随机向量
            let norm = (random_vector[0] * random_vector[0]
                + random_vector[1] * random_vector[1]
                + random_vector[2] * random_vector[2])
                .sqrt();

            for i in 0..3 {
                random_vector[i] /= norm;
                current_h_position[i] += random_vector[i] * press_distance * 0.5;
            }

            let h_l_position: Vec<Value> = current_h_position
                .iter()
                .map(|&pos| serde_json::Value::Number(serde_json::Number::from_f64(pos).unwrap()))
                .collect();

            init_finger_infos.insert("H_L".to_string(), Value::Array(h_l_position));
        }

        Ok(init_finger_infos)
    }

    pub fn calculate_right_pick(
        &self,
        string_index: i32,
        is_arpeggio: bool,
        should_stay_at_lower_position: bool,
    ) -> Result<HashMap<String, Vec<f64>>, Box<dyn Error>> {
        let mut result = HashMap::new();

        if is_arpeggio && should_stay_at_lower_position {
            let t_r = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "pend"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/pend in avatar data")?
                .as_array()
                .ok_or("pend is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("pend values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let h_r = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "Normal_Pend_H_R"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/Normal_Pend_H_R in avatar data")?
                .as_array()
                .ok_or("Normal_Pend_H_R is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("Normal_Pend_H_R values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let hp_r = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "Normal_Pend_HP_R"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/Normal_Pend_HP_R in avatar data")?
                .as_array()
                .ok_or("Normal_Pend_HP_R is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("Normal_Pend_HP_R values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let h_rotation_r = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_R", "Normal", "Pend"])
                .ok_or("Missing ROTATIONS/H_rotation_R/Normal/Pend in avatar data")?
                .as_array()
                .ok_or("H_rotation_R/Normal/Pend is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("H_rotation_R values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            result.insert("T_R".to_string(), t_r);
            result.insert("H_R".to_string(), h_r);
            result.insert("HP_R".to_string(), hp_r);
            result.insert("H_rotation_R".to_string(), h_rotation_r);
        } else {
            // 这里的后缀high和low分别表示高音和低音，它刚刚好和stringIndex顺序相反
            let tr_high = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "p3"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/p3 in avatar data")?
                .as_array()
                .ok_or("p3 is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("p3 values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let tr_low = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "p0"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/p0 in avatar data")?
                .as_array()
                .ok_or("p0 is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("p0 values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let h_r_high = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "Normal_P3_H_R"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/Normal_P3_H_R in avatar data")?
                .as_array()
                .ok_or("Normal_P3_H_R is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("Normal_P3_H_R values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let h_r_low = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "Normal_P0_H_R"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/Normal_P0_H_R in avatar data")?
                .as_array()
                .ok_or("Normal_P0_H_R is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("Normal_P0_H_R values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let hp_r_high = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "Normal_P3_HP_R"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/Normal_P3_HP_R in avatar data")?
                .as_array()
                .ok_or("Normal_P3_HP_R is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("Normal_P3_HP_R values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let hp_r_low = self
                .get_avatar_nested_field(&["RIGHT_HAND_POSITIONS", "Normal_P0_HP_R"])
                .ok_or("Missing RIGHT_HAND_POSITIONS/Normal_P0_HP_R in avatar data")?
                .as_array()
                .ok_or("Normal_P0_HP_R is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("Normal_P0_HP_R values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let h_rotation_r_high = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_R", "Normal", "P3"])
                .ok_or("Missing ROTATIONS/H_rotation_R/Normal/P3 in avatar data")?
                .as_array()
                .ok_or("H_rotation_R/Normal/P3 is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("H_rotation_R high values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let h_rotation_r_low = self
                .get_avatar_nested_field(&["ROTATIONS", "H_rotation_R", "Normal", "P0"])
                .ok_or("Missing ROTATIONS/H_rotation_R/Normal/P0 in avatar data")?
                .as_array()
                .ok_or("H_rotation_R/Normal/P0 is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("H_rotation_R low values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            // 计算tr_diff
            let tr_diff: f64 = tr_high
                .iter()
                .zip(tr_low.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();

            let finger_move_distance_while_play = tr_diff / 20.0;

            // 因为相反，所以这里的权重应该是用1减一下
            let thumb_weight = 1.0 - (string_index as f64 / (self.max_string_index as f64 + 1.0));

            let t_r: Vec<f64> = tr_high
                .iter()
                .zip(tr_low.iter())
                .map(|(high, low)| high * thumb_weight + low * (1.0 - thumb_weight))
                .collect();

            let h_r: Vec<f64> = h_r_high
                .iter()
                .zip(h_r_low.iter())
                .map(|(high, low)| high * thumb_weight + low * (1.0 - thumb_weight))
                .collect();

            let hp_r: Vec<f64> = hp_r_high
                .iter()
                .zip(hp_r_low.iter())
                .map(|(high, low)| high * thumb_weight + low * (1.0 - thumb_weight))
                .collect();

            // H_rotation_R = slerp(h_rotation_r_high, h_rotation_r_low, thumb_weight)
            let h_rotation_r_low = Quaternion::from_vector64(h_rotation_r_low);
            let h_rotation_r_high = Quaternion::from_vector64(h_rotation_r_high);
            let h_rotation_r = slerp(&h_rotation_r_high, &h_rotation_r_low, thumb_weight);

            // 如果当前pick位置在当前弦的位置之下，那么就是在低位置，否则就是在高位置
            let multiplier = if should_stay_at_lower_position {
                1.0
            } else {
                -1.0
            };

            // T_R += np.array(move) * fingerMoveDistanceWhilePlay * multiplier
            let move_vector = self
                .get_avatar_nested_field(&["RIGHT_HAND_LINES", "T_line", "vector"])
                .ok_or("Missing RIGHT_HAND_LINES/T_line/vector in avatar data")?
                .as_array()
                .ok_or("T_line/vector is not an array")?
                .iter()
                .map(|v| v.as_f64().ok_or("T_line/vector values are not numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let t_r: Vec<f64> = t_r
                .iter()
                .zip(move_vector.iter())
                .map(|(t, m)| t + m * finger_move_distance_while_play * multiplier)
                .collect();

            let h_rotation_r = h_rotation_r.to_vector64();

            result.insert("T_R".to_string(), t_r);
            result.insert("H_R".to_string(), h_r);
            result.insert("HP_R".to_string(), hp_r);
            result.insert("H_rotation_R".to_string(), h_rotation_r);
        }

        Ok(result)
    }
}
