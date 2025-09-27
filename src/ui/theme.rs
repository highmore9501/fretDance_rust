use eframe::egui;

// 应用主题
pub fn apply_theme(dark_mode: bool, ctx: &egui::Context) {
    if dark_mode {
        set_dark_theme(ctx);
    } else {
        set_light_theme(ctx);
    }
}

// 设置黑色主题
fn set_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // 设置暗色主题的基本颜色 - 使用更现代的深灰色而非纯黑
    style.visuals = egui::Visuals::dark();

    // 使用更柔和的深色背景
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(30, 30, 30); // 窗口背景 - 深灰
    style.visuals.panel_fill = egui::Color32::from_rgb(25, 25, 25); // 面板背景 - 深灰
    style.visuals.window_fill = egui::Color32::from_rgb(20, 20, 20); // 窗口填充 - 更深的灰
    style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)); // 窗口边框

    // 文字颜色设置为浅灰色而非纯白，减少眼部疲劳
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(224, 224, 224)); // 主要文字 - 浅灰
    style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(224, 224, 224); // 非交互部件文字
    style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(224, 224, 224); // 非活动部件文字
    style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 悬停部件文字
    style.visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 活动部件文字
    style.visuals.widgets.open.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 打开部件文字

    // 背景颜色使用更现代的配色方案
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(40, 40, 40); // 非交互部件背景
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 55, 55); // 非活动部件背景
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 70, 70); // 悬停部件背景
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(85, 85, 85); // 活动部件背景
    style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(85, 85, 85); // 打开部件背景

    // 滚动条颜色
    style.visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(70, 70, 70);

    // 选中项颜色 - 使用主色调
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(138, 43, 226); // 蓝紫色
    style.visuals.selection.stroke.color = egui::Color32::WHITE;

    // 分隔线颜色
    style.visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80));

    // 应用样式
    ctx.set_style(style);
}

// 设置浅色主题
fn set_light_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // 使用默认的浅色主题
    style.visuals = egui::Visuals::light();

    // 调整选中项颜色以匹配主色调
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(138, 43, 226); // 蓝紫色
    style.visuals.selection.stroke.color = egui::Color32::WHITE;

    // 应用样式
    ctx.set_style(style);
}
