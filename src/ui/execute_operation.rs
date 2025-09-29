use crate::fret_dancer::FretDancer;
use crate::ui::app::FretDanceApp;
use crate::ui::theme;
use eframe::egui;
use std::fmt::format;
use std::sync::mpsc;
use std::thread;

pub fn show_execute_operation(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            // 左半部分：显示所有设置参数和操作按钮
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() * 0.5);

                // 改进角色参数部分的样式
                egui::Frame::group(ui.style())
                    .fill(theme::get_midi_param_bg_color(ui)) // 深色背景
                    .rounding(4.0)
                    .inner_margin(10.0)
                    .stroke(egui::Stroke::new(1.0, theme::get_border_color(ui)))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.vertical(|ui| {
                            // 标题使用更明显的样式
                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new(format!("角色: {}", &app.avatar))
                                        .size(14.0)
                                        .color(theme::get_title_color(ui, true))
                                        .strong(),
                                ));
                                ui.separator();
                            });
                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("使用乐器:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                let instrument = app
                                    .current_avatar_info
                                    .as_ref()
                                    .map(|info| info.instrument.as_str())
                                    .unwrap_or("未知");
                                ui.add(egui::Label::new(
                                    egui::RichText::new(instrument).size(12.0),
                                ));
                            });

                            ui.add_space(3.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("FPS:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                ui.add(egui::Label::new(
                                    egui::RichText::new(app.fps.to_string()).size(12.0),
                                ));
                            });

                            ui.add_space(3.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("吉他弦音高:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                ui.add(egui::Label::new(
                                    egui::RichText::new(format!("{:?}", app.guitar_string_notes))
                                        .size(12.0),
                                ));
                            });

                            ui.add_space(3.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("降低八度:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                ui.add(egui::Label::new(
                                    egui::RichText::new(if app.octave_down_checkbox {
                                        "是"
                                    } else {
                                        "否"
                                    })
                                    .size(12.0),
                                ));
                            });

                            ui.add_space(3.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("变调夹位置:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                ui.add(egui::Label::new(
                                    egui::RichText::new(app.capo_number.to_string()).size(12.0),
                                ));
                            });

                            ui.add_space(3.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("使用泛音:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                ui.add(egui::Label::new(
                                    egui::RichText::new(if app.use_harm_notes {
                                        "是"
                                    } else {
                                        "否"
                                    })
                                    .size(12.0),
                                ));
                            });

                            ui.add_space(3.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("禁用横按:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                ui.add(egui::Label::new(
                                    egui::RichText::new(if app.disable_barre {
                                        "是"
                                    } else {
                                        "否"
                                    })
                                    .size(12.0),
                                ));
                            });
                        });
                    });

                ui.add_space(10.0);

                // 改进MIDI参数部分的样式
                egui::Frame::group(ui.style())
                    .fill(theme::get_midi_info_bg_color(ui)) // 深色背景
                    .rounding(4.0)
                    .inner_margin(10.0)
                    .stroke(egui::Stroke::new(1.0, theme::get_border_color(ui)))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical(|ui| {
                            // 显示MIDI文件名而不是完整路径
                            let midi_filename = std::path::Path::new(&app.midi_file_path)
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or(&app.midi_file_path);

                            // 标题使用更明显的样式
                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new(format!("MIDI: {}", midi_filename))
                                        .size(14.0)
                                        .color(theme::get_title_color(ui, true))
                                        .strong(),
                                ));
                                ui.separator();
                            });
                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("轨道号:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                ui.add(egui::Label::new(
                                    egui::RichText::new(&app.track_numbers_str).size(12.0),
                                ));
                            });

                            ui.add_space(3.0);

                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("通道号:")
                                        .color(theme::get_label_color(ui))
                                        .size(12.0),
                                ));
                                ui.add(egui::Label::new(
                                    egui::RichText::new(app.channel_number.to_string()).size(12.0),
                                ));
                            });
                        });
                    });

                ui.add_space(10.0);

                // 改进操作部分的样式
                egui::Frame::group(ui.style())
                    .fill(theme::get_midi_info_bg_color(ui)) // 绿色调背景
                    .rounding(4.0)
                    .inner_margin(10.0)
                    .stroke(egui::Stroke::new(1.0, theme::get_border_color(ui)))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            // 标题使用更明显的样式
                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("操作")
                                        .size(14.0)
                                        .color(theme::get_title_color(ui, false))
                                        .strong(),
                                ));
                                ui.separator();
                            });
                            ui.add_space(5.0);

                            if ui
                                .add_sized(
                                    [ui.available_width(), 24.0],
                                    egui::Button::new("1.初始化"),
                                )
                                .clicked()
                            {
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

                            ui.add_space(3.0);

                            if ui
                                .add_sized(
                                    [ui.available_width(), 24.0],
                                    egui::Button::new("2.生成左手动作"),
                                )
                                .clicked()
                            {
                                let (tx, rx) = mpsc::channel();

                                // 如果state未初始化，则初始化
                                if app.fret_dancer_state.is_none() {
                                    match FretDancer::initialize(app, tx.clone()) {
                                        Ok(state) => {
                                            app.fret_dancer_state = Some(state);
                                        }
                                        Err(e) => {
                                            app.append_console_output(&format!(
                                                "初始化失败: {}",
                                                e
                                            ));
                                            return;
                                        }
                                    }
                                }

                                let mut app_clone = app.clone();

                                thread::spawn(move || {
                                    let _ = FretDancer::generate_left_hand_motion(
                                        &mut app_clone,
                                        tx.clone(),
                                    );
                                });

                                app.output_receiver = Some(rx);
                                ui.ctx().request_repaint();
                            }

                            ui.add_space(3.0);

                            if ui
                                .add_sized(
                                    [ui.available_width(), 24.0],
                                    egui::Button::new("3.生成左手动画数据"),
                                )
                                .clicked()
                            {
                                let (tx, rx) = mpsc::channel();

                                // 如果state未初始化，则初始化
                                if app.fret_dancer_state.is_none() {
                                    match FretDancer::initialize(app, tx.clone()) {
                                        Ok(state) => {
                                            app.fret_dancer_state = Some(state);
                                        }
                                        Err(e) => {
                                            app.append_console_output(&format!(
                                                "初始化失败: {}",
                                                e
                                            ));
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
                                        app.append_console_output(&format!(
                                            "生成左手动画失败: {}",
                                            e
                                        ));
                                    }
                                }

                                app.output_receiver = Some(rx);
                                ui.ctx().request_repaint();
                            }

                            ui.add_space(3.0);

                            if ui
                                .add_sized(
                                    [ui.available_width(), 24.0],
                                    egui::Button::new("4.生成右手动作和动画数据"),
                                )
                                .clicked()
                            {
                                let (tx, rx) = mpsc::channel();

                                // 如果state未初始化，则初始化
                                if app.fret_dancer_state.is_none() {
                                    match FretDancer::initialize(app, tx.clone()) {
                                        Ok(state) => {
                                            app.fret_dancer_state = Some(state);
                                        }
                                        Err(e) => {
                                            app.append_console_output(&format!(
                                                "初始化失败: {}",
                                                e
                                            ));
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

                            ui.add_space(3.0);

                            if ui
                                .add_sized(
                                    [ui.available_width(), 24.0],
                                    egui::Button::new("5.生成弦振动数据"),
                                )
                                .clicked()
                            {
                                let (tx, rx) = mpsc::channel();

                                // 如果state未初始化，则初始化
                                if app.fret_dancer_state.is_none() {
                                    match FretDancer::initialize(app, tx.clone()) {
                                        Ok(state) => {
                                            app.fret_dancer_state = Some(state);
                                        }
                                        Err(e) => {
                                            app.append_console_output(&format!(
                                                "初始化失败: {}",
                                                e
                                            ));
                                            return;
                                        }
                                    }
                                }

                                let mut app_clone = app.clone();

                                thread::spawn(
                                    move || match execute_generate_string_vibration_data(
                                        &mut app_clone,
                                        tx.clone(),
                                    ) {
                                        Ok(file_path) => {
                                            app_clone.append_console_output(&format!(
                                                "生成弦振动数据成功: {}",
                                                file_path
                                            ));
                                        }
                                        Err(e) => app_clone.append_console_output(&format!(
                                            "生成弦振动数据失败: {}",
                                            e
                                        )),
                                    },
                                );

                                app.output_receiver = Some(rx);

                                ui.ctx().request_repaint();
                            }

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(3.0);

                            if ui
                                .add_sized(
                                    [ui.available_width(), 30.0],
                                    egui::Button::new("一键生成所有数据"),
                                )
                                .clicked()
                            {
                                let (tx, rx) = mpsc::channel();

                                let mut app_clone = app.clone();

                                thread::spawn(move || {
                                    let _ = FretDancer::main(&mut app_clone, tx.clone());
                                });

                                app.output_receiver = Some(rx);
                                ui.ctx().request_repaint();
                            }
                        });
                    });
            });

            // 右半部分：控制台输出
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());

                egui::Frame::group(ui.style())
                    .fill(theme::get_midi_info_bg_color(ui)) // 使用主题定义的背景色
                    .rounding(4.0)
                    .inner_margin(10.0)
                    .stroke(egui::Stroke::new(1.0, theme::get_border_color(ui))) // 使用主题定义的边框色
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            // 标题使用更明显的样式
                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("控制台输出")
                                        .size(14.0)
                                        .color(theme::get_title_color(ui, false)) // 使用主题定义的标题颜色
                                        .strong(),
                                ));
                                ui.separator();
                            });
                            ui.add_space(5.0);

                            egui::ScrollArea::vertical()
                                .auto_shrink(false)
                                .max_height(ui.available_height() - 30.0)
                                .show(ui, |ui| {
                                    ui.add(egui::Label::new(
                                        egui::RichText::new(&app.console_output)
                                            .size(14.0)
                                            .monospace(),
                                    ));
                                });
                        });
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
