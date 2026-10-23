mod app;
mod code_editor;
mod theme;
mod viewport;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("四次元 Yonjigen"),
        ..Default::default()
    };
    eframe::run_native(
        "Yonjigen",
        options,
        Box::new(|cc| Ok(Box::new(app::YonjigenApp::new(cc)))),
    )
}
