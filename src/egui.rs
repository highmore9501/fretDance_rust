use crate::midi::midi_to_note::MidiProcessor;
use crate::ui::app::FretDanceApp;
use std::fs;
use std::path::Path;

impl FretDanceApp {
    pub fn load_midi_options(&mut self) {
        let midi_path = "asset/midi";
        if let Ok(entries) = fs::read_dir(midi_path) {
            self.midi_options = entries
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("mid") {
                            if let Some(file_name) = path.file_name() {
                                return Some(file_name.to_string_lossy().to_string());
                            }
                        }
                    }
                    None
                })
                .collect();

            if !self.midi_options.is_empty() && self.midi_file_path.is_empty() {
                self.midi_file_path = format!("asset/midi/{}", self.midi_options[0]);
            }
        }
    }

    fn parse_track_numbers(&self) -> Vec<i32> {
        self.track_numbers_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }

    pub fn scan_midi_info(&mut self) {
        // 设置扫描状态
        self.scanning_midi = true;
        self.midi_info_result = String::new();

        // 获取MIDI文件名（不含路径和扩展名）
        let midi_file_name = Path::new(&self.midi_file_path)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // 在后台线程中执行扫描操作，避免阻塞UI
        let midi_file_name_clone = midi_file_name.clone();
        let result = std::thread::spawn(move || {
            // 创建MidiProcessor实例
            let midi_processor = MidiProcessor::new();

            // 调用export_midi_info方法
            match midi_processor.export_midi_info(&midi_file_name_clone) {
                Ok(result) => Ok(result),
                Err(e) => Err(format!("扫描MIDI信息时出错: {}", e)),
            }
        });

        // 获取结果
        match result.join() {
            Ok(Ok(info)) => {
                self.midi_info_result = info;
            }
            Ok(Err(e)) => {
                self.midi_info_result = e;
            }
            Err(_) => {
                self.midi_info_result = "扫描过程中发生未知错误".to_string();
            }
        }

        // 重置扫描状态
        self.scanning_midi = false;
    }
}
