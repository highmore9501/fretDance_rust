use crate::midi::midi_to_note::MidiProcessor;
use crate::ui::app::FretDanceApp;

impl FretDanceApp {
    pub fn scan_midi_info(&mut self) {
        // 设置扫描状态
        self.scanning_midi = true;
        self.midi_info_result = String::new();

        // 在后台线程中执行扫描操作，避免阻塞UI
        let midi_file_path_clone = self.midi_file_path.clone();
        let result = std::thread::spawn(move || {
            // 创建MidiProcessor实例
            let midi_processor = MidiProcessor::new();

            // 调用export_midi_info方法，传入完整的文件路径
            match midi_processor.export_midi_info(&midi_file_path_clone) {
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
