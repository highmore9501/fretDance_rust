use crate::ui::app::FretDanceApp;
use crate::ui::avatar_display;
use crate::ui::theme;
use eframe::egui;

pub fn show_parameter_setting(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    // 创建一个滚动区域以容纳所有控件
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            // 左半部分：原有的参数设置
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() * 0.5);

                // 第一部分：用户输入参数
                egui::Frame::group(ui.style())
                    .rounding(6.0) // 添加圆角
                    .inner_margin(egui::Margin::same(10.0)) // 增加内边距
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new("角色参数")
                                    .size(20.0)
                                    .color(theme::get_title_color(ui, true)) // 使用主题定义的标题颜色
                                    .strong(),
                            ));
                            ui.separator();

                            // Avatar选择
                            ui.horizontal(|ui| {
                                ui.label("Avatar:");
                                let response = egui::ComboBox::from_id_source("avatar_select")
                                    .selected_text(&app.avatar)
                                    .show_ui(ui, |ui| {
                                        for option in &app.avatar_options {
                                            ui.selectable_value(
                                                &mut app.avatar,
                                                option.clone(),
                                                option,
                                            );
                                        }
                                    });

                                // 当avatar选择改变时更新avatar信息
                                if let Some(_) = response.inner {
                                    app.update_current_avatar_info();

                                    ui.ctx().request_repaint(); // 请求立即重绘
                                }
                            });

                            // FPS输入
                            ui.horizontal(|ui| {
                                ui.label("FPS:");
                                ui.add(egui::DragValue::new(&mut app.fps).speed(1.0));
                            });

                            // 吉他弦音高设置
                            ui.label("吉他弦音高 (从最细的弦到最粗的弦):");
                            // 添加预设调弦下拉菜单
                            ui.horizontal(|ui| {
                                ui.label("常用调弦:");
                                egui::ComboBox::from_id_source("preset_tuning")
                                    .selected_text("选择预设调弦")
                                    .show_ui(ui, |ui| {
                                        for preset in &app.tuning_presets {
                                            if ui.selectable_label(false, &preset.name).clicked() {
                                                app.guitar_string_notes = preset.notes.clone();
                                            }
                                        }
                                    });

                                // 添加增减弦数按钮，使用更好的样式
                                ui.separator();
                                if ui.add(egui::Button::new(" ➕ 增加弦数")).clicked() {
                                    app.guitar_string_notes.push("".to_string());
                                }

                                if ui.add(egui::Button::new(" ➖ 减少弦数")).clicked()
                                    && app.guitar_string_notes.len() > 1
                                {
                                    app.guitar_string_notes.pop();
                                }
                            });

                            // 动态生成弦数和音高设置
                            for (i, note) in app.guitar_string_notes.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("第{}弦", i + 1));
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

                            // Use harm notes checkbox
                            ui.checkbox(&mut app.disable_barre, "禁用横按");
                        });
                    });

                ui.add_space(10.0);
            });

            // 右半部分：avatar信息显示或编辑界面
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                if app.show_edit_avatar_dialog {
                    avatar_display::show_edit_avatar_interface(app, ui);
                } else {
                    avatar_display::show_avatar_info(app, ui);
                }
            });
        });

        // 控制台输出部分
        egui::Frame::group(ui.style())
            .rounding(6.0) // 添加圆角
            .inner_margin(egui::Margin::same(10.0)) // 增加内边距
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("控制台输出")
                            .size(20.0)
                            .color(theme::get_title_color(ui, false)) // 使用主题定义的标题颜色
                            .strong(),
                    ));
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_height(150.0) // 限制最大高度
                        .show(ui, |ui| {
                            // 使用等宽字体显示控制台输出，并添加颜色区分
                            let output_lines: Vec<&str> = app.console_output.lines().collect();
                            for line in output_lines {
                                if line.contains("成功") || line.contains("完成") {
                                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), line); // 绿色表示成功
                                } else if line.contains("失败") || line.contains("错误") {
                                    ui.colored_label(egui::Color32::from_rgb(200, 100, 100), line); // 红色表示错误
                                } else if line.contains("警告") {
                                    ui.colored_label(egui::Color32::from_rgb(200, 200, 100), line); // 黄色表示警告
                                } else {
                                    ui.monospace(line);
                                }
                            }
                        });
                });

                // 强制在下一帧重绘
                ui.ctx().request_repaint();
            });
    });
}
