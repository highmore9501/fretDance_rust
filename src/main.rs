// main.rs 或 fret_dancer.rs
use fret_dance_rust::*;

use crate::fret_dancer::FretDancer;

fn main() {
    // 主函数
    fn run_fret_dance() -> Result<(), Box<dyn std::error::Error>> {
        let avatar = "户山香澄_E";
        let midi_file_path = "asset/midi/Sunburst.mid";
        let fps = 30;
        let guitar_string_notes = vec!["d", "b", "G", "D", "A", "D1"];
        let track_number = vec![1];
        let octave_down_checkbox = false;
        let capo_number = 0;
        let use_harm_note = false;
        let channel_number = -1;

        FretDancer::main(
            avatar,
            midi_file_path,
            track_number,   // 默认轨道号
            channel_number, // 默认通道号
            fps as f64,
            guitar_string_notes,
            octave_down_checkbox, // octave_down_checkbox
            capo_number,          // capo_number
            use_harm_note,
        )
        .map_err(|e| {
            eprintln!("FretDancer::main 执行失败: {}", e);
            e
        })?;
        Ok(())
    }

    match run_fret_dance() {
        Ok(_) => println!("程序执行成功"),
        Err(e) => {
            eprintln!("程序执行失败: {}", e);
            std::process::exit(1);
        }
    }
}
