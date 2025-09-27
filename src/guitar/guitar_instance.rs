// src/guitar/guitar.rs

use crate::guitar::guitar_string::GuitarString;

#[derive(Debug, Clone)]
pub struct HarmonicNote {
    pub index: i32,
    pub fret: i32,
    pub note: i32,
}
#[derive(Clone)]
pub struct Guitar {
    pub string_distance: f64,
    pub full_string: f64,
    pub guitar_strings: Vec<GuitarString>,
    pub use_harm_notes: bool,
    pub harm_notes: Vec<HarmonicNote>,
}

impl Guitar {
    pub fn new(
        guitar_strings: Vec<GuitarString>,
        use_harm_note: bool,
        string_distance: f64,
        full_string: f64,
    ) -> Self {
        let mut guitar = Guitar {
            string_distance,
            full_string,
            guitar_strings,
            use_harm_notes: use_harm_note,
            harm_notes: Vec::new(),
        };

        guitar.harm_notes = guitar.get_harmonic_notes();
        guitar
    }

    // 带默认值的构造函数
    pub fn with_defaults(guitar_strings: Vec<GuitarString>, use_harm_note: bool) -> Self {
        Self::new(guitar_strings, use_harm_note, 0.85, 64.7954)
    }

    pub fn get_string_distance(&self) -> f64 {
        self.string_distance
    }

    pub fn get_full_string(&self) -> f64 {
        self.full_string
    }

    pub fn get_harmonic_notes(&self) -> Vec<HarmonicNote> {
        let mut all_harm_notes = Vec::new();

        for string in &self.guitar_strings {
            let base_note_num = &string.get_base_note();
            let string_index = string.get_string_index();

            // 5th fret harmonic (24 semitones higher)
            all_harm_notes.push(HarmonicNote {
                index: string_index,
                fret: 5,
                note: base_note_num + 24,
            });

            // 7th fret harmonic (19 semitones higher)
            all_harm_notes.push(HarmonicNote {
                index: string_index,
                fret: 7,
                note: base_note_num + 19,
            });

            // 12th fret harmonic (12 semitones higher)
            all_harm_notes.push(HarmonicNote {
                index: string_index,
                fret: 12,
                note: base_note_num + 12,
            });

            // 4th fret harmonic (28 semitones higher)
            all_harm_notes.push(HarmonicNote {
                index: string_index,
                fret: 4,
                note: base_note_num + 28,
            });

            // 9th fret harmonic (28 semitones higher)
            all_harm_notes.push(HarmonicNote {
                index: string_index,
                fret: 9,
                note: base_note_num + 28,
            });
        }

        all_harm_notes
    }

    pub fn get_guitar_strings(&self) -> &Vec<GuitarString> {
        &self.guitar_strings
    }

    pub fn use_harm_notes(&self) -> bool {
        self.use_harm_notes
    }

    pub fn get_harm_notes(&self) -> &Vec<HarmonicNote> {
        &self.harm_notes
    }
}
