// Cargo.toml dependencies:
// [dependencies]
// midly = "0.5"
// rand = "0.8"

use midly::{MetaMessage, Smf, Timing, TrackEventKind};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

pub struct MidiProcessor {
    midi_instruments: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct TempoChange {
    pub track: usize,
    pub tempo: u32,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInfo {
    pub notes: Vec<i32>,
    pub real_tick: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchWheelInfo {
    pub pitchwheel: i16,
    pub real_tick: f64,
    pub frame: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub message: String,
    pub real_tick: f64,
}

impl MidiProcessor {
    pub fn new() -> Self {
        let midi_instruments = vec![
            "Acoustic Grand Piano",
            "Bright Acoustic Piano",
            "Electric Grand Piano",
            "Honky-tonk Piano",
            "Electric Piano 1",
            "Electric Piano 2",
            "Harpsichord",
            "Clavi",
            "Celesta",
            "Glockenspiel",
            "Music Box",
            "Vibraphone",
            "Marimba",
            "Xylophone",
            "Tubular Bells",
            "Dulcimer",
            "Drawbar Organ",
            "Percussive Organ",
            "Rock Organ",
            "Church Organ",
            "Reed Organ",
            "Accordion",
            "Harmonica",
            "Tango Accordion",
            "Acoustic Guitar (nylon)",
            "Acoustic Guitar (steel)",
            "Electric Guitar (jazz)",
            "Electric Guitar (clean)",
            "Electric Guitar (muted)",
            "Overdriven Guitar",
            "Distortion Guitar",
            "Guitar Harmonics",
            "Acoustic Bass",
            "Electric Bass (finger)",
            "Electric Bass (pick)",
            "Fretless Bass",
            "Slap Bass 1",
            "Slap Bass 2",
            "Synth Bass 1",
            "Synth Bass 2",
            "Violin",
            "Viola",
            "Cello",
            "Contrabass",
            "Tremolo Strings",
            "Pizzicato Strings",
            "Orchestral Harp",
            "Timpani",
            "String Ensemble 1",
            "String Ensemble 2",
            "Synth Strings 1",
            "Synth Strings 2",
            "Choir Aahs",
            "Voice Oohs",
            "Synth Voice",
            "Orchestra Hit",
            "Trumpet",
            "Trombone",
            "Tuba",
            "Muted Trumpet",
            "French Horn",
            "Brass Section",
            "Synth Brass 1",
            "Synth Brass 2",
            "Soprano Sax",
            "Alto Sax",
            "Tenor Sax",
            "Baritone Sax",
            "Oboe",
            "English Horn",
            "Bassoon",
            "Clarinet",
            "Piccolo",
            "Flute",
            "Recorder",
            "Pan Flute",
            "Blown Bottle",
            "Shakuhachi",
            "Whistle",
            "Ocarina",
            "Lead 1 (square)",
            "Lead 2 (sawtooth)",
            "Lead 3 (calliope)",
            "Lead 4 (chiff)",
            "Lead 5 (charang)",
            "Lead 6 (voice)",
            "Lead 7 (fifths)",
            "Lead 8 (bass + lead)",
            "Pad 1 (new age)",
            "Pad 2 (warm)",
            "Pad 3 (polysynth)",
            "Pad 4 (choir)",
            "Pad 5 (bowed)",
            "Pad 6 (metallic)",
            "Pad 7 (halo)",
            "Pad 8 (sweep)",
            "FX 1 (rain)",
            "FX 2 (soundtrack)",
            "FX 3 (crystal)",
            "FX 4 (atmosphere)",
            "FX 5 (brightness)",
            "FX 6 (goblins)",
            "FX 7 (echoes)",
            "FX 8 (sci-fi)",
            "Sitar",
            "Banjo",
            "Shamisen",
            "Koto",
            "Kalimba",
            "Bagpipe",
            "Fiddle",
            "Shanai",
            "Tinkle Bell",
            "Agogo",
            "Steel Drums",
            "Woodblock",
            "Taiko Drum",
            "Melodic Tom",
            "Synth Drum",
            "Reverse Cymbal",
            "Guitar Fret Noise",
            "Breath Noise",
            "Seashore",
            "Bird Tweet",
            "Telephone Ring",
            "Helicopter",
            "Applause",
            "Gunshot",
        ];

        MidiProcessor { midi_instruments }
    }

    pub fn calculate_frame(
        &self,
        tempo_changes: &Vec<TempoChange>,
        ticks_per_beat: u16,
        fps: f64,
        real_tick: f64,
    ) -> f64 {
        let mut total_frames = 0.0;

        for i in 0..tempo_changes.len() {
            let current = &tempo_changes[i];

            // 如果当前的时间已经超过了real_tick，那么就停止计算
            if current.time as f64 > real_tick {
                break;
            }

            // 获取下一个时间点，如果没有下一个时间点，或者下一个时间点超过了real_tick，那么就使用real_tick
            let next_time = if i + 1 < tempo_changes.len() {
                tempo_changes[i + 1].time as f64
            } else {
                real_tick
            }
            .min(real_tick);

            // 计算当前时间点和下一个时间点之间的秒数
            let seconds = (next_time - current.time as f64) * current.tempo as f64
                / (ticks_per_beat as f64 * 1000000.0);

            // 将秒数转换为帧数
            let frames = seconds * fps;

            // 累加帧数
            total_frames += frames;
        }

        total_frames
    }

    pub fn get_tempo_changes(
        &self,
        midi_file_path: &str,
    ) -> Result<(Vec<TempoChange>, u16), Box<dyn std::error::Error>> {
        let data = std::fs::read(midi_file_path)?;
        let smf = Smf::parse(&data)?;

        let ticks_per_beat = match smf.header.timing {
            Timing::Metrical(t) => t.as_int(),
            _ => 480, // 默认值
        };

        let mut tempo_changes = Vec::new();

        for (i, track) in smf.tracks.iter().enumerate() {
            let mut absolute_time = 0u64;

            for event in track {
                absolute_time += event.delta.as_int() as u64;

                if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event.kind {
                    tempo_changes.push(TempoChange {
                        track: i,
                        tempo: tempo.as_int(),
                        time: absolute_time,
                    });
                }
            }
        }

        Ok((tempo_changes, ticks_per_beat))
    }

    pub fn export_midi_info(&self, midi_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let midi_file_path = format!("asset/midi/{}.mid", midi_name);
        let data = std::fs::read(&midi_file_path)?;
        let smf = Smf::parse(&data)?;

        // 写入详细信息到文件
        let output_path = "output/current_midi_info.txt";
        let mut file = File::create(output_path)?;

        if let Some(track) = smf.tracks.get(0) {
            for event in track {
                writeln!(file, "{:?}", event)?;
            }
        }

        let mut result = String::new();

        // 用于统计每个channel的note_on事件数量
        let mut note_count_by_channel: std::collections::HashMap<u8, u32> =
            std::collections::HashMap::new();

        // 存储每个(track, channel)组合最新的乐器
        let mut channel_instruments: std::collections::HashMap<(usize, u8), &str> =
            std::collections::HashMap::new();

        // 统计所有note_on消息
        for track in &smf.tracks {
            for event in track {
                if let TrackEventKind::Midi { channel, message } = event.kind {
                    if let midly::MidiMessage::NoteOn { .. } = message {
                        *note_count_by_channel.entry(channel.as_int()).or_insert(0) += 1;
                    }
                }
            }
        }

        // 跟踪每个channel最后使用的乐器
        for (i, track) in smf.tracks.iter().enumerate() {
            // 从轨道事件中查找轨道名称
            let track_name = track
                .iter()
                .find_map(|event| {
                    if let TrackEventKind::Meta(MetaMessage::TrackName(name)) = event.kind {
                        std::str::from_utf8(name).ok()
                    } else {
                        None
                    }
                })
                .unwrap_or("Unknown");

            result.push_str(&format!("Track {}: {}\n", i, track_name));

            for event in track {
                if let TrackEventKind::Midi { channel, message } = event.kind {
                    if let midly::MidiMessage::ProgramChange { program } = message {
                        let instrument_idx = program.as_int() as usize;
                        if instrument_idx < self.midi_instruments.len() {
                            let instrument = self.midi_instruments[instrument_idx];
                            channel_instruments.insert((i, channel.as_int()), instrument);
                        }
                    }
                }
            }
        }

        // 创建一个按channel聚合的乐器信息
        let mut channel_latest_instrument: std::collections::HashMap<u8, &str> =
            std::collections::HashMap::new();
        for ((_, channel), instrument) in &channel_instruments {
            channel_latest_instrument.insert(*channel, instrument);
        }

        // 输出每个channel的信息
        let mut channels: Vec<&u8> = note_count_by_channel.keys().collect();
        channels.sort();

        for channel in channels {
            let note_count = note_count_by_channel[channel];
            if let Some(instrument) = channel_latest_instrument.get(channel) {
                result.push_str(&format!(
                    "Channel {}: {} ({} notes)\n",
                    channel, instrument, note_count
                ));
            } else {
                result.push_str(&format!(
                    "Channel {}: Unknown instrument ({} notes)\n",
                    channel, note_count
                ));
            }
        }

        Ok(result)
    }
    pub fn midi_to_guitar_notes(
        &self,
        midi_file_path: &str,
        tempo_changes: &Vec<TempoChange>,
        ticks_per_beat: u16,
        fps: f64,
        use_tracks: &[i32],
        use_channel: i32,
        octave_down_checkbox: bool,
        capo_number: i32,
    ) -> Result<(Vec<NoteInfo>, Vec<PitchWheelInfo>, Vec<MessageInfo>), Box<dyn std::error::Error>>
    {
        let data = std::fs::read(midi_file_path)?;
        let smf = Smf::parse(&data)?;

        let mut notes_map = Vec::new();
        let mut pitch_wheel_map = Vec::new();
        let mut messages = Vec::new();

        for &track_index in use_tracks {
            if track_index as usize >= smf.tracks.len() {
                continue;
            }

            let track = &smf.tracks[track_index as usize];
            let mut note = Vec::new();
            let mut real_tick: f64 = 0.0;
            let mut current_tick: f64 = 0.0; // 当前正在处理的音符时间点

            for event in track {
                let ticks = event.delta.as_int() as f64;
                real_tick += ticks;

                match event.kind {
                    TrackEventKind::Midi { channel, message } => {
                        let channel_num = channel.as_int() as i32;

                        if channel_num == use_channel || use_channel == -1 {
                            messages.push(MessageInfo {
                                message: format!("{:?}", event),
                                real_tick,
                            });

                            match message {
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    // 只有当音符开启(velocity > 0)时才处理
                                    if vel.as_int() > 0 {
                                        // 如果当前时间与之前记录的时间不同，说明是新的一组音符
                                        if current_tick != real_tick && !note.is_empty() {
                                            // 保存之前收集的音符
                                            note.sort();
                                            notes_map.push(NoteInfo {
                                                notes: note.clone(),
                                                real_tick: current_tick,
                                            });
                                            note.clear(); // 重置音符列表
                                        }

                                        // 更新当前时间点
                                        current_tick = real_tick;

                                        // 添加新音符
                                        let mut note_value = key.as_int() as i32;
                                        if octave_down_checkbox {
                                            note_value -= 12;
                                        }
                                        note_value -= capo_number;
                                        note.push(note_value);
                                    } else {
                                        // velocity为0表示音符关闭，处理非note_on事件，如果当前有音符则保存
                                        if !note.is_empty() {
                                            note.sort();
                                            notes_map.push(NoteInfo {
                                                notes: note.clone(),
                                                real_tick: current_tick,
                                            });
                                            note.clear();
                                        }
                                    }
                                }
                                midly::MidiMessage::NoteOff { .. } => {
                                    // 处理note_off事件，如果当前有音符则保存
                                    if !note.is_empty() {
                                        note.sort();
                                        notes_map.push(NoteInfo {
                                            notes: note.clone(),
                                            real_tick: current_tick,
                                        });
                                        note.clear();
                                    }
                                }
                                midly::MidiMessage::PitchBend { bend } => {
                                    pitch_wheel_map.push(PitchWheelInfo {
                                        pitchwheel: bend.0.as_int() as i16,
                                        real_tick,
                                        frame: self.calculate_frame(
                                            &tempo_changes,
                                            ticks_per_beat,
                                            fps,
                                            real_tick,
                                        ),
                                    });
                                }
                                _ => {
                                    // 处理其他MIDI事件，如果当前有音符则保存
                                    if !note.is_empty() {
                                        note.sort();
                                        notes_map.push(NoteInfo {
                                            notes: note.clone(),
                                            real_tick: current_tick,
                                        });
                                        note.clear();
                                    }
                                }
                            }
                        }
                    }
                    TrackEventKind::Meta(_) => {
                        // 处理元事件，如果当前有音符则保存
                        if !note.is_empty() {
                            note.sort();
                            notes_map.push(NoteInfo {
                                notes: note.clone(),
                                real_tick: current_tick,
                            });
                            note.clear();
                        }
                    }
                    _ => {
                        // 处理其他事件，如果当前有音符则保存
                        if !note.is_empty() {
                            note.sort();
                            notes_map.push(NoteInfo {
                                notes: note.clone(),
                                real_tick: current_tick,
                            });
                            note.clear();
                        }
                    }
                }
            }

            // 处理最后一个音符组
            if !note.is_empty() {
                note.sort();
                notes_map.push(NoteInfo {
                    notes: note,
                    real_tick: current_tick,
                });
            }
        }

        // 按real_tick排序
        notes_map.sort_by(|a, b| a.real_tick.partial_cmp(&b.real_tick).unwrap());
        pitch_wheel_map.sort_by(|a, b| a.real_tick.partial_cmp(&b.real_tick).unwrap());
        messages.sort_by(|a, b| a.real_tick.partial_cmp(&b.real_tick).unwrap());

        Ok((notes_map, pitch_wheel_map, messages))
    }
    pub fn processed_notes(&self, chord_notes: &[i32], min: i32, max: i32) -> Vec<i32> {
        let compressed = self.compress_notes(chord_notes, min, max);
        self.simplify_notes(&compressed)
    }

    pub fn compress_notes(&self, chord_notes: &[i32], min: i32, max: i32) -> Vec<i32> {
        let mut new_chord = Vec::new();

        for &note in chord_notes {
            let mut adjusted_note = note;

            while adjusted_note < min {
                adjusted_note += 12;
            }

            while adjusted_note > max {
                adjusted_note -= 12;
            }

            if !new_chord.contains(&adjusted_note) {
                new_chord.push(adjusted_note);
            }
        }

        new_chord.sort();
        new_chord
    }

    pub fn simplify_notes(&self, chord_notes: &[i32]) -> Vec<i32> {
        // 如果音符数量不大于6，直接返回音符
        if chord_notes.len() <= 6 {
            return chord_notes.to_vec();
        }

        let lowest_note = chord_notes[0];
        let highest_note = chord_notes[chord_notes.len() - 1];
        let mut middle_notes = chord_notes[1..chord_notes.len() - 1].to_vec();
        let number_of_notes_need_remove = chord_notes.len() - 6;
        let mut number_of_note_removed = 0;

        // 移除与最低音或者最高音有八度关系的音符
        middle_notes.retain(|&note| {
            let should_remove = (note - lowest_note) % 12 == 0 || (highest_note - note) % 12 == 0;
            if should_remove && number_of_note_removed < number_of_notes_need_remove {
                number_of_note_removed += 1;
                false
            } else {
                true
            }
        });

        // 如果还有音符需要移除，那么随机从中间音符里挑出来需要移除的音符
        while number_of_note_removed < number_of_notes_need_remove && !middle_notes.is_empty() {
            let mut rng = rand::thread_rng();
            let random_index = rng.gen_range(0..middle_notes.len());
            middle_notes.remove(random_index);
            number_of_note_removed += 1;
        }

        let mut result = vec![lowest_note];
        result.extend(middle_notes);
        result.push(highest_note);

        result
    }
}
