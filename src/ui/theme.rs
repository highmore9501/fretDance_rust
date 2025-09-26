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

    // 设置暗色主题的基本颜色
    style.visuals = egui::Visuals::dark();

    // 自定义颜色 - 黑色背景和白色文字
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(0, 0, 0); // 窗口背景 - 纯黑
    style.visuals.panel_fill = egui::Color32::from_rgb(20, 20, 20); // 面板背景 - 深灰黑
    style.visuals.window_fill = egui::Color32::from_rgb(15, 15, 15); // 窗口填充 - 更深的灰黑
    style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)); // 窗口边框

    // 文字颜色设置为白色
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255)); // 主要文字 - 纯白
    style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 非交互部件文字
    style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 非活动部件文字
    style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 悬停部件文字
    style.visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 活动部件文字
    style.visuals.widgets.open.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 打开部件文字

    // 背景颜色保持深色
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30); // 非交互部件背景
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 40, 40); // 非活动部件背景
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 60); // 悬停部件背景
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 50); // 活动部件背景
    style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(50, 50, 50); // 打开部件背景

    // 滚动条颜色
    style.visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(80, 80, 80);

    // 选中项颜色
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(70, 70, 70);
    style.visuals.selection.stroke.color = egui::Color32::WHITE;

    // 分隔线颜色
    style.visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100));

    // 应用样式
    ctx.set_style(style);
}

// 设置浅色主题
fn set_light_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // 使用默认的浅色主题
    style.visuals = egui::Visuals::light();

    // 应用样式
    ctx.set_style(style);
}
