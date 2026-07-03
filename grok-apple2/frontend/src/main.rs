mod app;
mod audio;
mod gui;
mod input;
mod serial;
mod video;

use app::App;
use eframe::egui;
use grok_apple2_core::settings;

const TITLE: &str = "Apple ][+";
/// Extra window height for the top toolbar, above the emulated display.
const TOOLBAR_HEIGHT: f32 = 28.0;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let disk_path = args.get(1).cloned();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITLE)
            .with_inner_size([
                (settings::DISP_WIDTH * settings::DISP_SCALE) as f32,
                (settings::DISP_HEIGHT * settings::DISP_SCALE) as f32 + TOOLBAR_HEIGHT,
            ]),
        ..Default::default()
    };

    eframe::run_native(
        TITLE,
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, disk_path)))),
    )
}
