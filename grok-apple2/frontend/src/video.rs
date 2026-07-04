use crate::gui::ScreenAspect;
use eframe::egui;
use grok_apple2_core::settings;

const DISP_W: usize = settings::DISP_WIDTH as usize;
const DISP_H: usize = settings::DISP_HEIGHT as usize;

/// Integer factor by which a saved screenshot is upscaled from the native
/// 280x192 framebuffer. Nearest-neighbour at an integer factor stays
/// pixel-perfect (no blur or distortion), matching the on-screen look.
const SCREENSHOT_SCALE: u32 = 3;

/// Owns the emulator display texture and turns raw framebuffers into on-screen
/// pixels, scaled to fit while preserving the 280:192 aspect ratio.
pub struct Display {
    tex: egui::TextureHandle,
    rgba: Vec<u8>,
    // How the framebuffer is fitted to the window (shape + texture filtering).
    aspect: ScreenAspect,
}

impl Display {
    pub fn new(ctx: &egui::Context) -> Self {
        let blank =
            egui::ColorImage::from_rgba_unmultiplied([DISP_W, DISP_H], &[0; DISP_W * DISP_H * 4]);
        let tex = ctx.load_texture("apple2-display", blank, egui::TextureOptions::NEAREST);
        Self {
            tex,
            rgba: Vec::with_capacity(DISP_W * DISP_H * 4),
            aspect: ScreenAspect::default(),
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
        self.tex.set(img, self.filter());
    }

    /// Texture sampling for the current aspect: crisp nearest for square pixels,
    /// soft linear for the 4:3 stretch (more CRT-like, and hides the uneven pixel
    /// doubling that a non-integer vertical stretch would otherwise produce).
    fn filter(&self) -> egui::TextureOptions {
        match self.aspect {
            ScreenAspect::Square => egui::TextureOptions::NEAREST,
            ScreenAspect::FourThree => egui::TextureOptions::LINEAR,
        }
    }

    /// Change how the framebuffer is fitted to the window. Re-uploads the current
    /// frame with the new sampling right away so a filter change shows even while
    /// paused; the geometry updates on the next `draw`.
    pub fn set_aspect(&mut self, aspect: ScreenAspect) {
        if self.aspect == aspect {
            return;
        }
        self.aspect = aspect;
        if self.rgba.len() == DISP_W * DISP_H * 4 {
            let img = egui::ColorImage::from_rgba_unmultiplied([DISP_W, DISP_H], &self.rgba);
            self.tex.set(img, self.filter());
        }
    }

    /// Draw the display, letterboxed within the available area at the selected
    /// aspect ratio (native square pixels, or a real 4:3 monitor shape).
    pub fn draw(&self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let target = match self.aspect {
            ScreenAspect::Square => DISP_W as f32 / DISP_H as f32,
            ScreenAspect::FourThree => 4.0 / 3.0,
        };
        // Fit the largest `target`-ratio rectangle inside the available area.
        let mut w = avail.x;
        let mut h = w / target;
        if h > avail.y {
            h = avail.y;
            w = h * target;
        }
        let img = egui::Image::new(egui::load::SizedTexture::new(
            self.tex.id(),
            egui::vec2(w, h),
        ));
        ui.centered_and_justified(|ui| {
            ui.add(img);
        });
    }

    /// Save the current frame to `path` as a PNG, upscaled by an integer factor
    /// with nearest-neighbour sampling so the pixels stay crisp. Logs and returns
    /// on any encode/write error rather than crashing the emulator.
    pub fn save_png(&self, path: &std::path::Path) {
        // `rgba` holds the last uploaded frame as tightly packed RGBA bytes.
        let Some(src) = image::RgbaImage::from_raw(DISP_W as u32, DISP_H as u32, self.rgba.clone())
        else {
            eprintln!("Failed to build screenshot image from framebuffer");
            return;
        };
        let scaled = image::imageops::resize(
            &src,
            DISP_W as u32 * SCREENSHOT_SCALE,
            DISP_H as u32 * SCREENSHOT_SCALE,
            image::imageops::FilterType::Nearest,
        );
        if let Err(e) = scaled.save(path) {
            eprintln!("Failed to save screenshot to {}: {e}", path.display());
        }
    }
}
