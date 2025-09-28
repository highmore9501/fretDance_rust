use crate::fret_dancer::{FretDancer, FretDancerState};
use crate::hand::right_hand;
use crate::ui::app::{EditAvatarMode, FretDanceApp, InstrumentType, Tab};
use eframe::egui;
use std::sync::mpsc;
use std::thread;

pub fn show_execute_operation(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            // 左半部分：显示所有设置参数
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() * 0.5);

                ui.group(|ui| {
                    ui.heading("角色参数");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Avatar:");
                        ui.monospace(&app.avatar);
                    });

                    ui.horizontal(|ui| {
                        ui.label("使用乐器:");
                        let instrument = app
                            .current_avatar_info
                            .as_ref()
                            .map(|info| info.instrument.as_str())
                            .unwrap_or("未知");
                        ui.monospace(instrument);
                    });

                    ui.horizontal(|ui| {
                        ui.label("FPS:");
                        ui.monospace(app.fps.to_string());
                    });

                    ui.horizontal(|ui| {
                        ui.label("吉他弦音高:");
                        ui.monospace(format!("{:?}", app.guitar_string_notes));
                    });

                    ui.horizontal(|ui| {
                        ui.label("降低八度:");
                        ui.monospace(if app.octave_down_checkbox {
                            "是"
                        } else {
                            "否"
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("变调夹位置:");
                        ui.monospace(app.capo_number.to_string());
                    });

                    ui.horizontal(|ui| {
                        ui.label("使用泛音:");
                        ui.monospace(if app.use_harm_notes { "是" } else { "否" });
                    });

                    ui.horizontal(|ui| {
                        ui.label("禁用横按:");
                        ui.monospace(if app.disable_barre { "是" } else { "否" });
                    });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.heading("MIDI参数");
                    ui.separator();

                    // 显示MIDI文件名而不是完整路径
                    let midi_filename = std::path::Path::new(&app.midi_file_path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(&app.midi_file_path);

                    ui.horizontal(|ui| {
                        ui.label("MIDI文件:");
                        ui.monospace(midi_filename);
                    });

                    ui.horizontal(|ui| {
                        ui.label("轨道号:");
                        ui.monospace(&app.track_numbers_str);
                    });

                    ui.horizontal(|ui| {
                        ui.label("通道号:");
                        ui.monospace(app.channel_number.to_string());
                    });

                    // 显示当前state状态
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("State状态:");
                        ui.monospace(if app.fret_dancer_state.is_some() {
                            "已初始化"
                        } else {
                            "未初始化"
                        });
                    });
                });
            });

            // 右半部分：操作按钮
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());

                ui.group(|ui| {
                    ui.heading("操作");
                    ui.separator();

                    if ui.button("初始化").clicked() {
                        let (tx, rx) = mpsc::channel();

                        match FretDancer::initialize(app, tx.clone()) {
                            Ok(state) => {
                                app.fret_dancer_state = Some(state);
                            }
                            Err(e) => {
                                app.append_console_output(&format!("初始化失败: {}", e));
                                return;
                            }
                        }

                        app.output_receiver = Some(rx);
                        ui.ctx().request_repaint();
                    }

                    if ui.button("生成左手动作").clicked() {
                        let (tx, rx) = mpsc::channel();

                        // 如果state未初始化，则初始化
                        if app.fret_dancer_state.is_none() {
                            match FretDancer::initialize(app, tx.clone()) {
                                Ok(state) => {
                                    app.fret_dancer_state = Some(state);
                                }
                                Err(e) => {
                                    app.append_console_output(&format!("初始化失败: {}", e));
                                    return;
                                }
                            }
                        }

                        let mut app_clone = app.clone();

                        thread::spawn(move || {
                            let _ =
                                FretDancer::generate_left_hand_motion(&mut app_clone, tx.clone());
                        });

                        app.output_receiver = Some(rx);
                        ui.ctx().request_repaint();
                    }

                    if ui.button("生成左手动画数据").clicked() {
                        let (tx, rx) = mpsc::channel();

                        // 如果state未初始化，则初始化
                        if app.fret_dancer_state.is_none() {
                            match FretDancer::initialize(app, tx.clone()) {
                                Ok(state) => {
                                    app.fret_dancer_state = Some(state);
                                }
                                Err(e) => {
                                    app.append_console_output(&format!("初始化失败: {}", e));
                                    return;
                                }
                            }
                        }

                        match FretDancer::generate_left_hand_animation(app) {
                            Ok(file_path) => {
                                app.append_console_output(&format!(
                                    "生成左手动画成功: {}",
                                    file_path
                                ));
                            }
                            Err(e) => {
                                app.append_console_output(&format!("生成左手动画失败: {}", e));
                            }
                        }

                        app.output_receiver = Some(rx);
                        ui.ctx().request_repaint();
                    }

                    if ui.button("生成右手动作和动画数据").clicked() {
                        let (tx, rx) = mpsc::channel();

                        // 如果state未初始化，则初始化
                        if app.fret_dancer_state.is_none() {
                            match FretDancer::initialize(app, tx.clone()) {
                                Ok(state) => {
                                    app.fret_dancer_state = Some(state);
                                }
                                Err(e) => {
                                    app.append_console_output(&format!("初始化失败: {}", e));
                                    return;
                                }
                            }
                        }

                        let mut app_clone = app.clone();

                        thread::spawn(move || {
                            // 在后台线程中执行
                            match FretDancer::generate_right_hand_motion_and_animation(
                                &mut app_clone,
                                tx.clone(),
                            ) {
                                Ok(file_path) => {
                                    app_clone.append_console_output(&format!(
                                        "生成右手动作和动画成功: {}",
                                        file_path
                                    ));
                                }
                                Err(e) => app_clone.append_console_output(&format!(
                                    "生成右手动作和动画失败: {}",
                                    e
                                )),
                            }
                        });

                        app.output_receiver = Some(rx);
                        ui.ctx().request_repaint();
                    }

                    if ui.button("生成弦振动数据").clicked() {
                        let (tx, rx) = mpsc::channel();

                        // 如果state未初始化，则初始化
                        if app.fret_dancer_state.is_none() {
                            match FretDancer::initialize(app, tx.clone()) {
                                Ok(state) => {
                                    app.fret_dancer_state = Some(state);
                                }
                                Err(e) => {
                                    app.append_console_output(&format!("初始化失败: {}", e));
                                    return;
                                }
                            }
                        }

                        let mut app_clone = app.clone();

                        thread::spawn(move || {
                            match execute_generate_string_vibration_data(&mut app_clone, tx.clone())
                            {
                                Ok(file_path) => {
                                    app_clone.append_console_output(&format!(
                                        "生成弦振动数据成功: {}",
                                        file_path
                                    ));
                                }
                                Err(e) => app_clone
                                    .append_console_output(&format!("生成弦振动数据失败: {}", e)),
                            }
                        });

                        app.output_receiver = Some(rx);

                        ui.ctx().request_repaint();
                    }

                    ui.separator();

                    if ui.button("一键生成所有数据").clicked() {
                        let (tx, rx) = mpsc::channel();

                        // 如果state未初始化，则初始化
                        if app.fret_dancer_state.is_none() {
                            match FretDancer::initialize(app, tx.clone()) {
                                Ok(state) => {
                                    app.fret_dancer_state = Some(state);
                                }
                                Err(e) => {
                                    app.append_console_output(&format!("初始化失败: {}", e));
                                    return;
                                }
                            }
                        }

                        let mut app_clone = app.clone();

                        thread::spawn(move || {
                            let _ = FretDancer::main(&mut app_clone, tx.clone());
                        });

                        app.output_receiver = Some(rx);
                        ui.ctx().request_repaint();
                    }

                    if ui.button("重置State").clicked() {
                        app.fret_dancer_state = None;
                        app.console_output = format!("{}\nState已重置\n", app.console_output);
                        ui.ctx().request_repaint();
                    }
                });
            });
        });

        // 处理后台线程的消息
        if let Some(ref rx) = app.output_receiver {
            let mut has_messages = false;
            let messages: Vec<String> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
            for message in messages {
                app.append_console_output(&message);
                has_messages = true;
            }

            if has_messages {
                ui.ctx().request_repaint();
            }
        }

        ui.group(|ui| {
            ui.heading("控制台输出");
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.monospace(&app.console_output);
                });
        });
    });
}

fn execute_generate_string_vibration_data(
    app: &mut FretDanceApp,
    tx: mpsc::Sender<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    // 使用已有的state
    let state = app.fret_dancer_state.as_ref().unwrap();
    let string_file = FretDancer::generate_string_vibration_data(state, tx.clone())?;

    match FretDancer::export_final_report(state, tx.clone()) {
        Ok(()) => {}
        Err(e) => {
            let _ = tx.send(format!("生成最终报告失败: {}", e));
        }
    };

    Ok(format!("弦振动数据已保存至: {}", string_file))
}
