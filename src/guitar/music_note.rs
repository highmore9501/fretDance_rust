use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref KEYNOTES: HashMap<&'static str, i32> = {
        let mut m = HashMap::new();
        m.insert("C", 48);
        m.insert("C#", 49);
        m.insert("D", 50);
        m.insert("D#", 51);
        m.insert("E", 52);
        m.insert("F", 53);
        m.insert("F#", 54);
        m.insert("G", 55);
        m.insert("G#", 56);
        m.insert("A", 45);
        m.insert("A#", 46);
        m.insert("B", 47);
        m
    };
}
#[derive(Debug, Clone)]
pub struct MusicNote {
    pub num: i32,
    pub key: String,
}

impl MusicNote {
    /// 创建一个新的 MusicNote 实例
    /// num: 音符数，以C音（吉它标准调弦的5弦3品）为48来计算
    pub fn new(num: i32) -> Self {
        let mut note = MusicNote {
            num,
            key: String::new(),
        };
        note.key = note.get_keynote();
        note
    }

    /// 获取音符名称，如 `C`, [d](file://g:\fretDance\asset\midi\Angelina.mid), `F1`
    pub fn get_keynote(&self) -> String {
        let octave = (self.num - 45) / 12;
        let current_keynotes = get_current_keynotes(octave);

        for (key, &value) in &current_keynotes {
            if value == self.num {
                return key.clone();
            }
        }

        // 默认返回 "c"
        "c".to_string()
    }

    /// 给当前音符添加一个音程
    /// num: 音程，大三度为4，小三度为3，其它类推
    pub fn add(&self, num: i32) -> MusicNote {
        MusicNote::new(self.num + num)
    }
}

/// 根据八度值返回一个当前的音符字典
/// octave: 八度
pub fn get_current_keynotes(octave: i32) -> HashMap<String, i32> {
    let mut current_keynotes = HashMap::new();

    for (key, value) in KEYNOTES.iter() {
        let new_key = if octave == 0 {
            key.to_string()
        } else if octave > 0 {
            // 将首字母转为小写并在后面加上八度数
            let mut chars = key.chars();
            if let Some(first_char) = chars.next() {
                let rest: String = chars.collect();
                format!("{}{}{}", first_char.to_lowercase(), octave, rest)
            } else {
                key.to_string()
            }
        } else {
            // 负八度的情况
            format!("{}{}", key.chars().next().unwrap_or('c'), -octave) + &key[1..]
        };

        current_keynotes.insert(new_key, value + 12 * octave);
    }

    current_keynotes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_music_note_creation() {
        let note = MusicNote::new(48);
        assert_eq!(note.num, 48);
        assert_eq!(note.key, "C");
    }

    #[test]
    fn test_get_current_keynotes() {
        let keynotes = get_current_keynotes(0);
        assert_eq!(keynotes.get("C"), Some(&48));
        assert_eq!(keynotes.get("A"), Some(&45));

        let keynotes_octave_1 = get_current_keynotes(1);
        assert_eq!(keynotes_octave_1.get("c1"), Some(&60)); // C in octave 1
    }

    #[test]
    fn test_add_interval() {
        let note = MusicNote::new(48); // C
        let major_third = note.add(4); // Major third is 4 semitones
        assert_eq!(major_third.num, 52); // E
    }
}
