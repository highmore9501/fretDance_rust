use crate::ui::app::FretDanceApp;
use eframe::egui;
use std::path::Path;

pub fn show_midi_info_scan(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.group(|ui| {
            ui.heading("MIDI信息扫描");
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
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("扫描MIDI信息").clicked() {
                    app.scan_midi_info();
                }

                if app.scanning_midi {
                    ui.spinner();
                    ui.label("扫描中...");
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.monospace(&app.midi_info_result);
                });
        });
    });
}
