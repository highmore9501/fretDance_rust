use crate::ui::app::{EditAvatarMode, FretDanceApp, InstrumentType};
use crate::utils::compare_json::compare_json_structure;
use eframe::egui;

/// 显示Avatar信息
pub fn show_avatar_info(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    egui::Frame::group(ui.style())
        .rounding(6.0) // 添加圆角
        .inner_margin(egui::Margin::same(10.0)) // 增加内边距
        .show(ui, |ui| {
            ui.vertical(|ui| {
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

                // 按钮行 - 使用更好的样式
                ui.horizontal(|ui| {
                    let button_width = (ui.available_width() - 30.0) / 3.0; // 计算每个按钮的宽度，留出间距

                    let modify_button = egui::Button::new("✏ 修改")
                        .fill(egui::Color32::from_rgb(138, 43, 226)) // 主色调 - 蓝紫色
                        .rounding(4.0); // 圆角

                    let new_button = egui::Button::new("➕ 新增")
                        .fill(egui::Color32::from_rgb(0, 206, 209)) // 青蓝色
                        .rounding(4.0); // 圆角

                    let delete_button = egui::Button::new("🗑 删除")
                        .fill(egui::Color32::from_rgb(200, 50, 50)) // 红色
                        .rounding(4.0); // 圆角

                    if ui.add_sized([button_width, 30.0], modify_button).clicked() {
                        // 准备修改数据
                        prepare_edit_avatar(app, EditAvatarMode::Edit);
                    }

                    ui.add_space(10.0);

                    if ui.add_sized([button_width, 30.0], new_button).clicked() {
                        // 准备新增数据
                        prepare_edit_avatar(app, EditAvatarMode::New);
                    }

                    ui.add_space(10.0);

                    if ui.add_sized([button_width, 30.0], delete_button).clicked() {
                        // 设置删除确认标志，实际的删除操作将在确认对话框中处理
                        app.show_delete_confirmation = true;
                    }

                    ui.add_space(10.0);
                });
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
                            if ui.button("❌ 取消").clicked() {
                                app.show_delete_confirmation = false;
                            }
                            if ui.button("✅ 确认删除").clicked() {
                                match app.delete_avatar(&avatar_name) {
                                    Ok(()) => {
                                        app.console_output.push_str(&format!(
                                            "成功删除Avatar: {}\n",
                                            avatar_name
                                        ));
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

/// 显示编辑Avatar界面（替换原显示界面）
pub fn show_edit_avatar_interface(app: &mut FretDanceApp, ui: &mut egui::Ui) {
    egui::Frame::group(ui.style())
        .rounding(6.0) // 添加圆角
        .inner_margin(egui::Margin::same(10.0)) // 增加内边距
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let is_new = app.edit_avatar_mode == EditAvatarMode::New;
                let title = if is_new {
                    "新增Avatar"
                } else {
                    "修改Avatar"
                };
                ui.heading(title);
                ui.separator();

                // Avatar名字输入框
                ui.horizontal(|ui| {
                    ui.label("名字:");
                    ui.text_edit_singleline(&mut app.edit_avatar_name);
                });

                ui.add_space(10.0);

                // 图片路径选择
                ui.horizontal(|ui| {
                    ui.label("图片:");
                    ui.text_edit_singleline(&mut app.edit_avatar_image);
                    if ui.add(egui::Button::new("📁 浏览")).clicked() {
                        // 创建文件选择对话框
                        let file_path = rfd::FileDialog::new()
                            .add_filter("图片文件", &["png", "gif"])
                            .set_directory(".")
                            .pick_file();

                        if let Some(path) = file_path {
                            app.edit_avatar_image =
                                path.file_name().unwrap().to_string_lossy().to_string();
                            app.edit_avatar_selected_image_path =
                                path.to_string_lossy().to_string(); // 保存完整路径
                            println!("Selected file: {}", app.edit_avatar_selected_image_path)
                        }
                    }
                });

                ui.add_space(10.0);

                // JSON文件路径选择
                ui.horizontal(|ui| {
                    ui.label("配置:");
                    ui.text_edit_singleline(&mut app.edit_avatar_json);
                    if ui.add(egui::Button::new("📁 浏览")).clicked() {
                        // 创建文件选择对话框
                        let file_path = rfd::FileDialog::new()
                            .add_filter("JSON文件", &["json"])
                            .set_directory(".")
                            .pick_file();

                        if let Some(path) = file_path {
                            app.edit_avatar_json =
                                path.file_name().unwrap().to_string_lossy().to_string();
                            app.edit_avatar_selected_json_path = path.to_string_lossy().to_string(); // 保存完整路径
                            println!("Selected JSON file: {}", app.edit_avatar_selected_json_path)
                        }
                    }
                });

                ui.add_space(10.0);

                // 乐器选择菜单
                ui.horizontal(|ui| {
                    ui.label("乐器:");
                    egui::ComboBox::from_id_source("instrument_select")
                        .selected_text(app.edit_avatar_instrument.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.edit_avatar_instrument,
                                InstrumentType::FingerStyleGuitar,
                                "finger_style_guitar",
                            );
                            ui.selectable_value(
                                &mut app.edit_avatar_instrument,
                                InstrumentType::Bass,
                                "bass",
                            );
                            ui.selectable_value(
                                &mut app.edit_avatar_instrument,
                                InstrumentType::ElectricGuitar,
                                "electric_guitar",
                            );
                        });
                });

                ui.add_space(20.0);

                // 按钮行
                ui.horizontal(|ui| {
                    let button_width = (ui.available_width() - 10.0) / 2.0; // 计算每个按钮的宽度，留出间距

                    if ui
                        .add_sized([button_width, 30.0], egui::Button::new("❌ 取消"))
                        .clicked()
                    {
                        app.show_edit_avatar_dialog = false;
                    }

                    let save_button = egui::Button::new("💾 保存")
                        .fill(egui::Color32::from_rgb(138, 43, 226)) // 主色调
                        .rounding(4.0); // 圆角

                    if ui.add_sized([button_width, 30.0], save_button).clicked() {
                        // 验证JSON文件
                        match validate_json_file(app) {
                            Ok(()) => {
                                match app.save_avatar() {
                                    Ok(()) => {
                                        let action = if is_new { "新增" } else { "修改" };
                                        app.console_output.push_str(&format!(
                                            "成功{}Avatar: {}\n",
                                            action, app.edit_avatar_name
                                        ));
                                        app.show_edit_avatar_dialog = false;

                                        // 重新加载avatar选项
                                        app.load_avatar_options();
                                    }
                                    Err(e) => {
                                        app.console_output.push_str(&format!(
                                            "{}Avatar失败: {}\n",
                                            if is_new { "新增" } else { "修改" },
                                            e
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                app.console_output
                                    .push_str(&format!("JSON文件验证失败: {}\n", e));
                            }
                        }
                    }
                });
            });
        });
}

/// 准备编辑Avatar数据
fn prepare_edit_avatar(app: &mut FretDanceApp, mode: EditAvatarMode) {
    match mode {
        EditAvatarMode::New => {
            // 新增模式 - 清空编辑字段
            app.edit_avatar_name = String::new();
            app.edit_avatar_image = String::new();
            app.edit_avatar_selected_image_path = String::new();
            app.edit_avatar_json = String::new();
            app.edit_avatar_selected_json_path = String::new();
            app.edit_avatar_instrument = InstrumentType::FingerStyleGuitar;
        }
        EditAvatarMode::Edit => {
            // 修改模式 - 填充当前Avatar信息
            if let Some(ref avatar_info) = app.current_avatar_info {
                app.edit_avatar_name = avatar_info.name.clone();
                app.edit_avatar_image = avatar_info.image.clone();
                app.edit_avatar_selected_image_path = String::new();
                app.edit_avatar_json = String::new();
                app.edit_avatar_selected_json_path = String::new();
                app.edit_avatar_instrument = InstrumentType::from_str(&avatar_info.instrument);
            }
        }
    }
    app.edit_avatar_mode = mode;
    app.show_edit_avatar_dialog = true;
}

/// 显示Avatar图片
fn show_avatar_image(ui: &mut egui::Ui, image_path: &str) {
    // 检查文件是否存在
    if std::path::Path::new(image_path).exists() {
        // 尝试直接从文件读取字节并显示
        match std::fs::read(image_path) {
            Ok(image_data) => {
                let max_size = egui::Vec2::new(350.0, 300.0);
                let image_widget =
                    egui::Image::from_bytes(format!("bytes://{}", image_path), image_data)
                        .fit_to_exact_size(max_size);
                ui.add(image_widget);
            }
            Err(e) => {
                println!("Failed to read image file {}: {}", image_path, e);
                show_placeholder_image(ui);
            }
        }
    } else {
        println!("Image file does not exist: {}", image_path);
        // 文件不存在，显示占位符
        show_placeholder_image(ui);
    }
}

/// 显示占位符图像
fn show_placeholder_image(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(egui::Color32::from_gray(30))
        .inner_margin(10.0)
        .rounding(8.0) // 更大的圆角
        .show(ui, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(egui::Vec2::new(150.0, 150.0), egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(8.0),
                egui::Color32::from_gray(50),
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "🎵 无图片",
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );
        });
}

/// 验证选中的JSON文件是否符合要求
fn validate_json_file(app: &FretDanceApp) -> Result<(), String> {
    // 检查是否选择了JSON文件
    if app.edit_avatar_json.is_empty() || app.edit_avatar_selected_json_path.is_empty() {
        return Err("请选择一个JSON配置文件".to_string());
    }

    // 检查文件是否存在
    if !std::path::Path::new(&app.edit_avatar_selected_json_path).exists() {
        return Err("选择的JSON文件不存在".to_string());
    }

    // 定义参考文件
    let electric_guitar_reference = "asset/controller_infos/Mavuika_E.json";
    let other_instruments_reference = "asset/controller_infos/神里绫华-花时来信.json";

    // 检查参考文件是否存在
    if !std::path::Path::new(electric_guitar_reference).exists() {
        return Err("电吉他参考文件不存在".to_string());
    }

    if !std::path::Path::new(other_instruments_reference).exists() {
        return Err("其他乐器参考文件不存在".to_string());
    }

    // 根据选择的乐器类型进行验证
    match app.edit_avatar_instrument {
        InstrumentType::ElectricGuitar => {
            // 对于电吉他，检查是否与Mavuika_E.json结构相同
            match compare_json_structure(
                &app.edit_avatar_selected_json_path,
                electric_guitar_reference,
            ) {
                Ok(same_structure) => {
                    if same_structure {
                        Ok(())
                    } else {
                        // 检查是否与其他参考文件结构相同
                        match compare_json_structure(
                            &app.edit_avatar_selected_json_path,
                            other_instruments_reference,
                        ) {
                            Ok(same) => {
                                if same {
                                    Err("选择的JSON文件与电吉他参考文件结构不匹配，但与指弹吉他/贝斯参考文件结构匹配，请检查乐器类型选择是否正确".to_string())
                                } else {
                                    Err("选择的JSON文件与任何参考文件结构都不匹配".to_string())
                                }
                            }
                            Err(_) => Err("选择的JSON文件与任何参考文件结构都不匹配".to_string()),
                        }
                    }
                }
                Err(e) => Err(format!("比较JSON文件结构时出错: {}", e)),
            }
        }
        InstrumentType::FingerStyleGuitar | InstrumentType::Bass => {
            // 对于指弹吉他和贝斯，检查是否与神里绫华-花时来信.json结构相同
            match compare_json_structure(
                &app.edit_avatar_selected_json_path,
                other_instruments_reference,
            ) {
                Ok(same_structure) => {
                    if same_structure {
                        Ok(())
                    } else {
                        // 检查是否与电吉他参考文件结构相同
                        match compare_json_structure(
                            &app.edit_avatar_selected_json_path,
                            electric_guitar_reference,
                        ) {
                            Ok(same) => {
                                if same {
                                    Err("选择的JSON文件与指弹吉他/贝斯参考文件结构不匹配，但与电吉他参考文件结构匹配，请检查乐器类型选择是否正确".to_string())
                                } else {
                                    Err("选择的JSON文件与任何参考文件结构都不匹配".to_string())
                                }
                            }
                            Err(_) => Err("选择的JSON文件与任何参考文件结构都不匹配".to_string()),
                        }
                    }
                }
                Err(e) => Err(format!("比较JSON文件结构时出错: {}", e)),
            }
        }
    }
}
