use criterion::{Criterion, criterion_group, criterion_main};
use fret_dance_rust::hand::right_hand::RightHand;
use fret_dance_rust::recorder::recorder_pool::{HandPoseRecordPool, HandRecorder};
use fret_dance_rust::recorder::right_hand_recorder::RightHandRecorder;

use std::time::Instant;

fn benchmark_update_right_hand_recorder_pool(c: &mut Criterion) {
    c.bench_function("update_right_hand_recorder_pool", |b| {
        b.iter_custom(|_| {
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

            let left_hand_recorder_file =
                "output\\hand_recorder\\Sunburst_1_lefthand_recorder.json";

            // 执行测试的函数
            let start_time = Instant::now();
            eprintln!("开始更新右手记录器池...");

            let result = right_hand_record_pool.update_right_hand_recorder_pool(
                left_hand_recorder_file,
                5, // max_string_index
                |message: &str| {
                    eprintln!("{}", message);
                },
            );

            let duration = start_time.elapsed();
            eprintln!("更新完成，耗时: {:?}", duration);

            match result {
                Ok(_) => {
                    // 获取最优解
                    let best_right_hand_pose_record = right_hand_record_pool.get_best_recorder();
                    let best_right_entropy = best_right_hand_pose_record.current_entropy();
                    eprintln!("最小消耗熵为：{}", best_right_entropy);
                }
                Err(e) => {
                    eprintln!("处理过程中出现错误: {}", e);
                }
            }

            duration
        })
    });
}

criterion_group!(benches, benchmark_update_right_hand_recorder_pool);
criterion_main!(benches);
