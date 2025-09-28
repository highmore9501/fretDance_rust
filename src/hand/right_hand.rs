// right_hand.rs
use crate::hand::right_finger::RightFingers;
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
        let positions = match right_finger_positions {
            Some(pos) => pos,
            None => self.right_finger_positions.clone(),
        };
        let fingers = match used_fingers {
            Some(f) => f,
            None => self.used_fingers.clone(),
        };

        self.validate_right_hand_by_finger_positions(&fingers, &positions)
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
    ) -> bool {
        for i in 0..right_finger_positions.len().saturating_sub(1) {
            // 检测手指的位置是否从左到右递减，如果手指分布不符合科学，判断为错误;

            if right_finger_positions[i] < right_finger_positions[i + 1] {
                println!("Invalid right hand: finger positions are not in descending order.");
                return false;
            }
        }

        let mut used_string = std::collections::HashSet::new();
        let mut used_p_strings = Vec::new();

        for finger in used_fingers {
            let current_string = right_finger_positions[RightFingers::get_finger_index(finger)];
            if finger == "p" {
                used_p_strings.push(current_string);
            } else {
                // 使用 HashSet 的 insert 方法，如果值已存在会返回 false
                if !used_string.insert(current_string) {
                    println!("Invalid right hand: duplicate finger.");
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

// 新的高效算法实现
pub fn generate_finger_placements(
    touched_strings: Vec<i32>,
    all_fingers: Vec<String>,
    all_strings: Vec<i32>,
) -> Vec<Vec<FingerStringPair>> {
    let mut results = Vec::new();

    // 对触弦按从高到低排序（数值从小到大）
    let mut sorted_touched_strings = touched_strings.clone();
    sorted_touched_strings.sort_by(|a, b| b.cmp(a));

    // 生成所有可能的手指分配方案
    let finger_assignments = generate_finger_assignments(&all_fingers, &sorted_touched_strings);

    // 为每个手指分配方案生成合理的弦分配
    for assignment in finger_assignments {
        let placements =
            generate_string_placements(&assignment, &sorted_touched_strings, &all_strings);
        results.extend(placements);
    }

    results
}

fn generate_finger_assignments(
    all_fingers: &Vec<String>,
    touched_strings: &Vec<i32>,
) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    let n_strings = touched_strings.len();

    // 生成所有可能的手指选择组合
    let finger_indices: Vec<usize> = (0..all_fingers.len()).collect();
    let combinations = generate_combinations(&finger_indices, n_strings);

    for combination in combinations {
        let mut assignment = Vec::new();
        for &idx in &combination {
            assignment.push(all_fingers[idx].clone());
        }

        results.push(assignment);
    }

    results
}

fn generate_combinations(indices: &Vec<usize>, k: usize) -> Vec<Vec<usize>> {
    fn generate_combinations_helper(
        indices: &[usize],
        k: usize,
        start: usize,
        current: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        // 基础情况
        if k == 0 {
            results.push(current.clone());
            return;
        }

        // 如果剩余元素不足，直接返回
        if start + k > indices.len() {
            return;
        }

        // 包含当前元素的情况
        current.push(indices[start]);
        generate_combinations_helper(indices, k - 1, start + 1, current, results);
        current.pop();

        // 不包含当前元素的情况
        generate_combinations_helper(indices, k, start + 1, current, results);
    }

    let mut results = Vec::new();
    let mut current = Vec::new();

    if indices.len() >= k {
        generate_combinations_helper(indices, k, 0, &mut current, &mut results);
    }

    results
}

fn generate_string_placements(
    fingers: &Vec<String>,
    touched_strings: &Vec<i32>,
    all_strings: &Vec<i32>,
) -> Vec<Vec<FingerStringPair>> {
    let mut results = Vec::new();

    // 为每个手指分配对应的弦
    let mut placement = Vec::new();
    for (i, finger) in fingers.iter().enumerate() {
        if i < touched_strings.len() {
            placement.push(FingerStringPair {
                finger: finger.clone(),
                string: touched_strings[i],
            });
        }
    }

    // 确定未使用的手指
    let standard_fingers = ["p", "i", "m", "a"];
    let used_fingers: std::collections::HashSet<&str> =
        fingers.iter().map(|s| s.as_str()).collect();
    let unused_fingers: Vec<String> = standard_fingers
        .iter()
        .filter(|&&finger| !used_fingers.contains(finger))
        .map(|&s| s.to_string())
        .collect();

    // 为未使用的手指选择合适的弦（避免使用已触弦的弦）
    let available_strings: Vec<i32> = all_strings
        .iter()
        .filter(|&&s| !touched_strings.contains(&s))
        .cloned()
        .collect();

    // 使用rest_finger_string_map_generator为未使用的手指生成所有可能的位置
    if !unused_fingers.is_empty() {
        // 构造只包含可用弦的all_strings副本

        let sub_results =
            rest_finger_string_map_generator(unused_fingers, available_strings, placement.clone());

        for mut result in sub_results {
            let mut combined = placement.clone();
            combined.append(&mut result);

            if validate_finger_placement(&combined) {
                results.push(combined);
            }
        }
    } else {
        // 没有未使用的手指
        if validate_finger_placement(&placement) {
            results.push(placement);
        }
    }

    results
}
fn validate_finger_placement(placement: &Vec<FingerStringPair>) -> bool {
    // 检查手指顺序是否合理（从p到a，弦位置应递减）
    let finger_order = HashMap::from([
        ("p".to_string(), 0),
        ("i".to_string(), 1),
        ("m".to_string(), 2),
        ("a".to_string(), 3),
    ]);

    let mut sorted_placement = placement.clone();
    sorted_placement.sort_by_key(|p| finger_order.get(&p.finger).unwrap_or(&0));

    // 检查不同手指间的顺序
    let mut prev_max_string = std::i32::MAX;
    let mut current_finger_type = String::new();

    for item in &sorted_placement {
        if item.finger != current_finger_type {
            // 新的手指类型
            current_finger_type = item.finger.clone();
            if item.string > prev_max_string {
                return false;
            }
            prev_max_string = item.string;
        } else {
            // 相同手指类型，更新最大弦号
            prev_max_string = prev_max_string.max(item.string);
        }
    }

    true
}
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

    // 使用引用避免重复克隆
    for (finger_idx, current_finger) in all_fingers.iter().enumerate() {
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
                    break; // 提前退出循环
                }
            }

            if current_pairing_is_legal {
                // 创建新的集合，避免多次克隆
                let mut next_all_fingers = all_fingers.clone();
                next_all_fingers.remove(finger_idx);

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

                // 使用容量预分配减少内存重新分配
                results.reserve(sub_results.len());
                for mut result in sub_results {
                    let mut combined = Vec::with_capacity(result.len() + 1);
                    combined.push(current_pairing.clone());
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

    // 这里每次配对以后，不再移除已经配对的弦，因为不演奏的手指可以放在任一根弦上
    // 但需要遵循手指自然分布规律
    for (finger_idx, current_finger) in unused_fingers.iter().enumerate() {
        for &cur_string in &all_strings {
            let current_finger_index = RightFingers::get_finger_index(current_finger);

            // 检测当前配对组合是否和之前生成的配对组合有冲突
            let mut current_pairing_is_legal = true;
            for pairing in &prev_finger_string_map {
                let prev_finger = &pairing.finger;
                let prev_string = pairing.string;
                let prev_index = RightFingers::get_finger_index(prev_finger);

                // 遵循手指自然分布规律：
                // 1. 手指索引大的（如a指）应该在弦数小的位置（高音弦）
                // 2. 手指索引小的（如p指）应该在弦数大的位置（低音弦）
                // 3. 相邻手指应该在相邻或相近的弦上

                if (current_finger_index > prev_index && cur_string > prev_string + 1)
                    || (current_finger_index < prev_index && cur_string < prev_string - 1)
                {
                    current_pairing_is_legal = false;
                }

                if !current_pairing_is_legal {
                    break; // 提前退出循环
                }
            }

            if current_pairing_is_legal {
                // 避免多次克隆
                let mut next_all_fingers = unused_fingers.clone();
                next_all_fingers.remove(finger_idx);

                let current_pairing = FingerStringPair {
                    finger: current_finger.clone(),
                    string: cur_string,
                };

                let mut updated_finger_string_map = prev_finger_string_map.clone();
                updated_finger_string_map.push(current_pairing.clone());

                // 递归调用前检查
                let sub_results = if next_all_fingers.is_empty() {
                    // 如果没有更多手指需要处理，直接返回包含当前配对的结果
                    vec![vec![]]
                } else {
                    rest_finger_string_map_generator(
                        next_all_fingers,
                        all_strings.clone(),
                        updated_finger_string_map,
                    )
                };

                for mut result in sub_results {
                    let mut combined = Vec::new();
                    combined.push(current_pairing.clone());
                    combined.append(&mut result);
                    results.push(combined);
                }
            }
        }
    }

    results
}
// 定义一个结构体来替代 HashMap<String, Value>
#[derive(Debug, Clone)]
pub struct RightHandCombination {
    pub used_fingers: Vec<String>,
    pub right_finger_positions: Vec<i32>,
}

pub fn generate_possible_right_hands(
    touched_strings: Vec<i32>,
    all_fingers: Vec<String>,
    all_strings: Vec<i32>,
) -> Vec<RightHandCombination> {
    let mut possible_combinations = Vec::new();

    // 简化处理：当触弦数超过4根时，使用琶音方式处理
    if touched_strings.len() > 4 {
        let used_fingers: Vec<String> = Vec::new();
        let right_finger_positions = vec![5, 2, 1, 0];

        possible_combinations.push(RightHandCombination {
            used_fingers,
            right_finger_positions,
        });
        return possible_combinations;
    }

    let finger_placements = generate_finger_placements(
        touched_strings.clone(),
        all_fingers.clone(),
        all_strings.clone(),
    );

    for placement in finger_placements {
        let right_finger_positions = sort_fingers(&placement);
        let used_fingers = get_used_fingers(&placement, &touched_strings);

        possible_combinations.push(RightHandCombination {
            used_fingers,
            right_finger_positions,
        });
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
