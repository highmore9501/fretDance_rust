use crate::fret_dancer::{FretDancer, FretDancerState};
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
                        match execute_initialization(app) {
                            Ok(output) => {
                                app.console_output = format!("初始化成功:\n{}\n", output);
                            }
                            Err(e) => {
                                app.console_output = format!("初始化失败: {}\n", e);
                            }
                        }
                        ui.ctx().request_repaint();
                    }

                    if ui.button("生成左手动作").clicked() {
                        // 创建通道用于通信
                        let (tx, rx) = mpsc::channel::<String>();

                        // 克隆需要的数据
                        let use_harm_notes = app.use_harm_notes;

                        // 如果state未初始化，则初始化
                        if app.fret_dancer_state.is_none() {
                            match initialize_state_if_needed(app) {
                                Ok(state) => {
                                    app.fret_dancer_state = Some(state);
                                }
                                Err(e) => {
                                    app.append_console_output(&format!("初始化失败: {}", e));
                                    ui.ctx().request_repaint();
                                    return;
                                }
                            }
                        }

                        // 克隆state用于线程
                        let state = app.fret_dancer_state.as_ref().unwrap().clone();

                        // 在后台线程中执行
                        thread::spawn(move || {
                            match FretDancer::generate_left_hand_motion(
                                &state,
                                use_harm_notes,
                                |message: &str| {
                                    let _ = tx.send(message.to_string());
                                },
                            ) {
                                Ok(_) => {
                                    let _ = tx.send("左手动作生成完成".to_string());
                                }
                                Err(e) => {
                                    let _ = tx.send(format!("生成失败: {}", e));
                                }
                            }
                        });

                        // 保存接收端用于后续处理
                        app.output_receiver = Some(rx);
                    }

                    if ui.button("生成左手动画数据").clicked() {
                        match execute_generate_left_hand_animation(app) {
                            Ok(output) => {
                                app.console_output = format!(
                                    "{}\n生成左手动画数据成功:\n{}\n",
                                    app.console_output, output
                                );
                            }
                            Err(e) => {
                                app.console_output = format!(
                                    "{}\n生成左手动画数据失败: {}\n",
                                    app.console_output, e
                                );
                            }
                        }
                        ui.ctx().request_repaint();
                    }

                    if ui.button("生成右手动作和动画数据").clicked() {
                        // 创建通道用于通信
                        let (tx, rx) = mpsc::channel::<String>();

                        // 如果state未初始化，则初始化
                        if app.fret_dancer_state.is_none() {
                            match initialize_state_if_needed(app) {
                                Ok(state) => {
                                    app.fret_dancer_state = Some(state);
                                }
                                Err(e) => {
                                    app.append_console_output(&format!("初始化失败: {}", e));
                                    ui.ctx().request_repaint();
                                    return;
                                }
                            }
                        }

                        // 克隆state用于线程
                        let state = app.fret_dancer_state.as_ref().unwrap().clone();

                        // 在后台线程中执行
                        thread::spawn(move || {
                            match FretDancer::generate_right_hand_motion_and_animation(
                                &state,
                                |message: &str| {
                                    let _ = tx.send(message.to_string());
                                },
                            ) {
                                Ok(animation_file) => {
                                    let _ = tx.send(format!(
                                        "右手动作和动画数据已保存至: {}",
                                        animation_file
                                    ));
                                    let _ = tx.send("右手动作和动画生成完成".to_string());
                                }
                                Err(e) => {
                                    let _ = tx.send(format!("生成失败: {}", e));
                                }
                            }
                        });

                        // 保存接收端用于后续处理
                        app.output_receiver = Some(rx);
                    }

                    if ui.button("生成弦振动数据").clicked() {
                        match execute_generate_string_vibration_data(app) {
                            Ok(output) => {
                                app.console_output = format!(
                                    "{}\n生成弦振动数据成功:\n{}\n",
                                    app.console_output, output
                                );
                            }
                            Err(e) => {
                                app.console_output =
                                    format!("{}\n生成弦振动数据失败: {}\n", app.console_output, e);
                            }
                        }
                        ui.ctx().request_repaint();
                    }

                    ui.separator();

                    if ui.button("一键生成所有数据").clicked() {
                        // 一键生成时重置state以确保使用最新参数
                        app.fret_dancer_state = None;
                        match execute_all_operations(app) {
                            Ok(output) => {
                                app.console_output =
                                    format!("{}\n{}\n", app.console_output, output);
                            }
                            Err(e) => {
                                app.console_output = format!(
                                    "{}\n一键生成所有数据失败: {}\n",
                                    app.console_output, e
                                );
                            }
                        }
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
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.monospace(&app.console_output);
                });
        });
    });
}
fn initialize_state_if_needed(
    app: &mut FretDanceApp,
) -> Result<FretDancerState, Box<dyn std::error::Error>> {
    let state = FretDancer::initialize(app)?;
    Ok(state)
}
fn execute_initialization(app: &mut FretDanceApp) -> Result<String, Box<dyn std::error::Error>> {
    let state = initialize_state_if_needed(app)?;
    let filename = state.filename.clone();
    app.fret_dancer_state = Some(state);
    Ok(format!("初始化完成，生成的文件名前缀: {}", filename))
}
fn execute_generate_left_hand_animation(
    app: &mut FretDanceApp,
) -> Result<String, Box<dyn std::error::Error>> {
    // 如果state未初始化，则初始化
    if app.fret_dancer_state.is_none() {
        app.fret_dancer_state = Some(initialize_state_if_needed(app)?);
    }

    // 使用已有的state
    let state = app.fret_dancer_state.as_ref().unwrap();
    let animation_file = FretDancer::generate_left_hand_animation(state)?;

    Ok(format!("左手动画数据已保存至: {}", animation_file))
}

fn execute_generate_string_vibration_data(
    app: &mut FretDanceApp,
) -> Result<String, Box<dyn std::error::Error>> {
    // 如果state未初始化，则初始化
    if app.fret_dancer_state.is_none() {
        app.fret_dancer_state = Some(initialize_state_if_needed(app)?);
    }

    let console_output = &mut app.console_output;

    // 使用已有的state
    let state = app.fret_dancer_state.as_ref().unwrap();
    let string_file = FretDancer::generate_string_vibration_data(state, console_output)?;

    Ok(format!("弦振动数据已保存至: {}", string_file))
}

fn execute_all_operations(app: &mut FretDanceApp) -> Result<String, Box<dyn std::error::Error>> {
    // 创建通道用于通信
    let (tx, rx) = mpsc::channel::<String>();

    // 克隆需要的数据
    let midi_file_path = app.midi_file_path.clone();
    let track_numbers_str = app.track_numbers_str.clone();
    let channel_number = app.channel_number;
    let fps = app.fps;
    let guitar_string_notes = app.guitar_string_notes.clone();
    let octave_down_checkbox = app.octave_down_checkbox;
    let capo_number = app.capo_number;
    let use_harm_notes = app.use_harm_notes;
    let disable_barre = app.disable_barre;
    let current_avatar_info = app.current_avatar_info.clone();

    // 在后台线程中执行
    thread::spawn(move || {
        // 创建一个新的App实例用于后台任务
        let mut background_app = FretDanceApp {
            avatar: String::new(),
            midi_file_path,
            track_numbers_str,
            selected_track: 1,
            channel_number,
            fps,
            guitar_string_notes,
            octave_down_checkbox,
            capo_number,
            use_harm_notes,
            disable_barre,
            tuning_presets: vec![],
            avatar_options: vec![],
            midi_options: vec![],
            console_output: String::new(),
            avatar_infos: vec![],
            current_avatar_info,
            show_delete_confirmation: false,
            show_edit_avatar_dialog: false,
            edit_avatar_name: String::new(),
            edit_avatar_image: String::new(),
            edit_avatar_selected_image_path: String::new(),
            edit_avatar_json: String::new(),
            edit_avatar_selected_json_path: String::new(),
            edit_avatar_instrument: InstrumentType::FingerStyleGuitar,
            edit_avatar_mode: EditAvatarMode::New,
            dark_mode: true,
            midi_info_result: String::new(),
            scanning_midi: false,
            current_tab: Tab::ExecuteOperation,
            fret_dancer_state: None,
            output_receiver: None,
            is_processing: false,
        };

        match FretDancer::main(&mut background_app, |message: &str| {
            let _ = tx.send(message.to_string());
        }) {
            Ok(_) => {
                let output_lines: Vec<&str> = background_app.console_output.lines().collect();
                for line in output_lines {
                    let _ = tx.send(line.to_string());
                }
                let _ = tx.send("一键生成所有数据完成".to_string());
            }
            Err(e) => {
                let _ = tx.send(format!("执行失败: {}", e));
            }
        }
    });

    // 保存接收端用于后续处理
    app.output_receiver = Some(rx);

    // 返回一个临时消息，实际结果将通过通道传递
    Ok("开始执行一键生成所有数据...".to_string())
}
