use crate::guitar::guitar_instance::Guitar;

#[derive(Debug, Clone)]
pub struct Chord {
    pub positions: Vec<NotePosition>,
}

#[derive(Debug, Clone)]
pub struct NotePosition {
    pub string_index: i32,
    pub fret: i32,
}

impl Chord {
    pub fn new(positions: Vec<NotePosition>) -> Self {
        Chord { positions }
    }

    pub fn get_frets(&self) -> Vec<i32> {
        self.positions
            .iter()
            .filter(|pos| pos.fret > 0)
            .map(|pos| pos.fret)
            .collect()
    }

    pub fn get_string_indices(&self) -> Vec<i32> {
        self.positions.iter().map(|pos| pos.string_index).collect()
    }

    pub fn has_duplicate_strings(&self) -> bool {
        let string_indices = self.get_string_indices();
        let unique_count = string_indices
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        string_indices.len() != unique_count
    }

    pub fn is_playable(&self) -> bool {
        if self.has_duplicate_strings() {
            return false;
        }

        let frets = self.get_frets();
        if frets.is_empty() {
            return true;
        }

        // 四个手指按不下超过4个不同的fret
        let unique_frets_count = frets.iter().collect::<std::collections::HashSet<_>>().len();
        if unique_frets_count > 4 {
            return false;
        }

        // 检查把位跨度限制
        if let (Some(&max_fret), Some(&min_fret)) = (frets.iter().max(), frets.iter().min()) {
            let span = max_fret - min_fret;
            let out_limit_on_low_bar = min_fret < 8 && span > 5;
            let out_limit_on_high_bar = min_fret >= 8 && span > 6;

            if out_limit_on_low_bar || out_limit_on_high_bar {
                return false;
            }
        }

        true
    }
}

pub fn convert_notes_to_chord(notes: &Vec<i32>, guitar: &Guitar) -> Vec<Chord> {
    let use_harm_notes = guitar.use_harm_notes;
    let mut note_positions: Vec<Vec<NotePosition>> = Vec::new();
    let mut result: Vec<Chord> = Vec::new();

    // 为每个音符找到所有可能的弦和品位组合
    for &note in notes {
        let mut possible_positions: Vec<NotePosition> = Vec::new();

        for guitar_string in &guitar.guitar_strings {
            let string_index = guitar_string.get_string_index();

            // 处理泛音音符
            if use_harm_notes {
                for harm_note in &guitar.harm_notes {
                    if harm_note.index == string_index && harm_note.note == note {
                        possible_positions.push(NotePosition {
                            string_index,
                            fret: harm_note.fret,
                        });
                    }
                }
            }

            // 处理普通音符
            if let Some(normal_fret) = guitar_string.get_fret_by_note(note) {
                // 低音弦的超高把位是无法按的
                if string_index <= 2 || normal_fret <= 16 {
                    possible_positions.push(NotePosition {
                        string_index,
                        fret: normal_fret,
                    });
                }
            }
        }

        note_positions.push(possible_positions);
    }

    // 生成所有可能的组合
    if !note_positions.is_empty() {
        let combinations = cartesian_product(&note_positions);

        for combination in combinations {
            let chord = Chord::new(combination);

            // 检查是否可演奏
            if chord.is_playable() {
                result.push(chord);
            }
        }
    }

    result
}

// 计算多个向量的笛卡尔积
fn cartesian_product<T: Clone>(v: &[Vec<T>]) -> Vec<Vec<T>> {
    if v.is_empty() {
        return vec![];
    }

    let mut result = vec![vec![]];

    for inner_vec in v {
        if inner_vec.is_empty() {
            continue;
        }

        let mut temp = Vec::new();
        for existing in &result {
            for item in inner_vec {
                let mut new_vec = existing.clone();
                new_vec.push(item.clone());
                temp.push(new_vec);
            }
        }
        result = temp;
    }

    result
}
