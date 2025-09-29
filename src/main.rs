use crate::ui::app::FretDanceApp;
use eframe::NativeOptions;
use fret_dance_rust::*;

fn main() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Fret Dance",
        native_options,
        Box::new(|cc| Ok(Box::new(FretDanceApp::new(cc)))),
    )
    .expect("Failed to start eframe application");
}
