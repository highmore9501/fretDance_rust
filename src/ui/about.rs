use crate::ui::app::FretDanceApp;
use eframe::egui;

// 展示关于对话框
pub fn show_about_dialog(app: &mut FretDanceApp, ctx: &egui::Context) {
    egui::Window::new("Fret Dance")
        .collapsible(false)
        .resizable(false)
        .default_width(320.0)
        .default_height(300.0)
        .show(ctx, |ui| {
            // 设置整体居中对齐
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                
                
                
                
                // 应用描述
                ui.add(egui::Label::new(
                    egui::RichText::new("一个将MIDI信息转换为吉他动画的工具")
                        .size(14.0)
                ));
                
                ui.add_space(15.0);
                
                // 作者信息部分
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("作者:")
                            .size(14.0)
                    ));
                    ui.hyperlink_to("海口大河马", "https://www.xiaohongshu.com/user/profile/678ce9d3000000000d0091d9");
                });
                
                ui.add_space(10.0);
                
                // 社交媒体链接部分，添加一个框架使其更突出
                egui::Frame::group(ui.style())
                    .rounding(4.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new("关注我:")
                                    .size(14.0)
                                    .strong()
                            ));
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("小红书:")
                                        .size(13.0)
                                ));
                                ui.hyperlink_to("@BigHippo78", "https://www.xiaohongshu.com/user/profile/678ce9d3000000000d0091d9");
                            });
                            
                            ui.add_space(3.0);
                            
                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("抖音:")
                                        .size(13.0)
                                ));
                                ui.hyperlink_to("@BigHippo78", "https://www.douyin.com/user/MS4wLjABAAAA4Q4Sr8ZKRBcPp4VwEzcyFmP7K2pZJgV0n4p2b9b9b9b");
                            });
                            
                            ui.add_space(3.0);
                            
                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(
                                    egui::RichText::new("B站:")
                                        .size(13.0)
                                ));
                                ui.hyperlink_to("@BigHippo78", "https://space.bilibili.com/1954959?spm_id_from=333.788.0.0");
                            });
                        });
                    });
                
                ui.add_space(15.0);
                
                // 版本信息
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("版本:")
                            .size(14.0)
                    ));
                    ui.label("1.0.0");
                });
                
                ui.add_space(5.0);
                
                // 项目链接
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("项目地址:")
                            .size(14.0)
                    ));
                    ui.hyperlink_to("https://github.com/highmore9501/fretDance", "https://github.com/highmore9501/fretDance");
                });
                
                ui.add_space(15.0);
                
                // 关闭按钮
                if ui.add_sized([80.0, 30.0], egui::Button::new("关闭")).clicked() {
                    app.show_about_dialog = false;
                }
            });
        });
}