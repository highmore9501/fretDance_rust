use serde_json;
use std::fs::File;
use std::sync::mpsc;

use crate::animate::animator::Animator;
use crate::guitar::guitar_instance::Guitar;
use crate::guitar::guitar_string::create_guitar_strings;
use crate::guitar::music_note::MusicNote;
use crate::hand::left_finger::LeftFinger;
use crate::hand::left_hand::LeftHand;
use crate::hand::right_hand::RightHand;
use crate::midi::midi_to_note::{MessageInfo, MidiProcessor, NoteInfo};
use crate::recorder::left_hand_recorder::LeftHandRecorder;
use crate::recorder::recorder_pool::{HandPoseRecordPool, HandRecorder};
use crate::recorder::right_hand_recorder::RightHandRecorder;
use crate::ui::app::{AvatarInfo, FretDanceApp};

pub struct FretDancer;

// 添加一个结构体来保存中间状态
#[derive(Clone)]
pub struct FretDancerState {
    pub filename: String,
    pub track_number_string: String,
    pub tempo_changes: Vec<crate::midi::midi_to_note::TempoChange>,
    pub ticks_per_beat: u16,
    pub notes_map: Vec<NoteInfo>,
    pub messages: Vec<MessageInfo>,
    pub guitar: Guitar,
    pub max_string_index: usize,
    pub fps: f64,
    pub disable_barre: bool,
    pub use_harm_notes: bool,
    pub capo_number: i32,
    pub avatar_info: AvatarInfo,
    pub left_hand_recorder_file: String,
    pub left_hand_animation_file: String,
    pub right_hand_recorder_file: String,
    pub right_hand_animation_file: String,
    pub guitar_string_recorder_file: String,
}

impl FretDancer {
    pub fn initialize(
        app: &mut FretDanceApp, // 保留这个参数用于输出信息
        tx: mpsc::Sender<String>,
    ) -> Result<FretDancerState, Box<dyn std::error::Error>> {
        // 创建通道用于通信
        let console_callback = move |message: &str| {
            let _ = tx.send(message.to_string());
        };

        let track_numbers: Result<Vec<i32>, _> = app
            .track_numbers_str
            .split(',')
            .map(|s| s.trim().parse::<i32>())
            .collect();

        let track_numbers = track_numbers.unwrap_or_else(|_| vec![1]);

        let guitar_string_notes: Vec<&str> =
            app.guitar_string_notes.iter().map(|s| s.as_str()).collect();

        let avatar_info = app
            .current_avatar_info
            .clone()
            .ok_or("Avatar info is missing")?;

        // 处理文件路径
        let filename = app
            .midi_file_path
            .split("/")
            .last()
            .unwrap_or(&app.midi_file_path)
            .split(".")
            .next()
            .unwrap_or("unknown");

        let track_number_string = track_numbers
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join("_");

        let notes_map_file = format!(
            "output/midi_info/{}_{}_notes_map.json",
            filename, track_number_string
        );
        let messages_file = format!(
            "output/midi_info/{}_{}_messages.json",
            filename, track_number_string
        );
        let left_hand_recorder_file = format!(
            "output/hand_recorder/{}_{}_lefthand_recorder.json",
            filename, track_number_string
        );
        let left_hand_animation_file = format!(
            "output/hand_animation/{}_{}_{}_lefthand_animation.json",
            avatar_info.name, filename, track_number_string
        );
        let right_hand_recorder_file = format!(
            "output/hand_recorder/{}_{}_righthand_recorder.json",
            filename, track_number_string
        );
        let right_hand_animation_file = format!(
            "output/hand_animation/{}_{}_{}_righthand_animation.json",
            avatar_info.name, filename, track_number_string
        );
        let guitar_string_recorder_file = format!(
            "output/string_recorder/{}_{}_guitar_string_recorder.json",
            filename, track_number_string
        );

        let midi_processor = MidiProcessor::new();

        // 获取MIDI信息
        let (tempo_changes, ticks_per_beat) =
            midi_processor.get_tempo_changes(&app.midi_file_path)?;
        let (notes_map, _pitch_wheel_map, messages) = midi_processor.midi_to_guitar_notes(
            &app.midi_file_path,
            &tempo_changes,
            ticks_per_beat,
            app.fps,
            &track_numbers,
            app.channel_number,
            app.octave_down_checkbox,
            app.capo_number,
        )?;

        // 保存MIDI信息
        let notes_map_file_handle = File::create(&notes_map_file)?;
        serde_json::to_writer_pretty(notes_map_file_handle, &notes_map)?;

        let messages_file_handle = File::create(&messages_file)?;
        serde_json::to_writer_pretty(messages_file_handle, &messages)?;

        // 打印速度变化信息
        console_callback("全曲的速度变化是:");
        // 正确的访问方式
        for tempo_change in tempo_changes.iter() {
            console_callback(&format!(
                "在{}轨，tick为{}时，速度变为{}",
                tempo_change.track, tempo_change.time, tempo_change.tempo
            ));
        }

        console_callback(&format!("全曲的每拍tick数是:{}", ticks_per_beat));

        // 计算总时间
        let total_tick = notes_map.last().map_or(0.0, |note| note.real_tick);
        let total_frame = midi_processor.calculate_frame(
            &tempo_changes,
            ticks_per_beat,
            app.fps as f64,
            total_tick,
        );
        let total_time = total_frame as f64 / app.fps as f64;

        console_callback(&format!(
            "如果以{}的fps做成动画，一共是{} ticks, 合计{}帧, 约{}秒",
            app.fps, total_tick, total_frame, total_time
        ));

        // 初始化吉他
        let guitar_string_list = create_guitar_strings(&guitar_string_notes);
        let max_string_index = guitar_string_list.len() - 1;
        let guitar = Guitar::with_defaults(guitar_string_list, app.use_harm_notes);

        let state = FretDancerState {
            filename: filename.to_string(),
            track_number_string,
            tempo_changes,
            ticks_per_beat,
            notes_map,
            messages,
            guitar,
            max_string_index,
            fps: app.fps,
            disable_barre: app.disable_barre,
            use_harm_notes: app.use_harm_notes,
            capo_number: app.capo_number,
            avatar_info,
            left_hand_recorder_file,
            left_hand_animation_file,
            right_hand_recorder_file,
            right_hand_animation_file,
            guitar_string_recorder_file,
        };
        Ok(state)
    }

    pub fn generate_left_hand_motion(
        app: &mut FretDanceApp,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 克隆state用于线程
        let state = app.fret_dancer_state.as_ref().unwrap().clone();
        // 创建通道用于通信
        let console_callback = move |message: &str| {
            let _ = tx.send(message.to_string());
        };
        // 更新吉他配置以使用泛音
        let guitar = Guitar::with_defaults(state.guitar.guitar_strings.clone(), app.use_harm_notes);

        // 设定各手指状态
        let left_fingers = vec![
            LeftFinger::new(1, &guitar.guitar_strings[2].clone(), 1, "Pressed"),
            LeftFinger::new(2, &guitar.guitar_strings[2].clone(), 2, "Pressed"),
            LeftFinger::new(3, &guitar.guitar_strings[2].clone(), 3, "Pressed"),
            LeftFinger::new(4, &guitar.guitar_strings[2].clone(), 4, "Pressed"),
        ];

        // 初始化左手
        let init_left_hand = LeftHand::new(left_fingers, false, 5.73);

        // 初始化第一个记录器
        let mut left_hand_pose_record = LeftHandRecorder::new();
        left_hand_pose_record.add_hand_pose(init_left_hand, 0.0, 0.0);

        // 初始化记录池
        let mut left_hand_pose_record_pool = HandPoseRecordPool::new(100);
        left_hand_pose_record_pool
            .insert_new_hand_pose_recorder(HandRecorder::Left(left_hand_pose_record), Some(0));

        let mut current_recorder_num = 0;
        let mut previous_recorder_num = 0;

        console_callback("==============================");
        console_callback("开始生成左手按弦数据");

        // 更新记录器池
        left_hand_pose_record_pool.update_left_handrecorder_pool(
            &guitar,
            &state.notes_map,
            &MidiProcessor::new(),
            &mut current_recorder_num,
            &mut previous_recorder_num,
            &console_callback,
        );

        // 获取最优解
        let best_hand_pose_record = left_hand_pose_record_pool.get_best_recorder();
        let best_entropy = best_hand_pose_record.current_entropy();

        console_callback(&format!("最小消耗熵为：{}", best_entropy));
        console_callback(&format!("总音符数应该为{}", state.notes_map.len()));
        console_callback(&format!("实际输出音符数为{}", best_hand_pose_record.len()));

        // 转换HandRecorder为LeftHandRecorder
        let left_hand_recorder = match best_hand_pose_record {
            HandRecorder::Left(recorder) => recorder,
            _ => return Err("Expected LeftHandRecorder".into()),
        };

        left_hand_recorder.save(
            &state.left_hand_recorder_file,
            &state.tempo_changes,
            state.ticks_per_beat,
            state.fps,
        )?;

        let unprocessable_notes = left_hand_pose_record_pool.get_unprocessable_notes();
        if !unprocessable_notes.is_empty() {
            console_callback("生成过程中碰到左手无法按弦的音符组合：");

            // 使用 HashSet 去重
            let mut unique_notes = std::collections::HashSet::new();
            for note in unprocessable_notes {
                unique_notes.insert(&note.notes);
            }

            // 转换为 Vec 并排序以便输出一致
            let mut sorted_notes: Vec<_> = unique_notes.into_iter().collect();
            sorted_notes.sort();

            for notes in sorted_notes {
                // 将数字音符转换为音符名称
                let note_names: Vec<String> = notes
                    .iter()
                    .map(|&num| {
                        let music_note = MusicNote::new(num);
                        music_note.get_keynote()
                    })
                    .collect();

                console_callback(&format!(
                    "音符数字: {:?}, 对应的音符名是: {:?}",
                    notes, note_names
                ));
            }
        }

        Ok(())
    }
    pub fn generate_left_hand_animation(
        app: &mut FretDanceApp,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let state = app.fret_dancer_state.as_ref().unwrap();
        let animator = Animator::new(
            state.avatar_info.file.clone(),
            state.left_hand_recorder_file.clone(),
            state.left_hand_animation_file.clone(),
            state.fps,
            state.max_string_index as f64,
        )?;

        animator.left_hand_2_animation(state.disable_barre)?;

        Ok(state.left_hand_animation_file.clone())
    }

    pub fn generate_right_hand_motion_and_animation(
        app: &mut FretDanceApp,
        tx: mpsc::Sender<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let progress_callback = move |message: &str| {
            let _ = tx.send(message.to_string());
        };

        // 克隆state用于线程
        let state = app.fret_dancer_state.as_ref().unwrap().clone();
        let animator = Animator::new(
            state.avatar_info.file.clone(),
            state.left_hand_recorder_file.clone(),
            state.left_hand_animation_file.clone(),
            state.fps,
            state.max_string_index as f64,
        )?;

        // 处理右手部分
        progress_callback(&format!(
            "开始生成右手演奏数据：{}",
            state.avatar_info.instrument
        ));

        if state.avatar_info.instrument == "electric_guitar" {
            let right_hand_recorder_data = animator.left_hand_2_electronic_right_hand(
                &state.left_hand_recorder_file,
                &state.right_hand_recorder_file,
            )?;

            animator.electronic_right_hand_2_animation(
                &right_hand_recorder_data,
                &state.right_hand_animation_file,
            )?;

            progress_callback("完成右手数据生成");
        } else {
            let is_playing_bass = state.avatar_info.instrument == "bass";
            let init_right_hand = RightHand::new(
                vec![],
                vec![state.max_string_index as i32, 2, 1, 0],
                vec![],
                false,
                is_playing_bass,
            );

            let mut init_right_hand_recorder = RightHandRecorder::new();
            init_right_hand_recorder.add_hand_pose(init_right_hand, 0.0, 0.0);

            let mut right_hand_record_pool = HandPoseRecordPool::new(100);
            right_hand_record_pool.insert_new_hand_pose_recorder(
                HandRecorder::Right(init_right_hand_recorder),
                Some(0),
            );

            right_hand_record_pool.update_right_hand_recorder_pool(
                &state.left_hand_recorder_file,
                state.max_string_index,
                &progress_callback,
            )?;

            // 获取最优解
            let best_right_hand_pose_record = right_hand_record_pool.get_best_recorder();
            let best_right_entropy = best_right_hand_pose_record.current_entropy();

            progress_callback(&format!("最小消耗熵为：{}\n", best_right_entropy));

            best_right_hand_pose_record.save(
                &state.right_hand_recorder_file,
                &state.tempo_changes,
                state.ticks_per_beat,
                state.fps,
            )?;

            animator.right_hand_2_animation(
                &state.right_hand_recorder_file,
                &state.right_hand_animation_file,
            )?;

            progress_callback("完成右手数据生成");
        }

        Ok(state.right_hand_animation_file.clone())
    }
    pub fn generate_string_vibration_data(
        state: &FretDancerState,
        tx: mpsc::Sender<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tx_cloned = tx.clone();
        let console_callback = move |message: &str| {
            let _ = tx_cloned.send(message.to_string());
        };
        let animator = Animator::new(
            state.avatar_info.file.clone(),
            state.left_hand_recorder_file.clone(),
            state.left_hand_animation_file.clone(),
            state.fps,
            state.max_string_index as f64,
        )?;

        // 输出分隔符
        console_callback("==============================");
        console_callback("开始生成吉他弦动画数据");

        animator.animated_guitar_string(
            &state.left_hand_recorder_file,
            &state.guitar_string_recorder_file,
        )?;

        console_callback("完成吉他弦动画数据生成");

        match FretDancer::export_final_report(state, tx.clone()) {
            Ok(()) => {}
            Err(e) => {
                let _ = tx.send(format!("生成最终报告失败: {}", e));
            }
        };

        Ok(state.guitar_string_recorder_file.clone())
    }

    pub fn export_final_report(
        state: &FretDancerState,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let console_callback = move |message: &str| {
            let _ = tx.send(message.to_string());
        };

        let avatar_name = &state.avatar_info.name;
        let file_name = &state.filename;
        let track_number_string = &state.track_number_string;
        let left_hand_animation_file = &state.left_hand_animation_file;
        let right_hand_animation_file = &state.right_hand_animation_file;
        let guitar_string_recorder_file = &state.guitar_string_recorder_file;

        // 获取当前工作目录的绝对路径
        let current_dir = std::env::current_dir()?;

        // 构造绝对路径
        let left_hand_absolute_path = current_dir
            .join(left_hand_animation_file)
            .to_string_lossy()
            .to_string();
        let right_hand_absolute_path = current_dir
            .join(right_hand_animation_file)
            .to_string_lossy()
            .to_string();
        let guitar_string_absolute_path = current_dir
            .join(guitar_string_recorder_file)
            .to_string_lossy()
            .to_string();

        let content = serde_json::json!({
            "left_hand_animation_file": left_hand_absolute_path,
            "right_hand_animation_file": right_hand_absolute_path,
            "guitar_string_recorder_file": guitar_string_absolute_path,
        });

        let report_file = format!(
            "output/final_result/{}_{}_{}.json",
            avatar_name, file_name, track_number_string
        );

        std::fs::create_dir_all("output/final_result")?;
        std::fs::write(&report_file, serde_json::to_string_pretty(&content)?)?;

        // 获取报告文件的绝对路径
        let report_absolute_path = current_dir.join(&report_file).to_string_lossy().to_string();
        console_callback(&format!("报告已保存至: {}", report_absolute_path));
        Ok(())
    }
    pub fn main(
        app: &mut FretDanceApp,
        tx: mpsc::Sender<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 初始化
        let state = Self::initialize(app, tx.clone())?;
        app.fret_dancer_state = Some(state);
        let state_clone = app.fret_dancer_state.as_ref().unwrap().clone();

        // 生成左手动作
        Self::generate_left_hand_motion(app, tx.clone())?;

        // 生成左手动画
        Self::generate_left_hand_animation(app)?;

        // 生成右手动作和动画
        Self::generate_right_hand_motion_and_animation(app, tx.clone())?;

        // 生成弦振动数据
        Self::generate_string_vibration_data(&state_clone, tx.clone())?;

        let final_info = format!(
            "全部执行完毕:\nrecorder文件被保存到了:{} 和 {}\n动画文件被保存到了:{} 和 {}\n吉它弦动画文件被保存到了:{}",
            state_clone.left_hand_recorder_file,
            state_clone.right_hand_recorder_file,
            state_clone.left_hand_animation_file,
            state_clone.right_hand_animation_file,
            state_clone.guitar_string_recorder_file
        );

        Ok(final_info)
    }
}
