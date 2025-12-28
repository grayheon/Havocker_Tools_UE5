use eframe::egui;
use image::imageops::FilterType;
use image::{ImageBuffer, Rgba};
use std::fs;
use std::path::PathBuf;

pub struct TxdViewerApp {
    zoom: f32,
    file_path: Option<PathBuf>,
    image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    offset: usize,
    x_shift: i32,
    y_shift: i32,
    x_jump_start: usize,
    y_jump: i32,
}

impl Default for TxdViewerApp {
    fn default() -> Self {
        Self {
            zoom: 0.0,
            file_path: None,
            image: None,
            offset: 60,
            x_shift: 0,
            y_shift: 0,
            x_jump_start: 0,
            y_jump: 0,
        }
    }
}

impl eframe::App for TxdViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("TXD Texture Debug Viewer");

            if let Some(path) = ui
                .button("Load TXD file")
                .clicked()
                .then(|| rfd::FileDialog::new().pick_file())
                .flatten()
            {
                self.file_path = Some(path.clone());
                self.image = fs::read(&path).ok().and_then(|data| {
                    extract_image(
                        &data,
                        &path,
                        self.offset,
                        self.x_shift,
                        self.y_shift,
                        self.x_jump_start,
                        self.y_jump,
                    )
                });
            }

            ui.add(egui::Slider::new(&mut self.offset, 0..=10_000).text("Offset"));
            ui.add(egui::Slider::new(&mut self.x_shift, -50..=50).text("X-Displacement"));
            ui.add(egui::Slider::new(&mut self.y_shift, -50..=50).text("Y-Displacement"));
            ui.add(egui::Slider::new(&mut self.x_jump_start, 0..=49).text("Column offset from X"));
            ui.add(egui::Slider::new(&mut self.y_jump, -50..=50).text("Y-Displacement (from X)"));

            if let Some(path) = &self.file_path {
                ui.label(format!("Loaded: {}", path.display()));
            }

            if let Some(path) = ui
                .button("Regenerate image")
                .clicked()
                .then(|| self.file_path.clone())
                .flatten()
            {
                self.image = fs::read(&path).ok().and_then(|data| {
                    extract_image(
                        &data,
                        &path,
                        self.offset,
                        self.x_shift,
                        self.y_shift,
                        self.x_jump_start,
                        self.y_jump,
                    )
                });
            }

            if let Some(img) = &self.image {
                ui.add(egui::Slider::new(&mut self.zoom, 1.0f32..=20.0f32).text("Zoom"));
                let zoomed_width = (img.width() as f32 * self.zoom) as usize;
                let zoomed_height = (img.height() as f32 * self.zoom) as usize;
                let resized = image::imageops::resize(
                    img,
                    zoomed_width as u32,
                    zoomed_height as u32,
                    FilterType::Nearest,
                );
                let tex = egui::ColorImage::from_rgba_unmultiplied(
                    [zoomed_width, zoomed_height],
                    resized.as_raw(),
                );
                let tex_handle = ctx.load_texture("image", tex, egui::TextureOptions::default());

                ui.add_sized(
                    egui::vec2(zoomed_width as f32, zoomed_height as f32),
                    egui::Image::new(&tex_handle),
                );
            }
        });
    }
}

fn extract_image(
    data: &[u8],
    path: &std::path::Path,
    offset: usize,
    x_shift: i32,
    y_shift: i32,
    x_jump_start: usize,
    y_jump: i32,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let (width, height) = {
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            if filename.to_ascii_lowercase().starts_with("m") {
                (32, 32)
            } else {
                (50, 50)
            }
        } else {
            (50, 50)
        }
    };
    const PIXEL_SIZE: usize = 4;
    let expected = width * height * PIXEL_SIZE;
    if offset + expected > data.len() {
        return None;
    }
    let raw = &data[offset..offset + expected];

    let mut img = ImageBuffer::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let mut px_y = ((y as i32 + y_shift) + height as i32) % height as i32;
            if x >= x_jump_start {
                px_y = ((px_y + y_jump) + height as i32) % height as i32;
            }
            let px_x = ((x as i32 + x_shift) + width as i32) % width as i32;
            let idx = (y * width + x) * PIXEL_SIZE;
            if idx + 4 <= raw.len() {
                let pixel = Rgba([raw[idx], raw[idx + 1], raw[idx + 2], raw[idx + 3]]);
                img.put_pixel(px_x as u32, px_y as u32, pixel);
            }
        }
    }
    Some(img)
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "TXD Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(TxdViewerApp::default()))),
    )
    .expect("TODO: panic message");
}
