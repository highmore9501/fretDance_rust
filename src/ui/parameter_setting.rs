use crate::ui::app::FretDanceApp;
use crate::ui::avatar_display;
use eframe::egui;
use std::path::Path;

pub fn show_parameter_setting(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    // 创建一个滚动区域以容纳所有控件
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            // 左半部分：原有的参数设置
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() * 0.5);

                // 第一部分：用户输入参数
                ui.group(|ui| {
                    ui.heading("1. 参数设置");
                    ui.separator();

                    // Avatar选择
                    ui.horizontal(|ui| {
                        ui.label("Avatar:");
                        let response = egui::ComboBox::from_id_source("avatar_select")
                            .selected_text(&app.avatar)
                            .show_ui(ui, |ui| {
                                for option in &app.avatar_options {
                                    ui.selectable_value(&mut app.avatar, option.clone(), option);
                                }
                            });

                        // 当avatar选择改变时更新avatar信息
                        if let Some(_) = response.inner {
                            app.update_current_avatar_info();

                            ui.ctx().request_repaint(); // 请求立即重绘
                        }
                    });

                    // MIDI文件选择
                    ui.horizontal(|ui| {
                        ui.label("MIDI文件:");
                        egui::ComboBox::from_id_source("midi_select")
                            .selected_text({
                                // 显示文件名而不是完整路径
                                Path::new(&app.midi_file_path)
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(&app.midi_file_path)
                                    .to_string()
                            })
                            .show_ui(ui, |ui| {
                                for option in &app.midi_options {
                                    let full_path = format!("asset/midi/{}", option);
                                    ui.selectable_value(&mut app.midi_file_path, full_path, option);
                                }
                            });
                    });

                    // Track Numbers输入
                    ui.horizontal(|ui| {
                        ui.label("轨道号 (逗号分隔):");
                        ui.text_edit_singleline(&mut app.track_numbers_str);
                    });

                    // Channel Number输入
                    ui.horizontal(|ui| {
                        ui.label("通道号:");
                        ui.add(egui::DragValue::new(&mut app.channel_number));
                    });

                    // FPS输入
                    ui.horizontal(|ui| {
                        ui.label("FPS:");
                        ui.add(egui::DragValue::new(&mut app.fps).speed(1.0));
                    });

                    // 吉他弦音高设置
                    ui.label("吉他弦音高 (从最细的弦到最粗的弦):");
                    for (i, note) in app.guitar_string_notes.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            let string_names = ["1弦", "2弦", "3弦", "4弦", "5弦", "6弦"];
                            ui.label(string_names[i]);
                            ui.text_edit_singleline(note);
                        });
                    }

                    // Octave down checkbox
                    ui.checkbox(&mut app.octave_down_checkbox, "降低八度");

                    // Capo number输入
                    ui.horizontal(|ui| {
                        ui.label("变调夹位置:");
                        ui.add(egui::DragValue::new(&mut app.capo_number).range(0..=12));
                    });

                    // Use harm notes checkbox
                    ui.checkbox(&mut app.use_harm_notes, "使用泛音");
                });

                ui.add_space(10.0);

                // 控制台输出部分
                ui.group(|ui| {
                    ui.heading("2. 控制台输出");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.monospace(&app.console_output);
                        });
                });
            });

            // 右半部分：avatar信息显示
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                avatar_display::show_avatar_info(app, ui);
            });
        });
    });
}
