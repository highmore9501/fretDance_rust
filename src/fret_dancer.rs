// main.rs 或 fret_dancer.rs
use serde_json;
use std::fs::File;

use crate::animate::animator::Animator;
use crate::guitar::guitar_instance::Guitar;
use crate::guitar::guitar_string::create_guitar_strings;
use crate::hand::left_finger::LeftFinger;
use crate::hand::left_hand::LeftHand;
use crate::hand::right_hand::RightHand;
use crate::midi::midi_to_note::MidiProcessor;
use crate::recorder::left_hand_recorder::LeftHandRecorder;
use crate::recorder::recorder_pool::{HandPoseRecordPool, HandRecorder};
use crate::recorder::right_hand_recorder::RightHandRecorder;

pub struct FretDancer;

impl FretDancer {
    pub fn main(
        avatar: &str,
        midi_file_path: &str,
        track_number: Vec<i32>,
        channel_number: i32,
        fps: f64,
        guitar_string_notes: Vec<&str>,
        octave_down_checkbox: bool,
        capo_number: i32,
        use_harm_notes: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 处理文件路径
        let filename = midi_file_path
            .split("/")
            .last()
            .unwrap_or(midi_file_path)
            .split(".")
            .next()
            .unwrap_or("unknown");

        let track_number_string = track_number
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
            avatar, filename, track_number_string
        );
        let right_hand_recorder_file = format!(
            "output/hand_recorder/{}_{}_righthand_recorder.json",
            filename, track_number_string
        );
        let right_hand_animation_file = format!(
            "output/hand_animation/{}_{}_{}_righthand_animation.json",
            avatar, filename, track_number_string
        );
        let guitar_string_recorder_file = format!(
            "output/string_recorder/{}_{}_guitar_string_recorder.json",
            filename, track_number_string
        );

        let midi_processor = MidiProcessor::new();

        // 获取MIDI信息
        let (tempo_changes, ticks_per_beat) = midi_processor.get_tempo_changes(midi_file_path)?;
        let (notes_map, _pitch_wheel_map, messages) = midi_processor.midi_to_guitar_notes(
            midi_file_path,
            &tempo_changes,
            ticks_per_beat,
            fps,
            &track_number,
            channel_number,
            octave_down_checkbox,
            capo_number,
        )?;

        // 保存MIDI信息
        let notes_map_file_handle = File::create(&notes_map_file)?;
        serde_json::to_writer_pretty(notes_map_file_handle, &notes_map)?;

        let messages_file_handle = File::create(&messages_file)?;
        serde_json::to_writer_pretty(messages_file_handle, &messages)?;

        // 打印速度变化信息
        println!("全曲的速度变化是:");
        // 正确的访问方式
        for tempo_change in tempo_changes.iter() {
            println!(
                "在{}轨，tick为{}时，速度变为{}",
                tempo_change.track, tempo_change.time, tempo_change.tempo
            );
        }

        println!("\n全曲的每拍tick数是:{}\n", ticks_per_beat);

        // 计算总时间
        let total_tick = notes_map.last().map_or(0.0, |note| note.real_tick);
        let total_frame =
            midi_processor.calculate_frame(&tempo_changes, ticks_per_beat, fps as f64, total_tick);
        let total_time = total_frame as f64 / fps as f64;

        println!(
            "如果以{}的fps做成动画，一共是{} ticks, 合计{}帧, 约{}秒",
            fps, total_tick, total_frame, total_time
        );

        // 初始化吉他
        let guitar_string_list = create_guitar_strings(&guitar_string_notes);
        let max_string_index = guitar_string_list.len() - 1;
        let guitar = Guitar::with_defaults(guitar_string_list, use_harm_notes);

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
        let total_steps = notes_map.len();

        println!("开始生成左手按弦数据");

        // 更新记录器池
        left_hand_pose_record_pool.update_left_handrecorder_pool(
            &guitar,
            &notes_map,
            &midi_processor,
            &mut current_recorder_num,
            &mut previous_recorder_num,
        );

        // 获取最优解
        let best_hand_pose_record = left_hand_pose_record_pool.get_best_recorder();
        let best_entropy = best_hand_pose_record.current_entropy();

        println!("最小消耗熵为：{}", best_entropy);

        best_hand_pose_record.save(
            &left_hand_recorder_file,
            &tempo_changes,
            ticks_per_beat,
            fps,
        )?;

        println!("总音符数应该为{}", total_steps);
        println!("实际输出音符数为{}", best_hand_pose_record.len());

        let animator = Animator::new(
            avatar.to_string(),
            left_hand_recorder_file.to_string(),
            left_hand_animation_file.to_string(),
            fps,
            max_string_index as f64,
        )?;

        animator.left_hand_2_animation(false)?;

        // 处理右手部分
        println!("开始生成右手演奏数据");

        if avatar.ends_with("_E") {
            let right_hand_recorder_data = animator.left_hand_2_electronic_right_hand(
                &left_hand_recorder_file,
                &right_hand_recorder_file,
            )?;

            animator.electronic_right_hand_2_animation(
                &right_hand_recorder_data,
                &right_hand_animation_file,
            )?;
        } else {
            let init_right_hand = RightHand::new(
                vec![],
                vec![max_string_index as i32, 2, 1, 0],
                vec![],
                false,
                false,
            );

            let mut init_right_hand_recorder = RightHandRecorder::new();
            init_right_hand_recorder.add_hand_pose(init_right_hand, 0.0, 0.0);

            let mut right_hand_record_pool = HandPoseRecordPool::new(100);
            right_hand_record_pool.insert_new_hand_pose_recorder(
                HandRecorder::Right(init_right_hand_recorder),
                Some(0),
            );

            right_hand_record_pool
                .update_right_hand_recorder_pool(&left_hand_recorder_file, max_string_index)?;

            // 获取最优解
            let best_right_hand_pose_record = right_hand_record_pool.get_best_recorder();
            let best_right_entropy = best_right_hand_pose_record.current_entropy();

            println!("最小消耗熵为：{}", best_right_entropy);

            best_right_hand_pose_record.save(
                &right_hand_recorder_file,
                &tempo_changes,
                ticks_per_beat,
                fps,
            )?;

            animator
                .right_hand_2_animation(&right_hand_recorder_file, &right_hand_animation_file)?;
        }

        println!("开始生成吉他弦动画数据");
        animator.animated_guitar_string(&left_hand_recorder_file, &guitar_string_recorder_file)?;

        let final_info = format!(
            "全部执行完毕:\nrecorder文件被保存到了:{} 和 {}\n动画文件被保存到了:{} 和 {}\n吉它弦动画文件被保存到了:{}",
            left_hand_recorder_file,
            right_hand_recorder_file,
            left_hand_animation_file,
            right_hand_animation_file,
            guitar_string_recorder_file
        );

        println!("{}", final_info);
        Ok(final_info)
    }
}
