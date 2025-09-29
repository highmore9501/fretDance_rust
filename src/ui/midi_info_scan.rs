use crate::ui::app::FretDanceApp;
use crate::ui::theme;
use eframe::egui;
use std::path::Path;

pub fn show_midi_info_scan(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // MIDI参数设置部分
        egui::Frame::group(ui.style())
            .fill(theme::get_midi_param_bg_color(ui)) // 使用主题定义的背景色
            .rounding(4.0)
            .inner_margin(10.0)
            .stroke(egui::Stroke::new(1.0, theme::get_border_color(ui))) // 使用主题定义的边框色
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    // 标题使用更明显的样式
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("MIDI参数设置")
                                .size(20.0)
                                .color(theme::get_title_color(ui, true)) // 使用主题定义的标题颜色
                                .strong(),
                        ));
                        ui.separator();
                    });
                    ui.add_space(5.0);

                    // MIDI文件选择
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("MIDI文件:")
                                .color(theme::get_label_color(ui)) // 使用主题定义的标签颜色
                                .size(16.0),
                        ));

                        egui::ComboBox::from_id_source("midi_scan_select")
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

                        if ui
                            .add_sized([80.0, 24.0], egui::Button::new("扫描MIDI信息"))
                            .clicked()
                        {
                            app.scan_midi_info();
                        }

                        if app.scanning_midi {
                            ui.spinner();
                            ui.add(egui::Label::new(
                                egui::RichText::new("扫描中...").size(12.0),
                            ));
                        }
                    });

                    ui.add_space(5.0);

                    // Track Numbers输入
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("轨道号:")
                                .color(theme::get_label_color(ui)) // 使用主题定义的标签颜色
                                .size(16.0),
                        ));

                        // 下拉菜单选择轨道号 (0-16)
                        egui::ComboBox::from_id_source("track_select")
                            .selected_text(format!("轨道 {}", app.selected_track))
                            .show_ui(ui, |ui| {
                                for i in 0..=16 {
                                    if ui.selectable_label(false, format!("轨道 {}", i)).clicked()
                                    {
                                        app.selected_track = i;
                                    }
                                }
                            });

                        // 添加轨道按钮
                        if ui
                            .add_sized([50.0, 24.0], egui::Button::new("添加"))
                            .clicked()
                        {
                            // 解析当前轨道号列表
                            let mut tracks: Vec<i32> = app
                                .track_numbers_str
                                .split(',')
                                .filter_map(|s| s.trim().parse().ok())
                                .collect();

                            // 如果新轨道号不在列表中，则添加
                            if !tracks.contains(&app.selected_track) {
                                tracks.push(app.selected_track);
                                tracks.sort(); // 排序轨道号
                                app.track_numbers_str = tracks
                                    .iter()
                                    .map(|n| n.to_string())
                                    .collect::<Vec<String>>()
                                    .join(",");
                            }
                        }

                        // 删除轨道按钮
                        if ui
                            .add_sized([50.0, 24.0], egui::Button::new("删除"))
                            .clicked()
                        {
                            // 解析当前轨道号列表
                            let mut tracks: Vec<i32> = app
                                .track_numbers_str
                                .split(',')
                                .filter_map(|s| s.trim().parse().ok())
                                .collect();

                            // 删除选中的轨道号
                            tracks.retain(|&x| x != app.selected_track);
                            tracks.sort(); // 排序轨道号
                            app.track_numbers_str = tracks
                                .iter()
                                .map(|n| n.to_string())
                                .collect::<Vec<String>>()
                                .join(",");
                        }
                    });

                    ui.add_space(5.0);

                    // 显示已选择的轨道号
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("已选轨道:")
                                .color(theme::get_label_color(ui)) // 使用主题定义的标签颜色
                                .size(16.0),
                        ));
                        ui.add(egui::Label::new(
                            egui::RichText::new(&app.track_numbers_str).size(12.0),
                        ));
                    });

                    ui.add_space(5.0);

                    // Channel Number输入
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("通道号:")
                                .color(theme::get_label_color(ui)) // 使用主题定义的标签颜色
                                .size(16.0),
                        ));
                        ui.add(egui::DragValue::new(&mut app.channel_number).speed(1.0));
                    });
                });
            });

        ui.add_space(10.0);

        // MIDI信息部分
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
                            egui::RichText::new("MIDI信息")
                                .size(20.0)
                                .color(theme::get_title_color(ui, false)) // 使用主题定义的标题颜色
                                .strong(),
                        ));
                        ui.separator();
                    });
                    ui.add_space(5.0);

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(&app.midi_info_result)
                                    .size(14.0)
                                    .monospace(),
                            ));
                        });
                });
            });
    });
}
