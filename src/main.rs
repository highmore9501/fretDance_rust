use crate::ui::app::FretDanceApp;
use eframe::NativeOptions;
use fret_dance_rust::*;

fn main() {
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([800.0, 600.0]), // 设置最小窗口大小
        ..Default::default()
    };
    eframe::run_native(
        "Fret Dance",
        options,
        Box::new(|cc| Ok(Box::new(FretDanceApp::new(cc)))),
    )
    .expect("Failed to start eframe application");
}
