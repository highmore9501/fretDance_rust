use crate::ui::app::FretDanceApp;
use eframe::egui;
use std::path::Path;

/// 显示Avatar信息
pub fn show_avatar_info(app: &mut FretDanceApp, ui: &mut egui::Ui) {
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
            let button_width = (ui.available_width() - 30.0) / 3.0; // 计算每个按钮的宽度，留出间距

            let widget_modify_button = egui::Button::new("修改")
                .fill(
                    egui::Color32::from_hex("#849204ff")
                        .unwrap_or(egui::Color32::from_rgb(249, 91, 137)),
                )
                .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE));

            let widget_new_button = egui::Button::new("新增")
                .fill(
                    egui::Color32::from_hex("#0fc508ff")
                        .unwrap_or(egui::Color32::from_rgb(249, 91, 137)),
                ) // 蓝色背景
                .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE));

            let widget_delete_button = egui::Button::new("删除")
                .fill(
                    egui::Color32::from_hex("#c4063cff")
                        .unwrap_or(egui::Color32::from_rgb(249, 91, 137)),
                ) // 蓝色背景
                .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE));

            if ui
                .add_sized([button_width, 20.0], widget_modify_button)
                .clicked()
            {
                // TODO: 实现修改功能
                println!("修改avatar: {}", app.avatar);
            }

            ui.add_space(10.0);

            if ui
                .add_sized([button_width, 20.0], widget_new_button)
                .clicked()
            {
                // TODO: 实现新增功能
                println!("新增avatar");
            }

            ui.add_space(10.0);

            if ui
                .add_sized([button_width, 20.0], widget_delete_button)
                .clicked()
            {
                // 设置删除确认标志，实际的删除操作将在确认对话框中处理
                app.show_delete_confirmation = true;
            }

            ui.add_space(10.0);
        });

        // 显示删除确认对话框
        if app.show_delete_confirmation {
            egui::Window::new("确认删除")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    let avatar_name = app.avatar.clone();
                    ui.label(format!("确定要删除Avatar \"{}\" 吗？", avatar_name));
                    ui.label("此操作不可撤销。");
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            app.show_delete_confirmation = false;
                        }
                        if ui.button("确认删除").clicked() {
                            match app.delete_avatar(&avatar_name) {
                                Ok(()) => {
                                    app.console_output
                                        .push_str(&format!("成功删除Avatar: {}\n", avatar_name));
                                }
                                Err(e) => {
                                    app.console_output
                                        .push_str(&format!("删除Avatar失败: {}\n", e));
                                }
                            }
                            app.show_delete_confirmation = false;
                        }
                    });
                });
        }
    });
}

/// 显示Avatar图片
fn show_avatar_image(ui: &mut egui::Ui, image_path: &str) {
    // 检查文件是否存在
    if std::path::Path::new(image_path).exists() {
        // 使用egui的图像功能显示图片
        let max_size = egui::Vec2::new(350.0, 300.0);
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
