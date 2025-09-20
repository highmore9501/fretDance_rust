// LeftHand.rs
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

// 假设已存在的模块和结构体
use crate::guitar::guitar_chord::NotePosition;
use crate::guitar::guitar_instance::Guitar;
use crate::hand::left_finger::{FingerPosition, LeftFinger, PressState};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HandPosition {
    pub positions: Vec<FingerPosition>,
}

impl HandPosition {
    pub fn new(positions: Vec<FingerPosition>) -> Self {
        HandPosition { positions }
    }

    pub fn fingerprint(&self) -> String {
        let mut sorted_positions: Vec<&FingerPosition> = self.positions.iter().collect();
        sorted_positions.sort_by_key(|p| p.string_index);
        sorted_positions
            .iter()
            .map(|p| format!("{}:{}:{}", p.string_index, p.fret, p.finger))
            .collect::<Vec<_>>()
            .join("|")
    }
}

#[derive(Debug, Clone)]
pub struct LeftHand {
    pub fingers: Vec<LeftFinger>,
    max_finger_distance: f64,
    finger_distance_to_fretboard: f64,
    pub hand_position: i32,
    pub use_barre: bool,
}

impl LeftHand {
    /// 创建新的左手实例
    ///
    /// # 参数
    /// * `left_fingers` - 手指列表
    /// * `use_barre` - 是否使用横按
    /// * `max_finger_distance` - 两只相邻手指所能打开的最大距离，单位是cm
    pub fn new(left_fingers: Vec<LeftFinger>, use_barre: bool, max_finger_distance: f64) -> Self {
        let mut hand = LeftHand {
            fingers: left_fingers,
            max_finger_distance,
            finger_distance_to_fretboard: 0.025,
            hand_position: 1,
            use_barre,
        };

        hand.hand_position = hand.calculate_hand_position();
        hand.rearrange_fingers();

        hand
    }

    /// 获取最大手指距离
    pub fn get_max_finger_distance(&self) -> f64 {
        self.max_finger_distance
    }

    /// 将所有手指设置为抬起
    pub fn all_open(&mut self) {
        for finger in &mut self.fingers {
            finger.press = PressState::Open;
        }
    }

    /// 将所有fret为0的手指设置为抬起，并重新计算它所在的fret
    pub fn rearrange_fingers(&mut self) {
        for finger in &mut self.fingers {
            if finger.finger_index != 0 && finger.fret == 0 {
                finger.press = PressState::Open;
                finger.fret = self.hand_position + finger.finger_index - 1;
            }
        }
    }

    /// 输出当前手型
    pub fn output(&self, show_open_finger: bool) {
        for finger in &self.fingers {
            finger.output();
        }

        // 计算所有手指的最小fret值
        let mut min_fret = 24;
        let mut min_fret_is_setted = false;

        for finger in &self.fingers {
            if finger.fret < min_fret && finger.fret != 0 {
                min_fret = finger.fret;
                min_fret_is_setted = true;
            }
        }

        let base_fret = if min_fret < 4 {
            0
        } else if min_fret < 6 {
            3
        } else if min_fret < 8 {
            5
        } else if min_fret < 10 {
            7
        } else if min_fret < 13 {
            9
        } else if min_fret_is_setted {
            12
        } else {
            0
        };

        println!("{}", base_fret);

        for string_index in 0..6 {
            let mut txt = String::new();
            let mut all_note_in_string_index = Vec::new();

            for finger in &self.fingers {
                if finger.string_index == string_index {
                    all_note_in_string_index.push(finger);
                }
            }

            // 如果这根弦上没有手指
            if all_note_in_string_index.is_empty() {
                txt.push_str(&format!("|{}{}", " --".repeat(8), string_index));
            } else {
                let mut pressed_finger = Vec::new();
                let mut open_finger = Vec::new();
                let mut has_open_string_note = false;

                for finger in &all_note_in_string_index {
                    // 也就是index为-1的手指存在，说明本弦是空弦音
                    if finger.finger_index == -1 {
                        has_open_string_note = true;
                    }

                    // 根据在这根弦上的手指的按弦状态，将手指分为按下和抬起两类
                    let press = finger.press;
                    if press != PressState::Open {
                        pressed_finger.push(*finger);
                    } else {
                        open_finger.push(*finger);
                    }
                }

                // 如果是空弦音，就以0起头
                if has_open_string_note {
                    txt.push('0');
                } else {
                    txt.push('|');
                }

                for character_index in 1..9 {
                    let mut character_index_in_pressed_finger = false;
                    let mut character_index_in_open_finger = false;
                    let mut finger_index = 0;
                    let mut open_fingers_in_fret = Vec::new();

                    for finger in &pressed_finger {
                        if finger.fret - base_fret == character_index {
                            character_index_in_pressed_finger = true;
                            finger_index = finger.finger_index;
                        }
                    }

                    for finger in &open_finger {
                        if finger.fret - base_fret == character_index {
                            character_index_in_open_finger = true;
                            open_fingers_in_fret.push(finger.finger_index);
                        }
                    }

                    if character_index_in_pressed_finger {
                        txt.push_str(&format!(" -{}", finger_index));
                    } else if character_index_in_open_finger && show_open_finger {
                        match open_fingers_in_fret.len() {
                            1 => {
                                let strikethrough_text =
                                    print_strikethrough(&open_fingers_in_fret[0].to_string());
                                txt.push_str(&format!(" 0{}", strikethrough_text));
                            }
                            2 => {
                                let strikethrough_text = print_strikethrough(&format!(
                                    "{}{}",
                                    open_fingers_in_fret[0], open_fingers_in_fret[1]
                                ));
                                txt.push_str(&format!(" {}", strikethrough_text));
                            }
                            3 => {
                                let strikethrough_text = print_strikethrough(&format!(
                                    "{}{}{}",
                                    open_fingers_in_fret[0],
                                    open_fingers_in_fret[1],
                                    open_fingers_in_fret[2]
                                ));
                                txt.push_str(&strikethrough_text);
                            }
                            _ => txt.push_str(" --"),
                        }
                    } else {
                        txt.push_str(" --");
                    }
                }

                txt.push_str(&string_index.to_string());
            }

            println!("{}", txt);
        }

        println!("把位：{}", self.hand_position);
        println!("-------------------------------");
    }

    /// 验证手型是否合法
    pub fn verify_valid(&self, all_fingers: Option<&Vec<LeftFinger>>) -> bool {
        let fingers = match all_fingers {
            Some(f) => f,
            None => &self.fingers,
        };

        if fingers.is_empty() {
            return false;
        }

        for finger in fingers {
            if finger.finger_index == -1 {
                continue;
            }

            // 最高的把位限制，四指在0弦上最多按到22品，在1弦上最多按到21品，其它弦和其它手指依此类推
            if finger.fret > 22 - (4 - finger.finger_index) - finger.string_index {
                return false;
            }
        }

        let mut sorted_fingers = fingers.clone();
        sorted_fingers.sort_by_key(|f| f.finger_index);

        // 如果在低把位，不能出现小拇指或者无名指延展两个品格的情况
        if self.hand_position < 10
            && (sorted_fingers[sorted_fingers.len() - 1].fret
                - sorted_fingers[sorted_fingers.len() - 2].fret
                > 1
                || sorted_fingers[sorted_fingers.len() - 2].fret
                    - sorted_fingers[sorted_fingers.len() - 3].fret
                    > 1)
        {
            return false;
        }

        let mut bigger_fret_counter = 0;

        for i in 0..sorted_fingers.len() - 1 {
            if (sorted_fingers[i].fret - sorted_fingers[i + 1].fret).abs() > 1 {
                bigger_fret_counter += 1;
            }

            let min_index_finger_is_higher = sorted_fingers[i].finger_index
                < sorted_fingers[i + 1].finger_index
                && sorted_fingers[i].fret > sorted_fingers[i + 1].fret;

            if min_index_finger_is_higher {
                return false;
            }

            let max_index_finger_is_lower = sorted_fingers[i].finger_index
                > sorted_fingers[i + 1].finger_index
                && sorted_fingers[i].fret < sorted_fingers[i + 1].fret;

            if max_index_finger_is_lower {
                return false;
            }

            let both_finger_is_not_zero =
                sorted_fingers[i].fret != 0 && sorted_fingers[i + 1].fret != 0;
            let finger_distance_is_too_large = (sorted_fingers[i].fret
                - sorted_fingers[i + 1].fret)
                .abs()
                > 2 * (sorted_fingers[i].finger_index - sorted_fingers[i + 1].finger_index).abs();

            if finger_distance_is_too_large && both_finger_is_not_zero {
                return false;
            }

            let finger_index_large_than_fret =
                sorted_fingers[i].finger_index > sorted_fingers[i].fret + 1;
            if finger_index_large_than_fret {
                return false;
            }

            let min_finger_is_outer = sorted_fingers[i].fret == sorted_fingers[i + 1].fret
                && sorted_fingers[i].string_index < sorted_fingers[i + 1].string_index;

            if min_finger_is_outer {
                return false;
            }
        }

        if bigger_fret_counter > 1 {
            return false;
        }

        true
    }

    /// 计算手型的位置
    /// 返回手的位置，也就是食指所在的品格
    pub fn calculate_hand_position(&self) -> i32 {
        let mut hand_position = 1;

        for finger in &self.fingers {
            if finger.finger_index == 1 && finger.fret > 1 {
                hand_position = finger.fret;
            }
        }

        hand_position
    }

    /// 生成下一个手型
    pub fn generate_next_hands(
        &self,
        guitar: &Guitar,
        finger_positions: &Vec<FingerPosition>,
    ) -> Option<(Vec<LeftFinger>, f64, bool)> {
        // 初始化空弦数据，按弦数据，横按数据，休息数据
        let mut empty_fingers = Vec::new();
        let mut empty_string_index_set = HashSet::new();

        let mut used_finger_index_set = HashSet::new();
        let mut used_finger_fret_set = HashSet::new();
        let mut used_string_index_set = HashSet::new();

        let mut pressed_fingers = Vec::new();
        let mut pressed_finger_dict = HashMap::new();

        let mut barre_fingers = Vec::new();
        let mut barre_finger_dict = HashMap::new();
        let mut barre_string_index = -1;
        let mut need_barre = false;
        let mut keep_barre = false;
        let use_barre;

        let mut keep_fingers = Vec::new();

        let mut rest_finger_index_set = HashSet::new();
        let mut rest_fingers = Vec::new();

        // 1~4 号手指的触弦数，下标 0 不用
        let mut finger_touch_string_counter = vec![0; 5];

        // 用于计算下一个手型把位的值，一开始它是统计的最低手指所按的品，统计完以后再根据情况来计算
        let mut new_hand_position = 0;

        // 第一次循环，处理空弦音，并且计算各个手指的触弦总数,更新最高和最低按弦索引，更新使用的手指索引
        for finger_position in finger_positions {
            let fret = finger_position.fret;
            let finger_index = finger_position.finger;
            let string_index = finger_position.string_index;

            // 生成空弦音的手指
            if finger_index == -1 {
                let empty_finger = LeftFinger::new(
                    -1,
                    &guitar.guitar_strings[string_index as usize].clone(),
                    0,
                    "Open",
                );
                empty_fingers.push(empty_finger);
                empty_string_index_set.insert(string_index);
            } else {
                // 统计按弦手指的触弦总数
                finger_touch_string_counter[finger_index as usize] += 1;

                // 添加按弦品格数据
                used_finger_fret_set.insert(fret);

                // 如果这个手指同时按了多个弦，那么就使用这些弦里最低一根的弦的索引
                if finger_touch_string_counter[finger_index as usize] > 1 {
                    // 首次发现某个手指同时按了多个弦，需要把它从按弦手指列表中移除，因为会添加到横按列表中
                    if finger_touch_string_counter[finger_index as usize] == 2 {
                        let (old_fret, old_string_index) =
                            pressed_finger_dict.remove(&finger_index).unwrap();
                        barre_finger_dict.insert(finger_index, (old_fret, old_string_index));
                    }

                    let (_, old_string_index) = barre_finger_dict[&finger_index];
                    let string_index = std::cmp::max(string_index, old_string_index);
                    barre_finger_dict.insert(finger_index, (fret, string_index));
                } else {
                    pressed_finger_dict.insert(finger_index, (fret, string_index));
                }

                used_finger_index_set.insert(finger_index);
                used_string_index_set.insert(string_index);
            }
        }

        // 生成按弦手指
        for (pressed_finger_index, (pressed_fret, pressed_string_index)) in &pressed_finger_dict {
            let pressed_finger = LeftFinger::new(
                *pressed_finger_index,
                &guitar.guitar_strings[*pressed_string_index as usize].clone(),
                *pressed_fret,
                "Pressed",
            );

            pressed_fingers.push(pressed_finger);
        }

        // 生成横按手指
        for (barre_finger_index, (barre_fret, barre_string_index_val)) in &barre_finger_dict {
            let touch_count = finger_touch_string_counter[*barre_finger_index as usize];

            let press_state = if touch_count > 1 && *barre_finger_index == 1 {
                need_barre = true;
                // 横按所按的弦，要比其它手指至少低一根弦，否则动画会很难看
                barre_string_index = std::cmp::max(
                    *barre_string_index_val,
                    *used_string_index_set.iter().max().unwrap_or(&0) + 1,
                );
                barre_string_index =
                    std::cmp::min(barre_string_index, guitar.guitar_strings.len() as i32 - 1);
                PressState::Barre
            } else if touch_count == 2 && *barre_finger_index == 4 {
                PressState::PartialBarre2Strings
            } else if touch_count == 3 && *barre_finger_index == 4 {
                PressState::PartialBarre3Strings
            } else {
                PressState::Pressed
            };

            let barre_finger = LeftFinger::new(
                *barre_finger_index,
                &guitar.guitar_strings[barre_string_index as usize].clone(),
                *barre_fret,
                press_state.to_str(),
            );

            barre_fingers.push(barre_finger);
        }

        // 处理未参与演奏的手指，看它们是横按还是否保留或者休息,在最后面生成保留指
        if !used_finger_index_set.is_empty() && !used_finger_fret_set.is_empty() {
            new_hand_position = std::cmp::max(
                1,
                *used_finger_fret_set.iter().min().unwrap()
                    - (*used_finger_index_set.iter().min().unwrap() - 1),
            );
        } else {
            // 如果没有按弦手指，使用当前手的位置或者默认位置
            new_hand_position = self.hand_position;
        }

        for old_finger_index in 1..5 {
            // 跳过已按的指
            if used_finger_index_set.contains(&old_finger_index) {
                continue;
            }

            // 发生了换把没必要处理保留指，把手指移动到休息位置
            if new_hand_position != self.hand_position {
                rest_finger_index_set.insert(old_finger_index);
                continue;
            }

            // 读取手指在前一个手型里的位置信息
            let same_finger = self
                .fingers
                .iter()
                .find(|f| f.finger_index == old_finger_index)
                .unwrap();

            let old_fret = same_finger.fret;
            let old_string_index = same_finger.string_index;

            // 检测旧位置是否在现在横按的情况下可用
            let mut finger_can_keep = true;
            for (barre_finger_index, (barre_fret, barre_string_index_val)) in &barre_finger_dict {
                if old_fret <= *barre_fret || old_string_index > *barre_string_index_val {
                    finger_can_keep = false;
                    break;
                }
            }

            // 如果旧位置与当前横按冲突，结束检测，将旧位置的手指索引添加到休息指索引列中
            if !finger_can_keep {
                rest_finger_index_set.insert(old_finger_index);
                continue;
            }

            // 检测旧位置是否与当前按的位置有重合
            for (pressed_finger_index, (pressed_fret, pressed_string_index)) in &pressed_finger_dict
            {
                if old_fret == *pressed_fret && old_string_index == *pressed_string_index {
                    finger_can_keep = false;
                    break;
                }
            }

            // 如果有重合，结束检测，直接将旧位置的手指索引添加休息指索引列中
            if !finger_can_keep {
                rest_finger_index_set.insert(old_finger_index);
                continue;
            }

            // 能运行到这里的都是保留指，直接用原状态生成新的保留指
            let press_state = same_finger.press;

            // 如果上一个手型有食指横按，判断一下是否需要保留食指横按
            if same_finger.press == PressState::Barre
                && same_finger.finger_index == 1
                && !used_string_index_set.is_empty()
            {
                if same_finger.string_index <= *used_string_index_set.iter().max().unwrap() {
                    keep_barre = true;
                }
            }

            // 小指不使用保留指，因为动画里横按时小指的状态太难处理
            let press_state = if old_finger_index == 4 {
                PressState::Open
            } else {
                press_state
            };

            let old_string = guitar.guitar_strings[old_string_index as usize].clone();
            let keep_finger = LeftFinger::new(
                old_finger_index,
                &old_string,
                old_fret,
                press_state.to_str(),
            );

            keep_fingers.push(keep_finger);
            used_string_index_set.insert(old_string_index);
        }

        // 现在来处理所有空闲指，判断它们应该放在哪里休息
        // 这一段还没考虑如果空闲指与其它手指冲突的处理
        for rest_finger_index in rest_finger_index_set.iter() {
            // 这是默认休息的弦索引，其实就是所有已经按弦弦索引的中位点
            let default_rest_string_index = if !used_string_index_set.is_empty() {
                if new_hand_position < 17 {
                    ((*used_string_index_set.iter().min().unwrap()
                        + *used_string_index_set.iter().max().unwrap())
                        / 2)
                } else {
                    0
                }
            } else {
                // 这里是为了区别吉他和bass的休息弦，一个是第三弦，一个是第二弦
                if guitar.guitar_strings.len() > 5 {
                    2
                } else {
                    1
                }
            };

            // 这里是默认放置休息的品位
            let default_rest_fret = new_hand_position + rest_finger_index - 1;
            let rest_string = guitar.guitar_strings[default_rest_string_index as usize].clone();

            let rest_finger =
                LeftFinger::new(*rest_finger_index, &rest_string, default_rest_fret, "Open");

            rest_fingers.push(rest_finger);
        }

        let all_fingers = [
            pressed_fingers,
            barre_fingers,
            empty_fingers,
            rest_fingers,
            keep_fingers,
        ]
        .concat();

        if !self.verify_valid(Some(&all_fingers)) {
            return None;
        }

        let diff = self.calculate_diff(&all_fingers, new_hand_position, guitar);

        use_barre = need_barre || keep_barre;
        Some((all_fingers, diff, use_barre))
    }

    /// 计算手型变化的差异（熵）
    pub fn calculate_diff(
        &self,
        all_fingers: &Vec<LeftFinger>,
        new_hand_position: i32,
        guitar: &Guitar,
    ) -> f64 {
        let mut entropy = 0.0;
        let hand_position_diff = (self.hand_position - new_hand_position).abs();

        // 如果要换把，首先要抬指
        if hand_position_diff > 0 {
            for finger in &self.fingers {
                if finger.press != PressState::Open {
                    entropy += self.finger_distance_to_fretboard;
                }
            }
        }

        // 计算每一个手指的位移
        for index in 1..5 {
            let old_finger = self.fingers.iter().find(|f| f.finger_index == index);
            let new_finger = all_fingers.iter().find(|f| f.finger_index == index);

            let distance = match (old_finger, new_finger) {
                (Some(old), Some(new)) => {
                    let dist = old.distance_to(guitar, new);
                    entropy += dist;
                    Some(dist)
                }
                _ => None,
            };

            // 计算按下手指的熵
            if let (Some(dist), Some(new)) = (distance, new_finger) {
                if dist > 0.0 && new.press != PressState::Open {
                    entropy += self.finger_distance_to_fretboard;
                }
            }
        }

        entropy
    }
}

/// 打印删除线文本
fn print_strikethrough(text: &str) -> String {
    format!("\x1b[9m{}\x1b[0m", text)
}

pub fn convert_chord_to_finger_positions(chord: &Vec<NotePosition>) -> Vec<HandPosition> {
    let mut result = Vec::new();
    let finger_list = vec![1, 2, 3, 4]; // 手指编号1-4
    let mut seen = HashSet::new();

    for combination in generate_combinations_iter(chord, &finger_list) {
        if verify_valid_combination(&combination) {
            let fingerprint = combination.fingerprint();
            if seen.insert(fingerprint) {
                result.push(combination);
            }
        }
    }

    result
}

fn generate_combinations_iter(
    note_list: &[NotePosition],
    finger_list: &[i32],
) -> Vec<HandPosition> {
    if note_list.is_empty() {
        return vec![HandPosition::new(vec![])];
    }

    let mut results = Vec::new();
    let first_note = &note_list[0];
    let rest_notes = &note_list[1..];

    // 空弦不需要分配手指
    if first_note.fret == 0 {
        for mut combination in generate_combinations_iter(rest_notes, finger_list) {
            let position = FingerPosition {
                string_index: first_note.string_index,
                fret: first_note.fret,
                finger: -1,
            };
            combination.positions.insert(0, position);
            results.push(combination);
        }
    } else {
        // 需要分配手指的音符
        for &finger in finger_list {
            for mut combination in generate_combinations_iter(rest_notes, finger_list) {
                let position = FingerPosition {
                    string_index: first_note.string_index,
                    fret: first_note.fret,
                    finger: finger,
                };
                combination.positions.insert(0, position);
                results.push(combination);
            }
        }
    }

    results
}

fn verify_valid_combination(combination: &HandPosition) -> bool {
    // 过滤出包含手指的元素
    let finger_positions: Vec<&FingerPosition> = combination
        .positions
        .iter()
        .filter(|p| p.finger != -1)
        .collect();

    if finger_positions.len() < 2 {
        return true;
    }

    // 按手指编号排序
    let mut sorted_positions = finger_positions.clone();
    sorted_positions.sort_by_key(|p| p.finger);

    // 检查手指编号和品位的关系
    for i in 0..sorted_positions.len() - 1 {
        let current = sorted_positions[i];
        let next = sorted_positions[i + 1];

        let current_finger = current.finger;
        let next_finger = next.finger;

        // 如果手指编号小但品位大，或手指编号大但品位小
        if (current_finger > next_finger && current.fret < next.fret)
            || (current_finger < next_finger && current.fret > next.fret)
        {
            return false;
        }

        // 检查非食指的跨弦横按
        if current_finger == next_finger && current.fret == next.fret {
            if current_finger != 1
                && (current.string_index as i32 - next.string_index as i32).abs() > 1
            {
                return false;
            }
        }
    }

    // 构建手指-品位映射
    let mut finger_fret_map = std::collections::HashMap::<i32, &FingerPosition>::new();
    for position in &sorted_positions {
        let finger = position.finger;
        if let Some(existing) = finger_fret_map.get(&finger) {
            // 同一个手指按了不同的品位
            if existing.fret != position.fret {
                return false;
            }
        } else {
            finger_fret_map.insert(finger, position);
        }
    }

    // 检查手指间距
    // 中指和无名指相差大于1
    if let (Some(index_finger), Some(middle_finger)) =
        (finger_fret_map.get(&2), finger_fret_map.get(&3))
    {
        if middle_finger.fret - 1 > index_finger.fret {
            return false;
        }
    }

    // 食指和中指相差大于1
    if let (Some(thumb_finger), Some(index_finger)) =
        (finger_fret_map.get(&1), finger_fret_map.get(&2))
    {
        if index_finger.fret - 1 > thumb_finger.fret {
            return false;
        }
    }

    // 食指和无名指相差大于1且跨弦超过1根
    if let (Some(middle_finger), Some(ring_finger)) =
        (finger_fret_map.get(&3), finger_fret_map.get(&4))
    {
        if ring_finger.fret - 1 > middle_finger.fret
            && (ring_finger.string_index as i32 - middle_finger.string_index as i32).abs() > 1
        {
            return false;
        }
    }

    true
}
