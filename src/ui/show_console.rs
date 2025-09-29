use crate::ui::app::FretDanceApp;
use crate::ui::theme;
use eframe::egui;

pub fn show_console_output(app: &mut FretDanceApp, ui: &mut egui::Ui, size: f32) {
    egui::Frame::group(ui.style())
        .fill(theme::get_midi_info_bg_color(ui)) // 使用主题定义的背景色
        .rounding(4.0)
        .inner_margin(10.0)
        .stroke(egui::Stroke::new(1.0, theme::get_border_color(ui))) // 使用主题定义的边框色
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                // 标题使用更明显的样式
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("控制台信息")
                            .size(size)
                            .color(theme::get_title_color(ui, false)) // 使用主题定义的标题颜色
                            .strong(),
                    ));
                    ui.separator();

                    // 添加清除按钮
                    if ui.button("清空输出").clicked() {
                        app.console_output.clear();
                    }
                });
                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(ui.available_height() - 30.0) // 限制最大高度
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
        });
}
