use crate::ui::app::FretDanceApp;
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

                ui.group(|ui| {
                    ui.heading("Avatar信息");
                    ui.separator();

                    // 显示avatar图片
                    ui.vertical_centered(|ui| {
                        // 尝试显示avatar图片，如果没有则显示默认图片
                        let image_path = if let Some(ref avatar_info) = app.current_avatar_info {
                            if avatar_info.image != "default.png" && !avatar_info.image.is_empty() {
                                format!("asset/img/{}", avatar_info.image)
                            } else {
                                "asset/img/default.png".to_string()
                            }
                        } else {
                            "asset/img/default.png".to_string()
                        };

                        // 加载并显示图片
                        show_avatar_image(ui, &image_path);
                    });

                    ui.add_space(10.0);

                    // 显示avatar使用的乐器
                    if let Some(ref avatar_info) = app.current_avatar_info {
                        ui.horizontal(|ui| {
                            ui.label("乐器:");
                            ui.colored_label(
                                egui::Color32::from_rgb(100, 200, 100),
                                &avatar_info.instrument,
                            );
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("乐器:");
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "未知");
                        });
                    }

                    ui.add_space(20.0);

                    // 按钮行
                    ui.horizontal(|ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                if ui.button("修改").clicked() {
                                    // TODO: 实现修改功能
                                    println!("修改avatar: {}", app.avatar);
                                }

                                ui.add_space(10.0);

                                if ui.button("新增").clicked() {
                                    // TODO: 实现新增功能
                                    println!("新增avatar");
                                }

                                ui.add_space(10.0);

                                if ui.button("删除").clicked() {
                                    // TODO: 实现删除功能
                                    println!("删除avatar: {}", app.avatar);
                                }
                            },
                        );
                    });
                });
            });
        });
    });
}

/// 显示Avatar图片
fn show_avatar_image(ui: &mut egui::Ui, image_path: &str) {
    // 检查文件是否存在
    if std::path::Path::new(image_path).exists() {
        // 使用egui的图像功能显示图片
        let max_size = egui::Vec2::new(350.0, 400.0);
        // 使用uri方式加载图片
        ui.add_sized(
            max_size,
            egui::Image::from_uri(
                format!(
                    "file://{}",
                    std::fs::canonicalize(image_path)
                        .unwrap_or(image_path.into())
                        .display()
                )
                .as_str(),
            ),
        );
    } else {
        println!("Image file does not exist: {}", image_path); // 添加调试信息
        // 文件不存在，显示占位符
        show_placeholder_image(ui);
    }
}

/// 显示占位符图像
fn show_placeholder_image(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_gray(30))
        .inner_margin(10.0)
        .rounding(5.0)
        .show(ui, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(egui::Vec2::new(150.0, 150.0), egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(5.0),
                egui::Color32::from_gray(50),
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "无图片",
                egui::FontId::default(),
                egui::Color32::WHITE,
            );
        });
}
