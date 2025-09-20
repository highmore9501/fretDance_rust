// GuitarString.rs

use crate::guitar::music_note::{KEYNOTES, MusicNote};
/// Represents a guitar string with its base note and index.
///
/// Params:
/// - base_note: Base note of the string. 弦的基音
/// - string_index: Index of the string, starting with the highest pitch as string 0. 弦的索引,以最高音为0弦开始计算
#[derive(Debug, Clone)]
pub struct GuitarString {
    base_note: MusicNote,
    string_index: i32,
}

impl GuitarString {
    /// Creates a new GuitarString instance.
    pub fn new(base_note: MusicNote, string_index: i32) -> Self {
        GuitarString {
            base_note,
            string_index,
        }
    }

    /// Returns the base note number of the string.
    pub fn get_base_note(&self) -> i32 {
        self.base_note.num
    }

    /// Returns the string index.
    pub fn get_string_index(&self) -> i32 {
        self.string_index
    }

    /// Calculates the fret position for a given note.
    ///
    /// Returns the fret number if valid, otherwise returns None.
    pub fn get_fret_by_note(&self, note: i32) -> Option<i32> {
        let fret = note - self.base_note.num;
        if fret < 0 || fret > 23 {
            None
        } else {
            Some(fret)
        }
    }
}

/// Creates a vector of GuitarString instances from a list of note strings.
pub fn create_guitar_strings(notes: &Vec<&str>) -> Vec<GuitarString> {
    let mut guitar_strings = Vec::new();

    for (index, note) in notes.iter().enumerate() {
        let note_num = get_keynote_by_value(note);
        let base_note = MusicNote::new(note_num);
        let index = index as i32;
        // 第一弦是高音e弦
        guitar_strings.push(GuitarString::new(base_note, index));
    }

    guitar_strings
}

/// Transforms the note to an integer value, C is 48.
///
/// Params:
/// - value: keynote such as "C", "F1"
/// Returns: an integer value. 一个整数值
pub fn get_keynote_by_value(value: &str) -> i32 {
    // 如果value在KEYNOTES中，直接返回
    if let Some(&num) = KEYNOTES.get(value) {
        return num;
    }
    // 如果value是单个小写字母，表示为高音
    else if value.len() == 1 && value.chars().all(|c| c.is_ascii_lowercase()) {
        let upper_value = value.to_uppercase();
        if let Some(&num) = KEYNOTES.get(&upper_value.as_str()) {
            return num;
        }
    }
    // 如果value长度大于1，并且最后一个值是一个数字
    else if value.len() > 1 && value.chars().last().unwrap().is_digit(10) {
        let last_char_index = value.len() - 1;
        let prefix = &value[..last_char_index];
        let digit = value[last_char_index..].parse::<i32>().unwrap();

        // 如果第一个值是小写并在KEYNOTES中，说明当前值是高音，数字越大音越高
        if prefix.chars().all(|c| c.is_ascii_lowercase()) {
            let first_char = &prefix[0..1].to_uppercase();
            if let Some(num) = KEYNOTES.get(&first_char.as_str()) {
                return num + 12 * digit;
            }
        }
        // 如果第一个值是大写并在KEYNOTES中，说明当前值是低音，数字越大音越低
        else if prefix.chars().all(|c| c.is_ascii_uppercase()) {
            let upper_prefix = prefix.to_uppercase();
            if let Some(&num) = KEYNOTES.get(&upper_prefix.as_str()) {
                return num - 12 * digit;
            }
        }
    }
    // 处理带#号的音符
    else if value.len() > 1
        && value.ends_with('#')
        && value[1..value.len() - 1].chars().all(|c| c.is_digit(10))
    {
        let prefix = &value[0..1];
        let middle_digits = &value[1..value.len() - 1];
        let digit = middle_digits.parse::<i32>().unwrap_or(0);

        // 如果第一个值是小写并在KEYNOTES中，说明当前值是高音，数字越大音越高
        if prefix.chars().all(|c| c.is_ascii_lowercase()) {
            let upper_prefix = prefix.to_uppercase();
            if let Some(&num) = KEYNOTES.get(&upper_prefix.as_str()) {
                return num + 12 * digit + 1;
            }
        }
        // 如果第一个值是大写并在KEYNOTES中，说明当前值是低音，数字越大音越低
        else if prefix.chars().all(|c| c.is_ascii_uppercase()) {
            let upper_prefix = prefix.to_uppercase();
            if let Some(&num) = KEYNOTES.get(&upper_prefix.as_str()) {
                return num - 12 * digit + 1;
            }
        }
    }
    // 处理不带数字的大写音符
    else if value.chars().all(|c| c.is_ascii_uppercase()) {
        if let Some(&num) = KEYNOTES.get(value) {
            return num;
        }
    }

    println!("音符格式有误：{}", value);
    0 // 默认返回0或根据需要返回其他默认值
}
