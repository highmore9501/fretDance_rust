use crate::ui::app::FretDanceApp;
use eframe::egui;
use std::path::Path;

pub fn show_midi_info_scan(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.group(|ui| {
            ui.heading("MIDI参数设置");
            ui.separator();

            // MIDI文件选择
            ui.horizontal(|ui| {
                ui.label("MIDI文件:");
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
                if ui.button("扫描MIDI信息").clicked() {
                    app.scan_midi_info();
                }

                if app.scanning_midi {
                    ui.spinner();
                    ui.label("扫描中...");
                }
            });

            // Track Numbers输入
            ui.horizontal(|ui| {
                ui.label("轨道号:");

                // 下拉菜单选择轨道号 (0-16)
                egui::ComboBox::from_id_source("track_select")
                    .selected_text(format!("轨道 {}", app.selected_track))
                    .show_ui(ui, |ui| {
                        for i in 0..=16 {
                            if ui.selectable_label(false, format!("轨道 {}", i)).clicked() {
                                app.selected_track = i;
                            }
                        }
                    });

                // 添加轨道按钮
                if ui.button("添加").clicked() {
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
                if ui.button("删除").clicked() {
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

            // 显示已选择的轨道号
            ui.horizontal(|ui| {
                ui.label("已选轨道:");
                ui.monospace(&app.track_numbers_str);
            });

            // Channel Number输入
            ui.horizontal(|ui| {
                ui.label("通道号:");
                ui.add(egui::DragValue::new(&mut app.channel_number));
            });

            ui.add_space(10.0);
        });

        ui.group(|ui| {
            ui.heading("MIDI信息");

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.monospace(&app.midi_info_result);
                });
        });
    });
}
