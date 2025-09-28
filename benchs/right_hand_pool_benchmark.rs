use criterion::{Criterion, criterion_group, criterion_main};
use fret_dance::hand::right_hand::RightHand;
use fret_dance::recorder::recorder_pool::{HandPoseRecordPool, HandRecorder};
use fret_dance::recorder::right_hand_recorder::RightHandRecorder;
use std::fs;
use std::path::Path;

fn benchmark_update_right_hand_recorder_pool(c: &mut Criterion) {
    // 创建测试用的左手记录文件
    let left_hand_recorder_file = "output\\hand_recorder\\Sunburst_1_lefthand_recorder.json";

    // 创建测试用的进度回调函数
    let progress_callback = |message: &str| {
        println!("{}", message);
    };

    c.bench_function("update_right_hand_recorder_pool", |b| {
        b.iter(|| {
            // 初始化右手记录器池
            let init_right_hand = RightHand::new(
                vec![],
                vec![5, 2, 1, 0], // 假设最大弦数为5
                vec![],
                false,
                false, // is_playing_bass
            );

            let mut init_right_hand_recorder = RightHandRecorder::new();
            init_right_hand_recorder.add_hand_pose(init_right_hand, 0.0, 0.0);

            let mut right_hand_record_pool = HandPoseRecordPool::new(100);
            right_hand_record_pool.insert_new_hand_pose_recorder(
                HandRecorder::Right(init_right_hand_recorder),
                Some(0),
            );

            // 执行测试的函数
            let _ = right_hand_record_pool.update_right_hand_recorder_pool(
                left_hand_recorder_file,
                5, // max_string_index
                &progress_callback,
                false, // is_play_bass
            );

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
        })
    });

    // 清理测试文件
    if Path::new(left_hand_recorder_file).exists() {
        let _ = fs::remove_file(left_hand_recorder_file);
    }
}

criterion_group!(benches, benchmark_update_right_hand_recorder_pool);
criterion_main!(benches);
