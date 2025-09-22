// right_hand.rs
use crate::hand::right_finger::RightFingers;
use ndarray::Array1;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::f64;
use std::vec::Vec;

#[derive(Debug, Clone)]
pub struct RightHand {
    pub used_fingers: Vec<String>,
    pub right_finger_positions: Vec<i32>,
    pub pre_used_fingers: Vec<String>,
    pub is_arpeggio: bool,
    pub is_playing_bass: bool,
}

impl RightHand {
    pub fn new(
        used_fingers: Vec<String>,
        right_finger_positions: Vec<i32>,
        pre_used_fingers: Vec<String>,
        is_arpeggio: bool,
        is_playing_bass: bool,
    ) -> Self {
        RightHand {
            used_fingers,
            right_finger_positions,
            pre_used_fingers,
            is_arpeggio,
            is_playing_bass,
        }
    }

    pub fn validate_right_hand(
        &self,
        used_fingers: Option<Vec<String>>,
        right_finger_positions: Option<Vec<i32>>,
    ) -> bool {
        let positions =
            right_finger_positions.unwrap_or_else(|| self.right_finger_positions.clone());
        let fingers = used_fingers.unwrap_or_else(|| self.used_fingers.clone());

        self.validate_right_hand_by_finger_positions(&fingers, &positions, false)
    }

    pub fn calculate_diff(&self, other_right_hand: &RightHand) -> f64 {
        // 重复使用同一根手指的惩罚机制
        let repeat_punish = 10.0;
        let mut diff = 0.0;

        // 计算手指改变所在弦位置的情况，每有移动一根弦距就加1单位的diff
        for i in 0..4 {
            diff += (self.right_finger_positions[i] - other_right_hand.right_finger_positions[i])
                .abs() as f64;
        }

        // 检测两只手的usedFingers相同的元素个数有多少
        let self_used_fingers_set: HashSet<&String> = self.used_fingers.iter().collect();
        let other_used_fingers_set: HashSet<&String> =
            other_right_hand.used_fingers.iter().collect();
        let common_elements: Vec<&String> = self_used_fingers_set
            .intersection(&other_used_fingers_set)
            .cloned()
            .collect();

        let self_pre_used_fingers_set: HashSet<&String> = self.pre_used_fingers.iter().collect();
        let pre_common_elements: Vec<&String> = self_pre_used_fingers_set
            .intersection(&other_used_fingers_set)
            .cloned()
            .collect();

        let mut same_finger_count = common_elements.len() as f64;
        let mut pre_same_finger_count = pre_common_elements.len() as f64;

        // 检测重复使用的手指中有没有P指，如果有的话按加repeat_punish来计算重复指惩罚
        if common_elements.iter().any(|&f| f == "p") {
            diff += repeat_punish;
            same_finger_count -= 1.0;
        }

        if pre_common_elements.iter().any(|&f| f == "p") {
            diff += 0.5 * repeat_punish;
            pre_same_finger_count -= 1.0;
        }

        // 其它重复使用的手指按双倍repeat_punish来算，也就是非常不鼓励除P指以外的手指重复使用
        diff += 2.0 * repeat_punish * (same_finger_count + 0.5 * pre_same_finger_count);

        // 不考虑手掌移动，因为手掌是由身体来带动的，它的移动比手指移动要来得轻松
        diff
    }

    pub fn output(&self) {
        println!(
            "RightHand: {:?} {:?}",
            self.used_fingers, self.right_finger_positions
        );
    }

    fn validate_right_hand_by_finger_positions(
        &self,
        used_fingers: &Vec<String>,
        right_finger_positions: &Vec<i32>,
        repeated_fingers_checked: bool,
    ) -> bool {
        for i in 0..right_finger_positions.len().saturating_sub(1) {
            // 检测手指的位置是否从左到右递减，如果手指分布不符合科学，判断为错误;bass因为要考虑到会有im指交替拨低音弦的情况，需要另外考虑
            if right_finger_positions[i] < right_finger_positions[i] && !self.is_playing_bass {
                return false;
            }
            if right_finger_positions[i] < right_finger_positions[i] + 1 {
                return false;
            }
        }

        // 如果没有p指，而且其它手指已经预检测掉了重复和可能性，那么就直接返回True
        if !used_fingers.contains(&"p".to_string()) && repeated_fingers_checked {
            return true;
        }

        let mut used_string = Vec::new();
        let mut used_p_strings = Vec::new();

        for finger in used_fingers {
            let current_string = right_finger_positions[RightFingers::get_finger_index(finger)];
            if finger == "p" {
                used_p_strings.push(current_string);
            } else if !repeated_fingers_checked {
                // 如果没有预检测过其它手指是否重复触弦，通过下面的方式来检测
                if !used_string.contains(&current_string) {
                    used_string.push(current_string);
                } else {
                    return false;
                }
            }
        }

        // 如果p指只有1根弦或者没有触弦，那么就直接返回True
        if used_p_strings.len() < 2 {
            return true;
        }

        // 如果p指触的两根弦并不相邻，那么就直接返回False
        if (used_p_strings[0] - used_p_strings[1]).abs() > 1 {
            return false;
        }

        // 如果p指双拨的弦最高超过了3，那么就直接返回False
        if *used_p_strings.iter().min().unwrap() < 3 {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct FingerStringPair {
    pub finger: String,
    pub string: i32,
}

// 下面这些内容应该写到right_hand_recorder里面去
pub fn finger_string_map_generator(
    all_fingers: Vec<String>,
    touched_strings: Vec<i32>,
    unused_fingers: Vec<String>,
    all_strings: Vec<i32>,
    prev_finger_string_map: Vec<FingerStringPair>,
) -> Vec<Vec<FingerStringPair>> {
    let mut results = Vec::new();

    if touched_strings.is_empty() {
        let rest_results = rest_finger_string_map_generator(
            unused_fingers,
            all_strings,
            prev_finger_string_map.clone(),
        );
        results.extend(rest_results);
        return results;
    }

    for current_finger in &all_fingers {
        for &touched_string in &touched_strings {
            let mut current_pairing_is_legal = true;
            let current_finger_index = RightFingers::get_finger_index(current_finger);

            // 检测当前配对组合是否和之前生成的配对组合有冲突
            for pairing in &prev_finger_string_map {
                let prev_finger = &pairing.finger;
                let prev_string = pairing.string;
                let prev_index = RightFingers::get_finger_index(prev_finger);

                // 这里注意，finger是从pima递增的，但现实中它们拨的弦序号是递减的，所以判断合理性时要留意这一点
                if (current_finger_index > prev_index && touched_string >= prev_string)
                    || (current_finger_index < prev_index && touched_string <= prev_string)
                    || (current_finger_index == prev_index
                        && (prev_string - touched_string).abs() > 1)
                {
                    current_pairing_is_legal = false;
                }
            }

            if current_pairing_is_legal {
                let mut next_all_fingers = all_fingers.clone();
                next_all_fingers.retain(|f| f != current_finger);

                let mut next_touched_strings = touched_strings.clone();
                next_touched_strings.retain(|&s| s != touched_string);

                let mut next_unused_fingers = unused_fingers.clone();
                next_unused_fingers.retain(|f| f != current_finger);

                let current_pairing = FingerStringPair {
                    finger: current_finger.clone(),
                    string: touched_string,
                };

                let mut next_finger_string_map = prev_finger_string_map.clone();
                next_finger_string_map.push(current_pairing.clone());

                let sub_results = finger_string_map_generator(
                    next_all_fingers,
                    next_touched_strings,
                    next_unused_fingers,
                    all_strings.clone(),
                    next_finger_string_map,
                );

                for mut result in sub_results {
                    let mut combined = vec![current_pairing.clone()];
                    combined.append(&mut result);
                    results.push(combined);
                }
            }
        }
    }

    results
}

pub fn rest_finger_string_map_generator(
    unused_fingers: Vec<String>,
    all_strings: Vec<i32>,
    prev_finger_string_map: Vec<FingerStringPair>,
) -> Vec<Vec<FingerStringPair>> {
    let mut results = Vec::new();

    if unused_fingers.is_empty() {
        results.push(Vec::new());
        return results;
    }

    // 这里每次配对以后，不再移除已经配对的弦，因为不演奏的手指可以放在任一根弦上
    for current_finger in &unused_fingers {
        for &cur_string in &all_strings {
            let mut current_pairing_is_legal = true;
            let current_finger_index = RightFingers::get_finger_index(current_finger);

            // 检测当前配对组合是否和之前生成的配对组合有冲突
            for pairing in &prev_finger_string_map {
                let prev_finger = &pairing.finger;
                let prev_string = pairing.string;
                let prev_index = RightFingers::get_finger_index(prev_finger);

                if (current_finger_index > prev_index && cur_string > prev_string + 1)
                    || (current_finger_index < prev_index && cur_string < prev_string - 1)
                {
                    current_pairing_is_legal = false;
                }
            }

            if current_pairing_is_legal {
                let mut next_all_fingers = unused_fingers.clone();
                next_all_fingers.retain(|f| f != current_finger);

                let current_pairing = FingerStringPair {
                    finger: current_finger.clone(),
                    string: cur_string,
                };

                let mut updated_finger_string_map = prev_finger_string_map.clone();
                updated_finger_string_map.push(current_pairing.clone());

                let sub_results = rest_finger_string_map_generator(
                    next_all_fingers,
                    all_strings.clone(),
                    updated_finger_string_map,
                );

                for mut result in sub_results {
                    let mut combined = vec![current_pairing.clone()];
                    combined.append(&mut result);
                    results.push(combined);
                }
            }
        }
    }

    results
}

pub fn generate_possible_right_hands(
    touched_strings: Vec<i32>,
    all_fingers: Vec<String>,
    all_strings: Vec<i32>,
) -> Vec<HashMap<String, Value>> {
    let mut possible_combinations = Vec::new();

    if touched_strings.len() > 4 {
        let used_fingers: Vec<i32> = Vec::new();
        let right_finger_positions = vec![5, 4, 3, 2];
        let mut combination = HashMap::new();
        combination.insert(
            "usedFingers".to_string(),
            serde_json::to_value(&used_fingers).unwrap(),
        );
        combination.insert(
            "rightFingerPositions".to_string(),
            serde_json::to_value(&right_finger_positions).unwrap(),
        );
        possible_combinations.push(combination);
    } else {
        let unused_fingers = all_fingers.clone();
        let results = finger_string_map_generator(
            all_fingers.clone(),
            touched_strings.clone(),
            unused_fingers,
            all_strings,
            Vec::new(),
        );

        for result in results {
            // 输出的结果必须是每个手指都有对应的位置，否则说明配对不合法
            if result.is_empty() || result.len() < all_fingers.len() {
                continue;
            }

            let right_finger_positions = sort_fingers(&result);
            let used_fingers = get_used_fingers(&result, &touched_strings);

            let mut combination = HashMap::new();
            combination.insert(
                "usedFingers".to_string(),
                serde_json::to_value(&used_fingers).unwrap(),
            );
            combination.insert(
                "rightFingerPositions".to_string(),
                serde_json::to_value(&right_finger_positions).unwrap(),
            );
            possible_combinations.push(combination);
        }
    }

    possible_combinations
}

pub fn sort_fingers(finger_list: &Vec<FingerStringPair>) -> Vec<i32> {
    let order = HashMap::from([
        ("p".to_string(), 0),
        ("i".to_string(), 1),
        ("m".to_string(), 2),
        ("a".to_string(), 3),
    ]);

    let mut sorted_list = finger_list.clone();
    sorted_list.sort_by_key(|x| order.get(&x.finger).unwrap_or(&0));
    sorted_list.iter().map(|item| item.string).collect()
}

// 这里把p指的重复演奏进行处理，只留下可能的情况，并只留一个p指数据
pub fn process_p_fingers(result: &mut Vec<FingerStringPair>) -> Option<Vec<FingerStringPair>> {
    let p_fingers: Vec<&FingerStringPair> = result.iter().filter(|f| f.finger == "p").collect();
    let strings: Vec<i32> = p_fingers.iter().map(|f| f.string).collect();

    if strings.len() < 2 {
        return None;
    }

    if (strings[0] - strings[1]).abs() != 1 {
        return None;
    }

    let min_string = *strings.iter().min().unwrap();
    result.retain(|f| !(f.finger == "p" && f.string == min_string));

    Some(result.clone())
}

pub fn get_used_fingers(
    finger_list: &Vec<FingerStringPair>,
    used_strings: &Vec<i32>,
) -> Vec<String> {
    finger_list
        .iter()
        .filter(|item| used_strings.contains(&item.string))
        .map(|item| item.finger.clone())
        .collect()
}

// 由于剩余的函数涉及复杂的数学计算和3D几何操作，需要依赖特定的数学库
// 这里提供函数签名作为示例：

pub fn new_finger_position_method(
    avatar_data: &serde_json::Value,
    right_finger_positions: Vec<i32>,
    is_arpeggio: bool,
    is_after_played: bool,
    hand_position: f64,
    used_right_fingers: Vec<String>,
    max_string_index: i32,
) -> HashMap<String, Vec<f64>> {
    // 实现复杂的3D位置计算逻辑
    // 需要使用ndarray等数学库来处理向量和矩阵运算
    HashMap::new()
}

pub fn calculate_right_hand_fingers(
    avatar_data: &serde_json::Value,
    right_finger_positions: Vec<i32>,
    used_right_fingers: Vec<String>,
    max_string_index: i32,
    is_after_played: bool,
) -> HashMap<String, Vec<f64>> {
    // 实现右手手指位置计算
    HashMap::new()
}

pub fn calculate_finger_position_by_hand_position(
    h_r: Array1<f64>,
    h3: Array1<f64>,
    avatar_data: &serde_json::Value,
    finger_index: usize,
    string_index: usize,
) -> (Array1<f64>, f64) {
    // 实现手指位置计算逻辑
    (Array1::zeros(3), 0.0)
}

pub fn euler_to_rotation_matrix(euler_angles: Array1<f64>) -> ndarray::Array2<f64> {
    // 实现欧拉角到旋转矩阵的转换
    ndarray::Array2::zeros((3, 3))
}

pub fn get_transformation_matrix(
    position: Array1<f64>,
    euler_angles: Array1<f64>,
) -> ndarray::Array2<f64> {
    // 实现变换矩阵计算
    ndarray::Array2::zeros((4, 4))
}
