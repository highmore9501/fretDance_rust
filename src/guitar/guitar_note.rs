use crate::guitar::guitar_string::GuitarString;
use crate::guitar::music_note::MusicNote;

pub struct GuitarNote {
    _fret: i32,
    _guitar_string: GuitarString,
    note: MusicNote,
}

impl GuitarNote {
    /// 创建新的 GuitarNote 实例
    ///
    /// # 参数
    /// * `guitar_string` - GuitarString 对象，吉他弦对象
    /// * `fret` - 品位数字
    pub fn new(guitar_string: GuitarString, fret: i32) -> Self {
        let num = guitar_string.get_base_note() + fret;
        let note = MusicNote::new(num);
        GuitarNote {
            _fret: fret,
            _guitar_string: guitar_string,
            note,
        }
    }

    /// 获取音符
    pub fn get_note(&self) -> &MusicNote {
        &self.note
    }

    /// 获取品位
    pub fn fret(&self) -> i32 {
        self._fret
    }

    /// 获取吉他弦
    pub fn guitar_string(&self) -> &GuitarString {
        &self._guitar_string
    }
}
