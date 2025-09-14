use eframe::egui;

mod ui;
mod backend;

use ui::LaoApp;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("LAO Orchestrator"),
        ..Default::default()
    };

    eframe::run_native(
        "LAO Orchestrator",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            Ok(Box::new(LaoApp::new(cc)))
        }),
    )
}