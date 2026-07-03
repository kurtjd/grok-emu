use eframe::egui;
use grok_apple2_core::settings;

const DISP_W: usize = settings::DISP_WIDTH as usize;
const DISP_H: usize = settings::DISP_HEIGHT as usize;

/// Owns the emulator display texture and turns raw framebuffers into on-screen
/// pixels, scaled to fit while preserving the 280:192 aspect ratio.
pub struct Display {
    tex: egui::TextureHandle,
    rgba: Vec<u8>,
}

impl Display {
    pub fn new(ctx: &egui::Context) -> Self {
        let blank =
            egui::ColorImage::from_rgba_unmultiplied([DISP_W, DISP_H], &[0; DISP_W * DISP_H * 4]);
        let tex = ctx.load_texture("apple2-display", blank, egui::TextureOptions::NEAREST);
        Self {
            tex,
            rgba: Vec::with_capacity(DISP_W * DISP_H * 4),
        }
    }

    /// Upload a fresh frame (packed as `0x00RRGGBB` per pixel) to the texture.
    pub fn update(&mut self, frame: &[u32]) {
        self.rgba.clear();
        self.rgba.extend(
            frame
                .iter()
                .flat_map(|&px| [(px >> 16) as u8, (px >> 8) as u8, px as u8, 0xFF]),
        );
        let img = egui::ColorImage::from_rgba_unmultiplied([DISP_W, DISP_H], &self.rgba);
        self.tex.set(img, egui::TextureOptions::NEAREST);
    }

    /// Draw the display, letterboxed within the available area.
    pub fn draw(&self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let scale = (avail.x / DISP_W as f32).min(avail.y / DISP_H as f32);
        let size = egui::vec2(DISP_W as f32, DISP_H as f32) * scale;
        let img = egui::Image::new(egui::load::SizedTexture::new(self.tex.id(), size));
        ui.centered_and_justified(|ui| {
            ui.add(img);
        });
    }
}
